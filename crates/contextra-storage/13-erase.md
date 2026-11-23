# SP-ERASE — Gate, Schlüsselvernichtung, Tickets, Receipts

| | |
|---|---|
| Modul | `erase/{gate,protocol,tickets,receipt}.rs` |
| Besitzt | Erase-Zustandsmaschine, Gate-Frame, ER1/ER2/ER6-Koordination, Ticket, kanonisches Receipt |
| Konsumiert | PORTS, COMMIT, CLUSTER, KEYS, LEDGER, PURGE, CATALOG, ANCHOR |
| Spec-Bezug | R8 §9; INV-ERA-1…12, INV-ANCH-1, INV-LED-1 |
| Phase | P1 gate skeleton; P2 ER4/ER6; P6 G-E |

## 1 Auftrag
Führt ausschließlich vorwärts: Gated → KeyDestroying → KeysDestroyed → Purging → Done. Gate ist ein Committer-Frame. Keine Schlüssel- oder Nutzdaten im Ledger; Ledger-Eintrag verwendet `tomb_id`. Keine Rücknahme nach Gate.

## 2 Vertrag
`Eraser::erase(request) -> Ticket`; `poll/resume(ticket)` liefert Status; `verify_receipt(bytes, ledger_slice)` prüft kanonische Bytes offline. Gate nimmt transitive DerivedFrom-Menge atomar auf; danach publiziert jede View `Erased{at:S}`. Schlüsselprovider-Aufrufe außerhalb Sperren, Batch `destroy_many`; Purge nur nach Schlüssel-/Restriction-/Extension-Prüfungen.

## 3 Ordnungsregeln
ER1 Gate durabel → ER2 Schlüsselvernichtung → ER6 Catalog-Edit/Barriere → Unlink. Cryptographic: ER6 erst bei `probe != Present`, Done nur bei `Absent`. BLOCK_ERASE stoppt jede Löschung. Fehler/Absturz setzen Roll-forward fort. `purge_deadline` ist Pflichtparameter, niemals Default. Keine Relaxed-Durability für Erase. Physischer Rewrite kann bei fehlendem Scratch-Platz blockieren; Ticket bleibt `Purging/Blocked`, bis es fortgesetzt werden kann. `space_drop` prüft die vollständige Cluster-/Restriction-Menge atomar. Range-Erase ist nie Crypto-Erase; Receipts unterscheiden Kern- und Extension-Abdeckung.

## 4 Invarianten / Umsetzung
INV-ERA-1…12, INV-ANCH-1, INV-LED-1. FaultFs-Sweep an jedem Persistenz-/Rename-/Unlink-Punkt; Modellzustand prüft nur Vorwärtskanten; Byte-Zähler für INV-ERA-11; Receipt-Fuzzer und kanonische Referenzvektoren.

## 5 Arbeitspakete
WP-13.1 atomarer Gate/Visibility (P1) · WP-13.2 Roll-forward ER4/ER6 (P2) · WP-13.3 Provider-Key-Batch/Probe (P6) · WP-13.4 Restrictions + Extension-Verifikation (P6) · WP-13.5 Receipt/Offline-Verifikation (P6) · WP-13.6 Deadline-/Metrikpfad (P6). Abnahme: alle Crash-Punkte konvergieren monoton zu sicherem Zustand.
