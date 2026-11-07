# AGENTS.md — `contextra-vector`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra-vector` stellt speichereffiziente Vektorindexierung und Vektor-Ähnlichkeitssuche (HNSW, DiskANN, SQ8-Quantisierung) mit strikter Mandantenisolation bereit (Ring 0).
Als autarker Baustein implementiert er den standardisierten Port `VectorIndex` aus `contextra-ports`.
Im Zuge der Baukasten-Transformation wurden sämtliche rechtlichen Löschketten- und DSGVO-Relikte entfernt: Das Modul arbeitet rein append- und supersession-basiert (RoaringBitmap-Tombstones) und stellt für `contextra-erasure` lediglich einen neutralen Bereinigungs-Port (`VectorScanner`, `Purger`) bereit.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Vektorindexierung & Approximate Nearest Neighbor)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-simd` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **VETO-F02:** Vektorknoten-Löschung und Index-Rebuild laufen als Tombstone-Pruning ohne Erzeugung kryptografischer Löschnachweise.
- **INV-TENANT-IDX-1:** Strikt mandantenisolierte Vektorräume und sichere Partitionierung nach `TenantId`.
- **REQ-VEC-001:** Eager DiskANN Header Validation (`DISKANN_MAGIC`, Sector Size) vor jedem I/O.
- **I/O über Ports:** Keine direkten `std::fs`-Aufrufe im Index-Kern; Persistenz erfolgt standardisiert über den `Vfs`-Port.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; saubere Fehlerbehandlung über `ContextraError`.

## 4. Verbotene Muster
- Kein `unsafe` (`#![forbid(unsafe_code)]` erzwungen; SIMD-Operationen delegieren strikt an `contextra-simd`).
- Keine direkten `Instant::now`-Aufrufe (Zeitstempel ausschließlich über injizierten `Clock`-Port).
- Keine Erzeugung von DeletionProofs oder DSGVO-Kontexten im Vektor-Pfad.
- Keine Querabhängigkeiten zu höheren Schichten (Ring 2, 3 oder 4).

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Implementiert:** `VectorIndex` (`HnswIndex`, `DiskAnnIndex`), `Purger`, `LayerScanner`.
- **Konsumiert:** `Clock`-Port, `contextra-simd` Primitiven, `Vfs`-Port.

## 6. Tests, Sabotage & Qualitätssicherung
- **Atomic Save & Rebuild Tests:** Verifikation der Index-Persistenz und des Rebuild-Notifiers nach Crash.
- **Fuzzing (REQ-QA-003):** HNSW Persistence- und Vektor-Input Fuzzer.
- **Sabotage-Tests:** Verifikation der Mandantenisolation bei cross-tenant Query-Versuchen.

## 7. Offene Punkte & Technische Schulden
- DEBT-01: Vollständige Bereinigung verbliebener Übergangs-Purger-Signaturen.
