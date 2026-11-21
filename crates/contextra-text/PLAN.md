# contextra-text — Implementierungsspezifikation

**Ebene:** E1 (Kern, R2)
**Normative Quelle:** `docs/SPEC.md` v4.1, Nachtrag v4.1.1 (maßgeblich bei Widerspruch). **Stand:** 21.11.2026, Abgleich mit SPEC v4.1. Verweist nur auf SPEC-Kapitel und IDs. Ist-Abweichungen sind markiert, ungeklärte Punkte stehen unter „Offene Punkte (Entscheidung nötig)“ in Abschnitt 6.

## 1. Zweck und Ebene
Textsuche: Tokenizer (REQ-TXT-1a), Stoppwörter (`stopwords/`, Daten), BM25, WAND, invertierter Index (Segmente), `TextIndex`, Merge, Scrub (`ScrubTarget`, Postings), Discovery (REQ-TXT-3), Typen, Fehler (SPEC 15, 6.4). Ziel-Umfang: 10.000 Zeilen (SPEC 3.1). Ist: 18.176 Zeilen laut Zielspalte (Altbestand).
Abhängigkeiten nach Anhang E: `ports`, `ids`, `env`, `durable-fs`.

## 2. Vertrag
Implementiert (SPEC 3.4): `TextIndex` und `ScrubTarget` für Postings. Ist: `TextIndex` ist in `contextra-ports` (Datei text_index.rs) definiert, `ScrubTarget` fehlt dort (siehe PLAN von `contextra-ports`).

## 3. Abhängigkeiten
**Erlaubt (Anhang E):** `ports`, `ids`, `env`, `durable-fs`.
**Ist (Cargo.toml):** erlaubt sind `contextra-ids`, `contextra-env`, `contextra-ports`. **Verboten:** `contextra-types` (R1, REM-30, 29 Dateien). Weitere: `regex`, `unicode-segmentation`, `bincode`, `serde`, `serde_json`, `thiserror`, `parking_lot`, `tracing`, `blake3`, `futures`, `bytes`, `itoa`.
**Ist-Abweichung:** `contextra-durable-fs` fehlt in der Ist-Liste, obwohl Anhang E es erlaubt.
**Features:** Ist `default`, `std`, `serde`, `tracing`, `inspect`, `experimental`, `workspace`, `types-compat`, `ports-compat`, `bm25f`, `docid-128`. Soll: keine Features außer SPEC 15. Ist-Abweichung: `workspace`, `types-compat`, `ports-compat` (R7, CMP-10). `experimental` und `bm25f` sind in SPEC 15 nicht genannt.

## 4. Module
| Datei | Verantwortung (Ist) | Befund |
|---|---|---|
| `src/bm25.rs` | BM25 | Soll |
| `src/inverted.rs`, `src/inverted/*` | Invertierter Index | Soll `inverted` |
| `src/domain/legal_de.rs`, `src/domain/medical_de.rs`, `src/domain/mod.rs` | Fachwortlisten für Deutsch | nicht in SPEC 15 genannt, offen |
| `src/memory*` | Speicherbezug | nicht in SPEC 15 genannt, offen |
| `src/compat/*` (inverted_ports, morph_index, scorer, storage, purge, scanner, types, tests) | Kompatibilitätsmodul | **entfernen** (R7, CMP-3, G-11); Inhalte in Soll-Module |
| `src/contract.rs` | Selbstdeklaration | REM-14 |
| `src/error.rs`, `src/lib.rs` | Fehler, Wurzel | Soll |

Soll-Module ohne Ist-Entsprechung: `tokenize` (REQ-TXT-1a), `wand`, `merge`, `scrub`, `discovery` (REQ-TXT-3), `index`, `types`. Zu prüfen, ob unter anderem Namen vorhanden.

## 5. Entfernungen
| ID | Gegenstand | Abnahme-Befehl |
|---|---|---|
| CMP-3 | `src/compat/`, `mock.rs` aus CMP-3 | `find crates/contextra-text -type d -name compat` leer |
| REQ-TXT-1a | Komposita-Code | `grep -rni "kompositum\|compound" crates/contextra-text/src` prüfen, offen |
| REM-30 | Import `contextra-types` (29 Dateien) | `grep -rln "contextra_types" crates/contextra-text` leer |
| CMP-10 | Features `workspace`, `types-compat`, `ports-compat` | `grep -n "compat\|workspace" crates/contextra-text/Cargo.toml` leer |
| REM-14 | ANCHOR-Dateien (52), `contract.rs` | `grep -rl "ANCHOR\[" crates/contextra-text` leer |

## 6. Umbau
1. `compat/` auflösen (CMP-3) und Typen aus `contextra-types` verlagern (REM-30).
2. Features `workspace`, `types-compat`, `ports-compat` entfernen (CMP-10).
3. Abhängigkeit `contextra-durable-fs` herstellen, falls Persistenz über Dateien läuft.
4. `unwrap`/`expect`/`panic!` (46 Treffer) nach R6 abbauen. `Instant::now`/`SystemTime::now` (4) auf `Clock` umstellen (G-12).

**Offene Punkte (Entscheidung nötig):**
- **O-1** `src/domain/legal_de.rs`, `medical_de.rs`: Fachwortlisten, nicht in SPEC 15. Behalten oder entfernen?
- **O-2** Features `experimental`, `bm25f`, `inspect`: nicht in SPEC 15.
- **O-3** Inhalt von REQ-TXT-1a (Komposita) nach SPEC 6.4 bestätigen.

## 7. Tests
- T-RAG-03, T-ER-02, Metrik F-25 (SPEC 15).
- Ist: `src/compat/tests/*` (zu entfernen, Inhalte umziehen). Zählung nicht erfolgt.

## 8. Abnahme
| Gate | Ist-Befund | Soll |
|---|---|---|
| G-02 | kein `unsafe`, `forbid` in 2 Dateien | eingehalten |
| G-07 | Import `contextra_types` (29) | verletzt (REM-30) |
| G-09 | 52 Dateien mit ANCHOR/FILE-CONTEXT, `contract.rs` | verletzt (REM-14) |
| G-11 | `compat/`-Verzeichnis, Features | verletzt (CMP-3, CMP-10) |
| G-12 | 4 Treffer | verletzt |
| G-13 | 46 Treffer | Abbau nach R6 |
