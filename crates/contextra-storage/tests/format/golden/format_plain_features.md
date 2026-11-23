# Golden `format_plain_features`

FORMAT, Modus Plain, feature_req = ENCRYPTED|SEAL_CLUSTER, feature_opt = 1<<32 (Pflicht-/Optionalbit-Layout). Länge: 53 B. Erzeugt von `tools/reference/generator.py`; chk nach Annahme A-1.

| Feld | Offset | Länge | Bytes (hex) |
|---|---|---|---|
| `magic` | 0 | 4 | `43545853` |
| `format_ver` | 4 | 2 | `0100` |
| `feature_req` | 6 | 8 | `0500000000000000` |
| `feature_opt` | 14 | 8 | `0000000001000000` |
| `integrity_mode` | 22 | 1 | `00` |
| `instance_id` | 23 | 16 | `0f0e0d0c0b0a09080706050403020100` |
| `algo_set` | 39 | 2 | `0100` |
| `created_ms` | 41 | 8 | `0000000000000000` |
| `chk` | 49 | 4 | `14c138e7` |

```
43545853010005000000000000000000000001000000000f0e0d0c0b0a090807060504030201000100000000000000000014c138e7
```
