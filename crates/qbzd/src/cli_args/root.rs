use clap::Parser;

use super::cmd::Cmd;

#[derive(Parser)]
#[command(name = "qbzd", version, arg_required_else_help = true,
          about = "Qoqobuz headless Qobuz playback daemon")]
pub struct Cli {
    /// Target daemon (default 127.0.0.1:8182; env QBZD_HOST)
    #[arg(long, global = true)]
    pub host: Option<String>,
    #[arg(short, long, global = true)]
    pub quiet: bool,
    #[command(subcommand)]
    pub cmd: Cmd,
}
