use crate::cli;
use crate::cli_args::Cli;
use crate::API_VERSION;

pub fn version(json: bool) -> i32 {
    if json {
        println!("{{\"version\":\"{}\",\"api_version\":{}}}",
                 env!("CARGO_PKG_VERSION"), API_VERSION);
    } else {
        println!("qbzd {} (api v{})", env!("CARGO_PKG_VERSION"), API_VERSION);
    }
    0
}

pub fn service(init: Option<String>, user: Option<String>, bin: Option<String>, system: bool) -> i32 {
    cli::service::service(init, user, bin, system)
}

pub fn completions(shell: clap_complete::Shell) -> i32 {
    use clap::CommandFactory;
    clap_complete::generate(shell, &mut Cli::command(), "qbzd", &mut std::io::stdout());
    0
}
