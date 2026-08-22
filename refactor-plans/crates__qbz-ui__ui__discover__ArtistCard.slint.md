# crates/qbz-ui/ui/discover/ArtistCard.slint (145 lines)

## Summary
A single artist tile component (160x220, ported 1:1 from the Tauri
SearchView `.artist-card`): circular avatar with gradient placeholder,
centered name, optional "Similar to X" subtitle, and a rectangular
Follow/Following chip whose visibility depends on `follow-mode`.

## Proposed split
Only 15 lines over budget — the smallest file in this batch. A single
extraction of the Follow-chip block (the most self-contained, visually
distinct sub-tree) is enough to bring both files comfortably under budget.

- `ArtistCard.slint` (~100 lines) — root `ArtistCard` component: the card
  container, avatar circle + gradient + placeholder icon + artwork image,
  name text, optional subtitle text, and the conditional mount of the new
  `ArtistFollowChip` component (lines 101-143 replaced by a single component
  instantiation).
- `discover/ArtistFollowChip.slint` (~55 lines) — extracted Follow/Following
  chip: takes `in property <bool> following`, `in property <string>
  follow-kind`, `in property <string> artist-id`, and
  `callback media-action(string, string, string)`; internally renders the
  chip Rectangle + icon + text + TouchArea exactly as today, calling
  `root.media-action(follow-kind, artist-id, "follow")` on click. The
  `follow-mode`-based visibility condition
  (`(follow-mode == "auto" && !following) || follow-mode == "toggle"`) stays
  in `ArtistCard.slint`'s `if` around the chip's mount point (it's a
  visibility gate for the whole component, appropriately owned by the
  parent), while the chip's internal Rectangle/TouchArea markup moves.

## Re-export surface
`ArtistCard.slint` remains the single import path
(`import { ArtistCard } from "discover/ArtistCard.slint";`) — its property
list (`artist`, `follow-mode`, `follow-kind`, `card-height`) and callbacks
(`clicked`, `media-action`) are unchanged; `ArtistFollowChip` is a new
internal-use component imported only by `ArtistCard.slint`.

## Coupling / watch out
- `media-action`'s signature (`string, string, string` = kind, id, action)
  must be forwarded unchanged from `ArtistFollowChip` up through
  `ArtistCard`'s existing `media-action` callback — don't accidentally
  rename or reorder the tuple.
- The chip reads `root.artist.following` today for both its icon/text choice
  AND the outer visibility `if` — after extraction, `ArtistCard.slint` keeps
  the visibility `if` (needs `root.artist.following` and `root.follow-mode`)
  while `ArtistFollowChip.slint` needs its OWN `following` property passed
  in explicitly for its icon/text choice — make sure both stay in sync (i.e.
  `ArtistCard.slint` passes `following: root.artist.following`).
- `card-height` is a caller-overridable property specifically because the
  Follow chip adds ~30px of height in some callers (per the comment at line
  26-28) — this coupling between "does this instance show the chip" and
  "how tall the card needs to be" is pre-existing caller responsibility, not
  something the split changes, but worth flagging since it's easy to miss
  when only skimming the extracted chip file.

## Verify after split
- Slint compile check on both files.
- `cargo build -p qbz-ui`.
- Smoke-test artist search results, Top Artists carousel, and "Artists to
  Follow" grid cards (the three `follow-mode` variants: "auto", "toggle",
  and implicitly "none" via label-artist carousels) to confirm the chip
  still shows/hides correctly and the follow toggle still round-trips.
