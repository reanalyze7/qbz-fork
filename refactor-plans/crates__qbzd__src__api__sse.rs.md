# crates/qbzd/src/api/sse.rs (156 lines)

## Summary
The `GET /api/events` Server-Sent Events endpoint: streams `CoreEvent`s from a
tokio broadcast bus to one HTTP client on a dedicated thread (so the
single-threaded control-plane serve loop isn't blocked), filtering out bulky/
internal event types and rendering the rest as SSE frames.

## Proposed split
Just over budget (156 vs 130). Split the HTTP-response wiring (IO-ish) from
the pure event-to-frame rendering/filtering (the two are already visually
separated by the file's own comments):

- `sse/mod.rs` (~50 lines) — module doc, `stream()` (the public entry point,
  27-37) and the small `header()` helper (39-42). This is the "IO surface":
  building the `tiny_http::Response` and headers.
- `sse/reader.rs` (~55 lines) — the `SseReader` struct + `impl Read for
  SseReader` + `SseReader::new`/`next_frame` (44-94): the blocking-`Read`
  adapter over the broadcast receiver. This is the trickiest piece (blocking
  recv, lag handling, EOF-on-close semantics) and benefits from being isolated
  with its own focused doc comment.
- `sse/format.rs` (~30 lines) — `format_event` and `emit` (96-124): the pure
  event-to-SSE-frame rendering and the allow/deny-list filter. Pure functions,
  no IO, the natural candidate for the most unit-testable piece.
- `sse/tests.rs` (~30 lines) — the `#[cfg(test)] mod tests` block (126-156):
  playback-event-becomes-frame, bulky/internal-events-not-emitted, volume/queue
  events-emitted. Since these only exercise `format_event`, could alternatively
  stay inline in `format.rs` as `#[cfg(test)] mod tests` instead of a separate
  file — either satisfies the line budget.

## Re-export surface
`sse/mod.rs` re-exports `stream` — the only item called from outside (per the
comment: "the `GET /api/events` Server-Sent Events stream", presumably wired
from `qbzd`'s route table via `sse::stream(req, rx)`). `SseReader`,
`format_event`, `emit`, `header` stay private (`pub(super)`/private) except
where cross-file visibility is needed (`SseReader` used by `mod.rs::stream`,
`format_event`/`emit` used by `reader.rs::next_frame`).

## Coupling / watch out
- `SseReader::next_frame`'s priming behavior (`!self.primed` sends the
  `": qbzd event stream\n\n"` comment FIRST, before ever touching `rx`) is a
  one-shot state machine — keep the `primed` field and its check exactly as-is;
  splitting `format.rs` out must not touch this priming logic in `reader.rs`.
- `next_frame` calls `format_event(&ev)` and loops (`// Not an emitted event —
  keep waiting`) when it returns `None` — this cross-file call (`reader.rs` ->
  `format.rs`) is the ONE coupling point between the IO-ish reader and the pure
  formatter; make sure `format_event` stays `pub(super)` visible to `reader.rs`.
- The `Lagged(n)` branch renders a comment frame INLINE in `next_frame` (not
  via `format_event`) — don't accidentally route it through `format_event`
  during the split, since lag notices are a transport-layer concern, not a
  `CoreEvent`.
- `emit()`'s deny-list (`SearchResultsReceived`, `LoadingStarted/Completed`,
  `DownloadProgress/Completed`, `Navigate*`, `AudioDiagnostic`) is a maintained
  list that must be kept current if `CoreEvent` gains new bulky/internal
  variants elsewhere in the workspace (e.g. if `qbz-models::CoreEvent` changes)
  — not a splitting risk per se, but worth a cross-reference comment.

## Verify after split
- `cargo build -p qbzd`.
- `cargo test -p qbzd sse` — all 3 tests green.
- Manual smoke-test: run `qbzd`, `curl -N http://localhost:<port>/api/events`
  (or whatever the control-plane port is) and confirm the priming comment line
  appears immediately, followed by real event frames during playback, per the
  `run` skill's guidance for confirming a change works in the real app.
