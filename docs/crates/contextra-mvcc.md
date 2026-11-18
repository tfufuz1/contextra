# contextra-mvcc — Crate-Spezifikation

**Ebene:** E1a (Kern-Closure: `ids`, `env`)  
**Bindung:** `docs/SPEC.md` §2, §5, §9

---

## 1 Zweck & Domäne

`contextra-mvcc` stellt Multi-Version Concurrency Control (MVCC) für Snapshot-Isolation und historische Zeitreise-Lesezugriffe bereit. Es verwaltet Transaktionssequenzen (`Seq`), Snapshot-Registrierungen und Pins für die Garbage Collection / Kompaktion.

## 2 Rolle in der Löschkette & Kapselung

- **Snapshot-Widerruf (`SnapshotRevoker`):** Bei Eintritt in die Purge-Phase (Schritt 4a der Lösch-Saga) widerruft `mvcc` alle aktiven Snapshot-Pins, deren Sequenz `seq <= gated_seq` ist, sofern sie betroffene Datenbereiche überdecken.
- **Tombstone-Lebenszyklus:** Tombstones für gelöschte Datensätze dürfen erst freigegeben werden, wenn kein aktiver Snapshot älter als der Löschzeitpunkt existiert.
- **Fail-Closed bei Lesezugriffen:** Historische Snapshot-Reads konsultieren vor der Datenauslieferung immer den aktuellen `ReadGate` des Datenbereichs. Historische Leserechte dürfen niemals ein gelöschtes Datum wieder sichtbar machen (Z-T5).

## 3 Schnittstellen

- `SnapshotRegistry`: Registrierung und Deregistrierung von Snapshot-Pins.
- `min_active_seqno()`: Liefert die älteste aktive Sequenznummer zur Steuerung von Kompaktionen.
- `revoke_covering_snapshots(target: &EraseTarget, gated_seq: u64)`: Zwangsentfernung von Snapshots nach Schlüsselvernichtung.

## 4 Invarianten

- Sequenznummern sind streng monoton steigend.
- Keine unsafe Codeblöcke (`#![forbid(unsafe_code)]`).
- Keine Panics im Produktionspfad.
