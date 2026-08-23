mod auth;
mod browse;
mod config;
mod library;
mod misc;
mod status;
mod transport;

use crate::cli_args::{Cli, Cmd};

pub async fn dispatch(cli: Cli) -> i32 {
    let host = cli.host;
    match cli.cmd {
        Cmd::Version { json } => misc::version(json),
        Cmd::Service { init, user, bin, system } => misc::service(init, user, bin, system),
        Cmd::Completions { shell } => misc::completions(shell),

        Cmd::Run => auth::run().await,
        Cmd::Login { callback_host, paste, token } => auth::login(callback_host, paste, token).await,
        Cmd::Logout => auth::logout(),

        Cmd::Status { json } => status::status(host, json).await,
        Cmd::Ping { json } => status::ping(host, json).await,
        Cmd::Now { json } => status::now(host, json).await,
        Cmd::Watch { raw } => status::watch(host, raw).await,

        Cmd::Search { query, kind, limit, offset, ids, json } => {
            browse::search(host, query, kind, limit, offset, ids, json).await
        }
        Cmd::Album { id, suggest, ids, json } => browse::album(host, id, suggest, ids, json).await,
        Cmd::Artist { id, top, albums, limit, ids, json } => {
            browse::artist(host, id, top, albums, limit, ids, json).await
        }
        Cmd::Similar { selector, limit, ids, json } => browse::similar(host, selector, limit, ids, json).await,
        Cmd::Suggest { seed, limit, ids, json } => browse::suggest(host, seed, limit, ids, json).await,
        Cmd::Discover { section, genre, tag, release_type, kind, limit, ids, json } => {
            browse::discover(host, section, genre, tag, release_type, kind, limit, ids, json).await
        }
        Cmd::Reco { cmd } => browse::reco(host, cmd).await,

        Cmd::Fav { cmd } => library::fav(host, cmd).await,
        Cmd::Playlist { cmd } => library::playlist(host, cmd).await,

        Cmd::Shuffle { mode } => transport::shuffle(host, mode).await,
        Cmd::Repeat { mode } => transport::repeat(host, mode).await,
        Cmd::Art { save } => transport::art(host, save).await,
        Cmd::Resolve { url } => transport::resolve(url),
        Cmd::Play { content } => transport::play(host, content).await,
        Cmd::Pause => transport::pause(host).await,
        Cmd::Toggle => transport::toggle(host).await,
        Cmd::Stop => transport::stop(host).await,
        Cmd::Next => transport::next(host).await,
        Cmd::Prev => transport::prev(host).await,
        Cmd::Seek { position } => transport::seek(host, position).await,
        Cmd::Volume { value, json } => transport::volume(host, value, json).await,
        Cmd::Mute { state } => transport::mute(host, state).await,
        Cmd::Queue { cmd } => transport::queue(host, cmd).await,

        Cmd::Settings { cmd } => config::settings(host, cmd).await,
        Cmd::Scrobble { cmd } => config::scrobble(host, cmd).await,
        Cmd::Config { cmd } => config::config(cmd),
        Cmd::Setup => config::setup().await,
    }
}
