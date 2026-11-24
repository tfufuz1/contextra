# SP-MEMVER — Memtable, Versionen, Registry, Graveyard

| | |
|---|---|
| Modul | `engine/{memtable,version,registry,graveyard}.rs` |
| Besitzt | immutable `Version`, Arc-Veröffentlichung, clusterpartitionierte Memtable, Retire-/Graveyard-Lebensdauer |
| Konsumiert | PORTS, ENV, CATALOG, REDO; keine Dateiformate |
| Spec-Bezug | R8 §6, §7, §9.5, §11; INV-VIEW-1/2, INV-GRAVE-1/2, INV-ERA-2 |
| Phase | P1–P3 (Crash-Sweep bis P1) |

## 1 Auftrag
Stellt den Übergang von angenommenen Änderungen zu unveränderlichen, lesbaren Versionen bereit. Memtable ist nach Cluster partitioniert (`ClusterMap → Skiplist`), damit Gate eine Partition O(1) aushängen kann (E-42). Reclamation und Datei-Unlink folgen der letzten `Arc<Version>`-Referenz.

## 2 Verträge
`Memtable::insert(frame)` akzeptiert nur bereits validierte/committete Frames. `freeze()` liefert unveränderliche Clusterpartitionen. `VersionRegistry::publish(visible_stamp, Arc<Version>)` veröffentlicht zuerst Stamp und danach Version gemäß dem atomaren Publikationsprotokoll; Leser sehen niemals neuen Inhalt mit altem Sichtbarkeitsstempel. `Graveyard::retire(version/files)` verzögert Freigabe bis Views/Iteratoren fallen.

## 3 Regeln
Keine Stamp-Vergabe hier. Keine Sperre bei Observer-/Provider-Rückrufen. Locks nach Rangordnung; Snapshots per Arc; keine mutierbare Version nach Publikation. Gate entfernt Partition aus dem aktiven Verzeichnis und queued Reclaim auf urgent Lane. Klartext-Reclamation ist nicht durch `Zeroizing`-Typ allein bewiesen: keine unkontrollierten `Bytes`-/`Arc`-Clone-Pfade; Generation invalidiert neue Leser, aktive Epoch-Leases werden gezählt und als Exposition geführt, bis Reclamation abgeschlossen ist. `gc_floor` ist das Minimum aus aktiven View-, Hold-, Consumer- und lebenden Fork-Stamps (ohne Pins: `durable_stamp+1`). `history_floor` ist der monoton fortgeschriebene, tatsächlich verfügbare Historienrand und darf erst nach abgeschlossener Compaction überlebt werden; stets `history_floor <= gc_floor` (E-37).

## 4 Invarianten / Umsetzung
`INV-VIEW-1/2`, `INV-GRAVE-1/2`, `INV-ERA-2`, `INV-LOCK-1/2`. Loom-/Sim-Interleavings für Publish, View-Klon, Gate, Retire, letzter Drop; Modelltest für O(1)-Detach und keine Klartext-Reclamation vor letztem Leser. OS-/Allocator-Reste bleiben Receipt-Restrisiko.

## 5 Arbeitspakete
WP-09.1 Memtable-Schnittstelle + Partitionierung (P1) · WP-09.2 Version/Registry Publikationsprotokoll (P1, ENV) · WP-09.3 Graveyard + Arc-Reclamation (P2, RUN/CATALOG) · WP-09.4 Floor/Branch/Hold Integration (P5–P7). Abnahme: kein unlink bei lebender Version; keine Mischsicht.

## R9.1-Synchronisierung

Die Pipeline publiziert ausschließlich nach erfolgreichem Sync in Stamp-Reihenfolge. Gate erhöht die Cluster-Generation, hängt die Clusterpartition aus und verhindert neue Zugriffe; aktive Leser werden epoch-basiert abgewartet oder als Restexposition ausgewiesen. Unkontrollierte Klartext-Clone-Pfade sind verboten; Zeroizing allein ist kein Vollständigkeitsbeweis.
