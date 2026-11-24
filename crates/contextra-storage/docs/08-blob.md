# SP-BLOB — `store::blob` (Körperdateien und Referenzlebenszyklus)

| | |
|---|---|
| Modul | `contextra-storage/src/store/blob/` |
| Besitzt | Blob-Dateiformat, Blob-Writer/Reader, Inkarnationen, Live-Referenzindex |
| Konsumiert | CODEC, ENV, KEYS, CATALOG, RUN; keine Committer-/Erase-Entscheidungen |
| Spec-Bezug | R8 §4.5, §5.7, §6, §9.5; INV-BLOB-1, INV-SEAL-1, INV-PRIV-1 |
| Phase | P4 (G-R) |

## 1 Auftrag
Körper oberhalb `value_separation_min` werden unveränderlich in Blob-Dateien geschrieben. Referenzen enthalten `(blob_id, incarnation, offset, len)`; Blob-IDs und Inkarnationen werden nie wiederverwendet. Nutzdaten-Hashes sind verboten. Dieses Modul entscheidet weder, ob eine Referenz lebendig ist, noch wann eine Datei entfernt werden darf.

## 2 Verträge
`BlobWriter::append(cluster, bytes, key_scope) -> BlobRef` schreibt nur in eine `.building`-Datei und liefert erst nach `sync_all` ein verwendbares Ergebnis. `BlobReader::read(ref, key_scope)` prüft Grenzen, CRC/AEAD und Schlüsselgeneration vor/nach Entsiegeln. `BlobGcIndex` wird aus lebenden Runs rekonstruiert bzw. durch Wartung aktualisiert; `remove_candidates` ist nur eine Kandidatenliste für MAINT. Purge ist unabhängig von `punch_hole`: unter Fragment-/Hole-Ratio-Grenzwerten wird eine neue Blob-Inkarnation mit gepackten Überlebenden geschrieben; Referenzwechsel ist ein atomarer Catalog-Edit.

## 3 Regeln und Fehler
**Positionsunabhängiges Siegel (E-58):** AAD bindet `instance/space/cluster/key_epoch/record_uid`, nie `blob_id` oder Offset. Ein unverändertes Record-SealFrame ist deshalb nach Repack in neue Blob-ID/Offset rohkopierbar; Directory/CRC werden neu geschrieben. `record_uid` bleibt über Rohkopien stabil und wird nie wiederverwendet.

Blob-Inhalt eines Sealed-Clusters ist mit dessen Clusterschlüssel versiegelt. Append nutzt ausschließlich vorallokierte Bereiche; ENOSPC ist F-2 und vergiftet den Store nicht. Purge-Repack benötigt freien Scratch-Platz; bei Platzmangel bleibt der Erase-Ticketstatus ehrlich `Purging/Blocked(DiskSpace)` und wird nach Wiederanlauf fortgesetzt, niemals `Done`. EIO/Sync-Fehler sind F-3 und werden nicht wiederholt. Ungültige Referenz/Frame ist F-4, unbekannte Version F-5. Eine Inkarnation darf nie stillschweigend auf eine neu angelegte Datei zeigen.

## 4 Invarianten / Umsetzung
`INV-BLOB-1`: Entfernung erst nach Abgleich aller lebenden Runs und atomarem Catalog-Edit, das Referenzen entfernt. `INV-SEAL-1`, `INV-PRIV-1`, `INV-ID-1/2` gelten. Decoder erhalten Fuzz-Ziele und Rundreise-Properties; Append/Rotation/Recovery kommen in den FaultFs-Crash-Sweep.

## 5 Arbeitspakete
WP-08.1 Format + Frame-Codec (P4, CODEC/ENV) · WP-08.2 Writer/Rotation (P4, WP-08.1) · WP-08.3 Reader + KeyScope (P4, WP-08.1) · WP-08.4 Referenzabgleich/GC-Index (P5, RUN/CATALOG). Abnahme: Referenz kann nicht auf andere Inkarnation zeigen; Bitflips werden erkannt; keine Inhalts-Hashes.

## R9.1-Synchronisierung

Blob-AAD bindet ausschließlich stabile `record_uid`, `instance_id`, `space_id`, `cluster_id` und `key_epoch`, niemals `blob_id` oder Offset. Survivor-Rohkopie in eine neue Blob-Inkarnation bleibt dadurch gültig; neue Positionen erhalten eigene CRC/Directory. Purge nutzt bei zu hohem Hole-Anteil Repack statt verpflichtendem Hole-Punch.
