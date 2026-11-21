# contextra-mvcc — Status (Umbenennung zu contextra-version ausstehend)

**Ebene:** E1 (Ist), Ziel `contextra-version` (ADR-06)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

Dieses Dokument enthält nur den Status. Es ist keine Umbauspezifikation, weil die Umbenennung eine Freigabe des Nutzers verlangt (CLAUDE.md §6, Punkt 4). Keine Änderung am Code in dieser Sitzung.

## 1. Zweck und Ebene
Ist-Zweck: MVCC mit Sequenzlog, Transaktionspuffer, Versionskatalog. Soll nach SPEC 3.1, 15 und ADR-06: `contextra-version` mit Versionskatalog (REQ-VER-1), Snapshots (REQ-VER-2), Aufbewahrung (REQ-VER-3), `visibility` (`Visibility`, `ContentLifecycle`-Sender) und `error`. Ebene E1. Ziel-Umfang: 2.500 Zeilen (Ist laut Zielspalte: 8.159).

## 2. Vertrag
Soll (SPEC 4.4, 3.4): `Visibility` und `ContentLifecycle` im Modul `ports::version`. Implementierung von `Visibility` und dem Sender `ContentLifecycle` liegt in `version`. Ist: kein `Visibility`-Trait im Crate, siehe `contextra-ports`.

## 3. Abhängigkeiten
**Soll (Anhang E):** `ports`, `ids`, `env`.
**Ist (Cargo.toml):** `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports`, sowie `parking_lot`, `ahash`, `serde`, `bytes`, `thiserror`, `tracing`, `loom` (Dev).
**Ist-Abweichung:** `contextra-types` (R1, REM-30) und die Features `workspace`, `types-compat`, `ports-compat` (R7, CMP-10).

## 4. Module (Ist)
`src/snapshot.rs`, `src/ssi.rs`, `src/tombstone.rs`, `src/tx_buffer.rs`, `src/mvcc_error.rs`, `src/lib.rs`, `src/contract.rs`, `src/compat/`. `compat/` und `contract.rs` sind nach R7 und REM-14 zu entfernen. `ssi.rs` und `tx_buffer.rs` sind nach ADR-06 zu streichen (kein SSI, kein TxBuffer).

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| ADR-06 | SSI und TxBuffer | `grep -rn "ssi\|tx_buffer\|TxBuffer" crates/contextra-mvcc/src` leer |
| REM-12 | in SPEC 15 für `version` genannt, Inhalt in SPEC 12 nachlesen (offen) | siehe SPEC 12 |
| CMP-10 | Features `workspace`, `types-compat`, `ports-compat` | `grep -n "compat\|workspace" crates/contextra-mvcc/Cargo.toml` leer |

## 6. Umbau
Nicht in dieser Sitzung. Schritte nach Freigabe (CLAUDE.md §6, Punkt 4): Umbenennung zu `contextra-version`, Anpassung aller Abhängigen in `Cargo.toml`, dann Entfernungen aus Abschnitt 5.

**Offene Punkte (Entscheidung nötig):**
- **O-1 Freigabe der Umbenennung** (ADR-06). Viele Abhängige betroffen.
- **O-2 REM-12** nach SPEC 12 bestätigen.

## 7. Tests
T-ER-10, T-ER-11, REQ-VER-5 (Zeitreise mit `index_history`, SPEC 15). Ist-Befund: nicht gezählt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-07 | Import `contextra_types` (9 Dateien), Feature-Verweise | verletzt (REM-30) |
| G-09 | 33 Dateien mit ANCHOR/FILE-CONTEXT, `contract.rs` vorhanden | verletzt (REM-14) |
| G-11 | `compat/`-Verzeichnis, Features mit `compat`/`workspace` | verletzt (CMP-10) |
| G-13 | 21 Treffer | Abbau nach R6 |
