# SP-PURGE — Physischer Purge und clusterreine Rohkopie

| | |
|---|---|
| Modul | `maint/purge.rs` |
| Besitzt | lebende Runs/Blobs umschreiben, Zieldateimengen bestimmen, per-Datei Purge-Edits, Graveyard und Unlink-Übergabe |
| Konsumiert | RUN (copy_sections), CATALOG, ERASE, CLUSTER, ENV, GRAVEYARD |
| Spec-Bezug | R8 §5, §9.5; INV-PL-2/4, INV-ERA-2/3/6/11 |
| Phase | P2 Basis; P6 vollständig |

## 1 Auftrag
Entfernt Zielcluster aus lebenden Dateien. Unveränderte Run-Abschnitte werden bytegleich kopiert, sofern CRC/Seal-Identität verifiziert ist; Blob-Repack folgt der stabilen logischen AAD-Identität und der Inkarnationsregel. Die Kosten hängen von `EraseLocality` und den betroffenen Dateien/Extents ab; es gilt keine pauschale O(Clusterbytes)-Zusicherung.

## 2 Protokoll
Ermittle betroffene Dateien/Inkarnationen und blockiere neue betroffene Writes gemäß Gate. Schreibe und synchronisiere pro Datei genau ein `PurgeBatchEdit`, das neue Dateien installiert und entfernte Dateien im persistenten Graveyard pinnt; nach letzter Reader-/Arc-Referenz folgt `unlink + sync_dir` und `UnlinkAckEdit`. Erst nach allen Datei-Acks, Owner-Attesten und finalem Verify darf `PurgeCompleteEdit` den Ticket-Graveyard leeren und `Purged/Done` setzen. Vollständig tote Dateien werden direkt entlinkt, ohne Survivor-Rewrite. Vor Rewrite sind getrennte, vorallokierte Scratch-Slots zu reservieren; Standardpool mindestens zwei maximale Run/Blob/Redo-Dateien (128 MiB bei 64-MiB-Grenzen). Diese Zusage gilt nur, wenn FS-CoW/Quota und Checkpoint-Pins die Platzfreigabe nicht verhindern; sonst `Purging/Blocked(DiskSpace)`.

`ClusterLocal` erzwingt eine Cluster-ID pro Run-/Blob-Datei und vermeidet dort fremde Survivor-Rewrites, erhöht aber Dateien/FDs und beseitigt weder gemeinsames Redo noch Pins. `PackedBounded` erlaubt Survivor-I/O bis zur Summe der betroffenen, je `max_file_bytes` begrenzten Dateien. Redo-Retention, Checkpoints und Extensions haben separate Limits/Atteste. Plain `ER4`-Barriere darf keinen versiegelten Consumer-Stream als Lagged verlieren; Consumer-Invalidierung erfolgt über epochgebundene durable Watermarks.

## 3 Regeln
Keine Schlüsselvernichtung ist Teil dieses SPs; Ciphertext-Rohkopie erfordert kein AEAD-Decrypt. Prüfe CRC/Scrub während Rohkopie. Keine Cluster fremder Claims entfernen. `purge_deadline` Pflicht; Budget/Backpressure im Scheduler. `ClusterLocal`- und `PackedBounded`-Kosten sowie Redo-/Pin-/Extension-Kosten getrennt ausweisen; kein Vollstore-Rewrite und keine unbewiesene Gesamt-Laufzeitgrenze. Fehlt Scratch-Platz, eine Owner-Attestation, Reader-Drain oder Platzfreigabe, darf kein physisches `Done` gemeldet werden. Cryptographic ER6 erst nach KeyProvider-Probe gemäß SP-ERASE.

## 4 Invarianten / Umsetzung
INV-PL-2/4, INV-ERA-2/3/6/11, INV-CLAIM-1, INV-CAT-1/2, INV-GRAVE-1. Crash-Sweep an jedem Bau-/Rename-/Catalog-/Unlink-Punkt; Byte-Zähler; Rohkopie Roundtrip.

## 5 Arbeitspakete
WP-19.1 Packgruppen-Purge und `PurgeBatchEdit` (P2) · WP-19.2 Graveyard/Reader-Drain/Unlink-Ack und `PurgeCompleteEdit` (P6) · WP-19.3 Plain barrier, bounded scratch und deadline (P6). Abnahme: kein lebender Run/Redo/Memtable enthält purged Cluster; pro-Datei-Zustandsautomat konvergiert bei Crash; Kosten- und Platzgrenzen werden je Lokalitätsmodus sowie separat für Redo, Pins und Extensions belegt.

## R9.1-Synchronisierung

`PurgeBatchEdit` ist je Datei/Inkarnation Pflicht; vollständig tote Dateien werden direkt entfernt, aber erst nach Reader-Drain und `sync_dir` bestätigt. `PurgeCompleteEdit` folgt erst bei leerem Ticket-Graveyard, finalem Verify und allen Extension-Attesten. `punch_hole` ist optional: Blob-Repack wird ab Hole-Ratio-/Fragmentgrenze in neue Inkarnation geschrieben; für Plain signalisiert die Barriere Consumer-Epoch/`Lagged` explizit.
