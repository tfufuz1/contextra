# Contextra Workspace Audit & Wiring Report

## 1. Complete Crate Inventory & Consumer Audit (Schritt 1a)

| Crate | LOC | Konsumenten | Ring | Verdacht auf Totcode |
|---|---|---|---|---|
| `contextra-store` | 49,443 | 8 (`contextra`, `contextra-agent`, `contextra-checkpoint`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-py`) | Ring 1 | nein |
| `contextra-graph` | 28,188 | 6 (`contextra-agent`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-router`, `contextra-store`) | Ring 0 | nein |
| `contextra-engine` | 26,168 | 2 (`contextra-cognition`, `contextra-db`) | Ring 3 | nein |
| `contextra-vector` | 24,221 | 6 (`contextra-agent`, `contextra-cognition`, `contextra-db`, `contextra-engine`, `contextra-router`, `contextra-store`) | Ring 0 | nein |
| `contextra-crypto` | 15,524 | 9 (`contextra`, `contextra-db`, `contextra-engine`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-privacy`, `contextra-store`, `contextra-vector`) | Ring 0 | nein |
| `contextra-db` | 13,835 | 4 (`contextra`, `contextra-agent`, `contextra-py`, `contextra-router`) | Ring 3 | nein |
| `contextra-text` | 11,901 | 4 (`contextra-agent`, `contextra-db`, `contextra-engine`, `contextra-router`) | Ring 0 | nein |
| `contextra-mcp` | 9,793 | 0 (Leaf Binary/Lib) | Ring 4 | nein |
| `contextra-agent` | 9,342 | 1 (`contextra-mcp`) | Ring 3 | nein |
| `contextra-cognition` | 8,557 | 1 (`contextra-db`) | Ring 3 | nein |
| `contextra-router` | 7,955 | 3 (`contextra`, `contextra-agent`, `contextra-py`) | Ring 3 | nein |
| `contextra-checkpoint` | 7,861 | 3 (`contextra-agent`, `contextra-db`, `contextra-engine`) | Ring 1 | nein |
| `contextra-infer-candle` | 6,996 | 5 (`contextra`, `contextra-cognition`, `contextra-engine`, `contextra-infer-onnx`, `contextra-mcp`) | Ring 2 | nein |
| `contextra-types` | 6,810 | 28 (Ring 0 Type Kernel) | Ring 0 | nein |
| `contextra-adapt` | 6,795 | 7 (`contextra-db`, `contextra-engine`, `contextra-graph`, `contextra-mcp`, `contextra-py`, `contextra-rank`, `contextra-router`) | Ring 0 | nein |
| `contextra-kvcache` | 6,623 | 1 (`contextra-engine` — Bypassed in practice) | Ring 1 | **ja** (Unverdrahteter TenantKvStore) |
| `contextra-rank` | 6,176 | 8 (`contextra`, `contextra-db`, `contextra-engine`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-mcp`, `contextra-py`) | Ring 0 | nein |
| `contextra-infer-ollama` | 5,237 | 2 (`contextra`, `contextra-mcp`) | Ring 2 | nein |
| `contextra-mvcc` | 4,875 | 3 (`contextra-core`, `contextra-engine`, `contextra-store`) | Ring 0 | nein |
| `contextra-privacy` | 4,167 | 3 (`contextra`, `contextra-mcp`, `contextra-router`) | Ring 3 | nein |
| `contextra-ports` | 3,806 | 25 (Ring 0 Abstraction Kernel) | Ring 0 | nein |
| `contextra-sandbox` | 3,021 | 2 (`contextra-engine`, `contextra-store`) | Ring 2 | nein |
| `contextra` | 2,711 | 1 (`contextra-mcp` — Workspace Facade) | Ring 4 | nein |
| `contextra-py` | 2,303 | 0 (Leaf FFI Library) | Ring 4 | nein |
| `contextra-infer-onnx` | 2,300 | 3 (`contextra`, `contextra-infer-ollama`, `contextra-mcp`) | Ring 2 | nein |
| `contextra-wire` | 2,133 | 3 (`contextra-core`, `contextra-mcp`, `contextra-router`) | Ring 0 | nein |
| `contextra-simd` | 1,776 | 1 (`contextra-vector`) | Ring 0 | nein |
| `contextra-license` | 1,536 | 2 (`contextra`, `contextra-mcp`) | Ring 4 | nein |
| `contextra-testkit` | 783 | 4 (`contextra-checkpoint`, `contextra-engine`, `contextra-graph`, `contextra-store`) | Tooling | nein |
| `contextra-sys` | 709 | 4 (`contextra-db`, `contextra-engine`, `contextra-store`, `contextra-vector`) | Ring 0 | nein |
| `contextra-audit-export` | 544 | 1 (`contextra` — Feature-Gated) | Ring 0 | **ja** (Feature-Gated / Mini-Crate) |
| `contextra-avv-generator` | 415 | 1 (`contextra` — Feature-Gated) | Ring 0 | **ja** (Feature-Gated / Mini-Crate) |
| `contextra-core` | 368 | 7 (`contextra`, `contextra-checkpoint`, `contextra-engine`, `contextra-py`, `contextra-simd`, `contextra-store`, `contextra-vector`) | Ring 0 | nein |

---

## 2. Prioritisierte Fundliste (Schritt 3)

| ID | Schweregrad | Crate | Datei & Zeile | Beschreibung / Befund | Beweisstufe |
|---|---|---|---|---|---|
| **FIND-01** | **S1 (Kritisch)** | `contextra-crypto` | `crates/contextra-crypto/src/kv_shredding.rs:227, 260` | **Split-Brain bei KeyRegistry-Widerrufen**: `revoke_group()` und `revoke_record()` aktualisieren nur den RAM-Status, schreiben aber nicht in den `revocation_log`. Bei Neustart gehen alle Widerrufe verloren. | `[BELEGT]` |
| **FIND-02** | **S1 (Kritisch)** | `contextra-kvcache` / `contextra-crypto` | `crates/contextra-kvcache/src/store.rs:292`, `crates/contextra-crypto/src/kv_segment/store.rs:108` | **Kein Cache-Invalidierungs-Event bei Revokation**: Widerrufene Schlüssel bleiben im RAM-KV-Cache lesbar, bis sie zufällig per LRU evictet werden. | `[BELEGT]` |
| **FIND-03** | **S1 (Kritisch)** | `contextra-engine` | `crates/contextra-engine/src/collection/crud/delete.rs:15` | **CRUD Delete ohne Crypto-Shredding**: Standard-Löschung schreibt nur LSM-Tombstones, ruft aber nicht `KeyRegistry::revoke_record` auf. | `[BELEGT]` |
| **FIND-04** | **S2 (Hoch)** | `contextra-mcp` | `crates/contextra-mcp/src/egress_guard.rs:23` | **Duplizierter EgressGuard in Ring 4**: `contextra-mcp` kocht eigene Egress-Filterregeln nach, statt Ring 2 `contextra-privacy` zu nutzen. | `[BELEGT]` |
| **FIND-05** | **S2 (Hoch)** | `contextra-engine` | `crates/contextra-engine/src/decay_controller.rs:64` | **Duplizierter DecayController in Ring 3**: `contextra-engine` hat eine isolierte Kopie des `AdaptiveDecayController` aus Ring 0 `contextra-adapt`. | `[BELEGT]` |
| **FIND-06** | **S3 (Mittel)** | `contextra-kvcache` | `crates/contextra-kvcache/src/store.rs:292` | **Totcode / Unverdrahtetes Subsystem**: `contextra-kvcache` mit Attention-Eviction wird von `contextra-engine` übergangen. | `[BELEGT]` |
| **FIND-07** | **S3 (Mittel)** | `contextra-ports` | `crates/contextra-ports/src/lib.rs` | **Verwaiste Ring-0-Ports**: Traits `Snapshot` und `TextGenerator` besitzen 0 Implementierungen im Workspace. | `[GEMESSEN]` |
| **FIND-08** | **S3 (Mittel)** | Workspace-weit | `crates/*/Cargo.toml` | **No-Op / Unreferenzierte Cargo Features**: 9 Cargo-Features existieren ohne jegliche `#[cfg(feature = "...")]` Referenz im Quellcode. | `[GEMESSEN]` |
| **FIND-09** | **S4 (Niedrig)** | `contextra-store` / `contextra-mcp` | `crates/contextra-store/src/wal/mod.rs:26`, `crates/contextra-mcp/src/proof_key.rs:19` | **Unnötiges `#[allow(dead_code)]`**: Toter Code wird in Produktionsdateien per Attribute unterdrückt. | `[GEMESSEN]` |
