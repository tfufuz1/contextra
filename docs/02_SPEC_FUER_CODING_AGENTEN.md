# Technische Spezifikation: Contextra Hybrid-Retrieval-Kern (für Coding-Agenten)

> Status: verbindlich für die Umstrukturierung · Basis: `tfufuz1/contextra` (Stand 9. November 2026)
> Zielgruppe: Coding-Agenten und menschliche Entwickler
> Begleitdokument: `01_STRATEGIE_UND_ENTSCHEIDUNGEN.md` (Begründungen)

---

## 1. Ziel und Nicht-Ziele

### 1.1 Ziel
Aus dem bestehenden Workspace einen **schlanken, produktionsnahen Kern** für eine eingebettete Hybrid-Suche bauen (Vektor, BM25, optional Graph, Rangfusion), dazu Python-Bindings und einen MCP-Server für Suche. Alle anderen Teile werden **archiviert, nicht gelöscht**.

### 1.2 Nicht-Ziele (ausdrücklich)
- Keine neue Funktion hinzufügen. Dies ist Konsolidierung, keine Feature-Entwicklung.
- Keine Löschlogik, keine Nachweis- oder Compliance-Funktionen, keine Lizenzprüfung.
- Keine Ring-Architektur und keine `tests/arch`-Regeln mit Ring-Zuordnung.
- Keine Umbenennung des Projekts oder der Crates in dieser Phase (Präfix `contextra-` bleibt).
- Keine Kryptografie-Änderungen außer dem Entfernen.
- Kein Umbau des Commit-Verlaufs.

### 1.3 Leitregeln für jeden Agenten
1. **Archivieren vor Löschen.** Code wird in `archive/` verschoben (siehe Abschnitt 6), nicht gelöscht.
2. **Ein Arbeitspaket, ein Commit.** Konventionelle Commit-Nachrichten (`refactor(scope): ...`).
3. **Jede Änderung dokumentiert.** Änderungen werden in `docs/MIGRATION_LOG.md` eingetragen.
4. **Im Zweifel stehen lassen und markieren.** Unklare Stellen mit `// TODO(migration): <Begründung>` markieren, nicht raten.
5. **Keine Änderung an öffentlichen Signaturen ohne Eintrag im Migrationslog.**

---

## 2. Zielstruktur

### 2.1 Kern (Produkt, bleibt und wird gehärtet)

| Crate | Rolle | Interne Abhängigkeiten (Ziel) |
|---|---|---|
| `contextra-types` | Gemeinsame Typen, IDs, Fehler, Filter | keine |
| `contextra-ports` | Nur die cross-crate Traits (siehe 4.2) | `types` |
| `contextra-durable-fs` | Crash-sichere Dateioperationen | keine |
| `contextra-mvcc` | Sequenz-Log und Transaktionspuffer | `types`, `ports` |
| `contextra-store` | LSM-Speicherkern | `types`, `ports`, `durable-fs`, `mvcc` |
| `contextra-simd` | Distanzfunktionen mit Skalar-Referenz | `types` |
| `contextra-vector` | HNSW-Vektorindex | `types`, `ports`, `simd`, `store`(nur über Port) |
| `contextra-text` | BM25-Textindex | `types`, `ports` |
| `contextra-graph` | CSR-Graph und Personalized PageRank | `types`, `ports` |
| `contextra-rank` | RRF (Standard), gewichtete RRF, optional mRRF | `types` |
| `contextra-engine` | Orchestrierung der Suche (nur Suche) | `types`, `ports`, `store`, `vector`, `text`, `graph`, `rank` |
| `contextra-db` | Produkt-Fassade (Embedded-API) | `engine`, `types` |
| `contextra-py` | PyO3-Bindings für `db` | `db` |
| `contextra-mcp` | MCP-Server, nur Suche | `db` |

### 2.2 Bausteine (entnehmbar, eigenständig nutzbar)

| Crate | Rolle | Abhängigkeiten |
|---|---|---|
| `contextra-kvcache` | Prefix-Radix-Tree, Segment-Pool, Tiering | `types` |
| `contextra-checkpoint` | Backup und Snapshots | `types`, `ports`, `store` |
| `contextra-router` | Routing (nach Entkopplung von `adapt`) | `types`, `ports` |
| `contextra-infer-ollama` | REST-Adapter für Ollama | `types`, `ports` |
| `contextra-ingest` | Dokument-Extraktion und Chunking | `types` |
| `contextra-testkit` | Dev-Dependency, nur in `[dev-dependencies]` | `types`, `ports` |

### 2.3 Tooling (bleibt)
- `benchmarks/contextra-bench`
- `fuzz/` Verzeichnisse der Kern-Crates

### 2.4 Archiv (aus dem Workspace entfernen, nach `archive/` verschieben)

Siehe Abschnitt 6 für die Regeln. Betroffen: `erasure`, `crypto`, `wire`, `sys`, `privacy`, `sandbox`, `cognition`, `adapt`, `agent`, `infer-candle`, `infer-onnx`, `license`, `audit-export`, `avv-generator`, `facade contextra` (wird in `db` aufgelöst).

**Entfernen (endgültig, nach Freigabe durch den Entwickler):** `audit-export`, `avv-generator`, `license`.

---

## 3. Arbeitspakete

Die Reihenfolge ist verbindlich. Jedes Paket hat Akzeptanzkriterien. Ein Paket gilt erst als abgeschlossen, wenn alle Kriterien erfüllt und im `MIGRATION_LOG.md` eingetragen sind.

### AP-0: Sicherung
- Tag `archive/pre-restructure-2026-11-09` auf den aktuellen HEAD setzen und pushen.
- Branch `main` bleibt die Arbeitslinie.
- **Akzeptanz:** Tag existiert im Remote, `git log` ist unverändert.

### AP-1: Archiv-Schnitt
- Archivierte Crates nach `archive/crates/` verschieben. Im Workspace-`Cargo.toml` aus `members` und `default-members` entfernen.
- `archive/README.md` anlegen: Liste der archivierten Crates mit einem Satz zum Grund und zur Abhängigkeit, die sie hatten.
- Die Verschiebung ist rein mechanisch. Kein Inhalt wird geändert.
- **Akzeptanz:** `Cargo.toml` enthält nur noch Kern, Bausteine und Tooling. Alle Verweise auf archivierte Crates in aktiven Crates sind als `TODO(migration)` markiert.

### AP-2: Fassaden und Verschlankung der Abhängigkeiten
- `contextra` (Facade) wird in `contextra-db` aufgelöst. Die öffentliche API von `db` übernimmt die Funktionen, die `contextra` bisher exportiert hat.
- Die Abhängigkeit `contextra-store → contextra-crypto` wird entfernt. Relikte in `store/src/kv/segment.rs` und `store/src/kv/delete_mode.rs` werden entweder entfernt oder hinter `#[cfg(feature = "legacy-erasure")]` gestellt. Der Standard-Build enthält sie nicht.
- Die Abhängigkeit `contextra-checkpoint → contextra-testkit` wird in `[dev-dependencies]` verschoben.
- **Akzeptanz:** `cargo tree -e normal` für `contextra-store` enthält keinen Eintrag mit `crypto`, `erasure` oder `sys`. Für `contextra-checkpoint` taucht `testkit` nicht in den normalen Abhängigkeiten auf.

### AP-3: Engine entschlacken
- `contextra-engine/src/contextra_impl/erasure.rs` und `lifecycle.rs` entfernen. Die Verantwortung für Löschung entfällt vollständig.
- Abhängigkeiten `crypto`, `sandbox`, `infer-onnx`, `mvcc` (falls nur für Löschung genutzt) aus `engine` entfernen.
- Engine enthält nur: Collection-Verwaltung, Hybrid-Suche, Index-Pflege. Prüfen, ob `collection/maintenance.rs` und `background_workers` nötig sind; wenn ja, behalten, wenn nicht, archivieren.
- **Akzeptanz:** `engine` hat höchstens 8 interne Abhängigkeiten (Ziel siehe 2.1). Keine Referenz auf `erasure` im Quellcode (`grep -r erasure crates/contextra-engine/src` liefert nichts außer Migrationskommentaren).

### AP-4: Ports verschlanken
- `contextra-ports` enthält nur Traits, die von mindestens zwei Crates genutzt werden. Verwendung prüfen mit `grep`.
- `Purger`, `LayerScanner` und alle Trait-Definitionen für Löschung werden entfernt.
- Verbleibende Traits: mindestens `Storage` (Key-Value-Schnittstelle), `VectorIndex`, `TextIndex`, `GraphIndex`, `Embedder` (falls genutzt). Jeder Trait bekommt eine Doku mit Beispiel.
- **Akzeptanz:** Jeder Trait in `ports` hat mindestens zwei Implementierungen oder eine Test-Implementierung in `testkit`.

### AP-5: Vector und Text auf Kern reduzieren
- `vector`: Module `acorn/` und `partial_rebuild.rs` prüfen. Was nicht für HNSW + Persistenz + Filter benötigt wird, archivieren.
- `text`: Morphologie auf die für BM25 nötige Tokenisierung reduzieren. Erweiterte Module nach `archive/`.
- `simd` wird von `vector` über Feature-Flag `simd-avx2` oder `simd-neon` gesteuert. Standard ist der Skalar-Kern.
- **Akzeptanz:** Öffentliche API von `vector` und `text` ist in einer `API.md` pro Crate dokumentiert. Nicht dokumentierte `pub`-Items sind `pub(crate)`.

### AP-6: Graph und Rank einschränken
- `graph`: Kern ist CSR, Persistenz und PPR. `community/`, `hyperedge.rs`, `session_dag.rs`, `cascade.rs` nach `archive/` oder hinter Feature `experimental-graph`.
- `rank`: RRF und gewichtete RRF bleiben Standard. mRRF und Kalibrierung (`isotonic`, `platt`, `drift`) hinter Feature `experimental-rank`.
- **Akzeptanz:** Default-Features von `rank` enthalten nur RRF. Default-Features von `graph` enthalten CSR und PPR.

### AP-7: Store entschlacken
- `store` hat 54k Zeilen inklusive Tests. Ziel: Kern (LSM, WAL, Memtable, SSTable, Compaction, Manifest) bleibt. `scanner/`, `system_pressure.rs`, `tenant_codec.rs` prüfen. Nicht benötigte Module archivieren.
- `unsafe` im Store (aktuell eine Stelle) ist zu dokumentieren oder zu entfernen.
- **Akzeptanz:** `grep -rn unsafe crates/contextra-store/src` liefert höchstens eine Stelle mit `// SAFETY:`-Kommentar.

### AP-8: SIMD-Entscheidung
- Vergleich: Eigener SIMD-Code (`contextra-simd`) gegen `simsimd` (externes Crate) gegen reine Autovektorisierung der Skalar-Implementierung.
- Vergleich per Benchmark in `benchmarks/`. Ergebnis in `docs/decisions/0001-simd.md` (ADR-Format).
- Entscheidung trifft der Entwickler, nicht der Agent. Der Agent liefert nur den Vergleich.
- **Akzeptanz:** ADR existiert, Entscheidung ist dokumentiert. Wenn eigener SIMD-Code bleibt: AVX-512-Kern hinter Feature `simd-avx512`, Standard ist Skalar.

### AP-9: Rückbau der Ring-Architektur
- `tests/arch` (Ring-Tabelle) entfernen. Ersetzt durch ein Skript `scripts/check_deps.sh`:
  - prüft, dass jeder Baustein nur die in Abschnitt 2 gelisteten internen Abhängigkeiten hat,
  - prüft, dass `#![forbid(unsafe_code)]` in allen Kern-Crates steht, außer in `simd`, `mvcc` (falls nötig) und dokumentierten Ausnahmen.
- **Akzeptanz:** Skript läuft durch, Ausnahmen sind in `scripts/unsafe_allowlist.txt` gelistet.

### AP-10: CI-Rückbau
- Entfernen: `jules-dispatch.yml`, `refresh-jules-context.yml`, `update-prompter-data.yml`, `verdict.yml`, `veto-deadline-gate.yml`, `shell-commit-gate.yml`, `commit-integrity-guard.yml`, `prune-branches.yml`, `nucleation-recall-history.yml`, `proof-correctness.yml`, `audit-integrity.yml`, `scheduled-audit.yml`.
- Behalten: `rust-ci.yml`, `coverage.yml`, `deny.yml`, `fuzz-extended.yml`, `mutation-testing.yml`, `bench.yml`, `bench-regression.yml`, `unwrap-gate.yml`, `contextra-py-ci.yml`, `build-wheels.yml`, `publish-pypi.yml`, `release-mcp-binary.yml`.
- Die übrigen Workflows prüfen und begründen oder entfernen.
- **Akzeptanz:** `docs/CI.md` listet jeden verbleibenden Workflow mit Zweck.

### AP-11: Dokumentation und Übergabe
- Siehe Abschnitt 7.
- **Akzeptanz:** Alle Pflichtdokumente aus Abschnitt 7 existieren.

---

## 4. Konventionen

### 4.1 Crate-Aufbau
Jedes Crate folgt demselben Aufbau:
```
crates/contextra-<name>/
├── Cargo.toml
├── README.md         # Zweck, Nutzung, Abhängigkeiten, Feature-Flags, Status
├── API.md            # Liste aller öffentlichen Items mit Kurzbeschreibung
├── src/
│   ├── lib.rs        # Nur Modul-Deklarationen und Re-Exports
│   └── ...
├── tests/            # Integrationstests
└── benches/          # optional
```
- `lib.rs` enthält keine Logik, nur Modulstruktur und `pub use`.
- Maximal 800 Zeilen pro Datei. Größere Dateien werden aufgeteilt.
- Maximal 12 Module pro Ebene. Mehr bedeutet, dass eine Untergliederung fehlt.

### 4.2 Schnittstellen
- Cross-Crate-Schnittstellen laufen über `contextra-ports`. Direkte Abhängigkeit auf konkrete Typen anderer Bausteine nur, wenn kein Port sinnvoll ist, und dann mit Kommentar.
- Öffentliche Typen sind `#[non_exhaustive]`, wo sich das Format künftig ändern kann.
- Fehlertypen pro Crate mit `thiserror`. Keine `Box<dyn Error>` in öffentlichen Signaturen.

### 4.3 Unsafe-Regel
- Standard: `#![forbid(unsafe_code)]` in jedem Crate.
- Ausnahmen nur in `simd` (wenn eigener SIMD-Code bleibt) und `store` (falls nötig), gelistet in `scripts/unsafe_allowlist.txt`.
- Jeder `unsafe`-Block hat einen `// SAFETY:`-Kommentar mit der konkreten Begründung.

### 4.4 Panics und `unwrap`
- In Bibliotheks-Code kein `unwrap()` und kein `expect()` außer in Tests und eindeutig beweisbaren Fällen mit Kommentar.
- Der bestehende Gate `unwrap-gate.yml` bleibt aktiv und darf nicht umgangen werden. Bestehende Verstöße werden in `docs/UNWRAP_DEBT.md` erfasst, nicht stillschweigend ignoriert.

### 4.5 Features
- Default-Features enthalten nur den stabilen Kern.
- Experimentelles hinter `experimental-*`.
- Plattformspezifisches hinter `simd-*`, `avx512`, `neon`.
- Keine Features mit Namen, die auf entfernte Funktionen verweisen (`erasure`, `shredding`, `audit`).

### 4.6 Benennung
- Modulnamen in `snake_case`, Typen in `PascalCase`, keine Abkürzungen außer etablierten (HNSW, BM25, LSM, WAL, PPR, RRF, CSR, MVCC).
- Keine Projektinterna wie „Welle“, „Klasse A“, „Ring“, „SPEC v4“ in Code, Kommentaren oder Namen. Diese Begriffe gehören nach `archive/`.

### 4.7 Sprache
- Code-Bezeichner und Code-Kommentare: Englisch.
- Dokumentation (README, API.md, Entscheidungen): Deutsch oder Englisch, pro Datei einheitlich.
- Fehlermeldungen: Englisch.

---

## 5. Testanforderungen

### 5.1 Pro Kern-Crate
- **Unit-Tests** für jede öffentliche Funktion mit nicht-trivialer Logik.
- **Integrationstests** in `tests/` für den typischen Nutzungspfad.
- **Property-Tests** (`proptest`) für Datenstrukturen: Store (Roundtrip, Reihenfolge), Vector (Nachbarschaft ist korrekt bei kleinen Mengen gegen Brute Force), Rank (Fusion ist permutationsinvariant bei gleichen Gewichten).
- **Referenztests** gegen eine Brute-Force-Implementierung für Vector-Suche (Recall@10 ≥ 0.95 auf Testdatensatz mit 10.000 Vektoren).

### 5.2 Pflicht-Pfade (End-to-End)
Für `contextra-db` existiert mindestens ein Test pro Pfad:
1. Dokument einfügen, Suche liefern Treffer.
2. Update eines Dokuments, alte Version wird nicht mehr gefunden.
3. Löschen eines Dokuments, Treffer verschwinden aus Vektor-, Text- und Graph-Index.
4. Neustart des Prozesses, Daten und Indizes sind wieder da.
5. Crash während Schreiben (mit `testkit::FaultVfs`), Zustand bleibt konsistent.
6. Hybrid-Suche mit einem, zwei und drei Signalen liefert erwartbare Rangfolge.

### 5.3 Testkit
- `contextra-testkit` wird nur in `[dev-dependencies]` verwendet.
- `ManualClock`, `InMemoryStorageEngine`, `FaultVfs` bleiben als Standard-Testhilfen.

### 5.4 Coverage-Ziel
- Kern-Crates: mindestens 75 % Zeilenabdeckung (gemessen mit `cargo llvm-cov`).
- Bausteine: mindestens 60 %.
- Archiv: nicht gemessen.
- Zahlen werden pro Crate in `docs/COVERAGE.md` festgehalten. Eine höhere Zahl bei weniger Tests ist kein Ziel.

### 5.5 Ignorierte Tests
- Die 20 `#[ignore]`-Tests werden gelistet in `docs/IGNORED_TESTS.md` mit Grund und Entscheidung (reparieren, löschen, archivieren).

### 5.6 Fuzzing
- Fuzz-Targets für: Store-Decoder (WAL, SSTable), Vector-Index-Deserialisierung, Text-Tokenisierung, MCP-Eingabe.
- Nicht ausgeführt werden müssen sie in dieser Phase; die Targets müssen kompilieren, sofern die Crates kompilieren.

### 5.7 Benchmarks
- Vergleichsbenchmark: RRF-Fusion gegen Vektor-only, BM25-only, Vektor+BM25. Auf festem Datensatz (mindestens 10.000 Dokumente, 200 Fragen mit Relevanzurteilen).
- Ergebnisse in `benchmarks/results/` als CSV, Reproduktionsbefehl in `benchmarks/README.md`.
- Vergleich gegen externe Systeme (z. B. Tantivy, LanceDB) ist ein Folgeschritt und nicht Teil dieser Phase, aber die Schnittstelle dafür wird vorbereitet.

---

## 6. Archivierungsregeln

Ein archiviertes Crate:
1. Liegt unter `archive/crates/<name>/` mit vollständigem Quellcode und Tests.
2. Ist aus dem Workspace entfernt (`Cargo.toml`), aber nicht gelöscht.
3. Hat eine Zeile in `archive/README.md`: Name, Originalzweck, letzter Commit vor Archivierung, Grund.
4. Verweist in `archive/README.md` auf die Stelle im Git-Verlauf (Tag `archive/pre-restructure-2026-11-09`).

Ein Crate wird **entfernt** (nicht archiviert), wenn es nach Freigabe des Entwicklers als unbrauchbar eingestuft ist: `audit-export`, `avv-generator`, `license`. Die Entfernung wird im Migrationslog begründet.

---

## 7. Pflichtdokumentation

Diese Dateien müssen existieren, bevor AP-11 als erledigt gilt:

| Datei | Inhalt |
|---|---|
| `README.md` (Repo-Root) | Was ist das, für wen, Schnellstart, Crate-Übersicht mit Kern/Baustein/Archiv |
| `ARCHITECTURE.md` | Abhängigkeitsgraph (als Mermaid), Begründung der Struktur, Ports-Konzept |
| `AGENTS.md` | Anleitung für Coding-Agenten: Konventionen aus Abschnitt 4, Leitregeln aus 1.3, verbotene Änderungen |
| `docs/MIGRATION_LOG.md` | Chronologisches Protokoll jeder Änderung mit Arbeitspaket-Nummer |
| `docs/decisions/` | ADRs für wesentliche Entscheidungen (mindestens SIMD, Archivierung, Ring-Rückbau) |
| `docs/CI.md` | Liste der Workflows mit Zweck |
| `docs/COVERAGE.md` | Abdeckung pro Crate |
| `docs/IGNORED_TESTS.md` | Liste der ignorierten Tests |
| `docs/UNWRAP_DEBT.md` | Liste der bestehenden `unwrap`-Stellen |
| `archive/README.md` | Liste archivierter Crates |
| `<crate>/README.md` und `<crate>/API.md` | Pro Kern- und Baustein-Crate |

**Hinweis zu bestehenden Dokumenten:** `DEPRECATED_DEBT.md`, `PLAN.md`, `capabilities.toml` und `mutants-guarantees.toml` enthalten veraltete Zielbilder (Spezifikation v4.1, Phasen, Ringe). Sie werden nicht gelöscht, sondern nach `archive/docs/` verschoben und durch die oben genannten Dokumente ersetzt. Das gilt auch für `DEPRECATED_DEBT.md`: Die darin beschriebenen Migrationsschritte sind in diesem Dokument erfasst.

---

## 8. Abschluss-Kriterien der Phase

Die Phase gilt als abgeschlossen, wenn:

- [ ] AP-0 bis AP-11 sind abgeschlossen, jeweils mit Eintrag im Migrationslog.
- [ ] Der Abhängigkeitsgraph der Kern-Crates entspricht Abschnitt 2.1 (geprüft mit `scripts/check_deps.sh`).
- [ ] Keine Referenz auf `erasure`, `shredding`, `audit`, `avv`, `license` im Quellcode der aktiven Crates (außer in Migrationskommentaren).
- [ ] `archive/` enthält alle Archiv-Crates mit README.
- [ ] Alle Pflichtdokumente aus Abschnitt 7 existieren.
- [ ] `ARCHITECTURE.md` enthält den aktuellen Abhängigkeitsgraph.
- [ ] Der Entwickler hat die Entscheidungen zu SIMD und zur Entfernung der drei Crates bestätigt (ADR).

**Ausdrücklich nicht Teil der Abschlusskriterien:**
- Ein erfolgreicher `cargo build --workspace`. Dieser ist nach der Umstrukturierung nicht gefordert. Er wird in einer Folgephase wiederhergestellt. Jeder Agent muss aber im Migrationslog festhalten, welche Crates er geändert hat und welche Compiler-Fehler dadurch entstehen können.
- Eine 75-%-Abdeckung in allen Crates. Die Ziele aus 5.4 gelten für die Kern-Crates nach der Folgephase.

---

## 9. Verbotene Handlungen für Agenten

- Eigenständig neue Features, Löschlogik, Compliance-Funktionen oder Nachweis-Mechanismen einbauen.
- Crates löschen, die nicht in Abschnitt 2.4 als „Entfernen“ stehen.
- Änderungen an `archive/` außer dem Hinzufügen von Dokumentation.
- Historie umschreiben (`git rebase`, `git push --force`, `git commit --amend` auf bereits gepushten Commits).
- Kryptografie-Code „verbessern“. Er wird archiviert, nicht gepflegt.
- Ring-Tabellen oder Ring-Kommentare neu einführen.
- Abhängigkeiten auf neue externe Crates hinzufügen, ohne Eintrag in `docs/decisions/` und Begründung.
- Test-Schwellen durch Entfernen von Tests erfüllen.
- `#[ignore]` ohne Eintrag in `docs/IGNORED_TESTS.md` setzen.
- Die Anzahl der `unsafe`-Stellen erhöhen.

---

## 10. Übergabe an Folgeentwickler

Jeder Folgeentwickler muss in unter 30 Minuten:
1. Den Zweck des Repos aus `README.md` verstehen.
2. Den Kern in `ARCHITECTURE.md` finden.
3. Einen Baustein auswählen und aus `<crate>/README.md` die Abhängigkeiten erkennen.
4. Den Migrationslog lesen und wissen, was zuletzt geändert wurde und warum.

Wenn ein Crate diese Anforderung nicht erfüllt, ist es nicht produktionsreif und muss in der Folgephase nachgearbeitet werden.
