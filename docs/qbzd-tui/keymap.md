# Key map

Two rules the whole map follows.

**Transport is global.** Space, `n`, `p`, seek, volume work identically on
every screen. Skipping a track should never cost you your place in a search.

**The verbs are learned once.** `Enter` plays, `a` appends, `f` favorites,
`→` opens — on a search result, a favorite, an album track, a queue row.
Anywhere a list holds something playable, those four mean the same thing.

## Global

| Key | Action |
|---|---|
| `q` | Quit the TUI. Playback continues — the daemon is not the UI. |
| `?` | Help overlay, generated from this table |
| `1`–`4` | Jump to a screen |
| `Tab` / `S-Tab` | Next / previous pane within a screen |
| `/` | Search screen with the field focused |
| `Esc` | Leave the field, close the overlay, or go back |
| `Ctrl-L` | Force a full redraw |

## Transport — every screen

| Key | Action |
|---|---|
| `Space` | Play / pause |
| `n` / `p` | Next / previous track |
| `←` / `→` | Seek −10 s / +10 s |
| `S-←` / `S-→` | Seek −60 s / +60 s |
| `+` / `-` | Volume ±5 |
| `m` | Mute toggle |
| `s` | Shuffle toggle |
| `r` | Cycle repeat: off → all → one |

`←`/`→` are seek and not list navigation on purpose: vertical lists are
`j`/`k` and the arrows, so the horizontal pair is free, and seeking is the
thing you reach for without looking.

## Lists

| Key | Action |
|---|---|
| `j` / `k` / `↑` / `↓` | Move |
| `g` / `G` | First / last row |
| `Ctrl-D` / `Ctrl-U` | Half page |
| `Enter` | Play now |
| `a` | Append to queue |
| `f` | Toggle favorite |
| `→` | Open in Browse (albums, artists) |

## Queue only

| Key | Action |
|---|---|
| `Enter` | Jump to this track |
| `d` | Remove the row |
| `J` / `K` | Move the row down / up |
| `C` | Clear the queue (confirms) |
| `.` | Stop after this track |

## Deliberately unbound

`h`/`l` stay free. The vim reflex is to make them move panes, but `Tab`
already does, and `l` is one slip away from `k` while a queue row is
selected — where the neighbouring binding deletes.

Nothing destructive is a single unconfirmed key except `d` on a queue row,
which is trivially undone by re-adding, and is the one operation you repeat
enough that a confirm would be worse.
