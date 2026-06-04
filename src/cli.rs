use clap::Parser;

/// Proxy and prune bash command output
#[derive(Parser)]
#[command(
    name = "prunifier",
    version = "0.1.0",
    about = "Proxy and prune bash command output"
)]
pub struct Cli {
    /// The command to proxy (everything after prunifier's own flags)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
    pub command: Vec<String>,

    /// Custom path to scheme files
    #[arg(long)]
    pub scheme_dir: Option<String>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable [PRUNED] and [UNKNOWN COMMAND] marks
    #[arg(long)]
    pub no_mark: bool,

    /// Reject unknown commands with error instead of passthrough
    #[arg(long)]
    pub strict: bool,
}
