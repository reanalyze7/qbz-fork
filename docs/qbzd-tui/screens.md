# Screens

Four screens, one persistent footer. The footer is not decoration: transport
works from every screen, so you never navigate away from what you are doing
to skip a track.

```
┌─ qbz ─────────────────────── Studio · 127.0.0.1:8182 ─┐
│ [1] Now  [2] Search  [3] Library  [4] Browse          │  ← tab strip
├───────────────────────────────────────────────────────┤
│                                                       │
│                    screen body                        │
│                                                       │
├───────────────────────────────────────────────────────┤
│ ▶ From The Start · Laufey      HI-RES 24/48   1:12 ▁▄ │  ← footer
└───────────────────────────────────────────────────────┘
```

## 1 · Now

The landing screen. Current track, position, quality, and the queue below it.

```
│  From The Start                                       │
│  Laufey — Bewitched                                   │
│  HI-RES 24-bit/48 kHz    ▶  1:12 ──────●───── 2:49    │
│                                                       │
├─ Queue (12)  shuffle off  repeat off ─────────────────┤
│  ▸ 1. From The Start        Laufey          2:49      │
│    2. Must Be Love          Laufey          3:04      │
│    3. Promise               Laufey          2:51      │
```

`▸` marks the playing row. `Enter` on any row jumps to it, `d` removes it,
`J`/`K` move it. Empty queue shows what to press, not a blank panel.

Quality comes from the daemon's *delivered* values (`bit_depth`,
`sample_rate`, `bit_perfect_mode`), never from the catalogue's advertised
tier — the two disagree exactly when it matters, which is when the device
forced a downgrade.

## 2 · Search

Type, results stream in by category. `Tab` moves between the result panes.

```
│ /  laufey▌                                            │
├─ Tracks ──────────────────────────────────────────────┤
│    From The Start        Bewitched      HI-RES  2:49  │
│    Valentine             Everything     CD      3:22  │
├─ Albums ──────────────────────────────────────────────┤
│    Bewitched             2023           24/48   13 tr │
├─ Artists ─────────────────────────────────────────────┤
│    Laufey                                             │
```

`Enter` plays, `a` appends to the queue, `→` on an album or artist opens it
in Browse. Results are request/response — see the SSE caveat in the README —
so the screen owns a small debounce and shows a spinner while in flight.

## 3 · Library

Your favorites and playlists, two panes, `Tab` between them. Same `Enter` /
`a` / `→` verbs as Search, on purpose: the verbs are learned once.

```
├─ Favorites ─────────┬─ Playlists ─────────────────────┤
│  Tracks      58     │   Late night            41 tr   │
│  Albums       7     │   Piano                 18 tr   │
│  Artists      3     │   Discover Weekly       30 tr   │
```

## 4 · Browse

An album or artist page, reached from Search or Library. Never a tab you open
cold — it always has a subject, and `Esc` returns to where you came from.

```
│  Bewitched · Laufey · 2023 · 13 tracks · 45:12        │
│  24-bit / 48 kHz                                      │
├───────────────────────────────────────────────────────┤
│    1. Dreamer                               3:12      │
│    2. Second Best                           3:41      │
```

`Enter` plays the track, `P` plays the whole release, `a` appends it, `f`
favorites it.
