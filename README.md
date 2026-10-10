# Evidence for chrisle/rbxport#316 (My Tags on players)

Files:

- `exportExt.pdb`: from a rekordbox 7 USB export of this library
  (2026-10-04). 56 My Tag rows (3 categories); type 4 has 572 rows, one per
  track-tag assignment (572 assignments across 48 tracks in master.db).
- `rb-mytag-link.pcapng` (capture 1) and `rb-mytag-link-2.pcapng`
  (capture 2): rekordbox 7 (Windows VM, EXPORT mode) serving My Tags to an
  XDJ-RX3 (firmware 1.20, rear USB-B passed through to the VM) over PRO DJ
  LINK, 2026-10-10.

Both captures were taken in the guest with `pktmon` (all components, full
packets) and converted with `pktmon etl2pcap`. pktmon logs a packet once
per component it passes, so frames repeat; deduplicate by TCP sequence
number. "No events lost" both times. RX3 = 169.254.222.109, rekordbox =
169.254.251.26.

| Capture | Port query (TCP 12523) | db server session |
|---|---|---|
| 1 | ~59.2 s | TCP 64053, ~59.3 s for ~115 s |
| 2 | ~34.1 s | TCP 60619, ~34.2 s for ~426 s |

The analysis below is ours, from decoding the db server messages (the usual
`11 87 23 49 ae` framing). Request names in quotes are our reading; rbx's
existing names are used where it has one. `r:m:s:t` is the usual first
argument (e.g. `184615937` = 0x0B010401).

## Requests involved

| Request | Args (after `r:m:s:t`) | Meaning (ours) | In rbx? |
|---|---|---|---|
| `0x1015` | `0, cat, 0` | "My Tag menu": `cat` 0 = the categories; `cat` n = the tags in category n. Reply `0x4000 [0x1015, count]`, then the rows via `0x3000` (render menu) as for any menu | no |
| `0x3407` | `0, cat, tag_id, on` | tick (`1`) or untick (`0`) one tag in category `cat`; reply `0x4000 [0x3407, 0]` | no |
| `0x3307` | `0, cat, on, mode` | turn category `cat`'s condition on (`1`) / off (`0`); `mode` **16 = AND, 17 = OR** between the ticked tags in that category; reply `0x4000 [0x3307, 0]` | no |
| `0x3007` | `on` | filter master switch | yes (`FILTER_SWITCH`) |
| `0x3107` | | filter state (`FCND` records) | yes (`FILTER_GET`) |
| `0x3207` | `6, 0, 12, blob` | set a filter condition; here type 6 = BPM, blob `01 06 02 00` + low and high BPM×100 as u32 LE | yes (`FILTER_SET`) |

Rows in the `0x1015` menus (`0x4101` items):

- categories: label = category name, id field = category index (1, 2, 3),
  last numeric field 16;
- tags: label = tag name, id field = the `djmdMyTag` id (e.g. reese =
  108516952), last numeric field 0;
- a ticked tag's row has one numeric field changed from 74 to 75 (how the
  deck draws the tick).

With conditions on, rekordbox does the filtering: the track lists the deck
asks for (`0x1105` playlist, `0x1004` all tracks) simply come back shorter.

The `0x3107` filter-state reply did not change when My Tags were ticked or
applied; only the BPM record (type 6) changed, with its enabled byte and
range. So the My Tag state does not appear to live in that reply.

## Capture 1: one tag per category

Library at the time: playlist with 41 tracks. Times are seconds from the
start of the capture.

| t | Deck action | Messages | Tracks in list |
|---|---|---|---|
| 88.5 | open Track Filter > My Tag | `0x1015 [0,0,0]` -> Sub-genre, Traits, Vocals; `0x1015 [0,1,0]` -> 5 tags; `[0,2,0]` -> 38; `[0,3,0]` -> 7 | 41 |
| 91.0 | tick reese (Traits) | `0x3407 [0, 2, 108516952, 1]` | 41 |
| 97.0 | apply | `0x3307 [0, 2, 1, 16]` | **30** |
| 100-113 | filter off/on several times | `0x3007 [0]` / `[1]` | 41 / 30 |
| 117.0 | Vocals category on | `0x3307 [0, 3, 1, 16]` | 30 |
| 120.9 | tick mc-bars (Vocals) | `0x3407 [0, 3, 199608653, 1]` | **1** |
| 142.7 | switch to OR | `0x3307 [0, 2, 1, 17]` | 1 |
| 163.7 | Vocals off | `0x3307 [0, 3, 0, 16]` | 30 |
| 165.3 | Traits off | `0x3307 [0, 2, 0, 17]` | 41 |

Categories combine with AND (reese AND mc-bars = 1). The OR switch made no
difference here because each category had one tag ticked.

## Capture 2: two tags in one category, untick, All Tracks, BPM

| t | Deck action | Messages | Tracks in list |
|---|---|---|---|
| 66.1 | Traits on, AND | `0x3307 [0, 2, 1, 16]` | 41 |
| 67.1 | tick reese | `0x3407 [0, 2, 108516952, 1]` | 30 |
| 67.8 | tick wubby | `0x3407 [0, 2, 242910998, 1]` | **21** (reese AND wubby) |
| 99-121 | filter off/on | `0x3007` | 41 / 21 |
| 135.9 | switch to OR | `0x3307 [0, 2, 1, 17]` | **35** (reese OR wubby) |
| 225.1 | untick wubby | `0x3407 [0, 2, 242910998, 0]` | **30** |
| 262.5 | browse TRACK (all tracks) | `0x1004` | 130 unfiltered / **35** filtered |
| 305-315 | set a BPM condition | `0x3207` (ranges tried while adjusting) | 0 ... 35 |
| ~360 | BPM 170 +/- 2% (deck shows 166.6-173.4) | `0x3207 [6, 0, 12, 01060200 14410000 bc430000]` = 166.60-173.40 | **7** (reese AND BPM) |
| 371-460 | filter off/on | `0x3007` | 130 / **7**: conditions kept |

So: tags within a category combine by the category's mode (16 AND, 17 OR),
categories combine with AND, and My Tag conditions combine with the other
filter conditions (BPM) with AND. Conditions survive the master switch.
