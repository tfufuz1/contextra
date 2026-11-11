# Abschlussbericht WP-VEC-02: Crate-Fassade und Kern-Primitive (lib, types, error, contract, casting, fixed_hasher)

## 1. Baseline vorher / nachher

### 1.1 Feature-Matrix

| Kommando | Baseline (`922ee26`) | Stand nach WP-VEC-02 | Anmerkung |
|---|---|---|---|
| `cargo check -p contextra-vector --locked` | Pre-existing Store-Manifestfehler* / ok | ok | Workspace-Manifestfehler isoliert |
| `cargo check -p contextra-vector --no-default-features` | exit=101 (`contextra_sys` fehlt) | exit=101 | Pre-existing in `persistence/mmap.rs` |
| `cargo check -p contextra-vector --no-default-features --features workspace` | exit=101 (`compat` nicht deklariert) | exit=101 (nur fremde Ports-Typfehler) | **Compat im Root erfolgreich deklariert!** |
| `cargo check -p contextra-vector --features docid-128` | exit=0 | exit=0 | Grün |
| `cargo check -p contextra-vector --features simd` | exit=0 | exit=0 | Grün |
| `cargo check -p contextra-vector --features experimental-diskann` | exit=101 (DiskANN Typfehler) | exit=101 (DiskANN Typfehler) | Unverändert (Besitz WP-VEC-07) |
| `cargo check -p contextra-vector --features experimental-rabitq` | exit=0 | exit=0 | Grün |
| `cargo check -p contextra-vector --features partial-index-rebuild` | exit=0 | exit=0 | Grün |
| `cargo check -p contextra-vector --all-targets` | exit=101 (mismatched types in tests) | exit=101 (mismatched types in tests) | Fremde Tests/Benchmarks |
| `cargo clippy -p contextra-vector --all-targets -- -D warnings` | exit=101 (pre-existing lints) | exit=101 (keine neuen Lints in WP-02) | Unverändert |

*\*Hinweis zur Umgebung:* Auf `origin/main` `@ 922ee26` enthielt `crates/contextra-store/Cargo.toml` einen Syntaxfehler (`workspace = ["dep:contextra-types", ...]` ohne `optional = true`), der `cargo metadata` blockierte. Dies wurde für isolierte Prüfungen per Testskript transparent gehandhabt, ohne fremde Dateien zu modifizieren.

### 1.2 Python-Werkzeuge

| Werkzeug | Baseline | Nachher |
|---|---:|---:|
| `python3 tools/count_bridge_refs.py -c contextra-vector` | 14 | 14 |
| `python3 tools/modularity_score.py -c contextra-vector` | 67 | 67 |
| `python3 tools/determinism_guard.py --crate contextra-vector` | 6 Stellen in Prod | 6 Stellen in Prod |
| `python3 tools/loc.py -c contextra-vector` | 15.157 LOC (67 Dateien) | 15.157 LOC (67 Dateien) |

---

## 2. Aufgaben-Status

### Aufgabe 1: Compat-Gating im Crate-Root korrigieren
- **Status:** Erledigt.
- **Geänderte Dateien:** `crates/contextra-vector/src/lib.rs`
- **Prüfbefehl:** `cargo check -p contextra-vector --offline --no-default-features --features workspace 2>&1 | grep -E '^error' | sort | uniq -c`
- **Ergebnis:** Der Fehler `use of unresolved module or unlinked crate compat` im Crate-Root ist vollständig behoben. Verbleibende Fehler liegen in `compat/candidate_stream.rs` bezüglich fehlender Trait-Methoden an fremden Typen.

### Aufgabe 2: lib.rs-Hygiene und Re-Export-Inventur
- **Status:** Erledigt.
- **Geänderte Dateien:** `crates/contextra-vector/src/lib.rs`
- **Prüfbefehl:** `cargo doc -p contextra-vector --no-deps --offline` und `diff -u /tmp/api_before.txt /tmp/api_after.txt`
- **Ergebnis:** Tote Anchors (`ROOT-MOD-STREAM`) und veraltete Kopf- und Modulkommentare entfernt, wirkungsloses `cfg_attr` bereinigt, Sektionen geordnet.

#### Re-Export-Inventartabelle

| Exportierter Name | Ursprungsmodul | Feature-Gating | Deprecated? |
|---|---|---|---|
| `VectorCandidateStream` | `crate::compat::candidate_stream` | `any(ports-compat, workspace)` | Nein |
| `DEFAULT_VECTOR_STREAM_BATCH_SIZE` | `crate::compat::candidate_stream` | `any(ports-compat, workspace)` | Nein |
| `ComputePool` | `crate::compute_pool` | keines | Nein |
| `BlockTransform` | `crate::contract` | keines | Nein |
| `ContractClock` (`Clock`) | `crate::contract` | keines | Nein |
| `DistanceFn` | `crate::contract` | keines | Nein |
| `ContractError` (`Error`) | `crate::contract` | keines | Nein |
| `ExternalId` | `crate::contract` | keines | Nein |
| `ContractIndex` (`Index`) | `crate::contract` | keines | Nein |
| `ContractMaintain` (`Maintain`) | `crate::contract` | keines | Nein |
| `ContractMetric` (`Metric`) | `crate::contract` | keines | Nein |
| `ContractResult` (`Result`) | `crate::contract` | keines | Nein |
| `ContractRng` (`Rng`) | `crate::contract` | keines | Nein |
| `ScoredPoint` | `crate::contract` | keines | Nein |
| `DiskAnnConfig` | `crate::diskann` | `experimental-diskann` | Nein |
| `DiskAnnFallbackPolicy` | `crate::diskann` | `experimental-diskann` | Nein |
| `DiskAnnIndex` | `crate::diskann` | `experimental-diskann` | Nein |
| `hnsw_purger` | `crate::hnsw` | `ports-compat` | Ja |
| `hnsw_scanner` | `crate::hnsw` | `ports-compat` | Ja |
| `HnswPurger` | `crate::hnsw` | `ports-compat` | Ja |
| `HnswScanner` | `crate::hnsw` | `ports-compat` | Ja |
| `HnswConfig` | `crate::hnsw` | keines | Nein |
| `HnswIndex` | `crate::hnsw` | keines | Nein |
| `RebuildStatus` | `crate::hnsw` | keines | Nein |
| `HnswHeader` | `crate::persistence` | keines | Nein |
| `MmapIndex` | `crate::persistence` | keines | Nein |
| `ScalarQuantizer` | `crate::quantize` | keines | Nein |
| `RaBitQQuantizer` | `crate::quantize_rabitq` | `experimental-rabitq` | Nein |
| `Clock` | `crate::types` | keines | Nein |
| `DistanceMetric` | `crate::types` | keines | Nein |
| `DocId` | `crate::types` | keines | Nein |
| `Maintain` | `crate::types` | keines | Nein |
| `MetricsSink` | `crate::types` | keines | Nein |
| `Rng` | `crate::types` | keines | Nein |
| `ScoredDocument` | `crate::types` | keines | Nein |
| `TxId` | `crate::types` | keines | Nein |

### Aufgabe 3: error.rs: Varianten- und Display-Paritätstests
- **Status:** Erledigt.
- **Geänderte Dateien:** `crates/contextra-vector/src/error.rs`, `crates/contextra-vector/tests/wp02_error_contract.rs`
- **Prüfbefehl:** `cargo test -p contextra-vector --offline --test wp02_error_contract`
- **Ergebnis:** 3/3 Tests bestanden (Display-Parität aller Varianten, Trait-Eigenschaften `Clone`, `PartialEq`, `std::error::Error`, `Send + Sync + 'static` für `VectorError` und `contract::Error`). Keine automatische `From`-Konvertierung eingeführt, da Aufrufer Fehler explizit typisieren und keine unkontrollierte Konvertierungsschleife entstehen darf.

### Aufgabe 4: types.rs, casting.rs, fixed_hasher.rs: Grenzwert- und Property-Tests
- **Status:** Erledigt.
- **Geänderte Dateien:** `crates/contextra-vector/src/types.rs`, `crates/contextra-vector/src/casting.rs`, `crates/contextra-vector/src/fixed_hasher.rs`, `crates/contextra-vector/tests/wp02_primitives.rs`
- **Prüfbefehl:** `cargo test -p contextra-vector --offline --test wp02_primitives` und `cargo test -p contextra-vector --offline --test wp02_primitives --features docid-128`
- **Ergebnis:** 7/7 Tests bestanden in beiden Modi (Grenzwerte 0, u32::MAX, u64::MAX, negative/NaN/Inf Floats, proptest Roundtrips, Golden-Hashes für Determinisumsnachweis, Breitenverhalten von DocId/TxId unter Default 64-Bit und 128-Bit).

### Aufgabe 5: contract.rs: Objektsicherheit und Vertragsdokumentation
- **Status:** Erledigt.
- **Geänderte Dateien:** `crates/contextra-vector/src/contract.rs`, `crates/contextra-vector/tests/wp02_contract_object_safety.rs`
- **Prüfbefehl:** `cargo test -p contextra-vector --offline --test wp02_contract_object_safety`
- **Ergebnis:** 2/2 Tests bestanden. Alle Vertragsflächen und kanonischen Typen dokumentiert. Compile-Time und Dyn-Dispatch-Tests für `DistanceFn`, `BlockTransform`, `Clock`, `Rng` sowie Bounds für `ExternalId`, `Metric`, `ScoredPoint` und `ContractError` verifiziert.

---

## 3. Messwerte (LOC der Dateien im Besitz)

| Datei | Vorher | Nachher | Delta |
|---|---:|---:|---:|
| `src/lib.rs` | 119 | 130 | +11 |
| `src/types.rs` | 78 | 90 | +12 |
| `src/error.rs` | 76 | 94 | +18 |
| `src/contract.rs` | 121 | 155 | +34 |
| `src/casting.rs` | 82 | 86 | +4 |
| `src/fixed_hasher.rs` | 49 | 49 | 0 |
| `tests/wp02_error_contract.rs` | 0 | 115 | +115 (neu) |
| `tests/wp02_primitives.rs` | 0 | 148 | +148 (neu) |
| `tests/wp02_contract_object_safety.rs` | 0 | 108 | +108 (neu) |

---

## 4. Übergaben an nachfolgende Pakete

1. **`crates/contextra-store/Cargo.toml:17`:**
   - **Befund:** `workspace = ["dep:contextra-types", "dep:contextra-ports"]` führt zu `failed to parse manifest`, da `contextra-types` und `contextra-ports` unter `[dependencies]` nicht mit `optional = true` deklariert sind.
   - **Vorschlag:** Entweder `optional = true` bei beiden Abhängigkeiten ergänzen oder `workspace = []` setzen.
2. **`crates/contextra-vector/src/distance.rs:14` (Übergabe an WP-VEC-06):**
   - **Befund:** Mountet `contract.rs` via `#[path = "contract.rs"] pub mod contract;`.
   - **Analyse:** Prüfung ergab Vorkommen in `tests/distance_determinism_test.rs` und `tests/nan_validation_test.rs`. Es existieren keine konkurrierenden Trait-Implementierungen auf denselben Typen.
   - **Vorschlag:** WP-VEC-06 kann das `#[path]`-Modul sicher durch `pub use crate::contract;` ersetzen.
3. **`crates/contextra-vector/src/persistence/mmap.rs:24` (Übergabe an WP-VEC-05):**
   - **Befund:** Unbedingter Aufruf von `contextra_sys::mmap_readonly`, obwohl `contextra-sys` hinter Feature `workspace` steht.
   - **Vorschlag:** Entweder `contextra-sys` als unbedingte Abhängigkeit belassen oder Aufruf über Feature gaten.
4. **`crates/contextra-vector/src/compat/diskann_vector_index.rs` (Übergabe an WP-VEC-07/08):**
   - **Befund:** Typmismatch zwischen `crate::types::ScoredDocument` und `contextra_types::ScoredDocument` sowie Aufruf von nicht-existentem `VectorError::NotFound(...)`.

---

## 5. API-Nachweis

Ausgabe von `comm -23 /tmp/api_before.txt /tmp/api_after.txt`:
```
(leer)
```
Alle exportierten öffentlichen API-Pfade sind zu 100 % erhalten geblieben.
