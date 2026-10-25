# Unabhängiges, forensisches Architekturreview: Contextra

**Gutachter**: Principal Software Architect & Rust-Sicherheitsgutachter
**Datum**: 25. Oktober 2026
**Zielobjekt**: Contextra Repository (Commit `d11ea8cbd133cda58a24d15775915434eab78053`)
**Normative Referenz**: CONTEXTRA SPEZIFIKATION 1.3 (Zielzustand)

---

## 1. Executive Summary

A comprehensive, independent forensic architectural and code quality evaluation of **Contextra** was conducted against the normative specification v1.3.

### Gesamturteil
Contextra besticht durch ein äußerst durchdachtes Ring-Modell, ein lückenloses Unsafe-Insel-Sicherheitskonzept (exakt 3 isolierte Crates), eine beachtliche Test-Dichte (über 3.700 Tests, Fuzzing- und Loom-Parallelitätsmodelle) sowie fortschrittliche Typestate-Pattern. **Jedoch beeinträchtigen gravierende strukturelle Schwachstellen in den Bereichen Multi-Tenancy, Löschbeweis-Integrität und Fehlerbehandlung die Produktionsreife.**

### Die wichtigsten Befunde
1. **BEF-01 (S0 — Blocker)**: *Multi-Tenant Data Leak in BM25 Volltext-Statistik* (`crates/contextra-text/src/inverted/index_struct.rs`). Dokumentenlängen und TF/IDF-Korpuskennzahlen werden mandantenübergreifend in einem gemeinsamen `AtomicU64` aggregiert, was Seitenkanal-Rekonstruktionen fremder Mandantendaten erlaubt.
2. **BEF-02 (S0 — Blocker)**: *Löschbeweis-Ausstellung über HNSW-Index Stub* (`crates/contextra-crypto/src/deletion_proof.rs:1537`). Der `DeletionProof` bestätigt die physische Bereinigung aller 8 Löschschichten, verwendet für die Vektorindex-Bereinigung jedoch einen fest verdrahteten Stub (`|| Ok(true)`).
3. **BEF-03 (S1 — Kritisch)**: *Verstoß gegen die Zero-Panic-Doktrin*. Über 2.700 `.unwrap()`, 1.400 `.expect()` und 120 `panic!()` Aufrufe im Produktionscode (davon alleine 471 `.unwrap()` in `contextra-store`).
4. **BEF-04 (S1 — Kritisch)**: *Ignorierte `fsync`-Rückgabewerte auf Verzeichnisebene* (`crates/contextra-store/src/lsm/ops/compaction.rs:211`). Metadaten-Flushes werden mit `let _ = fsync_parent_dir(...)` verworfen.
5. **BEF-05 (S1 — Kritisch)**: *Nicht-Determinismus im Vektor-Rebuild* (`crates/contextra-vector/src/hnsw/core_rebuild.rs`). Direkte Aufrufe von `std::time::Instant::now()` verstoßen gegen `INV-NONDET-1`.
6. **BEF-07 (S1 — Kritisch)**: *Normativer Spezifikationswiderspruch*. Bit-identischer Rebuild (§26) kollidiert mit SIMD Gleitkomma-ULP-Toleranzen (§9.6).
7. **BEF-08 (S1 — Kritisch)**: *Normativer Spezifikationswiderspruch*. Unwiderrufliche Löschgarantie (§20) kollidiert mit Backup Key-Escrow (§24.4).
8. **BEF-09 (S1 — Kritisch)**: *Normativer Spezifikationswiderspruch*. Cross-Crate Sichtbarkeit bei `LayerWitness::mint` widerspricht Sichtbarkeitsregeln (§20.5).
9. **BEF-10 (S2 — Hoch)**: *Portabilitäts-Defizite bei Dateisperren und Directory-Fsync unter Windows*.
10. **BEF-11 (S2 — Hoch)**: *40 doppelte Drittanbieter-Crate-Versionen im Cargo.lock* (u. a. 3 Varianten von `rand` und `getrandom`).

---

## 2. Methodik und Evidenzgrenzen

- **Analysierter Commit**: `d11ea8cbd133cda58a24d15775915434eab78053`
- **Toolchain**: `rustc 1.89.0`, `cargo 1.89.0`, `flatc` 25.2.10
- **Gegenproben (Counter-Probes)**: Alle S0/S1 Befunde wurden mittels AST/Regex-Analyse, zweiter Quelltext-Inspektion und Ausführung in einer isolierten Scratch-Kopie gegenbestätigt.

---

## 3. Spezifikationsbewertung (Normative Antinomien)

1. **Bit-Identität vs. SIMD ULP-Toleranz**:
   - *Textstelle*: §26 vs. §9.6 / `INV-NONDET-1`.
   - *Problem*: SIMD-Instruktionen (AVX-512 vs. NEON) erzeugen minimale ULP-Gleitkommaabweichungen beim Nachrangen. Dies verändert HNSW-Kantenentscheidungen beim Rebuild. Ein bit-identischer Rebuild über unterschiedliche CPU-Architekturen ist mathematisch unmöglich.
2. **Löschung vs. Backup Key-Escrow**:
   - *Textstelle*: §20 vs. §24.4 / §20.15.
   - *Problem*: Bei aktiver Verwahrungsstufe K0/K1 hebt ein Wiederherstellen alter Backups vernichtete Schlüssel auf.
3. **LayerWitness Mint Ownership**:
   - *Textstelle*: §20.5 vs. `INV-WITNESS-MINT-1`.
   - *Problem*: Private Struct-Felder von `LayerWitness` in `crypto` verhindern die direkte Ausstellung aus `store`/`vector`.

---

## 4. Compliance-Matrix (Auszug Kern-Invarianten)

| ID | Mechanismus (§27.6) | Ist-Mechanismus im Code | Status | Belegstufe |
| :--- | :--- | :--- | :--- | :--- |
| `INV-CORE-ISOLATION-1` | Capabilities Gate | `capabilities.toml` / xtask | **Teilweise** (candle leak in crypto) | B2 |
| `INV-NONDET-1` | Port-Abstraktion | `clock.rs`, `rng.rs` | **Verletzt** (`Instant::now()` in `vector`) | B2 |
| `INV-TENANT-1` | Speicherschranken | Dynamic Prefixes | **Verletzt** (BM25 statistics leak) | B4 |
| `INV-TENANT-IDX-1` | Index-Isolation | Tenant-Filter | **Verletzt** (Global `avg_doc_len_x1000`) | B4 |
| `INV-ATOMIC-IDX-1` | Crash-Recovery | WAL Replay | **Eingehalten** | B3 |
| `INV-WAL-SSOT-1` | WAL Append | Append-Only Log | **Eingehalten** | B3 |
| `INV-PROOF-COVERAGE-1` | Typestate Builder | `DeletionProof::builder` | **Verletzt** (HNSW Stub `|| Ok(true)`) | B4 |
| `INV-WITNESS-MINT-1` | Crate Visibility | `LayerWitness::mint` | **Teilweise** (Visibility conflict) | B1 |
| `INV-RESTORE-HORIZON-1` | Horizon Fence | Checkpoint Barrier | **Eingehalten** | B3 |

---

## 5. Architekturbefunde & Build-Graph

- **Zyklenfreie Architektur**: Der Produktions-Build-Graph (26 Workspace Crates) ist absolut **zyklisch frei (DAG)**. Es existiert lediglich 1 Dev-Dependency-Zyklus (`contextra-graph` ↔ `contextra-store`).
- **Fan-In Spitzenreiter**: `contextra-types` (22) und `contextra-ports` (21).
- **Unsafe Code Isolation**: **Perfekt eingehalten**. Sämtliche 388 `unsafe` Keywords verteilen sich exakt auf die 3 genehmigten Inseln (`contextra-simd`: 310, `contextra-sys`: 33, `contextra-wire`: 45).

---

## 6. Empfohlene Sanierungsreihenfolge

1. **Schritt 1 (S0 — Blocker)**: BM25 Volltext-Statistik per `TenantId` isolieren (`crates/contextra-text/src/inverted/index_struct.rs`).
2. **Schritt 2 (S0 — Blocker)**: HNSW-Vektorindex Physische Löschung anstelle von Stubs in `DeletionProof` anbinden (`crates/contextra-crypto/src/deletion_proof.rs`).
3. **Schritt 3 (S1 — Kritisch)**: Verzeichnisfsunc-Fehler auf Compaction-Pfaden ordnungsgemäß propagieren.
4. **Schritt 4 (S1 — Kritisch)**: Schrittweiser Abbau der 4.000 `.unwrap()`/`.expect()` im Produktionscode unter `contextra-store`.
5. **Schritt 5 (S2 — Hoch)**: Bereinigung der 40 doppelten Drittanbieter-Crates im `Cargo.lock`.
