# crates/qbz-ui/ui/primitives/ExpandableSearch.slint (165 lines)

One-line summary: the magnifier-that-expands-into-a-search-box primitive (JUMP TO bar pattern) — closed toggle + open overlay in one file.

## Proposed split
- `ExpandableSearch.slint` (~95 lines) — **stays the public re-export** (`export component ExpandableSearch`). Keeps the `open`/`query` properties, the focus-timer (lines 29-42), the closed-state magnifier toggle (lines 50-72), and composes the overlay below.
- `ExpandableSearchOverlay.slint` (~75 lines, new, not exported) — the right-anchored open overlay: input + placeholder + trailing clear-X (lines 76-164). Takes `query` (in-out), `placeholder`, `sm`, `max-open-width`, and an `edited(string)` / `closed()` callback.

## Tricky coupling to flag
- The overlay's `x: parent.width - self.width` anchoring and its width-animation (`animate width`) depend on being a direct child of the same root the closed toggle sits in — when extracted, the new component must still be placed as a sibling Rectangle inside `ExpandableSearch`'s root so the anchoring math (`parent.width`) still resolves against the same parent.
- `input.has-focus` currently drives the overlay's border-color directly; when split, either expose a `has-focus` output property from the overlay or keep the border logic inside the overlay component itself (simpler — recommended).
- The `focus-timer` in the main file calls `input.focus()` where `input` is the TextInput inside the overlay — this needs the overlay's TextInput to be reachable by id across the split (Slint requires either exposing a function on the overlay component, e.g. `overlay.focus-input()`, or keeping the timer's trigger logic inside the overlay component instead of the parent).

## Verify after split
- Compiles; the JUMP TO bar (ArtistView) and any other caller still opens smoothly (200ms width animation), focuses the input via the 30ms deferred timer without the "Recursion detected" panic the comment warns about, and the trailing X still clears + closes.
