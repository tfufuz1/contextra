# contextra-checkpoint — Crate-Spezifikation

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Bindung:** `docs/SPEC.md` §2, §4, §5

---

## 1 Zweck & Domäne

`contextra-checkpoint` stellt Mechanismen zur schnellen Erstellung von Point-in-Time Checkpoints und Verzweigungen (Forking) bereit:
- Hardlink-basiertes Klonen von persistenten SSTable-Dateien
- Sicherung von Manifest- und Metadaten-Ständen
- Koordination mit MVCC-Snapshot-Pins

## 2 Rolle in der Löschkette & Crash-Sicherheit

- **Hardlink-Sicherheit:** Da Hardlinks auf bestehende SSTables verweisen, müssen Checkpoints den gleichen Krypto-Shredding-Mechanismen unterliegen wie der Primärspeicher.
- **Widerruf von Checkpoints:** Bei Vernichtung des KEK (Collection-Drop) oder DEK/SubjectKey werden auch alle Checkpoints, die betroffene Daten enthalten, kryptographisch unlesbar.
- **Snapshot-Koordination:** Checkpoints halten MVCC-Pins. Wenn `SnapshotRevoker` aufgerufen wird, werden betroffene Checkpoint-Pins gelöst oder als invalid markiert.

## 3 Invarianten

- Checkpoints dürfen niemals unverschlüsselte Klone erzeugen, wenn der Primärspeicher verschlüsselt ist.
- `#![forbid(unsafe_code)]`.
