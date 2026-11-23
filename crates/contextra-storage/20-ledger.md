# SP-LEDGER — Erasure-Ledger, Segmente und Anker

| | |
|---|---|
| Modul | `erase/ledger.rs`, `env/anchor.rs` |
| Besitzt | append-only Ledger-Kette, `tomb_id`, Segmentseal/archive, Ankersequenzierung |
| Konsumiert | PORTS, CODEC, ENV, CATALOG, ERASE, ColdTier |
| Spec-Bezug | R8 §9.7, §10.3; INV-LED-1, INV-ANCH-1 |
| Phase | P1 Minimal ledger; P6 Segmente/Archiv (G-E) |

## 1 Auftrag
Ledger ist getrennt konfigurierbar (`ledger_dir`) und überlebt Datenvolume-Restore. Es speichert pseudonyme Tomb-IDs, Fortschritt, Epoch/Receipt-Daten und nur AEAD-verschlüsselte selektive Scope-Selectoren; niemals Klartext-ClusterId, SpaceId, Benutzerkeys oder Nutzdaten. `LedgerScopeKey` wird aus RootKey mit festem Label abgeleitet und ist für Restore erforderlich. Ledger/Anker haben je genau einen Writer.

## 2 Verträge
Append-Protokoll ist write-ahead für Erase: einzelner begrenzter `EraseIntent` (max. 4096 tomb_ids) oder `SpaceDropIntent`-Record (max. 256 KiB) wird vor Catalog-Gate durabel; Catalog-Gate und `GateApplied` folgen; Anchor-CAS folgt Gate und muss vor KeyDestroy/Purge fortgeschrieben sein. Vor KMS-Aufruf wird `KeyDestroyIntent` angehängt. `request_id_hmac=HMAC(ledger_key,EraseRequestId)` und HMAC des kanonischen Scopes machen Retry idempotent. Torn/unvollständige Records sind nicht angenommen; ein vollständig validierter und synchronisierter Intent ist angenommen und wird bei Retry über `request_id_hmac` idempotent wiedergefunden. `advance(expect,next)` monoton und CAS-durabel. Segmentrotation endet mit `seg_digest`; `ColdTier` archiviert nur versiegelte vollständige Segmente. `verify_receipt` bekommt Receipt plus erforderlichen Segment-Slice.

## 3 Regeln / Fehler
Anker höher als lokaler Zustand ⇒ F-9, nur Diagnose/ReadOnly. Kein Rollback des Ankers; kein Öffnen zum Schreiben mit Ledger hinter Anker. Fsync-Fehler nie wiederholen. Ledgerrecord-Decoder strikt kanonisch; unknown tag/version ⇒ Unsupported.

## 4 Invarianten / Umsetzung
INV-LED-1, INV-ANCH-1, INV-ERA-8. Crash-Sweep an Intent-Append/Sync, Catalog-Gate, GateApplied, Anchor, KeyDestroyIntent/KMS-Antwort; jeder Wiederanlauf convergiert vor Open. Monotonicity property; Canaryscan beweist keine Klartext-ID/Selectoren/Secrets.

## 5 Arbeitspakete
WP-20.1 Ledgerappend + Hashkette (P1) · WP-20.2 Rotation/seg_digest/ColdTier (P6) · WP-20.3 Ankerkoordination und Receipt-Slice (P6). Abnahme: kein Anker vor durabler Wahrheit und kein identifizierender Inhalt im Ledger.
