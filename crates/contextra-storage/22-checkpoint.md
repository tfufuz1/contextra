# SP-CKPT — Checkpoint, Backup, Restore und Reconcile

| | |
|---|---|
| Modul | `checkpoint/` |
| Besitzt | Checkpointmanifest, konsistente Dateimenge, Restoreprüfung, Ledger-Reconcile |
| Konsumiert | PORTS, CATALOG, LEDGER, ANCHOR, RUN, BLOB, KEYS |
| Spec-Bezug | R8 §10.2–10.3; INV-ANCH-1, INV-ERA-9, INV-KEY-1…4 |
| Phase | P6 (G-E) |

## 1 Auftrag
Checkpoint ist konsistente Referenzmenge, kein neuer Datenkatalog. Nur durabel gelistete Dateien werden aufgenommen. Hardlink ist opt-in (SD-13). Backup policy gibt `keys_in_backups` wahrheitsgetreu an; cryptographic Erase darf nur zugesagt werden, wenn Schlüsselkopien ausgeschlossen sind.

## 2 Verträge
`checkpoint(options)` erstellt Manifest mit Catalog-Epoche, Redo-Boden, Dateimengen und Erase-Epoch; `restore(source, anchor, ledger)` prüft Format/Integrity und reconciled jedes Ledger-`tomb_id`, sodass betroffene Cluster mindestens Retired werden. Restore schreibt nicht hinter externen Anker.

## 3 Regeln
Keine Schlüssel im Manifest; versiegelte Inhalte bleiben bytegenau. Restore kann fehlende Schlüssel als KeyUnavailable/KeyLost differenzieren. Anker/Catalog/Ledger werden als zusammenpassender Stand validiert; mismatch ⇒ read-only/rollback diagnosis.

## 4 Invarianten / Umsetzung
INV-ANCH-1, INV-ERA-9, INV-KEY-1…4. Crash-Sweep Checkpoint-Create; tamper/partial backup cases; Restore vor/nach Gate/ER2; no resurrection test.

## 5 Arbeitspakete
WP-22.1 manifest/snapshot capture (P6) · WP-22.2 optional hardlink/reflink (P6) · WP-22.3 restore verification/reconcile (P6). Abnahme: kein wiederhergestellter Tombstone-Cluster wird Live.
