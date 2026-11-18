# contextra-checkpoint — Crate-Spezifikation (Checkpoints & Verzweigungen)

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Status:** Normativ  
**Version:** 1.0  
**Geltungsbereich:** `crates/contextra-checkpoint`  
**Referenz:** `docs/SPEC.md` §2, §4, §5

---

## 1. Zweck & Systemrolle

`contextra-checkpoint` stellt leichtgewichtige Point-in-Time Checkpoints und Verzweigungsmechanismen (Forking/Branching) für Contextra bereit:
- **Zero-Copy Checkpointing:** Erstellung von konsistenten Momentaufnahmen des Datenbestands mittels Dateisystem-Hardlinks auf unveränderliche SSTables (`.sst`).
- **Agent- & Session-Forking:** Verzweigen von Datenständen für isolierte Experimente oder Hintergrund-Analysen ohne Duplizierung des primären Speichervolumens.
- **Krypto-Shredding-Sicherheit:** Checkpoint-Dateien teilen die physikalischen Datenblöcke mit dem Hauptspeicher. Das Vernichten des Schlüssels im Haupttresor macht Daten in allen Checkpoints unverzüglich unlesbar.

---

## 2. Architektur & Hardlink-Koordination

```
┌─────────────────────────────────────────────────────────────┐
│                    contextra-checkpoint                     │
│                                                             │
│   ┌─────────────────────────────────────────────────────┐   │
│   │                  CheckpointManager                  │   │
│   │                                                     │   │
│   │   Primary Storage Dir            Checkpoint Dir     │   │
│   │   ┌─────────────────┐            ┌────────────────┐ │   │
│   │   │ Manifest        │            │ Manifest (copy)│ │   │
│   │   │ sst-001.sst ────┼──Hardlink──┼► sst-001.sst   │ │   │
│   │   │ sst-002.sst ────┼──Hardlink──┼► sst-002.sst   │ │   │
│   │   └─────────────────┘            └────────────────┘ │   │
│   │                                                     │   │
│   │   ┌────────────────────┐    ┌───────────────────┐   │   │
│   │   │ HardlinkCloner     │    │ SnapshotPinHolder │   │   │
│   │   │ (Fallback: Copy)   │    │ (MVCC Pin Guard)  │   │   │
│   │   └────────────────────┘    └───────────────────┘   │   │
│   └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## 3. Interaktion mit der DSGVO-Löschkette

Checkpoints stellen eine besondere Herausforderung für das physische Löschen dar, da Hardlinks verhindern, dass eine Datei durch einfaches `unlink` im Primärverzeichnis physisch von der Festplatte verschwindet.

### 3.1 Garantien durch Krypto-Shredding
- Da alle vertraulichen Payloads (Klasse K und P) verschlüsselt in den SSTables liegen, führt die Vernichtung des Dokumenten-DEK oder des Betroffenen-Schlüssels im `contextra-erasure`-Vault dazu, dass der Chiffretext **in allen Checkpoints gleichzeitig unlesbar wird** ($O(1)$ Crypto-Shredding).

### 3.2 Bereinigung von Checkpoints im Purge-Fenster
- **`SnapshotRevoker`:** Wenn Schritt 4a der Löschsaga aufgerufen wird, ermittelt `CheckpointManager` alle aktiven Checkpoint-Verzeichnisse, die auf SSTables $\le gated\_seq$ verweisen.
- Betroffene Hardlinks in Checkpoints werden invalidiert oder getrennt, um das physische Überschreiben (`shred_file`) der primären SSTables zu ermöglichen.

---

## 4. Schnittstellen

```rust
pub struct CheckpointOptions {
    pub allow_copy_fallback: bool,
    pub sync_manifest: bool,
}

pub trait Checkpointer: Send + Sync {
    fn create_checkpoint(
        &self,
        src_dir: &Path,
        dst_dir: &Path,
        opts: &CheckpointOptions,
    ) -> Result<CheckpointInfo, ContextraError>;

    fn revoke_checkpoints_for_seq(
        &self,
        max_seq: u64,
    ) -> Result<usize, ContextraError>;
}
```

---

## 5. Invarianten

1. **`#![forbid(unsafe_code)]`:** Vollständig speichersichere Implementierung.
2. **Kein Klartext-Kopieren:** Checkpoints dürfen niemals entschlüsselte Klartexte auf Dateisystemebene erzeugen.
3. **Plattenreserve:** Vor dem Anlegen eines Checkpoints wird geprüft, ob genügend Inode- und Speicherplatzkapazität vorliegt (`contextra-durable-fs`).
