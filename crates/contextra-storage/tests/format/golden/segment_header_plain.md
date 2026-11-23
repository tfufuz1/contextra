# Golden `segment_header_plain`

Segmentkopf Redo-Log, Modus Plain. Länge: 36 B. Erzeugt von `tools/reference/generator.py`; chk nach Annahme A-1.

| Feld | Offset | Länge | Bytes (hex) |
|---|---|---|---|
| `magic` | 0 | 4 | `4354584c` |
| `ver` | 4 | 2 | `0100` |
| `flags` | 6 | 2 | `0000` |
| `segment_id` | 8 | 8 | `8877665544332211` |
| `first_stamp` | 16 | 8 | `0100000000000000` |
| `root_gen` | 24 | 4 | `07000000` |
| `chain_seed` | 28 | 4 | `ddccbbaa` |
| `chk` | 32 | 4 | `e04fdb25` |

```
4354584c010000008877665544332211010000000000000007000000ddccbbaae04fdb25
```
