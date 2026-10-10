# Evidence for chrisle/rbxport#276 (and #274)

`rb-uncoloured-cue-link.pcapng`: rekordbox 7 (Windows VM, EXPORT mode)
serving an XDJ-RX3 (firmware 1.20, rear USB-B passed through to the VM)
over PRO DJ LINK, 2026-10-10. Captured in the guest with `pktmon` (all
components, full packets; frames repeat once per component, so deduplicate
by TCP sequence number), started before the RX3 was attached.

- RX3 169.254.222.109, rekordbox 169.254.251.26.
- Port queries on TCP 12523 at ~76.6 s and ~110.1 s; db server sessions on
  TCP 57102.
- The deck loaded Supershy - Happy Music (Club Edit). Its hot cues as
  rekordbox sent them in the extended cue list (`4e02` reply), colour block
  = code then RGB:

  | pad | code | RGB |
  |---|---|---|
  | A | 22 | 1a ff 00 |
  | B, C, D, F | 42 | ff 00 00 |
  | E | 56 | b3 00 ff |
  | G | 0 (no colour) | 00 00 ff |

  The deck showed pad G blue. The same track's `PCO2` on a rekordbox 7 USB
  export (2026-10-04) also has pad G = code 0, RGB 00 00 ff.
- The capture also has rekordbox's 48-byte kind-`0x16` packet to the RX3's
  UDP 50002 (name `rekordbox`, then `01 01 11`), from ~74.7 s.
