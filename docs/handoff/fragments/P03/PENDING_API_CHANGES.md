# Pending API Changes AUFTRAG P03: contextra-vector

Format: Symbol | betroffene Crates | gewünschte Änderung

| Symbol | Betroffene Crates | Gewünschte Änderung |
|---|---|---|
| `contextra-vector/experimental-diskann` | `contextra-engine`, `contextra-db` | Entfernung aus externen Konsumenten nach Abschluss der Kern-Stabilisierung (DiskANN wird experimentell gehalten). |
| `hnsw_purger`, `hnsw_scanner` | `contextra-vector` | Veraltete Erasure-SPI-Brückenfunktionen verbleiben als `#[deprecated]` bis Auslagerung nach `contextra-erasure-vector`. |
