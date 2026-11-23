# SP-ERASE — Gate, Schlüsselvernichtung, Tickets, Receipts

| | |
|---|---|
| Modul | `erase/{gate,protocol,tickets,receipt}.rs` |
| Besitzt | Erase-Zustandsmaschine, Gate-Frame, ER1/ER2/ER6-Koordination, Ticket, kanonisches Receipt |
| Konsumiert | PORTS, COMMIT, CLUSTER, KEYS, LEDGER, PURGE, CATALOG, ANCHOR |
| Spec-Bezug | R8 §9; INV-ERA-1…12, INV-ANCH-1, INV-LED-1 |
| Phase | P1 gate skeleton; P2 ER4/ER6; P6 G-E |

## 1 Auftrag
Führt ausschließlich vorwärts. Ein vollständig versiegeltes Ledger-Intent ist der dauerhafte Annahmepunkt; danach wird das Catalog-Gate idempotent geschrieben und vor jeder Sicht-/Schreibfreigabe publiziert. Range-/Branch-Selector liegen nur AEAD-verschlüsselt im Ledger unter `LedgerScopeKey`. Ein EraseIntent mit maximal 4096 Tomb-IDs ist ein einzelner <=256-KiB-Record; Retry ist über HMAC-Request-ID/-Digest idempotent. Keine Klartext-IDs, Benutzerkeys oder Nutzdaten im Ledger. Keine Rücknahme nach durablem Intent.

## 2 Vertrag
`Eraser::erase(request) -> Ticket`; `poll/resume(ticket)` liefert `Complete | CompleteWithResidual | Incomplete | Blocked | Failed`; `verify_receipt(bytes, ledger_slice, anchor_attestation)` prüft kanonische Bytes offline. `Complete` verlangt alle erforderlichen Kern- und Owner-Atteste. `CompleteWithResidual` ist nur für abgeschlossene Aufträge mit ausdrücklich benannten, außerhalb des Vertrags liegenden Restrisiken zulässig. Fehlender erforderlicher Owner-/Scope-Nachweis ist `Incomplete`, ein sicher pausierter und fortsetzbarer Auftrag `Blocked`, ein terminaler Integritäts-/Protokollfehler `Failed`. Gate nimmt transitive DerivedFrom-Menge atomar auf; danach publiziert jede View `Erased{at:S}`. Schlüsselprovider-Aufrufe außerhalb Sperren, Batch `destroy_many`; Purge nur nach Schlüssel-/Restriction-/Extension-Prüfungen.

## 3 Ordnungsregeln
Sequencer prüft BLOCK_ERASE/Restriction und friert Scope/Generation; Request-ID und Digest sind instanzgebunden, domain-separiert und semantisch kanonisch. Dann genau einen bounded `EraseIntent`-Datensatz (max. 4096 sortierte Tomb-IDs, höchstens 256 KiB) synchron durable schreiben. Erst danach: Catalog-Gate durable → Gate im RAM publizieren → `GateApplied` durable → erforderlicher Anchor-CAS. Vor irreversibler Folge muss Required-Anchor den Intent/Gate-Head enthalten. Vor KMS-Aufruf zusätzlich `KeyDestroyIntent` durable; `destroy_many` ist idempotent, Erfolg nur bei `probe == Absent`, danach `KeysDestroyed`-Event. Ein `Disabled`, timeout, `Unavailable` oder `Lost` Key ist kein `Absent` und schließt Crypto-Erase nicht ab. Purge folgt je Datei via `PurgeBatchEdit`, Reader-Drain, Unlink+`sync_dir`, `UnlinkAck`; nach allen Dateien/Owner-Attesten und final Verify `PurgeComplete`/ER6. `Done` nur nach allen Bedingungen. Crash-Recovery setzt jede Stufe fort; ein unvollständiger/torn Ledger-Record ist nicht angenommen, ein vollständig validierter durabler Record ist angenommen. `purge_deadline` ist Pflichtparameter, niemals Default; Fristüberschreitung bleibt sicher sichtbar als `Blocked` mit Ursache und Fortsetzungspfad. Keine Relaxed-Durability für Erase. Scratchmangel bleibt `Purging/Blocked`. `space_drop` friert nie wiederverwendete `SpaceId u64` und Mitgliedschaft ein, prüft Restrictions atomar und schreibt vor Gate pseudonymes `SpaceDropIntent`; Crypto auf Space-Ebene ist v1 unsupported, Range-Erase ist nie Crypto-Erase.

## 4 Invarianten / Umsetzung
INV-ERA-1…12, INV-ANCH-1, INV-LED-1. FaultFs-Sweep an jedem Persistenz-/Rename-/Unlink-Punkt; Modellzustand prüft nur Vorwärtskanten; Byte-Zähler für INV-ERA-11; Receipt-Fuzzer und kanonische Referenzvektoren.

## 5 Arbeitspakete
WP-13.1 atomarer Gate/Visibility (P1) · WP-13.2 Roll-forward ER4/ER6 (P2) · WP-13.3 Provider-Key-Batch/Probe (P6) · WP-13.4 Restrictions + Extension-Verifikation (P6) · WP-13.5 Receipt/Offline-Verifikation (P6) · WP-13.6 Deadline-/Metrikpfad (P6). Abnahme: alle Crash-Punkte konvergieren monoton zu sicherem Zustand.
