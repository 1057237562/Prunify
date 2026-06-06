use clap::Parser;

/// Proxy and prune bash command output
#[derive(Parser)]
#[command(
    name = "prunify",
    version = "0.1.0",
    about = "Proxy and prune bash command output"
)]
pub struct Cli {
    /// The command to proxy (everything after prunify's own flags).
    /// If omitted, prunify enters interactive bash mode.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Option<Vec<String>>,

    /// Shell wrapper mode: pass the full command as a single string.
    /// Used when prunify is configured as the default shell in OpenCode
    /// (e.g., `prunify -c "git status"`). The string is split on whitespace
    /// and routed through the same pipeline as positional commands.
    #[arg(short = 'c', long)]
    pub command_string: Option<String>,

    /// Custom path to scheme files
    #[arg(long)]
    pub scheme_dir: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable [PRUNED] and [UNKNOWN COMMAND] marks (which prompt use of `prunify skill`)
    #[arg(long)]
    pub no_mark: bool,

    /// Reject unknown commands with error instead of passthrough
    #[arg(long)]
    pub strict: bool,

    /// Change to this directory before executing the command.
    /// When used with `-c`, the directory change happens before the
    /// command string is split and executed.
    #[arg(short = 'C', long, value_name = "DIR")]
    pub chdir: Option<String>,

    /// Force rebuild of cached tries under ~/.prunify/
    #[arg(long)]
    pub rebuild_trie: bool,

    /// List all known commands from project and global schemes
    #[arg(short, long)]
    pub list_commands: bool,
}
