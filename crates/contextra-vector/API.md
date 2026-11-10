# contextra-vector Öffentliche API

## Kern-Schnittstellen
- `HnswIndex`: Approximativer Nächste-Nachbarn-Graph-Index (HNSW).
- `HnswConfig`: Konfiguration mit Parametern (M, ef_construction, ef_search, dimension, distance_metric).
- `Index`: Baukasten Klasse-A Trait für Vektor-Indizierung (`insert`, `remove`, `search`).
- `Maintain`: Trait für Index-Wartung (`compact`, `vacuum`).
- `DistanceKernel` / `ScalarDistance`: Skalare Referenz und Kernel-Abstraktion.
- `HnswHeader`: Serialisierungs-Header mit Versionierung und BLAKE3-Prüfsumme.
- `MmapIndex`: Memory-Mapped Index-Reader.
