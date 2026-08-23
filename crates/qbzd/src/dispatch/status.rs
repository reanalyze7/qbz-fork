use crate::{cli, paths};

pub async fn status(host: Option<String>, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::status::status(host, json, &roots).await
}

pub async fn ping(host: Option<String>, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::status::ping(host, json, &roots).await
}

pub async fn now(host: Option<String>, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::transport::now(host, json, &roots).await
}

pub async fn watch(host: Option<String>, raw: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::watch::watch(host, raw, &roots).await
}
