# Data flow

Every screen is a view over the daemon's HTTP API. The TUI holds no music
state of its own — it holds a cache of what the daemon last said, and the
daemon is always right.

## Startup

| Call | Fills |
|---|---|
| `GET /api/ping` | Fail fast with "no daemon on `<host>`" instead of an empty UI |
| `GET /api/info` | Header: version, bind address |
| `GET /api/now-playing` | Footer + Now screen, before the stream has said anything |
| `GET /api/queue` | Queue pane |

Then subscribe to `GET /api/events` and stop asking.

## Live — `/api/events`

Typed `CoreEvent` frames. `api/sse/format.rs` decides what is worth pushing;
what arrives is playback, queue, volume, auth, favorites, playlists, errors
and device changes.

| Event | Effect |
|---|---|
| playback | Footer, Now screen, progress bar, quality line |
| queue | Queue pane, and the `▸` marker |
| volume | Footer indicator |
| favorites | The `f` state on every visible row |
| playlists | Library right pane |
| auth | Header badge; a lost session must be visible, not silent |
| error | Toast in the footer, three seconds |
| device | Quality line — this is how a forced downgrade surfaces |

**Search results are not in that list, and that is intentional**, not an
oversight to work around: `format_event` drops bulky search payloads so the
stream stays cheap. Search owns its own refresh.

## Per-screen

| Screen | Reads | Writes |
|---|---|---|
| Now | `/api/now-playing`, `/api/queue` | `/api/queue/{jump,remove,move,clear,stop-after}` |
| Search | `/api/search` | `/api/play`, `/api/queue/add` |
| Library | `/api/favorites`, `/api/playlists`, `/api/playlist` | `/api/favorites/{add,remove}`, `/api/queue/add` |
| Browse | `/api/album`, `/api/artist`, `/api/similar` | `/api/play`, `/api/queue/add`, `/api/favorites/*` |
| Transport | — | `/api/playback/{toggle,next,previous,seek,volume,shuffle,repeat,stop}` |

## Three rules that keep it honest

**Optimistic, then corrected.** A keypress redraws immediately and fires the
request. The event that comes back is the truth; if it disagrees, the event
wins. Waiting for the round trip makes the UI feel broken over a network,
which is exactly the case that justifies the HTTP-client design.

**The daemon outlives the TUI.** `q` quits the client. Playback continues.
Nothing in the TUI may stop playback as a side effect of exiting.

**A dropped stream is a visible state.** If `/api/events` disconnects, the
header says so and the TUI reconnects with backoff. It must never keep
rendering stale state as though it were live — that is worse than an error,
because it looks like it is working.
