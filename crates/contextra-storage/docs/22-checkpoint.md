# SP-CKPT — Checkpoint, Backup, Restore und Reconcile

| | |
|---|---|
| Modul | `checkpoint/` |
| Besitzt | Checkpointmanifest, konsistente Dateimenge, Restoreprüfung, Ledger-Reconcile |
| Konsumiert | PORTS, CATALOG, LEDGER, ANCHOR, RUN, BLOB, KEYS |
| Spec-Bezug | R8 §10.2–10.3; INV-ANCH-1, INV-ERA-9, INV-KEY-1…4 |
| Phase | P6 (G-E) |

## 1 Auftrag
Checkpoint ist konsistente Referenzmenge, kein neuer Datenkatalog. Nur durabel gelistete Dateien werden aufgenommen. Für die physische Purge-/DiskFull-Garantie sind unabhängige `Copy`-Dateien vorgeschrieben. `Reflink`/`Hardlink` sind nur mit registriertem Owner zulässig, der vor Scratch-Recycling die Referenzen auf betroffene Extents nachweisbar entfernt und attestiert; sonst sind sie nicht verwaltete Retention-Copies und Restrisiko im Receipt. Backup policy gibt `keys_in_backups` wahrheitsgetreu an; cryptographic Erase darf nur zugesagt werden, wenn Schlüsselkopien ausgeschlossen sind.

## 2 Verträge
`checkpoint(options)` erstellt Manifest mit Catalog-Epoche, Redo-Boden, Dateimengen und Erase-Epoch; `restore(source, anchor, ledger)` erhält dieselbe `InstanceId`, prüft Format/Integrity und reconciled alle durable `EraseIntent`-/`SpaceDropIntent`-Records vor jeder Sichtbarkeit. Cluster-Tombstones retire'n Cluster; verschlüsselte Range-/Branch-Selector werden via `LedgerScopeKey` angewandt; Space-Tombstone gated die komplette SpaceId. Vor schreibendem Restore muss das Deployment den bisherigen Instanzbesitzer extern gefenced haben; lokales `LOCK` verhindert keine Kopie auf einem zweiten Host, und ein Rollback-Anchor ist keine Besitz-Lease. Fehlendes Ledger, LedgerScopeKey, erforderlicher Anchor oder Fencing-Nachweis ⇒ `RestoreQuarantine`, kein Read-/Write-Open. Restore schreibt nicht hinter externen Anker.

## 3 Regeln
Keine Schlüssel im Manifest; versiegelte Inhalte bleiben bytegenau. Restore kann fehlende Schlüssel als KeyUnavailable/KeyLost differenzieren. Anker/Catalog/Ledger werden als zusammenpassender Stand validiert; mismatch ⇒ read-only/rollback diagnosis.

## 4 Invarianten / Umsetzung
INV-ANCH-1, INV-ERA-9, INV-KEY-1…4. Crash-Sweep Checkpoint-Create; tamper/partial backup cases; Restore vor/nach Intent/Gate/ER2, für Cluster/Range/Branch/Space; no resurrection test und Scope-Selector-Canary.

## 5 Arbeitspakete
WP-22.1 manifest/snapshot capture (P6) · WP-22.2 optional hardlink/reflink (P6) · WP-22.3 restore verification/reconcile (P6). Abnahme: kein wiederhergestellter Tombstone-Cluster wird Live.

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §10.3, §10.4; `INV-LED-2`; E-74, E-81.

- Das Manifest enthält `erase_epoch`, `created_at` und die höchste offene `tomb_epoch` (`BackupMark`); externe Backups müssen die Marke über `Storage::backup_mark()` mitführen.
- Restore verweigert eine Kopie mit `mark.erase_epoch < restore_floor` oder ohne Marke (sobald eine Epoche retired ist) mit `BackupBeyondHorizon`; kein Override. Gegen nicht retired Epochen wird abgeglichen; Tomb-Schlüssel sind nicht im Checkpoint (`INV-KEY-5`).

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-22.4 | `BackupMark` im Manifest, Restore-Floor-Prüfung, Epochen-Matching | 1.0 | WP-22.3, WP-20.4 | P6 | Restore-Test: Kopie unter Floor abgelehnt, ohne Marke abgelehnt bei Retire, zwei Epochen korrekt abgeglichen |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

R9.1 ändert kein Checkpointformat. Restore führt vor Schreiböffnung zusätzlich die R9-Open-Admission aus und verarbeitet unvollständige `PurgeBatchEdit`-/Graveyard-Zustände roll-forward. Fehlende Owner-, Ledger-, Anker- oder Fencing-Nachweise bleiben `RestoreQuarantine`; sie dürfen nicht durch eine lokale Dateiprüfung ersetzt werden.
