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

    /// Force rebuild of command trie cache (ignores cached .prunify/trie.json)
    #[arg(long)]
    pub rebuild_trie: bool,
}
