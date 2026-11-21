# contextra-db — Implementierungsspezifikation

**Ebene:** E2 (Orchestrierung)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Orchestrierung: Schreib-Task je Shard, Tenant-Koordinator, Commit-Lock, Erasure-Lock, Startsequenz (8.3), Scheduler, Latenzfenster (ADR-43), Verdrahtung der Ports (3.4, 7.13) (SPEC 15, Kapitel 8). Features nach SPEC 15: `graph`, `text`, `vector`, `embed`, `ingest` (Standard alle außer `graph`). Ziel-Umfang: 6.000 Zeilen (SPEC 3.1). Ist: 412 Zeilen in `src` (laut `CLAUDE.md`, 21.11.2026). Der Ist-Stand ist dünn und muss ausgebaut werden.

## 2. Vertrag
Der Crate implementiert keine Ports. Er verdrahtet sie (SPEC 3.4). Die öffentliche API ist die Schnittstelle zur Fassade `contextra`. Ist: `lib.rs`, `erasure.rs`, `lineage.rs`, `storage.rs`. Die Verdrahtung ist im Ist nicht vollständig (Abhängigkeit zu `checkpoint` und `mvcc` ist im Ist vorhanden, siehe 3).

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `contextra-crypto`, `contextra-erasure`, `contextra-version`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-rank`, `contextra-privacy`, `contextra-checkpoint`, `contextra-ports`, `contextra-ids`, `contextra-env`, `contextra-ingest`, `contextra-embed`.
**Ist (Cargo.toml):** alle E1-Crates außer `version` (im Ist `contextra-mvcc`), dazu `contextra-ids`, `contextra-env`, `contextra-ports`, `contextra-durable-fs`, **`contextra-types`** (verboten, R1, REM-30, 2 Dateien).
**Ist-Abweichungen:** `contextra-durable-fs` ist in Anhang E für `db` nicht erlaubt. `contextra-mvcc` ist noch der Ist-Name (ADR-06). `contextra-ingest` und `contextra-embed` fehlen im Ist-Workspace-Stand von `db`.
**Features:** Ist nur `default`. Soll: `graph`, `text`, `vector`, `embed`, `ingest` (SPEC 15, 9). Ohne Feature `graph` muss `db` ohne Graph-Code kompilieren (T-GR-12).

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/lib.rs` | Wurzel, Verdrahtung | Soll, zu erweitern |
| `src/erasure.rs` | Anbindung Löschung | Soll (Tenant-Koordinator, Erasure-Lock) |
| `src/storage.rs` | Anbindung Speicher | Soll (Commit-Lock, Schreibpfad) |
| `src/lineage.rs` | Herkunftsverfolgung | nicht in SPEC 15 genannt, offen |

Soll-Module ohne Ist-Entsprechung: Schreib-Task je Shard, Scheduler, Startsequenz (8.3), Latenzfenster (ADR-43), Commit-Lock, Lesepfad, Schreibpfad. Sie werden neu aufgebaut.

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| REM-30 | Import `contextra-types` (2 Dateien) | `grep -rln "contextra_types" crates/contextra-db` leer |
| Verweis auf entfernten Crate (REM-19) | 1 Datei in `src` verweist auf den entfernten Crate-Namen | `grep -rn "engine" crates/contextra-db/src` nach Prüfung leer |
| REM-14 | ANCHOR-Dateien (0), `contract.rs` (0) | — (bereits erfüllt) |

## 6. Umbau
1. `contextra-types` aus `Cargo.toml` und Quellen entfernen (REM-30).
2. `contextra-mvcc` → `contextra-version` (ADR-06, Freigabe nötig, CLAUDE.md §6 Punkt 4).
3. Abhängigkeit `contextra-durable-fs` klären: nach Anhang E nicht erlaubt.
4. Features `graph`, `text`, `vector`, `embed`, `ingest` anlegen (SPEC 9, 15).
5. Crate `contextra-ingest` und `contextra-embed` anlegen oder bestätigen (Anhang E). Ohne diese Crates kein vollständiger Schreibpfad.
6. Verdrahtung nach 3.4 und 7.13. Ein Umbauschritt je Commit, eigene Sitzung je Schritt (CLAUDE.md §8).

**Offene Punkte (Entscheidung nötig):**
- **O-1** Erlaubnis für `contextra-durable-fs` in `db` (Anhang E sagt nein, Ist nutzt es).
- **O-2** Ziel-Umfang 6.000 gegenüber Ist 412: Der Umfang entsteht erst durch den Umbau. Priorität mit dem Nutzer klären (CLAUDE.md §6 Punkt 2).
- **O-3** `lineage.rs`: nicht in SPEC 15.

## 7. Tests
T-OPS-01 bis T-OPS-10, T-SEC-03, T-RAG-11, T-ER-07 (SPEC 15). Ist: keine dieser Tests gefunden, Zählung nicht erfolgt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | `forbid(unsafe_code)` in `lib.rs` | eingehalten |
| G-03 | `contextra-types`, `contextra-durable-fs`, `contextra-mvcc` | verletzt |
| G-07 | Import `contextra_types` (2) | verletzt (REM-30) |
| G-09 | keine ANCHOR-Dateien, kein `contract.rs` | eingehalten |
| G-13 | 0 Treffer | eingehalten |
