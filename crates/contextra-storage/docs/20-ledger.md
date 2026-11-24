# SP-LEDGER — Erasure-Ledger, Segmente und Anker

| | |
|---|---|
| Modul | `erase/ledger.rs`, `env/anchor.rs` |
| Besitzt | append-only Ledger-Kette, `tomb_id`, Segmentseal/archive, Ankersequenzierung |
| Konsumiert | PORTS, CODEC, ENV, CATALOG, ERASE, ColdTier |
| Spec-Bezug | R8.2 §9.7, §10.3, §10.4; INV-LED-1/2, INV-ANCH-1 |
| Phase | P1 Minimal ledger; P6 Segmente/Archiv (G-E) |

## 1 Auftrag
Ledger ist getrennt konfigurierbar (`ledger_dir`) und überlebt Datenvolume-Restore. Es speichert pseudonyme Tomb-IDs, Fortschritt, Epoch/Receipt-Daten und nur AEAD-verschlüsselte selektive Scope-Selectoren; niemals Klartext-ClusterId, SpaceId, Benutzerkeys oder Nutzdaten. `LedgerScopeKey` wird aus RootKey mit festem Label abgeleitet und ist für Restore erforderlich. Ledger/Anker haben je genau einen Writer.

## 2 Verträge
Append-Protokoll ist write-ahead für Erase: ein einzelner begrenzter `EraseIntent` (höchstens 4096 sortierte Tomb-IDs; max. 256 KiB) oder `SpaceDropIntent` wird vor dem Catalog-Gate vollständig geschrieben und synchronisiert. Danach folgen Catalog-Gate, `GateApplied` und Anchor-CAS; die irreversible Folgestufe darf erst beginnen, wenn der Anker Intent und Gate-Head enthält. Vor jedem KMS-Aufruf wird `KeyDestroyIntent` synchronisiert. Torn Records sind nicht angenommen; ein vollständig validierter, synchronisierter Intent ist angenommen und Recovery setzt ihn vor Freigabe von Views fort.

Kanonische, instanzgebundene Pseudonyme: `tomb_id = HMAC(ledger_key, "ctxs cluster tomb v1" || instance_id[16] || ClusterId[16])`; `space_tomb_id = HMAC(ledger_key, "ctxs space tomb v1" || instance_id[16] || be_u64(SpaceId))`. Beide sind linkbare Pseudonyme, keine anonymen Daten. `SpaceDropIntent` speichert `space_tomb_id`, HMAC-`scope_digest`, Count und Level; `scope_digest` bindet die vor Intent eingefrorene Space-Mitgliedschaft (sortierte Tomb-IDs) und ersetzt den Space-ID-Matcher bei Restore nicht.

`request_id_hmac = HMAC(ledger_key, "ctxs erase request id v1" || instance_id[16] || EraseRequestId[16])`; `request_digest = HMAC(ledger_key, "ctxs erase request digest v1" || canonical_erase_request)`, wobei `canonical_erase_request` versioniert, tag-/längencodiert und semantisch normalisiert ist (sortierte eindeutige Ziele, Level, semantische Optionen; keine Deadline/Retry-Metadaten). Gleiche ID+gleicher Digest liefert dasselbe Ticket, gleicher ID+anderer Digest ergibt `IdemConflict`. Scope-Digest und Tomb-HMACs nutzen ausschließlich die im normativen R8 §9.7 definierten Domain-Labels und Byteordnungen; alle HMAC-Labels plus `instance_id` verhindern protokollübergreifende Kollisionen. Ledger-Key-Epoche und `LedgerScopeKey` müssen über die gesamte Ledger-Retention verfügbar sein. `advance(expect,next)` ist monoton und CAS-durabel. Segmentrotation endet mit `seg_digest`; `ColdTier` archiviert nur versiegelte vollständige Segmente. `verify_receipt` bekommt Receipt plus erforderlichen Segment-Slice und Anchor-Attestation. `Complete` verlangt alle Level-Atteste; `CompleteWithResidual` nur explizit benannte, nicht vom Vertrag umfasste Restexposition; `Incomplete` bedeutet fehlender Owner-/Scope-Nachweis; `Blocked` ist sicher pausiert/fortsetzbar; `Failed` ist terminal.

## 3 Regeln / Fehler
Anker höher als lokaler Zustand ⇒ F-9, nur Diagnose/ReadOnly. Kein Rollback des Ankers; kein Öffnen zum Schreiben mit Ledger hinter Anker. Fsync-Fehler nie wiederholen. Ledgerrecord-Decoder strikt kanonisch; unknown tag/version ⇒ Unsupported.

## 4 Invarianten / Umsetzung
INV-LED-1, INV-ANCH-1, INV-ERA-8. Crash-Sweep an Intent-Append/Sync, Catalog-Gate, GateApplied, Anchor, KeyDestroyIntent/KMS-Antwort; jeder Wiederanlauf convergiert vor Open. Monotonicity property; Canaryscan beweist keine Klartext-ID/Selectoren/Secrets.

## 5 Arbeitspakete
WP-20.1 Ledgerappend + Hashkette (P1) · WP-20.2 Rotation/seg_digest/ColdTier (P6) · WP-20.3 Ankerkoordination und Receipt-Slice (P6). Abnahme: kein Anker vor durabler Wahrheit und kein identifizierender Inhalt im Ledger.

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §4.1, §9.7, §10.4; `INV-LED-2`, `INV-KEY-5` (Ledger-Seite); K-31; E-74, E-81.

1. **Format-Slot (vor G-F):** `tomb_epoch u32` in jedem Event mit `tomb_id`; `tomb_id = HMAC(tomb_key(epoch), "ctxs cluster tomb v1" || instance_id || ClusterId)`; neue Event-Kinds `TombEpochOpened`, `TombEpochClosed`, `TombEpochRetireIntent{epoch, restore_floor}`, `TombEpochRetired`. Die Kette nutzt weiter `ledger_key`; `space_tomb_id` bleibt unter `ledger_key`. Referenzvektoren: zwei Epochen, ein Retire (SPEC Anhang A).
2. **Retire-Ablauf (§10.4):** Bedingungen prüft der Ledger-Writer (Epoche `Closed`, `now − closed_at ≥ backup_horizon` bei verifizierter Uhr, keine offenen Tickets/Intents/Restores mit dieser Epoche). Reihenfolge: `RetireIntent` durabel < Anker < `retire_tomb_epoch` = `Absent` < `Retired`; Roll-forward nach Absturz.
3. **Geräte-/Verfügbarkeitsregeln:** `LedgerSameDevice` (Production) und `LedgerUnavailable` (schreibendes Open) laut §9.7/§10.1.
4. `restore_floor` ist Teil des Ledger-Zustands und wird von RECOVERY/CKPT gelesen.

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-20.4 | Tomb-Epochen: Format-Slot, Events, Rotation, Retire-Ablauf, `restore_floor`, Same-Device-/Unavailable-Prüfung | 2.5 | WP-20.1, WP-04.7 | Slot vor G-F, Ablauf P6 | Goldene Vektoren, Crash-Matrix des Retire, `INV-LED-2`, `verify_receipt` ohne Tomb-Schlüssel |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

R9.1 ändert das Ledger-Format nicht. Die Terminologie verwendet OP1…OP10; die Reihenfolge bleibt Ledger-Intent < Gate < `GateApplied`/Anchor < KeyDestroy < Purge. Der Ledger-Writer protokolliert außerdem Owner-Timeouts und hält solche Tickets `Blocked`; ein fehlendes Attest kann nicht als `CompleteWithResidual` maskiert werden, wenn es zum Vertrag gehört.
