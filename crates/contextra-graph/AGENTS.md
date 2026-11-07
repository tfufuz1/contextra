# AGENTS.md — `contextra-graph`

> **Ring 1 (`capabilities.toml`: Ring 0 ≠)** · Klasse `[K]` im SPEC-Katalog (SPEC §0.5 archiviert; Root-§10 K5) · Reifegrad `stable` · Unsafe: nein · Rolle: CSR-Graphindex für semantische Netze, Pfad-RAG, PPR, Communities. Implementiert `GraphIndex`.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-graph --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | `ports`, `types` |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | `ports`, `types` |
| Features | `apprh-diffusion`, `k-path-diffusion`, `physio-percolation`, `ppr-forward-push`, `edge-reinforcement-learning`, `graph-connectivity-health`, `docid-128` |
| Lints | `#![forbid(unsafe_code)]`, `deny(dead_code)`, `deny(clippy::await_holding_lock)` |

## 2. Abhängigkeiten & Schulden

- `capabilities.toml` nennt noch `adapt` (Cargo nicht mehr): bereinigen. Dev-Kante auf `store` (M6, WP 4.3).
- ~21.600 Zeilen inkl. Tests; `entity_extraction.rs` ist Extraktionslogik (Zuordnung prüfen, evtl. `engine`/`ingest`).
- `purger.rs` implementiert `Purger`.

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `csr/` (`graph_index`, `graph_read`, `graph_write`, `inner`, `graph_persist`, `visibility`) | CSR-Struktur, Lesen/Schreiben, Persistenz, Sichtbarkeit |
| `ppr/`, `path_rag/`, `apprh/`, `tl_hfd/`, `percolation.rs` | Diffusions- und Pfadverfahren (Feature-gesteuert) |
| `community.rs`, `hyperedge*.rs`, `cascade.rs` | Communities, Hyperkanten, Kaskade |
| `purger.rs`, `provider.rs`, `provenance.rs` | Port-Implementierung, Provider, Herkunft |

## 4. Verträge & Invarianten

1. **Lock-freier Lesepfad** Ziel: CSR-Segment unveränderlich, Schreiben in Delta-Puffer, Merge atomar (WP 4.3).
2. **Tenant-Isolation (VETO-F10):** keine Kante über Tenants.
3. **Sichtbarkeit bitemporal** (`csr/visibility.rs`); Snapshots respektieren `born_seq`/`died_seq`.
4. Persistenz robust gegen Teilschreiben (`tests/graph_persist_robustness.rs`).
5. Keine Löschlogik; Bereinigung nur via `Purger`.

## 5. Ports

**Implementiert:** `GraphIndex`, `Purger`. **Konsumiert:** `Clock`.

## 6. Nebenläufigkeit & Performance

- Kein Guard über `.await` (Lint `deny`). Komplexität je Verfahren in `benches/` (`csr_traversal_bench`) und `tests/csr_complexity_bench.rs` belegen; Hub-Knoten separat (`hub_node_benchmark.rs`).

## 7. Anti-Patterns

- Graph-Kanten über Tenant-Grenzen.
- `graph → store`-Normalkante.
- Diffusionsverfahren ohne Schattenmodus produktiv schalten.

## 8. Tests & Messung

```bash
cargo test -p contextra-graph --locked
cargo clippy -p contextra-graph --all-targets -- -D warnings
cargo bench -p contextra-graph
```

- Vorhanden: Golden-Tests, Determinismus-Permutationstests, Crash-Recovery der Kaskade, Fuzz.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 4.3 | Delta-Puffer, lock-freier Lesepfad, Dev-Kante `store` entfernen |
| 1.1 | Ring auf 1; `adapt` aus `may_depend_on` |


---


# Kommentar-Syntax (verbindlich)

| Form | Zweck |
|---|---|
| `//!` | Modul-/Crate-Doku am Dateianfang (vor jedem Item, nach `#![...]`-Attributen der Datei; nie mitten in der Datei) |
| `///` | Rustdoc für das **direkt folgende** Item (Struct, Enum, Variante, Feld, Trait, Methode, Funktion, Konstante). Nie vor `let`, Ausdrücken, Match-Armen oder Makroaufrufen; dort `//` verwenden. |
| `// NOTE:` | Warum etwas so gebaut ist (nicht was) |
| `// TODO:` / `// FIXME:` / `// STUB:` | Offene Arbeit / bekanntes Risiko / Platzhalter; immer mit konkreter Aussage |
| `// SAFETY:` | Pflicht direkt über jedem unsafe-Konstrukt (Invariante, Herstellungsort, Folge bei Verletzung) |
| `// ANCHOR[contextra-graph:<BEREICH>-<NNN>]` | Sprungmarke; vor dem Item/Abschnitt, auf den sie zeigt |
| `// FILE-CONTEXT` + `// ZWECK:` / `// INVARIANTEN:` / `// HOTSPOTS: <von>-<bis>` / `// STAND: <Datum>` | Repo-Konvention für den Dateikopf. Vorhandene Blöcke ergänzen; `HOTSPOTS`-Zeilenbereiche nach dem Kommentieren auf die neuen Zeilennummern aktualisieren. `STAND` auf das heutige Datum setzen, wenn du den Block änderst. |

Weitere Tags führst du nicht ein. Beim Kommentieren markierter Altlasten gilt: `// NOTE: Deprecated Debt DEBT-NN – <Beschreibung>`.

## ANCHOR-Ids (Kollisionsschutz)

Im Repo existieren ca. 3.900 Anchors; Ids sind workspace-weit eindeutig zu halten. Format: `ANCHOR[contextra-graph:<BEREICH>-<NNN>]` mit `<BEREICH>` in Großbuchstaben/Ziffern/Bindestrichen, `<NNN>` dreistellig. Vorgehen:

1. Vor jeder neuen Id: `grep -rnF "ANCHOR[contextra-graph:<BEREICH>-<NNN>]" crates benchmarks` muss leer sein.
2. Nummeriere ab dem Maximum des Bereichs weiter. Höchste bestehende Nummern in dieser Crate:

| Bereich | Höchste Nr. | Nächste freie Id |
|---|---:|---|
| `SRC-SESSION-DAG` | 49 | `ANCHOR[contextra-graph:SRC-SESSION-DAG-050]` |
| `SRC-HYPEREDGE` | 45 | `ANCHOR[contextra-graph:SRC-HYPEREDGE-046]` |
| `SRC-CONSISTENCY-ENFORCEMENT` | 29 | `ANCHOR[contextra-graph:SRC-CONSISTENCY-ENFORCEMENT-030]` |
| `SRC-CASCADE` | 28 | `ANCHOR[contextra-graph:SRC-CASCADE-029]` |
| `SRC-PATH-RAG-TESTS` | 22 | `ANCHOR[contextra-graph:SRC-PATH-RAG-TESTS-023]` |
| `SRC-CSR-TESTS-BASIC-TESTS` | 20 | `ANCHOR[contextra-graph:SRC-CSR-TESTS-BASIC-TESTS-021]` |
| `SRC-PPR-TESTS-ALGO-TESTS` | 20 | `ANCHOR[contextra-graph:SRC-PPR-TESTS-ALGO-TESTS-021]` |
| `SRC-CSR-TESTS-GRAPH-INDEX-TESTS` | 17 | `ANCHOR[contextra-graph:SRC-CSR-TESTS-GRAPH-INDEX-TESTS-018]` |
| `SRC-CSR-TESTS-PERSISTENCE-TESTS` | 17 | `ANCHOR[contextra-graph:SRC-CSR-TESTS-PERSISTENCE-TESTS-018]` |
| `SRC-PATH-RAG-MOD` | 16 | `ANCHOR[contextra-graph:SRC-PATH-RAG-MOD-017]` |
| `SRC-PPR-COST` | 16 | `ANCHOR[contextra-graph:SRC-PPR-COST-017]` |
| `SRC-COMMUNITY-TESTS` | 15 | `ANCHOR[contextra-graph:SRC-COMMUNITY-TESTS-016]` |

3. Neue Bereiche nach dem Modul benennen (z. B. `WAL`, `PUT`, `GATE`); fange bei `001` an, nach Prüfung 1. Setze Anchors sparsam: pro Modul, pro öffentlichem Einstiegspunkt und pro nicht-trivialem Ablauf, nicht pro Zeile.
4. Bereits vorhandene doppelte Ids anderer Stellen nicht „reparieren“; nur im Report melden.

Für Abläufe in Funktionen nutze nummerierte Schritte:

```rust
// ANCHOR[contextra-graph:PUT-001]
// Ablauf von `put`:
// 1. Eingabe prüfen (Länge, Schlüsselformat).
// 2. Snapshot der Transaktion nehmen.
// 3. Eintrag ins WAL schreiben und mit fsync bestätigen; ein Fehler wird zurückgegeben (Fail-Closed).
// 4. Erst danach im Speicher übernehmen.
```

# Was zu dokumentieren ist

Gehe **jede** Rust-Datei unter `crates/contextra-graph` durch (`find crates/contextra-graph -name "*.rs"`, ohne die generierten Dateien oben) einschließlich `tests/`, `benches/`, `examples/` und `build.rs`:

- **Modul** (`//!`): Zweck, Ring-Zugehörigkeit, Nutzer, wichtigste Typen, Invarianten, Verweis auf Nachbarmodule.
- **Öffentliche Items:** Zweck in einem Satz, dann bei Bedarf `# Errors` (welcher Fehler wann), `# Panics` nur wenn der Code abbrechen kann (sollte nach Zero-Panic-Regel nicht vorkommen: dann `FIXME`), `# Safety` bei `unsafe fn`, Vorbedingungen, Nachbedingungen, Nebenwirkungen (I/O, Zeit, Zufall, IDs). Zeit, Zufall und IDs laufen nur über `Clock`/`Rng`/`IdGen`; steht es im Code, steht es im Kommentar.
- **Nicht-triviale private Funktionen:** Warum sie existieren und was sie garantieren.
- **Nebenläufigkeit:** Sperren, atomare Operationen, Reihenfolge, Warum kein Deadlock.
- **Persistenz und Fehler:** Wo Daten WAL oder Dateisystem-Port passieren; `fsync`-Fehler werden propagiert.
- **Tests:** Ein Satz je Testfunktion oder -block: welche Invariante oder welches Verhalten bewiesen wird.
- **Architektur:** Welche Ports die Crate nutzt/anbietet, welche Abhängigkeitsrichtung gilt (DAG, `may_depend_on`), was bewusst *nicht* hier liegt.

## Rustdoc-Fallen (jeder Punkt bricht sonst `cargo clippy -D warnings` oder `cargo doc`)

- Kein `///` vor Nicht-Items (`unused_doc_comments`); keine Leerzeile zwischen `///` und Item (`empty_line_after_doc_comments`).
- Fortsetzungszeilen in Listen und Zitaten korrekt einrücken (`doc_lazy_continuation`). **Vier Leerzeichen Einrückung in Doc-Text erzeugen einen Doctest**; Code immer in Fences mit Sprache. Beispiele, die nicht kompilieren oder laufen sollen: `ignore` oder `text`, begründet.
- Intra-Doc-Links `[`Typ`]` nur auf existierende, erreichbare Items; spitze Klammern im Text (`Vec<u8>`) in Backticks setzen, sonst „unclosed HTML tag“.
- `// SAFETY:` ohne Leerzeile und ohne dazwischenliegende Zeilen direkt über dem unsafe-Ausdruck.
- Doc-Beispiele müssen gegen die tatsächliche Signatur kompilieren; im Zweifel `no_run`/`ignore` mit Grund.
