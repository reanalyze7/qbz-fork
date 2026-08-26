# `qbzd tui` — design

A full-screen terminal player for the qbz daemon. Design document only: no
code exists yet. Read this, argue with it, then it gets built.

| | |
|---|---|
| [screens.md](screens.md) | What each screen shows and how they connect |
| [keymap.md](keymap.md) | Every binding, and why it is that key |
| [data.md](data.md) | Which API call or SSE event feeds which pixel |

## Why this exists

The Slint GUI costs **36 min 49 s** to build in CI, ships a **150.8 MB**
binary and holds **1 168 MB** resident. `qbzd` costs **5 min 42 s**, ships
**25.6 MB** and holds **9.6 MB** — measured on this machine, same workflow
run, 2026-08-26.

The whole difference is Slint: `slint-build` generates a ~1.6M-line Rust
module, that one crate is where the build time and the ~30 GB rustc peak
live, and **every `.slint` edit invalidates it**. `qbzd` is Slint-free by
construction — `release-linux.yml` fails the build if `cargo tree` shows
`slint` in its graph — so none of that applies. A TUI iterates in minutes.

MPRIS already covers play/pause/next without opening 1.2 GB (`playback.mpris`
is on). What it cannot do is browse, search, or build a queue. That gap is
the entire justification for this; anything MPRIS already gives you is not a
reason to build a screen.

## Two decisions that shape everything else

**It is an HTTP client, not an in-process consumer.** It talks to
`127.0.0.1:8182` through the same client the CLI verbs use
(`crates/qbzd/src/cli/client/`), so `--host` works and the TUI can drive a
daemon on another machine — a Pi in the hall, from the laptop. An in-process
design would have been marginally simpler and would have thrown that away.

**Live state is pushed, not polled.** `/api/events` streams typed `CoreEvent`
frames. The TUI subscribes once and redraws on what arrives. One consequence
is load-bearing and is easy to miss: `api/sse/format.rs` deliberately drops
bulky search payloads and internal hints from the stream. Playback, queue,
volume, favorites, playlists, auth and device changes arrive live; **search
results do not**. Search is request/response and the TUI owns refreshing it.

## What is deliberately out of scope

- **Cover art.** `/api/artwork/current` exists and sixel/kitty could show it.
  It is a per-terminal capability with a real fallback burden, and it earns
  nothing that the album/artist text does not. Later, or never.
- **Settings.** Six TUI screens already configure the daemon (`qbzd setup`).
  Duplicating them here would mean two places to change one value.
- **Login.** `qbzd login` is a one-shot browser flow. A TUI cannot improve it.
