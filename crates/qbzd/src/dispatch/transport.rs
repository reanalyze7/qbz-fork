use crate::cli_args::QueueCmd;
use crate::{cli, paths};

pub async fn shuffle(host: Option<String>, mode: Option<String>) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::mode::shuffle(host, mode, &roots).await
}

pub async fn repeat(host: Option<String>, mode: String) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::mode::repeat(host, mode, &roots).await
}

pub async fn art(host: Option<String>, save: Option<String>) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::art::art(host, save, &roots).await
}

pub fn resolve(url: String) -> i32 {
    cli::resolve::resolve(url)
}

pub async fn play(host: Option<String>, content: Option<String>) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::play::play(host, content, &roots).await
}

pub async fn pause(host: Option<String>) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::transport::pause(host, &roots).await
}

pub async fn toggle(host: Option<String>) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::transport::toggle(host, &roots).await
}

pub async fn stop(host: Option<String>) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::transport::stop(host, &roots).await
}

pub async fn next(host: Option<String>) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::transport::next(host, &roots).await
}

pub async fn prev(host: Option<String>) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::transport::prev(host, &roots).await
}

pub async fn seek(host: Option<String>, position: String) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::transport::seek(host, &roots, position).await
}

pub async fn volume(host: Option<String>, value: Option<String>, json: bool) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::transport::volume(host, &roots, value, json).await
}

pub async fn mute(host: Option<String>, state: Option<String>) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    cli::transport::mute(host, &roots, state).await
}

pub async fn queue(host: Option<String>, cmd: QueueCmd) -> i32 {
    let roots = paths::ProfileRoots::resolve(None, None);
    match cmd {
        QueueCmd::List { json } => cli::queue::list(host, json, &roots).await,
        QueueCmd::Add { track_id, next } => {
            cli::queue::add(host, &roots, track_id, next).await
        }
        QueueCmd::Remove { index } => cli::queue::remove(host, &roots, index).await,
        QueueCmd::Clear { keep_current } => {
            cli::queue::clear(host, &roots, keep_current).await
        }
        QueueCmd::Move { from, to } => cli::queue::move_(host, &roots, from, to).await,
        QueueCmd::Jump { position } => cli::queue::jump(host, &roots, position).await,
        QueueCmd::StopAfter { arg } => cli::queue::stop_after(host, &roots, arg).await,
    }
}
