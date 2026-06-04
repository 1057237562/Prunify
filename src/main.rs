use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;

use prunify::cli::Cli;
use prunify::config::{default_prunify_dir, ConfigLoader, PrunifyConfig};
use prunify::engine::CommandTrie;
use prunify::error::PrunifyResult;
use prunify::proxy::{
    CommandExecutor, DispatchMode, Dispatcher, OutputMarker, RecursionGuard, TtyDetector,
    register_handler,
};
use prunify::scheme::{Scheme, SchemeLoader};

fn main() -> PrunifyResult<()> {
    register_handler();

    let cli = Cli::parse();

    // If --list-commands, show all known schemes and exit
    if cli.list_commands {
        return list_known_commands(&cli);
    }

    // If no command, enter interactive bash mode
    let command = match cli.command {
        Some(ref cmd) if !cmd.is_empty() => cmd.clone(),
        _ => return run_interactive(cli),
    };

    // Load config, schemes, and tries
    let (config, project_schemes, fallback_schemes, local_trie, global_trie) = load_setup(&cli)?;
    let dispatcher = Dispatcher::new(local_trie, global_trie, project_schemes, fallback_schemes);

    // Execute single command through the pipeline
    let exit_code = execute_and_print(&command, &config, &dispatcher, &cli)?;
    std::process::exit(exit_code);
}

/// Helper: build a `CommandTrie` from the keys of a scheme map.
fn build_trie(schemes: &HashMap<String, Scheme>) -> CommandTrie {
    let mut t = CommandTrie::new();
    for cmd in schemes.keys() {
        t.insert(cmd, cmd);
    }
    t
}

/// Load config from `.prunify.yaml`, merge CLI overrides, load schemes,
/// and build separate tries for local and global schemes.
///
/// Returns (config, project_schemes, fallback_schemes, local_trie, global_trie).
fn load_setup(
    cli: &Cli,
) -> PrunifyResult<(
    PrunifyConfig,
    HashMap<String, Scheme>,
    HashMap<String, Scheme>,
    CommandTrie,
    CommandTrie,
)> {
    let config_path = std::path::Path::new(".prunify.yaml");
    let config = ConfigLoader::load(if config_path.exists() {
        Some(config_path)
    } else {
        None
    })?;

    let config = if cli.scheme_dir.is_some() || cli.verbose || cli.strict {
        PrunifyConfig {
            scheme_dir: cli.scheme_dir.clone().map(PathBuf::from),
            verbose: if cli.verbose {
                Some(true)
            } else {
                config.verbose
            },
            no_color: config.no_color,
            strict: if cli.strict {
                Some(true)
            } else {
                config.strict
            },
        }
    } else {
        config
    };

    let fallback_dir = default_prunify_dir().join("schemes");
    let loader = SchemeLoader::new(fallback_dir.clone());
    let (project_schemes, fallback_schemes) = loader.load(&config)?;

    // Build separate tries — local trie only has project commands,
    // global trie only has fallback commands. This ensures prefix
    // matching never picks a deeper global match over a local one.
    let local_trie = build_trie(&project_schemes);
    let global_trie = build_trie(&fallback_schemes);

    Ok((config, project_schemes, fallback_schemes, local_trie, global_trie))
}

/// List all known commands grouped by source (project-local vs global fallback).
fn list_known_commands(cli: &Cli) -> PrunifyResult<()> {
    let config_path = std::path::Path::new(".prunify.yaml");
    let config = ConfigLoader::load(if config_path.exists() {
        Some(config_path)
    } else {
        None
    })?;

    let config = if cli.scheme_dir.is_some() || cli.verbose || cli.strict {
        PrunifyConfig {
            scheme_dir: cli.scheme_dir.clone().map(PathBuf::from),
            verbose: if cli.verbose { Some(true) } else { config.verbose },
            no_color: config.no_color,
            strict: if cli.strict { Some(true) } else { config.strict },
        }
    } else {
        config
    };

    let fallback_dir = default_prunify_dir().join("schemes");
    let loader = SchemeLoader::new(fallback_dir.clone());
    let (project_schemes, fallback_schemes) = loader.load(&config)?;

    let project_dir: PathBuf = config
        .scheme_dir
        .clone()
        .unwrap_or_else(|| {
            let local = PathBuf::from(".prunify").join("schemes");
            if local.exists() {
                local
            } else {
                fallback_dir.clone()
            }
        });

    // Build sorted command lists
    let mut project_cmds: Vec<&String> = project_schemes.keys().collect();
    project_cmds.sort();
    let mut fallback_cmds: Vec<&String> = fallback_schemes.keys().collect();
    fallback_cmds.sort();

    println!("Known commands:");
    println!();

    // Always show both sections so the user knows both directories are checked.
    // Use "(none)" when a section has no commands.

    println!("  Local ({}):", project_dir.display());
    if project_cmds.is_empty() {
        println!("    (none)");
    } else {
        for cmd in &project_cmds {
            println!("    {cmd}");
        }
    }
    println!();

    println!("  Global ({}):", fallback_dir.display());
    if fallback_cmds.is_empty() {
        println!("    (none)");
    } else {
        for cmd in &fallback_cmds {
            println!("    {cmd}");
        }
    }
    println!();

    Ok(())
}

/// Execute a command and return its (possibly pruned) output along with the exit code.
///
/// Returns `(formatted_stdout, stderr, exit_code)`. The output is fully formatted
/// (scheme-pruned + marked) but NOT printed — the caller handles output.
fn execute_and_format(
    args: &[String],
    _config: &PrunifyConfig,
    dispatcher: &Dispatcher,
    cli: &Cli,
) -> PrunifyResult<(String, String, i32)> {
    let command_str = args.join(" ");

    // Recursion guard
    if RecursionGuard::is_recursive(&command_str) {
        return Ok((String::new(), "prunify: recursion detected — bypassing proxy\n".to_string(), 0));
    }

    // TTY passthrough — interactive commands bypass proxy
    if TtyDetector::should_passthrough(&args[0]) {
        let status = std::process::Command::new(&args[0])
            .args(&args[1..])
            .status()?;
        return Ok((String::new(), String::new(), status.code().unwrap_or(1)));
    }

    // Execute
    let result = CommandExecutor::execute(args)?;

    // Dispatch (mode routing + pruning)
    let stdout_str = String::from_utf8_lossy(&result.stdout);
    let (pruned, mode) = dispatcher.dispatch(&command_str, &stdout_str)?;

    let pruned_stderr = dispatcher.dispatch(&command_str, &result.stderr)?.0;

    // Mark output
    let tokens = match &mode {
        DispatchMode::PrefixMatch(n) => *n,
        _ => 0,
    };
    let output = OutputMarker::mark_pruned(&pruned, &mode, tokens, cli.no_mark);

    // In passthrough+no-mark mode, use the raw bytes to preserve binary data.
    // Convert lossily here since we're returning a String anyway.
    let formatted = if cli.no_mark && mode == DispatchMode::Passthrough {
        String::from_utf8_lossy(&result.stdout).to_string()
    } else {
        output
    };

    Ok((formatted, pruned_stderr, result.exit_code))
}

/// Execute a command and write its output directly to stdout/stderr.
/// Returns the exit code.
fn execute_and_print(
    args: &[String],
    config: &PrunifyConfig,
    dispatcher: &Dispatcher,
    cli: &Cli,
) -> PrunifyResult<i32> {
    let (stdout, stderr, exit_code) = execute_and_format(args, config, dispatcher, cli)?;
    print!("{}", stdout);
    if !stderr.is_empty() {
        eprint!("{}", stderr);
    }
    Ok(exit_code)
}

/// Interactive bash mode: enter a REPL where each command is executed
/// and processed through the prunify pipeline.
fn run_interactive(cli: Cli) -> PrunifyResult<()> {
    let (config, project_schemes, fallback_schemes, local_trie, global_trie) = load_setup(&cli)?;
    let dispatcher = Dispatcher::new(local_trie, global_trie, project_schemes, fallback_schemes);

    let stdin = std::io::stdin();
    loop {
        print!("prunify $ ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        if stdin.read_line(&mut input)? == 0 {
            // EOF (Ctrl+D)
            println!();
            break;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed == "exit" {
            break;
        }

        // Split into args by whitespace
        let args: Vec<String> = trimmed.split_whitespace().map(String::from).collect();
        if args.is_empty() {
            continue;
        }

        // Run through the prunify pipeline, but don't exit on failure
        let result = execute_and_format(&args, &config, &dispatcher, &cli);
        match result {
            Ok((stdout, stderr, _exit_code)) => {
                let has_output = !stdout.is_empty() || !stderr.is_empty();

                if !stdout.is_empty() {
                    print!("{}", stdout);
                }
                if !stderr.is_empty() {
                    eprint!("{}", stderr);
                }

                // Ensure the next prompt starts on a fresh line.
                // If neither stdout nor stderr ended with a newline,
                // or if there was no output at all, insert one.
                if !has_output || (!stdout.ends_with('\n') && !stderr.ends_with('\n')) {
                    println!();
                }
            }
            Err(e) => {
                eprintln!("prunify: error: {e}");
            }
        }
    }

    Ok(())
}
