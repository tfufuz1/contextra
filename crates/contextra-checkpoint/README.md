# contextra-checkpoint

Persistent Checkpoint Engine und Blake3 Manifest-Verifikation.

Normativitätsklasse: **Kern `[K]`** (Ring 1, `stable`; §6.0, `capabilities.toml`).

`contextra-checkpoint` verwaltet Time-Travel Checkpoints, Transaktions-Guards und Snapshots für Contextra (Ring 1).

## Zweck

Ermöglicht deterministische Wiederherstellung von Zuständen, Session-Snapshots und RAII-gesicherte Transaktions-Rollbacks über eine saubere Port-Schnittstelle ohne globalen Zustand.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 1 (Persistenz)
- **Status:** 🟢 Stabil / Entkoppelt
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Abgrenzung: CheckpointGuard vs. Erasure RestoreGuard

- **`contextra-checkpoint::CheckpointGuard`:**
  - Kapselt die Transaktions-RAII-Grenze für operative Agenten-Schritte oder Transaktionen.
  - Erzwingt explizites `commit()` oder `rollback()`.
  - Bei unkontrolliertem Abbruch (`drop` ohne Commit/Rollback) erfolgt die Vormerkung in der instanzgebundenen `InstanceOrphanRegistry` zur sauberen Wiederanlauf-Bereinigung.
  - Besitzt **keinerlei** Lösch- oder Nachweissemantik.

- **`contextra-erasure::restore::RestoreGuard`:**
  - Dient als Schutzbarriere gegen das unbemerkte Wiederauftauchen gelöschter Daten (Resurrection-Schutz, INV-NO-RESURRECTION-1, INV-RESTORE-HORIZON-1).
  - Prüft den Wiederherstellungshorizont gegen den kryptografischen Erasure Ledger.

## Baukasten-Eigenschaften

- **Minimale Abhängigkeiten:** Hängt zur Laufzeit nur an `contextra-types` und `contextra-ports`.
- **Keine Fremd-Dev-Kanten:** Tests laufen rein über `contextra-testkit` (`InMemoryStorageEngine`, `FaultVfs`).
- **Injizierte Umwelt:** Zeit über `Clock`, Dateisystem über `Vfs`.

## Öffentliche API-Übersicht

- **Snapshot Guards:** `CheckpointGuard`, `PinGuard`
- **Manifest & Metadaten:** `CheckpointManifest`, `CheckpointMeta`, `StateCheckpoint`
- **Registry & Store:** `PersistentCheckpointStore`, `CheckpointRegistry`
- **Cloner:** `CheckpointHardlinkCloner`, `DefaultHardlinkCloner`, `SnapshotPinner`

## Minimalbeispiel (Benutzung auf InMemoryStorageEngine)

```rust
use contextra_checkpoint::PersistentCheckpointStore;
use contextra_testkit::InMemoryStorageEngine;
use contextra_types::TxId;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = Arc::new(InMemoryStorageEngine::new());
    let store = PersistentCheckpointStore::open(storage, "example_namespace").await?;

    let meta = store.create_checkpoint(
        "snapshot_1",
        "collection_a",
        1,
        TxId::new(TxId::INTERNAL_BASE + 1),
        json!({"phase": "initial"}),
    ).await?;

    println!("Created checkpoint: {}", meta.name);
    Ok(())
}
```
