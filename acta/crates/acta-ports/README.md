# acta-ports

Abstrakte Port-Schnittstellen und Trait-Definitionen für Entkopplung.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, Anhang A, `capabilities.toml`).

## Rolle & Zweck
`acta-ports` ist das Schnittstellen-Fundament des modularen Baukastensystems. Es definiert alle abstrakten Port-Traits (VFS, Storage, Vektor-Index, Volltext, Inferenz, Policy, Bereinigung/Purger), über die die einzelnen Lego-Bausteine miteinander kommunizieren.

Durch die Verwendung von Ports existieren keine direkten Abhängigkeiten zwischen konkreten Speicher- und Index-Engines.

## Baukasten-Einordnung & Status
- **Ring:** Ring 0 (Port-Fundament, deterministisch)
- **Status:** 🟢 Autarker Baustein (Contract M1-M11)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Abhängigkeiten:** Ausschließlich `acta-types` (Normal- und Build-Kanten).
- **Features:** `contract-tests`, `testkit` (Default: minimal).

## Modul-Übersicht & Sub-Preludes
- **`env`:** Injektion der Umgebung (`Clock`, `Rng`, `IdGen`, `MetricsSink`). Sub-Prelude: `acta_ports::env::prelude::*`.
- **`fs`:** Virtuelles Dateisystem (`Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia`). Sub-Prelude: `acta_ports::fs::prelude::*`.
- **`storage`:** Key-Value- und LSM-Speicherabstraktionen (`StorageEngine`, `StorageStats`, `KvPrefixStore`, `MAX_SCAN_MERGE_ACCUMULATOR`). Sub-Prelude: `acta_ports::storage::prelude::*`.
- **`index`:** Vektor-, Volltext- und Graph-Indizes (`VectorIndex`, `TextIndex`, `GraphIndex`). Sub-Prelude: `acta_ports::index::prelude::*`.
- **`inference`:** Modell-, Embedding- und Reranker-Ports (`TextEmbeddingEngine`, `EmbeddingProvider`, `Reranker`, `QueryRewriter`). Sub-Prelude: `acta_ports::inference::prelude::*`.
- **`policy`:** Governance- und Autorisierungsports (`LicenseGate`, `EgressPolicy`, `EgressDecision`). Sub-Prelude: `acta_ports::policy::prelude::*`.
- **`erasure_legacy`:** Isolierte Übergangsmodule für Lösch- und Scanschnittstellen (gemäß Entscheidung D-E1 vor Auslagerung nach `acta-erasure-spi`).

## Einbettungsbeispiel
Ein eigenständiges Beispiel befindet sich unter `examples/standalone.rs`:
```bash
cargo run -p acta-ports --example standalone
```
