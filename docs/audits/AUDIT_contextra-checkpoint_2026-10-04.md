# Architecture Audit: `contextra-checkpoint`

**Audit-Datum:** 2026-10-04
**Auditor:** Principal Senior Rust Architect (`Contextra` Architecture Group)
**Target Crate:** `crates/contextra-checkpoint`
**Claim-Log:** `logs/audits/contextra-checkpoint-claim.log`
**Test-Log:** `logs/audits/checkpoint-test.log`
**Clippy-Log:** `logs/audits/checkpoint-clippy.log`
**Session Hash:** `f133653a`
**HEAD Commit:** `8ff64018a365e945355dff1e328dedc8c942dabf`

---

## 1. Prüfpunkte-Matrix (P1 – P6)

| ID | Prüfpunkt | Status | Audit-Befund / Nachweis |
| :--- | :--- | :---: | :--- |
| **P1** | **INV-CHECKPOINT-DETERMINISM-1** | 🟢 **PASS** | `grep` in `crates/contextra-checkpoint/src/` bestätigt: **Kein** `SystemTime::now()` in Nicht-Test-Code. Alle Checkpoint-Zeitstempel (`timestamp_ms`) werden über die injizierte `Clock`-Port-Schnittstelle (`now_unix_nanos() / 1_000_000`) berechnet. `Instant::now()` wird ausschließlich lokal für Histogramm-Dauer-Metriken (`checkpoint_duration_seconds`) verwendet. Commit `647276c5` injizierte den `Clock`-Port bereits in `Checkpointer` (`contextra-store`). |
| **P2** | **RAII-Guard-Vollständigkeit** | 🟢 **PASS** | `CheckpointGuard` und `PinGuard` sind mit `#[must_use]` annotiert. `CheckpointGuard` kann ausschließlich via `commit()`, `rollback().await` oder `rollback_blocking()` konsumiert werden. Dropped ein Uncommitted Guard (z.B. bei Panic/Unwind), registriert der `Drop`-Handler die Transaktion synchron in `InstanceOrphanRegistry` (`register_checkpoint_sync`). Kein lautloser Drop möglich. |
| **P3** | **Orphan-Reaper-Implementierung** | 🟢 **PASS** | `InstanceOrphanRegistry` verarbeitet verwaiste Pins und Checkpoints entkoppelt von Tokio-Worker-Threads. Der `Drop`-Handler führt strikt synchrone In-Memory-Mutationen aus (`is_dirty`). Die Bereinigung (`unpin_checkpoint`, `rollback_to_tx`) erfolgt bei `PersistentCheckpointStore::open` (Start-Recovery) sowie über Piggyback-Flushes bei nachfolgenden `commit`/`rollback`/`unpin` Operationen. Altlast-Aktionen (`await_pending_rollbacks`) sind gemäß ADR-053 gezielt deprecated. |
| **P4** | **Snapshot Pinning / Unpinning Symmetrie** | 🟢 **PASS** | Bei `drop_checkpoint(name)` wird der Checkpoint zuerst aus dem Storage gelöscht und im direkten Anschluss `storage.unpin_checkpoint(seq_no).await` aufgerufen. Bei `create_checkpoint` wird die neue Sequenznummer vor dem Schreiben gepinnt (`PinGuard::pin`) und nach erfolgreichem Überschreiben der alte Checkpoint entpinnt. |
| **P5** | **Blake3 Manifest Verifikation** | 🟢 **PASS** | `CheckpointManifest::new` berechnet eine Blake3-Hex-Checksumme über die serialisierte Metadaten- und Komponenten-Payload. `CheckpointManifest::verify()` verifiziert die Checksumme beim Laden (`get_checkpoint_internal`, `list_checkpoints`). Bei Manipulation/Korruption wird `ContextraError::Serialization` zurückgegeben. |
| **P6** | **TxId-Bereichs-Separation** | 🟢 **PASS** | System-Checkpoints vergeben TxIds über `allocate_tx()` ab `TxId::INTERNAL_BASE` (`1 << 62` = `4_611_686_018_427_387_904`) aufwärts. Reguläre Dokument/Kanten-Transaktionen nutzen TxIds ab `1`. Konflikte sind durch bitweise Separation ausgeschlossen. `open_with_orphan_registry` prüft und erzwingt diese Untergrenze. |

---

## 2. RAII-Guard-Lifecycle-Nachweis

Der Lifecycle des `CheckpointGuard` ist in `crates/contextra-checkpoint/src/guard.rs` lückenlos abgesichert:

```rust
// Contract: Explicit Finalization via commit() or rollback()
#[must_use = "CheckpointGuard must be explicitly finalized via .commit() or .rollback().await"]
pub struct CheckpointGuard<S: contextra_ports::StorageEngine> {
    pub(crate) checkpoint: Option<StateCheckpoint>,
    pub(crate) storage: Arc<S>,
    pub(crate) orphan_registry: Arc<InstanceOrphanRegistry>,
    pub(crate) skipped_rollbacks: Arc<AtomicU64>,
    pub(crate) created_at: std::time::Instant,
    ...
}
```

### Finalisierungs-Pfade:
1. **Explicit Commit (`commit(self)`)**:
   - Entnimmt den `StateCheckpoint` via `self.checkpoint.take()`.
   - Zeichnet Dauer-Metrik für `status: commit` auf.
   - Spawnt Piggyback-Flush der Orphan-Registry.
   - Guard konsumiert, `Drop` wird zur No-Op.
2. **Explicit Async Rollback (`rollback(self).await`)**:
   - Entnimmt den Checkpoint via `self.checkpoint.take()`.
   - Evaluiert Serialisierungsbarriere (`last_tx > cp.tx_id`).
   - Führt `storage.rollback_to_tx(cp.tx_id)` aus.
   - Guard konsumiert, `Drop` wird zur No-Op.
3. **Explicit Sync Rollback (`rollback_blocking(self)`)**:
   - Erfordert synchronen Kontext. Bei Aufruf im Tokio-Runtime-Kontext wird sofort `ContextraError::Internal` geliefert, um Deadlocks zu verhindern.
4. **Implicit Drop / Panic / Early Return (`drop(&mut self)`)**:
   - Prüft, ob `self.checkpoint` noch `Some(...)` ist.
   - Wenn unkonsumiert: Erhöht `skipped_rollbacks` Counter.
   - Emittert `tracing::error!` Log-Event.
   - Registriert Checkpoint in `self.orphan_registry.register_checkpoint_sync(cp)`.

---

## 3. Orphan-Reaper-Implementierungsstatus

Gemäß **ADR-053** ("Instance-Scoped Orphan Management & Zero-Blocking Drop") wurde das globale, hängende Background-Task-Modell vollständig abgelöst.

- **Non-Blocking Drop**: In `CheckpointGuard::drop` und `PinGuard::drop` finden keine asynchronen `.await`-Aufrufe oder blockierenden OS-Locks/File-I/O-Operationen auf Tokio-Threadpool-Workern statt. `register_checkpoint_sync` und `register_orphan_sync` mutieren strikt In-Memory `Mutex<Vec<...>>` Strukturen und setzen atomic `is_dirty = true`.
- **Startup-Recovery**: `PersistentCheckpointStore::open_with_orphan_registry` stellt beim Starten/Reopen alle unentpinnten Sequenzen wieder her und verarbeitet registrierte verwaiste Pins sofort.
- **Piggyback-Flushes**: `flush_orphan_registry()` flasht den In-Memory-Zustand entkoppelt auf Festplatte, sobald synchrone/asynchrone Transaktionen committet, gerollbackt oder entpinnt werden.
- **Controlled Reaping**: `recover_orphaned_checkpoints()` prüft vor dem Rollback explizit die Serialisierungsbarriere gegen `storage.last_tx_id()`.

---

## 4. Blake3-Verifikations-Nachweis

Die Manifest-Verifikation in `crates/contextra-checkpoint/src/manifest.rs` schützt gespeicherte Snapshots vor silent corruption:

```rust
impl CheckpointManifest {
    pub fn new(meta: CheckpointMeta, components: Vec<String>) -> Result<Self> {
        ...
        let payload = serde_json::to_vec(&(&meta, &components))
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        let checksum = blake3::hash(&payload).to_hex().to_string();
        Ok(Self { meta, components, checksum })
    }

    pub fn verify(&self) -> Result<()> {
        let payload = serde_json::to_vec(&(&self.meta, &self.components))
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        let expected = blake3::hash(&payload).to_hex().to_string();
        if self.checksum != expected {
            return Err(ContextraError::Serialization(format!(
                "Checkpoint manifest checksum mismatch for '{}': expected {}, got {}",
                self.meta.name, expected, self.checksum
            )));
        }
        Ok(())
    }
}
```

Bei jedem Lesevorgang (`get_checkpoint_internal`, `list_checkpoints`) wird `manifest.verify()?` ausgeführt. Ein verfälschtes Byte in der JSON-Datei löst sofort einen Verifikationsfehler aus.

---

## 5. Session Bootstrap & IST-Zustandserfassung (Sequenz)

1. **Session-Identität:** `SESSION: f133653a | TS: 2026-10-04T06:52:18Z`
2. **Blocker/Critical Scan:** 0 Funde in `crates/`.
3. **In-Progress Anchors:** 0 offene `IN-PROGRESS` Anchors.
4. **Working State:** `WORKING_STATE.md` eingesehen. 0 offene CRITICAL AI-Tags.
5. **Veto-Check:** Active VETOs `F-02` (HNSW Rewiring Veto, due 2026-10-07) & `OP-03` (No Realtime Audio Veto, due 2026-10-07) innerhalb der 3-Tages-Grenzfrist.
6. **Build-Grundlage:** `cargo check --workspace --exclude contextra-py` mit Exit-Code 0 erfolgreich.
7. **Git-Kontext:** HEAD Commit `8ff64018a365e945355dff1e328dedc8c942dabf` (`docs(adr): add ADR-108 to ADR-114`).
8. **Preflight:** Preflight-Checks ausgeführt.
9. **Ring-Layering:** `contextra-checkpoint` liegt in Ring 1 (L1) und importiert ausschließlich L0/L1 Ports/Types (`contextra-core`, `contextra-ports`, `contextra-types`). Keine unzulässigen Upward-Dependencies.
10. **Unsafe-Inseln:** `contextra-checkpoint` enthält `#![forbid(unsafe_code)]`. Zero unsafe blocks.
11. **ADR-Deadlines:** Alle ADR-Fristen eingehalten (0 überfällige Deprecation-Fristen).

---

## 6. Audit Verdict & Evidence

```text
===============================================================================
EVIDENCE LOG: AUDIT CONTEXTRA-CHECKPOINT
===============================================================================
[EVIDENCE-P1-1] Deterministic clock injection verified in guard.rs, store.rs, and orphan.rs. Zero direct SystemTime::now() calls in crate src/.
[EVIDENCE-P2-1] CheckpointGuard enforces RAII via #[must_use] and Drop orphan registration in guard.rs.
[EVIDENCE-P3-1] InstanceOrphanRegistry provides zero-blocking drop handling with startup/open recovery and piggyback flushes in orphan.rs & store.rs.
[EVIDENCE-P4-1] Symmetrical snapshot unpinning verified in PersistentCheckpointStore::drop_checkpoint and create_checkpoint.
[EVIDENCE-P5-1] Blake3 checksum computed in CheckpointManifest::new and verified on load in store.rs.
[EVIDENCE-P6-1] System TxIds allocated starting at TxId::INTERNAL_BASE (1 << 62) in store.rs, strictly separated from user TxIds.
===============================================================================
VERDICT: PASSED
===============================================================================
```
