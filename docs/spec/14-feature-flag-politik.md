---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "14"
---
## 14. Feature-Flag-Politik: Produktions-Default vs. Opt-in

Ein Breaking-Change- oder Performance-Trade-off-Feature wird hinter einem Cargo-Feature isoliert, bis eine
explizite Produktentscheidung den Wechsel des Defaults auslöst. Dies ist **kein Mangel**, sondern verbindliche
Politik.

| Feature-Flag | Reifegrad | Beschreibung |
|---|---|---|
| `cloud-egress-guard` | 🟢 | DLP/Egress-Kontrolle, Surrogat-Tokenisierung, Bulk-Exfiltration-Detektor |
| `bandit-routing` | 🟢 | LinUCB-Grundfunktion, Lyapunov-Kopplung, gedeckelte Drift-Eskalation |
| `egress-sherman-morrison` | 🟡 | Mathematisch korrekte Ridge-Regression; einziger Pfad mit LinUCB-Regret-Garantie |
| `kv-bridge` | 🟢 | KV-Cache-Bridge inkl. LSM-Fallback, AES-256-GCM-SIV |
| `wasm-sandbox` | 🟢 | Fuel- und Wall-Clock-Budget orthogonal |
| `experimental-diskann` | 🟢 (Tier) | Native Tombstones, SQ8-Perzentil-Clipping |
| `docid-128` | 🟡 | 128-Bit-BLAKE3-DocId, Rollout vollzogen, Default bleibt `u64` |
| `block-cache-v2` | 🟡 | S3-FIFO-Backend (`quick_cache`), Default bleibt LRU |
| `bm25f` | 🟢 | Feldgewichtete BM25-Bewertung |
| `flatbuffers-drift-gate` (xtask) | 🟢 | CI-Gate gegen Schema-Drift |
| `fault-injection` | 🟢 | Test-only |
| `loom` | Dev | Nebenläufigkeits-Modelltests |
| `adaptive-decay` / `-control` | 🟢 | Kalibrierungs-Feintuning |
| `partial-index-rebuild` | 🟢 | Inkrementeller Indexaufbau |
| `edge-reinforcement-learning` | 🟢 (Gate) | Kantenverstärkung als optionales Fusionsverhalten |
| **Hyperkanten (`relate_n_ary`, `HyperEdge`)** | **🔴** | Nicht implementiert — kein Flag, siehe §6 |

---

<a id="15-tests"></a>
