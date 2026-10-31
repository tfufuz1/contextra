# Contextra – Umsetzungsplan 4.0 (Reduktion: IST → SOLL)

<!-- doc-stamp: 2026-10-30 | Ersetzt: PLAN 3.1 (Baukasten-Fassung). Normativ: docs/SPEC.md Fassung 4.0 -->

**Stand:** 30.10.2026 · **Bezugsdokument:** `docs/SPEC.md` (Fassung 4.0)
**Ausgangslage:** Die Reduktion ist committet (`ed402e21a`). Der Workspace ist auf Kern v1 verkleinert, aber **nicht lauffähig**. Dieser Plan führt von dort zu einem grünen v1-Kandidaten.

---

## 0. Ausgangslage in einer Seite

| Merkmal | Ist (30.10.2026) |
|---|---|
| Workspace-Mitglieder | 14 Kern-Crates, `contextra-testkit`, `contextra-bench` (`benchmarks/`), `xtask` (**fehlt**) |
| Kompilierbarkeit | Nicht gegeben (SPEC §16, B1–B4) |
| Quality-Gates | Nicht ausführbar: Runner `xtask` entfernt, Katalog `governance/gates.toml` verwaist |
| Dokumentation | Vorzustand (SPEC 3.1, PLAN 3.1, README, AGENTS, CLAUDE, ARCHITECTURE) → in dieser Revision ersetzt |
| Restore-Punkt | `3c1f144fa` (nicht der Tag `pre-reduction-1.0`, der fehlt) |
| Größte Residuenfläche | `contextra-engine` (29 Dateien), `contextra-db` (12) |

**Leitgedanke:** Erst bauen, dann verifizieren, dann dokumentieren, dann Gates neu aufbauen. Kein Gate wird abgeschwächt, um einen Fehler zu verdecken (Regel 2, §3.2 der SPEC).

---

## 1. Phasenübersicht

| Phase | Ziel | Abhängig von | Ergebnis |
|---|---|---|---|
| **R0** | Workspace lädt, `cargo check --workspace` grün | – | Build-Blocker B1–B4 beseitigt |
| **R1** | Lints und Tests grün, Doppeltypen bereinigt | R0 | Tests grün, Regel 11 erfüllt |
| **R2** | Governance und Tooling an Ist-Stand angeglichen | R0 | `capabilities.toml`, Gates, CI, Justfile konsistent |
| **R3** | Dokumentation an Ist-Stand angeglichen | R0 (parallel zu R1, R2) | Alle Markdown-Dokumente korrekt, Historie gekennzeichnet |
| **R4** | Abnahme des v1-Kandidaten | R1, R2, R3 | Release-Entscheidung D-R7 |
| **R5** | Offene Scope-Entscheidungen | R4 | D-R4, D-R9 entschieden |
| **R6** | Wiedereintritt entfernter Bausteine | eigener Plan je Baustein | Außerhalb v1 |

Aufwand: S = bis 1 Tag, M = 2 bis 4 Tage, L = mehr als 4 Tage (Einzelperson, grobe Schätzung).

---

## 2. Phase R0 – Build wiederherstellen

**Ziel:** `cargo check --workspace` läuft ohne Fehler durch. Keine Ausnahmen über `#[allow]` für Modulfehler.

### WP R0.1 – Workspace-Manifest bereinigen (S)

- `Cargo.toml`: `"xtask"` aus `members` entfernen. Die Entscheidung über einen neuen Runner fällt in D-R2 (SPEC §15). Bis dahin entfallen alle `cargo xtask`-Aufrufe.
- Kommentarzeilen zu `contextra-cluster`, `contextra-sandbox`, `contextra-saos-agent` und `contextra-embed` entfernen (keine Wirkung, verwirren).
- `[workspace.dependencies]`: Einträge `contextra-index` (Alias auf `contextra-vector`) und die auskommentierten Frozen-Zone-Abhängigkeiten prüfen. Ungenutzte Einträge löschen.

**Abnahme:** `cargo metadata --no-deps --format-version 1 > /dev/null` läuft fehlerfrei.

### WP R0.2 – Fassade `contextra` (S)

- `crates/contextra/src/lib.rs`: `contextra_core::error::ContextraError`, `contextra_core::types::domain::{DocId, ScoredDocument}` und `contextra_core::DistanceMetric` auf die Pfade in `contextra-types` umstellen. Pfade vorher mit `grep -rn "pub use" crates/contextra-types/src` belegen, nicht raten.
- Feature-Block `#[cfg(feature = "candle")] pub use contextra_infer_candle as candle;` ersatzlos entfernen.
- Tests in `crates/contextra/tests/` (`wiring_matrix.rs`, `llm_query_rewriter.rs`, `collection_profile_matrix.rs`) auf verbleibende Pfade prüfen.

**Abnahme:** `cargo check -p contextra` grün.

### WP R0.3 – Engine von Residuen befreien (M)

Größter Block: 29 Dateien mit `contextra_graph`, `contextra_adapt` oder `contextra_rank::…` außerhalb des Kerns.

- `collection/relate.rs`: Hyperedge-Logik basiert auf `contextra_graph::hyperedge`. Die Funktion ist Teil der entfernten Graph-Funktionalität. Entscheidung: Funktion und Aufrufer entfernen (keine Stub-Implementierung, siehe Regel 2 und das Verbot von Security-Stubs). Betroffene Tests entsprechend streichen.
- `background_workers/expiry_workers.rs`, `background_workers/tests.rs`: `AdaptiveDecayController` ist entfernt. Expiry-Worker auf die verbleibende Logik reduzieren oder streichen (Entscheidung im PR begründen).
- `collection/maintenance.rs`: `detect_communities` und Perkolations-Aufrufe (`contextra_graph::percolation`) entfernen. Feature `graph-connectivity-health` ist nicht deklariert.
- `collection/query_builder/*`: Feature `adaptive-candidate-pool-sizing` (PID-Regler aus `contextra_adapt`) entfernen. Fester Kandidatenpool als Standard, Konfigurationsfeld nur, wenn ein Test es braucht.
- `collection/crud/auto_extraction.rs`: Feature `entity-extraction` und `auto-extraction-opt-out` sind nicht deklariert. Die Funktion ist Teil der entfernten Extraktion, also den Block samt Aufrufern entfernen.
- `contextra_impl/erasure.rs` fehlt bereits. Alle `mod`- und `use`-Verweise darauf entfernen.

**Abnahme:** `cargo check -p contextra-engine` grün. `grep -rnE "contextra_(graph|adapt|cognition|wire|infer|privacy|mcp|checkpoint)" crates/contextra-engine` liefert keine Treffer.

### WP R0.4 – Fassade `contextra-db` (M)

12 Dateien mit Residuen. Schwerpunkt: `advanced`-Modul (Re-Exporte aus `adapt`, `cognition`, `graph`), `multistep`/`homeostat` (entfernt, Dateien fehlen), `wire`-Referenzen im Modul-Kopf.

- `advanced`-Modul auf verbleibende Subsysteme beschränken (D-R9 iii). Der Modul-Kopf in `lib.rs` wird in R3 neu formuliert.
- Beispiele `examples/quickstart.rs`, `hybrid_search.rs`, `collections.rs`: an verbleibende API anpassen.
- Feature `volatile-vault`, `background-maintenance`, `bench` bleiben, sofern deklariert.

**Abnahme:** `cargo check -p contextra-db --all-targets` grün.

### WP R0.5 – Store, Rank, Types, Bench (S)

- `contextra-store`: 1 Datei mit Scope-Verweis auf entfernte Scanner-Logik; 7 Dateien mit entfernten Bezeichnern. Prüfen, ob `scanner`-Verweise im Store reine Kommentare sind (dann bereinigen) oder Code (dann entfernen).
- `contextra-types`: 3 Dateien mit Crate-Verweisen. `types/src/types/saos.rs` ist Restmodul des entfernten Agenten-Kontexts. Prüfen, ob es von irgendetwas außerhalb `types` genutzt wird. Wenn nein: Modul und `mod`-Zeile entfernen.
- `contextra-rank`: 2 Dateien. `dibud`-Feature ist deklariert, der Modul-Kopf verweist auf die entfernte Kognition. Kommentar anpassen.
- `benchmarks/contextra-bench`: 1 Datei. Externe Benchmarks (`locomo`, `long_mem_eval`) sind laut Reduktionsskript entfernt. Verweise bereinigen, Feature `onnx-bench` entfernen.

**Abnahme:** `cargo check --workspace --all-targets` grün (ohne Tests).

### WP R0.6 – Feature-Konsistenz (M)

Befund B4. Jedes verwendete, nicht deklarierte Feature wird entschieden:

| Feature | Vorgehen |
|---|---|
| `entity-extraction`, `auto-extraction-opt-out`, `onnx`, `onnx-bench`, `candle`, `reranking`, `sandbox`, `experimental-diskann`, `graph-connectivity-health`, `adaptive-candidate-pool-sizing`, `coherence-bonus-fusion` | Entfernen. Code und Tests entsprechend anpassen (siehe R0.3). |
| `encryption-at-rest`, `wal-integrity` | **Nicht als Feature führen** (Regel 8). Der gesicherte Pfad ist Standard, `cfg`-Blöcke entfernen. Ein Test prüft, dass der Schutz im Default-Build aktiv ist. |
| `test-utils`, `test-hooks`, `fault-injection`, `partial-index-rebuild`, `docid-128` | Deklaration in allen Crates prüfen, in denen sie verwendet werden (`grep -rn 'feature = "<name>"'` gegen `Cargo.toml`). Fehlende Deklaration ergänzen oder Verwendung entfernen. |
| `graph` (in `contextra-vector`) | Entfernen. Verweist auf die entfernte Graph-Crate. |

**Abnahme:** `RUSTFLAGS="-D unexpected_cfgs" cargo check --workspace --all-targets` ohne Fehler. Zusätzlich Skript, das die Liste aus SPEC §16 B4 gegen `Cargo.toml` abgleicht (siehe R2.2).

### WP R0.7 – Lockfile und Erstcheck (S)

- `Cargo.lock` aktualisieren: `cargo update --workspace` nur, wenn nötig. Keine Versionssprünge ohne Begründung.
- `cargo check --workspace --all-targets` vollständig grün.
- `cargo test --workspace --no-run` grün (Kompilierung der Tests, noch ohne Ausführung).

**Exit-Kriterium R0:** Build grün, dokumentiert im Commit `WP-R0.7: Build grün`.

---

## 3. Phase R1 – Lints, Tests, Doppeltypen

### WP R1.1 – Clippy mit Workspace-Lints (M)

- `cargo clippy --workspace --all-targets -- -D warnings` durchlaufen lassen. Erwartung: Verstöße gegen `unwrap_used`, `expect_used`, `indexing_slicing`, `panic` in Produktionscode und Tests. Tests dürfen `#[cfg(test)]`-Ausnahmen nutzen, nur wenn die Lints dort explizit gelockert sind. Die Ausnahme wird im Crate-Kopf begründet.
- Keine globalen `#![allow(...)]` einführen.

**Abnahme:** `cargo clippy --workspace --all-targets -- -D warnings` grün.

### WP R1.2 – Testsuite (M)

- `cargo test --workspace` ausführen. Fehlschlagende Tests triagieren: (a) Test testet entfernte Funktion → Test streichen, im PR begründen; (b) Test testet verbleibende Funktion → Code reparieren.
- Testdateien mit ausschließlichem Bezug auf Entferntes (Muster: `graph`, `adapt`, `agent`, `cognition`, `wire`, `deletion_proof`) sind nach Plan streichbar. Die Liste erzeugen wir in R1.2 mit `grep -l` und reviewen sie einzeln.
- Property-Tests (`proptest-regressions/`) unverändert lassen. Ihre Regressionsdateien sind Evidenz.

**Abnahme:** `cargo test --workspace --locked` grün.

### WP R1.3 – Doppeltypen (S)

Befund B7 (SPEC §16).

| Typ | Vorgehen |
|---|---|
| `BoundedTopK` | Kanonische Definition in `contextra-types` verschieben. `rank/src/fusion/topk.rs` und `text/src/wand.rs` importieren von dort. Eine neue Abhängigkeit `text → rank` wird nicht eingeführt. Ist die Verschiebung in `types` nicht sauber möglich (Generik-Bounds), wird die Textvariante als private Implementierung mit eigenem Namen (`WandTopK`) geführt. |
| `HybridQueryBuilder` | Nur in `contextra-engine` behalten. Typ in `types/saos.rs` entfernen. |
| `InMemoryStorageEngine` | Nur in `contextra-testkit`. Kopie in `store/src/engine/memory_only.rs` prüfen: Feature `memory-only-storage` ist deklariert und genutzt, daher Entscheidung nötig. Bevorzugt: Die Store-Variante bleibt die einzige Implementierung. `contextra-testkit` (Tooling) darf `store` als Abhängigkeit nutzen und die Store-Variante re-exportieren oder umhüllen. Ring 1 darf Tooling nicht kennen. |
| `Signal` | Kanonisch in `contextra-rank` (`fusion/types.rs`). `types/saos.rs` entfernen. |

**Abnahme:** `grep -rhoE "^\s*pub(\(crate\))? (struct|enum|trait|type) [A-Za-z0-9_]+" crates --include=*.rs | awk '{print $NF}' | sort | uniq -d` liefert nur bewusst gewollte Mehrfachnamen (`Result`, `DocId` über Modulgrenzen mit Re-Export). Liste in `docs/TYPE_REGISTRY.md` begründen.

### WP R1.4 – Ports bereinigen (S)

- Für jeden Port mit Status [O] in SPEC §6: Konsumenten suchen (`grep -rn "<Trait>"`). Ohne Konsument und ohne Implementierung: Port entfernen.
- Ports mit Konsumenten, aber ohne Implementierung in v1 (z. B. `embedding`, `reranker`): entweder Implementierung (kein Stub) oder Entfernen. Kein `NullEmbedding` als Standard, das ist ein Security-Stub im weiteren Sinn und entspricht Regel 8 nicht.

**Abnahme:** SPEC §6 wird aktualisiert, Status jedes Ports ist [K], [T] oder entfernt.

### WP R1.5 – Unsafe-Inseln (S)

- `tests/unsafe_islands.rs` ausführen. Erwartung: nur `contextra-sys` und `contextra-simd` enthalten `unsafe`.
- Jeder `unsafe`-Block in `contextra-simd` (rund 270 Stellen) und `contextra-sys` (21 Stellen) trägt einen `// SAFETY:`-Kommentar. Fehlende Kommentare werden nachgetragen. Das ist kein Refactoring, sondern Dokumentation der bestehenden Annahmen.

**Abnahme:** `tests/unsafe_islands.rs` grün. `grep -rn "unsafe" crates/contextra-simd/src crates/contextra-sys/src | grep -v SAFETY` ist nur auf Kommentar- und Deklarationszeilen beschränkt. Diese Prüfung ersetzt das bisherige Gate `unsafe-audit` vorübergehend.

### WP R1.6 – Fuzz-Targets (S)

- `contextra-text/fuzz/`, `contextra-db/fuzz/`, `contextra-crypto/fuzz/`, `contextra-vector/fuzz/`, `contextra-store/fuzz/`: keine Workspace-Mitglieder. Kompilierbarkeit prüfen (`cargo check --manifest-path … `). Bei Residuen entsprechend bereinigen. Keine Ausführung in R1, nur Kompilierung.

---

## 4. Phase R2 – Governance, CI und Tooling

### WP R2.1 – `capabilities.toml` neu erzeugen (M)

- Ein Eintrag je verbleibendem Crate (15 Einträge). Felder: `ring`, `maturity`, `path`, `may_depend_on` (aus SPEC §4.2), `ci_class`, `test`.
- Alle Einträge für entfernte Crates werden gelöscht. Die Datei enthält derzeit 98 Verweise auf Entferntes (B5).
- `may_depend_on` wird aus den Manifesten generiert, nicht von Hand geschrieben. Dafür ein kleines Skript in `tools/` (Python, Standardbibliothek, `tomllib`), das gegen die Manifeste prüft.

**Abnahme:** Skript meldet keine Abweichungen zwischen `capabilities.toml` und Manifesten.

### WP R2.2 – Ring-Layering-Test (S)

- `tests/layering.rs`: Ring-Zuordnung aus SPEC §4.1 übernehmen (`engine` = Ring 2). Die Ringtabelle dort ist die einzige Quelle, keine zweite Liste im Testcode.
- Prüfung: Ring-0-Crates ohne `tokio` (Ist-Prüfung zuerst, dann als Test festschreiben).
- Skript-Prüfung B4 aus R0.6 als Test ablegen, damit sie in `cargo test` läuft.

**Abnahme:** `cargo test --test layering` grün.

### WP R2.3 – Gate-Katalog bereinigen (M)

- `governance/gates.toml`: Gates für entfernte Komponenten (Python, MCP, PyO3, Ollama, Candle, ONNX, Sandbox, Privacy, Ingest, Löschkette-Scanner) streichen. Das betrifft laut Katalogstruktur einen großen Teil der Einträge.
- Verbleibende Gates auf direkte `cargo`-Befehle oder Tests abbilden (D-R2 Option b). Jedes Gate erhält einen ausführbaren `cmd`-Eintrag.
- Gates, die nur mit dem alten Runner funktionieren, werden als `status = "pending"` markiert, mit Verweis auf D-R2.

**Abnahme:** Jedes Gate mit `status = "active"` lässt sich mit dem im Eintrag hinterlegten `cmd` ausführen und ist grün.

### WP R2.4 – Justfile (S)

- 23 Aufrufe von `cargo xtask` (Justfile und Workflows zusammen) durch die entsprechenden Gate-Befehle aus R2.3 ersetzen oder streichen.
- `just check` als einziger Einstieg für lokale Prüfung: `cargo fmt --check`, `cargo clippy`, `cargo test`, Layering-Test.

**Abnahme:** `just check` läuft lokal grün.

### WP R2.5 – CI-Workflows (M)

- 34 Workflows in `.github/workflows/` inventarisieren. Für jeden eine Entscheidung: behalten, anpassen, streichen. Vorschlag:
  - **Streichen (Relikte, Funktion eindeutig entfernt):** `publish-pypi.yml`, `build-wheels.yml` (Python-Wheels), `release-mcp-binary.yml` (MCP-Binary).
  - **Vor Entscheidung lesen:** `nucleation-recall-history.yml`, `update-prompter-data.yml`, `refresh-jules-context.yml`. Ihre Funktion ist im Rahmen dieser Fassung nicht geprüft. Entscheidung erst nach Lektüre.
  - **Anpassen:** `rust-ci.yml`, `merge-gate.yml`, `coverage.yml`, `deny.yml`, `unsafe*`, `tsan.yml`, `loom-gate.yml`, `bench*.yml`, `mutation-testing.yml`, `audit-integrity.yml`.
  - **Zu prüfen:** `jules-dispatch.yml`, `veto-deadline-gate.yml`, `verdict.yml`, `shell-commit-gate.yml`, `commit-integrity-guard.yml`. Diese Workflows koppeln an Agentenprozesse, deren Regeln in `.agents/` beschrieben sind. Entscheidung in R2.5 mit Maintainer.
- `.github/CODEOWNERS` und `.github/rulesets/*.json` auf entfernte Pfade prüfen.

**Abnahme:** Kein Workflow ruft `cargo xtask` auf. CI-Lauf auf einem Testbranch ist grün für `rust-ci` und `deny`.

### WP R2.6 – Schutzpfade und Ratchet (S)

- `governance/protected-paths.toml`: Eintrag `xtask/**` entfernen, Einträge für entfernte Crates streichen.
- `governance/ratchet.toml`: Baselines nach R0/R1 neu messen und setzen. Der Vorgang wird im Changelog mit Begründung dokumentiert (D-R6). Ratchets dürfen nicht über alte Werte hinaus gesetzt werden, aber nach der Reduktion sind die alten Zahlen nicht mehr vergleichbar.
- `governance/security_ports.toml` und `governance/key_purposes.toml`: Ports und Schlüsselzwecke auf den Ist-Stand bringen. Entfernte Zwecke (Löschkette) raus.

**Abnahme:** Governance-Dateien referenzieren nur existierende Crates und Pfade. Skript prüft das.

---

## 5. Phase R3 – Dokumentation

Diese Phase läuft parallel zu R1 und R2. Die Dokumente werden zusammen mit der Code-Änderung gemergt, sonst entsteht erneut Drift.

### WP R3.1 – Zentrale Dokumente (erledigt in dieser Revision, Review offen)

- `docs/SPEC.md` (Fassung 4.0), `docs/PLAN.md` (Fassung 4.0), `docs/ARCHITECTURE.md`, `README.md`, `AGENTS.md`, `CLAUDE.md`.
- **Offen:** Review durch den Maintainer. Insbesondere D-R2 bis D-R9 sind Entscheidungen, keine Fakten.

### WP R3.2 – Crate-Dokumentation (M)

- Für jedes verbliebene Crate `README.md` und `AGENTS.md` an den Ist-Stand angleichen: Ring, Klasse, Abhängigkeiten, Entfernte Teile, offene Punkte aus SPEC §16.
- Crates mit besonders vielen Residuen (`store`, `db`, `engine`, `crypto`, `testkit`) zuerst.
- Tabellen mit `may_use`/`may_depend_on` aus `capabilities.toml` (R2.1) übernehmen, nicht aus dem Gedächtnis.

### WP R3.3 – Historische Dokumente kennzeichnen (S)

- `docs/analysis/*`, `docs/spec/GOVERNANCE-PATHS.md`, `docs/spec/MIGRATION-EDGES.md`, `REDUCTION_RESIDUALS.md`, `.agents/jules/*LOG*`: Statuskopf „Historisch, Stand vor Reduktion“ mit Verweis auf SPEC §1.1. Inhalt nicht umschreiben.
- `docs/CHANGELOG.md`: Eintrag für die Reduktion oben einfügen. Ältere Einträge unverändert.
- `REDUCTION_RESIDUALS.md`: Restore-Hinweis korrigieren (Tag existiert nicht, D-R6).

### WP R3.4 – ADR-Status (S)

| ADR | Neuer Status |
|---|---|
| 0001 Gemessene Löschung | Historisch, ausgesetzt (D-R5) |
| 0002 Ingest-Adapter-Crate | Überholt: Crate entfernt |
| 0003 Archiv-Crates-Beförderung | Überholt durch Reduktion (D-R1) |
| 0004 AGENTS/CLAUDE Abschnitt 5.4 | Historisch, Inhalt in AGENTS 4.0 übernommen |
| 0005 Panic-Measurement-Modell | Gültig, Prüfung gegen Regel 2 |
| 0006, 0007, 0008 Löschkette | Ausgesetzt (D-R5) |
| 0077, 0098, 0099 | Gültig, Prüfung auf Verweise in R3.2 |

Die Statusänderung wird im jeweiligen ADR vermerkt, nicht durch Löschen.

### WP R3.5 – `docs/TYPE_REGISTRY.md` (S)

- Vollständige Neufassung auf Basis von SPEC §16 B7 und der Ergebnisse aus R1.3. Die frühere Matrix mit über 35 Typen aus entfernten Crates wird durch die Liste der verbleibenden Doppelnamen ersetzt. Der Vorzustand bleibt über `3c1f144fa` erreichbar.

---

## 6. Phase R4 – Abnahme des v1-Kandidaten

**Exit-Kriterien (alle erforderlich):**

1. `cargo fmt --all -- --check` grün.
2. `cargo clippy --workspace --all-targets -- -D warnings` grün.
3. `cargo test --workspace --locked` grün.
4. `tests/layering.rs` und `tests/unsafe_islands.rs` grün.
5. Loom-Tests grün: `crates/contextra-mvcc/tests/loom_snapshot_registry.rs`. Die Datei `crates/contextra-db/tests/loom_relate_n_ary.rs` gehört zur Hyperedge-Funktionalität aus R0.3 und ist zu streichen.
6. `cargo deny check` grün (`deny.toml`).
7. Benchmark-Regressionslauf (`contextra-bench`, `regression_gate`) gegen neu gesetzte Baseline.
8. Keine Verweise auf entfernte Crates in `docs/`, `README.md`, `AGENTS.md`, `CLAUDE.md` (außer in historischen Dokumenten mit Statuskopf).
9. Release-Entscheidung D-R7 dokumentiert.

**Hinweis zu Mutation-Tests und Fuzzing:** Die Workflows existieren, sind aber nicht Exit-Kriterien von v1. Sie laufen nach R4 als nachgelagerte Gates.

---

## 7. Phase R5 – Offene Scope-Entscheidungen

| Entscheidung | Vorbereitung | Ergebnis-Dokument |
|---|---|---|
| **D-R4 RaBitQ** | Messung gegen SQ8 auf `contextra-bench` (Recall und Latenz). Entscheidung: Freigabe als `experimental` mit Messung oder Entfernung. | ADR-0009 |
| **D-R9 (i) KDF** | Abgleich HKDF-Ist mit Fassung-3.1-Vorgabe Argon2id. Ob Passwortpfad existiert. | ADR-0010 |
| **D-R9 (ii) Audit-Kette und Anti-Tamper** | Prüfen, ob Teil von v1. Ohne Konsument: entfernen. | ADR-0011 |
| **D-R5 Löschkette** | Nur bei Bedarf. Dann eigener Scope-Plan, nicht Fortführung der 3.1-Fassung. | ADR-00xx |

---

## 8. Phase R6 – Wiedereintritt entfernter Bausteine (außerhalb v1)

Jeder Baustein aus SPEC §5.2 braucht einen eigenen Plan. Die Reihenfolge entscheidet der Maintainer. Vorbedingungen für jeden Wiedereintritt:

1. Ein ADR, der den Nutzen und die Abweichung vom Nicht-Ziel begründet.
2. Der Baustein wird über `git checkout 3c1f144fa -- <Pfad>` in einen Branch geholt und gegen den aktuellen Stand neu geprüft (keine Übernahme ungeprüfter Teile).
3. Ring- und Abhängigkeitsregeln aus SPEC §4 werden eingehalten. Ein Baustein, der Ring 0 mit einer Abhängigkeit auf Ring 2 überschreitet, wird nicht übernommen.
4. Gates des Bausteins sind wieder aktiv und grün.

---

## 9. Testmatrix (SPEC-Anforderung → Prüfung → Arbeitspaket)

| SPEC-Anforderung | Prüfung | WP |
|---|---|---|
| Regel 1: Ring-Abhängigkeiten | `tests/layering.rs` | R2.2 |
| Regel 2: Kein Panic in Produktion | Clippy-Lints (`-D warnings`) | R1.1 |
| Regel 3: Determinismus über Ports | Review; Ziel ist ein Lint oder Test in R5 | offen |
| Regel 5: `fsync`-Fehler propagieren | Clippy-Lint `let_underscore_must_use` prüfen; Review | R1.1 |
| Regel 6: Unsafe nur in Inseln | `tests/unsafe_islands.rs`; SAFETY-Prüfung | R1.5 |
| Regel 7: Mandant aus RequestContext | `contextra/tests/tenant_isolation_facade.rs`, `contextra-db` Mandantentests | R1.2 |
| Regel 8: Sicherheit ist kein Feature | Test, dass Verschlüsselung im Default-Build aktiv ist | R0.6 |
| Regel 10: Fail-Closed | `contextra-store` Recovery-Tests | R1.2 |
| Regel 11: Eine Typfamilie | Doppeltyp-Prüfung | R1.3 |
| VETO-F02: HNSW-Löschung | `contextra-vector` Partial-Rebuild-Tests | R1.2 |
| VETO-F10: Mandantentrennung | `contextra/tests/tenant_isolation_facade.rs` | R1.2 |
| Persistenz: WAL-Replay, Recovery | `contextra-store/tests/` (Wal- und Lifecycle-Tests) | R1.2 |

---

## 10. Risiken

| Risiko | Wirkung | Gegenmaßnahme |
|---|---|---|
| Residuen in `engine` lösen Folgefehler aus (Typen, Traits, Re-Exports) | R0 dauert länger als geschätzt | Schrittweise Kompilierung je Modul, Fehlerzahl pro Commit protokollieren |
| Tests testen entfernte Funktion und verdecken fehlende Abdeckung | Falsches Vertrauen in v1 | Jeder Test-Streichung wird ein Eintrag im PR zugeordnet |
| Gate-Katalog wird verkürzt, um Grün zu erreichen | Verstoß gegen REQ-QA-010 | Gate-Streichungen nur mit ADR-D-R2-Verweis; Review durch Maintainer |
| Dokumentation driftet wieder vom Code ab | Agenten arbeiten gegen falsche Vorgaben | Doku-Änderungen werden mit Code-Änderungen im selben PR gemergt; R2.1 erzeugt Konsistenzprüfung |
| Sicherheitsrelevante Funktion fällt in Reduktion weg (z. B. Verschlüsselung) | Schwerer Befund | Regel 8 und Test aus R0.6; Verschlüsselung ist kein optionales Feature |
| Fehlender Rust-Toolchain in der Prüfumgebung | Befunde bleiben unbestätigt | Erste Aktion in R0: lokale Kompilierung, Befunde aus SPEC §16 bestätigen oder korrigieren |

---

## 11. Definition of Done je Phase

- **R0:** `cargo check --workspace --all-targets` grün. Commit enthält keine `#[allow]`-Erweiterungen.
- **R1:** `cargo test --workspace --locked` und Clippy mit `-D warnings` grün. `docs/TYPE_REGISTRY.md` aktualisiert.
- **R2:** Jedes Gate mit Status `active` ist ausführbar und grün. `capabilities.toml` stimmt mit den Manifesten überein. Keine Workflow-Datei ruft `cargo xtask` auf.
- **R3:** Alle Markdown-Dokumente außerhalb von `docs/analysis/` und `docs/decisions/` haben keine Verweise auf entfernte Crates ohne Statuskopf. ADR-Status gesetzt.
- **R4:** Alle Exit-Kriterien aus §6 erfüllt, Release-Entscheidung dokumentiert.

---

## 12. Nächste Schritte (erste zwei Wochen)

1. **Tag 1:** Rust-Toolchain (Version laut `rust-toolchain.toml`) bereitstellen. `cargo metadata` und `cargo check --workspace` ausführen, Befunde B1–B4 gegen die Ausgabe verifizieren. Ergebnis in SPEC §16 vermerken.
2. **Tag 1–2:** WP R0.1 und R0.2. Ziel: Manifest lädt, Fassade kompiliert.
3. **Tag 2–6:** WP R0.3 (Engine). Fortschritt je Modul als Commit.
4. **Tag 6–8:** WP R0.4 bis R0.6.
5. **Tag 8–9:** WP R0.7. Ergebnis: grüner Build.
6. **Parallel ab Tag 9:** R1.1 und R1.2 (Clippy und Tests), R3.2 (Crate-Dokumentation).
7. **Maintainer-Entscheidungen bis Ende Woche 2:** D-R2 (Gate-Runner), D-R3 (Ring von `engine`), D-R7 (Release-Version).

---

## Anhang P1. Abbildung PLAN 3.1 → PLAN 4.0

| PLAN 3.1 | PLAN 4.0 | Hinweis |
|---|---|---|
| Phase 0 – Baseline und Wahrheit | Entfallen als Phase; Befunde in SPEC §16 | Baseline durch R0 ersetzt |
| Phase 1 – Sofortschäden | Teilweise durch Reduktion entfallen (u. a. Löschkette, MCP). Verbleibende Befunde (Stubs, Literal-Schlüssel) im Code prüfen, bevor sie als erledigt gelten. | Kein eigenes WP mehr; Prüfung in R1 |
| Phase 2 – Entkopplung | R0.3, R0.4 (Residuen entfernen statt entkoppeln) | Entkopplung entfällt, da Module entfernt |
| Phase 3 – Modul-Infrastruktur | Entfällt (Module entfernt) | Wiedereintritt in R6 |
| Phase 4 – Persistenz-Fundament | R1.2 (Tests Store) | Sidecar-Logs entfallen |
| Phase 5 – Löschkette | Entfällt (SPEC §9, D-R5) | Wiederaufnahme nur über ADR |
| Phase 6 – Modul-Härtung F1–F5 | Entfällt (Module entfernt) | R6 |
| Phase 7 – Profile, Beispiele, Dokumentation | R3 | Profile entfallen |
| Phase 8 – Abnahme und Release | R4 | Version D-R7 |

## Anhang P2. Änderungsverlauf

| Fassung | Datum | Änderung |
|---|---|---|
| 3.1 | 28.10.2026 | Baukasten-Fassung |
| 4.0 | 30.10.2026 | Umstellung auf Reduktion: Build-Wiederherstellung (R0), Qualität (R1), Governance (R2), Dokumentation (R3), Abnahme (R4), offene Entscheidungen (R5), Wiedereintritt (R6) |
