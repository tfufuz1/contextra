# contextra-rank — Implementierungsspezifikation

**Ebene:** E1 (Kern, R2)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Rang und Fusion: RRF, Top-k, Zeitgewichtung (`valid_at`), Erklärung (SPEC 15, REM-18). Feature `std` ist laut R3 begründete Ausnahme (SPEC 3.2, 12.6). Ziel-Umfang: 4.000 Zeilen (SPEC 3.1). Ist: 10.809 Zeilen laut Zielspalte (Altbestand).
Abhängigkeiten nach Anhang E: `ports`, `ids`.

## 2. Vertrag
Keine Port-Implementierung nach SPEC 3.4. `Reranker` (6.2) liegt im Ports-Crate, Implementierung im Adapter `contextra-embed`. Das Crate liefert Rangfunktionen, die über DTO-Typen aus `ports::dto` arbeiten (SPEC 15).

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `ports`, `ids`.
**Ist (Cargo.toml):** **`contextra-types`** (**verboten**, R1, REM-30, 6 Dateien), `contextra-ports` (erlaubt), `serde`, `serde_json`, `ahash`, `tracing`, `thiserror`. `contextra-ids` fehlt in Ist, obwohl erlaubt und nötig.
**Ist-Abweichung:** `contextra-text` wird in `src` referenziert (Messung dieses Abgleichs). Das ist eine Kante E1 → E1 und verletzt R2 (Anhang E, rank hat keine E1-Abhängigkeit). Prüfen, Befund siehe 6.
**Features:** `default = [\"std\"]` (R3 Ausnahme). Ist-Abweichung: `workspace`, `types-compat`, `ports-compat` (R7, CMP-6).

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/fusion/rrf.rs`, `src/fusion/mod.rs` | Reciprocal Rank Fusion | Soll (RRF) |
| `src/fusion/mrrf.rs`, `src/fusion/normalized.rs`, `src/fusion/global.rs`, `src/fusion/ext.rs`, `src/fusion/resonance.rs`, `src/fusion/provenance.rs`, `src/fusion/signal*` | weitere Fusionsverfahren | nicht in SPEC 15 genannt, offen |
| `src/explain.rs` | Erklärung | Soll |
| `src/calibration/*` (conformal, isotonic, platt) | Kalibrierung | nicht in SPEC 15 genannt, offen |
| `src/dibud/*` | DIBUD-Treiber und Zustand | nicht in SPEC 15 genannt, offen |
| `src/drift.rs` | Drift-Erkennung | nicht in SPEC 15 genannt, offen |
| `src/compat/*` (clock, types, error) | Kompatibilitätsmodul | **entfernen** (CMP-6, R7, G-11). `clock.rs` entfällt laut CMP-6, `Clock` kommt aus `env` |
| `src/contract.rs` | Selbstdeklaration | REM-14 |
| `src/error.rs`, `src/lib.rs` | Fehler, Wurzel | Soll |

Zeitgewichtung (`valid_at`) und Top-k sind in SPEC 15 genannt, im Ist nicht eindeutig zugeordnet (prüfen).

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| CMP-6 | `src/compat/`, `src/compat/clock.rs` | `find crates/contextra-rank -type d -name compat` leer |
| REM-06 | in SPEC 15 genannt, Inhalt in SPEC 12 nachlesen (offen) | siehe SPEC 12 |
| REM-18 | dito (Zeitgewichtung/Fusion, SPEC 12 prüfen) | siehe SPEC 12 |
| REM-30 | Import `contextra-types` | `grep -rln "contextra_types" crates/contextra-rank` leer |
| REM-14 | ANCHOR-Dateien (2), `contract.rs` | `grep -rl "ANCHOR\[" crates/contextra-rank` leer |

## 6. Umbau
1. `compat/` auflösen (CMP-6). `Clock` aus `contextra-env` beziehen.
2. Typen aus `contextra-types` nach `ports::dto` verlagern (REM-30).
3. Kante zu `contextra-text` klären. Entweder entfernen oder über Ports anbinden. Entscheidung nötig.
4. Features `workspace`, `types-compat`, `ports-compat` entfernen.

**Offene Punkte (Entscheidung nötig):**
- **O-1** `dibud/`, `calibration/`, `drift.rs` und weitere Fusionsverfahren: nicht in SPEC 15. Behalten, verlagern oder entfernen?
- **O-2** Kante `rank → text` (R2-Verletzung, Anhang E). Auflösung festlegen.
- **O-3** Inhalt von REM-06 und REM-18 nach SPEC 12 bestätigen.

## 7. Tests
- Determinismus bei gleichen Eingaben, Tie-Break (REQ-SRC-1, Schritt 7, SPEC 15).
- Ist: nicht gezählt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-03 | verbotene Abhängigkeit `contextra-types`, Kante zu `text` | verletzt |
| G-07 | Import `contextra_types` | verletzt (REM-30) |
| G-09 | 2 Dateien mit ANCHOR, `contract.rs` | verletzt (REM-14) |
| G-11 | `compat/`, Features | verletzt (CMP-6, CMP-10) |
| G-12 | 2 Treffer | verletzt |
| G-13 | 0 Treffer in `src` | eingehalten |
