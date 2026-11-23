# SP-RECOVERY — Öffnen, Integritätsprüfung, Replay

| | |
|---|---|
| Modul | `recovery/` |
| Besitzt | Open-Reihenfolge, Format-/Integritätsprüfung, Redo-Scan, Replay und ReadOnly/Rollback-Entscheid |
| Konsumiert | CODEC, ENV, REDO, CATALOG, LEDGER, ANCHOR, MEMVER |
| Spec-Bezug | R8 §5.8, §10; INV-LOG-5/7/8/9, INV-ANCH-1, INV-ERA-9 |
| Phase | P1 replay basis; P2 vollständige Recovery (G-F) |

## 1 Auftrag
Recovery rekonstruiert ausschließlich aus durablen Catalog-/Redo-/Ledgerzuständen. Integritätsmodus liegt instanzweit im FORMAT-Record und ist unveränderlich: Plain, Authenticated oder Encrypted; `personal` erzwingt Encrypted (E-41).

## 2 Reihenfolge
Lese Format/Version und IntegrityMode; prüfe Anker gegen Catalog-/Ledger-Epoche; öffne Catalog read-only bis Rollbackprüfung bestanden; lade Schlüsselhandles; replaye nur vollständige Redo-Gruppen ab replay_floor bis redo_mark; rekonstruiere Memtable, aktive Idem-Ergebnisse aus SYS_IDEM-Runs plus Redo-Suffix und Erase-Tickets; erst dann schreibend öffnen. Aktive Idem-Ergebnisse dürfen nicht allein aus Redo rekonstruiert werden, da `replay_floor` hinter ihrer TTL liegen kann. Compaction/flush erhält alle Tokens vor Ablauf und durablem Trim. Anker darf niemals vor durablem Edit fortschreiten.

## 3 Regeln / Fehler
Torn Tail nur am letzten Segmentende und nur nach Redo-Mark-Regel abschneiden; beschädigter Mid-Log ist Corrupt. Unbekannte Tags/Versionen Unsupported. Anker voraus ⇒ F-9 Rollback und keine schreibende Öffnung. Fsync/EIO nicht wiederholen. Recovery ist deterministisch und meldet minimale Fehlerlokation ohne Inhalte.

## 4 Invarianten / Umsetzung
INV-LOG-5/7/8/9, INV-ANCH-1, INV-ERA-9, INV-CONS-1. Crash-Sweep jede Write-Grenze; Replay idempotent; Restore setzt Ledger-Cluster mindestens Retired; Differenz gegen unabhängiges Modell.

## 5 Arbeitspakete
WP-14.1 Format/Catalog/Redo Open-Reihenfolge (P1) · WP-14.2 Replayfloor/Torn-tail/Recovery-Scan (P2) · WP-14.3 Anker-Rollback und Restore-Reconcile (P6). Abnahme: Recovery nur aus durablen Präfixen und kein Rollback-Schreibzugriff.
