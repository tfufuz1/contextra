# Contextra — Architekturübersicht

**Version:** 1.0 (SOLL-Zustand 14.10.2026) · **Master Spec:** [`docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md`](docs/SOLL_ZUSTAND_SPEZIFIKATION_1.0.md)

---

## 1. Ring-Modell (§4.1)

Contextra verwendet genau **ein** Schichtenmodell (Ringe 0 bis 4 plus Tooling):

```
Ring 0:  types < ports < wire < sys < durable-fs < simd < crypto-core < mvcc < text < rank
              < vector < adapt < graph < crypto-kv < proof-verify < audit-export < avv-generator
Ring 1:  store < checkpoint < kvcache
Ring 2:  sandbox, infer-candle, infer-ollama, infer-onnx   (untereinander unabhängig)
Ring 3:  privacy < engine < erasure-evidence < db < cognition < router < agent
Ring 4:  license < contextra < mcp, py
```

| Ring | Rolle | Crates |
|---|---|---|
| **Ring 0** | Core Primitives & Kernel | `types`, `ports`, `wire`, `sys`, `durable-fs`, `simd`, `crypto-core`, `mvcc`, `text`, `rank`, `vector`, `adapt`, `graph`, `crypto-kv`, `proof-verify`, `audit-export`, `avv-generator` |
| **Ring 1** | Durable Storage & Caching | `store`, `checkpoint`, `kvcache` |
| **Ring 2** | Isolation & External Inference | `sandbox`, `infer-candle`, `infer-ollama`, `infer-onnx` |
| **Ring 3** | High-Level Execution Engine | `privacy`, `engine`, `erasure-evidence`, `db`, `cognition`, `router`, `agent` |
| **Ring 4** | Facade, Extensions & Entrypoints | `license`, `contextra`, `mcp`, `py` |

---

## 2. Unsafe-Inseln (REG-04)

Inseln mit `unsafe`:
1. `contextra-sys` (OS/mmap, AlignedBuf)
2. `contextra-simd` (SIMD Vector-Kernel)
3. `contextra-wire` (FlatBuffers generierter Code)

Alle übrigen Crates tragen `#![forbid(unsafe_code)]`.

---

## 3. Product Hülle EP1 (§4.4)

EP1 Contextra Memory erzwingt eine maximale Hüllengröße von 18 Crates (Ziel 17 Crates) im Default-Build (`cargo tree -p contextra-mcp -e normal --no-default-features --features fast`).
