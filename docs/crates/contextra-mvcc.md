# contextra-mvcc — Crate-Spezifikation (Snapshot-Isolation & Concurrency)

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-mvcc`  
**Referenz:** `docs/SPEC.md` §2, §5, §9

---

## 1. Zweck & Systemrolle

`contextra-mvcc` implementiert das Multi-Version Concurrency Control (MVCC) Subsystem für Contextra:
- **Snapshot-Isolation:** Ermöglicht nebenläufigen Lesern den Zugriff auf einen konsistenten Datenstand zu einer bestimmten Sequenznummer (`Seq`), ohne dass Schreiber blockiert werden.
- **Pin-Verwaltung:** Verfolgung aktiver Snapshot-Leser zur Bestimmung der Mindest-Sequenznummer (`min_active_seqno`), die für die Speicherbereinigung und Kompaktion im LSM-Store (`contextra-store`) freigegeben werden darf.
- **Snapshot-Widerruf:** Zwanghafte Entwertung veralteter oder betroffener Snapshots während der DSGVO-Löschkette.

---

## 2. Architektur & Komponenten

```
┌─────────────────────────────────────────────────────────────┐
│                        contextra-mvcc                       │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                  SnapshotRegistry                   │   │
│   │                                                     │   │
│   │   Sequenz-Achse:                                    │   │
│   │   [seq=100] ──► [seq=105 (Pin A)] ──► [seq=110]     │   │
│   │                   │                                 │   │
│   │                   ▼                                 │   │
│   │         min_active_seqno = 105                      │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ SnapshotPin        │    │ SnapshotRevoker   │   │   │
│   │   │ (RAII-Guard)       │    │ (Port-Impl)       │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Rolle in der DSGVO-Löschkette

### 3.1 Snapshot-Widerruf (`SnapshotRevoker`)
Normalerweise dürfen SSTables erst kompaktiert und Tombstones entfernt werden, wenn kein Snapshot mehr auf ältere Versionen verweist (`min_active_seqno`).
Im Falle einer DSGVO-Löschung darf jedoch **kein aktiver Snapshot die Löschung unbegrenzt blockieren**:

1. Bei Eintritt in Schritt 4a der Löschsaga ruft `ErasureCore` das Trait `SnapshotRevoker::revoke_covering` auf:
   ```rust
   pub trait SnapshotRevoker: Send + Sync {
       fn revoke_covering<'a>(
           &'a self,
           target: &'a EraseTarget,
           gated_seq: u64,
       ) -> BoxFuture<'a, Result<u64, ErasureError>>;
   }
   ```
2. Alle Snapshot-Pins mit `seq <= gated_seq`, die den Zielbereich abdecken, werden als invalidiert markiert.
3. Anschließende Leseversuche über diesen Snapshot scheitern sofort mit `Err(ContextraError::SnapshotRevoked)`.
4. Der Wert von `min_active_seqno` springt sofort auf $> gated\_seq$, wodurch die Kompaktion zur physischen Vernichtung freigegeben wird.

---

## 4. Schnittstellen

```rust
pub struct SnapshotPin {
    seq: u64,
    registry: Arc<SnapshotRegistry>,
}

impl Drop for SnapshotPin {
    fn drop(&mut self) {
        self.registry.release(self.seq);
    }
}

impl SnapshotRegistry {
    pub fn acquire(&self) -> SnapshotPin;
    pub fn min_active_seqno(&self) -> u64;
    pub fn is_valid(&self, seq: u64) -> bool;
    pub fn revoke_before(&self, gated_seq: u64) -> usize;
}
```

---

## 5. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig sichere Rust-Implementierung.
2. **Monotone Sequenzen:** Sequenznummern werden atomar vergeben und fallen niemals zurück.
3. **Keine verwaisten Pins:** `SnapshotPin` nutzt RAII; bei Abbruch oder Panic des Threads wird der Pin zuverlässig aus der Registry deregistriert.
