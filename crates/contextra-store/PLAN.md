# contextra-store — Implementierungsplan zum Soll-Zustand

**Normative Basis:** `docs/SPEC.md` §§4–7 und `docs/crates/contextra-store.md`. **Ausgangslage:** LSM/WAL/SSTable sind umfangreich, aber 55 Nicht-Test-Panikmuster und zusätzliche MVCC-/Kompatibilitätskopplung stehen gegen die E1-Core-Closure.

1. Begrenze Abhängigkeiten auf `ids`, `env`, `sys`, `durable-fs`, `crypto`; migriere MVCC-/Port-/E2-Logik in den Integrator und entferne no-op-`workspace`-Features.
2. Stabilisiere WAL, Manifest, MemTable, SSTable und Recovery zuerst mit versionierten Binärformaten und Golden-/Crash-Tests (INV-1/6).
3. Integriere `ReadGate` vor jeder Entschlüsselung und vor jeder Ausgabe von get, Snapshot-get und Scan; bei gesperrtem Ziel sofort `Erased`/auslassen, nie Datenfetzen.
4. Implementiere Layer-Vertrag: `on_gate` sperrt MemTable/Block-Cache, `purge` dräniert WAL, flusht und rewritet gemischte SSTables unter Barriere; `witness` prüft Residuen und Cache.
5. Ersetze sämtliche Produktions-Paniken in fsync, Erasure, WAL und Utilities durch `StoreError`; Key-/RNG-Fehler sind fail-closed.

**Abnahme:** WAL-/Manifest-Recovery-Matrix, T-DEL-1/2/4/6, Gate je Lesepfad, Compaction-Crash-Tests, Formatkompatibilität und Zero-Panic-Gate.
