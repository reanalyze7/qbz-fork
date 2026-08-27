# Release channels

Pushing a branch publishes a `.deb`. Two branches are wired as channels;
every other branch is inert (a push to it builds nothing) — **`main`
included**.

| Branch | Channel | Build profile | Rolling tag | Marked latest |
|---|---|---|---|---|
| `prod` | prod | `--release` (opt-level 2, stripped) | `channel-prod` | yes |
| `int` | int | `--release` | `channel-int` | no — pre-release |

`main` was a third channel until 2026-08-27 and is not one any more: it is
the trunk `int` is cut from and `prod` is promoted from, and a push to it
now builds and publishes nothing. `channel-meta.sh` rejects it like any
other non-channel branch, so a `workflow_dispatch` on `main` fails in the
`meta` job instead of starting an hours-class build.

A retired channel leaves its rolling release behind — no build will ever
refresh it, and it keeps serving a `.deb` from the top of the release list.
`prune-stale-channels.sh` is what clears that (dry run by default):

```sh
gh auth login                                     # once
.github/scripts/prune-stale-channels.sh           # show what would go
.github/scripts/prune-stale-channels.sh --apply   # delete release + tag
```

It does not carry its own list of live channels: it asks `channel-meta.sh`,
so retiring or adding a channel stays a one-line edit in one file.

Install or upgrade from a channel — the URL is stable forever:

```sh
sudo apt install -y https://github.com/reanalyze7/qbz-fork/releases/download/channel-prod/QOQOBUZ_prod_amd64.deb
```

## Why it is built this way

**Rolling tags, not a tag per push.** The whole point of a channel is that
something downstream can hardcode one URL. A new tag per push would move
that URL on every build and bury the release list. The cost — no history of
superseded channel builds — is accepted: the commits are still in git, and
a superseded build is one `git checkout` away from being rebuilt.

**Two channels, not three.** A release build of `qbz` is hours-class (the
`qbz_ui` rustc peak is ~30 GB and forces `CARGO_BUILD_JOBS=1`). `main` used
to dodge that with a `[profile.dev]` build, but a channel nobody installs
from is an hour of runner time per merge for nothing: integration testing
happens on `int`, which builds the same release recipe users actually get.
The dev-profile path in `release-channel.yml` went away with it.

**Version separators are load-bearing.** `prod` mints `2.0.2+prod.…` and
`int` mints `2.0.2~int.…`. In dpkg ordering `+` sorts above a plain
`2.0.2` and `~` sorts below it, so a prod machine is never silently
downgraded by the tagged release, and an int box is correctly "older" than
the release it is working towards. `channel-meta.test.sh` asserts this
against real `dpkg --compare-versions`.

**Deb only, and nothing else at all.** On 2026-08-27 every other
distribution path was deleted: the `v*` pipelines (rpm, AppImage, tarballs,
dmg), the AUR and Gentoo publishers, the Nix flake, Flatpak and Snap. One
`.deb` per channel, built on GitHub, is the whole story. Anyone who wants
another format builds it themselves from `crates/`.

## Layout

| Path | Why it exists |
|---|---|
| `workflows/release-channel.yml` | The pipeline: resolve channel → build qbz → build qbzd → publish. |
| `scripts/channel-meta.sh` | Pure branch → (channel, version, tag) derivation, and the gate that rejects every non-channel branch. Runnable and testable outside CI. |
| `scripts/channel-meta.test.sh` | Tests for the above, including the dpkg ordering guarantees. |
| `scripts/prune-stale-channels.sh` | Deletes the release + tag of a channel that no longer exists. Dry run unless `--apply`. |
| `scripts/prune-stale-channels.test.sh` | Tests the selection — a false positive would delete a live channel's install URL. |
| `actions/qbz-linux-runner-prep/` | Free disk + swap + system deps + Rust + cargo cache. Shared so the ~30 GB recipe is written once. |
| `actions/qbz-channel-publish/` | nfpm `.deb` + rolling tag + `gh release create`. |

There is no version-tag pipeline any more: a `v*` tag builds nothing.
Version identity lives in `crates/Cargo.toml` and is stamped into the
package version by `channel-meta.sh`.
