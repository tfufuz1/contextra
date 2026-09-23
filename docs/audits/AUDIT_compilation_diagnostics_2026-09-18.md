# Workspace-weite Kompilierungs-Diagnose (AUDIT Report)

**Auditor:** Google-Jules (Principal Rust Systems Engineer)
**Datum:** 2026-09-18
**HEAD Commit:** `5685639c8b0132af1886aa8f7488ed8a4ab8ccba`
**VERDICT:** FAILED WITH DIAGNOSTICS (6 Harte Kompilierfehler-Muster, 1 Feature-Matrix-Kompilierfehler, Lints/Unsafe-Guard-Interaktionen über downstream Crates, 6 DAG Layer-Inversionen)

---

## 1. Zusammenfassungstabelle pro Crate

| Crate | CHECK | CLIPPY | TEST | FEATURE-MATRIX | ERRORS | DAUER(s) |
|---|---|---|---|---|---|---|
| `contextra-core-ipc-gen` | PASS | PASS | PASS | N/A | 0 | 2 |
| `contextra-core` | PASS | PASS | PASS | `docid-128`: PASS | 0 | 29 |
| `contextra-store` | **FAIL** | SKIP | SKIP | `block-cache-v2`: **FAIL**<br>`fault-injection`: **FAIL** (Kaskade) | 8 | 11 |
| `contextra-crypto` | PASS | **FAIL** | PASS | N/A | 0 (Check) | 153 |
| `contextra-text` | **FAIL** | SKIP | SKIP | N/A | 5 | 1 |
| `contextra-index` | **FAIL** | SKIP | SKIP | `experimental-diskann`: **FAIL** (Kaskade) | 241 | 3 |
| `contextra-graph` | **FAIL** | SKIP | SKIP | `edge-reinforcement-learning`: **FAIL** (Kaskade) | 4 | 4 |
| `contextra-checkpoint` | PASS | PASS | PASS | N/A | 0 | 3 |
| `contextra-calibration` | PASS | PASS | PASS | N/A | 0 | 1 |
| `contextra-sandbox` | PASS | PASS | PASS | N/A | 0 | 64 |
| `contextra-db` | **FAIL** | SKIP | SKIP | N/A | 102 | 28 |
| `contextra-router` | **FAIL** | SKIP | SKIP | `egress-sherman-morrison`: **FAIL** (Kaskade)<br>`bandit-routing`: **FAIL** (Kaskade) | 100 | 10 |
| `contextra-candle` | PASS | PASS | **FAIL** | `kv-bridge`: **FAIL** (Kaskade) | 0 (Check) | 48 |
| `contextra-ollama` | PASS | PASS | PASS | N/A | 0 | 47 |
| `contextra-embed` | PASS | PASS | PASS | N/A | 0 | 72 |
| `contextra-agent` | **FAIL** | SKIP | SKIP | N/A | 100 | 19 |
| `contextra-mcp` | **FAIL** | SKIP | SKIP | `cloud-egress-guard`: **FAIL** (Kaskade)<br>`wasm-sandbox`: **FAIL** (Kaskade) | 100 | 5 |
| `contextra-bench` | **FAIL** | SKIP | SKIP | N/A | 100 | 4 |

---

## 2. Fehler & Warnungen gruppiert nach Datei

### `crates/contextra-store/src/wal/replay.rs` (4 Fehler-Muster / 8 Diagnosen in `contextra-store`)
- **[E0453]** @ `crates/contextra-store/src/wal/replay.rs:105:13`: `allow(unsafe_code) incompatible with previous forbid` (overruled by `-F unsafe-code`)
- **[unsafe_code]** @ `crates/contextra-store/src/wal/replay.rs:118:20`: `usage of an unsafe block` (unterbrochen durch `-F unsafe-code`)
- **[E0453]** @ `crates/contextra-store/src/wal/replay.rs:153:13`: `allow(unsafe_code) incompatible with previous forbid`
- **[unsafe_code]** @ `crates/contextra-store/src/wal/replay.rs:171:24`: `usage of an unsafe block`
  - *Ursache:* `-F unsafe-code` auf Crate-Ebene (oder CLI-RUSTFLAGS) kollidiert mit `#[allow(unsafe_code)]` in `wal/replay.rs`. Da `contextra-store` fehlschlägt, kaskadiert dieser Fehler in alle downstream Crates (`contextra-graph`, `contextra-candle[kv-bridge]`).

### `crates/contextra-text/tests/alloc_profiler.rs` (5 Fehler)
- **[unsafe_code]** @ `crates/contextra-text/tests/alloc_profiler.rs:25:1`: `implementation of an unsafe trait`
- **[unsafe_code]** @ `crates/contextra-text/tests/alloc_profiler.rs:28:5`: `implementation of an unsafe method`
- **[unsafe_code]** @ `crates/contextra-text/tests/alloc_profiler.rs:33:19`: `usage of an unsafe block`
- **[unsafe_code]** @ `crates/contextra-text/tests/alloc_profiler.rs:38:5`: `implementation of an unsafe method`
- **[unsafe_code]** @ `crates/contextra-text/tests/alloc_profiler.rs:42:9`: `usage of an unsafe block`
  - *Ursache:* `alloc_profiler.rs` nutzt `unsafe` für Profiling in Tests, was wegen globaler `-F unsafe-code` Flags beim Bauen aller Targets (`--all-targets`) abgelehnt wird.

### `crates/contextra-index/src/distance.rs` & `hnsw.rs` & `persistence.rs` (241 Fehler)
- **[E0453]** & **[unsafe_code]** @ `crates/contextra-index/src/distance.rs` (193 Diagnosen), `hnsw.rs` (44 Diagnosen), `persistence.rs` (4 Diagnosen)
  - *Ursache:* SIMD-Optimierungen und Memory-Mapping in `contextra-index` enthalten `unsafe`-Blöcke mit local `#[allow(unsafe_code)]`, die von Command-Line `-F unsafe-code` überschrieben werden. Kaskadiert direkt in `contextra-db`, `contextra-router`, `contextra-agent`, `contextra-mcp`, `contextra-bench`.

### `crates/contextra-crypto/src/anti_tamper.rs`, `kv_segment/segment.rs`, `tests/kv_segment_proptests.rs` (5 Clippy-Lints)
- **[unsafe_code]** @ `crates/contextra-crypto/src/anti_tamper.rs:126, 136`, `kv_segment/segment.rs:216, 226`, `tests/kv_segment_proptests.rs:88`
  - *Ursache:* `cargo clippy` aktiviert `-D warnings`, wodurch `unsafe_code` Lints trotz `check` PASS als Clippy-FAIL gewertet werden.

### `crates/contextra-store/src/sstable.rs` (2 Feature-Matrix-Fehler)
- **[E0053]** @ `crates/contextra-store/src/sstable.rs:182:59`: `method weight has an incompatible type for trait`
  - *Hinweis:* Tritt **nur** mit `--features block-cache-v2` auf. Trait-Signatur erwartet `fn(...) -> u64`, Implementierung liefert `u32`.

---

## 3. Kategorisierung aller Befunde

### (a) Harte Kompilierfehler (`cargo check` schlägt fehl)
1. **Unsafe-Guard-Konflikte (`-F unsafe-code` vs `#[allow(unsafe_code)]`):**
   - `crates/contextra-store/src/wal/replay.rs:105, 118, 153, 171` [E0453 / unsafe_code]
   - `crates/contextra-text/tests/alloc_profiler.rs:25, 28, 33, 38, 42` [unsafe_code]
   - `crates/contextra-index/src/distance.rs`, `hnsw.rs`, `persistence.rs` [E0453 / unsafe_code]

### (b) Clippy-Lints (`-D warnings` schlägt fehl)
1. **Unsafe-Deny in `contextra-crypto`:**
   - `crates/contextra-crypto/src/anti_tamper.rs:126, 136`, `kv_segment/segment.rs:216, 226`, `tests/kv_segment_proptests.rs:88` [unsafe_code]

### (c) Deprecated-API-Nutzung
- Keine ungelösten Deprecated-API-Warnungen in den kompilierten Basiskomponenten im aktuellen HEAD (`5685639c8b01`).

### (d) Sonstige Compiler-Warnungen
- Keine direkten Compiler-Warnungen in den erfolgreich gebauten Layer-0/Layer-1 Crates (`contextra-core`, `contextra-checkpoint`, `contextra-calibration`, `contextra-sandbox`, `contextra-ollama`, `contextra-embed`).

---

## 4. Feature-Matrix-Abschnitt (Default-off Feature Failures)

### 1. Feature `contextra-store/block-cache-v2`
- **Command:** `cargo check -p contextra-store --features block-cache-v2`
- **`crates/contextra-store/src/sstable.rs:182:59` [E0053]:** Signature Mismatch in `BlockWeighter`: Trait erwartet Rückgabetyp `u64`, Implementierung liefert `u32`.

### 2. Feature `contextra-core/docid-128`
- **Status:** **PASS** (im vorherigen Audit gefundene Mismatches in `docid-128` wurden im aktuellen HEAD behoben).

---

## 5. DAG-Abschnitt (cargo metadata --dag-check)

Folgende 6 Layer-Inversionen wurden durch `--dag-check` identifiziert:
1. `contextra-store` (Layer 2) hängt von `contextra-crypto` (Layer 3) ab.
2. `contextra-db` (Layer 10) hängt von `contextra-candle` (Layer 12) ab.
3. `contextra-db` (Layer 10) hängt von `contextra-embed` (Layer 14) ab.
4. `contextra-db` (Layer 10) hängt von `contextra-ollama` (Layer 13) ab.
5. `contextra-ollama` (Layer 13) hängt von `contextra-embed` (Layer 14) ab.
6. `contextra-router` (Layer 11) hängt von `contextra-embed` (Layer 14) / `contextra-ollama` (Layer 13) ab.

---

## 6. Abgleich mit früheren Audit-Läufen (`results/20260917_211619` vs `20260918_HEAD`)

1. **`docid-128` Feature Mismatch in `contextra-core`:**
   *Ergebnis:* **BEHOBEN** in HEAD `5685639c8b01`. `--features docid-128` baut nun fehlerfrei.
2. **`relate_bidirectional` E0599 in `benches/relate_bench.rs`:**
   *Ergebnis:* **BEHOBEN** in HEAD `5685639c8b01`.
3. **`clippy::too_many_arguments` in `hnsw.rs`:**
   *Ergebnis:* Überdeckt/blockiert durch Unsafe-Guard-Konflikt in `contextra-index`.
4. **PERSISTENTER Unsafe-Guard-Blocker (`-F unsafe-code` vs `#[allow(unsafe_code)]`):**
   *Ergebnis:* **PERSISTENT & KRITISCH**. Da `verify_workspace.sh` cargo-Befehle mit `-F unsafe-code` ausführt, scheitern alle Crates mit notwendigen/berechtigten `unsafe`-Blöcken (`contextra-store`, `contextra-text`, `contextra-index`).

---

## 7. Priorisierte Kandidatenliste für nachgelagerte FIX-Prompts

### Priorität 1: Harte Kompilierfehler (Unsafe-Guard Governance & Features)
1. `crates/contextra-store/src/wal/replay.rs` — Harmonisierung der Crate/Script-Ebene `forbid(unsafe_code)` Lint-Grenzen für WAL-Mmap/Replay.
2. `crates/contextra-text/tests/alloc_profiler.rs` — Entkopplung des Allocator Profilers für `-F unsafe-code` Testläufe.
3. `crates/contextra-index/src/distance.rs`, `hnsw.rs`, `persistence.rs` — Attribut-Sanitierung (`#[allow(unsafe_code)]` vs `-F unsafe-code`) an SIMD / Persistence-Schnittstellen.
4. `crates/contextra-store/src/sstable.rs` — Feature `block-cache-v2`: Rückgabetyp von `BlockWeighter::weight` von `u32` auf `u64` anpassen.

### Priorität 2: Clippy / Lints (-D warnings)
1. `crates/contextra-crypto/src/anti_tamper.rs`, `kv_segment/segment.rs`, `tests/kv_segment_proptests.rs` — Behebung des `-D warnings` Unsafe-Lint-Fehlers unter Clippy.
