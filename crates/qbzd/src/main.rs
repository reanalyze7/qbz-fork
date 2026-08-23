use clap::Parser;

mod adapter;
mod api;
mod cli;
mod cli_args;
mod config;
mod daemon;
mod dispatch;
mod lock;
mod login;
mod mpris;
mod paths;
mod roots;
mod scrobble_engine;
mod state;
mod tui;

pub const API_VERSION: u32 = 1; // 02-cli-and-api.md §1.6

#[tokio::main]
async fn main() {
    let cli = cli_args::Cli::parse();
    let code = dispatch::dispatch(cli).await;
    std::process::exit(code);
}
