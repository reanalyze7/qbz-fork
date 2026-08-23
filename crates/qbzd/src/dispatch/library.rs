use crate::cli_args::{FavCmd, PlaylistCmd};
use crate::{cli, paths};

pub async fn fav(host: Option<String>, cmd: FavCmd) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    match cmd {
        FavCmd::List { kind, ids, json } => cli::fav::list(host, kind, ids, json, &roots).await,
        FavCmd::Add { fav_type, id, current } => {
            cli::fav::add(host, fav_type, id, current, &roots).await
        }
        FavCmd::Remove { fav_type, id } => cli::fav::remove(host, fav_type, id, &roots).await,
    }
}

pub async fn playlist(host: Option<String>, cmd: PlaylistCmd) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    match cmd {
        PlaylistCmd::List { json } => cli::playlist::list(host, json, &roots).await,
        PlaylistCmd::Show { id, ids, json } => {
            cli::playlist::show(host, id, ids, json, &roots).await
        }
        PlaylistCmd::Create { name, desc, public } => {
            cli::playlist::create(host, name, desc, public, &roots).await
        }
        PlaylistCmd::Edit { id, name, desc, public, private } => {
            cli::playlist::edit(host, id, name, desc, public, private, &roots).await
        }
        PlaylistCmd::Rm { id, yes } => cli::playlist::rm(host, id, yes, &roots).await,
        PlaylistCmd::Add { id, track_ids } => {
            cli::playlist::add(host, id, track_ids, &roots).await
        }
        PlaylistCmd::Remove { id, track_ids } => {
            cli::playlist::remove(host, id, track_ids, &roots).await
        }
    }
}
