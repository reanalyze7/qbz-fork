<p align="center">
  <img src="static/logo.png" alt="Qoqobuz logo" width="180" />
</p>

<p align="center">
  <a href="https://github.com/reanalyze7/qbz-fork"><img src="https://img.shields.io/badge/github-reanalyze7%2Fqbz--fork-0b0b0b?style=flat-square&logo=github" alt="GitHub repo" /></a>
  <a href="https://github.com/vicrodh/qbz"><img src="https://img.shields.io/badge/fork%20of-vicrodh%2Fqbz-0b0b0b?style=flat-square&logo=git" alt="Fork of vicrodh/qbz" /></a>
  <a href="https://github.com/reanalyze7/qbz-fork/releases/tag/channel-prod"><img src="https://img.shields.io/badge/channel-prod-0b0b0b?style=flat-square&logo=debian" alt="prod channel" /></a>
  <a href="https://github.com/reanalyze7/qbz-fork/releases/tag/channel-int"><img src="https://img.shields.io/badge/channel-int-0b0b0b?style=flat-square&logo=debian" alt="int channel" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-0b0b0b?style=flat-square" alt="License MIT" /></a>
  <a href="#"><img src="https://img.shields.io/badge/platform-Linux%20%7C%20macOS-0b0b0b?style=flat-square&logo=linux" alt="Platform" /></a>
</p>

# Qoqobuz

**Qoqobuz is a fork of [QBZ](https://github.com/vicrodh/qbz) by [@vicrodh](https://github.com/vicrodh).**
It exists to keep only the part of QBZ I actually use — bit-perfect playback —
and to ship that as one `.deb` I control, instead of a dozen packaging paths
and features I never open.

It is not the upstream project, it is not endorsed by it, and issues found here
belong [here](https://github.com/reanalyze7/qbz-fork/issues) — never upstream.
Everything good about the player was built there; what this fork changes is
listed below and nowhere else.

Qoqobuz is a free and open source high-fidelity streaming client for Linux and
macOS with fully native playback: a single native Rust process with a Slint UI —
no browser engine, no webview — doing DAC passthrough, per-track sample-rate
switching, exclusive mode and bit-perfect delivery.

No API keys needed. No telemetry. No tracking. Just music.

## What this fork changes

| | Upstream QBZ | Qoqobuz |
|---|---|---|
| Package / command | `qbz` | `qoqobuz` (the cargo crate is still `qbz` — see below) |
| App ID | `com.blitzfc.qbz` | `io.github.reanalyze7.qoqobuz` |
| Distribution | deb, rpm, AppImage, tarball, dmg, AUR, Gentoo, Nix, Flatpak, Snap, APT repo | **one `.deb`**, built on GitHub, on two rolling channels |
| Release trigger | `v*` version tags | a push to `int` or `prod` |
| macOS build | published | not published (the code is still there) |
| Casting (Chromecast/DLNA) | shipped | not in this tree |
| Qobuz Connect | shipped | not in this tree |
| Plex library | shipped | not in this tree |

The **cargo workspace deliberately keeps upstream's crate names** (`qbz`,
`qbz-ui`, `qbzd`, …). The rename happens at packaging time — the binary built
from the `qbz` crate installs as `/usr/bin/qoqobuz`. That way pulling upstream
changes stays a merge instead of a rename war, and only what a user actually
sees carries the fork's name.

## Legal / Branding

- This application uses the Qobuz API but is not certified by Qobuz.
- Qobuz is a trademark of Qobuz. Qoqobuz is not affiliated with, endorsed by, or certified by Qobuz.
- Qoqobuz is a fork of QBZ by vicrodh, MIT-licensed like its upstream, and not affiliated with it either.
- **Offline cache** is a temporary playback store for listening without an internet connection while you have a valid subscription. If your subscription becomes invalid, cached content is removed after 3 days.
- **Local library** is a "bring your own music" feature — play your own files with bit-perfect audio and the full interface, no streaming subscription required.
- Qobuz Terms of Service: https://www.qobuz.com/us-en/legal/terms

## Why it exists

Upstream's reasons still hold, and this fork does not dilute them: browsers cap
audio output at 48 kHz and resample everything through WebAudio, so a web
wrapper can never be bit-perfect. Qoqobuz uses a native playback pipeline with
direct device control, and your DAC receives the original resolution, up to
24-bit / 192 kHz, with no forced resampling.

It is not a download tool and never will be. It has no built-in equalizer or
DSP either — the point is to send audio to your DAC untouched, bit for bit, and
any processing would break that. Add effects at the system level instead
(EasyEffects on PipeWire, a JACK graph).

## Installation

Qoqobuz ships as **rolling channel `.deb` packages**. Each channel has one
permanent download URL that always serves that channel's current build, so an
install script or a provisioning job can hardcode it forever.

| Channel | Who it is for | Built from |
|---|---|---|
| **prod** | what you want unless you know otherwise | the `prod` branch |
| **int** | integration builds, ahead of prod, expect rough edges | the `int` branch |

```sh
# stable channel
sudo apt install -y https://github.com/reanalyze7/qbz-fork/releases/download/channel-prod/QOQOBUZ_prod_amd64.deb

# integration channel
sudo apt install -y https://github.com/reanalyze7/qbz-fork/releases/download/channel-int/QOQOBUZ_int_amd64.deb
```

Upgrading is the same command again — the URL does not move. Requires glibc
2.39+ (Ubuntu 24.04+, Debian 13+) and an x86_64 machine. Full details:
[.github/RELEASE-CHANNELS.md](.github/RELEASE-CHANNELS.md).

**There is no other package.** No rpm, no AppImage, no tarball, no dmg, no
AUR, no Gentoo overlay, no Nix flake, no Flatpak, no Snap, no APT repository —
all of it was deleted along with the pipelines that produced it. Anything
other than the `.deb` above, you build yourself from source.

macOS is not published either. The CoreAudio backend (including Core Audio
Direct passthrough) is still in the code and still builds, there is simply no
release for it: build it locally if you want it.

## Features

### Audio and playback

- **Bit-perfect playback** with DAC passthrough and per-track sample rate switching (44.1–192 kHz)
- **Linux backends:** PipeWire, ALSA (with a Direct `hw:` bypass mode), PulseAudio and JACK
- **macOS backend:** CoreAudio, including a Core Audio Direct passthrough path
- **HiFi Wizard** — hardware auto-detection and a guided bit-perfect setup
- Native decoding: FLAC, MP3, AAC, ALAC, WavPack, Ogg Vorbis, Opus (Symphonia)
- **DSD support** — DSF/DFF playback with DSD-to-PCM conversion, DoP, and native DSD passthrough (ALSA)
- CMAF stream support (`qbz-cmaf`: init/segment parsing, key derivation, frame decryption)
- Gapless playback on all backends
- **Loudness normalization** (EBU R128) with ReplayGain support
- Two-level audio cache with next-track prefetching, and streaming playback that starts before the download completes

### Queue and library

- Queue with shuffle, repeat (track/queue/off) and history
- Favorites and playlists from your Qobuz account, including native playlist follow/unfollow
- **Local library** — directory scanning, metadata extraction, CUE sheets, SQLite indexing; usable without ever logging into Qobuz
- **Offline cache** with fully offline playlists and automatic reconnection
- **Artist/album blacklist** — block artists or individual albums, fully reversible
- Tag editor with sidecar storage (original files preserved), virtualized lists for large libraries

### Discovery

- **Scene Discovery** — explore artists by location and musical scene (MusicBrainz-powered)
- **3-tab Home:** customizable Home, Editor's Picks, personalized For You
- **Recommendations** from Last.fm and ListenBrainz/MusicBrainz plus local-listen vectorization
- Live search overlay with a cache layer that learns what you never click
- Genre filtering, artist similarity, radio stations, musician pages, label pages, album credits
- **Mixtapes** — generated DJ-mix / random-queue sets from your own library and favorites

### Integrations

- **MPRIS** media controls and media keys
- **Last.fm** scrobbling and now-playing, **ListenBrainz** scrobbling with an offline queue
- **MusicBrainz** artist enrichment, musician credits, relationships (one-way pull, no telemetry)
- **Discogs** artwork for the local library
- **Music-link resolver** — paste a Qobuz, Spotify, Apple Music, Tidal, Deezer, song.link or album.link URL and it finds the Qobuz equivalent
- Playlist import from Spotify, Apple Music, Tidal and Deezer
- Desktop notifications with artwork

### Immersive player

- Full-screen player in two layouts: full-bleed focus, or split (artwork + metadata beside a data panel)
- Focus views — Album Reactive, Static, Coverflow, Spectrum, Lyrics (one giant centered line), Queue
- GPU shader scenes behind it: Plasma, Tunnel, Aurora, Spectral Ribbon, Line Bed, Liquid Spectrum, plus an app-wide Ambient underlay
- Synchronized lyrics. In the split layout only the Lyrics panel is implemented so far — Track Info, Suggestions and Queue are not there yet

### Interface

- **37 themes** (Dark, OLED, Nord, Dracula, Tokyo Night, the four Catppuccins, Breeze, Adwaita, Rose Pine Dawn, WCAG/High-Contrast/Colorblind accessibility sets…)
- Auto-theme from the desktop environment, wallpaper or a custom image
- Mini player, configurable keyboard shortcuts, UI scale presets (XS–XL)
- **8 languages:** English, Spanish, German, French, Portuguese, Russian, Japanese, Dutch
- Album booklets download to your device
- **Offline mode** usable without ever logging into Qobuz

## Headless daemon (qbzd)

<p align="center">
  <img src="static/readme-qbzd.png" alt="qbzd — headless daemon CLI and TUI" width="760" />
</p>

`qbzd` turns a headless Linux box — a Raspberry Pi, a NAS, a living-room
mini-PC — into a bit-perfect playback endpoint. It is a standalone ~25 MB
binary, shipped inside the `.deb` and as its own tarball.

- Daemon + full CLI + terminal setup wizard (TUI) in one binary
- Browser-based login that works over SSH; one-file settings hand-off from the desktop app
- HiFi wizard with copyable audio-stack config blocks (clipboard works over SSH)
- MPRIS out of the box, live JSON events (`qbzd watch`), service files for systemd/OpenRC/runit

The daemon keeps upstream's `qbzd` name everywhere — its CLI, its config paths,
its unit file. Only the desktop app was renamed.

## Tech stack

A single native Rust process: the **Slint UI runs in-process** and talks to the
Rust core crates directly through callbacks and shared state. No browser
engine, no webview, no IPC bridge to serialize across.

| Layer | Technology |
|-------|-----------|
| **Desktop shell + UI** | Rust + Slint (native, single process) |
| **Audio decoding** | Symphonia (all codecs) via rodio |
| **Audio backends** | Linux: PipeWire, ALSA (incl. Direct `hw:`), PulseAudio, JACK. macOS: CoreAudio (incl. Core Audio Direct) |
| **Networking** | reqwest (rustls-tls) |
| **Database** | rusqlite (bundled SQLite, WAL mode) |
| **Desktop integration** | mpris-server (Linux MPRIS), souvlaki (macOS media controls), ksni (Linux tray), keyring |
| **i18n** | qbz-i18n, gettext-style `.po` bundles compiled into the binary (8 locales) |

### Workspace

The Rust workspace lives entirely under `crates/` (manifest
`crates/Cargo.toml`) and has 29 members. `qbz` is the binary crate, `qbz-ui`
holds the Slint views and view-models:

```
crates/
  qbz/                   Binary crate: app entrypoint, window/renderer setup, wiring
  qbz-ui/                Slint views (.slint) and view-model glue
  qbz-slint-common/      Slint-coupled helpers shared out of the binary
  qbz-app/               Application-level orchestration (non-UI)
  qbz-core/              Orchestrator (player + audio + API)
  qbz-audio/             Audio backends, loudness, device management
  qbz-player/            Playback engine, streaming, queue
  qbz-dsd/               DSD (DSF/DFF) decoding, DoP, native DSD packing
  qbz-cmaf/              CMAF init/segment parsing, key derivation, frame decryption
  qbz-qobuz/             Qobuz API client and auth
  qbz-models/            Shared domain types
  qbz-library/           Local library scanning and metadata
  qbz-cache/             L1 memory + L2 disk audio caching
  qbz-offline-cache/     Offline playback store (frontend-agnostic)
  qbz-integrations/      Last.fm, ListenBrainz, MusicBrainz, Discogs, remote metadata
  qbz-reco/ qbz-external-reco/  Recommendations engine
  qbz-mixtape/           Mixtape / DJ-mix generation
  qbz-music-link/        Cross-platform music-link resolver
  qbz-playlist-import/   Spotify, Apple Music, Tidal, Deezer import
  qbz-media-controls/    MPRIS / macOS media controls
  qbz-dac-wizard/        HiFi Wizard (hardware auto-detection)
  qbz-theme/             Theme engine (37 themes)
  qbz-i18n/              Bundled translations (8 locales)
  qbz-credentials/ qbz-secrets/  Auth/token storage
  qbz-text-utils/ qbz-log/       Text/date utilities, logging core
  qbzd/                  Headless daemon: service, CLI and TUI
```

## Building from source

Pure Rust workspace — no Node.js, no `npm install`, no webview. The manifest is
`crates/Cargo.toml` and the app binary is the `qbz` crate inside it.

### Prerequisites

- **Rust stable** for a plain `cargo build`, or **nightly** if you use the repo's build scripts (they pass `-Z threads` to parallelize the compiler frontend).
- Linux or macOS with audio support. On Linux the scripts use [`mold`](https://github.com/rui314/mold) as linker (install it, or edit the `RUSTFLAGS` they set); plain `cargo build` needs neither nightly nor mold.

### System dependencies

Verified against this repo's own Linux CI build.

**Debian/Ubuntu:**
```bash
sudo apt install build-essential pkg-config cmake clang libclang-dev nasm \
  libasound2-dev libjack-jackd2-dev libfontconfig1-dev libfreetype-dev \
  libxkbcommon-dev libwayland-dev libxcb1-dev libgl1-mesa-dev libegl1-mesa-dev \
  libdbus-1-dev libssl-dev
```

**Other distros:** look for the equivalents — a C compiler + clang/libclang,
cmake, nasm, ALSA + JACK dev headers, fontconfig/freetype, Wayland/X11/
xkbcommon, Mesa GL/EGL, D-Bus and OpenSSL dev headers.

**macOS:** Xcode Command Line Tools (`xcode-select --install`) and a Rust
toolchain — that's it.

### ⚠ The memory wall (read this before your first build)

Slint compiles the entire UI into ONE generated Rust module (~1.6 M lines). A
single **release** `rustc` invocation for that crate peaks at **20–30 GB of
RAM**. It is a one-time cost per profile — once the UI crate is cached,
incremental builds are cheap — but the first build WILL hit it, and on a
machine without enough headroom the compiler gets OOM-killed (or swap-freezes
the box).

| Knob | Effect |
|------|--------|
| `CARGO_BUILD_JOBS=1` | one rustc at a time — the single biggest saver |
| `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=256` | smaller codegen chunks (slightly less optimized binary) |
| `CARGO_PROFILE_RELEASE_OPT_LEVEL=2` | opt 2 instead of 3 shaves several GB |
| `-Z threads=1..2` (nightly RUSTFLAGS) | fewer frontend threads = less parallel memory |
| swap | the peak tolerates swap; 16 GB RAM + ~18 GB swap is CI-proven |

**Under 32 GB of RAM** — this is literally the recipe the 16 GB CI runners use;
expect slow (2–3 h from scratch) rather than failing:

```bash
CARGO_BUILD_JOBS=1 \
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=256 \
CARGO_PROFILE_RELEASE_OPT_LEVEL=2 \
cargo build --release --manifest-path crates/Cargo.toml -p qbz
```

**With ≥ 32 GB free**, the plain build is fine and much faster:

```bash
git clone https://github.com/reanalyze7/qbz-fork.git && cd qbz-fork
cargo build --release --manifest-path crates/Cargo.toml -p qbz
./crates/target/release/qbz
```

The built binary is `qbz` — the `qoqobuz` name is applied by the packaging
step, not by cargo.

### The convenient way (Linux): `scripts/slint-run.sh`

It reads `MemAvailable` and picks a tier automatically:

| Free RAM | Tier | Settings |
|----------|------|----------|
| ≥ 26 GB | FAST | threads=16, cgu=16, opt=3 — the distribution-grade build |
| 14–26 GB | SAFE | threads=2, cgu=256, opt=3 |
| < 14 GB | MIN | threads=1, cgu=256, opt=2 — slow but never freezes |

```bash
./scripts/slint-run.sh          # build (auto-tier) and run
NORUN=1 ./scripts/slint-run.sh  # build only
FAST=1  ./scripts/slint-run.sh  # force the fast tier (close your apps first)
THREADS=4 CODEGEN_UNITS=128 OPT=3 ./scripts/slint-run.sh  # manual override
```

Requires nightly and `mold`. SAFE/MIN produce a functionally identical but not
byte-identical binary vs FAST, and switching tiers forces a one-time rebuild of
the UI crate.

### Development iteration

- **`./scripts/slint-dev.sh`** — builds and runs in **release** mode with the parallel compiler frontend. The default loop: runtime performance is the whole reason this is a native app, so a regression can never hide behind "it's just dev mode".
- **`./scripts/slint-dev-fast.sh`** — **debug** build, far faster to compile and lighter on RAM. For visual/layout iteration only; never benchmark on it.
- **`./scripts/slint-dev-mac.sh`** on macOS (Apple-toolchain flags; an 8 GB M-series Mac builds fine, just slowly).

### Packaging it yourself

The whole packaging surface is two files:
[packaging/nfpm/nfpm.yaml](packaging/nfpm/nfpm.yaml) (the `.deb`) and
[packaging/linux/qoqobuz.desktop](packaging/linux/qoqobuz.desktop), driven by
[.github/actions/qbz-channel-publish](.github/actions/qbz-channel-publish).
Run `nfpm package -f packaging/nfpm/nfpm.yaml -p deb` against a staged `./qbz`
and `./qbzd` to get the same package the CI publishes. Any other format is
yours to write.

### API proxy

Last.fm, Discogs, Tidal, Spotify-import and MusicBrainz traffic goes through a
hosted Cloudflare Workers proxy that holds the credentials server-side, so
**no API keys or `.env` file are required** — inherited from upstream and
unchanged. To run your own, the proxy source is at
[`vicrodh/qbz-api-proxy`](https://github.com/vicrodh/qbz-api-proxy); deploy it
with `wrangler deploy`, then edit the `*_PROXY_URL` constants in
`crates/qbz-integrations/src/lastfm/client.rs`,
`crates/qbz-integrations/src/discogs/mod.rs`,
`crates/qbz-playlist-import/src/providers/tidal.rs` and
`crates/qbz-integrations/src/musicbrainz/client/core.rs`.

### Environment variables

A working renderer (GPU wgpu → femtovg/OpenGL → software) is auto-detected at
startup across Wayland/X11/Metal, so there is normally nothing to configure.
One override remains for diagnostics or a broken GPU stack:

| Variable | Effect |
|----------|--------|
| `QBZ_RENDERER=software` (or `cpu`, `soft`) | Force the software renderer (crash recovery, VMs) |
| `QBZ_RENDERER=gl` (or `gles`, `femtovg`) | Force the femtovg/OpenGL renderer (mid-tier GPUs) |
| `QBZ_RENDERER=wgpu` (or `gpu`, `hardware`, `hw`) | Force the wgpu (GPU) renderer |

If it fails to start, try `QBZ_RENDERER=software qoqobuz` first. The variable
names keep the `QBZ_` prefix: they are read by the `qbz` crate, and renaming
them would break every existing script for no gain.

## Known issues

- **Hi-Res seeking** — seeking in tracks >96 kHz can take 10–20 s (the decoder must scan from the start). Use prev/next for instant navigation.
- **ALSA Direct** — exclusive access blocks other apps. Use your DAC/amplifier's physical volume control.
- **DSD DoP / native mode** — seeking is disabled and volume is fixed while a DoP or native-DSD stream is active (any sample manipulation would corrupt the stream). Convert-to-PCM mode has no such limits.
- **What's New** reads this fork's `v*` releases, and this fork cuts none — the panel stays empty by design.

## Contributing

Work integrates on `int`, lands on `main`, ships from `prod`. Only `int` and
`prod` are wired to CI — pushing one of them is what publishes its `.deb`, and
nothing else in this repo publishes anything. A push to `main` builds nothing,
and a PR opened against `main` is retargeted at `int` automatically. Read
[CONTRIBUTING.md](CONTRIBUTING.md) before opening an issue or a pull request.

## Credits

Qoqobuz exists because [@vicrodh](https://github.com/vicrodh) wrote QBZ. The
upstream project and its contributors are the authors of essentially all of
this code:

- [@vorce](https://github.com/vorce)
- [@boxdot](https://github.com/boxdot)
- [@arminfelder](https://github.com/arminfelder)
- [@afonsojramos](https://github.com/afonsojramos) — macOS port
- [@Vudgekek](https://github.com/Vudgekek) — macOS audio
- [@GwendalBeaumont](https://github.com/GwendalBeaumont) — i18n
- [@AdamArstall](https://github.com/AdamArstall)
- [@DoubleGate](https://github.com/DoubleGate)
- [@hoyon](https://github.com/hoyon), [@mxnix](https://github.com/mxnix), [@TerminalTilt](https://github.com/TerminalTilt)

## License

MIT, same as upstream. No telemetry, no tracking, no hidden services.
