# SP-RECOVERY — Öffnen, Integritätsprüfung, Replay

| | |
|---|---|
| Modul | `recovery/` |
| Besitzt | Open-Reihenfolge, Format-/Integritätsprüfung, Redo-Scan, Replay und ReadOnly/Rollback-Entscheid |
| Konsumiert | CODEC, ENV, REDO, CATALOG, LEDGER, ANCHOR, MEMVER |
| Spec-Bezug | R8 §5.8, §10; INV-LOG-5/7/8/9, INV-ANCH-1, INV-ERA-9 |
| Phase | P1 replay basis; P2 vollständige Recovery (G-F) |

## 1 Auftrag
Recovery rekonstruiert ausschließlich aus durablen Catalog-/Redo-/Ledgerzuständen. Integritätsmodus liegt instanzweit im FORMAT-Record und ist unveränderlich: Plain, Authenticated oder Encrypted; `personal` erzwingt Encrypted (E-41). Restore ersetzt dieselbe Instanz und erhält dieselbe `InstanceId`; lokaler `LOCK` und Rollback-Anchor sind kein hostübergreifendes Eigentümer-Fencing.

## 2 Reihenfolge
Lese Format/Version und IntegrityMode; prüfe Anker gegen Catalog-/Ledger-Epoche; öffne Catalog read-only bis Rollbackprüfung bestanden; lade Root-/LedgerScope-Keyhandles; replaye nur vollständige Redo-Gruppen ab replay_floor bis redo_mark. Rekonstruiere Gate-Wahrheit aus validen Catalog-/Redo-Frames **und** vollständig validierten durable Ledger-`EraseIntent`/`SpaceDropIntent`; Selector-Typen Cluster/Range/Branch/Space getrennt halten. Space-Drops matchen die nie wiederverwendete `SpaceId u64`; IDs 0…255 bleiben reservierte Systemzone. Vor Views/Schreibzugriff alle fehlenden Catalog-Gates idempotent schreiben, `GateApplied`/Anchor nachziehen und offene Erase-Tickets fortsetzen. Fehlt Ledger, Scope-Key, erforderlicher Anchor oder ein externer Fencing-Nachweis für den bisherigen Instanzbesitzer, bleibt die Instanz `RestoreQuarantine`/ReadOnly/Locked; ein starker Anchor ist kein Lease. Kein Restore-Zustand darf vor Gate-Reconcile Nutzer-Views öffnen. Rekonstruiere Memtable, aktive Idem-Ergebnisse aus SYS_IDEM-Runs plus Redo-Suffix; Idem-Ergebnisse dürfen nicht allein aus Redo rekonstruiert werden, da `replay_floor` hinter ihrer TTL liegen kann. Compaction/flush erhält Tokens vor Ablauf und durablem Trim.

## 3 Regeln / Fehler
Torn Tail nur am letzten Segmentende und nur nach Redo-Mark-Regel abschneiden; beschädigter Mid-Log ist Corrupt. Unbekannte Tags/Versionen Unsupported. Anker voraus ⇒ F-9 Rollback und keine schreibende Öffnung. Fsync/EIO nicht wiederholen. Recovery ist deterministisch und meldet minimale Fehlerlokation ohne Inhalte.

## 4 Invarianten / Umsetzung
INV-LOG-5/7/8/9, INV-ANCH-1, INV-ERA-9, INV-CONS-1. Crash-Sweep jede Write-Grenze; Replay idempotent; Restore setzt Ledger-Cluster mindestens Retired; Differenz gegen unabhängiges Modell.

## 5 Arbeitspakete
WP-14.1 Format/Catalog/Redo Open-Reihenfolge (P1) · WP-14.2 Replayfloor/Torn-tail/Recovery-Scan (P2) · WP-14.3 Anker-Rollback und Restore-Reconcile (P6). Abnahme: Recovery nur aus durablen Präfixen und kein Rollback-Schreibzugriff.

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §9.5.2, §10.1 (R2a, R8), §10.3/§10.4; `INV-ERA-13`, `INV-LED-2`; E-74, E-75, E-81.

- **R2a:** Ledger-Gerät nicht lesbar ⇒ schreibend `Err(LedgerUnavailable)`; `ReadOnly` nur mit Diagnose, Catalog-Gates gelten weiter.
- **R8:** `EraseIntent{origin: Poisoned}` ohne `GateApplied` wird wie jeder andere Intent idempotent in ein Gate überführt; zuvor lösen R6/R7 `Indeterminate`-Changes auf, damit das Gate über dem endgültigen Zustand liegt.
- **Restore (WP-22.4):** Matching gegen alle nicht retired Tomb-Epochen; Kopie unter `restore_floor` ⇒ `BackupBeyondHorizon`; fehlender oder verlorener Tomb-Schlüssel einer nicht retired Epoche ⇒ Quarantäne.

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-14.4 | R8.2-Open/Restore: `LedgerUnavailable`, Poisoned-Intents in R8, Epochen-Matching, `BackupBeyondHorizon` | 1.5 | WP-14.3, WP-20.4 | P6 | Absturzmatrix um R2a/R8-Fälle erweitert, Restore-Reconcile mit zwei Epochen und einem Retire |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

Recovery-Schritte heißen verbindlich OP1…OP10. Sie lösen vor Sichtbarkeit `Indeterminate`-Gruppen, Ledger-Intents und ausstehende Purge-Batches vorwärts auf; ein Zwischenbatch kann nie `Done` bedeuten. Open führt Cluster-Index-/Datei-Admission aus und bleibt bei Anker-, Ledger-, Schlüssel- oder Owner-Abweichungen in ReadOnly bzw. `RestoreQuarantine`.
