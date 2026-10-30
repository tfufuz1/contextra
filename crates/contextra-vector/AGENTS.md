# AGENTS.md — `contextra-vector`

> **Status 30.10.2026 (Reduktion):** Dieses Crate gehört zum Kern v1. Ring und Abhängigkeiten stimmen mit `docs/SPEC.md` §4 überein. Verweise auf entfernte Crates oder Funktionen sind veraltet. Enthält Code für DiskANN (entfernt) und RaBitQ (Nicht-Ziel, offen: D-R4). Maßgeblich ist der Quelltext. Bereinigung: `docs/PLAN.md` Phase R0 bis R3.

## 1. Rolle
`contextra-vector` implementiert speichereffiziente Vektorindexierung und Ähnlichkeitssuche (HNSW, DiskANN, SQ8) mit Mandantenisolation und Tombstone-Pruning.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Vektorindexierung)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-simd` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33, §9.2, §9.3)
- **VETO-F02:** Vektornoten-Löschung und Index-Rebuild laufen als Tombstone-Pruning ohne Erzeugung kryptografischer Löschnachweise.
- **INV-TENANT-IDX-1:** Mandantenisolierte Vektorräume und sichere Partitionierung nach TenantId.
- **REQ-VEC-001:** Eager DiskANN Header Validation (`DISKANN_MAGIC`, Sector Size) vor I/O.
- **Durchsetzungsmechanismus (§27.6):** `#![forbid(unsafe_code)]`, Rebuild-Notifier Tests, Integrationstests.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]`; SIMD-Operationen werden an `contextra-simd` delegiert).
- Keine direkten `Instant::now`-Aufrufe (Zeit über Injected `Clock`-Port).
- Keine Erzeugung von DeletionProofs in Vektor-Löschpfaden.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Implementiert: `VectorIndex` (`HnswIndex`, `DiskAnnIndex`).
- Konsumiert: `Clock` Port, `contextra-simd` Primitiven.

## 6. Tests & Gates
- **Persistence & Rebuild Tests:** Atomic Save & Rebuild-Notify Tests.
- **Fuzzing (REQ-QA-003):** HNSW Persistence Fuzzer.

## 7. Offene Punkte
- keine.
