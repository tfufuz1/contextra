# ADR-RING-crypto-durable-fs: Bewertung der Ring-0-Abhängigkeiten von contextra-vector und contextra-crypto auf contextra-durable-fs

* **Status:** Vorschlag
* **Datum:** 2026-10-12
* **Kontext / Auslöser:**
  Gemäß der Contextra-Architekturspezifikation (§6) und `capabilities.toml` befinden sich `contextra-vector`, `contextra-crypto` sowie `contextra-durable-fs` alle in **Ring 0** (Foundation & Core Domain Logic).
  Im Rahmen des DAG-Gleichgewichts und der Ring-Architektur wurde analysiert, ob die direkten Abhängigkeiten von `contextra-vector` und `contextra-crypto` auf `contextra-durable-fs` (für synchrone, dauerhafte Dateisystem-Operationen wie `atomic_replace`, `durable_remove` und `sync_dir`) durch einen abstrakten Trait/Port aus `contextra-ports` ersetzt werden sollen oder als genehmigte Ring-0-Direktabhängigkeit verbleiben.

## Analyse der Port-Passung in `contextra-ports`
Eine Untersuchung aller Traits und Schnittstellen in `contextra-ports` ergab:
- `contextra-ports` enthält Abstraktionen für Storage (`StorageEngine`), KV-Stores, Clocks, Random Number Generators, Reranker, Embeddings, Graph Indices und Vector Indices.
- Es existiert derzeit **kein** Trait oder Port in `contextra-ports`, der synchrone low-level Dateisystem-Durable-Operationen (`atomic_replace`, `durable_remove`, `sync_dir`) abstrahiert.
- Eine Entkopplung von `contextra-vector` über `contextra-ports` ist daher ohne Neudefinition eines Filesystem-Ports in `contextra-ports` aktuell nicht möglich.

## Entscheidungsoptionen

### Option 1: Einführung eines `DurableFs`-Ports in `contextra-ports`
- **Beschreibung:** Erstellung eines neuen Trait-Interface `DurableFs` (oder `FileSystemPort`) in `contextra-ports`, welches von `contextra-durable-fs` implementiert wird und in `contextra-vector` sowie `contextra-crypto` via Inversion of Control / Dependency Injection genutzt wird.
- **Vorteile:** Vollständige Entkopplung der Ring-0-Kernmodul-Implementierungen von konkreten Dateisystem-Helfern; erleichterte Mockbarkeit in Unit-Tests.
- **Nachteile:** Zusätzliche Abstraktionsschicht für rein synchrone, OS-nahe I/O-Primitive; Aufwand für Trait-Verdrahtung in allen Aufrufern; mögliches Indirektions-Overhead.

### Option 2: Beibehaltung der direkten Ring-0-Abhängigkeit auf `contextra-durable-fs`
- **Beschreibung:** `contextra-vector` und `contextra-crypto` behalten ihre direkte Cargo-Abhängigkeit auf `contextra-durable-fs`.
- **Vorteile:** Zero-Cost-Abstraktion (direkte statische Funktionsaufrufe ohne Dynamic Dispatch / Boxed Futures); `contextra-durable-fs` ist selbst ein fokussiertes, unabhängiges Ring-0-Modul ohne Upward-Dependencies (`may_depend_on = []`); konform mit Spezifikation §6 (Ring-0-Crates dürfen von anderen Ring-0-Crates abhängen, sofern der DAG zyklusfrei bleibt).
- **Nachteile:** Kopplung der Persistence-Helfer in `hnsw` und `diskann` sowie `revocation_log` und `proof_store` an die konkrete `contextra-durable-fs`-Implementierung.

## Empfehlung
**Empfehlung: Option 2 (Beibehaltung der direkten Ring-0-Abhängigkeit).**

*Begründung:*
1. **DAG-Validität:** Sowohl `contextra-vector`, `contextra-crypto` als auch `contextra-durable-fs` sind als Ring-0-Crates klassifiziert. `contextra-durable-fs` hat keinerlei eigene Abhängigkeiten (`may_depend_on = []`). Die Abhängigkeiten `contextra-vector -> contextra-durable-fs` und `contextra-crypto -> contextra-durable-fs` verlaufen daher horizontal innerhalb von Ring 0 und verletzen nicht das Schichtenmodell (DAG läuft nach unten oder bleibt auf gleicher Ring-Ebene ohne Zyklen).
2. **Effizienz und Einfachheit:** Synchrone Dateisystem-Operationen für Index-Persistenz (DiskANN, HNSW) und Revocation-Logs / Proof-Store benötigen direkte, performante Atomic-Replace- und Dir-Sync-Primitive ohne unnötige Dynamic-Dispatch-Indirektionen.
3. **Keine Port-Passung:** Ein passender Port in `contextra-ports` existiert derzeit nicht. Ein künstlicher Port würde die Komplexität erhöhen, ohne funktionalen Mehrwert im Ring 0 zu bieten.

## Konsequenzen
- `crates/contextra-vector/Cargo.toml` behält die Abhängigkeit `contextra-durable-fs`.
- `crates/contextra-crypto/Cargo.toml` behält die Abhängigkeit `contextra-durable-fs`.
- Keine Änderungen an `capabilities.toml` erforderlich.
- In `contextra-vector` nutzen `src/hnsw/durable_fs.rs` und `src/diskann/durable_fs.rs` weiterhin `contextra_durable_fs`.
- Sollte künftig eine Entkopplung für alternative Speichermedien (z. B. WASM-In-Memory-FS) zwingend notwendig werden, kann Option 1 in einer späteren Architekturphase evaluiert werden.
