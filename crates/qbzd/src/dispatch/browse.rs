use crate::cli_args::RecoCmd;
use crate::{cli, paths};

pub async fn search(host: Option<String>, query: String, kind: String, limit: u32, offset: u32, ids: bool, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::search::search(host, query, kind, limit, offset, ids, json, &roots).await
}

pub async fn album(host: Option<String>, id: String, suggest: bool, ids: bool, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::browse::album(host, id, suggest, ids, json, &roots).await
}

pub async fn artist(host: Option<String>, id: u64, top: bool, albums: bool, limit: u32, ids: bool, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::browse::artist(host, id, top, albums, limit, ids, json, &roots).await
}

pub async fn similar(host: Option<String>, selector: String, limit: u32, ids: bool, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::browse::similar(host, selector, limit, ids, json, &roots).await
}

pub async fn suggest(host: Option<String>, seed: Option<String>, limit: u32, ids: bool, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::browse::suggest(host, seed, limit, ids, json, &roots).await
}

#[allow(clippy::too_many_arguments)]
pub async fn discover(host: Option<String>, section: Option<String>, genre: Option<String>, tag: Option<String>, release_type: Option<String>, kind: Option<String>, limit: u32, ids: bool, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::discover::discover(host, section, genre, tag, release_type, kind, limit, ids, json, &roots).await
}

pub async fn reco(host: Option<String>, cmd: RecoCmd) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    match cmd {
        RecoCmd::Playlist { id, limit, ids, json } => {
            cli::reco::playlist(host, id, limit, ids, json, &roots).await
        }
    }
}
