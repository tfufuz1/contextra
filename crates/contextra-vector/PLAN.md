# contextra-vector — Implementierungsspezifikation

**Ebene:** E1 (Kern, R2)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Vektorindex: HNSW (`hnsw/`, Graph, Delta, Persistenz nach REQ-VEC-4), exakte Suche mit SIMD (`exact`), SQ8 mit Kalibrierung (`sq8`, REQ-VEC-1a), ACORN-Filtertraversierung (`acorn`), Planer (`planner`, REQ-FLT-3), Scrub (`scrub`, `ScrubTarget`, REQ-VEC-5), Port `VectorIndex`, Typen, Fehler (SPEC 15, 6.3). Ziel-Umfang: 11.000 Zeilen (SPEC 3.1). Ist: 34.675 Zeilen laut Zielspalte (Altbestand). **DiskANN und RaBitQ entfallen** (ADR-04, SPEC 3.1).
Abhängigkeiten nach Anhang E: `ports`, `ids`, `env`, `simd`, `durable-fs`.

## 2. Vertrag
Implementiert (SPEC 3.4): `VectorIndex` und `ScrubTarget`. Ist: `VectorIndex` ist in `contextra-ports` (`vector_index.rs`) definiert. Die Ist-Signatur ist gegen SPEC 4.4 und 15 zu prüfen. `ScrubTarget` fehlt im Ports-Crate.

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `ports`, `ids`, `env`, `simd`, `durable-fs`.
**Ist (Cargo.toml):** `contextra-ids`, `contextra-env`, `contextra-durable-fs`, `contextra-simd` (erlaubt, optional), `contextra-ports` (erlaubt), dazu **`contextra-types`** (**verboten**, R1 und REM-30). Weitere: `roaring`, `subtle`, `ahash`, `parking_lot`, `rand`, `tracing`, `serde`, `zeroize`, `thiserror`, `blake3`.
**Ist-Abweichung:** `contextra-types` wird in 86 Dateien importiert (Messung dieses Abgleichs). Das ist der größte Befund des Crates.
**Features:** Ist `default = []`, `std`, `simd`, `serde`, `tracing`, `inspect`, `experimental`, `mmap`, `diskann`, `quantize-rabitq`, `experimental-diskann`, `experimental-rabitq`, `experimental-predicate-augmented-search`, `graph`, `experimental-all`, `partial-index-rebuild`, `workspace`, `ports-compat`, `docid-128`. Soll (SPEC 15): keine Features außer den in Kapitel 15 genannten. Ist-Abweichung: alle `experimental-*`, `diskann`, `quantize-rabitq` (ADR-04: raus), `workspace`, `ports-compat` (CMP-10).

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/hnsw/*` | HNSW-Index | Soll `hnsw/` |
| `src/acorn/*` (gamma_augmentation, naive_reference, mod) | ACORN-Traversierung | Soll `acorn` |
| `src/diskann/*` | DiskANN | **entfernen** (ADR-04) |
| `src/compat/*` | Kompatibilitätsmodul (Kandidaten-Stream, DiskANN-/HNSW-Adapter, Purge, Scanner, Typen) | **entfernen** (R7, CMP-8, G-11); Inhalte in Soll-Module verschieben |
| `src/contract.rs` | Selbstdeklaration | REM-14 |
| `src/casting.rs`, `src/compute_pool.rs` | Hilfen | zu prüfen |

Soll-Module ohne Ist-Entsprechung: `exact` (SIMD), `sq8` (Kalibrierung), `planner`, `scrub`, `index`, `types`, `error` (SPEC 15). Zu prüfen, ob im Ist unter anderem Namen vorhanden.

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| ADR-04 / REM-09 | `src/diskann/*`, Features `diskann`, `experimental-diskann`, `quantize-rabitq`, `experimental-rabitq` | `ls crates/contextra-vector/src/diskann` nicht vorhanden |
| REM-23 | laut SPEC 15 genannt, Inhalt in SPEC 12 nachlesen (offen) | siehe SPEC 12 |
| CMP-8 | `src/compat/` | `find crates/contextra-vector -type d -name compat` leer |
| REM-30 | Import `contextra-types` (86 Dateien) | `grep -rln "contextra_types" crates/contextra-vector` leer |
| CMP-10 | Features `workspace`, `ports-compat` | `grep -n "compat\|workspace" crates/contextra-vector/Cargo.toml` leer |
| REM-14 | ANCHOR-Dateien (185), `contract.rs` | `grep -rl "ANCHOR\[" crates/contextra-vector` leer |

## 6. Umbau
1. `diskann/` und die experimentellen Features entfernen (ADR-04). Grenze: Vor dem Entfernen prüfen, ob `compat/` davon abhängt.
2. `compat/` auflösen, in `hnsw/`, `exact`, `planner` verschieben (CMP-8).
3. Typen aus `contextra-types` in `ports::dto` oder `ids` verlagern (REM-30, 86 Dateien). Eigene Sitzung je Schritt.
4. `unwrap`/`expect`/`panic!` (162 Treffer) nach R6 abbauen.

**Offene Punkte (Entscheidung nötig):**
- **O-1** REM-23 und die Dateien in `casting.rs`, `compute_pool.rs`: Zuordnung ungeklärt.
- **O-2** Feature `graph` im Vector-Crate: nicht in SPEC 15 genannt.
- **O-3** Freigabe für die Umbaureihenfolge (große Änderung, CLAUDE.md §8).

## 7. Tests
- T-RAG-04, T-RAG-05, T-IDX-01, T-ER-03 (SPEC 15).
- Ist-Befund: nicht gezählt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | `forbid(unsafe_code)` in 2 Dateien, kein `unsafe`-Gebrauch im Crate | eingehalten |
| G-03 | `contextra-types` (R1) | verletzt (REM-30) |
| G-07 | Erwähnung `contextra-types` in 86 Dateien | verletzt |
| G-09 | 185 Dateien mit ANCHOR/FILE-CONTEXT, `contract.rs` | verletzt (REM-14) |
| G-11 | `compat/`-Verzeichnis, Features `workspace`, `ports-compat` | verletzt (CMP-8, CMP-10) |
| G-13 | 162 Treffer | Abbau nach R6 |
