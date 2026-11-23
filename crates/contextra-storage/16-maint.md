# SP-MAINT — Flush, Compaction, GC, Scrub, Scheduler

| | |
|---|---|
| Modul | `maint/{scheduler,flush,compaction,blob_gc,scrub,reserve}.rs` |
| Besitzt | Wartungsplanung/-lanes, Run-Erzeugung/-ersetzung, Floors, Scrub und Reserven |
| Konsumiert | PORTS, MEMVER, RUN, BLOB, CATALOG, PURGE, ENV |
| Spec-Bezug | R8 §5, §7, §9, §11; INV-CAT-1/2, INV-PL-1…5, INV-GRAVE-1/2 |
| Phase | P2 flush; P5 vollständige Wartung (G-M) |

## 1 Auftrag
Wartung ist außerhalb des Committers und nach Lanes `urgent > flush > compact > blobgc > idle` budgetiert. Flush wandelt eingefrorene Memtable-Partitionen in Runs; Compaction nimmt ausschließlich kompatible Wartungsgruppen; Blob GC darf nur nach lebenden Run-Referenzen handeln.

## 2 Commit-Vertrag
Neue Datei: `.building` → `sync_all` → rename → `sync_dir` → ein typisierter Catalog-Edit für vollständige Dateimenge und Zustandsdelta. Keine Datei wird vor Catalog-Lebendigkeit als sichtbar behandelt. Veraltete Datei geht in Graveyard; unlink nach letzter Versionreferenz.

## 3 Regeln
E-37 `gc_floor = min(history_floor, fork_stamp lebender Branches, Hold-Stamps)`. L0-Dateizahl unabhängig von aktiven Packgruppen (H-19). `maint_io_fraction` begrenzt Hintergrund-I/O; ENOSPC löst DiskFull-Notbetrieb aus, kein Poison. Reserveauffüllung passiert vor Druckgrenze; Purge/Key-Erase-Reserve bleibt verwendbar.

## 4 Invarianten / Umsetzung
INV-CAT-1/2, INV-PL-1…5, INV-GRAVE-1/2, INV-BLOB-1. Crash-Sweep für Flush/Compaction; Output gegen Merge-Orakel; Branch-/Hold-Pinning, Purge-Konflikte, DiskFull-Szenarien.

## 5 Arbeitspakete
WP-16.1 Flush und Catalog-Kopplung (P2) · WP-16.2 Compactiongruppen (P5) · WP-16.3 Blob GC (P5) · WP-16.4 Scrub/Integritätsisolierung (P5) · WP-16.5 Scheduler/Reservebudget (P5). Abnahme: lebende Daten erhalten, obsolete Dateien referenzsicher entfernt.
