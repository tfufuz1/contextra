# contextra-checkpoint

Persistent Checkpoint Engine, Hardlink Cloner und Blake3 Manifest-Verifikation für Contextra Storage.

## Zweck
`contextra-checkpoint` verwaltet Time-Travel Checkpoints, konsistente Snapshot-Exporte, RAII-gesicherte Transaktions-Rollbacks und Orphan-Recovery für heterogene Storage-Engines über standardisierte Port-Schnittstellen (`StorageEngine`, `Vfs`, `Clock`, `IdGen`).

## Reifegrad
- **Status:** Stabil / Produktionsreif (P1–P6 erfüllt)
- **Zero-Panic-Doktrin:** Vollständig typisierte Fehler (`CheckpointError`), kein `unwrap`/`expect` im Produktionscode.
- **Transaktions-Garantien:** Deterministische Rollbacks bei Panic/Drop über `CheckpointGuard`, Inodes-Erkennung bei Hardlink-Klonen.

## Interne Abhängigkeiten
`contextra-checkpoint` hat minimale interne Abhängigkeiten und hängt ausschließlich an Typen- und Port-Definitionen:
- `contextra-types`: Kern-Typen (`TxId`, `Seq`, `Result`, etc.)
- `contextra-ports`: Port-Schnittstellen (`StorageEngine`, `Vfs`, `Clock`, `MetricsSink`, etc.)
- `contextra-testkit` (nur `[dev-dependencies]`): Für Integrationstests und Mocks (`InMemoryStorageEngine`).
- Keine Abhängigkeiten zu höheren Ringen (`graph`, `vector`, `db`, `engine`, `store`).

## Features
- `default = []`: Reines Standard-Build ohne optionale Features.

## Minimalbeispiel
```rust,ignore
use std::sync::Arc;
use contextra_checkpoint::{PersistentCheckpointStore, StateCheckpoint, TxId};
use contextra_ports::StorageEngine;

async fn run<S: StorageEngine>(storage: Arc<S>) -> Result<(), Box<dyn std::error::Error>> {
    let store = PersistentCheckpointStore::new(storage);
    let cp = StateCheckpoint::new(TxId::new(1), "checkpoint-1");
    store.create_checkpoint(cp).await?;
    let loaded = store.get_checkpoint("checkpoint-1").await?;
    assert!(loaded.is_some());
    Ok(())
}
```

## Entnahme-Rezept
Dieser Baustein kann autark in beliebige Rust-Projekte mit einer Port-kompatiblen Storage-Engine eingebunden werden:
1. Kopiere den Ordner `crates/contextra-checkpoint` in dein Ziel-Repository.
2. Binde die Crates `contextra-types` und `contextra-ports` ein.
3. Ergänze die Abhängigkeit in deiner `Cargo.toml`:
```toml
[dependencies]
contextra-checkpoint = { path = "../contextra-checkpoint" }
```

## Bekannte Grenzen
- Checkpoint-Namen sind auf gültige ASCII/UTF-8 Bezeichner ohne Pfadtrennzeichen (`/`, `\`) und maximal 256 Bytes begrenzt.
- Hardlink-Cloning setzt ein Dateisystem voraus, das Hardlinks unterstützt; bei Cross-Device-Links erfolgt ein deterministischer Fallback auf Copy.
