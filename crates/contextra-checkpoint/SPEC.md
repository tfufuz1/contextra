# SPEC.md — `contextra-checkpoint`

**Klasse:** B (Storage Bundle)
**Normative Architektur:** SPEC v4.0 §5 (Klasse B), Ring 1
**Verantwortung:** Point-in-Time-Checkpoints, konsistente Snapshot-Exporte, Zwei-Spuren-Backup (Daten getrennt von Schlüsseln), RAII-Guards und deterministisches Orphan-Reaping.

---

## 1. Verwendungszweck & Scope

`contextra-checkpoint` bietet atomare Point-in-Time-Checkpoints und Transaktions-Guards über generische `StorageEngine`- und `Vfs`-Ports.
- **Daten ohne Schlüssel:** Checkpoints exportieren SSTables, MemTables oder Indexdaten. Verschlüsselungsschlüssel werden getrennt im Key-Vault / Keyring verwaltet und sind niemals Teil des Daten-Snapshots.
- **Keine Lösch- oder Saga-Logik:** Die Crate enthält keine juristischen Löschmechanismen, D-01-Säulen oder Deletion-Proofs. Diese verbleiben exklusiv in `contextra-erasure`.

---

## 2. Ports & Schnittstellen

### 2.1 Konsumierte Ports (`contextra-ports`)
- `StorageEngine`: Persistieren und Laden von Checkpoint-Manifesten, Pinning/Unpinning von Sequenznummern (`pin_checkpoint`, `unpin_checkpoint`), atomare Commit-/Rollback-Operationen.
- `Vfs`: Dateisystem-Operationen für Hardlink-Klone und Orphan-State-Persistenz ohne ungeprüfte I/O-Fehler.
- `Clock`: Deterministische Zeitbasis für Metadaten-Zeitstempel (`timestamp_ms`).
- `IdGen`: Deterministische Identifikator-Generierung.

### 2.2 Angebotene Schnittstellen
- `PersistentCheckpointStore<S>`: Primäre Koordination für benannte Checkpoints, Listung, Wiederherstellung und Orphan-Bereinigung.
- `CheckpointGuard<S>`: RAII-Wächter für Transaktionen. Ein Drop ohne `commit()` registriert automatisch ein Orphan zur späteren Bereinigung.
- `PinGuard<S>`: RAII-Schutz für gepinnte Sequenznummern während Snapshot-Operationen.
- `CheckpointHardlinkCloner`: Schnelle Snapshot-Erstellung über atomare Hardlinks mit Fallback auf Datenkopie bei Cross-Device-Links (`EXDEV`).

---

## 3. Invarianten (INV-*)

- **INV-CKP-NO-KEYS-1:** Daten-Backups enthalten keine kryptografischen Schlüsselmaterialien.
- **INV-CKP-GUARD-RAII-1:** Ein unvollendeter `CheckpointGuard` erzwingt bei Drop die Registrierung in der `InstanceOrphanRegistry`.
- **INV-CKP-PIN-1:** Ein Checkpoint bleibt gepinnt, bis er explizit freigegeben oder per Reaper bereinigt wird; verhindert vorzeitigen Version-GC.
- **INV-CKP-DETERMINISM-1:** Zeit und IDs stammen ausschließlich aus injizierten Ports (`Clock`, `IdGen`).

---

## 4. Concurrency- und Durability-Garantien

- **Zero Unsafe:** `#![forbid(unsafe_code)]` wird strikt durchgesetzt.
- **Zero Panic:** Alle Fehler werden typisiert über `ContextraError` bzw. `Result` propagiert.
- **Locking-Disziplin:** Synchrone Mutexe schützen nur kurze In-Memory-Zustände; keine Locks über `.await`-Aufrufe.

---

## 5. Fehlermodell

Alle Fehler kapseln sich in `contextra_types::error::ContextraError`:
- `ContextraError::NotFound`: Checkpoint mit dem angegebenen Namen oder TxId existiert nicht.
- `ContextraError::InvalidState`: Korrupte Manifeste, unerlaubte Whitespaces oder ungültige Komponenten.
- `ContextraError::Storage`: Fehler bei I/O- oder StorageEngine-Operationen.
