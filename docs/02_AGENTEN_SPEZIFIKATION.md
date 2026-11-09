# Contextra – Technische Spezifikation für Coding-Agenten

**Zweck:** Die reifsten Komponenten (`store`, `vector`, `graph`, `engine`, dazu `text`, `rank`, `mvcc`, `simd`, `crypto`, `ingest`, `kvcache`) produktionsreif, einzeln entnehmbar und nachvollziehbar machen. Lösch-/Nachweis-Altlasten entfernen.
**Zeitrahmen:** 2 Tage, ca. 195 Sitzungen pro Tag, **1 Sitzung = 1 Aufgabe (T-xx) = 1 Commit/PR**.
**Basis:** statische Analyse vom 9. November 2026. Nichts wurde dabei kompiliert. Alle Aussagen unten sind **Hypothesen über den Ist-Zustand**, bis du sie am Code verifiziert hast (Regel 1).

---

## 0. Verbindliche Arbeitsregeln

1. **Verifiziere vor dem Handeln.** Pfade, Namen und Signaturen mit `grep`/`cargo metadata`/`cargo tree` prüfen. Alte Doku (`AGENTS.md`, `PLAN.md`, `capabilities.toml`) ist teilweise veraltet und **nicht** maßgeblich; maßgeblich ist der Code.
2. **Eine Aufgabe pro Sitzung.** Nicht mehrere T-Nummern mischen. Reihenfolge der Abschnitte 4 und 5 einhalten, weil spätere Aufgaben frühere voraussetzen.
3. **Jede Aufgabe endet mit ihrem Prüfbefehl.** Schlägt er fehl und die Ursache liegt außerhalb deiner Aufgabe: nicht „mal eben" fremde Crates reparieren, sondern in `docs/handoff/OPEN_ISSUES.md` eine Zeile eintragen und stoppen.
4. **Keine Umbenennung von Crates oder Paketnamen.** Namen bleiben `contextra-*`. Umbenennung erfolgt erst bei der Entnahme in neue Projekte.
5. **Keine neuen Features** in den ersten 2 Tagen außer den ausdrücklich genannten (Eval-Harness, Tests).
6. **Löschen nur nach Archivierung.** Vor dem ersten Löschen existiert der Git-Tag `pre-cleanup-2026-11-09` (T-00). Archivierte Crates liegen unter `archive/`, **außerhalb** des Workspace.
7. **Kein `unwrap`/`expect`/`panic!`/`todo!`** in neuem Nicht-Test-Code. Fehler über `thiserror`-Typen. Bestehende Treffer nicht massenhaft anfassen, außer die Aufgabe verlangt es.
8. **`unsafe`** nur in `simd`, `sys`, `wire`, jeweils mit `// SAFETY:`-Kommentar. Alle anderen Crates behalten `#![forbid(unsafe_code)]`.
9. **Keine Rechts- oder Garantieversprechen** in Doku, Docstrings oder READMEs („DSGVO-konform", „revisionssicher", „garantiert gelöscht"). Stattdessen sachlich: „entfernt Daten aus Store, Text-, Vektor- und Graph-Index; Verifikation über Test X".
10. **Kein neuer Governance-Überbau:** keine neuen Workflows, Gates, Ratchets, Skripte unter `tools/`.
11. **Sprache:** Doku und Doc-Kommentare Deutsch oder Englisch, aber pro Crate einheitlich. Öffentliche API-Namen Englisch.
12. **Commit-Nachricht:** `T-xx: <Kurzbeschreibung>` plus Zeile „Prüfbefehl: … → grün/rot".

---

## 1. Zielbild

Ein Archiv, aus dem Entwickler **Bausteine entnehmen**. Jedes Bundle hat einen klaren Zweck; jede Crate hat ein README mit dem Entnahme-Rezept.

### 1.1 Bundles (Gruppierung per Metadaten und `CATALOG.md`, **Verzeichnisse bleiben in `crates/`**)

| Bundle | Crates | Rolle |
|---|---|---|
| `foundation` | `types`, `ports`, `testkit` (dev), `sys`, `durable-fs`, `wire` | Typen, Schnittstellen, Systemzugriff |
| `compute` | `simd`, `rank`, `adapt` | Reine Rechenbausteine ohne I/O |
| `storage` | `store`, `mvcc`, `checkpoint`, `crypto` (verschlankt) | Persistenz, Snapshots, Verschlüsselung |
| `retrieval` | `vector`, `text`, `graph` | Index-Engines |
| `ingest` | `ingest` | Dokumentextraktion und Chunking (einzige Chunk-Implementierung) |
| `inference` | `infer-onnx`, `infer-candle`, `infer-ollama` | Embedding/Rerank/LLM-Clients (hinter Features) |
| `orchestration` | `engine`, `contextra` (Fassade, nimmt `db` auf) | Zusammenführung für Endanwender |
| `interfaces` | `mcp`, `py` | Schnittstellen nach außen |
| `edge` | `privacy`, `router`, `kvcache` | Eigenständige Spezialbausteine |
| `experimental` (eingefroren, nicht im Default-Build) | `agent`, `cognition`, `sandbox` | Nicht weiterentwickeln |
| `archive/` (aus Workspace entfernt) | `erasure`, `audit-export`, `avv-generator`, `license` | Verworfene Nachweis-/Rechtsansätze |

Umsetzung in Phase 1 **ohne Verzeichnisumzug**: Feld `bundle = "<name>"` in `[package.metadata.contextra]` jeder `Cargo.toml` plus generierte Tabelle in `docs/CATALOG.md`. Physisches Umsortieren in Unterordner ist ein späteres Projekt.

### 1.2 Zielzustand des Default-Builds
`default-members` = Bundles `foundation`, `compute`, `storage`, `retrieval`, `ingest`, `orchestration`, `interfaces` (ohne `py`), plus `testkit`. Inferenz-Crates nur per Feature. `experimental` nicht im Default.

---

## 2. Bestandsaufnahme (Evidenz, verifizieren)

| Crate | src (Zeilen) | Tests (Zeilen in `tests/`) | Normale interne Abhängigkeiten (laut Cargo.toml) |
|---|---|---|---|
| types | 7,3k | 2,7k | – |
| ports | 6,9k | 0,2k | types |
| mvcc | 4,5k | 2,3k | types, ports |
| simd | 3,5k | 2,0k | types |
| rank | 5,3k | 3,3k | types, ports |
| text | 7,5k | 9,1k | types, ports |
| graph | 17,9k | 14,5k | types, ports |
| vector | 17,8k | 10,2k | types, ports, mvcc, simd, sys |
| store | 22,8k | 31,3k | types, ports, sys, mvcc, crypto |
| crypto | 9,3k | 9,1k | types, ports |
| kvcache | 6,9k | 2,9k | types, ports, crypto |
| engine | 17,6k | 12,6k | **14**: types, ports, mvcc, crypto, sandbox, infer-onnx, adapt, store, vector, text, checkpoint, rank, graph |
| ingest | 21,4k | 8,1k | – |
| db | 2,6k | 12,3k | – (hängt an engine u. a.) |

**Bekannte Probleme (zu bestätigen):**

- B1: `crates/contextra-store/src/lib.rs` deklariert `pub mod kv;`, Verzeichnis/Datei fehlt. `grep` nach `contextra_store::kv` im Workspace: 0 Treffer.
- B2: `crates/contextra-crypto/src/kv_segment/` ist leer.
- B3: `rank::fusion::rrf` enthält `rrf_fuse` und `reciprocal_rank_fusion` (doppelt). `engine/src/fusion.rs` ist nur Re-Export.
- B4: Chunker doppelt: `engine/src/chunker/` (ca. 1,5k Zeilen) und `ingest/src/chunking/`.
- B5: `engine` hängt hart an `infer-onnx` und `sandbox`.
- B6: `contextra-infer-candle` hängt an `crypto` (laut Manifest); `contextra` hängt hart an `infer-candle` und `license`.
- B7: Drei I/O-Abstraktionen: `ports::Vfs`, `store::io_seam::StoreIo`, `durable-fs`.
- B8: Löschdomäne in Fundamentcrates: `types/src/erasure.rs`, `types/src/witness.rs`; in `crypto`: `deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs`; in `engine`: `contextra_impl/erasure.rs`, Re-Exports `crypto::deletion_proof`.
- B9: `store` hängt an `tokio` (ca. 380 `async fn`).
- B10: Veraltete Doku: `AGENTS.md` §4 widerspricht den Manifesten (z. B. `ingest`→`engine`, `graph`→`adapt`).
- B11: Benchmarks zu klein/abgebrochen (siehe Abschnitt 6).

---

## 3. Zielarchitektur – Regeln für Abhängigkeiten

Diese Regeln ersetzen das bisherige M1–M11-Regelwerk **inhaltlich vereinfacht**:

- **R1:** Bausteine der Bundles `compute`, `retrieval`, `ingest` hängen nur von `types` und `ports` (plus die in 1.1 genannten Ausnahmen `vector→mvcc/simd/sys`).
- **R2:** `store` hängt nicht von `vector/text/graph/engine` ab (auch nicht per Dev-Dependency außer `testkit`; Dev-Kanten auf höhere Crates sind zu entfernen oder in ein separates Integrations-Testcrate zu verlegen: `tests/integration` als Workspace-Member).
- **R3:** `engine` darf alle Bundles `storage`, `retrieval`, `compute` kennen; `infer-*`, `sandbox`, `ingest` nur **optional** per Feature.
- **R4:** Nichts im Default-Build hängt von `archive/`, `experimental`.
- **R5:** Prüfung: `cargo tree -p <crate> -e normal --depth 1` entspricht der Tabelle 2 (Ziel) – in `docs/CATALOG.md` hinterlegt.
- **R6:** `README.md` jeder Crate hat die Abschnitte: **Zweck · Reifegrad · Interne Abhängigkeiten · Features · Minimalbeispiel (kompiliert als Doctest) · Entnahme-Rezept · Bekannte Grenzen**.

---

## 4. Aufgaben Tag 1: Build, Archiv, Struktur

Reihenfolge strikt einhalten. Jede Aufgabe: *Ziel · Schritte · Prüfbefehl*.

**T-00 Sicherung.** `git tag pre-cleanup-2026-11-09 && git push --tags`. Prüfung: `git tag | grep pre-cleanup`.

**T-01 Store-Build-Blocker.** In `crates/contextra-store/src/lib.rs` die Zeile `pub mod kv;` entfernen (B1). Danach alle weiteren Fehler **nur im Store** beheben, die direkt daraus folgen (Verweise auf `KvDeleteMode`, `kv::…`). Nicht-Store-Fehler → `OPEN_ISSUES.md`. Prüfung: `cargo check -p contextra-store --locked`.

**T-02 Fundament baut.** Der Reihe nach: `types`, `ports`, `mvcc`, `simd`, `sys`, `wire`, `durable-fs`, `rank`, `adapt` – je `cargo check -p <crate> --locked`. Fehler beheben, die aus der Restrukturierung stammen. Prüfung: `cargo check -p contextra-types -p contextra-ports -p contextra-mvcc -p contextra-simd -p contextra-sys -p contextra-wire -p contextra-durable-fs -p contextra-rank -p contextra-adapt`.

**T-03 Index-Crates bauen.** `text`, `graph`, `vector`, dann `checkpoint`, `crypto`. Prüfung je `cargo check -p <crate>`; danach `cargo test -p <crate> --no-run`.

**T-04 Engine baut.** `cargo check -p contextra-engine`. Nicht-trivial: Fehler aus der Umstrukturierung beheben, nichts Neues bauen.

**T-05 Rest baut.** `ingest`, `kvcache`, `router`, `privacy`, `infer-*`, `mcp`, `py`, `contextra`, `db`. Was zu langwierig ist: Crate in `experimental`-Liste setzen und in `OPEN_ISSUES.md` begründen.

**T-06 Archiv: `erasure`.** Zuerst prüfen: `grep -rn "contextra_erasure\|contextra-erasure" --include=*.rs --include=*.toml .` – erwartet nur Treffer in `crates/contextra-erasure` selbst und im Workspace-Manifest. Dann `git mv crates/contextra-erasure archive/contextra-erasure`, aus `[workspace].members` und `default-members` entfernen, aus `capabilities.toml` entfernen. Prüfung: `cargo metadata --no-deps >/dev/null` und `cargo check -p contextra-types`.

**T-07 Archiv: `audit-export`, `avv-generator`, `license`.** Wie T-06. `contextra` (Fassade) hängt an `license` → Abhängigkeit und Nutzung entfernen (nur Feature-Gating-Code, keine Neuimplementierung). Prüfung: `cargo check -p contextra`.

**T-08 `crypto` aufteilen.** Verschiebe in `archive/contextra-crypto-proofs/`: `deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs`. Prüfe vorher mit `grep` pro Modul, wer es nutzt (Store: `wal_crypto`, `anti_tamper`, `wal_completeness`, `audit_chain` könnten nötig sein). Behalte jedes Modul, das von `store`, `vector`, `kvcache` tatsächlich genutzt wird; entferne alles andere. `ed25519-dalek` aus den Abhängigkeiten entfernen, falls ungenutzt. Prüfung: `cargo check -p contextra-crypto -p contextra-store -p contextra-vector -p contextra-kvcache` und `cargo tree -p contextra-crypto -e normal | grep -c ed25519` → 0, wenn alle Proof-Module raus.

**T-09 Löschdomäne aus Fundament.** `types/src/erasure.rs` und `types/src/witness.rs` samt Re-Exports entfernen oder ins Archiv schieben; `ports`: doppelte `LayerWitness`-Familie auf **eine** reduzieren. **Behalten:** die schlanken Ports `Purger` und `LayerScanner` (mechanisches Entfernen/Messen). Prüfung: `grep -rniE "deletion_proof|kv_shredding|revocation_log|saga|witness" crates/*/src | wc -l` – Zielwert deutlich kleiner als vorher (vorher in Aufgabe notieren), plus `cargo check --workspace --exclude contextra-py`.

**T-10 Engine entschlacken (Löschung).** `engine/src/contextra_impl/erasure.rs` und Re-Exports von `crypto::deletion_proof` entfernen. Die mechanische Löschpfad-API (Dokument löschen über Store, Text, Vektor, Graph; Transaktions-Kompensation) bleibt und erhält einen Test (T-31). Prüfung: `cargo check -p contextra-engine`.

**T-11 `mcp` entschlacken.** Lösch-/Beweis-Werkzeuge in `server_tools.rs` und `proof_key.rs` entfernen. `forget`-Werkzeug nur als einfaches „Dokument löschen" ohne Nachweisobjekt erhalten, falls vorhanden. Prüfung: `cargo check -p contextra-mcp` und `grep -rniE "proof|witness|erasure_report" crates/contextra-mcp/src` → 0 Treffer.

**T-12 `db` einfalten.** Prüfe öffentliche API von `contextra-db` und Nutzer (`grep -rn "contextra_db" crates`). Verschiebe nutzbare Funktionalität in `contextra` (Fassade) bzw. `engine`; hänge `py`, `agent`, `router`-Tests um; markiere `db` als `experimental`, wenn Umhängen zu groß ist, und trage in `OPEN_ISSUES.md` ein. **Die bestehenden 12k Zeilen Tests sind nicht wegzuwerfen:** verschiebe sie mit in `contextra`/`engine`-Tests. Prüfung: `cargo test -p contextra --no-run`.

**T-13 Chunker vereinheitlichen.** Vergleiche `engine/src/chunker/` und `ingest/src/chunking/` (API, Tests). Entscheide die vollständigere Implementierung als **die** Implementierung in `ingest`; `engine` verwendet sie optional via Feature `ingest` (`default = []`). Doppelte Tests nicht löschen, sondern in die verbleibende Implementierung portieren. Prüfung: `cargo test -p contextra-ingest`, `cargo check -p contextra-engine --no-default-features`.

**T-14 Engine-Abhängigkeiten als Features.** `infer-onnx`, `sandbox`, (und `ingest` aus T-13) in `contextra-engine` hinter Features legen (`onnx`, `sandbox`, `ingest`). `default` minimal. Prüfung: `cargo check -p contextra-engine --no-default-features` und `cargo tree -p contextra-engine -e normal --no-default-features --depth 1` – darf weder `infer-onnx` noch `sandbox` enthalten.

**T-15 `infer-candle`/Fassade.** `infer-candle`: Abhängigkeit zu `crypto` entfernen, falls ungenutzt. `contextra`: `infer-candle` hinter Feature `candle`. Prüfung: `cargo check -p contextra --no-default-features`.

**T-16 Dev-Kanten säubern.** In `store`, `graph`, `checkpoint`, `router`: Dev-Dependencies auf höhere Crates (z. B. `store`→`graph/vector/sandbox`) entfernen oder in Integrations-Testcrate `tests/integration` verlegen. Prüfung je Crate: `cargo test -p <crate> --no-run`.

**T-17 Default-Members aktualisieren.** `default-members` gemäß 1.2; `experimental` (`agent`, `cognition`, `sandbox`, ggf. `db`) entfernen. Prüfung: `cargo check` (Default) und `cargo metadata`.

**T-18 Doku-Wahrheit.** `AGENTS.md` kürzen auf: Regeln aus Abschnitt 0 und 3 dieser Spezifikation, Verweis auf `docs/CATALOG.md`. Entferne Behauptungen, die der Code widerlegt (B10). `PLAN.md`, `DEPRECATED_DEBT.md`, `docs/archive/*`, `docs/reports/*` nach `docs/_history/` verschieben (nicht löschen). Prüfung: `grep -n "ring\|Ring" AGENTS.md | wc -l` stark reduziert; keine toten Links (`grep -rn "](" AGENTS.md README.md`).

**T-19 CATALOG.md erzeugen.** Tabelle aller Crates: Bundle, Zweck (1 Satz), Reifegrad (`stable`/`beta`/`experimental`/`frozen`), interne Abhängigkeiten (aus `cargo metadata`, nicht von Hand), Entnahmeaufwand (`trivial`/`mittel`/`hoch`), Link zum README.

**T-20 Crate-READMEs.** Pro Crate gemäß R6. **Beginne mit:** `store`, `vector`, `graph`, `text`, `rank`, `mvcc`, `simd`, `ingest`, `engine`, `kvcache`. Minimalbeispiele müssen als Doctests kompilieren (`cargo test --doc -p <crate>`). Keine Aussage ohne Codebeleg.

**T-21 Workflows reduzieren.** Zwei Workflows behalten: `ci.yml` (Build + Test aller Default-Members, `cargo clippy -D warnings`, `cargo fmt --check`, `cargo deny check`) und optional `bench.yml` (manuell). Die übrigen 33 nach `.github/_disabled/` verschieben. Prüfung: Workflow-Syntax mit `yamllint`/`actionlint`, falls verfügbar.

---

## 5. Aufgaben Tag 2: Produktionsreife der Kernbausteine

Gemeinsame Definition **„produktionsreif"** für jede Crate im Kern:

- **P1:** `cargo build --no-default-features` und `cargo build --all-features` sind grün.
- **P2:** `cargo clippy -D warnings` sauber im Nicht-Test-Code; neue `unwrap/expect` 0.
- **P3:** Öffentliche API vollständig dokumentiert (`#![warn(missing_docs)]`), Fehlertypen mit `thiserror`.
- **P4:** Jede öffentliche Hauptfunktion hat mindestens einen Test, der **unabhängig vom Implementierungsweg** prüft (siehe 5.7).
- **P5:** Persistenzformate haben eine **Versionsnummer** und einen Kompatibilitätstest (alte Fixture-Datei lesen).
- **P6:** README nach R6 inkl. dokumentierter „Bekannte Grenzen".
- **P7:** Fuzz- oder Property-Test an jeder Grenze, an der fremde Bytes gelesen werden (WAL, SSTable, HNSW-Header, Ingest-Parser).

### 5.1 `store`

**T-30 Tokio entkoppeln.** Ermitteln, wo `tokio` wirklich nötig ist (Hintergrund-Flush/Compaction). Ziel: Kern-API (`put/get/delete/scan/snapshot`) **synchron nutzbar**; `tokio`-Teile hinter Feature `async` (Default: an, damit nichts bricht; aber Minimalbau ohne Tokio möglich, wenn machbar). Wenn nicht in einer Sitzung machbar: **dokumentieren** („benötigt Tokio-Runtime") und auf „Bekannte Grenzen" setzen. Nicht halb umbauen.
**T-31 Crash-Konsistenz-Test.** WAL-Truncation an jeder Byte-Grenze eines Beispiel-Logs: Wiederöffnen muss entweder konsistent bis zum letzten kompletten Eintrag laden oder sauber fehlschlagen, nie panicken. Property-Test (proptest) mit zufälligen Operationsfolgen plus Differenz gegen `redb` (existiert; ausführen, Fehler dokumentieren, nicht abschalten).
**T-32 Shredding-Reste.** Treffer `grep -rniE "shredd|deletion_proof|kv_delete" crates/contextra-store/src` abbauen (laut Analyse bei `compaction/engine.rs`, `compaction/adaptive.rs`, ggf. weitere). Neutral benennen: „tombstone purge".
**T-33 I/O-Abstraktion klären.** Entscheidung dokumentieren: `StoreIo` bleibt die eine Abstraktion des Stores; `durable-fs` implementiert sie oder wird in `OPEN_ISSUES` als „später" vermerkt. Nicht beide neu verdrahten.
**T-34 Format-Versionen.** SSTable V4, WAL-Format, Manifest: Versionsfelder dokumentieren; Fixture-Dateien unter `tests/fixtures/` ablegen, Test „liest V(n-1)".
Prüfung (Crate): `cargo test -p contextra-store` und `cargo test -p contextra-store --features test-hooks`.

### 5.2 `vector`

**T-40 Recall-Referenztest.** Brute-Force-Referenz auf 10k zufälligen Vektoren (128d, fester Seed), `k=10`; HNSW-Recall@10 ≥ 0,95 bei dokumentierter Konfiguration (`M`, `ef`). Ergebnis in `README.md`. Bei Verfehlung: nicht Ziel senken, sondern Konfiguration/Bug dokumentieren.
**T-41 Lösch-Konsistenz.** Test: n Vektoren einfügen, Teilmenge löschen (`purge`/`deletion`), Suche darf gelöschte IDs nie liefern; Recall der verbliebenen Menge bleibt ≥ Schwelle; Rebuild (`partial_rebuild`) stellt Konnektivität sicher. Das ist der **einzige** Löschnachweis, den wir führen (mechanisch, kein Rechtsbeweis).
**T-42 Experimentelles markieren.** `diskann`, `rabitq` bleiben feature-gesperrt; im README unter „experimentell". Keine neuen Arbeiten daran.
**T-43 Persistenz.** `HnswHeader` Versions-/Checksummentest; Fuzz auf `MmapIndex`-Header-Parser (`cargo fuzz` Ziel anlegen, nicht nur Doku).
Prüfung: `cargo test -p contextra-vector`.

### 5.3 `graph`

**T-50 Referenzwerte.** PPR und Community Detection gegen **handgerechnete** Kleingraphen (≤ 10 Knoten) mit festen Erwartungswerten; PPR gegen Power-Iteration-Referenz auf Zufallsgraphen (Toleranz dokumentiert).
**T-51 Entity-Extraktion bewerten.** Kleiner deutsch-/englischsprachiger Goldstandard (30–50 Sätze) mit erwarteten Entitäten; Precision/Recall der Regel-Extraktion als Testbericht. **Keine** Verbesserungen, nur Messung – das Ergebnis geht in die Strategie (Graph-Signal ja/nein).
**T-52 Dev-Kante `db`.** Dev-Dependency auf `contextra-db` entfernen (siehe T-16).
Prüfung: `cargo test -p contextra-graph`.

### 5.4 `text`

**T-60 BM25-Referenz.** Gegen `tantivy` (als dev-dependency) auf einem kleinen Korpus: Top-10-Überlappung ≥ dokumentierter Schwelle; Abweichungen (Tokenisierung, Stopwörter) im README erklären. Block-Max-WAND-Ergebnis muss **identisch** zur exhaustiven Auswertung sein (Property-Test).
**T-61 Purge-Test.** Gelöschtes Dokument darf in keiner Query erscheinen (analog T-41).
Prüfung: `cargo test -p contextra-text`.

### 5.5 `rank` (höchster Hebel, kleinster Aufwand)

**T-70 Fusion vereinheitlichen.** Eine Public-API für RRF: Funktion mit `k` (Default 60), optional Gewichten. `rrf_fuse` und `reciprocal_rank_fusion` zusammenführen (eine als `#[deprecated]` Alias, die andere kanonisch). `engine/src/fusion.rs` bleibt dünner Re-Export oder entfällt.
**T-71 Golden-Tests.** Handgerechnete Fälle mit fester Erwartung: 2 Listen, 3 Listen, Gleichstand, fehlende Dokumente in einer Liste, Gewichte 0/1, leere Eingabe, doppelte IDs. Jeder Fall dokumentiert die Rechnung im Test.
**T-72 Reduktion.** Default-Strategien: RRF, gewichtetes RRF, normalisierte Linearkombination. `mrrf`, `resonance`, `global`, `dibud`, Drift, EdgeReinforcement hinter Feature `experimental`; README führt sie unter „experimentell, nicht evaluiert".
**T-73 Determinismus.** Gleicher Input → gleiche Reihenfolge, auch bei Gleichstand (stabile Tie-Breaker nach ID). Property-Test.
Prüfung: `cargo test -p contextra-rank`; `grep -rn "unwrap()" crates/contextra-rank/src | wc -l` bleibt 0.

### 5.6 `engine`, `mvcc`, `simd`, `crypto`, `ingest`, `kvcache`

**engine**
- **T-80:** End-to-End-Test **ohne** Inferenz: Collection anlegen, N Dokumente mit festen Vektoren und Text einfügen, Hybridsuche (Vektor+BM25+Graph), Fusion prüfen, Dokument löschen, erneut suchen → Dokument verschwunden in allen drei Signalen. Läuft mit `--no-default-features`.
- **T-81:** Transaktions-Rollback-Test (`compensating_actions`): Fehler im zweiten Index-Schritt → keine Teil-Schreibung sichtbar.
- **T-82:** Hintergrund-Worker (`expiry`, `orphan`, `hyperedge`) einzeln abschaltbar; Standard minimal.

**mvcc:** **T-85:** Isolationstests (Snapshot-Lesen sieht keine späteren Commits, Write-Konflikte), loom-Test für zentrale Struktur, falls noch nicht vorhanden.

**simd:** **T-86:** Jeder Kernel hat einen Skalar-Referenztest mit zufälligen Eingaben, inkl. nicht ausgerichteten Längen (0, 1, 7, 8, 15, 16, 31, 33). Miri nur dort, wo er läuft; Ausnahmen im README nennen. `// SAFETY:` je `unsafe`-Block prüfen (`grep -c "unsafe" ` vs. `grep -c "SAFETY"`).

**crypto (verschlankt):** **T-87:** Known-Answer-Tests (offizielle Testvektoren) für AEAD und KDF. README: *„Nicht extern auditiert. Nicht für Produktivumgebungen mit Sicherheitsversprechen ohne Audit empfohlen."*

**ingest:** **T-88:** Pro Format ein Golden-File-Test (kleine Beispieldatei in `tests/fixtures/`), Fuzz-Ziel je Parser (`cargo fuzz`), Resource-Limits (`limits.rs`) durch Test belegen (z. B. Zip-Bombe, tief verschachteltes XML, riesige Tabelle). **Keine neuen Formate.**

**kvcache:** **T-89:** Sämtliche Cache-Funktionalität in dieser Crate bündeln: grep nach `kv_cipher`, `kv_segment`, `kv_shredding` in `crypto`, `store`, `privacy`, `mcp`. Verwendetes verschieben (Cipher-Logik, falls nur von `kvcache` genutzt), Rest ins Archiv. README schreibt ehrlich: *„Prefix-Cache-Verwaltung für KV-Segmente; keine Anbindung an eine konkrete Inferenz-Runtime im Repo."* Keine neuen Features.

### 5.7 Teststrategie – einheitliche Ebenen

| Ebene | Werkzeug | Pflicht für |
|---|---|---|
| Unit | `cargo test` | alle |
| Golden/Referenz | handgerechnete Erwartung, `tantivy`, Brute-Force | `rank`, `text`, `vector`, `graph`, `simd` |
| Differential | `redb` | `store` |
| Property | `proptest` | `store`, `rank`, `text`, `mvcc` |
| Nebenläufigkeit | `loom` | `mvcc`, `store` (vorhanden) |
| Fuzz | `cargo fuzz` | Parser (`ingest`, WAL, SSTable, HNSW-Header) |
| Integration | `tests/integration` (separates Crate) | Zusammenspiel `engine` |

**Regel:** Ein Test zählt für die Abdeckung nur, wenn seine Erwartung **nicht** aus der getesteten Funktion selbst abgeleitet wird. Reine Wiederholung der Implementierung im Test ist nicht zulässig.

**Abdeckung:** `cargo llvm-cov` (oder `tarpaulin`) einmalig pro Kernel-Crate ausführen; Ergebnis in `docs/COVERAGE.md` festhalten. Zielwert für `rank`, `mvcc`: Linienabdeckung ≥ 90 %; `store`, `vector`, `text`, `graph`: ≥ 80 %; `engine` ≥ 70 %. **Nicht erfüllte Ziele werden dokumentiert, nicht durch Test-Attrappen kaschiert.**

---

## 6. Evaluations-Harness (wichtigster Beitrag für Produkt und Antrag)

**T-90 Harness reaktivieren.** `benchmarks/contextra-bench` (laut Analyse ca. 4,2k Zeilen) baut wieder; veraltete Benchmarks (50 Chunks, Reranker mit +0 %) als `historical` markieren.
**T-91 Datensatz.** Eine **öffentliche** IR-Testmenge (z. B. ein BEIR-Teilset; Lizenz prüfen, Download-Skript, keine Daten committen) plus ein **eigener deutscher Mini-Korpus** (mind. 200 Dokumente, 50 Queries mit Relevanzurteilen, selbst erstellt, Lizenz-unkritisch).
**T-92 Metriken.** Recall@10, nDCG@10, MRR@10, Latenz p50/p95/p99, RAM-Spitze, Indexaufbauzeit.
**T-93 Vergleichsmatrix.** Vektor allein · BM25 allein · RRF(V+T) · RRF(V+T+G) · gewichtete Variante · normalisierte Linearkombination. Ausgabe als `benchmarks/results/<datum>.md` und `.json`.
**T-94 Skalierungslauf.** 100k Dokumente; falls der Lauf wieder wegen „Memory budget exceeded" scheitert: Ursache im Bericht beschreiben (Konfiguration des Budgets vs. echter Bedarf), **nicht** das Budget stillschweigend erhöhen. Zahlen sind das Ergebnis, auch wenn sie schlecht sind.
**T-95 Bericht.** `docs/EVALUATION.md`: ehrliche Tabelle, Umgebung (CPU, RAM, Rust-Version), Reproduktionsbefehl. Kein Marketington.
Prüfung: `cargo run -p contextra-bench --release -- --help` und ein Lauf auf dem Mini-Korpus.

---

## 7. Extraktions-Rezept (für spätere Entnahme)

**T-96 `docs/EXTRACTION.md`.** Für jede Crate: (1) Abhängigkeitsliste aus `cargo tree`, (2) zu kopierende Crates in Reihenfolge, (3) Workspace-Dependencies, die in die neue `Cargo.toml` übernommen werden müssen (`version.workspace = true` ersetzen), (4) Minimalprogramm, (5) bekannte Stolpersteine. Das vorhandene `tools/export_crate.py` prüfen und, falls funktionstüchtig, im Dokument referenzieren; wenn nicht, im Dokument als „nicht verlässlich" vermerken und **nicht** reparieren.

---

## 8. Entfernen und Archivieren – verbindliche Liste

| Objekt | Aktion |
|---|---|
| `crates/contextra-erasure` | → `archive/` (T-06) |
| `crates/contextra-audit-export`, `contextra-avv-generator`, `contextra-license` | → `archive/` (T-07) |
| `crypto/{deletion_proof,ed25519_proof,kv_shredding,revocation_log}` (+ nach Prüfung `audit_chain`, `anti_tamper`, `wal_completeness`) | → `archive/contextra-crypto-proofs` (T-08) |
| `types/src/{erasure,witness}.rs`, doppelte `LayerWitness` in `ports` | entfernen/archivieren (T-09) |
| `engine/src/contextra_impl/erasure.rs` | entfernen (T-10) |
| Lösch-/Beweis-Tools und `proof_key.rs` in `mcp` | entfernen (T-11) |
| `crates/contextra-db` | in `contextra` einfalten (T-12) |
| 33 Workflows | nach `.github/_disabled/` (T-21) |
| `docs/archive/*`, `docs/reports/*`, `PLAN.md`, `DEPRECATED_DEBT.md` | nach `docs/_history/` (T-18) |
| `tools/*.py` Governance-Skripte | **nicht anfassen** in 2 Tagen; Ordner in `tools/README.md` als „Legacy, nicht gepflegt" kennzeichnen |

**Nicht entfernen:** `Purger`/`LayerScanner`-Ports und die mechanischen Purge-Implementierungen in `vector`, `text`, `graph`, `store`.

---

## 9. Priorisierung bei knapper Zeit

Wenn nicht alles in 2 Tagen fertig wird, gilt diese Rangfolge:

1. **T-00 bis T-05** (Build grün) – sonst ist alles andere nutzlos.
2. **T-06 bis T-11** (Archiv und Löschdomäne) – reduziert Ballast und Auditfläche.
3. **T-70 bis T-73** (`rank`) und **T-90 bis T-95** (Eval) – belegen den Wert.
4. **T-18 bis T-20** (Doku-Wahrheit, CATALOG, READMEs) – ermöglicht die Übergabe.
5. **T-13 bis T-17** (Engine-Features, Chunker, Dev-Kanten).
6. Tests der Kern-Crates (Abschnitt 5.1–5.6).
7. **T-21** (CI) und **T-96** (Extraktion).

---

## 10. Abnahme am Ende von Tag 2

Eine Checkliste für die Übergabe an Nachfolger (jede Zeile mit Befehl und Ergebnis in `docs/handoff/STATUS.md` festhalten):

- [ ] `cargo check --workspace` (Default-Members) grün – oder jedes Fehlschlagen in `OPEN_ISSUES.md` mit Ursache.
- [ ] `cargo test -p contextra-rank -p contextra-mvcc -p contextra-simd -p contextra-text` grün.
- [ ] `cargo test -p contextra-store -p contextra-vector -p contextra-graph -p contextra-engine` – Ergebnis dokumentiert (grün oder Liste fehlschlagender Tests).
- [ ] `cargo tree -p <crate> -e normal --depth 1` entspricht `docs/CATALOG.md` für alle Kern-Crates.
- [ ] `grep -rniE "deletion_proof|kv_shredding|revocation_log|avv|audit_report" crates/*/src` → 0 Treffer außerhalb Archiv.
- [ ] `archive/` enthält die 4 verworfenen Crates plus `contextra-crypto-proofs`; Workspace referenziert sie nicht.
- [ ] `docs/CATALOG.md`, `docs/EVALUATION.md`, `docs/EXTRACTION.md`, `docs/COVERAGE.md`, `docs/handoff/OPEN_ISSUES.md` existieren und sind aktuell.
- [ ] Jedes Kern-Crate-README erfüllt R6; Doctests laufen.
- [ ] Git-Tag `handoff-2026-11-11` gesetzt.

**Ehrlichkeitsregel für den Statusbericht:** Jede nicht erfüllte Zeile bleibt als nicht erfüllt stehen. Ein ehrlicher roter Status ist besser als ein erfundener grüner.
