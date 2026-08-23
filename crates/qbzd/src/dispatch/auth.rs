use crate::{cli, config, daemon, login, paths, roots::login_roots};

pub async fn run() -> i32 {
    // Phase 1: resolve the config root and load qbzd.toml. The config's
    // `data_root` (a container override) can redirect the data/cache
    // roots, so resolve those in phase 2 once it is known.
    let bootstrap = paths::ProfileRoots::resolve(None, None);
    let cfg_path = bootstrap.config.join("qbzd.toml");
    let (cfg, warns) = match config::QbzdConfig::load(&cfg_path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            eprintln!("  → fix or remove the config:  {}", cfg_path.display());
            std::process::exit(1);
        }
    };
    // Phase 2: honor an explicit `data_root` container override.
    let data_root = cfg.data_root.clone();
    let roots = paths::ProfileRoots::resolve(
        None,
        data_root.as_deref().map(std::path::Path::new),
    );
    match daemon::run(roots, cfg, warns).await {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

pub async fn login(callback_host: Option<String>, paste: bool, token: Option<String>) -> i32 {
    let roots = login_roots();
    let result = if let Some(tok) = token {
        login::login_with_token_arg(&roots, &tok).await
    } else if paste {
        login::login_paste(&roots).await
    } else {
        login::login_browser(&roots, callback_host).await
    };
    match result {
        Ok(session) => {
            println!("{}", cli::copy::login_success(&session));
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

pub fn logout() -> i32 {
    let roots = login_roots();
    match login::logout(&roots) {
        Ok(daemon_nudged) => {
            println!("{}", cli::copy::logout_success(daemon_nudged));
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}
