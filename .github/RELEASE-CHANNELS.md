# Release channels

Pushing a branch publishes a `.deb`. Three branches are wired as channels;
every other branch is inert (a push to it builds nothing).

| Branch | Channel | Build profile | Rolling tag | Marked latest |
|---|---|---|---|---|
| `prod` | prod | `--release` (opt-level 2, stripped) | `channel-prod` | yes |
| `int` | int | `--release` | `channel-int` | no — pre-release |
| `main` | main | `[profile.dev]` (ours at 1, deps at 3) | `channel-main` | no — pre-release |

Install or upgrade from a channel — the URL is stable forever:

```sh
sudo apt install -y https://github.com/<owner>/<repo>/releases/download/channel-prod/QBZ_prod_amd64.deb
```

## Why it is built this way

**Rolling tags, not a tag per push.** The whole point of a channel is that
something downstream can hardcode one URL. A new tag per push would move
that URL on every build and bury the release list. The cost — no history of
superseded channel builds — is accepted: the commits are still in git, and
real history lives on the `v*` releases from `release-linux.yml`.

**`main` builds debug, not release.** A release build of `qbz` is
hours-class (the `qbz_ui` rustc peak is ~30 GB and forces
`CARGO_BUILD_JOBS=1`). Paying that on every commit to `main` is not worth
it. `[profile.dev]` in `crates/Cargo.toml` puts our crates at opt-level 1
and every dependency at 3, which is what makes a debug build usable at all —
at opt-level 0 the Slint/wgpu UI renders at about 3 FPS.

**Version separators are load-bearing.** `prod` mints `2.0.2+prod.…` and
`int`/`main` mint `2.0.2~int.…`. In dpkg ordering `+` sorts above a plain
`2.0.2` and `~` sorts below it, so a prod machine is never silently
downgraded by the tagged release, and an int box is correctly "older" than
the release it is working towards. `channel-meta.test.sh` asserts this
against real `dpkg --compare-versions`.

**Deb only.** rpm, AppImage and the AUR/Gentoo tarballs stay in
`release-linux.yml`, which is the version-tag pipeline. Channels exist to
get a runnable build into someone's hands quickly, not to fan out packaging.

## Layout

| Path | Why it exists |
|---|---|
| `workflows/release-channel.yml` | The pipeline: resolve channel → build qbz → build qbzd → publish. |
| `scripts/channel-meta.sh` | Pure branch → (channel, profile, version, tag) derivation. Runnable and testable outside CI. |
| `scripts/channel-meta.test.sh` | Tests for the above, including the dpkg ordering guarantees. |
| `actions/qbz-linux-runner-prep/` | Free disk + swap + system deps + Rust + cargo cache. Shared so the ~30 GB recipe is written once. |
| `actions/qbz-channel-publish/` | nfpm `.deb` + rolling tag + `gh release create`. |

`release-linux.yml`'s `build` job still carries its own copy of the runner
prep — it is the original and the source of truth for the recipe. If you
bump the toolchain or the dep list in one, bump the other.
