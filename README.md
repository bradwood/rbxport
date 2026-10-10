# Evidence for chrisle/rbxport#316 (My Tags on players)

- `exportExt.pdb`: from a rekordbox 7 USB export of this library
  (2026-10-04). 56 My Tag rows (3 categories); type 4 has 572 rows, one per
  track-tag assignment (572 assignments across 48 tracks in master.db).
- `rb-mytag-link.pcapng`: rekordbox 7 (Windows VM, EXPORT mode) serving
  My Tags to an XDJ-RX3 (firmware 1.20, rear USB-B passed through) over
  PRO DJ LINK, 2026-10-10. Captured in the guest with `pktmon`, converted
  with `pktmon etl2pcap`; frames repeat once per component they pass.
  - RX3 169.254.222.109, rekordbox 169.254.251.26.
  - Port query TCP 12523 at ~59.2 s; db server session on TCP 64053 from
    ~59.3 s for ~115 s.
  - On the deck: browsed the rekordbox source, held TRACK FILTER/EDIT,
    opened My Tag, filtered by "reese" and a tag from another category,
    with AND and with OR, and browsed the filtered lists.
