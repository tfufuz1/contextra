# API Dokumentation · contextra-engine

## Hauptkomponenten
- `Contextra`: Hauptfassade zur Datenbankinstanz (`open`, `open_with_config`, `collection`, `close`).
- `Collection`: Verwaltung von Dokumenten, Vektor-, Text- und Graphindizes sowie CRUD-Operationen (`insert`, `update`, `delete`, `get`, `query`, `relate`).
- `HybridQueryBuilder`: Fluent Query Builder für Multi-Signal-Suchen (Vektor, BM25-Text, Graph-Traversal) mit SignalFusionStrategies (RRF).
- `DbTransaction`: 2-Phasen-Commit-Koordinator für Konsistenz über LSM-Store und Hilfsindizes mit Kompensationsmechanismus (`rollback`).
- `ContextraConfig`: Zentrale Konfiguration für Dimension, Speichergrenzen und Systemparameter.
