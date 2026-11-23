# SP-PURGE — Physischer Purge und clusterreine Rohkopie

| | |
|---|---|
| Modul | `maint/purge.rs` |
| Besitzt | lebende Runs umschreiben, Zieldateimengen bestimmen, atomarer ER6-Edit, Unlink-Übergabe |
| Konsumiert | RUN (copy_sections), CATALOG, ERASE, CLUSTER, ENV, GRAVEYARD |
| Spec-Bezug | R8 §5, §9.5; INV-PL-2/4, INV-ERA-2/3/6/11 |
| Phase | P2 Basis; P6 vollständig |

## 1 Auftrag
Entfernt Zielcluster aus lebenden Dateien, ohne Nutzdaten zu entsiegeln. Kopiert unveränderte Clusterabschnitte bytegleich in neue Runs und schreibt Directory/Footer neu. Kosten bleiben proportional zu betroffenen Packgruppen, nicht gesamtem Store.

## 2 Protokoll
Ermittle vollständige betroffene Gruppe; blockiere neue betroffene Writes gemäß Gate; reserviere vor Start den benötigten Scratch-Platz oder halte Ticket sichtbar `Purging/Blocked(DiskSpace)`; baue Ersatzdateien; sync/rename/dir-sync; ein Catalog-Edit ersetzt alte und neue Dateimengen und setzt Purge-Stufe atomar; erst danach Graveyard darf alte Dateien nach letzter Arc-Referenz unlinken. Plain purge nutzt gebündelte Barriere ohne Segmentschnitt am Gate und markiert betroffene Consumer mit Epoch-Wechsel/`Lagged`.

## 3 Regeln
Kein Schlüssel nötig, kein AEAD decrypt. Prüfe CRC/Scrub während Rohkopie. Keine Cluster fremder Claims entfernen. `purge_deadline` Pflicht; Budget/Backpressure im Scheduler. Kosten sind proportional zur betroffenen begrenzten Gruppe, nicht generell zum Einzelcluster. Fehlt Scratch-Platz, darf kein physisches `Done` gemeldet werden. Cryptographic ER6 erst nach KeyProvider-Probe gemäß SP-ERASE.

## 4 Invarianten / Umsetzung
INV-PL-2/4, INV-ERA-2/3/6/11, INV-CLAIM-1, INV-CAT-1/2, INV-GRAVE-1. Crash-Sweep an jedem Bau-/Rename-/Catalog-/Unlink-Punkt; Byte-Zähler; Rohkopie Roundtrip.

## 5 Arbeitspakete
WP-19.1 Packgruppen-Purge + ER6 edit (P2) · WP-19.2 Plain barrier, batching, deadline (P6). Abnahme: kein lebender Run/Redo/Memtable enthält purged Cluster; Laufzeit skaliert mit Gruppe.
