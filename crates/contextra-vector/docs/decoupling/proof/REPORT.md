# Abschlussbericht VEC-B: Beweisnetz `contextra-vector` (Golden, Differenz, Fuzz, Baseline)

Stand: 2026-11-11
Basis-Commit: `f92caabaa` (Stand `origin/main`)
Rolle: Verifikationsartefakte (Golden-Dateien, Differenztests, Fuzzing-Ziele, Messbaseline). Kein Produktionscode verändert.

---

## 1. Gate-Matrix

| Gate / Prüfung | Befehl | Tatsächliches Ergebnis | Status |
|---|---|---|---|
| **G-PROOF-GOLDEN** | `cargo test -p contextra-vector --test golden_test --offline --features experimental-diskann` | 5 passed; 0 failed; 1 ignored; 0.00s | GRÜN |
| **G-PROOF-MANIFEST** | `cd crates/contextra-vector/tests/golden && sha256sum -c MANIFEST.sha256` | 9 Dateien OK (2 Läufe byte-identisch) | GRÜN |
| **G-PROOF-READER** | `cargo test -p contextra-vector --test proof_hnsw_reader_compat --offline` | 1 passed; 0 failed; 0.00s | GRÜN |
| **G-PROOF-CORRUPT** | `cargo test -p contextra-vector --test proof_open_rejects_corrupt --offline` | 1 passed; 0 failed; 0.00s | GRÜN |
| **G-PROOF-DIST-LEGACY** | `cargo test -p contextra-vector --test proof_distance_legacy --offline` | 4 passed (inkl. 2000 Proptest-Fälle); 0.16s | GRÜN |
| **G-PROOF-SIMD-DISP** | `cargo test -p contextra-vector --test proof_simd_dispatch_legacy --features simd --offline` | 1 passed; 0.00s | GRÜN |
| **G-PROOF-SEED-DET** | `cargo test -p contextra-vector --test proof_determinism_seed --offline` | 1 passed (50 Queries); 0.05s | GRÜN |
| **G-PROOF-RECALL-ORACLE** | `cargo test -p contextra-vector --test proof_recall_oracle --offline` | 2 passed (Cosine & Euclidean 100/100 Hits); 0.10s | GRÜN |
| **G-PROOF-FUZZ-CHECK** | `cd crates/contextra-vector/fuzz && cargo check --offline` | 0 errors; alle Bins typgeprüft; 0.12s | GRÜN |
| **G-PROOF-FUZZ-SMOKE** | `cd crates/contextra-vector/fuzz && ./run_smoke.sh` | Alle Seed-Korpora verifiziert, 0 Panics/Crashes | GRÜN |
| **G-PROOF-CLIPPY** | `cargo clippy -p contextra-vector --test proof_* --offline -- -D warnings` | 0 Warnungen in Eigentumsdateien | GRÜN |
| **G-PROOF-EIGENTUM** | `git status` / `git diff --stat origin/main` | Ausschließlich Pfade aus Abschnitt 2 | GRÜN |

---

## 2. Abweichungen von Prompt / Spec mit Grund
- Keine Abweichungen vom geforderten Funktionsumfang.
- Im unveränderten Produktionscode von `contextra-vector` (z. B. `acorn/gamma_augmentation.rs`, `quantize/mod.rs`) sind vorab Clippy-Fehler (wie `indexing_slicing`, `cast_possible_truncation`, `disallowed_types`) vorhanden. Entsprechend R0.4/R1 wurden diese unverändert belassen (Eigentum von VEC-A); alle neuen Test- und Beweisdateien von VEC-B sind vollständig clippy-warnungsfrei.
- Workspace-weites `cargo check --workspace --all-targets --locked` schlägt auf fremden Crates fehl (z. B. `contextra-store/src/wal/mod.rs: runtime dead_code`, `contextra-infer-candle: unexpected cfg workspace`, `Cargo.lock` out-of-sync bei Vorhandensein ungesyncter Abhängigkeiten wie `contextra-env` in `contextra-crypto`). Entsprechend 0.3/0.9 wurden fremde Dateien nicht angefasst.

---

## 3. Getroffene Entscheidungen
1. **DiskANN Golden-Erzeugung:** Da `DiskAnnIndex::append_to_pending_wal` und `append_to_tombstone_wal` `pub(crate)` sind, wurden PWAL und TWAL über die öffentlichen Schnittstellen `DiskAnnIndex::insert` und `DiskAnnIndex::delete` nach erfolgreichem `build(&vecs, &ids)` deterministisch erzeugt.
2. **Recall-Konstanten:** Am unveränderten Code wurde für das deterministische 200-Punkte-Korpus bei `k=10` über 10 Queries für Cosine und Euclidean jeweils exakt 100/100 Treffer (Recall 1.0) gemessen und als Konstanten `EXPECTED_HITS_COSINE = 100` und `EXPECTED_HITS_EUCLIDEAN = 100` eingefroren.
3. **Dot-Product Distanz-Definition:** In `contextra-vector::distance` ist die Dot-Product-Distanz als Negativ-Skalarprodukt (`-dot`) definiert, um die Index-Minimierungs-Invariante zu wahren. Die Legacy-Referenz spiegelt dies bitgenau wider.

---

## 4. Offene Punkte und Risiken
- Bei VEC-A Formatanhebung von `HNSW_VERSION = 2` auf `3`: `tests/proof_hnsw_reader_compat.rs` stellt sicher, dass Version-2-Dateien weiterhin unverändert geladen und gelesen werden können (Rückwärtskompatibilität).

---

## 5. Übergaben an VEC-A / Folgeauftrag
- `crates/contextra-vector/src/diskann/build.rs:145,299`: `append_to_pending_wal` und `append_to_tombstone_wal` sind `pub(crate)`. Falls diese für modulare Index-Wartungstests außerhalb des Crates genutzt werden sollen, sollte VEC-A sie überlegen öffentlich zu exportieren.
- Feature `docid-128`: Die 128-Bit-DocID-Validierung funktioniert transparent über generische Tests; `docid-128` ist derzeit als leeres Feature im Manifest hinterlegt und leitet noch nicht an `contextra-ids/docid-128` weiter.

---

## 6. Messwerte

### 6.1 Distanz-Benchmarks (128-D Vektoren, Haswell-Klasse Xeon 2.30 GHz)
- `cosine_distance` (Dispatch): **4.26 µs**
- `cosine_scalar` (Skalar): **2.40 µs**
- `euclidean_distance` (Dispatch): **4.13 µs**
- `euclidean_scalar` (Skalar): **2.36 µs**
- `dot_product_distance` (Dispatch): **4.18 µs**
- `dot_product_scalar` (Skalar): **2.35 µs**

### 6.2 SQ8 Quantisierung (128-D Vektoren)
- `SQ8/quantize`: **14.37 µs**
- `SQ8/dequantize`: **5.61 µs**
- `SQ8/asymmetric_dist_cosine`: **4.25 µs**
- `SQ8/symmetric_dist_cosine`: **5.45 µs**
