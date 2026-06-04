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
use prunify::scheme::SchemeLoader;

fn main() -> PrunifyResult<()> {
    // Register signal handler for SIGINT forwarding to child processes.
    // This must be called before any child process is spawned.
    register_handler();

    let cli = Cli::parse();

    // Join positional args into command string
    let command = cli.command.join(" ");

    // Handle empty command (shouldn't happen with clap required=true, but safeguard)
    if command.trim().is_empty() {
        eprintln!("prunify: empty command — nothing to proxy");
        std::process::exit(1);
    }

    // 1. Recursion guard
    if RecursionGuard::is_recursive(&command) {
        eprintln!("prunify: recursion detected — bypassing proxy");
        return Ok(());
    }

    // 2. TTY passthrough — interactive commands bypass proxy
    // Spawn directly with raw args (no sh -c) to preserve quoted arguments.
    if TtyDetector::should_passthrough(&cli.command[0]) {
        let status = std::process::Command::new(&cli.command[0])
            .args(&cli.command[1..])
            .status()?;
        std::process::exit(status.code().unwrap_or(1));
    }

    // 3. Load config
    let config_path = std::path::Path::new(".prunify.yaml");
    let config = ConfigLoader::load(if config_path.exists() {
        Some(config_path)
    } else {
        None
    })?;

    // Merge CLI overrides onto config (if any CLI flag was provided)
    let config = if cli.scheme_dir.is_some() || cli.verbose || cli.strict {
        PrunifyConfig {
            scheme_dir: cli.scheme_dir.map(PathBuf::from),
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

    // 4. Load schemes
    let default_dir = default_prunify_dir().join("schemes");
    let loader = SchemeLoader::new(default_dir.clone());
    let schemes = loader.load(&config)?;

    // 5. Populate trie (cached to ~/.prunify/trie.json for fast startup)
    let project_dir: PathBuf = config
        .scheme_dir
        .clone()
        .unwrap_or_else(|| default_prunify_dir().join("schemes"));
    let trie_path = default_prunify_dir().join("trie.json");
    let rebuild_trie = cli.rebuild_trie
        || CommandTrie::is_trie_stale(&trie_path, &[&default_dir, &project_dir]);

    let trie = if rebuild_trie {
        let mut t = CommandTrie::new();
        for cmd in schemes.keys() {
            t.insert(cmd, cmd);
        }
        if let Err(e) = t.save_to_file(&trie_path) {
            eprintln!("prunify: warning: failed to cache trie: {e}");
        }
        t
    } else {
        match CommandTrie::load_from_file(&trie_path) {
            Ok(t) => t,
            Err(_e) => {
                // Corrupted or missing cache — rebuild from schemes
                let mut t = CommandTrie::new();
                for cmd in schemes.keys() {
                    t.insert(cmd, cmd);
                }
                t
            }
        }
    };

    // 6. Execute command — pass raw args directly, no join/split round-trip
    let result = CommandExecutor::execute(&cli.command)?;

    // 7. Dispatch (mode routing + pruning)
    // Use lossy string conversion for scheme matching — dispatcher operates on text.
    let stdout_str = String::from_utf8_lossy(&result.stdout);
    let dispatcher = Dispatcher::new(trie, schemes);
    let (pruned, mode) = dispatcher.dispatch(&command, &stdout_str)?;

    // 8. Mark output
    let tokens = match &mode {
        DispatchMode::PrefixMatch(n) => *n,
        _ => 0,
    };
    let output = OutputMarker::mark_pruned(&pruned, &mode, tokens, cli.no_mark);

    // 9. Print
    // For --no-mark passthrough, write raw bytes to preserve binary data.
    // For all other modes, use the string output (pruned/marked as appropriate).
    if cli.no_mark && mode == DispatchMode::Passthrough {
        std::io::stdout().write_all(&result.stdout)?;
    } else {
        print!("{}", output);
    }
    if !result.stderr.is_empty() {
        eprint!("{}", result.stderr);
    }

    // 10. Exit with command's exit code
    std::process::exit(result.exit_code);
}
