use std::io::Write;
use std::path::PathBuf;

use clap::Parser;

use prunifier::cli::Cli;
use prunifier::config::{ConfigLoader, PrunifierConfig};
use prunifier::engine::CommandTrie;
use prunifier::error::PrunifierResult;
use prunifier::proxy::{
    CommandExecutor, DispatchMode, Dispatcher, OutputMarker, RecursionGuard, TtyDetector,
    register_handler,
};
use prunifier::scheme::SchemeLoader;

fn main() -> PrunifierResult<()> {
    // Register signal handler for SIGINT forwarding to child processes.
    // This must be called before any child process is spawned.
    register_handler();

    let cli = Cli::parse();

    // Join positional args into command string
    let command = cli.command.join(" ");

    // Handle empty command (shouldn't happen with clap required=true, but safeguard)
    if command.trim().is_empty() {
        eprintln!("prunifier: empty command — nothing to proxy");
        std::process::exit(1);
    }

    // 1. Recursion guard
    if RecursionGuard::is_recursive(&command) {
        eprintln!("prunifier: recursion detected — bypassing proxy");
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
    let config_path = std::path::Path::new(".prunifier.yaml");
    let config = ConfigLoader::load(if config_path.exists() {
        Some(config_path)
    } else {
        None
    })?;

    // Merge CLI overrides onto config (if any CLI flag was provided)
    let config = if cli.scheme_dir.is_some() || cli.verbose || cli.strict {
        PrunifierConfig {
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
    let default_dir = std::path::PathBuf::from(".prunifier/schemes/");
    let loader = SchemeLoader::new(default_dir);
    let schemes = loader.load(&config)?;

    // 5. Populate trie
    let mut trie = CommandTrie::new();
    for cmd in schemes.keys() {
        trie.insert(cmd, cmd);
    }

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
