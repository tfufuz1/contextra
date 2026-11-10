# contextra-vault

Ephemerer, speichergesicherter Chunk- und Kontext-Tresor (Ebene 1, Kern-Closure `{contextra-sys}`).

## Zweck
`contextra-vault` stellt flüchtige In-Memory-Speicherstrukturen bereit, die sensible Zwischendaten und Kontext-Chunks aufnehmen, via `contextra-sys::vault::LockedRegions` vor Swap schützen (`mlock`) und beim Verwerfen oder regulären Drop via `Zeroize` zuverlässig kryptographisch nullen.

## Architektur & Abhängigkeiten
- **Ebene 1**: Kern-Closure enthält ausschließlich `{contextra-sys}`.
- **Speichersicherheit**: `#![forbid(unsafe_code)]` im gesamten Crate.
- **Tatsächliche Abhängigkeiten (Kern)**:
  - `contextra-sys` (Pfad-Abhängigkeit, Ring 0)
  - `thiserror` (Fehlerbehandlung `VaultError`)
  - `zeroize` (ZeroizeOnDrop und Zeroize)
  - `tracing` (Warnmeldungen bei ungesichertem Swap)
- **Kompatibilität**:
  - `workspace`: optionales Feature, bindet `contextra-types` isoliert unter `src/compat/` für Konvertierungen ein.

## Invarianten
- **INV-VAULT-1**: `purge()` nullt alle Chunk-Inhalte via `ZeroizeOnDrop` vor der Freigabe.
- **INV-VAULT-2**: `ingest()` schreibt niemals auf persistenten Speicher oder in Logs.
- **INV-VAULT-3**: Kapazitätsüberschreitung führt zu `VaultError::CapacityExceeded` (Fail-Closed, kein Disk-Spillover).
