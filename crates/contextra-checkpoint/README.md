# contextra-checkpoint

Persistent Checkpoint Engine, Hardlink Cloner und Blake3 Manifest-Verifikation.

Normativitätsklasse: **B (Storage Bundle)** (Ring 1, `stable`; SPEC v4.0 §5).

`contextra-checkpoint` verwaltet Time-Travel Checkpoints, Snapshot-Exporte, Transaktions-Guards und Snapshots für Contextra (Ring 1).

## Zweck

Ermöglicht deterministische Wiederherstellung von Zuständen, konsistente Snapshot-Exporte und RAII-gesicherte Transaktions-Rollbacks über eine saubere Port-Schnittstelle (`StorageEngine`, `Vfs`, `Clock`, `IdGen`) ohne globalen Zustand.

## Standalone Copy-Paste-Anleitung (Gate G3)

Um diesen Baustein in einem externen Projekt autark zu nutzen:

```bash
python3 tools/export_crate.py contextra-checkpoint --out /pfad/zu/deinem/projekt/contextra-checkpoint
```

## Externe Abhängigkeiten

- `blake3`: Kryptografische Prüfsummen für Manifest-Integrität.
- `bytes`: Pufferverwaltung für I/O.
- `parking_lot`: Schnelle, blockierungsfreie In-Memory Mutexe.
- `serde`, `serde_json`: Serialisierung von Manifesten und Metadaten.
- `tokio`: Asynchrone Laufzeit und I/O-Primitiven.
- `tracing`: Strukturierte Diagnose-Logs.

## Ring-Zugehörigkeit & Status

- **Klasse:** Klasse B (Storage Bundle)
- **Ring:** Ring 1 (Persistenz)
- **Status:** 🟢 Stabil / Entkoppelt
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Nicht-Ziele (Out of Scope)

- **Keine juristische Löschlogik:** Enthält keine DSGVO/GDPR-Konzepte, Saga-Orchestrierung oder Deletion-Proofs. Diese verbleiben exklusiv in `contextra-erasure`.
- **Keine Schlüsselpersistenz:** Daten-Checkpoints enthalten niemals kryptografische Schlüsselmaterialien (Zwei-Spuren-Prinzip).
- **Keine direkte Dateisystem-Bindung:** Alle I/O-Operationen laufen über die Ports `StorageEngine` und `Vfs`.

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
