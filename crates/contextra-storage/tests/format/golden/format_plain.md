# Golden `format_plain`

FORMAT, Modus Plain, keine Features. Länge: 53 B. Erzeugt von `tools/reference/generator.py`; chk nach Annahme A-1.

| Feld | Offset | Länge | Bytes (hex) |
|---|---|---|---|
| `magic` | 0 | 4 | `43545853` |
| `format_ver` | 4 | 2 | `0100` |
| `feature_req` | 6 | 8 | `0000000000000000` |
| `feature_opt` | 14 | 8 | `0000000000000000` |
| `integrity_mode` | 22 | 1 | `00` |
| `instance_id` | 23 | 16 | `000102030405060708090a0b0c0d0e0f` |
| `algo_set` | 39 | 2 | `0100` |
| `created_ms` | 41 | 8 | `0807060504030201` |
| `chk` | 49 | 4 | `b258592f` |

```
4354585301000000000000000000000000000000000000000102030405060708090a0b0c0d0e0f01000807060504030201b258592f
```
