# Contributing to Qoqobuz

Qoqobuz is a fork of [QBZ](https://github.com/vicrodh/qbz) by
[@vicrodh](https://github.com/vicrodh). Contributions to the *player itself*
are often worth sending upstream too — this fork exists for its own packaging,
release channels and trimmed feature set, not to compete with it.

This project is actively evolving. Contributions are welcome, but we have a few rules to keep releases stable and avoid regressions (especially around audio output).

## Where the code lives

The live app is the Rust workspace under `crates/` — a single native process
with a Slint UI. UI code is in `crates/qbz-ui` (`.slint` + generated bindings)
and `crates/qbz` (the binary). The old Svelte `src/` and Tauri `src-tauri/`
trees were **deleted in 2.0.2**; they survive only at the git tag
`legacy-tauri-svelte` for reference. PRs against those paths cannot be merged —
port the change to `crates/` instead.

## Quick rules

- Write clear, concise English (no emojis in code, comments, or commit messages).
- Keep PRs focused and small when possible.
- Do not change app branding or legal disclaimers without discussing it first.
- Do not modify protected audio-backend behavior unless explicitly requested by the maintainer.

## Branch naming

We use a consistent branch naming scheme:

`<type>/<origin>/<branch_name>`

- `type`: `feature` | `bugfix` | `hotfix` | `refactor` | `release` | `chore` | `docs`
- `origin`:
  - `internal`: created/owned by maintainers
  - `external`: branches/commits authored by third-party contributors (PRs)

Examples:

- `feature/internal/offline-cache-encryption`
- `bugfix/internal/login-footer-alignment`
- `docs/internal/contributing-process`
- `feature/external/add-album-to-playlist`

## Branch workflow

Work integrates on **`int`**, lands on **`main`**, and ships from **`prod`**.

```
feature/xyz ──┐
bugfix/abc  ──┼──> int ──> main ──> prod
hotfix/123  ──┘        (trunk)   (shipped)
```

### Branch hierarchy

1. **`int`** - Integration branch. All features and fixes merge here first.
   Every push builds and publishes the `channel-int` `.deb` (pre-release).
2. **`main`** - The trunk: stable, protected, and **inert in CI**. A push to
   it builds nothing and publishes nothing.
3. **`prod`** - What users install. Every push publishes the `channel-prod`
   `.deb`, marked latest.
4. **`feature/*`, `bugfix/*`, etc.** - Individual work branches.

Only `int` and `prod` are wired to CI — see
[.github/RELEASE-CHANNELS.md](.github/RELEASE-CHANNELS.md). Version-tagged
(`v*`) releases are a separate pipeline that also ships rpm/AppImage/tarballs,
and its tags must sit on `main`'s history.

### For contributors

**All PRs must target `int`, not `main`.**

A PR opened against `main` is retargeted at `int` automatically by
`.github/workflows/redirect-pr-to-int.yml`.

### Procedure (maintainer)

1. **Triage**
   - Confirm scope and that it does not touch protected areas (audio routing/backends, credential storage, etc.) unless requested.
   - Verify PR targets `int` (not `main`).
2. **Check out the PR**
   - `gh pr checkout <PR_NUMBER>`
3. **Rename the checked-out branch (local)**
   - Use an `external` branch name so it's obvious these commits are third-party authored:
   - `git branch -m <type>/external/<topic>`
4. **Merge to int**
   - `git checkout int`
   - `git merge --no-ff <type>/external/<topic>`
5. **Run checks**
   - Build/validate a touched core crate: `cargo check -p <crate>` (run from
     `crates/`). The full UI (`qbz`/`qbz-ui`) is a ~20–30 GB compile — see the
     README "Building from Source" section before attempting it.
6. **Push int**
   - `git push origin int` — this publishes a fresh `channel-int` `.deb`.
7. **Close the PR with a comment** explaining it was merged to `int`.

### Promoting: int -> main -> prod

Once the `int` channel build has been exercised:

```bash
git checkout main && git merge int && git push origin main   # no CI runs here
git checkout prod && git merge main && git push origin prod  # publishes channel-prod
```

There is nothing else to do: the `.deb` is the only artifact, and pushing
the branch is what publishes it. Tags build nothing.

This is done exclusively by maintainers.

### Merge strategy note (to preserve “external” authorship)

If you want the git history to clearly show third-party authored commits, avoid “squash merge”.
Prefer:

- **Create a merge commit**, or
- **Rebase and merge** (preserves individual commits/authors)

## What to include in PRs

- A short description of the problem and solution.
- Screenshots for UI changes when possible.
- Notes about any breaking changes or migrations.

## What not to include

- Large refactors mixed with feature work.
- Changes that reintroduce removed UI/UX patterns (for example, exporting offline cache files).

---

## Internationalization (i18n)

Qoqobuz ships 8 locales (`en es de fr pt ru ja nl`) as gettext `.po` files, bundled
via the `qbz-i18n` crate. Rules:

- **No hardcoded UI strings in `.slint`** — every string goes through
  `@tr("...")`.
- Adding or changing a string means updating **all** locale `.po` files, not
  just English.
- `@tr` property defaults are not reactive — re-seed from Rust on language
  change (see `crates/qbz-i18n` and `select_bundled_translation()`, called
  after `AppWindow::new()`).

### Checklist for PRs with UI Text

- [ ] No hardcoded strings in `.slint` — all text via `@tr`
- [ ] Every new/changed string updated across all 8 `.po` locales
- [ ] Reused an existing string where one already fit
