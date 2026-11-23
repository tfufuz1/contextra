# SP-LEDGER — Erasure-Ledger, Segmente und Anker

| | |
|---|---|
| Modul | `erase/ledger.rs`, `env/anchor.rs` |
| Besitzt | append-only Ledger-Kette, `tomb_id`, Segmentseal/archive, Ankersequenzierung |
| Konsumiert | PORTS, CODEC, ENV, CATALOG, ERASE, ColdTier |
| Spec-Bezug | R8 §9.7, §10.3; INV-LED-1, INV-ANCH-1 |
| Phase | P1 Minimal ledger; P6 Segmente/Archiv (G-E) |

## 1 Auftrag
Ledger ist getrennt konfigurierbar (`ledger_dir`) und überlebt Datenvolume-Restore. Es speichert ausschließlich `tomb_id`, Fortschritt, Epoch/Receipt-Daten; niemals ClusterId, Nutzdaten oder Nutzerschlüssel. Ledger/Anker haben je genau einen Writer.

## 2 Verträge
Append-Datensatz wird vor Ankerfortschreibung durabel. Anker `advance(expect,next)` monoton und CAS-durabel; Catalog/Ledger-Edit zuerst, Anker danach. Segmentrotation endet mit `seg_digest`; `ColdTier` archiviert nur versiegelte vollständige Segmente. `verify_receipt` bekommt Receipt plus erforderlichen Segment-Slice.

## 3 Regeln / Fehler
Anker höher als lokaler Zustand ⇒ F-9, nur Diagnose/ReadOnly. Kein Rollback des Ankers; kein Öffnen zum Schreiben mit Ledger hinter Anker. Fsync-Fehler nie wiederholen. Ledgerrecord-Decoder strikt kanonisch; unknown tag/version ⇒ Unsupported.

## 4 Invarianten / Umsetzung
INV-LED-1, INV-ANCH-1, INV-ERA-8. Crash-Sweep zwischen Edit, Ledger append, segment seal, anchor advance; monotonicity property; Canaryscan für ClusterId/Secrets.

## 5 Arbeitspakete
WP-20.1 Ledgerappend + Hashkette (P1) · WP-20.2 Rotation/seg_digest/ColdTier (P6) · WP-20.3 Ankerkoordination und Receipt-Slice (P6). Abnahme: kein Anker vor durabler Wahrheit und kein identifizierender Inhalt im Ledger.
