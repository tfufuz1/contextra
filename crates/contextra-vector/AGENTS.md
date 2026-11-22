# AGENTS.md — `contextra-vector`

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

## Storage-Umbau (`contextra-storage`)

Normativ: `docs/SPEC.md` (R8). Umsetzungsplan und Verträge: `docs/IMPLEMENTATION_SPEC.md` (nicht normativ). Bis Gate G-5 ändert sich dieses Crate nicht (Spec §22); die Angaben hier sind Dokumentation, kein Auftrag zur Codeänderung.

- Rolle: Indexstruktur; `contextra_store` nur in Kommentaren (1 Datei).
- Vertrag IF-6: später `ExtensionBuilder`/Segment (Spec §10.9, §15.9); Pflicht `rewrite_without`.
- Verboten: der Kern kennt keine Vektorindizes.
- Arbeitspakete: nach G-6.
