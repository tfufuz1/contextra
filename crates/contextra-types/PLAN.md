# contextra-types — Auflösungsplan (kein Zielcrate)

**Ebene:** keine Zielebene. Das Crate wird nach SPEC aufgelöst (REM-30, ADR-33).
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch)
**Stand:** 21.11.2026, Abgleich mit SPEC v4.1

Dieses PLAN.md folgt der Vorlage 13.5 in der Form eines Auflösungsplans. Es gibt kein Soll-Crate, deshalb wird der Vertrag als Verteilungstabelle geführt. Abweichungen des Ist-Zustands sind als **Ist-Abweichung** markiert, ungeklärte Punkte stehen unter **Offene Punkte (Entscheidung nötig)** in Abschnitt 6.

---

## 1. Zweck und Ebene

`contextra-types` ist ein Querschnitts-Crate ohne Besitzer (SPEC F-08). Es erzeugt Kopplung über fast alle Crates. Nach ADR-33 wird es aufgelöst: IDs nach `contextra-ids`, DTOs und Port-Typen nach `contextra-ports`, der Rest zum jeweiligen Besitzer (SPEC 3.4).

Ziel: kein Workspace-Mitglied `contextra-types` mehr, keine Importe `contextra_types` im Code (REM-30, G-07, SPEC 3.1).

Ist: 7.532 Zeilen `src`, 14 Testdateien, 188 `#[test]` (laut `CLAUDE.md`, 21.11.2026). Abhängig von `contextra-ids` und mehreren externen Crates.

## 2. Vertrag

Es gibt keine Zielschnittstelle. Jeder Typ wird dem Besitzer zugeordnet. Die Zuordnung gilt, sobald SPEC eine Zielstelle nennt. Sonst steht der Eintrag unter Offene Punkte.

| Ist-Datei | Inhalt (Ist) | Zielbesitzer | Quelle |
|---|---|---|---|
| `src/ids.rs`, `src/types/domain/ids.rs`, `src/refs.rs`, `src/tenant_scope.rs` | IDs, Referenzen, Mandantenbindung | `contextra-ids` | SPEC 15, ADR-33 |
| `src/error.rs` | Fehlertypen `ContextraError` | `contextra-ports` | SPEC 15 (`ContextraError`) |
| `src/error_dto.rs` | serialisierbare Fehler-DTO | `contextra-ports::dto` | SPEC 15 (DTO-Modul), ADR-33 |
| `src/erasure.rs` | Löschtypen, typisierter Zeuge | `contextra-ports::erasure` bzw. `contextra-erasure` | SPEC 4.4, 15 |
| `src/witness.rs` | Restmessung, Löschnachweise | offen (siehe 6) | — |
| `src/types/budget.rs` | Ressourcen- und Token-Budget | `contextra-ports::dto` (`IoBudget`) oder offen | SPEC 15 |
| `src/types/filter.rs` | Metadaten-Filter-DSL | offen (siehe 6) | — |
| `src/types/saos.rs` | Hybrid-Query und Signal-Fusion | `contextra-ports::dto` (`SearchQuery`) oder `contextra-rank` | SPEC 15 |
| `src/types/importance.rs` | Relevanz und Recency-Decay | `contextra-rank` | SPEC 15 (Zeitgewichtung) |
| `src/types/domain/misc.rs` | Entity, Edge, `MemoryType`, `WorkflowState`, Verfallskonstanten | `contextra-graph` (Entity/Edge) gilt nur für diese Typen; `MemoryType`/`WorkflowState` **entfernen** (REM-24) | SPEC 12.2 REM-24 |
| `src/types/domain/document.rs` | Dokumente, Chunks, Embeddings, Metriken | offen (siehe 6) | — |
| `src/schema.rs` | Schema-Versionierung für SSTable und WAL | `contextra-store` | SPEC 5.10 (Plattenformate), offen |
| `src/tombstone.rs` | Tombstone-Semantikprüfung | offen (siehe 6) | — |
| `src/types/domain/mod.rs`, `src/types.rs`, `src/lib.rs` | Modulwurzeln, Re-Exporte | entfallen mit dem Crate | REM-30 |
| `src/model_fingerprint.rs` | `ModelFingerprint` | offen (siehe 6) | — |
| `src/retrieval_strategy.rs` | Retrieval-Strategie (Spec §9.5) | offen (siehe 6) | — |
| `src/tenant_scope.rs` | `TenantScoped`, Scope-Verletzungen | `contextra-ids` (vorläufig) | ADR-33 |

Tests und Beispiele: `tests/` (14 Dateien), `examples/standalone.rs`, `benches/types_bench.rs`. Sie folgen dem Zielbesitzer jeweils ihres Typs.

## 3. Abhängigkeiten

**Ist (Cargo.toml):** `contextra-ids`, `thiserror`, `serde`, `serde_json`, `bincode`, `ahash` (mit Feature `serde`), `blake3`, `sha2`, `bytes`.
**Dev:** `proptest`, `criterion`.
**Features:** `default = []`, `docid-128` (`contextra-ids/docid-128`), `auto-extraction-opt-out`.

**Soll:** Das Crate hat nach REM-30 keine Abhängigkeitsvorgabe mehr, weil es verschwindet. Abhängigkeiten gehen an die Zielbesitzer. Keine der Ist-Abhängigkeiten darf in `contextra-ports` übernommen werden, außer SPEC 15 nennt sie (siehe PLAN von `contextra-ports`, Offene Punkte O-3).

**Ist-Abweichung:** `ports` und `rank` und mehrere weitere Crates nutzen `contextra-types`. Das ist der Hauptbefund für R1 (ports → types, REM-27).

## 4. Module

Die Module werden nicht neu gegliedert. Sie gehen in der Verteilung aus Abschnitt 2 auf die Zielcrates über. Befunde, die bei der Verteilung entstehen:

- `src/types/domain/misc.rs`: Teile fallen weg (REM-24), der Rest verteilt sich.
- `src/types/domain/tests/` und `tests/`: Tests folgen den Typen.
- `src/types/budget.rs` (709 Zeilen) und `src/error.rs` (1.186 Zeilen) sind die größten Dateien. Sie werden nicht getrennt, bevor der Zielbesitzer feststeht.

## 5. Entfernungen

| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-30 | Crate `contextra-types` als Ganzes | `ls crates/contextra-types` leer; `grep -rn "contextra_types\|contextra-types" crates` leer |
| REM-24 | `MemoryType`, `WorkflowState`, Zerfallskonstanten, `crates/contextra-types/src/types/domain/misc.rs` | `grep -rn "MemoryType\|WorkflowState" crates` leer |
| REM-27 | Verletzung R1 (`ports → types`), `simd → types` | `python3 tools/check_spec_sync.py layering` ohne Befund dafür |
| REM-14 | Selbstdeklarationen (ANCHOR, FILE-CONTEXT) in 17 Dateien | `grep -rl "ANCHOR\[\|FILE-CONTEXT" crates/contextra-types` leer |

## 6. Umbau

Schritte, je Schritt ein Commit, Workspace grün. Die Auflösung ist eine Querschnittsaufgabe und braucht eine Freigabe des Nutzers (CLAUDE.md §8, Punkt 4 und „Bekannte Zuständigkeitsprobleme“).

1. Verteilungstabelle aus Abschnitt 2 bestätigen (Entscheidung des Nutzers für jede Zeile „offen“).
2. IDs nach `contextra-ids` (ADR-33).
3. Fehlertypen und DTOs nach `contextra-ports` (Ports-Schritt 1, siehe dort).
4. Pro Besitzer den Rest verschieben. Jeder Besitzer-Umbau ist eine eigene Sitzung (CLAUDE.md §8).
5. `misc.rs` teilweise entfernen (REM-24).
6. `contextra-types` aus `Cargo.toml`-Workspace-Mitgliedern und `[workspace.dependencies]` entfernen, sobald nichts mehr importiert.

**Offene Punkte (Entscheidung nötig):**
- **O-1 Zielbesitzer.** `witness.rs`, `filter.rs`, `document.rs`, `tombstone.rs`, `model_fingerprint.rs`, `retrieval_strategy.rs`, `schema.rs`, `budget.rs`: SPEC 3.4 und 15 nennen keinen Besitzer. Entscheidung des Nutzers.
- **O-2 `MemoryType`/`WorkflowState`.** REM-24 entfernt sie. Ist die Entfernung vollständig oder gehört ein Rest zu `contextra-graph`?
- **O-3 Umbenennung von `contextra-mvcc` zu `contextra-version`** (ADR-06). Betrifft viele `Cargo.toml`. Freigabe des Nutzers (CLAUDE.md §6, Punkt 4).
- **O-4 `contextra-erasure` und `contextra-types`.** `erasure.rs` in `types` hat Löschtypen. Die Verteilung auf `ports::erasure` oder `contextra-erasure` ist zu entscheiden.

## 7. Tests

- Serde-Roundtrip über Domänentypen (`tests/serde_roundtrips_suite.rs`), Typ-Invarianten (`tests/newtype_invariants_suite.rs`), Fehlerpfade (`tests/error_and_witness_suite.rs`).
- Golden-Daten (`tests/golden_bytes_suite.rs`, `tests/golden_ids_parity.rs`, `tests/generate_golden_ids.rs`): Format-Stabilität. Die Golden-Daten gehen mit den Typen an den Zielbesitzer.
- Die Tests folgen den Typen. Ein Test wird nicht in ein Zielcrate kopiert, wenn er nur den alten Gesamtbestand prüft.

## 8. Abnahme

Das Crate ist abgenommen, wenn alle Schritte aus Abschnitt 6 erledigt sind und die Befehle aus Abschnitt 5 leer liefern. Zusätzlich gelten:

| Gate | Ist-Befund | Soll |
|---|---|---|
| G-01 | Build laut `CLAUDE.md` ok | grün, bis das Crate entfernt ist |
| G-02 | `#![forbid(unsafe_code)]` in `lib.rs`; kein echtes `unsafe` | eingehalten |
| G-07 | Import `contextra_types` in vielen Crates | verletzt, bis REM-30 erledigt ist |
| G-09 | 17 Dateien mit ANCHOR/FILE-CONTEXT | verletzt (REM-14) |
| G-11 | keine `compat`-Verzeichnisse | eingehalten |
| G-13 | 133 Treffer `unwrap`/`expect`/`panic!` in `src` (grob) | Abbau nach R6 bei der Verteilung |
