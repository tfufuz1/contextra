# Contextra – Gesamtspezifikation 3.1 (Baukasten-Fassung, Zielzustand v1.0)

```
Dokument:   Normative Gesamtspezifikation (Architektur, Verhalten, Module, Prüfverträge)
Fassung:    3.1 (Entwurf zur Freigabe); überarbeitet und erweitert 3.0 um die Baseline-, Metrik- und Scan-Befunde vom 28.10.2026
            (U-15 bis U-27), Projektziele mit Messkriterien (§2.5), Wartbarkeitsregeln (§4.9), Crate-Review (§5.5) und
            Schnittstellenvertrag samt Rechtsmatrix der Löschkette (§9.11, §9.12)
Stand:      28.10.2026
Belegbasis: **drei Commit-Stände** (siehe §1.2): (a) Repo-Klon HEAD 21a09ad (flach, 28.10.2026, statisch gelesen, E1);
            (b) Baseline-Lauf auf Commit 8f8fa7cd (Toolchain 1.89.0, E2, nur Summaries und Metrik-Datei liegen vor);
            (c) Skript-Scan auf Commit 52d3227 (statisch, Heuristik, E1);
            35 Workspace-Mitglieder (33 Crates unter `crates/`, `benchmarks/contextra-bench`, `xtask`)
Ablage:     docs/SPEC.md
Ersetzt:    SPEC 3.0 (Entwurf) sowie SPEC 2.4/2.5 (Repo-Stand `docs/SPEC.md`: Produktzuschnitt, Archivierungsentscheidung, Kern-Definition); der
            Löschpfad (alt §12) bleibt inhaltlich erhalten und wird in §9 als Baustein gefasst (3.1: mit Schnittstellenvertrag und Rechtsmatrix)
Zielgruppe: Menschen und LLM-Agenten, die Contextra bauen, prüfen oder in ein KMU-Produkt einbetten
```

> **Leitidee der Fassung 3.0.** Contextra ist **keine Komplettlösung** mehr, sondern eine **robuste, datenschutzkonforme und entkoppelte Rust-Bibliothek (Baukasten) als Fundament für lokale LLM-Lösungen im Mittelstand in Deutschland und Europa**. Jedes Modul MUSS **einzeln gebaut, auditiert, verifiziert und leistungsgemessen** werden können. Die gesetzlich begründete Löschung samt Löschkette ist ein **eigenständiger Baustein** dieses Baukastens, kein Gesamtversprechen des Systems.

> **Selbständigkeit.** Dieses Dokument ist die einzige Quelle der Wahrheit für Architektur, Modulverträge, Konstanten, Gates und Plan. Jedes `§x` meint einen Abschnitt dieses Dokuments. Fehlt ein Detail, gilt: **nicht raten**, Fail-Closed wählen (INV-FAILCLOSED-1) und den Punkt in §16 melden. Der Umsetzungsplan steht in `PLAN.md` und darf dieser Spezifikation nicht widersprechen.

> **Ergänzung 3.1 (28.10.2026).** Die Baseline zeigt: `cargo check -p <crate>` gelingt für alle 35 Mitglieder, aber `cargo check --workspace --all-targets`, `cargo clippy -D warnings` (33 von 35 rot) und die Testläufe (2 Compile-Fehler, 13 Timeouts im 45-s-Harness) sind **nicht grün**. Die Aussage „alle Crates kompilieren fehlerfrei“ gilt damit nur für den kleinsten Prüfumfang (§1.2a). Diese Fassung macht daraus Pflichten: **Baseline nach ADR-D11** (je Crate **und** Workspace, alle Targets), **eine Messquelle** (ADR-D12), **keine Doppeldefinitionen** (ADR-D13), **keine Test-Hooks in der Produktions-API** (ADR-D14), **Löschkette als entkoppelter Baustein mit Schnittstellenvertrag** (ADR-D15) und **Wartbarkeitsregeln als Gates** (ADR-D16, §4.9).

---

## 0. Schnellstart für Menschen und LLM-Agenten

### 0.1 Was Contextra ist

Contextra ist ein **Baukasten aus Rust-Crates** für lokale (On-Premise, ohne Cloud-Zwang) Wissens-, Such- und LLM-Anwendungen. Er besteht aus fünf **Baustein-Familien** (§5): Speicher und Suche, Löschung und Nachweis, Inferenz und Embedding, Agenten und Gedächtnis, Schutz und Schnittstellen. Ein Integrator wählt **nur die Crates, die er braucht** (z. B. nur Hybrid-Suche + LSM-Store, oder nur MCP-Server + Inferenz + Agenten-Pipeline), und kann jedes gewählte Crate **für sich** prüfen.

Typische Einsatzfälle (Hypothesen, §16): Speisekarten- und Reservierungsassistent in der Gastronomie, lokaler Produktberater im Online-Shop, DSGVO-konforme Vor-Ort-Verarbeitung in Kommunen, Jobcentern und Agenturen, interne Wissensdatenbanken von KMU.

### 0.2 Die sechzehn Regeln für jede Änderung

1. **Ring und Klasse zuerst.** Jede Kante zeigt auf gleichen oder niedrigeren Ring (§4.1) **und** ist in `may_depend_on` des Crates deklariert (§4.4).
2. **Jedes Crate ist ein Modul mit Vertrag.** Es hat ein Modul-Manifest, ein Evidence-Pack und ein isoliertes Prüfkommando (§7). Ein Crate ohne diese drei Dinge darf nicht höher als `experimental` stehen.
3. **Kein Panic in Produktion.** Kein `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `unreachable!`, ungeprüftes Indexieren, **auch nicht hinter `#[allow(clippy::panic)]`** (P7, §3.2).
4. **Zeit, Zufall, IDs nur über Ports** `Clock`, `Rng`, `IdGen` (P28). Ausnahme: Schlüssel- und Salt-Material aus dem System-RNG.
5. **Alles Persistente geht durch das WAL** (P24); Sicherheitszustand liegt nur in den Sidecar-Logs und im Keystore (§8.5).
6. **I/O nur über `Vfs`**; `fsync`-Fehler werden propagiert, nie mit `let _ =` verworfen.
7. **Unsafe nur in drei Inseln** (`simd`, `sys`, `wire`), jeder Block mit `// SAFETY:` (P2, §4.6).
8. **Mandant kommt aus dem `RequestContext`**, nie aus Literalen oder Nutzerparametern (INV-TENANT-1).
9. **Sicherheit ist kein Feature-Schalter.** Wer ein Sicherheitsmodul einbindet, bekommt es aktiv; kein `cfg(not(feature))`-Ersatz für Sicherheitscode (A1).
10. **Messen vor Behaupten.** Ein Restbestand-Zähler entsteht nur aus einem Scanner der besitzenden Schicht (INV-CLAIMS-1).
11. **Fail-Closed.** Unbekannter Zustand, unbekannte Formatversion, fehlender Sicherheitsport, gebrochene Kette: Ablehnung mit benanntem Grund.
12. **Kein Netz ohne Erlaubnis.** Kein Crate öffnet ausgehende Netzverbindungen, außer es steht in der Egress-Tabelle (§10.6) und ist per Konfiguration freigeschaltet; Standard ist **nur Loopback**.
13. **Eine Typfamilie je Konzept.** Löschtypen existieren nur in `crypto`/`ports`; Ports nur in `ports` (§6).
14. **Ein PR = ein Arbeitspaket = höchstens ein Crate-Schnitt**; nach jedem Schnitt Build, Modul-Prüfung, Zeilenzählung (REQ-CHG-2).
15. **Keine Test-Hooks, Mocks oder Fake-Beweise in der öffentlichen Produktions-API.** `*_for_test`, `Mock*`, `Fake*` und Panic-Auslöser liegen hinter `cfg(test)` oder `test-hooks`/`testkit` (§12.4, U-24).
16. **Kein blockierendes I/O in `async`-Code.** Dateizugriff über `Vfs` oder `spawn_blocking`; kein `std::fs`/`std::thread::sleep` in `async fn` (U-23).

### 0.3 Karte: Konzept → Crate

|Konzept|Crate|Ring|Familie|
|---|---|---|---|
|IDs, Fehlerklassen, `RequestContext`|`types`|0|Fundament|
|Alle Austausch-Traits (Ports)|`ports`|0|Fundament|
|Snapshots, SSI, Sequenzen|`mvcc`|0|Speicher/Suche|
|HNSW, SQ8, ACL-Bitset|`vector`|0|Speicher/Suche|
|BM25, Analyzer, Block-Max-WAND|`text`|0|Speicher/Suche|
|RRF-Fusion, Kalibrierung|`rank`|0|Speicher/Suche|
|WAL, LSM, SSTable, Kompaktion|`store`|1|Speicher/Suche|
|Checkpoints, Restore-Guard|`checkpoint`|1|Speicher/Suche|
|AEAD, KDF, Signaturen, Löschbericht, AuditChain|`crypto`|0|Löschung/Nachweis|
|Löschsaga, `ErasedSet`, Planung (Ziel-Crate)|`erasure` (neu)|3|Löschung/Nachweis|
|Planner, Collections, Chunking, Retention|`engine`|3|Speicher/Suche|
|Embedded-API|`db`|3|Speicher/Suche|
|Candle / ONNX / Ollama|`infer-candle` / `infer-onnx` / `infer-ollama`|2|Inferenz|
|KV-Cache, Latenzregelung|`kvcache` / `adapt`|1 / 0|Inferenz|
|Graph, Kognition, Router, Agent|`graph` / `cognition` / `router` / `agent`|0 / 3 / 3 / 3|Agenten/Gedächtnis|
|PII-Filter, Egress-Gateway|`privacy`|3|Schutz/Schnittstellen|
|MCP (stdio), Python, Lizenz, Compliance-Export|`mcp` / `py` / `license` / `audit-export` / `avv-generator`|4 / 4 / 4 / 0 / 0|Schutz/Schnittstellen|
|SIMD / OS-Primitive / Wire-Layout|`simd` / `sys` / `wire`|0|Unsafe-Inseln|

### 0.4 Vorrang- und Lückenregeln

1. Normativ sind Sätze mit **MUSS, DARF NICHT, SOLL, KANN**.
2. Bei Widerspruch gilt: speziellerer Abschnitt vor allgemeinerem; Invarianten (§3.4) und Vetos (§3.1) vor allem anderen.
3. Zahlen mit „Startwert“ sind kalibrierbar; Zahlen mit „fest“ nicht.
4. Was als **Annahme**, **Hypothese**, **nicht gemessen** oder **offen** gekennzeichnet ist, ist keine Zusage.
5. Berichte, Audits und Agentenaussagen sind **Hypothesen**, bis ein Gate sie belegt (INV-EVIDENCE-1). Das gilt ausdrücklich für `SYSTEMATIC_CRATE_AUDIT.md` (§1.3).

---

## 1. Geltung, Belegbasis, Konventionen

### 1.1 Geltung und Änderung gegenüber 2.4

Diese Fassung ändert den **Produktzuschnitt**, nicht die Sicherheitsverfassung. Übernommen bleiben: Vetos, Invarianten, Ringmodell, Durability-Regeln, Löschpfad (zehn Schichten, Klassen L/P/C*/C1–C3/M, Destroy-Protokoll D1–D6, Bericht V5), Gates und Evidenzstufen. Geändert werden:

|Thema|SPEC 2.4|SPEC 3.0|
|---|---|---|
|Produkt|ein Wissensspeicher mit gemessener Löschung|Baukasten aus entkoppelten Modulen; Löschkette ist ein Baustein|
|Graph, Kognition, Router, Agent, KV-Cache, ONNX, Ollama|archiviert (Tag `archiv/v0`), Nicht-Ziel|**aktive Module** der Familien Inferenz und Agenten, optional, je mit eigenem Vertrag (§10)|
|„Kern“|ein geschlossener Kern von 16 Crates|**Kern = Fundament + Speicher/Suche + Löschung/Nachweis**; Familien Inferenz, Agenten, Schnittstellen sind **Ränder mit Modulvertrag** (§4.2)|
|Prüfbarkeit|Gates über den Gesamt-Workspace|**Isolierte Modulprüfung** je Crate (§7)|
|Zielmarkt|Berufsgeheimnisträger (Hypothese)|Mittelstand, Kommunen, Agenturen in DE/EU (§2.1)|
|Entkopplung|`engine`, `db`, `mcp` hängen an Graph, Kognition, Router|Rand-Abhängigkeiten **nur über Ports und Features**; Soll-Graph in §4.5|

Die alte Entscheidung E-Archiv („Graph/Kognition/Router/Agent ins Archiv“) ist **aufgehoben** (ADR-D01, §15). Der Workspace führt diese Crates bereits wieder als `default-members`; diese Fassung gibt ihnen Verträge statt sie zu verstecken.

### 1.2 Belegbasis dieser Fassung

|Quelle|Stufe|
|---|---|
|Repo-Klon, HEAD 52d3227f0, 28.10.2026: Workspace-`Cargo.toml`, alle Crate-Manifeste, `capabilities.toml`, Quelltexte (Stichproben und maschinelle Zählung); Schlüsselbefunde U-02, U-05, U-06, U-07, U-08, U-09, U-11 am neuen HEAD **nachgemessen** (Abweichungen in §1.3 vermerkt)|gelesen / statisch gemessen (E1)|
|Statische Kennzahlen (§5.2): eigener Zähler, nicht ausgerichtet am Gate `loc-budget`; Zeilen = nicht leere Nicht-Kommentarzeilen, Testcode (`tests/`, `#[cfg(test)]`, `*_tests.rs`) getrennt|E1, nicht vergleichbar mit Altwerten|
|Abhängigkeitsgraph aus den Manifesten (Regex-Auswertung, `[dependencies]`/`[dev-dependencies]`)|E1|
|`SYSTEMATIC_CRATE_AUDIT.md` (Fassung 2.3, 28.10.2026)|Hypothese (E0), in §1.3 gegen Code geprüft|
|Baseline-Lauf 28.10.2026 (Commit 8f8fa7cd, Toolchain `cargo 1.89.0` / `rustc 1.89.0`): `BASELINE.md`, `per_crate_summary.json`, vier `*.summary`-Dateien, `archived_crate_dependencies.txt`, `env.txt`; **rohe Logs liegen nicht vor**|E2 (teilweise: Exit-Codes und Dauer; keine Logs, keine zweite Runner-Wiederholung)|
|Metrik-Datei `metrics-2026-10-28.json` (je Mitglied: Ring, Reife, Zeilen, harte/optionale/Dev-Kanten, transitive Last, Panic-/Allow-/Unsafe-/Netz-/Zeitzähler; Messwerkzeug und Methode nicht dokumentiert)|E1 (Methode unbekannt)|
|Skript-Scan `CONTEXTRA_SCAN_ERGEBNIS.md` (`contextra_scan.py` v2, Commit 52d3227, statisch, prüft gegen **ältere** Spezifikationsnummern)|E1, Heuristik (Prüfaufträge, keine Beweise)|

**Nicht geprüft in dieser Fassung:** Miri, Fuzz, Benchmarks, CI-Stand, Wiederholung der Baseline auf einem zweiten Runner (damit **kein E3**), vollständige Testläufe der 13 Crates mit Timeout. Der im Repo liegende Lauf `evidence/baseline-a547a42/` stammt von einem **älteren Commit** und belegt weder 8f8fa7cd noch den heutigen HEAD. **Der Commit 8f8fa7cd ist im flachen Klon nicht auflösbar** (`git cat-file` schlägt fehl); Befunde aus dem Baseline-Lauf wurden deshalb, soweit möglich, **statisch gegen den Klon-HEAD 21a09ad** geprüft und Abweichungen in §1.2a und §1.3 vermerkt. Jede Zahl trägt ihre Stufe; Zahlen aus drei Commit-Ständen werden **nicht** vermischt, bis WP 0.1 eine Baseline auf **einem** Commit erzeugt.

### 1.2a Baseline-Ergebnis vom 28.10.2026 (E2, teilweise)

|Lauf|Ergebnis|Aussage|
|---|---|---|
|`cargo check -p <crate> --locked` je Mitglied (35)|**35 von 35 Exit 0**, auch `xtask`|belegt nur Kompilierbarkeit der Bibliotheks-/Binär-Targets, **nicht** der Test-, Bench- und Beispiel-Targets|
|Standalone-Builds (`contextra --no-default-features --features embedded`; `contextra-mcp`)|beide Exit 0 (65,5 s; 0,47 s)|0,47 s deutet auf einen Cache-Treffer, kein Kaltbau|
|`cargo check --workspace --all-targets --locked`|**Exit 101** (42,5 s laut Summary, 45,4 s laut `BASELINE.md`)|mindestens ein Test-Target kompiliert nicht (U-15)|
|`cargo clippy --workspace --all-targets --locked -- -D warnings`|**Exit 101**; je Crate **2 grün** (`wire`, `sys`), **33 rot**|Ursache nicht belegt (U-16)|
|`cargo test -p <crate> --no-fail-fast` je Mitglied, Limit 45 s|20 grün (`py` ohne ausgeführte Tests), **2 rot** (`mvcc`, `crypto`: Test-Compilation), **13 Timeout**|Timeout ist **weder grün noch rot**, sondern unbekannt (U-17)|
|`cargo test --workspace --locked --no-fail-fast`|Exit 101, 374.955 ms (≈ 6,2 min)|Log liegt nicht vor; Ergebnis je Test unbekannt|
|Testzähler (nur erfolgreich ausgeführte Suites)|1.259 bestanden, 0 fehlgeschlagen, 2 ignoriert|**keine** Aussage über die 15 nicht ausgeführten Crates|
|`cargo tree -p contextra` (normal, features)|Exit 0|Ausgabe liegt nicht vor; Metrik-Datei liefert die transitive Last (19)|

**Grenzen dieser Baseline (verbindlich zu lesen).**

1. **Drei Commit-Stände** (8f8fa7cd Baseline, 52d3227 Scan, 21a09ad Klon). Zahlen sind nur innerhalb eines Standes vergleichbar.
2. **Die Ursachenaussagen in `BASELINE.md` sind Hypothesen (E0/E1), keine Messung.** Beispiele: Der Klon enthält `LayerCleanupProof` **nirgends als Definition**, und `deletion_proof_integration.rs` sowie `audit_chain_tests.rs` importieren den Typ aus `contextra_crypto`, **nicht** aus `contextra_engine`; `CustodyLevel` ist in `crypto/src/lib.rs` exportiert, die Baseline-Meldung „undeclared type CustodyLevel“ ist am Klon nicht nachvollziehbar; `MetricEvent`/`TestMetricsSink` liegen in `testkit/src/mocks.rs`, und `mvcc/tests/ssi_metrics_observability.rs` importiert sie dort (nicht aus `ports::metrics`). Entweder ist die Baseline auf einem anderen Stand entstanden oder die Meldungen wurden verkürzt; ohne Logs nicht entscheidbar.
3. **Die Clippy-Dauer ist bei 31 Crates nahezu gleich (≈ 2,2 s)**; das spricht für einen frühen, gemeinsamen Abbruch, nicht für 33 unabhängige Lint-Haufen. „Unkonfigurierte Warnings“ ist unbelegt.
4. **Die Kantenliste in `BASELINE.md` §7** stammt aus `rg` über `Cargo.toml` und **unterscheidet nicht** zwischen `[dependencies]` und `[dev-dependencies]`. Laut Metrik-Datei sind `store → graph`, `engine → core`, `rank → adapt`, `infer-ollama → infer-onnx` und `router → graph` **Dev-Kanten**, keine harten Kanten. Für die Entkopplung gilt die Metrik-Datei (sie bestätigt alle Ist-Werte aus §4.5).
5. **Zeitangaben weichen zwischen `*.summary` und `BASELINE.md` ab** (Check 42.476 vs. 45.368 ms, Clippy 3.806 vs. 6.148 ms); Zeiten sind kein Beleg, Exit-Codes sind es.

### 1.3 Abweichungen zwischen Audit-Bericht und Code (Befunde U-01 bis U-14)

|ID|Aussage im Audit-Bericht|Befund am Code (E1)|Folge|
|---|---|---|---|
|U-01|35 Crates „Stable / Produktionsreif“, Mehrzahl 9–10/10|`capabilities.toml` führt `crypto`, `db`, `engine`, `mcp`, `store`, `text`, `vector` als **`beta`**, `adapt`, `audit-export`, `avv-generator`, `infer-onnx`, `license` als **`experimental`**|Audit-Noten sind **ungültig**; maßgeblich ist `capabilities.toml` und §5.3|
|U-02|`text`: „Panic-freie Regex-Initialisierung (`LazyLock<Option<Regex>>`)“|`tokenizer.rs:30` enthält `panic!` hinter `#[allow(clippy::panic)]`; Initialisierung über `OnceLock`|Verstoß gegen Regel 3; WP 1.1|
|U-03|`vector`: „`VectorScanner` für Audits“|Der Typ `VectorScanner` existiert nicht; vorhanden ist `HnswScanner` (`vector/src/hnsw/scanner.rs`). Gemessen werden nur HNSW-Slots; Snapshot-Datei und Quantisierungscodes sind nicht belegt|Schicht S2 teilweise gemessen; WP 5.2|
|U-04|Löschpfad (alt §12) als umgesetzt vorausgesetzt|**Vorhanden (E1):** `LayerScanner`-/`Purger`-Port; `MemtableScanner`, `SstableScanner`, `WalScanner` (`store`); `HnswScanner` (`vector`); `TextScanner` (`text`); `DeletionProof` mit `SignatureVersion` bis **V5** und Feldern für Annahmen, Reste, Ausschlüsse (`crypto`, 49 Treffer); `AuditChain`, `RevocationLog`, `KmsProvider`. Die in der Vorfassung gemeldeten Stubs (`no_crypto_stubs`, `default_test_deletion_proof_key`) sind **nicht mehr vorhanden**. **Keine Treffer** für `ErasedSet`, `EffaceableKeySlot`, `DurableLog`, `RecipientLedger`, `CacheScanner`, `CheckpointScanner`, `KeyScanner`, Blob-/Lineage-Scanner, Löschsaga. `WitnessLayer` kennt noch die alten sieben Varianten plus `Custom`; der Doku-Kommentar in `deletion_proof.rs` nennt V3 als „aktuell“|Löschkette **teilweise** umgesetzt: Messung und Bericht ja, Schlüsselhierarchie, Sperre, Saga und fünf von zehn Schichten nein; Reife `experimental` (§9.10)|
|U-05|`contextra-core` „9/10, Kompatibilitätsfassade“|61 Vorkommen von `contextra_core` in 32 Dateien außerhalb des Crates (zeilenbasiert gezählt; die Vorfassung nannte 41); hängt von `mvcc` und `wire`|Fassade ist nicht trivial; WP 2.8|
|U-06|„Entkoppelt“, „vollständig entkoppelt“ (`agent`, `graph`, `cognition`)|`engine` → `graph` (hart, `collection/relate.rs`); `db` → `cognition`, `adapt` (hart); `mcp` → `router`, `privacy`, `crypto`, `rank` (hart); `cognition` → `engine`; Dev-Zyklus `graph` ↔ `store`; `ports` enthält `graph.rs`, `graph_index.rs`, `kv.rs`, `attention.rs`|Echte Entkopplung fehlt (§4.5); Phase 2|
|U-07|`mcp`: „kein HTTP/Axum“|Bestätigt für `mcp`. **Aber:** `infer-onnx` (Feature `onnx`) lädt Modell und Tokenizer per `reqwest` von `huggingface.co` herunter (`src/lib.rs`, ca. Z. 90–135); `infer-ollama` nutzt `reqwest` gegen konfigurierbare URL (Beispiele `http://localhost:11434`)|Verstoß gegen VETO-NET1/INV-NET-1: Download nur nach ausdrücklicher Freigabe, sonst Offline-Modell mit Prüfsumme; Ollama nur Loopback per Default; WP 2.7|
|U-08|„Drei Unsafe-Inseln, nur dort `unsafe`“|Bestätigt: `unsafe` nur in `simd` (310 Zeilen), `sys` (43), `wire` (45, überwiegend generiert). `xtask` zählt nur Muster-Strings. `wire` hat `deny(unsafe_code)` statt `forbid` und generierten Code mit `.unwrap()`|Inseln korrekt, aber `wire`-Generat braucht Panic-Prüfung; WP 1.4|
|U-09|Tokio, Features, Zero-Overhead|Workspace-`tokio` mit `features = ["full"]`; Fassade hat `default = ["embedded"]` (gut), aber `candle`, `compliance` als Features|`tokio full` entfernen; WP 2.9|
|U-10|„Alle 35 Crates kompilieren fehlerfrei“|Nicht belegt. `default-members` enthält 26 von 35 Mitgliedern; **nicht** darin: `infer-candle`, `sandbox`, `py`, `privacy`, `license`, `audit-export`, `avv-generator`, `bench`, `xtask`|Baseline nötig (WP 0.1); CI muss alle Mitglieder je einzeln bauen|
|U-11|`crypto`: „AEAD, Löschbericht, WAL-Integrität“ (Fachlichkeit Löschung)|`crypto/src` enthält zusätzlich `kv_cipher.rs`, `kv_segment/`, `kv_shredding.rs` und die Features `kv-encryption`, `cloud-egress-guard`; KV-Cache-Typen (`KvSegment`) werden in 17 `crypto`-, 18 `kvcache`-, 11 `infer-candle`-Dateien genutzt|Fachfremde KV-Cache-Logik im Löschkern; Verlagerung nach `kvcache` (hinter Port), `crypto` bleibt I/O- und domänenfrei; WP 2.5|
|U-12|`xtask` „10/10, Quality Gates“|`xtask` hat 32.572 Code-Zeilen und ist damit **mehr als doppelt so groß wie `store`** (14.288); mindestens ein Teil der Prüfungen ist regex-basiert (z. B. `check_duplicate_core_primitives.rs`), daneben stehen Agenten-/Prozess-Prüfungen (`check_agents_*`, `jules_*`), die für die Bibliotheksqualität nicht zentral sind|Wartungsrisiko; Gate-Inventar, Meta-Tests und Rückbau nach Wirksamkeit; WP 3.6|
|U-13|Modul-„Isolation“ nur für Normal-Abhängigkeiten gedacht|Dev-Kanten brechen die isolierte Testbarkeit (M2): `router` → `db`, `graph`, `text`, `vector`; `cognition` → `infer-candle`; `infer-ollama` → `infer-onnx`; `rank` → `adapt`; `checkpoint` → `store`; `infer-candle`, `privacy` → `crypto`; `agent` → `text`, `vector`; `store`/`crypto` haben Selbst-Dev-Kanten (Feature-Aktivierung, kein echter Zyklus). Echter Zyklus nur `graph` ↔ `store`|Dev-Kanten gelten für `module-isolation`; WP 2.10, 2.11|
|U-14|Ein Baseline-Lauf liegt im Repo|`evidence/baseline-a547a42/` dokumentiert einen **roten** Build eines älteren Commits (`contextra-engine`: `contextra_graph` unaufgelöst); am heutigen HEAD hängt `engine` hart an `graph`, der Lauf ist damit **nicht übertragbar**|Als historisch kennzeichnen, neue Baseline; WP 0.1|
|U-15|Baseline (BASELINE.md): „Workspace `--all-targets` schlägt fehl“|Drei Test-Targets kompilieren nicht bzw. sind gegen den Klon-Stand nicht erklärbar: `db/tests/deletion_proof_integration.rs` und `crypto/tests/audit_chain_tests.rs` importieren `LayerCleanupProof`, den der Klon **nirgends definiert**; `mvcc/tests/ssi_metrics_observability.rs` (Baseline: Import aus `ports::metrics`, Klon: Import aus `testkit`, dort definiert). Betroffen ist ausgerechnet der **Löschbericht-Test** (G2)|Kern-Garantie ohne lauffähige Integrationstests; Löschkette bleibt `experimental` bis WP 0.5|
|U-16|„Clippy scheitert an unkonfigurierten Warnings“|33 von 35 Crates rot unter `-D warnings`; nahezu gleiche Dauer (≈ 2,2 s) spricht für gemeinsame Ursache (z. B. ein früh geprüftes Workspace-Lint oder eine gemeinsame Abhängigkeit); Ursache **nicht belegt**|Zielkriterium „Clippy grün“ (§2.5) verfehlt; WP 0.6 (Ursache zuerst, dann Behebung)|
|U-17|Testläufe mit Limit 45 s je Crate|13 Crates im Timeout (`store`, `graph`, `vector`, `db`, `cognition`, `engine`, `text`, `infer-candle`, `mcp`, `contextra`, `agent`, `bench`, `xtask`), 2 Compile-Fehler (`mvcc`, `crypto`); Workspace-Lauf dauert ≈ 6,2 min; die Teststatistik (1.259 bestanden) deckt diese 15 Crates nicht ab|Timeout ≠ grün; Test-Laufzeitbudget je Crate nötig (W10); WP 0.7|
|U-18|Kennzahlen aus einer Quelle|**Drei widersprüchliche Messungen**: Produktions-Panics (Scan: `store` 2 expect + 1 panic!, `text` 6 expect + 1 panic!, `db` 0; Metrik: `db` 8 expect + 2 panic, `store` 2 + 4, `text` 6 + 1, `vector` 1 unwrap, `wire` 16 unwrap im Generat; `ratchet.toml`: 1.175 / 502 / 242 Altwerte); Codezeilen (`types` 4.236 vs. 2.643, `store` 14.288 vs. 13.521); Zeitaufrufe (`store` 7 vs. 14)|Keine Zahl ist verbindlich, bis WP 0.4 eine AST-Messung liefert (ADR-D12)|
|U-19|§4.4-Kantenregeln als erfüllt behandelt|Metrik-Datei (harte Kanten gegen §4.4 gerechnet, Details §5.5): `db` 10 von 13 direkten Kanten außerhalb Soll; `mcp` 5, `py` 6, `cognition` 3, `agent` 2, `engine` 2 (`graph`, `sys`), `checkpoint` (`core`), `vector` (`crypto`); Fassade `contextra` hängt **hart** an `license` (`[E]`, verletzt P30) und `core`; `router` hängt hart an `privacy`, `wire` (Soll: optional); optional: `infer-onnx` → `infer-candle`, `mcp` → `infer-candle`, `engine` → `sandbox`|Soll-Graph wird erst nach WP 2.2–2.8, 2.14 erreicht|
|U-20|A2 „Eine Typfamilie je Konzept“|Scan: **35 Typnamen** in mehreren Dateien/Crates, darunter `DeletionProof` (`crypto`, `graph/cascade.rs`), **`TenantId`** (`types`, `privacy/processing_registry.rs`: VETO-F10-relevant), `KvSegment`, `EvictionWorker`, `SpillHandler`, `TenantIsolatedKvStore`, `EncryptedSegmentPayload` (`crypto` und `kvcache`: bestätigt U-11), `AdaptiveDecayController`/`DecayControllerConfig`/`DecaySignalInputs` (`adapt`, `engine`), `HyperEdgeId`/`RoleId`/`RoleBinding`/`GraphMutationError` (`graph`, `ports`), `EgressGuard` (`mcp`, `privacy`), `AvvContext` (`avv-generator`, `privacy`), `DriftDetector` (`adapt`, `rank`), `ConformalCalibrator` (`rank`, `router`), `DeletionProofKeyPair` doppelt in `crypto`|Wartungs- und Sicherheitsrisiko (zwei Wahrheiten); Gate `duplicate-types`, WP 2.13|
|U-21|Regel 4 „Zeit nur über `Clock`“|Scan: **44 direkte Zeitaufrufe** in Produktion (`mvcc` 14 in **Ring 0**, `engine` 7, `store` 7, `kvcache` 3, `ports` 3, `router` 3, `checkpoint` 2, `sandbox` 2, `adapt`/`db`/`privacy` je 1); `clippy.toml` ohne `disallowed_methods`/`disallowed_types`. Ob die Stellen in `determinism-allow.toml` gedeckt sind, ist je Stelle zu prüfen|Determinismus (G6, T-DET-1) nicht durch Compiler erzwungen; WP 2.12|
|U-22|Features „tot und doppelt löschen“, A1 „Sicherheit ist kein Feature-Schalter“|Scan: `store` nutzt `cfg(feature = "encryption-at-rest")` und `cfg(feature = "wal-integrity")` **ohne Definition im Manifest**: der dahinterliegende Code wird **nie kompiliert**; 21 definierte, im Code nicht genutzte Features (u. a. `compliance` der Fassade, `cloud-egress-guard` in `crypto`, mehrere in `db`, `adapt`, `graph`, `engine`; reine Weiterleitungen im Manifest sind als Fehlalarm zu prüfen)|Sicherheitsrelevant; WP 1.5 vor allen Architekturarbeiten|
|U-23|Regel 6 „I/O nur über `Vfs`“, Async-Disziplin|Scan: 15 Stellen mit `std::fs`/`std::thread::sleep` in `async fn` (`store/wal`, `store/sstable/reader.rs`, `vector/diskann/build.rs`, `vector/hnsw/core_rebuild.rs`, `infer-onnx`, `mcp`-Binary)|Blockiert den Executor; verletzt Regel 16 und teilweise Regel 6; WP 1.7|
|U-24|§12.4 Test-Hygiene|`py` exportiert `_trigger_panic_for_test` dreifach öffentlich; `store` hat `pub async fn simulate_wal_append_failure_for_test`; `mcp/config.rs` enthält `MockEmbedder`/`MockLlmGenerator` **ohne `cfg`** im Produktionscode; `LoomFile::flush/sync_all` und `LoomReadDir::next_entry` sind Konstanten-Stubs (loom-Hilfen, Feature-Zugehörigkeit prüfen); Fuzz-Target nutzt `mint_witness_for_tests`|Regel 15; WP 1.6|
|U-25|EP3 „Wired-or-it-doesn't-exist“|Scan: Port-Traits **UNWIRED** (`TextGenerator`, `Snapshot`, `KmsProvider` in `crypto/wal_crypto.rs`), **NULL_ONLY** (`MetricsSink`, `ReplayProgressSink`, `EntityRecognizer`), **TEST_ONLY** (`KvBridgeStorage`, `PluginManifest`, `SemanticEmbedder`, `AgentTool`); 89 öffentliche Items ohne Verwendung (`unwired-pub`). Heuristik, je Fund zu prüfen; Sicherheitsports nur mit Null-Implementierung machen das Modul laut §7.5 `experimental`|`KmsProvider` ist Grundlage der Löschkette (§9.11); WP 3.4, 5.3|
|U-26|§4.6 „unsafe nur in drei Inseln“|Bestätigt für Produktion. **Ausnahmen in Testcode:** `graph/tests/ppr_alloc_test.rs`, `text/tests/alloc_profiler.rs` (Allocator-Profiler mit `unsafe`); `wire`-Generat: 43 `unsafe`-Stellen ohne `// SAFETY:`; die Metrik zählt in `xtask` 39 `unsafe` (vermutlich Muster-Strings der Prüfungen, nicht Blöcke; AST-Messung klärt)|Regel 7 ist für Testcode zu präzisieren (§17, F-01); WP 1.4, 0.4|
|U-27|M5/M8, Dokumentation|`capabilities.toml`: `may_depend_on` unvollständig (Scan: 68 Warnungen, u. a. fehlende `types`/`ports`-Kanten) und enthält noch `contextra-core`; 5 Produktionsdateien > 800 Zeilen (`store/sstable/reader.rs` 948, `crypto/deletion_proof.rs` 910, `graph/ppr.rs` 887, `vector/hnsw/core_rebuild.rs` 850, `store/wal/io.rs` 835); `ARCHITECTURE.md` trägt einen veralteten Commit-Stempel; Deprecation `TenantId::new` wird noch aufgerufen (`db/examples/verify_deletion_proof_external.rs:83`)|WP 0.2, 2.1, 3.8|

Zählung (Nachmessung am HEAD 52d3227f0): `unsafe` nur in `simd` (310 Zeilen), `sys` (43), `wire` (45); `contextra_core`-Verwendungen und Allow-Zahlen sind **zeilenbasierte Handzählungen** (Allows außerhalb von `tests/`: 136 bis 158 je nach Muster) und werden durch den AST-Messbefehl aus WP 0.4 ersetzt. Die in der Vorfassung gemeldete `RING-LAYER-EXCEPTION` in `infer-ollama` ist am HEAD **nicht mehr auffindbar**. 33 Crates unter `crates/`, dazu `contextra-bench` und `xtask` = 35 Mitglieder; der Audit-Bericht zählt `xtask` und `bench` als „Crates“ mit.

### 1.4 Konventionen

- **Klassen:** **`[K]` Kern**, **`[M]` Modul** (Rand mit Modulvertrag, optional), **`[E]` experimentell** (nicht normativ).
- **ID-Schemata:** `INV-*` Invariante, `P*`/`EP*`/`A*` Prinzipien, `VETO-*`, `REQ-*` Anforderung, `U-*` Ist-Befund, `ADR-D*` Entscheidung dieser Fassung, `O-*` offener Punkt, `T-*` Pflichttest, `AC-*` Abnahmekriterium, `WP x.y` Arbeitspaket (PLAN.md), `G*` Garantie, `M*` Modulprinzip. IDs sind stabil.
- **Evidenzstufen:** **E0** Behauptung · **E1** am Code gelesen · **E2** Gate-/Skriptlauf auf bekanntem Commit, Protokoll archiviert · **E3** Test-/Chaoslauf reproduzierbar auf unabhängigem Runner · **E4** E3 plus unabhängiges Orakel, Mutation-Wirksamkeit und externer Review.
- **Reifegrade (`maturity`):** `experimental` (E0–E1), `beta` (E2), `stable` (E3 + alle Pflicht-Gates des Moduls), `certified-ready` (E4, nur wenn ein externer Review vorliegt). Der Begriff „produktionsreif“ darf nur für `stable` oder höher verwendet werden (INV-CLAIMS-1).

---

## 2. Produkt

### 2.1 Zielbild und Zielgruppe

**Contextra ist ein Baukasten für lokale LLM-Anwendungen im europäischen Mittelstand**, in dem Datenschutz **Bauprinzip** ist: Daten bleiben im Haus, Verarbeitung ist mandantengetrennt, Löschung ist technisch begründet und prüfbar, ausgehender Verkehr ist standardmäßig ausgeschlossen.

|Zielgruppe|Anforderung|Bausteine|
|---|---|---|
|Integratoren / Agenturen (Hauptnutzer der Bibliothek)|Module einzeln wählen, einzeln prüfen, einzeln dokumentieren|alle, über Profile (§11)|
|KMU (Gastronomie, Handel, Handwerk)|kleine Hardware, einfache Bedienung, kein Cloud-Zwang|Familie 1 + 3 (Profil „Local-RAG“)|
|Öffentliche Stellen (Kommunen, Jobcenter)|Vor-Ort-Verarbeitung, nachvollziehbare Löschung, AVV-Unterlagen|Familie 1 + 2 + 5 (Profil „Compliance“)|
|Entwickler von Agenten|deterministischer, budgetierter, wiederanlauffähiger Agentenlauf|Familie 4 (Profil „Agent“)|

**Designpunkt (Annahme, per Benchmark zu belegen):** ≤ 10⁷ Vektoren, ≤ 10² Mandanten je Knoten, Single-Node, ein Prozess hält ein Datenverzeichnis. Kein Sharding, keine Replikation, kein Netz-Listener, keine Telemetrie nach außen.

### 2.2 Modulprinzipien M1–M8 (neu)

|ID|Prinzip|Prüfmechanismus|
|---|---|---|
|M1|**Isoliert baubar:** `cargo build -p <crate> --no-default-features` gelingt mit ausschließlich den **hart deklarierten** Abhängigkeiten|Gate `module-isolation`|
|M2|**Isoliert testbar:** Alle Tests des Crates laufen ohne andere Rand-Crates (auch ohne Dev-Abhängigkeiten auf sie); Fremdverhalten kommt über Ports und `testkit`|Gate `module-isolation`|
|M3|**Isoliert auditierbar:** Jedes Crate hat ein Evidence-Pack (§7.2): Vertrag, Invarianten, Unsafe-Inventar, Panic-Inventar, Abhängigkeitsbudget, Lizenzbericht|Gate `evidence-pack`|
|M4|**Isoliert messbar:** Jedes Crate mit Leistungsversprechen hat einen Benchmark mit Fixture und Budget (§7.4)|Gate `module-bench`|
|M5|**Schmaler Vertrag:** Öffentliche API ≤ Budget (Startwert 40 Elemente je Rand-Crate, 20 für Fassaden); jede Änderung am Vertrag ist SemVer-relevant|`cargo-semver-checks`, `api-surface`|
|M6|**Transitive Last begrenzt:** Jedes Crate hat ein Budget für die Zahl hart abhängiger Workspace-Crates (§4.5)|Gate `transitive-budget`|
|M7|**Ersetzbar:** Jede Austauschrolle (Embedder, Generator, Store, Scanner, KMS) ist ein Port mit Referenzimplementierung **und** Test-Double|Gate `port-ownership`|
|M8|**Ehrlich:** Reife, Nicht-Ziele und Annahmen stehen im Manifest; ein Crate behauptet nie mehr, als sein Evidence-Pack belegt|Gate `claims-consistency`|

### 2.3 Produktgarantien (Modul-bezogen)

Die Garantien G1–G7 der Fassung 2.4 bleiben bestehen, werden aber **dem Baustein zugeordnet, der sie trägt**. Sie gelten nur, wenn der Integrator das jeweilige Modul einbindet **und** die in §9 genannten Annahmen erfüllt sind.

|ID|Garantie|Trägerbaustein|Test|
|---|---|---|---|
|G1|Nach Klasse P finden Byte-Scans in allen Ebene-A-Strukturen keine Klartextreste des Scopes; nach Klasse M zusätzlich nicht in SSTables, WAL, Blob-Segmenten, Checkpoints|Löschkette (§9)|T-DEL-1|
|G2|Ein Löschbericht V5 weist nur die Klasse aus, deren Voraussetzungen gemessen sind; Signatur verifiziert mit falschem Schlüssel nicht|`crypto`, `erasure`|T-DEL-2/3|
|G3|Absturz an jeder Stelle der Saga führt zu Fortsetzung oder `Partial{outstanding}`, nie zu Bericht ohne Löschung und nie zu lebendem Schlüssel einer gelöschten Einheit|`erasure`, `store`|T-DEL-4/5|
|G4|Ab `Frozen` liefert kein Lesepfad Daten des Scopes; ein Restore stellt sie nicht wieder her|`erasure`, `checkpoint`|T-DEL-7/8/9|
|G5|Kein Mandant liest, findet oder löscht Daten eines anderen|`types`, `engine`, `vector`, `text`, `infer-*` (KV-Cache je Mandant)|T-TEN-1/2|
|G6|Nach hartem Abbruch ist der Zustand aus dem WAL rekonstruierbar|`store`, `mvcc`|T-REC-2|
|G7|Jede Löschzusage ist maschinell widerlegbar (Lösch-Mutationskatalog)|Löschkette|T-DEL-15|
|G8 (neu)|Kein Modul sendet Daten ins Netz ohne ausdrückliche Freigabe (Egress-Tabelle §10.6)|alle `[M]`-Module, `privacy`|T-EGR-1|
|G9 (neu)|Jedes Modul ist ohne die übrigen baubar und prüfbar (M1–M2)|alle|T-ISO-1|

**Was „Garantie“ bedeutet:** ein dreistufiger technischer Vertrag – (1) Typ- oder Gate-Beweis, (2) reproduzierbarer Test, (3) benannte Annahme. Keine Garantie ist ein Rechtsnachweis (§9.9).

### 2.4 Nicht-Ziele v1.0

Verteilter Betrieb, Replikation, Echtzeit-Audio, Windows für Server und MCP, ausgelieferte Schlüsselverwahrung K2/K3 (Port vorhanden), Löschbeweis als Rechtsnachweis, Dokumentformat-Parser im Kern, Telemetrie, eigene Modelltrainings, GPU-Pflicht. Wer sie benötigt, braucht einen ADR (§15).

### 2.5 Projektziele und Messkriterien (neu in 3.1)

Das Projektziel lautet: **eine robuste, datenschutzkonforme und entkoppelte Rust-Bibliothek als Fundament für lokale LLM-Lösungen im Mittelstand in Deutschland und Europa.** Jedes Wort ist hier ein prüfbares Kriterium, kein Anspruch:

|Ziel|Definition|Messgröße (Gate / Test)|Ist am 28.10.2026 (Quelle)|
|---|---|---|---|
|**Robust**|Kein Absturz durch Eingaben; Zustand nach hartem Abbruch rekonstruierbar; Bau und Prüfung reproduzierbar|Produktions-Panics = 0 (`panic-inventory`); `check`, `clippy -D warnings`, `test` je Crate **und** `--workspace --all-targets` grün (ADR-D11); Crash-Matrix; Evidence-Pack|check je Crate 35/35; clippy 2/35; test 20/35 grün, 2 rot, 13 Timeout; Workspace `--all-targets` rot (§1.2a)|
|**Datenschutzkonform**|Datenschutz ist Bauprinzip (Art. 25 DSGVO): Mandantentrennung, Netz aus, Datenminimierung, technisch begründete Löschung samt messbarem Nachweis, keine Telemetrie|T-TEN-1/2, T-EGR-1, T-DEL-*, `egress-inventory`, Rechtsmatrix (§9.12)|Löschtyp-Messung teilweise (5 von 10 Schichten), Löschintegrationstests nicht kompilierbar (U-15), `infer-onnx` mit Download (U-07), `TenantId`-Duplikat (U-20)|
|**Entkoppelt**|Jedes Crate baut, testet, auditiert und misst sich mit seinen deklarierten Abhängigkeiten allein; Kopplung nur über Ports und `types`|`module-isolation`, `transitive-budget`, `no-cycles`, `may-depend-on`; Profile bestehen `profile-audit`|Ring-Richtung eingehalten, aber 10 Crates über Budget und 8 Crates (plus Fassade) mit harten Kanten außerhalb §4.4 (§5.5); 1 echter Dev-Zyklus (`graph` ↔ `store`)|
|**Wartbar**|Änderungen bleiben lokal; Wahrheit steht an einer Stelle; Regeln sind Gates, nicht Konvention|Wartbarkeitsregeln W1–W11 (§4.9); `duplicate-types`; `file-size`; Ratchets|35 doppelt definierte Typnamen, 5 Dateien > 800 Zeilen, `xtask` 32.572 Zeilen (§1.3 U-20, U-27, U-12)|
|**Zukunftsfähig**|Formate, APIs und Algorithmen sind versioniert und austauschbar; Rechts- und Plattformänderungen erzeugen Arbeit an einer Stelle|SemVer je Crate, Format-Register, Algorithmus-IDs (W5), Deprecation-Policy (W4), gepinnte Toolchain (W7)|Format-Register teilweise (WAL, SSTable, Löschbericht V1–V5); Algorithmus-IDs nicht durchgängig belegt|
|**Ehrlich**|Berichte, README und Manifeste behaupten nie mehr als das Evidence-Pack|`claims-consistency`, INV-CLAIMS-1|Audit-Noten ungültig (U-01); Baseline-Ursachen als Hypothese gekennzeichnet (§1.2a)|

**Nicht-Aussagen:** „rechtssicher“, „zertifiziert“, „produktionsreif“ (ohne `stable`), „vollständig entkoppelt“ (ohne `transitive-budget`-Beleg).


---

## 3. Verfassung

### 3.1 Vetos (unverhandelbar)

- **VETO-F10 – Keine mandantenübergreifenden Datenflüsse.** Kein Cache, Index, Embedding, KV-Cache-Block, Statistik (z. B. IDF) oder Audit-Eintrag überschreitet Mandantengrenzen. Der Mandant stammt aus dem `RequestContext`, nie aus Nutzerparametern.
- **VETO-F02 – HNSW-Löschung nur per Tombstone + Slot-Nullung + Rebuild**, kein Rewiring der Nachbarn.
- **VETO-OP03 – Kein Echtzeit-Audio/STT** im Kern.
- **VETO-NET1 (neu) – Kein stiller Netzverkehr.** Ein Modul, das Daten ins Netz sendet, ist nur nach ausdrücklicher Konfiguration aktiv und steht in der Egress-Tabelle (§10.6).
- **Single-Node-Doktrin** – kein Sharding, kein P2P, keine verteilte Replikation.

### 3.2 Engineering-Prinzipien (Auszug, unverändert gültig)

EP1 Fail-closed · EP2 Messbare Testtiefe (jede `INV-*` hat einen Test, der ihre Verletzung erkennt) · EP3 _Wired-or-it-doesn't-exist_ (eine Fähigkeit existiert nur, wenn sie verdrahtet ist **und** ein Integrationstest sie über den echten Pfad beweist) · EP4 Benchmarks mit fremder Baseline · EP5 Atomare Zyklen · EP6 Jepsen-artiges Chaos-Testing mit `FaultVfs` · EP7 Deterministische Verdrahtung nur in der Fassade · EP8 IoC über Ports · EP9 Audits sind selbst Testobjekte · EP10 Absolute Verifizierbarkeit · EP11 Compiler vor Heuristik · EP12 Bauen-oder-kaufen belegt · EP13 Evidenz vor Behauptung · EP14 Tiefe vor Breite · EP15 Unabhängiges Orakel je Kerngarantie.

**Ergänzung EP16 (neu): Ein Lint-Allow ist eine Schuld mit Frist.** Jedes `#[allow(clippy::panic|unwrap_used|expect_used|indexing_slicing)]` in Produktionscode steht in `governance/allow-register.toml` mit Begründung und Frist; das Gate `allow-register` bricht bei einem nicht registrierten Allow. (Anlass: U-02.)

**Ergänzung EP17 (neu): Audit-Berichte dürfen keine Reifegrade vergeben.** Reife entsteht nur aus `capabilities.toml` und dem Evidence-Pack (§7); Gate `claims-consistency` vergleicht Berichte, README und Crate-Dokumentation mit dem Manifest.

### 3.3 Architekturprinzipien

P2 Unsafe nur in Inseln · P7 Zero-Panic · P24 WAL First · P28 Determinismus via Port-Injektion · P30 Kern hängt nie von Rand ab · A1 Sicherheit ist kein Feature-Schalter · A2 Eine Typfamilie je Konzept.

### 3.4 Invariantenregister (Auszug der tragenden Invarianten)

|ID|Invariante|Besitzer|
|---|---|---|
|INV-TENANT-1|Jeder Datenzugriff trägt die `TenantId` aus dem `RequestContext`|`types`, `engine`|
|INV-TENANT-IDX-1|Index je Mandant; ACL-Prüfung vor Distanzberechnung|`vector`, `text`|
|INV-FAILCLOSED-1|Unbekannter Zustand ⇒ Ablehnung mit Grund|alle|
|INV-WAL-1|Alle persistenten Änderungen laufen durch das WAL|`store`|
|INV-IDX-1|LSM-Tree ist Single Point of Commit|`engine`, `store`|
|INV-HNSW-SLOT-1|Slot-Nullung bei Löschung; keine Wiederverwendung bis Neuaufbau|`vector`|
|INV-CLASS-MEASURED-1|Ein Löschbericht weist nur die gemessene höchste Klasse aus|`crypto`, `erasure`|
|INV-NO-RESURRECTION-1|Kein lebender Schlüssel einer gelöschten Einheit|`erasure`, `store`|
|INV-SURFACE-1|Jede persistente Datei-/Strukturklasse hat genau eine Löschschicht oder einen dokumentierten Ausschluss|`store`, `erasure`|
|INV-DERIVED-1|Ableitungen (Embeddings, Zusammenfassungen, Graphkanten, KV-Blöcke) kaskadieren bei Löschung|`engine`, `graph`, `kvcache`|
|INV-CLAIMS-1|Aussagen nur aus Messung; „produktionsreif“ nur ab `stable`|alle|
|INV-EVIDENCE-1|Zusagen sind Ziele, bis ein Gate sie belegt|alle|
|INV-NET-1 (neu)|Kein ausgehender Netzverkehr außer Loopback ohne Freigabe|`infer-*`, `mcp`, `privacy`|
|INV-ISO-1 (neu)|Jedes Crate baut und testet mit ausschließlich seinen deklarierten Abhängigkeiten|alle|
|INV-KVTENANT-1 (neu)|KV-Cache-Blöcke sind mandanten- und modellidentitätsgebunden und werden bei Löschung/Mandantenende zeroisiert|`kvcache`, `infer-candle`|

---

## 4. Architektur und Entkopplung

### 4.1 Ringe und Klassen

`capabilities.toml` ist die **einzige** Quelle für Ring, Klasse, `maturity` und Ausnahmen (INV-DOC-SINGLE-SOURCE-1). Richtung: Ring 4 → 3 → 2 → 1 → 0.

```
Ring 4  Schnittstellen    contextra (Fassade)   [M] mcp   [M] py   [E] license
Ring 3  Orchestrierung    engine   db   erasure(neu)   [M] cognition   [M] router   [M] agent   [M] privacy
Ring 2  Isolation/Infer.  [M] infer-candle   [M] infer-onnx   [M] infer-ollama   [M] sandbox
Ring 1  Persistenz        store   checkpoint   [M] kvcache
Ring 0  Fundament         types   ports   mvcc   crypto   vector   text   rank
                          [M] graph   [M] adapt   [E] audit-export   [E] avv-generator
        Inseln            sys   simd   wire
Werkzeug                  testkit   bench   xtask
```

**Wirkung der Ringe:** Das Ring-Gate erlaubt jede Abwärtskante. Ringe allein verhindern Kopplung nicht (der Ist-Graph hat **keine** Ringverletzung, aber hohe transitive Last, §4.5). Maßgeblich sind `may_depend_on` (§4.4) und das Budget `transitive-budget`.

**Klassenregel:** Kern `[K]` hängt nie von `[M]`/`[E]` ab (P30). `[M]` darf von `[K]` und von anderen `[M]` nur abhängen, wenn die Kante in §4.4 steht.

### 4.2 Was „Kern“ und „Modul“ bedeuten

|Klasse|Bedeutung|Pflichten|
|---|---|---|
|`[K]` Kern|Fundament, Speicher/Suche, Löschung/Nachweis, Fassade|Zielreife `stable`; alle Pflicht-Gates; Evidence-Pack; Fuzz/Miri/Mutation nach §12|
|`[M]` Modul|Optionaler Baustein mit eigenem Vertrag (Inferenz, Agenten, Schnittstellen)|Zielreife mindestens `beta`; **eigenes Evidence-Pack und isoliertes Prüfkommando**; darf Kern-Garantien nie schwächen|
|`[E]` Experimentell|Nicht normativ|nur hinter Feature; Release-Default aus|
|Werkzeug|`testkit`, `bench`, `xtask`|nie in Release-Features; `testkit` nur als `dev-dependency`|

Der **Kern ist nicht mehr geschlossen** (alt §4.8): Ein Modul darf aufgenommen werden, wenn es Modulvertrag, Evidence-Pack und isolierte Prüfung erfüllt (ADR-D02). Die Reife eines Moduls wird nie höher geführt als die seiner Kern-Abhängigkeiten.

### 4.3 Modulvertrag (Manifest je Crate)

Jedes Crate MUSS in `capabilities.toml` (Pflichtfelder) und in `crates/<name>/MODULE.md` (Kurzvertrag, ≤ 120 Zeilen) folgendes führen:

|Feld|Inhalt|
|---|---|
|`ring`, `class`, `family`, `maturity`|siehe §4.1, §1.4|
|`may_depend_on`|erlaubte Normal-Abhängigkeiten (Workspace-Crates)|
|`transitive_budget`|maximale Zahl hart abhängiger Workspace-Crates (§4.5)|
|`provides` / `requires`|bereitgestellte und benötigte **Ports** (§6)|
|`guarantees`|IDs der Garantien/Invarianten, die das Crate trägt|
|`assumptions`|benannte Annahmen (z. B. A-FLASH, A-FS-ATOMIC-RENAME)|
|`egress`|`none` oder Verweis auf Egress-Tabelle (§10.6)|
|`bench`|Benchmark-IDs und Budgetdatei|
|`non_goals`|was das Crate ausdrücklich **nicht** leistet|
|`evidence`|Pfad des Evidence-Packs, letzte Evidenzstufe, Commit|

`MODULE.md` hat feste Abschnitte: _Zweck · Öffentliche API (Auswahl) · Ports · Invarianten · Fehlermodell · Annahmen · Nicht-Ziele · Prüfkommando · Benchmarks_.

### 4.4 Kantenregeln (`may_depend_on`, strikt) und Soll-Tabelle

|Crate|darf (normal) abhängen von|Anmerkung zur Entkopplung (Ist → Soll)|
|---|---|---|
|`types`|–|–|
|`ports`|`types`|graph-/kv-spezifische Ports (`graph.rs`, `graph_index.rs`, `kv.rs`, `attention.rs`) wandern **nach Nutzungsanalyse** in die besitzenden Crates (WP 2.4)|
|`sys`, `simd`, `wire`|`types` (nur falls nötig)|–|
|`mvcc`|`types`, `ports`|–|
|`crypto`|`types`, `ports`, `sys`|KV-Cache-Kryptografie (`kv_cipher`, `kv_segment`, `kv_shredding`) nach `kvcache` verlagern (U-11)|
|`vector`|`types`, `ports`, `mvcc`, `simd`, `sys`|`crypto` nur hinter Feature `experimental-diskann` (Ist: hart; nur Test-/DiskANN-Verwendung gefunden)|
|`text`|`types`, `ports`|–|
|`rank`|`types`, `ports`|–|
|`store`|`types`, `ports`, `mvcc`, `crypto`, `sys`|Dev-Kanten `graph`, `vector`, `sandbox` verlagern nach `testkit`/`tests/`|
|`checkpoint`|`types`, `ports`|`contextra-core` entfernen; Zugriff auf Store nur über Port `StorageEngine`|
|`kvcache`|`types`, `ports`, `crypto`|–|
|`engine`|`types`, `ports`, `mvcc`, `crypto`, `store`, `vector`, `text`, `rank`, `checkpoint`|**`graph` nur über Port `RelationIndex`** (Ist: hart in `collection/relate.rs`); `sandbox` optional|
|`db`|`types`, `ports`, `engine`|**kein** `cognition`, **kein** `adapt` (Ist: hart, Re-Exports und `execute_sleep_cycle`) – Verlagerung in `cognition`/Fassade|
|`erasure` (neu)|`types`, `ports`, `crypto` (nur Berichts- und Schlüsseltypen)|**nicht** `engine`, `db`, `store`; alle Messung und Reinigung über Ports (§9.11); Verdrahtung nur in der Fassade|
|`graph`|`types`, `ports`, `adapt`|Dev-Kante `store` entfernen (Zyklus `graph` ↔ `store`)|
|`adapt`|`types`, `ports`|–|
|`cognition`|`types`, `ports`, `graph` (optional), `adapt`|**nicht** `engine`/`store`/`vector` direkt: Zugriff über Ports `DocumentStore`, `VectorIndex`, `RelationIndex` (Ist: `engine`-Typen in `consolidation_executor.rs`)|
|`router`|`types`, `ports`, `adapt`|`privacy`, `wire` nur optional; Provider-Ports in `ports`|
|`agent`|`types`, `ports`, `checkpoint`, `graph` (optional), `router` (optional)|**nicht** `db`/`store`/`vector`/`text` direkt; Zugriff über Ports (Ist: transitive Last 20)|
|`infer-*`|`types`, `ports`, `rank` (nur ONNX/Ollama-Reranker), `crypto` (nur KV-Verschlüsselung)|Netzclient hinter Egress-Policy (§10.6)|
|`privacy`|`types`, `ports`|–|
|`mcp`|`types`, `ports`, `wire`, `contextra` (Fassade)|`router`, `privacy`, `crypto`, `rank` nur optional und über Fassaden-Re-Export (Ist: hart, 22 transitive)|
|`py`|`types`, `contextra` (Fassade)|Ist: 7 direkte Abhängigkeiten, transitive Last 20|
|`contextra`|alle `[K]`; `[M]`/`[E]` nur `optional = true`|`contextra-core` und `license` nicht Pflicht|

**Kantenregeln:**

1. **Fundament** (`types`, `ports`, Inseln) darf jedes Crate nutzen, soweit seine Zeile es nennt. Typen, die mehrere Kerne teilen, liegen in `ports`/`types`.
2. **Rand-Crates reden über Ports, nicht über Nachbarn.** Ein `[M]`-Crate hängt nicht von `engine`, `db` oder `store` ab; es erhält deren Fähigkeit als Port-Implementierung vom Composition Root.
3. **Nur die Fassade `contextra` instanziiert und verdrahtet** (EP7). `mcp` und `py` erreichen Kern und Module nur über sie.
4. **Löschtypen** existieren genau einmal in `crypto`/`ports`.
5. **Test-Doubles** liegen in `testkit` oder hinter `test-utils`/`testkit`-Features, die in keinem Release- oder Default-Set liegen.
6. **Aufwärts- und Quer-Dev-Kanten** sind nur als deklarierte Ausnahme in `capabilities.toml` erlaubt, mit Ablauf-WP; Ziel: keine außer `testkit`. Ist-Stand (U-13): `router` → `db`, `graph`, `text`, `vector`; `cognition` → `infer-candle`; `infer-ollama` → `infer-onnx`; `rank` → `adapt`; `checkpoint` → `store`; `infer-candle`, `privacy` → `crypto`; `agent` → `text`, `vector`; `store` → `graph`, `vector`, `sandbox`; `graph` → `store` (WP 2.10, 2.11). Selbst-Dev-Kanten (Feature-Aktivierung in Tests) sind keine Zyklen und werden im Evidence-Pack ausgewiesen.
7. **Zyklen** (auch über Dev-Kanten) sind verboten (Gate `no-cycles`; Ist: `graph` ↔ `store`).
8. **Port-Besitz:** Jeder Port mit Austauschrolle ist in `ports` definiert; domänenspezifische Ports (Graph, KV) liegen im besitzenden Crate und werden dort versioniert (WP 2.4).

### 4.5 Transitive Last: Ist und Soll (Startwerte)

`transitive_hart` = Zahl der Workspace-Crates, die ein Crate über **nicht optionale** Normal-Abhängigkeiten zieht. Ziel: ein Integrator, der nur `text` oder `vector` will, zieht ≤ 5 Workspace-Crates.

|Crate|Ist|Soll|Hauptursache im Ist|
|---|--:|--:|---|
|`mcp`|22|16|Fassade + `router`, `privacy`, `crypto`, `rank` hart|
|`agent`|20|10|`db` → `engine` → alles|
|`py`|20|12|7 direkte Abhängigkeiten inkl. `router`, `adapt`, `core`|
|`contextra`|19|14|`core`, `license`, `store`, `db` hart|
|`db`|17|12|`cognition`, `adapt`|
|`cognition`|16|6|`engine`, `store`, `vector`, `graph`|
|`engine`|15|11|`graph` hart|
|`vector`|6|5|`crypto`, `sys`|
|`store`, `checkpoint`|5|5 / 4|`crypto`; `core`|
|`router`|5|4|`privacy`, `wire`|
|`erasure` (neu)|–|4|`types`, `ports`, `crypto` (+ `sys` transitiv)|
|alle übrigen|≤ 3|≤ 3|–|

Das Gate `transitive-budget` bricht, wenn ein Crate sein Budget überschreitet; Budgets dürfen nur sinken (Ratchet).

**Beleg (3.1):** Die Metrik-Datei vom 28.10.2026 bestätigt alle Ist-Werte der Tabelle (`mcp` 22, `agent` 20, `py` 20, `contextra` 19, `db` 17, `cognition` 16, `engine` 15, `vector` 6, `store`/`checkpoint`/`router` 5). Über Budget liegen damit `mcp`, `agent`, `py`, `contextra`, `db`, `cognition`, `engine`, `vector`, `checkpoint`, `router` (10 Crates). Die **direkten** Kanten außerhalb §4.4 stehen in §5.5.

### 4.6 Unsafe-Inseln (genau drei)

`simd`, `sys`, `wire`. Jede Insel exponiert eine sichere API; jeder `unsafe`-Block trägt `// SAFETY:`; Miri- und Differential-Tests gegen eine sichere Referenz (SIMD gegen Skalar); kein anderes Crate enthält `unsafe` (`#![forbid(unsafe_code)]`; Ist: bestätigt, §1.3 U-08). Generierter Code (FlatBuffers) wird getrennt ausgewiesen und auf Panic-Muster geprüft; `wire` soll `forbid` für handgeschriebenen Code und ein gekapseltes `allow` nur für das Generat nutzen. `xtask` und `bench` dürfen kein `unsafe` einführen.

### 4.7 Features und Standardbau

- `default = []` in allen Crates, Ausnahme Fassade `contextra` mit `default = ["embedded"]` (Speicher + Hybridsuche, kein Inferenz-Backend).
- Rand-Module nur über benannte Fassaden-Features: `candle`, `onnx`, `ollama`, `agents`, `graph`, `cognition`, `router`, `privacy`, `mcp`, `sandbox`, `compliance`, `license`.
- **Tote und doppelte Features** werden gelöscht; `unexpected_cfgs = "deny"` (Ist: `store` setzt `allow(unexpected_cfgs)`).
- `tokio` ohne `features = ["full"]`; je Crate nur benötigte Features (U-09).
- Das Gate `feature-matrix` baut jede zulässige Feature-Kombination der Fassade und jedes Moduls mit `--no-default-features`.

### 4.8 Composition Root und Profile

Die Fassade `contextra` ist der **einzige** Composition Root. Sie bietet `ContextraBuilder` und benannte **Profile** (§11), die Ports mit Implementierungen verdrahten. Ein Integrator kann den Composition Root **ersetzen** und Module direkt verdrahten; dafür muss jeder Port eine dokumentierte Referenzverdrahtung im `MODULE.md` haben.

### 4.9 Wartbarkeits- und Zukunftsregeln W1–W11 (neu in 3.1)

Wartbarkeit entsteht durch **wenige, maschinell erzwungene Regeln mit Ratchet** (Werte dürfen nur besser werden), nicht durch Sammelumbau. Jede Regel hat ein Gate; Startwerte stammen aus der Messung vom 28.10.2026.

|ID|Regel|Gate / Nachweis|Startwert (Ist)|
|---|---|---|---|
|W1|**Dateigröße:** Produktionsdatei ≤ 800 Zeilen (generierter Code ausgenommen und ausgewiesen)|`file-size` (Ratchet)|5 Dateien darüber (U-27)|
|W2|**Schmale öffentliche API:** Budget je Crate (M5); öffentliche Items ohne Verwendung außerhalb des Crates werden `pub(crate)` oder entfernt, außer sie stehen im `MODULE.md`-Vertrag|`public-api`, `unwired-pub` (Warnung bis Phase 6, danach rot)|89 unverdrahtete `pub`-Items (U-25)|
|W3|**Erweiterbarkeit:** wachsende öffentliche Enums und Structs (Fehler, Schichten, Klassen, Berichtsfelder) tragen `#[non_exhaustive]`, damit eine neue Variante kein SemVer-Major erzwingt; bewusst geschlossene Typen (Wire-Layout) sind ausgenommen und markiert|`semver` (cargo-semver-checks), Review-Liste|nicht gemessen|
|W4|**Deprecation-Policy:** `#[deprecated(since, note)]` mindestens ein Minor-Release vor Entfernung; Aufrufer im Workspace werden im selben PR migriert; `-D deprecated` im Workspace|`clippy-lintset`|`TenantId::new` noch aufgerufen (U-27)|
|W5|**Formate und Algorithmen sind austauschbar:** jedes persistente oder signierte Format trägt Formatversion **und** Algorithmus-ID (AEAD, KDF, Hash, Signatur); unbekannte ID ⇒ Fail-Closed; Altformate werden gelesen und migriert oder abgelehnt, nie still falsch interpretiert. Post-Quanten-Verfahren sind kein Ziel v1.0, werden aber **nicht verbaut**|Format-Register (§8.2 Regel 4), Golden-Files|Löschbericht V1–V5 versioniert; übrige Formate je Register zu prüfen|
|W6|**Eine Typfamilie je Konzept (A2):** kein öffentlicher Typname in zwei Crates ohne Registereintrag mit Frist|`duplicate-types` (AST)|35 Namen (U-20)|
|W7|**Toolchain und MSRV:** Toolchain gepinnt (Ist: 1.89.0); `rust-version` je Crate = Pin; ein Toolchain-Wechsel ist ein eigener PR mit voller Baseline|`rust-toolchain.toml`, `fmt`, `clippy-lintset`|Pin vorhanden|
|W8|**Abhängigkeitshygiene:** `default-features = false` für externe Crates, soweit möglich; neue Netz-, Krypto- oder `unsafe`-Abhängigkeit nur mit ADR; Lizenz und Herkunft je Abhängigkeit im SBOM|`deny`, `audit`, `sbom`, `egress-inventory`|`tokio full` (U-09)|
|W9|**Dokumentation am Code:** `MODULE.md` ≤ 120 Zeilen je Crate, `//!`-Kopf mit Zweck und Invarianten je Datei (im Repo bereits üblich), Doctests für öffentliche Beispiele; Integrationsleitfäden deutsch, `rustdoc` einheitlich (Entscheidung E-3.8)|`doc-truth`, `claims-consistency`|0 von 33 `MODULE.md`|
|W10|**Testlaufzeit und -isolation:** `cargo test -p <crate>` ≤ Budget (Startwert 120 s `quick`; langsame Tests hinter `slow-tests`/nextest-Profil); keine Uhr, kein Netz, keine Reihenfolgeabhängigkeit|`test-runtime-budget` (nightly), `test-integrity`|13 Crates > 45 s (U-17)|
|W11|**Schnitt nach Änderungsgrund:** Crates über 10.000 Produktionszeilen (`xtask` 32.572, `store` 13.521) werden auf Teilung geprüft; Ergebnis ist eine begründete Schnittstudie (WP 3.9), **keine** Vorab-Entscheidung|Schnittstudie, ADR|—|

**Zukunftsfähigkeit der Löschkette:** Rechtsauslegung, Fristen und Aufsichtspraxis ändern sich häufiger als Code. Deshalb enthalten Berichte `assumptions[]` und `residuals[]`, Fristen und Legal-Hold-Kataloge sind **Konfiguration, kein Code** (O-02), und die Rechtsmatrix (§9.12) macht sichtbar, welcher Baustein bei einer Rechtsänderung angefasst wird.


---

## 5. Modulkatalog

### 5.1 Baustein-Familien

|Nr.|Familie|Crates|Frage, die der Baustein beantwortet|
|---|---|---|---|
|F1|Speicher und Suche|`mvcc`, `store`, `checkpoint`, `vector`, `text`, `rank`, `engine`, `db`|Wie speichere und finde ich Wissen lokal, konsistent und schnell?|
|F2|Löschung und Nachweis|`crypto`, Scanner in `store`/`vector`/`text`/`checkpoint`, `erasure` (neu), `audit-export`, `avv-generator`|Wie lösche ich nach DSGVO nachweisbar, und was belege ich dafür?|
|F3|Inferenz und Embedding|`infer-candle`, `infer-onnx`, `infer-ollama`, `kvcache`, `adapt`|Wie betreibe ich lokale Modelle und Einbettungen datenschutzkonform und performant?|
|F4|Agenten und Gedächtnis|`graph`, `cognition`, `router`, `agent`|Wie baue ich budgetierte, wiederanlauffähige Agenten mit Gedächtnis?|
|F5|Schutz und Schnittstellen|`privacy`, `mcp`, `py`, `sandbox`, `license`|Wie schütze ich Ein-/Ausgänge und binde das System an Anwendungen an?|
|F0|Fundament|`types`, `ports`, `sys`, `simd`, `wire`, `testkit`|Gemeinsame Typen, Verträge, OS- und SIMD-Primitive|

### 5.2 Katalog mit Ist-Kennzahlen und Zielwerten

Methode: nicht leere Nicht-Kommentarzeilen aus `*.rs`; „Test-Z.“ = `tests/`, `benches/`, `#[cfg(test)]`-Blöcke und `*_tests.rs`. `wire` enthält ~1.200 generierte Zeilen. Werte sind E1 und **nicht** mit dem Altgate `loc-budget` vergleichbar. Reife Ist = `capabilities.toml`; Reife Ziel = Zielwert für v1.0 nach §1.4 (`stable` verlangt E3). „Hart abh.“ = transitive harte Workspace-Abhängigkeiten (aus den Manifesten berechnet; Soll aus §4.5).

|Crate|Ring|Kl.|Familie|Code-Z.|Test-Z.|Hart abh. Ist → Soll|Reife Ist|Reife Ziel v1.0|
|---|:-:|:-:|---|--:|--:|:-:|---|---|
|`contextra-types`|0|K|F0|4.236|876|0|stable|stable|
|`contextra-ports`|0|K|F0|1.679|1.250|1|stable|stable|
|`contextra-sys`|0|K|F0|591|193|0|stable|stable|
|`contextra-simd`|0|K|F0|1.114|398|1|stable|stable|
|`contextra-wire`|0|K|F0|346 (+1.281 generiert)|312|0|stable|stable|
|`contextra-testkit`|Werkzeug|–|F0|712|97|2|stable|n. a.|
|`contextra-mvcc`|0|K|F1|2.475|1.088|2|stable|stable|
|`contextra-store`|1|K|F1|14.288|27.392|5 → 5|beta|stable|
|`contextra-checkpoint`|1|K|F1|2.252|4.177|5 → 4|stable|stable|
|`contextra-vector`|0|K|F1|8.950|11.545|6 → 5|beta|stable|
|`contextra-text`|0|K|F1|2.829|7.484|2|beta|stable|
|`contextra-rank`|0|K|F1|2.972|1.997|2|stable|stable|
|`contextra-engine`|3|K|F1|10.290|11.485|15 → 11|beta|stable|
|`contextra-db`|3|K|F1|1.284|9.646|17 → 12|beta|stable|
|`contextra-crypto`|0|K|F2|5.609|7.089|2|beta|stable (nach V8)|
|`contextra-audit-export`|0|E|F2|214|180|2|experimental|beta|
|`contextra-avv-generator`|0|E|F2|196|154|1|experimental|beta|
|`contextra-infer-candle`|2|M|F3|2.662|2.954|3|stable|beta|
|`contextra-infer-onnx`|2|M|F3|971|818|3|experimental|beta|
|`contextra-infer-ollama`|2|M|F3|2.756|1.601|3|stable|beta|
|`contextra-kvcache`|1|M|F3|2.434|2.716|3|stable|beta|
|`contextra-adapt`|0|M|F3|2.193|2.794|2|experimental|beta oder Rückbau|
|`contextra-graph`|0|M|F4|8.711|13.722|3|stable|beta|
|`contextra-cognition`|3|M|F4|3.191|3.647|16 → 6|stable|beta|
|`contextra-router`|3|M|F4|1.691|4.751|5 → 4|stable|beta|
|`contextra-agent`|3|M|F4|2.574|4.871|20 → 10|stable|beta|
|`contextra-privacy`|3|M|F5|2.243|982|2|stable|beta|
|`contextra-mcp`|4|M|F5|3.806|3.165|22 → 16|beta|beta|
|`contextra-py`|4|M|F5|1.426|449|20 → 12|stable|beta|
|`contextra-sandbox`|2|M|F5|1.105|1.276|0|stable|beta|
|`contextra-license`|4|E|F5|307|883|2|experimental|experimental|
|`contextra`|4|K|Fassade|722|1.977|19 → 14|stable|stable|
|`contextra-core`|0|–|entfällt|106|181|4|stable|entfällt|
|`contextra-bench`|Werkzeug|–|Werkzeug|4.303|2.007|–|–|n. a.|
|`xtask`|Werkzeug|–|Werkzeug|32.572|11.599|–|–|n. a.|

Lesehilfe: **Reife Ist** stammt aus `capabilities.toml` und ist **nicht belegt** (kein Build-/Testlauf, §1.2); `stable` dort ist bis zum Evidence-Pack eine Behauptung (U-01). **Reife Ziel** folgt §4.2 und §9.10: `[K]` strebt `stable` an, `[M]` mindestens `beta`, `[E]` bleibt `experimental` bzw. `beta`. `crypto` erreicht `stable` erst nach externem Review (V8). `erasure` (neu, Ziel `beta`, §9.10) hat noch keinen Quelltext. Zählung: eigener Zähler (Python, Klammerzählung für `#[cfg(test)]`-Blöcke), **nicht** AST-basiert; verbindlich wird der Wert aus WP 0.4. Messwerte weichen von der Vorfassung ab (z. B. `store` 14.288 statt 13.500 Code-Z.), weil Zählmethode und HEAD sich unterscheiden.

### 5.3 Modulsteckbriefe (Vertrag, Ist-Befund, Pflicht bis Zielreife)

Die Steckbriefe fassen je Crate **Verantwortung**, **Befund am Code (E1)** und **Pflicht bis Zielreife** zusammen. Aussagen des Audit-Berichts, die nicht am Code belegt sind, stehen nicht als Befund.

#### F0 – Fundament

|Crate|Verantwortung|Befund (E1)|Pflicht bis Zielreife|
|---|---|---|---|
|`types`|`DocId`, `TxId`, `TenantId`, `RequestContext`, `ContextraError`, `DistanceMetric`; Feature `docid-128`|`forbid(unsafe_code)`; 29 abhängige Crates (höchster Fan-in)|Öffentliche API einfrieren (SemVer); Feature `docid-128` in Matrix testen; Property-Tests für ID-Roundtrip|
|`ports`|Austausch-Traits (§6)|enthält domänenspezifische Dateien `graph.rs`, `graph_index.rs`, `kv.rs`, `kv_bridge_port.rs`, `attention.rs`; `WitnessLayer` mit alten sieben Varianten|Domänen-Ports auslagern (WP 2.4); `DeletionLayer` auf zehn Schichten (WP 5.1); Test-Doubles nur über Feature `testkit`|
|`sys`|`mmap`, FD-Limits, atomare Temp-Dateien, Durability-Primitive|42 `unsafe`-Zeilen, `forbid` fehlt (Insel)|`// SAFETY:` je Block belegen (Gate `unsafe-inventory`); Miri; Plattformmatrix (Linux Pflicht, macOS optional)|
|`simd`|Distanzkerne (AVX2/AVX-512/NEON) mit Skalarfallback|310 `unsafe`-Zeilen; Differential-Tests gegen Skalar (laut Audit)|Skalar-Standard, bis Benchmark ≥ 20 % Vorteil belegt (E33); Miri-Lauf der Skalarreferenz; Fuzz über Längen|
|`wire`|FlatBuffers-Layout, IPC-Protokoll|47 `unsafe`-Zeilen, 16 Panic-Muster im Generat (`.unwrap()` in `contextra_generated.rs`)|Generat isolieren, `flatc`-Version pinnen (xtask-Prüfung `check_flatbuffers_drift`), Fuzz des Verifiers|
|`testkit`|Fixtures, `ManualClock`, `FaultVfs`, Chaos-Matrix, Modellorakel (`ModelStore`)|nur `dev-dependency`|Orakel unabhängig vom Produktivcode halten (INV-ORACLE-1)|

#### F1 – Speicher und Suche

|Crate|Verantwortung|Befund (E1)|Pflicht bis Zielreife|
|---|---|---|---|
|`mvcc`|Snapshots, SSI-Validator, Sequenzlog|hängt nur von `types`, `ports`|SSI-Nachweis (loom + Modellorakel), Write-Skew-Tests|
|`store`|LSM, WAL v3 mit HMAC-Kette, Manifest, Kompaktion, Memtable/SSTable-Scanner|`beta`; 14.288 Code-Z., 27.392 Test-Z.; fremde Dev-Kanten (`graph`, `vector`, `sandbox`)|Crash-Matrix (`crash_kill_durability`), Fehlerpropagation von `fsync`, Dev-Kanten entfernen, Blob-/Key-Scanner (F2)|
|`checkpoint`|Checkpoints, Pin-/Orphan-Handling, Manifest-Verifikation|hängt von `contextra-core`|`core` entfernen; Restore-Guard und `CheckpointScanner` (F2)|
|`vector`|HNSW, SQ8, ACL-Bitset, `HnswScanner`; DiskANN/RaBitQ experimentell|`beta`; `crypto` nur im DiskANN-Pfad genutzt|`crypto` hinter Feature; Slot-Nullung belegt (INV-HNSW-SLOT-1); Recall-Benchmark vs. fremde Baseline|
|`text`|BM25/BM25F, Block-Max-WAND, Tokenizer mit deutscher Komposita-Zerlegung, `TextScanner`|`beta`; **`panic!` in `tokenizer.rs:30`**|Panic entfernen (WP 1.1); Recall-/Latenz-Benchmark; Fuzz des Tokenizers|
|`rank`|RRF-Fusion, Kalibrierung, Coherence-Bonus|Feature `dibud`; 2.972 Code-Z.|Aufteilung `fusion/` vs. Rest prüfen (Kernanteil minimal halten)|
|`engine`|Planner, Collections, Chunking, Retention, Bounded-Compute-Pool, SSI-Gate|`beta`; 10.290 Code-Z.; harte Kante auf `graph`|`graph` hinter Port; Saga nach `erasure` verlagern; Mandanten-Isolationstests|
|`db`|Embedded-API, Vault, Query-Rewriter, Homeostat|`beta`; harte Kanten auf `cognition`, `adapt`; 10 Panic-Muster nur in Testcode|Kanten entfernen; Re-Exports in Fassade|

#### F2 – Löschung und Nachweis

|Crate|Verantwortung|Befund (E1)|Pflicht bis Zielreife|
|---|---|---|---|
|`crypto`|AEAD (AES-256-GCM-SIV), KDF, Signaturen (Ed25519), `DeletionProof`, `AuditChain`, `RevocationLog`, `KmsProvider`|`beta`; enthält KV-Cache-Kryptografie (U-11); `forbid(unsafe_code)` in 12 Dateien|KV-Teile auslagern; KeyTable/Destroy-Protokoll nach §9.4; Zeroize- und Constant-Time-Tests; externer Krypto-Review vor `stable`|
|`erasure` (**neu**)|Löschsaga, `ErasedSet`, Planung, Berichtsbildung, Lesesperre-Port|existiert nicht; Teile liegen in `engine`, `mcp`, `crypto`|Neues Ring-3-Crate (WP 5.5), hängt nur von Ports|
|`audit-export`|Verarbeitungsverzeichnis (Art. 30)|`experimental`; 214 Code-Z.|Formatvertrag (JSON-Schema), Golden-Files, Dokumentation der Grenzen|
|`avv-generator`|AVV-Vorlagen (Art. 28)|`experimental`; 196 Code-Z.; **keine Rechtsberatung**|Vorlagen versionieren, juristische Prüfung als externe Abnahme (O-05)|

#### F3 – Inferenz und Embedding

|Crate|Verantwortung|Befund (E1)|Pflicht bis Zielreife|
|---|---|---|---|
|`infer-candle`|lokale GGUF-Inferenz (Q4_K_M/Q8_0), Embeddings, ChatML; Features `kv-bridge`, `kv-stage-b`, `cuda`|laut Capabilities `stable` ohne belegtes E3; 2.662 Code-Z.|Determinismustest (festes Seed), Mandanten-KV-Test (INV-KVTENANT-1), Abbruch-/Stream-Tests, Latenz- und Speicher-Benchmark je Modellgröße|
|`infer-onnx`|ONNX-Embeddings (`ort`), Reranking|Feature `onnx` zieht `reqwest`, `tokenizers`, `ndarray`; `experimental`|Modell-Download von `huggingface.co` **entfernen oder hinter Egress-Freigabe**; Prüfsummen der Modelldateien erzwingen (U-07)|
|`infer-ollama`|REST-Client für lokalen Ollama-Dienst|`reqwest`; Vorfassung meldete `RING-LAYER-EXCEPTION` (eigener PRNG), am HEAD 52d3227f0 nicht auffindbar|Nur Loopback standardmäßig; TLS-/Host-Allowlist; Gate `determinism` bestätigt, dass keine eigene Zeit-/Zufallsquelle genutzt wird (sonst `Clock`-/`Rng`-Port)|
|`kvcache`|quantisierter KV-Cache, Eviction, Zeroize beim Drop|`forbid(unsafe_code)` in 4 Dateien; Features `kv-encryption`, `content-addressed-kv-cache`|Löschkaskade (INV-DERIVED-1, `CacheScanner`); Zeroize-Test per Prozessspeichersonde|
|`adapt`|PID-Latenzregelung, Sherman-Morrison, Catoni, Bandit|`experimental`; 2.193 Code-Z.; Re-Exports|Nur eigenständig nutzbar halten; numerische Stabilitätstests; Nutzen per Benchmark belegen oder zurückbauen|

#### F4 – Agenten und Gedächtnis

|Crate|Verantwortung|Befund (E1)|Pflicht bis Zielreife|
|---|---|---|---|
|`graph`|CSR-Graph, PageRank/PPR, Hyperkanten, Sitzungsbaum|8.711 Code-Z., 13.722 Test-Z.; Zyklus mit `store` (Dev); Mandantenbezug prüfen|Mandanten-Isolation und Löschkaskade (INV-DERIVED-1, VETO-F10); `RelationIndex`-Port|
|`cognition`|Konsolidierung, Wartungsplaner, Sleep-Cycle|3.191 Code-Z.; hängt an `engine`-Typen|Auf Ports umstellen; Löschkaskade; Determinismus mit `ManualClock`|
|`router`|Kontextueller Bandit, Drift-Erkennung, Konformale Kalibrierung|Abhängigkeiten `privacy`, `wire`|Sicherer Fallback bei Drift testen; Provider-Ports; Reproduzierbarkeit mit `Rng`-Port|
|`agent`|Orchestrator, State-Graph, DLQ, Checkpointing, `TokenBudget`|transitive Last 20|Entkoppeln (WP 2.6); Crash-/Replay-Tests; Budget-Race-Tests; Berechtigungsmodell für Werkzeuge|

#### F5 – Schutz und Schnittstellen

|Crate|Verantwortung|Befund (E1)|Pflicht bis Zielreife|
|---|---|---|---|
|`privacy`|PII-Klassifikation (E-Mail, IBAN, API-Key …), Schwärzung, Egress-Gateway|`stable` laut Capabilities; deutsche Formate (IBAN, Steuer-ID, Telefon) als Prüfliste|Golden-Corpus deutscher PII; Falsch-Negativ-Rate messen und ausweisen (keine Vollständigkeitszusage)|
|`mcp`|MCP-Server über stdio, Prompt-Injection-Guard, Exfiltrations-Detektor|`beta`; kein HTTP; transitive Last 22|Fuzz der JSON-RPC-Schicht; Werkzeug-Allowlist; Last verringern|
|`py`|PyO3-Bindings|`stable` laut Capabilities, 449 Test-Z. bei 1.426 Code-Z.|Zielreife `beta`; FFI-Panic-Grenze (xtask-Prüfung `check_ffi_panic_boundary`); Python-Tests|
|`sandbox`|Wasmtime-Sandbox, Fuel, Timeout|`stable` laut Capabilities, keine Abhängigkeiten|Escape-Tests, Fuel-Determinismus; Reife `beta`|
|`license`|Ed25519-Lizenzprüfung|`experimental`; **nicht** Teil des Pflichtbaus|Über `LicenseGate`-Port; Offline-Prüfung; keine Netzverbindung|

### 5.4 Entfallende Crates

`contextra-core` (106 Code-Z.) ist eine Kompatibilitätsfassade mit Re-Exports von `types`, `ports`, `mvcc`, `wire`; sie hat **61 Vorkommen in 32 Dateien** im Workspace (zeilenbasierte Zählung). Sie entfällt in zwei Schritten: (1) als `deprecated` markieren, (2) Verwendungen auf direkte Crates umstellen, dann entfernen (ADR-D06, WP 2.8).

### 5.5 Crate-Review: Wartbarkeit, Entkopplung, Baseline (neu in 3.1)

Dieser Abschnitt prüft **jedes der 35 Mitglieder** gegen §4.4 (Kantenregeln), §4.5 (Budgets), §4.9 (Wartbarkeit) und die Baseline (§1.2a). Er ist **generiert** aus `metrics-2026-10-28.json` und `per_crate_summary.json` (beide E1/E2, Methode der Metrik-Datei unbekannt, U-18) und wird durch `cargo xtask gen-module-docs` (WP 3.1) abgelöst.

**Lesehilfe.** *Prod-Z.* = Produktionszeilen laut Metrik-Datei (weichen von §5.2 ab, U-18). *Trans.* = harte transitive Workspace-Crates, Ist → Soll (fett = über Budget). *Hart außerhalb §4.4* = direkte, nicht optionale Normalkanten, die die Soll-Tabelle nicht nennt (`router` → `privacy`/`wire` gilt dort als „optional“ und wird in §5.5 nicht als Abweichung gezählt, steht aber in U-19). *Fremde Dev-Kanten* = Dev-Abhängigkeiten außer `testkit` und Selbstkante. *Panic* = `unwrap/expect/panic` laut Metrik, Wert für `wire` überwiegend Generat, für `xtask` unsicher (U-26). *check/clippy/test* = Ergebnis je Crate (`T/O` = Timeout 45 s, unbekannt).

|Crate|Ring|Prod-Z.|Trans.|Hart außerhalb §4.4|Fremde Dev-Kanten|Panic u/e/p; allow|check|clippy|test|Maßnahmen (WP)|
|---|:-:|--:|:-:|---|---|---|:-:|:-:|:-:|---|
|`contextra-types`|0|2643|0|–|–|0/0/0; allow 1|grün|rot|grün|API einfrieren (W3); 1 `allow` registrieren (1.2); `TenantId`-Duplikat in `privacy` auflösen (2.13); Clippy-Ursache (0.6)|
|`contextra-ports`|0|1679|1|–|–|0/0/0; allow 0|grün|rot|grün|Domänen-Ports auslagern (2.4); Duplikate `HyperEdgeId`/`RoleId`/`RoleBinding`/`GraphMutationError` (2.13); 3 Zeitaufrufe → `Clock` (2.12); `MetricsSink` nur Null-Impl. (3.4)|
|`contextra-sys`|0|591|0|–|–|0/0/0; allow 0|grün|grün|grün|Referenz-Crate (check, clippy, test grün); `// SAFETY:` je Block belegen (3.2)|
|`contextra-simd`|0|1114|1|–|–|0/0/0; allow 0|grün|rot|grün|Clippy-Ursache (0.6); Miri und Differential-Tests (6.1)|
|`contextra-wire`|0|1530|0|–|–|16/0/0; allow 3|grün|grün|grün|Generat isolieren: 16 `unwrap`, 43 `unsafe` ohne `// SAFETY:` im Generat; `forbid` für Handcode (1.4)|
|`contextra-testkit`|Tool|622|2|–|–|0/0/0; allow 0|grün|rot|grün|Doubles je Port und Konformitätssuiten (3.4); Clippy (0.6)|
|`contextra-mvcc`|0|1585|2|–|–|0/0/0; allow 0|grün|rot|rot|Test-Target `ssi_metrics_observability.rs` klären (0.5); 14 Zeitaufrufe in Ring 0 → `Clock` (2.12); loom (4.4)|
|`contextra-store`|1|13521|5 (Soll 5)|–|`vector`, `graph`, `sandbox`|0/2/4; allow 0|grün|rot|T/O|Timeout klären (0.7); undefinierte `cfg(feature)` (1.5); blockierendes I/O (1.7); Dev-Kanten (2.10); 2 `expect` + `panic!` (1.2); `_for_test` nicht öffentlich (1.6); Dateien > 800 Z. (W1)|
|`contextra-checkpoint`|1|1630|**5 → 4**|`core`|`store`|0/0/0; allow 0|grün|rot|grün|`core` entfernen (2.8); Dev `store` → Port-Double (2.11); Konstanten-Stubs `hardlink_cloner` prüfen (1.6)|
|`contextra-vector`|0|7878|**6 → 5**|`crypto`|–|1/0/0; allow 0|grün|rot|T/O|Timeout (0.7); `crypto` hinter Feature (2.10); blockierendes I/O in `diskann`/`core_rebuild` (1.7); 1 `unwrap` (1.2)|
|`contextra-text`|0|2819|2|–|–|0/6/1; allow 1|grün|rot|T/O|Timeout (0.7); `panic!` im Tokenizer, 6 `expect` in `purge`/`scanner` (1.1, 1.2); Test-`unsafe` `alloc_profiler` (§17 F-01)|
|`contextra-rank`|0|2972|2|–|`adapt`|0/0/0; allow 0|grün|rot|grün|Dev `adapt` entfernen (2.11); Duplikate `DriftDetector`, `ConformalCalibrator`, `BoundedTopK` (2.13)|
|`contextra-engine`|3|9372|**15 → 11**|`sys`, `graph`|`core`, `vector`|0/0/0; allow 0|grün|rot|T/O|Timeout (0.7); `graph` über `RelationIndex` (2.2); `sys` entfernen (2.14); Duplikate `AdaptiveDecayController` u. a. (2.6, 2.13); 7 Zeitaufrufe (2.12)|
|`contextra-db`|3|1011|**17 → 12**|`cognition`, `sys`, `crypto`, `store`, `vector`, `text`, `checkpoint`, `graph`, `rank`, `adapt`|`vector`|0/8/2; allow 0|grün|rot|T/O|Timeout (0.7); 10 von 13 Kanten außerhalb Soll (2.3, 2.14); Test-Target `deletion_proof_integration.rs` (0.5); 8 `expect` + 2 `panic` (1.2); `TenantId::new` migrieren (W4)|
|`contextra-crypto`|0|3690|2|–|–|0/0/0; allow 1|grün|rot|rot|Test-Target `audit_chain_tests.rs` (0.5); KV-Teile samt Duplikaten → `kvcache` (2.5, 2.13); `KmsProvider` verdrahten (5.3); `deletion_proof.rs` 910 Z. (W1)|
|`contextra-audit-export`|0|214|2|–|–|0/0/0; allow 0|grün|rot|grün|Formatvertrag, Golden-Files (6.5); Duplikat `AvvContext` (2.13)|
|`contextra-avv-generator`|0|196|1|–|–|0/0/0; allow 0|grün|rot|grün|Vorlagen versionieren; juristische Prüfung (O-05); Duplikat `AvvContext` (2.13)|
|`contextra-infer-candle`|2|2423|3|–|`crypto`|0/0/0; allow 0|grün|rot|T/O|Timeout (0.7); Dev `crypto` → Double (2.11); KV-Duplikate (2.5)|
|`contextra-infer-onnx`|2|892|3|–|–|0/0/0; allow 0|grün|rot|grün|Download entfernen/freigabepflichtig, blockierendes `std::fs` (2.7, 1.7); optionale Kante `infer-candle` entfernen (2.14)|
|`contextra-infer-ollama`|2|1684|3|–|`infer-onnx`|0/0/0; allow 0|grün|rot|grün|Loopback-Standard, 18 Netzstellen inventarisieren (2.7); Dev `infer-onnx` (2.11)|
|`contextra-kvcache`|1|2191|3|–|–|0/0/0; allow 0|grün|rot|grün|KV-Typen aus `crypto` übernehmen (2.5); 3 Zeitaufrufe (2.12); Löschkaskade S10 (5.2)|
|`contextra-adapt`|0|2193|2|–|–|0/0/0; allow 0|grün|rot|grün|`experimental`; Duplikat `AdaptiveDecayController` (2.13); Nutzen belegen oder zurückbauen (6.3)|
|`contextra-graph`|0|7796|3|–|`store`|0/0/0; allow 0|grün|rot|T/O|Timeout (0.7); Dev `store` (Zyklus, 2.10); `DeletionProof`-Duplikat in `cascade.rs` (2.13, 5.1); Test-`unsafe` `ppr_alloc_test` (§17 F-01)|
|`contextra-cognition`|3|3187|**16 → 6**|`engine`, `store`, `vector`|`infer-candle`|0/0/0; allow 0|grün|rot|T/O|Timeout (0.7); `engine`, `store`, `vector` über Ports (2.6); Dev `infer-candle` (2.11)|
|`contextra-router`|3|1344|**5 → 4**|–|`db`, `vector`, `graph`, `text`|0/0/0; allow 0|grün|rot|grün|4 Dev-Kanten → Doubles (2.11); `privacy`, `wire` optional (2.14)|
|`contextra-agent`|3|2415|**20 → 10**|`db`, `store`|`vector`, `text`|0/0/0; allow 0|grün|rot|T/O|Timeout (0.7); `db`, `store` entfernen (2.6); Dev `text`, `vector` (2.11)|
|`contextra-privacy`|3|1331|2|–|`crypto`|0/0/0; allow 0|grün|rot|grün|`TenantId`-Duplikat (VETO-F10, 2.13); `EgressGuard`-Duplikat mit `mcp` (2.13); `EntityRecognizer` nur Null-Impl. (3.4)|
|`contextra-mcp`|4|3111|**22 → 16**|`license`, `rank`, `crypto`, `privacy`, `router`|–|0/0/0; allow 0|grün|rot|T/O|Timeout (0.7); 5 harte Kanten (2.7, 2.14); Mock im Produktionscode (1.6); blockierendes `std::fs` im Binary (1.7)|
|`contextra-py`|4|1415|**20 → 12**|`router`, `rank`, `adapt`, `core`, `store`, `db`|–|0/0/0; allow 0|grün|rot|grün|6 harte Kanten (2.7, 2.14); `_trigger_panic_for_test` nicht öffentlich (1.6)|
|`contextra-sandbox`|2|1105|0|–|–|0/0/0; allow 0|grün|rot|grün|Clippy (0.6); 2 Zeitaufrufe (2.12); Escape-/Fuel-Tests (6.5)|
|`contextra-license`|4|307|2|–|–|0/0/0; allow 0|grün|rot|grün|nicht mehr hart von Fassade und `mcp` ziehen (2.14)|
|`contextra`|4|722|**19 → 14**|`core`, `license`|–|0/0/0; allow 0|grün|rot|T/O|Timeout (0.7); `core`, `license` nicht hart (2.8, 2.14); Feature `compliance` ungenutzt (2.9); Profile (3.7)|
|`contextra-core`|0|106|4|–|–|0/0/0; allow 0|grün|rot|grün|entfällt (2.8)|
|`contextra-bench`|–|4303|20|–|`sys`|0/0/0; allow 0|grün|rot|T/O|Timeout (0.7); nicht in `default-members`; Netz/Clock als Werkzeug einstufen|
|`xtask`|–|32572|0|–|–|57/9/2; allow 4|grün|rot|T/O|Timeout (0.7); 32.572 Z., 57 `unwrap`/9 `expect`/2 `panic`; Rückbau und Budget (3.6); `unsafe`-Zählung klären (§17 F-01)|

**Auswertung.**

1. **Entkopplung:** 8 Crates haben harte Kanten außerhalb §4.4 (`db`, `mcp`, `py`, `cognition`, `agent`, `engine`, `checkpoint`, `vector`, dazu die Fassade mit `license`); 10 Crates liegen über ihrem Transitiv-Budget. Der größte Hebel ist `db` (10 von 13 direkten Kanten außerhalb Soll): wer `db` entkoppelt (WP 2.3, 2.14), senkt zugleich `agent`, `py` und `contextra`.
2. **Qualität:** Nur `wire` und `sys` bestehen Clippy; die übrigen 33 sind rot bei **unbekannter** Ursache (U-16). Vor jeder Lint-Bereinigung wird die Ursache an den rohen Logs bestimmt (WP 0.6), sonst werden Symptome gejagt.
3. **Robustheit:** Produktions-Panics und `allow`s sind in `db`, `store`, `text`, `vector` und `wire` (Generat) zu behandeln; `xtask` ist nicht Bibliothek, wird aber mit eigenem Budget geführt (WP 3.6).
4. **Wartbarkeit:** Doppelte Typen (U-20) und fehlende `MODULE.md` (0 von 33) sind die größten, billig behebbaren Risiken für Folgearbeiten (auch durch LLM-Agenten).
5. **Zukunftsfähigkeit:** Ring-0-Crates mit direkter Zeitnutzung (`mvcc`) und KV-Kryptografie in `crypto` widersprechen den Zielgrenzen; ihre Behebung ist Voraussetzung für deterministische Recovery-Tests (G6).


---

## 6. Ports (Vertragsschicht)

### 6.1 Grundsatz

Ein Port ist ein Trait mit Austauschrolle. Jedes Modul **bietet** (`provides`) oder **fordert** (`requires`) Ports; zwischen Modulen existiert kein anderer Kopplungsweg als Ports und Typen aus `types`. Jeder Port hat: (1) Referenzimplementierung, (2) Test-Double in `testkit`, (3) Konformitätstests, die **jede** Implementierung bestehen muss (`ports-conformance`).

### 6.2 Port-Katalog (Ziel)

|Gruppe|Ports|Besitzer-Crate der Referenzimplementierung|
|---|---|---|
|Umwelt|`Clock`, `Rng`, `IdGen`, `Vfs`, `MetricsSink`, `Observer`|`sys`, `testkit`, Fassade|
|Persistenz|`StorageEngine`, `DurableLog`, `MergeOperator`, `CheckpointPort`|`store`, `checkpoint`|
|Suche|`VectorIndex`, `TextIndex`, `Reranker`, `QueryRewriter`, `DocumentStore`|`vector`, `text`, `rank`, `engine`|
|Inferenz|`EmbeddingProvider`, `LlmTextGenerator`|`infer-*`|
|Sicherheit|`KmsProvider`, `EffaceableKeySlot`, `LicenseGate`, `EgressPolicy` (neu)|`crypto`, `license`, `privacy`|
|Löschung|`LayerScanner`, `Purger`, `ErasedSetPort`, `ControlToken`|je besitzende Schicht, `erasure`|
|Domäne (im besitzenden Crate, nicht in `ports`)|`RelationIndex`, `KvStore`, `AttentionHook`|`graph`, `kvcache`|

### 6.3 Regeln

1. Ein Port darf **keine** Typen eines Rand-Crates in seiner Signatur tragen (Ist: `graph.rs`, `kv.rs`).
2. Ports sind **asynchron nur dort, wo I/O stattfindet**; Ring-0-Rechenports sind synchron (Gate `ring0-async`).
3. Fehler sind `ContextraError`-Varianten; kein `Box<dyn Error>` über Crate-Grenzen.
4. Ein Port-Vertrag ändert sich nur mit SemVer-Major des besitzenden Crates und ADR.
5. Ein Port ohne produktive Verdrahtung ist `experimental` (EP3).

---

## 7. Isolierte Prüfbarkeit: Audit, Verifikation, Leistung je Modul

Dies ist die zentrale neue Anforderung dieser Fassung: **Jedes Modul MUSS ohne die übrigen Module auditiert, verifiziert und leistungsgemessen werden können.**

### 7.1 Isoliertes Prüfkommando

```
cargo xtask module-audit <crate> [--profile quick|full|release]
```

Das Kommando (neu, WP 3.2) tut für ein einzelnes Crate:

|Schritt|Inhalt|Vorhandene Gates, die es nutzt|
|---|---|---|
|1 Graph|`cargo metadata`: hart abhängige Workspace-Crates, Zyklen, `may_depend_on`, `transitive_budget`|`ring-dag`, neu: `no-cycles`, `transitive-budget`|
|2 Build|`cargo build -p <crate> --no-default-features` und je zulässige Feature-Kombination|`feature-matrix`|
|3 Lint|`clippy` mit Produktions-Lintset; `allow`-Register|`clippy-lintset`, neu: `allow-register`|
|4 Inventare|Unsafe-Inventar, Panic-Inventar (inkl. Generat getrennt), öffentliche API|`unsafe-inventory`, `public-api`|
|5 Tests|Unit-, Integrations-, Property-Tests nur mit Ports/`testkit`|`ratchet`, `test-integrity`|
|6 Tiefe|Fuzz-Smoke, Miri (wo `unsafe`), loom (wo Lock-Protokoll), Mutationstests auf Invarianten|`fuzz-matrix`, `miri`, xtask `check_mutation_score_gate`|
|7 Egress|statische Prüfung auf Netz-Crates (`reqwest`, `hyper`, `std::net`) gegen die Egress-Tabelle|neu: `egress-inventory`|
|8 Bench|Benchmarks des Crates gegen `budgets.toml`|neu: `module-bench`|
|9 Pack|schreibt das Evidence-Pack (§7.2)|`evidence-complete`|

Ein Modul besteht die Isolation, wenn Schritt 1–2 **ohne** Nicht-Dependency-Crates gelingen und Schritt 5 ohne Netz und ohne Fremdmodule läuft (INV-ISO-1).

### 7.2 Evidence-Pack je Modul

Verzeichnis `evidence/<crate>/` mit maschinell erzeugten, reproduzierbaren Dateien:

|Datei|Inhalt|
|---|---|
|`manifest.json`|Crate, Version, Commit, Toolchain (1.89.0), Zeitpunkt, Befehl, Exit-Codes|
|`deps.json`|harte/optionale Abhängigkeiten, transitive Last, Lizenzen, bekannte Advisories (`cargo deny`, `cargo audit`)|
|`api.txt`|öffentliche API (`cargo public-api`)|
|`unsafe.json`, `panic.json`|Inventare, Generat getrennt|
|`tests.json`|Anzahl, Dauer, Flaky-Rate, Seeds|
|`depth.json`|Fuzz-Läufe, Miri-/loom-Status, Mutationsrate auf Invarianten|
|`bench.json`|Messwerte mit Hardware, Konfidenzintervall, Baseline|
|`claims.json`|Garantien/Invarianten des Moduls → Test-IDs → Evidenzstufe|

Das Pack bestimmt die **Reife** (§1.4): fehlt eine Datei, bleibt das Crate auf `experimental`.

### 7.3 Verifikationsleiter je Modul

|Stufe|Mittel|Gilt für|
|---|---|---|
|V1|Typsystem, Lints, Ringe, Kantenregeln|alle|
|V2|Unit- und Property-Tests (`proptest`) mit Ports/`testkit`|alle|
|V3|Differentialtests gegen Referenz (SIMD vs. Skalar; HNSW vs. Brute-Force; BM25 vs. naive Implementierung; Store vs. `ModelStore`)|F0, F1|
|V4|Fuzzing (Parser, Dekoder, Tokenizer, JSON-RPC, WAL-Replay)|`wire`, `store`, `text`, `mcp`|
|V5|Miri, loom (Locks, SSI)|Inseln, `mvcc`, `store`, `graph`|
|V6|Chaos mit `FaultVfs` (Absturz, `ENOSPC`, `EIO`, Teilschreibung)|`store`, `checkpoint`, `erasure`|
|V7|Mutationstests auf Invarianten|alle `[K]`|
|V8|Unabhängiger Review (Krypto, Löschkette, Rechtsfragen)|`crypto`, F2|

`stable` verlangt V1–V6 sowie V7 für `[K]`; V8 ist Voraussetzung für `certified-ready`.

### 7.4 Leistungsmessung je Modul

Jedes Modul mit Leistungsversprechen führt **Benchmark-ID**, **Fixture**, **Metrik**, **Budget** und **Baseline**. Messregeln: feste Hardwareklasse (Referenzgerät „KMU-Mini-PC“: 8 Kerne, 32 GB RAM, NVMe, ohne GPU; zweite Klasse mit GPU optional), `p50/p99/p99.9` mit Konfidenzintervall, Warm-up, ≥ 30 Wiederholungen, Commit und Toolchain im Ergebnis. Fremde Baselines sind Pflicht für Vergleichsaussagen (EP4).

|Modul|Benchmark (Auswahl)|Metrik|
|---|---|---|
|`store`|WAL-Commit-Latenz, Scope-Purge, SSTable-Lookup, Kompaktion unter Last|p50/p99/p99.9, MB/s|
|`vector`|HNSW-Build, Suche, SQ8-Recall@k, Slot-Nullung|Recall@10, QPS, ms|
|`text`|BM25-Suche, Index-Aufbau, deutsche Komposita|Latenz, Recall gegen Referenz|
|`rank`|RRF-Fusion bei k=10..100|µs|
|`simd`|Distanzkerne je Dimension (128–1536)|GB/s, Speedup vs. Skalar|
|`engine`|`remember`/`recall` Ende-zu-Ende (TTC)|p50/p99|
|`infer-*`|Tokens/s, TTFT, Peak-RSS je Modellgröße (3B/7B/14B), Embedding-Durchsatz|Tokens/s, MB|
|`kvcache`|Treffer-Rate, Eviction-Kosten, Zeroize-Kosten|%, µs|
|`router`|Entscheidungslatenz, Regret unter Drift|µs, Regret|
|`agent`|Schrittkosten, Checkpoint-/Recovery-Zeit, DLQ-Durchsatz|ms|
|`privacy`|PII-Scan-Durchsatz, Falsch-Negativ-/Falsch-Positiv-Rate|MB/s, %|
|F2|Löschlaufzeit je Scope-Größe, Scanner-Durchsatz, Berichtsgröße|s, MB|

Die vorhandene `contextra-bench` (4.303 Code-Z.) wird in **Benchmarks je Crate** (`crates/<x>/benches/`) zerlegt; `contextra-bench` bleibt für Ende-zu-Ende-Läufe und Fremdvergleiche.

### 7.5 Reife-Regeln

- `maturity = stable` setzt E3 und V1–V6 (+V7 für `[K]`) voraus; ein Crate, dessen Sicherheits-Ports nur Null- oder Test-Implementierungen haben, ist `experimental`.
- Die Reife eines Moduls ist höchstens die seiner schwächsten **harten** Abhängigkeit.
- Berichte (z. B. `SYSTEMATIC_CRATE_AUDIT.md`) tragen den Vermerk „Hypothese“ und werden vom Gate `claims-consistency` gegen `capabilities.toml` geprüft; Abweichung = rot.

---

## 8. Persistenz und Recovery (Rahmen)

Der Detailtext zu WAL, LSM, Formaten, Durability-Primitiven, Sidecar-Logs und Betriebszuständen steht unverändert in **Anhang C** (aus 2.4 §7 und §8). Dieser Abschnitt legt nur den **Modulrahmen** fest.

### 8.1 Besitz und Schnitt

|Verantwortung|Besitzer|Schnittstelle|
|---|---|---|
|WAL v3 (HMAC-Kette), Memtable, SSTable, Manifest, Kompaktion|`store`|Port `StorageEngine`|
|Durable Sidecar-Journale (Löschjournal, Audit, Revocation)|`store` implementiert, `crypto` nutzt|Port `DurableLog` (neu, WP 4.2)|
|Snapshots, SSI|`mvcc`|Typen in `ports`|
|Checkpoints, Restore-Guard|`checkpoint`|Port `CheckpointPort`|
|Dateisystem-Zugriff|`sys`, `store`|Port `Vfs`|

### 8.2 Verbindliche Regeln

1. **WAL First** (P24): kein persistenter Zustand außerhalb von WAL und Sidecar-Logs.
2. **Commit = durabler Flush** mit plattformgerechter Primitive (`fdatasync` Linux, `F_FULLFSYNC` macOS); `fsync`-Fehler werden propagiert (Gate `result-dropped-on-io`, xtask `check_result_dropped_on_io`).
3. **Atomare Datei-Ersetzung:** temporäre Datei → `fsync` → `rename` → `fsync` des Verzeichnisses.
4. **Format-Register:** jede persistente Struktur hat Magic, Version, Prüfsumme und Golden-File; unbekannte Version ⇒ Fail-Closed.
5. **Deterministische Recovery:** gleicher Anfangszustand + gleiche WAL-Folge ⇒ gleicher Zustand (G6).
6. **Plattform:** Linux x86_64/aarch64 ist Garantiebereich; macOS Entwicklungs- und Edge-Betrieb ohne Garantieaussage; Windows nicht Teil v1.0 (§2.4).

### 8.3 Isolierte Prüfung (Modulpflicht)

`store` MUSS ohne `vector`, `text`, `engine` prüfbar sein: Crash-Matrix (`crash_kill_durability`), Fehlerinjektion (`FaultVfs`), WAL-Replay-Fuzz, Orakelvergleich gegen `ModelStore`, Latenz-Benchmarks (§7.4). `mvcc` MUSS mit loom und Modellorakel prüfbar sein.

---

## 9. Baustein Löschkette (DSGVO-Löschung, Messung, Nachweis)

### 9.1 Zweck und Abgrenzung

Die Löschkette ist ein **eigenständiger, optional einbindbarer Baustein** (Familie F2). Er liefert für ein lokales Datenverzeichnis:

1. **technisch begründete Löschung** nach Art. 17 DSGVO (physische Bereinigung von Klartext-Strukturen und kryptografische Vernichtung persistenter Reste),
2. **Messung** der Löschwirkung durch Scanner der besitzenden Schicht,
3. einen **signierten Löschbericht** (`DeletionProof` V5), der nur ausweist, was gemessen ist,
4. **Lesesperre, Fristen und Aufbewahrungspflichten** (Einschränkung nach Art. 18, Legal Hold).

Sie ist **kein Rechtsnachweis** und ersetzt weder Rechtsberatung noch die Verantwortung des Verantwortlichen (§9.9). Sie bleibt **entkoppelt**: Jede Schicht-Messung gehört dem Crate, das die Daten hält; die Orchestrierung liegt in einem eigenen Crate `erasure` und kennt nur Ports.

### 9.2 Rechtlicher Anker (keine Rechtsberatung)

Technische Anforderungen leiten sich ab aus Art. 5 Abs. 1 lit. e und f (Speicherbegrenzung, Integrität/Vertraulichkeit), Art. 17 Abs. 1 und 2 (Löschung, Information von Empfängern), Art. 18 (Einschränkung), Art. 19 (Mitteilungspflicht), Art. 25 (Datenschutz durch Technikgestaltung), Art. 28 (Auftragsverarbeitung), Art. 30 (Verzeichnis) und Art. 32 (Sicherheit der Verarbeitung). Gesetzliche Aufbewahrungspflichten (z. B. handels- und steuerrechtlich) gehen der Löschung vor und führen zu `Restricted` statt `Deleted`. Quellenlage, Rechtsstabilitätsmatrix und Perspektivenmatrix stehen in Anhang C §12.1; offene Rechtsfragen in §16 (O-01 bis O-06). **Keine Aussage dieses Dokuments ist eine Rechtsauskunft.**

### 9.3 Zerlegung in Bausteine mit Besitzer und Ist-Stand

|Nr.|Baustein|Besitzer|Ist (E1)|Arbeitspaket|
|---|---|---|---|---|
|D1|Löschtypen (`DeletionLayer` zehn Varianten, `LayerWitness`, `ReachedClass`)|`types`/`ports`/`crypto`|`WitnessLayer` mit sieben Alt-Varianten; `DeletionLayer` in `crypto` vorhanden; **zweiter `DeletionProof` in `graph/src/cascade.rs`** (U-20); `LayerCleanupProof` nirgends definiert, aber von zwei Tests importiert (U-15)|WP 5.1, 0.5|
|D2|Schlüsselhierarchie: Keystore, KeyTable, `DocDek`, `ArtKey`, KEK-Epochen|`crypto` (Logik), `store` (Persistenz)|nicht gefunden (`KeyScanner`, `EffaceableKeySlot` fehlen); `KmsProvider` laut Scan **nicht verdrahtet** (U-25)|WP 5.3|
|D3|Destroy-Protokoll D1–D6, Epoch Seal|`crypto` + `store`|nicht gefunden|WP 5.4|
|D4|Scanner Ebene A/B je Schicht|`store`, `vector`, `text`, `checkpoint`, `kvcache`, `graph`, `engine`|Memtable, SSTable, WAL, HNSW, Text vorhanden; Blob, Cache, Checkpoint, Key, Lineage fehlen|WP 5.2|
|D5|Purger je Schicht (`compact_range(scope)`, Slot-Nullung, Segment-Rewrite)|besitzende Crates|`Purger`-Port vorhanden; Implementierungen teilweise|WP 5.2|
|D6|`ErasedSet`, Lesesperre, Key-Lock|`erasure` + Lesepfade|nicht gefunden|WP 5.5|
|D7|Löschsaga (Zustandsmaschine, Journal, Recovery)|`erasure`|nicht gefunden|WP 5.5|
|D8|Bericht `DeletionProof` V5, Siegel, Claim-Generator|`crypto` (Format), `erasure` (Bildung)|V5-Format im Code; Klassen-/Siegelbildung unklar (E0); Integrationstests `crypto/audit_chain_tests.rs` und `db/deletion_proof_integration.rs` **kompilieren nicht** (U-15)|WP 0.5, 5.6|
|D9|Backup/Restore (Zwei-Spuren), Restore-Guard, `ErasedSetMirror`|`checkpoint`, `erasure`|`checkpoint` vorhanden, Guard fehlt|WP 5.7|
|D10|Fristen, `RecipientLedger`, Legal Hold, Mixed-Policy|`erasure`|nicht gefunden|WP 5.8|
|D11|Lösch-Mutationskatalog (G7), Abnahmekriterien AC|`testkit`, `xtask`|nicht gefunden|WP 5.9|

### 9.4 Die zehn Löschschichten (Ziel)

Ebene **A** = Klartext-Strukturen (physisch bereinigen), Ebene **B** = persistente Reste unter einem Schlüssel (kryptografisch vernichten). Jede Schicht hat **genau einen** Besitzer, der Scanner **und** Purger stellt.

|Schicht|Typ|Ebene|Besitzer|Maßnahme|Ist-Scanner|
|---|---|---|---|---|---|
|S1 Primärdaten|`Lsm`|A, B|`store`|Records unter `DocDek`; Memtable zeroizen; `compact_range`, WAL-Scrub|Memtable, SSTable, WAL ✔|
|S1b Blob-Segmente|`BlobSegments`|B|`store`|Segment-Rewrite unter `DocDek`|✘|
|S2 Vektorindex|`VectorIndex`|A, B|`vector`|Slots nullen, Snapshot unter `DocDek`, Graph unter `ArtKey`|`HnswScanner` ✔ (Snapshot/Quantisierung ✘)|
|S3 Textindex|`TextIndex`|A, B|`text`|Segmente neu schreiben, Terme/Statistik bereinigen|`TextScanner` ✔|
|S4 Caches und Vault|`CachesAndVault`|A|`store`, `engine`, `privacy`|Block-/Embedding-/RRF-/Antwort-Cache und Vault invalidieren und zeroizen|✘|
|S5 Audit, Lineage, Sidecars|`AuditLineage`|–|`engine`, `crypto`|Ableitungen kaskadieren (INV-DERIVED-1); nur Referenz-Hashes|✘|
|S6 Interne Checkpoints|`Checkpoints`|B|`checkpoint`|Policy `Rewrite` (Standard), `ExpireNow`, `ExpireAfterHold`|✘|
|S7 Schlüsselmaterial|`KeyMaterial`|B|`crypto`, `store`|Nadelscan; Keystore, Wrap-Blobs, Temp-/`.bak`-Kopien|✘|
|S9 Graph (**neu aktiv**)|`Graph`|A|`graph`|Knoten/Kanten/Sitzungsbäume nach Scope entfernen, Neuaufbau|✘|
|S10 KV-Cache (**neu aktiv**)|`KvCache`|A|`kvcache`|Blöcke und Quantisierungspuffer des Mandanten/der Quelle zeroizen|✘|
|S8 Externe Sicherungen|Ausschluss|–|–|im Bericht ausweisen (`ExternalBackups`)|–|

**Änderung gegenüber 2.4:** `Graph` und `KvCache` waren `FeatureDisabled` (Archiv). Mit ADR-D01 sind sie **aktive, messbare Schichten**, sobald das Modul im Build ist. Ist das Modul **nicht** gebunden, ist `FeatureDisabled` nur zulässig, wenn der Start (B3) den fehlenden Datenbestand belegt; sonst `NotMeasured(NoPhysicalScanner)` und der Bericht ist höchstens `Partial`.

**Regeln.** (1) Jede Schicht stellt genau einen Witness-Typ aus. (2) Eine Schicht ohne Scanner, in der Daten liegen können, ist `NotMeasured`. (3) Embeddings und Vektorbytes sind vorsorglich personenbezogen. (4) CDC-Ereignisse sind inhaltsfrei. (5) **INV-SURFACE-1:** jede persistente Dateiklasse steht in genau einer Zeile dieser Tabelle oder im Ausschluss; ein neues Format ohne Schichtzuordnung bricht den Build (Gate `surface-coverage`). (6) Embeddings, Zusammenfassungen, Graphkanten und KV-Blöcke aus gelöschten Quellen kaskadieren über die Lineage.

### 9.5 Wirkungsklassen

Ein Bericht weist **nur die höchste Klasse** aus, deren Voraussetzungen gemessen sind (INV-CLASS-MEASURED-1).

|Klasse|Name|Aussage|
|---|---|---|
|**L**|Gesperrt|Scope für alle Lesepfade gesperrt (entspricht einer Einschränkung nach Art. 18); `ErasedSet`-Eintrag durabel|
|**P**|Klartext bereinigt|In keiner Klartextstruktur, die Contextra beschreibt, ist ein Inhalt des Scopes auffindbar (alle Ebene-A-Schichten gemessen, Positivkontrolle bestanden)|
|**C***|Vorläufig kryptografisch|Schlüssel vernichtet (D3 vollzogen), Epoch Seal steht aus|
|**C1**|Reste unlesbar (Software-Schlüssel)|Persistente Kopien liegen nur als Chiffrat unter vernichteten Schlüsseln; **Annahme A-FLASH**|
|**C2 / C3**|wie C1, Wurzel im Modul / externer KMS|A-FLASH entfällt bzw. wird durch Anbieterattest ersetzt (Prüfpflicht des Verantwortlichen)|
|**M**|Messvollständig|P und C erreicht; Hygiene (Kompaktion, Checkpoint-Policy, Graph-Neuaufbau) gemessen|

Orthogonale Pflichtangaben im Bericht: `legal_state` (`Eingeschränkt`, `TechnischGelöscht`, `BeschränktAufbewahrt`), `backup_window`, `assumptions[]`, `residuals[]` (mit Frist), `platform_guarantee`, `excluded[]`.

### 9.6 Schlüsselvernichtung und Saga (Kurzfassung)

**Destroy-Protokoll** (durabel, idempotent, Journal-Intent **vor** Wirkung): D1 Intent + Pre-Destruction-Inventar · D2 Key-Lock, `KeyCache` zeroizen · D3 neue KeyTable-Generation ohne Eintrag (tmp → `fsync` → `rename` → Verzeichnis-`fsync`) · D4 alte Generation und Temp-/`.bak`-Kopien entfernen · D5 `RevocationLog`-Eintrag · D6 Journal-Done + Nadelscan. Ein Schlüssel gilt erst nach D3 als vernichtet; der Bericht trägt `key_destroyed` erst nach D6. Auf SSD und Copy-on-Write-Dateisystemen trägt die Wirkung der **Epoch Seal** (KEK-Wechsel), nicht das Überschreiben.

**Saga:**

```
Requested → Authorized → Frozen → LogicallyDeleted → PlaintextPurged → PlaintextMeasured
  → KeyDestroyed → CryptoMeasured → (Sealed) → Hygiene → HygieneMeasured → Reported → Completed
        └─(Aufbewahrungspflicht/Legal Hold ab Frozen)→ Restricted{basis, until}
        └→ Rejected        (jeder Zustand) → Partial{outstanding, reason} → Retry/Escalate
```

**INV-DEL-ORDER-1:** Schritt `KeyDestroyed` beginnt erst, wenn Klasse P für jede Ebene-A-Schicht gemessen und das Pre-Destruction-Inventar durabel ist (Ausnahme: Notpfad, nur L erreichbar). **INV-NO-RESURRECTION-1:** Nach `KeyDestroyed` darf kein Restore und kein Retry einen Schlüssel der Einheit wiederherstellen.

### 9.7 Lesesperre, Backup, Fristen

- **Lesesperre (G4):** ab `Frozen` liefert kein Lesepfad Daten des Scopes (inkl. `as_of`, Export, Checkpoint-Erzeugung); Spiegelung des `ErasedSet` an die Sicherungsspur.
- **Zwei-Spuren-Backup:** Spur D (Daten, von Contextra erzeugt) trägt den Restore-Guard; Spur K (Schlüssel) wird getrennt verwahrt. Restore stellt Gelöschtes nicht wieder her.
- **Fristen:** Standard und Obergrenzen kalibrierbar (Anhang C §12.19); `RecipientLedger` dokumentiert Empfänger für Art. 17 Abs. 2 und Art. 19.

### 9.8 Isolierte Prüfung der Löschkette

Jeder Löschbaustein MUSS einzeln prüfbar sein:

|Baustein|Isolierter Test|
|---|---|
|Scanner je Schicht|Positivkontrolle (bekanntes Muster wird gefunden), Negativkontrolle, Differenz gegen Brute-Force-Scan der Rohdateien|
|Purger je Schicht|Vorher/Nachher-Scan mit `FaultVfs`-Abbrüchen an jeder Stelle|
|Destroy-Protokoll|Crash-Matrix D1–D6, kein lebender Schlüssel nach Wiederanlauf|
|Saga|Zustandsmaschinen-Modelltest, Idempotenz, `Partial`-Pfade|
|Bericht|Fuzz des Verifiers, Golden-Files V1–V5, falscher Schlüssel ⇒ `Err`, Bericht ohne Witness ⇒ nicht kompilierbar|
|Prozessspeicher|`/proc/self/mem`-Sonde (`testkit`) belegt Klartext-Reste im RAM (INV-HEAP-ZERO-1)|

**Lösch-Mutationskatalog (G7):** jede bewusste Fehlerinjektion (Scanner meldet 0, Purger überspringt Segment, Schlüssel nicht vernichtet, Bericht ohne Messung) MUSS mindestens einen Test scheitern lassen.

### 9.9 Ehrliche Grenzen (im Bericht verpflichtend)

Nicht durch Contextra kontrollierbar und deshalb **ausgewiesen**, nie behauptet: externe Sicherungen des Betreibers, Datenträger-Firmware (A-FLASH, Wear-Leveling, Überprovisionierung), Cloud-/Anbieter-Replikation, Host-Prompt-Historien, Daten bereits an Dritte übermittelt, Auslagerung (Swap) und Core-Dumps außerhalb des gehärteten Prozesses. Das Ergebnis ist ein **technischer Befund über das lokale Datenverzeichnis**, kein Rechtsnachweis und keine Zertifizierung. Externe Zertifizierung (z. B. nach Medienlöschvorgaben) ist Aufgabe des Betreibers (O-04).

### 9.10 Reife der Löschkette

|Teil|Reife Ist|Reife Ziel v1.0|Voraussetzung|
|---|---|---|---|
|Scanner-Port und vorhandene Scanner|`beta` (E1–E2)|`stable`|Differentialtests, Chaos|
|`DeletionProof` V5 (Format, Verifikation)|`beta` laut Manifest, **belegt nur E1**, solange die Integrationstests nicht kompilieren (U-15)|`stable`|Golden, Fuzz, externer Krypto-Review|
|Schlüsselhierarchie, Destroy, Seal|`experimental` (nicht umgesetzt)|`beta` (v1.0), `stable` danach|WP 5.3–5.4|
|Saga, `ErasedSet`, Lesesperre|`experimental` (nicht umgesetzt)|`beta`|WP 5.5|
|Backup/Restore-Guard|`experimental`|`beta`|WP 5.7|
|Gesamtbaustein „Löschkette“|**`experimental`**|`beta` mit Evidenz E3 je Teil|Phase 5 des PLAN.md|

Bis Phase 5 abgeschlossen ist, DARF kein Produkt, README oder Bericht die Löschkette als „produktionsreif“ oder „rechtssicher“ bezeichnen (INV-CLAIMS-1).

### 9.11 Schnittstellenvertrag und Entkopplung der Löschkette (neu in 3.1)

Die Löschkette ist als **Bibliotheksbaustein** so geschnitten, dass sie ohne `engine`, `db` und `store` gebaut, geprüft und ersetzt werden kann (M1, M2, ADR-D15). Maßgeblich ist die **Richtung der Abhängigkeiten**:

```
            Fassade `contextra` (einziger Composition Root, EP7)
     verdrahtet Scanner/Purger der Besitzer in den Koordinator
                                  │
   ┌──────────────────────────────┼─────────────────────────────────────────┐
   │  `erasure` (Ring 3)          │   Besitzer-Crates (je Schicht)          │
   │  Saga, ErasedSet, Bericht-   │   store    S1, S1b, S4(Teil), S7(Teil)  │
   │  Bildung, Lesesperre-Port    │   vector   S2      text   S3            │
   │  kennt NUR Ports + crypto-   │   engine   S4, S5  checkpoint  S6       │
   │  Berichtstypen               │   crypto   S7      graph  S9  kvcache S10│
   └───────────────┬──────────────┴──────────────┬──────────────────────────┘
                   ▼                             ▼
             `ports` (Verträge)  ◄───────  Besitzer implementieren Ports
```

**Entkopplungs-Invarianten des Bausteins**

|ID|Invariante|Prüfung|
|---|---|---|
|E-1|`erasure` hängt nur von `types`, `ports`, `crypto`; nie von `engine`, `db`, `store`, `vector`, `text`, `graph`, `kvcache`|`may-depend-on`, `module-isolation`|
|E-2|Messung und Reinigung einer Schicht liegen im Crate, das die Daten hält; `erasure` ruft nur `LayerScanner` und `Purger`|`ports-conformance`|
|E-3|Die Lesesperre wirkt über `ErasedSetPort` in **jedem** Lesepfad der Besitzer; ein Lesepfad ohne Prüfung ist ein Mutationstest-Fall (T-DEL-15)|Mutationskatalog|
|E-4|Das Berichtsformat (`DeletionProof` V5) liegt genau einmal in `crypto`; `graph` oder `engine` definieren keinen eigenen Beweistyp|`deletion-types-once`, `duplicate-types`|
|E-5|Zeit, Zufall, IDs, Dateizugriff kommen über `Clock`, `Rng`, `IdGen`, `Vfs`, `DurableLog`; Schlüssel- und Salt-Material aus dem System-RNG|`determinism`, `result-dropped-on-io`|
|E-6|Der Koordinator **lehnt den Start ab**, wenn eine Schicht, in der Daten liegen können, weder Scanner noch dokumentierten Ausschluss hat (INV-SURFACE-1, Fail-Closed)|Build-/Startfehler-Test|
|E-7|Ein Profil ohne `erasure` (z. B. **Search**) behauptet nichts über Löschung; die Fassade stellt dann weder `forget` mit Berichtsanspruch noch `Frozen`-Semantik bereit|`profile-audit`, `claims-consistency`|

**Verdrahtung (Pseudocode, nicht kompiliert; Namen sind Richtwerte, normativ sind E-1 bis E-7):**

```text
let erasure = ErasureBuilder::new(clock, id_gen, durable_log)
    .layer(store.scanners(), store.purgers())      // S1, S1b
    .layer(vector.scanner(), vector.purger())      // S2
    .layer(text.scanner(),   text.purger())        // S3
    .layer(cache_scanner,    cache_purger)         // S4
    .layer(lineage_scanner,  lineage_purger)       // S5
    .layer(checkpoint.scanner(), checkpoint.purger()) // S6
    .layer(key_scanner,      key_purger)           // S7
    .optional_layer(graph)                         // S9, nur wenn Modul gebunden
    .optional_layer(kvcache)                       // S10, nur wenn Modul gebunden
    .key_slot(effaceable_key_slot)                 // K0/K1 Software, K2/K3 Port
    .erased_set(erased_set_port)
    .build()?;   // Err, wenn Schicht ohne Scanner und ohne Ausschluss (E-6)
```

**Ist-Abgleich (E1):** Ports `LayerScanner` und `Purger` sowie fünf Scanner sind vorhanden (U-04); `erasure`, `ErasedSetPort`, `EffaceableKeySlot`, `DurableLog` fehlen; `DeletionProof` existiert doppelt (U-20); Löschintegrationstests kompilieren nicht (U-15); `KmsProvider` ist laut Scan unverdrahtet (U-25).

### 9.12 Rechtsmatrix: gesetzliche Vorgabe → Baustein → Nachweis (neu in 3.1)

Keine Rechtsberatung (§9.2, §9.9). Die Matrix zeigt, **welcher Baustein welche technische Anforderung trägt** und womit sie belegt wird; sie ist die Änderungsstelle, wenn sich Recht oder Aufsichtspraxis ändern (W-Zukunftsfähigkeit, §4.9). Maschinenlesbare Fassung: `docs/spec/legal-matrix.toml` (WP 5.10).

|Vorgabe (DSGVO, sinngemäß)|Technische Anforderung|Baustein / Crate|Nachweis|Status|
|---|---|---|---|---|
|Art. 5 Abs. 1 lit. e Speicherbegrenzung|Aufbewahrung nur so lange nötig; Fristen durchsetzbar|Retention-Reaper (`engine`), Fristen und Legal Hold (`erasure`, D10)|T-RET-*, Zustandsübergangstests|Reaper vorhanden (Anhang C-11.6); Fristen offen|
|Art. 5 Abs. 1 lit. f, Art. 32 Integrität, Vertraulichkeit|Verschlüsselung, Integrität, Mandantentrennung|`crypto`, `store` (WAL-HMAC), `privacy`, INV-TENANT-*|Krypto-Tests, T-TEN-1/2, externer Review (V8)|teilweise (`crypto` `beta`)|
|Art. 5 Abs. 2, Art. 24 Rechenschaft|Nachweis der Maßnahmen|Evidence-Pack, `AuditChain`, `DeletionProof`|`evidence-pack`, Verifier-Fuzz|Format vorhanden, Integrationstest rot (U-15)|
|Art. 17 Abs. 1 Löschung|technisch begründete Löschung aller Schichten|Saga (`erasure`), Scanner/Purger je Schicht, Destroy-Protokoll (D1–D6)|T-DEL-1/2/3/4/5, Mutationskatalog (T-DEL-15)|5 von 10 Schichten gemessen; Saga fehlt|
|Art. 17 Abs. 2, Art. 19 Information von Empfängern|Empfänger dokumentieren und benachrichtigen können|`RecipientLedger` (D10)|Zustandstests; Bericht weist Empfänger aus|fehlt (WP 5.8)|
|Art. 18 Einschränkung|Sperre ab Anordnung, auch in Export, `as_of`, Checkpoint|`ErasedSet`, Lesesperre (`Frozen`, `Restricted`)|T-DEL-7/8/9|fehlt (WP 5.5)|
|Gesetzliche Aufbewahrungspflichten|Löschung weicht Aufbewahrung: `Restricted{basis, until}`|Legal-Hold-Mechanismus (D10); **Kataloge sind Rechtsfrage (O-02)**|Zustandsübergänge, Bericht `BeschränktAufbewahrt`|Mechanismus offen|
|Art. 25 Datenschutz durch Technikgestaltung und Voreinstellung|sichere Standardwerte|Netz aus (VETO-NET1), Mandantentrennung, `default = []`, Fail-Closed|T-EGR-1, `egress-inventory`, `feature-matrix`|`infer-onnx` Download (U-07), `store`-cfg-Features (U-22)|
|Art. 28 Auftragsverarbeitung|AVV-Unterlagen|`avv-generator`|Golden-Files; juristische Prüfung (O-05)|`experimental`|
|Art. 30 Verzeichnis von Verarbeitungstätigkeiten|Verzeichnis erzeugen|`audit-export`|Formatvertrag, Golden-Files|`experimental`|
|Art. 15, 20 Auskunft, Übertragbarkeit|Export der Daten einer betroffenen Person|**nicht Teil von v1.0** (Nicht-Ziel); Export-Baustein optional|ADR vor Aufnahme (§2.4)|offen|
|Art. 33, 34 Meldung von Verletzungen|Erkennung und Meldung|**nicht Teil von v1.0**; Hooks über `Observer` (Integrator)|—|Nicht-Ziel|
|VO (EU) 2024/1689 (KI-Verordnung)|Einordnung je Anwendungsfall|Integrator (O-06); Bausteine liefern Protokollierung und Budgets|—|offen|
|VO (EU) 2024/2847 (Cyber Resilience Act)|Anforderungen an Produkte mit digitalen Elementen, Schwachstellenmanagement|SBOM, `cargo audit`, `deny`, Release-Prozess (§13.2); **Einordnung offen (O-11)**|—|offen|


---

## 10. Bausteine für lokale LLM-Anwendungen (F3, F4, F5)

### 10.1 Port-Verträge der Inferenz

|Port|Vertrag|Pflichtverhalten|
|---|---|---|
|`EmbeddingProvider`|Texte → Vektoren fester Dimension; trägt `ModelIdentity` (Name, Hash, Dimension, Normalisierung)|Index und Modell dürfen nicht vermischt werden (INV-EMBED-1/-2): Abfrage mit fremder `ModelIdentity` ⇒ Ablehnung|
|`LlmTextGenerator`|Prompt → Token-Strom, abbrechbar, mit Budget|Abbruch gibt Mutex/Semaphore/KV-Blöcke sofort frei; Blockierung des Async-Executors verboten (`spawn_blocking`)|
|`Reranker`, `QueryRewriter`|optional|Ausfall ⇒ Rückfall auf unveränderte Ergebnisse, nie Fehler im Suchpfad|

### 10.2 Backends

|Crate|Eignung|Netz|Hinweise|
|---|---|---|---|
|`infer-candle`|vollständig lokal in Rust (GGUF Q4_K_M/Q8_0, CPU; CUDA optional); Basis für KMU-Edge|keiner|KV-Cache je Mandant; Determinismus über festes Seed prüfbar|
|`infer-onnx`|spezialisierte Embedding-Modelle (z. B. BGE/E5-Klasse) über `ort`|**Download in Ist (U-07)**|nur mit lokal bereitgestelltem, prüfsummengesichertem Modell|
|`infer-ollama`|Betrieb gegen vorhandene Ollama-Infrastruktur im Firmennetz|HTTP zum Ollama-Dienst|Standard nur Loopback; andere Hosts per Allowlist|

**Modell-Lieferkette:** Jedes Modell wird durch `ModelIdentity` samt SHA-256 geführt. Das Modul lädt Modelle **nie selbst aus dem Netz** (Standard); Beschaffung ist ein separater, dokumentierter Schritt des Integrators. Die **Lizenz der Modelle** (z. B. Qwen, Llama) ist Sache des Integrators; Contextra weist sie im `MODULE.md` und im SBOM-Anhang als externe Abhängigkeit aus.

### 10.3 KV-Cache (`kvcache`)

Quantisierter KV-Cache mit Eviction. **INV-KVTENANT-1:** Blöcke sind an `TenantId` und `ModelIdentity` gebunden; kein Block wird mandantenübergreifend wiederverwendet (auch nicht über inhaltsadressierte Deduplizierung, solange `content-addressed-kv-cache` nicht mandantengetrennt belegt ist). Puffer werden beim Drop zeroisiert. Löschung kaskadiert (INV-DERIVED-1, Schicht S10). KV-Kryptografie wird aus `crypto` nach `kvcache` verlagert (U-11).

### 10.4 Routing und Adaption (`router`, `adapt`)

Der Router wählt zwischen kleinen lokalen Modellen (z. B. 3B/7B) und größeren (z. B. 14B/32B) nach Kontext, Kosten, Latenz und Qualität (kontextueller Bandit, Drift-Erkennung, konformale Kalibrierung). Verbindlich: (1) **sicherer Fallback** auf das konservativste lokale Modell bei Drift oder Fehler, (2) **Reproduzierbarkeit** (`Rng`-Port, `Clock`-Port), (3) **kein Datenabfluss:** Routing-Ziele sind ausschließlich lokale Provider, außer das Egress-Gateway (§10.6) erlaubt etwas anderes. `adapt` ist ein **eigenständig nutzbarer** Regler (PID, robuste Statistik) und hängt von keinem Speicherkern ab; sein Nutzen MUSS per Benchmark belegt werden, sonst wird er zurückgebaut (EP12).

### 10.5 Agenten und Gedächtnis (`graph`, `cognition`, `agent`)

|Anforderung|Inhalt|Prüfung|
|---|---|---|
|Budgetierung|`TokenBudget`, Zeit- und Schrittbudgets; Überschreitung ⇒ definierter Abbruch|Budget-Race-Tests|
|Wiederanlauf|Checkpoint je Schritt (RAII-Guard), Replay deterministisch, Dead-Letter-Queue|Crash-/Replay-Tests mit `ManualClock`|
|Werkzeugrechte|Werkzeuge nur aus Allowlist je Mandant; keine impliziten Rechte|Negativtests|
|Mandantentrennung|Graph, Sitzungsbäume und Konsolidierung je Mandant (VETO-F10)|Isolationstests T-TEN-*|
|Löschung|Graphkanten, Konsolidierungsprodukte und Sitzungszustände kaskadieren (S5, S9)|Löschtests je Schicht|
|Prompt-Injection|Eingaben aus Dokumenten sind **unvertrauenswürdig**; `mcp` und `privacy` filtern, Agent führt keine Werkzeugaufrufe aus Dokumentinhalt ohne Freigabe aus|Red-Team-Korpus|

`cognition` (Konsolidierung, Sleep-Cycle) ist **Hintergrundarbeit** und läuft nie im Anfragepfad; Abbruch über Cancel-Token (`tokio::select!` mit `biased;`), kein Datenverlust bei Abbruch.

### 10.6 Egress-Tabelle (INV-NET-1, G8)

Standard: **kein ausgehender Verkehr außer Loopback**. Jede Ausnahme steht hier und ist per Konfiguration ausdrücklich freizuschalten.

| Modul                                  | Verkehr                                  | Standard              | Freigabe                                             |
| -------------------------------------- | ---------------------------------------- | --------------------- | ---------------------------------------------------- |
| `infer-ollama`                         | HTTP zu Ollama                           | nur `127.0.0.1`/`::1` | Host-Allowlist in Konfiguration                      |
| `infer-onnx` (Ist: Download)           | HTTPS zu Modell-Host                     | **Soll: aus**         | Feature `model-download` + Prüfsumme; sonst entfernt |
| `privacy` (Cloud Query Egress Gateway) | Anfragen an Cloud-LLM nach PII-Filterung | aus                   | Feature `cloud-egress` + Allowlist + Protokollierung |
| `mcp`                                  | stdio only; kein Listener                | –                     | –                                                    |
| `license`                              | keiner (Offline-Prüfung)                 | –                     | –                                                    |
| alle übrigen                           | keiner                                   | –                     | –                                                    |

Prüfung: Gate `egress-inventory` (statisch: Netz-Crates und `std::net` je Crate gegen diese Tabelle) und Test T-EGR-1 (Laufzeit: Netzwerk-Namespace ohne Route, Modul-Tests müssen bestehen).

### 10.7 Datenschutz-Bausteine (`privacy`, `audit-export`, `avv-generator`)

`privacy` klassifiziert und schwärzt personenbezogene Muster (E-Mail, IBAN, API-Schlüssel, Passwörter, in Planung: deutsche Telefonnummern, Steuer-ID, Kfz-Kennzeichen) vor **jedem** Egress und vor Protokollierung. Es gibt **keine Vollständigkeitszusage**: Falsch-Negativ- und Falsch-Positiv-Rate werden an einem Korpus gemessen und im Evidence-Pack ausgewiesen. `audit-export` erzeugt ein Verzeichnis von Verarbeitungstätigkeiten (Art. 30) aus Konfiguration und Laufzeitdaten, `avv-generator` Vorlagen für Art. 28; beides sind **Entwürfe zur Prüfung durch den Verantwortlichen**.

---

## 11. Kompositionsprofile für KMU

Profile sind **benannte Verdrahtungen** der Fassade (Feature-Sets) und zugleich **Prüfeinheiten**: Für jedes Profil muss `cargo xtask profile-audit <profil>` bestehen (Build, Isolation, Egress, Smoke-Benchmark).

| Profil         | Enthält                                                                                                                     | Zielfälle                                                                       | Transitive Last (Soll) |
| -------------- | --------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | ---------------------: |
| **Search**     | `types`, `ports`, `sys`, `simd`, `mvcc`, `crypto`, `store`, `vector`, `text`, `rank`, `engine`, `db`, `contextra[embedded]` | Speisekarte, Produktkatalog, Intranet-Suche (ohne LLM)                          |                   ≤ 14 |
| **Local-RAG**  | Search + `infer-candle` **oder** `infer-ollama` + `privacy`; optional `mcp`                                                 | Lokaler Produktberater im Shop, Reservierungs-/Auskunftsassistent               |                   ≤ 17 |
| **Agent**      | Local-RAG + `checkpoint`, `graph`, `router`, `agent`, `kvcache`, `adapt`                                                    | Mehrschritt-Workflows (Rückfragen, Terminierung, Sachbearbeitungs-Vorbereitung) |                   ≤ 22 |
| **Compliance** | Search + `checkpoint`, `erasure`, `audit-export`, `avv-generator`                                                           | Kommunen, Jobcenter, Agenturen: Löschung, Nachweis, Verzeichnisse               |                   ≤ 18 |

Für jedes Profil liefert das Repository ein **lauffähiges Beispiel** (`examples/<profil>/`) und eine **deutschsprachige Integrationsanleitung** (Betrieb, Backup, Löschung, Grenzen).

---

## 12. Qualitätssicherung und Gates

### 12.1 Bestand

`governance/gates.toml` führt 58 Gates (Stufen: 39 `pr`, 10 `merge`, 6 `heavy`, 3 `nightly`), darunter `fmt`, `clippy-lintset`, `ring-dag`, `ring0-async`, `capabilities-consistency`, `unsafe-inventory`, `public-api`, `feature-matrix`, `wired`, `security-ports`, `deletion-types-once`, `deletion-honesty`, `doc-truth`, `ratchet`, `determinism`, `test-integrity`, `deny`, `audit`, `sbom`, `miri`, `fuzz-matrix`, `fuzz-regression`, `evidence-complete`. Aktive Phase im Repo: `PH0`. Wirksamkeit und Laufzeiten der Gates sind **nicht gemessen** (§1.2).

### 12.2 Neue Modul-Gates

|Gate|Prüft|Stufe|
|---|---|---|
|`module-isolation`|jedes Crate baut/testet mit `--no-default-features` und nur harten Abhängigkeiten (INV-ISO-1)|pr|
|`transitive-budget`|transitive harte Workspace-Abhängigkeiten ≤ Budget (§4.5), nur sinkend|pr|
|`no-cycles`|keine Zyklen, auch nicht über Dev-Kanten|pr|
|`may-depend-on`|alle Kanten in §4.4 deklariert; Warnung = Fehler|pr|
|`allow-register`|jedes `allow(clippy::panic/unwrap_used/…)` im Register (EP16)|pr|
|`egress-inventory`|Netz-Crates je Crate gegen §10.6|pr|
|`evidence-pack`|Evidence-Pack vollständig und zum Commit passend|merge|
|`claims-consistency`|Berichte, README, Manifest widersprechen sich nicht (EP17)|pr|
|`module-bench`|Benchmark-Budgets je Modul|nightly|
|`ports-conformance`|jede Port-Implementierung besteht die Konformitätstests|pr|
|`surface-coverage`|jede Dateiklasse hat eine Löschschicht (INV-SURFACE-1)|pr|
|`profile-audit`|Profile bauen, laufen, halten Budget|merge|
|`baseline-all-targets`|`cargo check --workspace --all-targets --locked` und `cargo test --workspace --no-run --locked` (ADR-D11)|pr|
|`clippy-baseline`|`cargo clippy -p <crate> --all-targets --locked -- -D warnings` je Mitglied; Ergebnis je Crate im Protokoll|pr|
|`duplicate-types`|kein öffentlicher Typname in zwei Crates ohne Registereintrag mit Frist (A2, W6)|pr|
|`undefined-cfg-feature`|`cfg(feature = "x")` ohne Manifest-Definition ist ein Fehler (U-22); tote Features melden|pr|
|`test-hooks-private`|keine öffentlichen `*_for_test`-Items und keine `Mock*`/`Fake*` in `src/` ohne `cfg(test)`/`test-hooks` (Regel 15)|pr|
|`async-blocking-io`|kein `std::fs`/`std::thread::sleep` in `async fn` ohne `spawn_blocking`/`Vfs` (Regel 16)|pr|
|`file-size`|Produktionsdatei ≤ 800 Zeilen, Ratchet (W1)|pr|
|`measure-single-source`|Kennzahlen in Dokumenten stammen aus `cargo xtask metrics` (ADR-D12)|merge|
|`test-runtime-budget`|`cargo test -p <crate>` ≤ Budget (W10)|nightly|

### 12.3 Pflichtprüfungen nach Crate-Klasse

|Prüfung|Inseln|F1 `[K]`|F2|F3–F5 `[M]`|
|---|:-:|:-:|:-:|:-:|
|V1–V2 Typ/Lints/Unit/Property|✔|✔|✔|✔|
|V3 Differential gegen Referenz|✔|✔|✔|empfohlen|
|V4 Fuzz|`wire`|`store`, `text`|Verifier|`mcp`, `privacy`|
|V5 Miri / loom|Miri|loom (`mvcc`, `store`)|loom (Saga)|loom (`agent`, `graph`)|
|V6 Chaos (`FaultVfs`)|–|✔|✔|`agent` (Checkpoint)|
|V7 Mutation auf Invarianten|✔|✔|✔|–|
|V8 externer Review|–|–|✔ (Pflicht)|–|

### 12.4 Test-Hygiene

Test-Hooks (Panic-Auslöser, Schlüsselzugänge, Fake-Beweise) existieren nur hinter `cfg(test)` oder `test-hooks`/`test-utils`; diese Features liegen in keinem Release- oder Default-Set (Gate `test-integrity`). Testcode darf `unwrap`/`expect` nutzen, **Produktionscode nicht** (Gate `panic-inventory`, getrennt nach Produktions- und Testcode).

### 12.5 Pflichttests dieser Fassung (Ergänzung zu Anhang C-17.5)

Die Pflichttests T-DEL-1 bis T-DEL-9, T-TEN-1/2, T-REC-2, T-SSI-1, T-EMB-1, T-DET-1 stehen in Anhang C-17.5. Diese Fassung ergänzt:

|ID|Garantie / Invariante|Inhalt|Art|
|---|---|---|---|
|T-DEL-15|G7|Lösch-Mutationskatalog (Anhang C-17.11, erweitert um: Lesepfad ohne Sperrprüfung, Restore überschreibt `ErasedSet`): jede Injektion lässt mindestens einen Test scheitern|Mutation|
|T-ISO-1|G9, INV-ISO-1|Jedes Crate baut und testet mit `--no-default-features`, nur mit seinen hart **und per Dev-Kante** deklarierten Abhängigkeiten, ohne Netz|Build + Test|
|T-EGR-1|G8, INV-NET-1|Alle Modul-Tests bestehen in einem Netzwerk-Namespace **ohne Route** (nur Loopback); ein Modul mit Netzzugriff außerhalb §10.6 scheitert|Laufzeit|
|T-KV-1|INV-KVTENANT-1|Zwei Mandanten mit identischem Prompt teilen keinen KV-Block; Mandantenende zeroisiert die Blöcke (Prozessspeichersonde)|Property + Sonde|
|T-MOD-1|M8, INV-CLAIMS-1|`claims-consistency`: ein Bericht mit höherer Reife als `capabilities.toml` bricht den Build|Gate-Meta-Test|

---

## 13. Betrieb, Lieferkette, Release

### 13.1 Referenzhardware und Betriebsmodell

Referenzgerät KMU-Mini-PC (8 Kerne, 32 GB RAM, NVMe, ohne GPU). Ein Primärprozess hält ein Datenverzeichnis; weitere lokale Prozesse sprechen über IPC mit ihm. Logs ausschließlich nach `stderr`/Datei, **keine Telemetrie nach außen**. `doctor` prüft Datenverzeichnis, Formate, Schlüsselzustand und Egress-Konfiguration.

### 13.2 Lieferkette

`cargo deny` (Lizenzen, Advisories, Quellen), `cargo audit`, SBOM je Crate, reproduzierbarer Build, gepinnte Toolchain (1.89.0), `Cargo.lock` versioniert, Aktionen in CI auf Commit-Hash gepinnt. Lizenz: MIT OR Apache-2.0. Abhängigkeiten mit nicht kompatibler Lizenz sind verboten. Bei ML-Modellen: Lizenz und Herkunft im SBOM-Anhang.

### 13.3 Versionierung und Release je Modul

Jedes Crate hat eine eigene SemVer-Version; `cargo-semver-checks` ist Pflicht für `[K]`. Ein Modul wird freigegeben, wenn (a) Evidence-Pack vollständig, (b) alle Pflicht-Gates grün, (c) `MODULE.md` aktuell, (d) Änderungsprotokoll vorhanden. Ein Release des Gesamtprodukts („Contextra 1.0“) setzt voraus, dass jedes **im Profil enthaltene** Modul seine Zielreife erreicht hat.

### 13.4 Release-Kriterien v1.0

|Kriterium|Nachweis|
|---|---|
|Baseline grün (Build, Test, Clippy) auf CI für **alle** 35 Mitglieder|WP 0.1|
|`cargo check --workspace --all-targets --locked` und `cargo clippy --workspace --all-targets --locked -- -D warnings` grün; kein Timeout in `cargo test -p <c>`|WP 0.5, 0.6, 0.7 (ADR-D11)|
|Keine Doppeldefinition öffentlicher Typen ohne Registereintrag; keine Test-Hooks in der Produktions-API|`duplicate-types`, `test-hooks-private`|
|Rechtsmatrix (§9.12) vollständig, jede Zeile mit Nachweis oder ausgewiesenem Nicht-Ziel|WP 5.10|
|Kein Produktions-Panic und kein unregistriertes `allow`|`panic-inventory`, `allow-register`|
|Soll-Graph (§4.5) erreicht, keine Zyklen|`transitive-budget`, `no-cycles`|
|Egress-Tabelle eingehalten, ONNX-Download entfernt oder freigabepflichtig|`egress-inventory`, T-EGR-1|
|Evidence-Pack je Modul|`evidence-pack`|
|Profile Search, Local-RAG, Compliance bestehen `profile-audit`; Agent mindestens `beta`|`profile-audit`|
|Löschkette mindestens `beta` mit Chaos- und Mutationsnachweis (§9.10)|AC-Matrix|
|Dokumentation (deutsch) und Beispiele je Profil|`doc-truth`|
|Externer Review der Krypto- und Löschbausteine (für `certified-ready`)|Review-Bericht|

---

## 14. Datenlebenszyklus (Kurzfassung)

`remember` (Chunking → Embedding → WAL-Commit → Index) · `recall` (Planner → ACL-Filter vor Distanz → Hybrid-Suche → RRF → Provenienz) · `forget` (Saga, §9.6) · Checkpoint/Backup (§9.7) · Mandantenende (Löschung aller Scopes und Schlüssel). Details: Anhang C.

---

## 15. Entscheidungen dieser Fassung (ADR-D)

|ID|Entscheidung|Begründung|Ersetzt|
|---|---|---|---|
|ADR-D01|Graph, Kognition, Router, Agent, KV-Cache, ONNX, Ollama, Adapt sind **aktive Module**, nicht Archiv|Zielbild „LLM-Baukasten“ braucht sie; sie liegen bereits in den `default-members`|E-Archiv (2.4 §2.3)|
|ADR-D02|Kern ist nicht geschlossen; **Aufnahme in den Baukasten** über Modulvertrag und Evidence-Pack|Wachstum ohne Qualitätsverlust|2.4 §4.8|
|ADR-D03|Entkopplung über Ports; Rand-Crates kennen `engine`/`db`/`store` nicht|isolierte Prüfbarkeit (M1–M2)|2.4 Kantenregeln|
|ADR-D04|Neues Crate `erasure` (Ring 3) für Saga und Berichtsbildung|Löschkette einzeln prüfbar; `engine` schrumpft|2.4 §12.13|
|ADR-D05|Domänen-Ports (Graph, KV) wandern in besitzende Crates|`ports` bleibt klein und stabil|–|
|ADR-D06|`contextra-core` entfällt (deprecate → migrieren → entfernen)|zweite Fassade ohne Eigenwert|–|
|ADR-D07|Netz-Standard „aus“; Egress-Tabelle|Datenschutz als Bauprinzip|–|
|ADR-D08|Reife nur aus Evidence-Pack; Audit-Berichte sind Hypothesen|U-01, U-02|–|
|ADR-D09|`xtask` wird nach Wirksamkeit inventarisiert und zurückgebaut|U-12|–|
|ADR-D10|`tokio` ohne `full`; Features je Crate minimal|Bauzeit, Angriffsfläche|–|
|ADR-D11|**Baseline-Definition:** grün heißt `check`, `clippy -D warnings` und `test` je Crate **und** `--workspace --all-targets`; ein Timeout ist **rot**|U-15 bis U-17|–|
|ADR-D12|Kennzahlen stammen aus **einer** AST-Messung (`cargo xtask metrics`); Handzählungen, Regex-Zähler und Altwerte sind historisch|U-18|–|
|ADR-D13|Eine Typfamilie je Konzept wird per Gate `duplicate-types` erzwungen; Duplikate werden zusammengeführt oder umbenannt|U-20, A2|–|
|ADR-D14|Test-Hooks, Mocks und Fake-Beweise gehören nie in die öffentliche Produktions-API|U-24|–|
|ADR-D15|Löschkette: Orchestrierung in `erasure`, Messung in den besitzenden Crates, Berichtsformat in `crypto`, Verdrahtung nur in der Fassade; Algorithmus-IDs in Formaten|§9.11, W5|ADR-D04 (ergänzt)|
|ADR-D16|Wartbarkeitsregeln W1–W11 als Gates mit Ratchet statt Sammelumbau|§4.9|–|

---

## 16. Offene Punkte

|ID|Punkt|Zuständigkeit|
|---|---|---|
|O-01|Rechtliche Einordnung von Embeddings als personenbezogene Daten (vorsorglich ja) und von Backups (Aufsichtsauffassungen)|externe Rechtsprüfung|
|O-02|Aufbewahrungsfristen und Legal-Hold-Katalog für Zielbranchen|externe Rechtsprüfung|
|O-03|Marktprüfung der Hypothesen aus §2.1 (Gastronomie, Shop, Kommune)|Produkt|
|O-04|Medien-/Datenträgerlöschung nach anerkannten Normen ist Betreiberpflicht; Abgrenzung im Bericht|Betreiber/Recht|
|O-05|Juristische Prüfung der AVV-Vorlagen|externe Rechtsprüfung|
|O-06|Einordnung nach EU-KI-Verordnung (VO (EU) 2024/1689) für Integratoren je Anwendungsfall|Integrator/Recht|
|O-07|Hardware-Referenzklassen und Modellgrößen für Benchmarks festschreiben|Technik|
|O-08|Welche `xtask`-Prüfungen sind wirksam (Mutations-/Selbsttest)?|WP 3.6|
|O-09|Notwendigkeit von DiskANN/RaBitQ im Designpunkt ≤ 10⁷ Vektoren|Benchmark|
|O-10|macOS und Windows als unterstützte Entwicklungs-/Edge-Plattformen|Technik|
|O-11|Einordnung der Bibliothek und der Integrator-Produkte nach VO (EU) 2024/2847 (Cyber Resilience Act): Pflichten, Meldewege, Open-Source-Besonderheiten|externe Rechtsprüfung|
|O-12|Reproduzierbarkeit der Baseline: Commit 8f8fa7cd im flachen Klon nicht auflösbar; Zielstand für die erneute Baseline festlegen|Technik (WP 0.1)|
|O-13|Ursache der 33 Clippy-Fehlschläge (gemeinsamer Abbruch oder echte Lints) anhand der rohen Logs|Technik (WP 0.6)|
|O-14|Welche Tests verursachen die 13 Timeouts (Fixture, echte Last, Deadlock)?|Technik (WP 0.7)|
|O-15|Dokumentationssprache: `rustdoc` englisch oder deutsch; Integrationsleitfäden deutsch|Produkt (E-3.8)|

---

## 17. Offene Spezifikationsfragen (Dokumentationsabgleich 28.10.2026, Fassung 3.1)

### 17.1 Frühere Fragen (aus 3.0): Stand nach Prüfung des Klons

|Thema|Frühere Aussage|Befund am Klon HEAD 21a09ad (E1) und Baseline|Ergebnis|
|---|---|---|---|
|`contextra-sys`: `pub mod crypto_rng;` ohne Datei|`cargo check --workspace` scheitere daran|`sys/src/lib.rs` enthält keine Deklaration `crypto_rng`, die Datei fehlt ebenfalls; Baseline: `cargo check -p contextra-sys` Exit 0, Clippy und Test grün|**erledigt/nicht mehr zutreffend**|
|`capabilities.toml` vs. Ring-Modell (`graph`, `adapt` in Ring 0)|angeblicher Widerspruch zu §4.1|§4.1 führt `graph` und `adapt` **ebenfalls** in Ring 0; kein Widerspruch. Echte Lücke: `may_depend_on` unvollständig (68 Scan-Warnungen) und veraltet (`contextra-core`)|**umformuliert:** WP 0.2, 2.1|
|`xtask`: doppelter Match-Arm `check-commit-diff-integrity`|doppelte Deklaration|Im Klon genau **0** Treffer in `xtask/src/main.rs`; Baseline: `xtask` check Exit 0|**erledigt/nicht reproduzierbar**|

### 17.2 Neue Fragen

|ID|Thema|Befund und Fragestellung|Vorschlag|
|---|---|---|---|
|F-01|`unsafe` in Testcode und Zählung in `xtask`|Test-Dateien in `graph` und `text` (Allocator-Profiler) enthalten `unsafe`; die Metrik zählt 39 `unsafe` in `xtask`. §4.6 verbietet `unsafe` außerhalb der Inseln, unterscheidet aber nicht zwischen Produktions- und Testcode|Regel präzisieren: Produktion nur Inseln; Testcode nur in `tests/` mit `// SAFETY:` und Eintrag in einem Register; `xtask` per AST messen|
|F-02|Commit-Stände|Baseline (8f8fa7cd), Scan (52d3227), Klon (21a09ad) unterscheiden sich; Zahlen sind nicht addierbar|WP 0.1 erzeugt **eine** Baseline auf dem Zielstand und archiviert rohe Logs|
|F-03|Scan-Skript gegen alte Spezifikation|`contextra_scan.py` prüft gegen Spezifikationsnummern von 2.x (z. B. „Spec §4.9.4“, „§7.1“) und liest Governance-Dateien an alten Pfaden; ohne Pfadkorrektur sind Gates „FEHLT“|Skript nicht als Gate verwenden; seine Prüfaufträge (U-20 bis U-25) werden durch AST-Gates in `xtask` ersetzt (WP 0.4, 3.6)|
|F-04|Gate-Wirksamkeit|Scan meldet 7 blockierende Gates rot (u. a. `determinism` 44, `async-io` 15, `unsafe-audit` 3), während Baseline/`gates.toml` 58 Gates führt; Überlappung und Wirksamkeit unbekannt|WP 3.6 (Meta-Test je Gate)|
|F-05|`KmsProvider` und Löschkette|Trait liegt in `crypto/wal_crypto.rs`, laut Scan ohne produktive Verdrahtung; §6.2 ordnet `KmsProvider` dem Sicherheits-Port-Katalog zu|Besitz klären (`ports` oder `crypto`), Verdrahtung in WP 5.3|

---

## Anhang A. Abbildung 2.4 → 3.0

|2.4|3.0|
|---|---|
|§0 LLM-Schnellstart|§0|
|§1 Geltung/Belegbasis|§1 (neu belegt, Befunde U-01–U-12)|
|§2 Produkt|§2 (Baukasten, M1–M8, G8/G9)|
|§3 Verfassung|§3 (+EP16/EP17, VETO-NET1, INV-NET-1/ISO-1/KVTENANT-1)|
|§4 Workspace-Architektur|§4 (Ringe, Modulvertrag, Soll-Graph), §5 (Katalog)|
|§5 Ports|§6|
|§3 Verfassung (Vollregister)|§3 (Auszug) + Anhang C (C-3)|
|§6 Nebenläufigkeit, §7 Persistenz, §8 MVCC|§8 (Rahmen) + Anhang C (C-6, C-7, C-8)|
|§9 Vektor, §10 Text, §11 Engine/Abfragepfad|§5.3 (Steckbriefe) + Anhang C (C-9 bis C-11)|
|§12 Löschung|§9 (Baustein) + Anhang C (C-12)|
|§13 Schnittstellen, §14 Konfiguration, §15 Startwerte|§10.6, §10.7, §11, §13 + Anhang C (C-13, C-15)|
|§16/§17 Benchmarks, QS|§7, §12 + Anhang C (C-17.4, C-17.5, C-17.8, C-17.11)|
|§18 Governance|§12, §13, §15|
|§24 Datenlebenszyklus|§14 + Anhang C (C-24)|
|§19–§26 Kataloge|ersetzt durch §1.3, §15, §16; REQ-Katalog wird in WP 0.3 nach `docs/spec/requirements.toml` überführt|

## Anhang B. Änderungsverlauf

|Fassung|Datum|Änderung|
|---|---|---|
|2.4|27.10.2026|Löschpfad optimiert, Quellenbereinigung|
|**3.0**|**28.10.2026**|Produktzuschnitt auf entkoppelten LLM-Baukasten; Modulvertrag und isolierte Prüfbarkeit; Befunde U-01–U-12 aus dem Abgleich Audit ↔ Code; ADR-D01–D10; Löschkette als Baustein (§9); Aufhebung der Archivierungsentscheidung; Nachmessung am HEAD 52d3227f0 (U-13, U-14, korrigierte Zahlen); Katalogtabelle §5.2; Pflichttests §12.5; Anhang C vollständig aus dem Repo-Stand übernommen|
|**3.1**|**28.10.2026**|Baseline (E2, teilweise), Metrik-Datei und Scan eingearbeitet (§1.2a, U-15 bis U-27); Projektziele mit Messkriterien (§2.5); Regeln 15–16; Wartbarkeitsregeln W1–W11 (§4.9); `erasure` in §4.4/4.5; Crate-Review aller 35 Mitglieder (§5.5); Schnittstellenvertrag und Rechtsmatrix der Löschkette (§9.11, §9.12); neue Gates (§12.2); Release-Kriterien (§13.4); ADR-D11–D16; O-11–O-15; §17 neu gefasst (drei Altfragen geprüft: erledigt bzw. umformuliert)|

## Anhang C. Übernommener Detailtext aus Fassung 2.4/2.5 (nachrangig)

**Herkunft.** Textauszug aus dem Repo-Stand `docs/SPEC.md` (Fassung 2.5, auf 2.4 aufbauend; SHA-256 der Quelldatei beginnt mit `b6eeb29d3566b7f9`, HEAD 52d3227f0), **textlich unverändert**; geändert wurden nur die Überschriftsnummern (`## 12.` wird `### C-12.`). Enthalten sind die Abschnitte: C-3 Verfassung (Vollregister der Invarianten), C-6 Nebenläufigkeit, C-7 Persistenz, C-8 MVCC, C-9 Vektor, C-10 Text, C-11 Engine und Abfragepfad, C-12 Kryptografie/Löschung/Bericht/Audit (inkl. Abnahmekriterien AC-1 ff.), C-13 Schnittstellen, C-15 Startwerte, C-17.4/17.5/17.8/17.11 (Chaos, Pflichttests T-*, Orakel, Lösch-Mutationskatalog), C-24 Datenlebenszyklus. **Nicht übernommen** (durch §1 bis §16 ersetzt oder überholt): Produkt-/Workspace-Kapitel der Vorfassung (§2, §4, §5), Gate-Katalog (§17.3), Anforderungskatalog (§19, wandert nach `docs/spec/requirements.toml`, WP 0.3), Ist-Befunde und Phasen (§21, §22), Archiv (Anhang B der Vorfassung).

**Lesehinweise (verbindlich).**

1. `§`-Verweise **innerhalb** dieses Anhangs meinen die Nummerierung der Vorfassung (hier „C-§“); Verweise außerhalb des Anhangs meinen §1 bis §16.
2. **Bei Widerspruch gelten §1 bis §16.** Insbesondere gelten die folgenden Aussagen des Anhangs **nicht mehr**: (a) „Archiv“, `archiv/v0`, Klassen `[R]`/`[A]` und „Rest ins Archiv“ (ersetzt durch `[M]`, ADR-D01); (b) „Graph und `KvCache` sind `FeatureDisabled`“ und „acht von zehn Schichten gemessen“ (jetzt aktive Schichten S9/S10, §9.4; `FeatureDisabled` nur mit Startbeleg); (c) „Kern ist geschlossen“ und die Crate-Zeilenbudgets (ADR-D02; Budgets gelten nur noch als Ratchet aus §4.5); (d) Kantenregeln der Vorfassung (ersetzt durch §4.4); (e) Stub-Befunde (am HEAD nicht mehr vorhanden, U-04); (f) Reifegrad-Definition C-17.6 (ersetzt durch §1.4 und §7.5).
3. Löschspezifikation C-12 bleibt **inhaltlich maßgeblich** für Schlüsselhierarchie, Destroy-Protokoll, Saga, Bericht V5, Lesesperre, Backup, Fristen und Ausschlüsse; §9 ordnet sie dem Baustein „Löschkette“ zu und legt Besitzer, Ist-Stand und Arbeitspakete fest.
4. Nach WP 0.3 werden die Texte in `crates/<c>/MODULE.md` und Fachkapitel überführt und hier gestrichen.

---

### C-3. Verfassung

#### C-3.1 Vetos (unverhandelbar)

- **VETO-F10 – Keine mandantenübergreifenden Datenflüsse.** Kein Cache, Index (auch keine Kante innerhalb eines Index), Embedding, Statistik (z. B. IDF) oder Audit-Eintrag überschreitet Tenant-Grenzen. Der Tenant wird aus der Transaktion abgeleitet, nie aus Nutzerparametern. Status: `permanent_rejected`.
- **VETO-F02 – HNSW-Löschung nur per Tombstone + Slot-Nullung + Rebuild**, **kein** Rewiring der Nachbarn. Status: bestätigt (Review-Frist 2026-10-07 war abgelaufen; Bestätigung in WP 0.3).
- **VETO-OP03 – Kein Echtzeit-Audio/STT** im Kern. Status: geschlossen (kein betroffener Code, WP 0.3).
- **Single-Node-Doktrin** – kein Sharding, kein P2P, keine verteilte Replikation.

Vetos und Invarianten sind maschinenlesbar in der Verfassungsdatei des Repos versioniert (je ID); Änderungen brauchen einen ADR mit Begründung, Alternativen und signierter Freigabe (§18.6).

#### C-3.2 Engineering-Prinzipien EP1–EP15

1. **Fail-closed**, kein Prozessabbruch durch erwartbare Fehler; I/O-Fehler sind typisierte Fehler, nie SIGBUS/Abort (INV-IO-1).
2. **Messbare Testtiefe:** jede `INV-*` hat einen Test, der ihre Verletzung erkennt (Mutation belegt Wirksamkeit); Schwellen stehen in `governance/ratchet.toml` und dürfen nur besser werden. Tests werden aus Spezifikation und Invarianten abgeleitet, nicht aus der Implementierung desselben Erzeugers.
3. **Null technische Schuld; Wired-or-it-doesn't-exist (P29):** Eine Fähigkeit existiert nur, wenn die Composition Root sie verdrahtet **und** ein Integrationstest sie über den echten Pfad beweist.
4. **Benchmarking mit fremder Baseline:** jede veröffentlichte Zahl nennt Commit, Modell-Hash, Hardware, Workload, Vergleichsbasis (§16).
5. **Atomare Zyklen:** ein Zyklus = ein Rot→Grün-Schritt, ein Scope, ein verifizierter Commit; Merges serialisiert über eine Merge-Queue auf dem **Merge-Ergebnis** (§18.2).
6. **Jepsen-artiges Chaos-Testing** mit `FaultVfs` (§17.4).
7. **Deterministische Verdrahtung:** nur die Fassade instanziiert und verbindet; keine globalen Singletons.
8. **IoC:** Merge, Metrik, Analyzer, Observer, Embedder, Ranker sind Ports mit Registrierung.
9. **Automatisierte Audits sind selbst Testobjekte** (Meta-Gate, §17.3).
10. **Absolute Verifizierbarkeit:** jede Behauptung (gelöscht, berechtigt, unverändert) ist belegt oder ausdrücklich als nicht belegt deklariert.
11. **Compiler vor Heuristik:** was `rustc`, `clippy`, `cargo metadata`, `cargo public-api` prüfen können, prüfen sie; Text-/Regex-Audits ergänzen nur.
12. **Bauen oder kaufen – belegt:** Eigenbau eines Standardbausteins (LSM, HNSW, BM25, Block-Cache, SIMD-Kernel, MCP-Protokollschicht) bleibt nur bei belegtem Vorteil in Kerneigenschaften (Löschmessung, Determinismus, Mandantentrennung) oder Leistung gegenüber der besten fremden Implementierung hinter demselben Port. Kryptografische Primitive werden nie selbst gebaut. Fremdimplementierungen dienen auch dort als Orakel im Test, wo sie als Abhängigkeit nicht taugen.
13. **Evidenz vor Behauptung:** maßgeblich ist maschinell erzeugte, wiederholbare Evidenz (Befehl, Toolchain, Commit, Exit-Code, Dauer, Ausführungszahlen, Hash); ein nicht ausgeführtes Prüfwerkzeug ist **rot**, nie „bedingt grün“.
14. **Tiefe vor Breite:** der Kern ist geschlossen (§4.8); eine neue Fähigkeit betritt ihn erst, wenn die Evidenz des bestehenden Kerns die Zielstufe erreicht hat.
15. **Ein unabhängiges Orakel je Kerngarantie** (RB1–RB4): Referenzmodell, Brute-Force, Canary-Scanner; nicht aus denselben Datenstrukturen und nicht vom selben Erzeuger (INV-ORACLE-1, §17.8).

#### C-3.3 Architekturprinzipien

**Prinzipien P** (Nummern stabil, Lücken gewollt):

|ID|Prinzip|
|---|---|
|P1|Ring-Disziplin: Kanten (normal/build) zeigen auf gleichen oder niedrigeren Ring.|
|P2|`unsafe` nur in drei Inseln (`simd`, `sys`, `wire`) im eigenen Code; jeder handgeschriebene Block trägt `// SAFETY:` und ist durch Test/Miri gedeckt. Generierter Code nur in `@generated`-Dateien, Generator und Version gepinnt, Ausgabe in der CI reproduzierbar geprüft, nur über sichere Adapter genutzt, von Fuzz/Miri erfasst. Edition-2024-Attribute `#[unsafe(no_mangle)]`/`#[unsafe(export_name)]`/`#[unsafe(link_section)]` zählen als `unsafe`. Native Drittabhängigkeiten stehen im Supply-Chain-Inventar (§18.4).|
|P5|DAG-Integrität: keine Aufwärtskanten, keine Zyklen (normal+build, **mit aufgelösten Features**); Dev-Aufwärtskanten nur als in `capabilities.toml` deklarierte Ausnahme.|
|P7|Zero-Panic in Produktion (Liste in §0.2 Regel 2, zusätzlich unchecked Slicing); Lint-Satz in §17.3 auf `deny`. Ergebnisse von `Mutex::lock()`/`RwLock::*` (Poisoning) sind typisierte Fehler mit definiertem Gesundheitsübergang (§7.8). Ratchet-Baselines dürfen nur sinken; Ziel 0, ein Ratchet ist kein Beleg für „null“. Release-Profil: `overflow-checks = true` für Kern-Crates. FFI-Builds (`py`): `panic = "unwind"` mit Abfangen an der Grenze; Binaries: `panic = "abort"`.|
|P24|Aller **Anwendungszustand** (Dokumente, Metadaten, Indexabsichten, Lineage, Retention-Entscheidungen) geht durch das WAL. Die einzigen weiteren persistenten Zustände sind die Sidecar-Logs (§7.9, INV-SIDECAR-1).|
|P26|Sync-Kern, Async-Schale: Ring-0-Crates führen **keine Async-Runtime** (`tokio`, `async-std`) als Produktions-/Build-Abhängigkeit und blockieren nie auf einer Runtime; Algorithmen sind synchron. Port-Traits dürfen Futures zurückgeben. Umsetzung in v1.0 als Ratchet (§6).|
|P28|Determinismus: Zeit, Zufall, IDs aus `Clock`, `Rng`, `IdGen`; `SystemTime::now`, `Instant::now`, `thread_rng`, `HashMap/HashSet` (RandomState) in Kernlogik per `clippy::disallowed_*` verboten; Ausnahme: Schlüssel-/Salt-Material.|
|P29|Wired-or-it-doesn't-exist (EP3).|
|P30|Kern-Isolation: `[K]`-Crates hängen weder normal noch build (im Release-Default-Feature-Set) von `[R]`/`[E]`/`[A]` ab; Rand-Fähigkeiten erreichen den Kern nur über Ports, die die Fassade injiziert.|

**Architekturregeln A1–A10 (dieser Fassung):**

|ID|Regel|Durchsetzung|
|---|---|---|
|A1|**Sicherheit ist kein Feature.** Messung, Verschlüsselung und Löschbericht sind in jedem Build aktiv; kein `cfg(not(feature))`-Modul ersetzt einen Typ aus `crypto`.|Gate `no-security-stub`|
|A2|**Eine Typfamilie je Konzept.** Berichts-/Beweistypen existieren nur in `contextra-crypto`.|Gate (REQ-PRF-4)|
|A3|**Messen vor Behaupten.** `LayerWitness` hat keinen öffentlichen Konstruktor außerhalb der Scanner der besitzenden Crates.|Typsystem + Gate|
|A4|**Kern kennt keinen Rand (P30), aber auch keinen Plugin-Typ.** Verbindung nur über Ports und den Composition Root.|`ring-layering` strikt mit `may_depend_on` (§4.4)|
|A5|**Kein Default ohne Absicht.** Schlüssel, Mandant, Zeit, Zufall und IDs kommen aus `RequestContext` bzw. Ports; kein Env-/Literal-Fallback.|Gate `determinism` (Ratchet), `disallowed-methods`|
|A6|**Ein Fehlerbaum.** Alle Crates münden in `ContextraError`; FFI und MCP mappen auf stabile Codes (§13.6).|Gate `error-domain`|
|A7|**Kleine öffentliche Oberfläche.** `pub` nur für dokumentierte API, sonst `pub(crate)`.|`cargo public-api`-Snapshot|
|A8|**Jede Invariante hat einen Test oder wird gestrichen** (Tabelle INV → Testname).|Gate `inv-coverage` (v1.0: Invarianten zu G1–G6)|
|A9|**Ein Dokument je Thema.** Normatives steht nur in `docs/spec/SPEC.md`.|INV-DOC-SINGLE-SOURCE-1, Gate `doc-stamp`|
|A10|**Keine neue Fähigkeit vor dem zugehörigen Test.**|Review, Phasenabnahme|

#### C-3.4 Invariantenregister (einzige Tabelle; jede `INV-*` einmal definiert)

Spalten: ID · Klasse · Invariante · Mechanismus (Kürzel §1.4) · Anforderung (§19). `INV-SHRED-1` der Fassung 1.3 bleibt in dieser Form entfallen; seine Funktion übernehmen `INV-DEL-ORDER-1`, `INV-NO-RESURRECTION-1`, `INV-BACKUP-SPLIT-1` und `INV-CLASS-MEASURED-1` (E44, §26.2). Archivierte Invarianten `[A]` stehen in Anhang B.

|ID|Kl.|Invariante|Mech.|REQ|
|---|---|---|---|---|
|INV-CORE-ISOLATION-1|K|Umsetzung von P30: Gate über `cargo metadata` (aufgelöst, **inkl. Default-Features der Kern-Crates**) lehnt jede Kante `[K] → [R]/[E]/[A]` ab; Features dieser Klassen sind im Release-Default aus.|G|REQ-ARCH-003|
|INV-KMS-1|K|Jeder Schlüssel stammt von einem `KmsProvider`; kein Modul hält Master-Material außerhalb des Providers; mindestens die lokale Implementierung ist verdrahtet. Ableitung direkt aus Passphrase mit festem/globalem Salt (Muster `CryptoKey::try_new(passphrase, konstanter_salt)`) ist außerhalb des Keystore-Pfads verboten.|G+I|REQ-KEY-001|
|INV-DELETION-WIRE-1|K|`forget` löst **alle zehn** Schichten über die Composition Root aus: acht werden gemessen, `Graph` und `KvCache` sind `FeatureDisabled` (Start belegt „keine Daten dieses Typs“); externe Sicherungen stehen als Ausschluss im Bericht (§12.4, §12.20). Ein Bericht mit Klasse P oder höher entsteht nur, wenn jede Schicht belegt oder ausgewiesen ist (Typestate; Auslassen = Kompilierfehler).|T+I|REQ-DEL-001|
|INV-SSI-ACTIVE-1|K|SSI ist im Produktionspfad aktiv; Read-Sets inkl. Prefix-/Range-Phantomschutz werden bei jedem Commit validiert.|P+C+L|REQ-MVCC-001|
|INV-INDEX-SNAPSHOT-1|K|Jeder Indexeintrag trägt `born_seq`/`died_seq`; Suchen am Snapshot `S` sehen nur `born_seq ≤ S < died_seq`; `as_of` gilt für Indexe nur bis zum Retention-Horizont der Tombstones.|P+C+L|REQ-MVCC-002|
|INV-DOC-SINGLE-SOURCE-1|K|Jede Tatsache (Konstante, Ring, Reifegrad, Format, ID) hat genau eine kanonische Quelle (`capabilities.toml`, Verfassungsdatei, Code); Abgeleitetes wird generiert oder per Doc-Truth-Gate geprüft; doppelt vergebene IDs sind Gate-Fehler.|G|REQ-GOV-004|
|INV-RING0-DETERMINISM-1|K|Ring 0 ist deterministisch: kein I/O außerhalb der Ports (insb. `Vfs`), keine Uhr/kein Zufall ohne Port, kein Ergebnis hängt von Hash-Iterationsreihenfolge, Thread-Scheduling oder unfixierter paralleler Float-Reduktion ab.|G+P|REQ-ARCH-015|
|INV-NONDET-1|K|Ergebnisse nichtdeterministischer Quellen (Embedding-Inferenz, LLM-Ausgabe, Zufall) werden als **Werte** ins WAL geschrieben; Recovery und Index-Rebuild lesen sie dort und rufen die Quelle nie erneut auf.|P+G|REQ-DUR-012|
|INV-WAL-SSOT-1|K|Jeder persistente **Anwendungszustand** ist aus WAL + Manifest rekonstruierbar; Indexe sind abgeleitet. Sicherheitszustand, der nicht aus dem WAL ableitbar ist (weil WAL-Scrub ihn entfernt), liegt ausschließlich in den Sidecar-Logs.|C+P|REQ-DUR-016|
|INV-ATOMIC-IDX-1|K|Metadaten- und Indexänderung gehören zu **einem** WAL-Batch; Recovery wiederholt Index-Intents idempotent.|C+P|REQ-DUR-011|
|INV-INDEX-APPLY-1|K|Indexe werden von **einem** Applier strikt in WAL-Reihenfolge angewendet; HNSW-Level folgen aus `hash(doc_id, seed)`, nie aus einem Laufzeit-RNG. Parallelität nur bei Rebuild mit deterministischer Partitionierung.|C+P|REQ-DUR-011|
|INV-TENANT-1|K|Tenant-Ableitung aus der Transaktion; jede Schlüsselkonstruktion enthält das Tenant-Präfix.|T+P+F+G|REQ-TEN-001|
|INV-TENANT-IDX-1|K|Vektor- und Text-Indexe sind **physisch je (Tenant, Collection[, Modell-Hash])** getrennt; keine tenantübergreifende Kante, Posting-Liste oder Statistik.|T+P+F+G|REQ-IDX-001|
|INV-KEYSPACE-1|K|LSM-Schlüssel, Index-IDs und Dateinamen enthalten nur opake IDs/Hashes, nie personenbezogene Inhalte.|T+P+F+G|REQ-IDX-010|
|INV-DERIVED-1|K|`forget` kaskadiert über die Lineage auf Ableitungen (Chunk-Präfixe, Zusammenfassungen, Konsolidierungsfakten, Embeddings-Caches); Ableitungen mit weiteren Quellen werden neu abgeleitet oder als `stale` markiert, nie stehen gelassen.|I+P|REQ-DEL-007|
|INV-ENCRYPT-REST-1|K|Nutzdaten sind at-rest AEAD-verschlüsselt in zwei Ebenen (§12.2): Engine-Schlüssel `K_eng` (Struktur/Integrität) und DocDek/ArtKey (Nutzwerte, §12.2). Verschlüsselung ist Standard; ohne sie weist `doctor` rot aus; nur `EdgeMinimal`/Entwicklung.|I+G+P|REQ-KEY-003|
|INV-ENCRYPT-REST-2|K|Per `mmap` zero-copy gelesene Dateien sind nur zulässig, wenn `doctor` ein verschlüsseltes Volume nachweist; sonst `pread`/Page-Decrypt-Cache.|I+G+P|REQ-KEY-004|
|INV-FAILCLOSED-1|K|Unklarer Zustand ⇒ Ablehnung, nie Raten.|C+I|REQ-ARCH-016|
|INV-KDF-1|K|Passphrasen-Schlüssel über Argon2id mit zufälligem, persistiertem Salt und persistierten Parametern (`KdfHeader`; m ≥ 64 MiB, t ≥ 3, Kalibrierung ≥ 250 ms). Reines HKDF auf eine Passphrase und feste/globale Salts sind verboten. Passphrase nie ausschließlich per Umgebungsvariable; Keyfile, OS-Keyring oder interaktive Eingabe stehen bereit.|I+G+P|REQ-KEY-002|
|INV-FSYNC-1|K|Schlägt `fsync`/`fdatasync` fehl, ist der Handle unbestimmt: kein Ack, kein Retry auf demselben Handle, WAL-Handle wird vergiftet (`Poisoned`), Weiterbetrieb erst nach Neustart mit Recovery-Replay.|C|REQ-DUR-002|
|INV-DURABILITY-PRIM-1|K|Der Commit-Punkt nutzt die plattformgerechte Durability-Primitive (§7.7); „fsync“ ohne diese ist kein Commit.|C|REQ-DUR-001|
|INV-ENOSPC-1|K|Volles Medium hinterlässt keinen Phantom-Eintrag und keinen halb angewendeten Batch; WAL-Segmente werden vorab reserviert, sodass `ENOSPC` bei Rotation statt im Record auftritt; Wechsel in einen definierten lesenden Zustand, retrybarer Fehler.|C|REQ-DUR-003|
|INV-IO-1|K|Lesepfade geben I/O-Fehler als typisierte Fehler zurück. `mmap` nur für vollständig prüfsummenverifizierte, unveränderliche, volumenlokale Dateien; sonst `pread`.|G+C|REQ-DUR-004|
|INV-FORMAT-1|K|Jede persistente Datei trägt Magic und Formatversion; neuere Version ⇒ Öffnen verweigert; Migration nur explizit, getestet, mit Vorab-Backup. Formate sind explizit spezifiziert (Feldreihenfolge, Endianness, Längen), nicht aus `serde`-Derives abgeleitet.|G+F|REQ-DUR-005|
|INV-LOCK-ORDER-1|K|Vollständige, geordnete Sperrenhierarchie; keine Sperr-Guards über `.await` (§8.2).|L+G|REQ-MVCC-004|
|INV-HNSW-SLOT-1|K|Arena-Slots gelöschter HNSW-Knoten werden nicht wiederverwendet, solange veraltete Kantenreferenzen möglich sind (§9.3).|L+G|REQ-IDX-002|
|INV-EMBED-1|K|Kein stiller Platzhalter-Embedder; Mock nur hinter `cfg(test)`/`test-utils`; `test-utils` in keinem Release-/Default-Set.|G+I|REQ-IDX-011|
|INV-EMBED-2|K|Collection persistiert `embedding_model_hash` und Dimension; abweichender Hash wird abgewiesen (§11.1).|G+I|REQ-IDX-011|
|INV-WIRING-2|K|Sicherheits-Ports (Liste in `governance/security_ports.toml`, §5.2) haben in Produktion eine **reale** Implementierung; Null-, Test- und Accept-All-Implementierungen (konstantes `true`/`Ok(())`) als Standard sind unzulässig.|G+I|REQ-FAC-003|
|INV-DELETION-PORTS-1|K|Die Löschpfad-Ports `LayerScanner`, `Purger`, `ErasureJournal`, `Vfs`, `DurableLog`, `KmsProvider` sind Sicherheits-Ports (INV-WIRING-2) und in `security_ports.toml` geführt.|G+I|REQ-DEL-023|
|INV-PROOF-COVERAGE-1|K|Jede der zehn Schichten erscheint in einem Bericht genau einmal als belegt (`Structural`), `NotMeasured` oder `FeatureDisabled`.|T+I|REQ-DEL-003|
|INV-WITNESS-MINT-1|K|`LayerWitness` entsteht ausschließlich in `ScanSession::finish` (Konstruktoren `pub(crate)`, kein `Default`/`Deserialize`); `ScanSession` ist nur über `LayerScanner::begin_scan` mit bestandener Positivkontrolle (`ControlToken`) erhältlich; Konstruktion außerhalb davon, auch in Produktionstests, ist verboten (Ausnahme `testkit`).|T+G|REQ-DEL-004|
|INV-PROOF-KEY-1|K|Der Signierschlüssel des Berichts stammt aus dem `KeyManager`/`KmsProvider`, nie aus einem frei übergebenen Byte-String; es gibt keinen Fallback-/Standard-/Testschlüssel.|T+P|REQ-DEL-005|
|INV-CLAIMS-1|K|Externe Zusicherungen werden aus dem Bericht (`outcome`, `claims`, `excluded`) **erzeugt**, nie aus dem Zielzustand; Außentexte entstehen nur aus der Satzbibliothek (§12.11).|G|REQ-DEL-006|
|INV-PROVENANCE-1|K|Merges sind scope-treu, beschreibend benannt, serialisiert über eine Merge-Queue und auf dem Merge-Ergebnis kompilierend (§18.2).|G|REQ-GOV-001|
|INV-PLATFORM-1|K|Durabilitäts- und Löschgarantien gelten **nur auf der Garantieplattform** (§13.7); andere Plattformen sind Best Effort: `doctor` weist `BestEffort` aus, Berichte tragen `platform_guarantee = false`. Löschlogik stützt sich nie auf Unix-Unlink-Semantik offener Dateien. Funktionen ohne Plattformäquivalent (`mlock`) sind in `doctor` deklariert.|CI+C|REQ-PLT-001|
|INV-IPC-1|K|Genau ein Primärprozess hält das Datenverzeichnis; weitere Clients nutzen ausschließlich lokales IPC (Unix-Socket/Named Pipe, Peer-Credentials, kein TCP/UDP).|I+C+G|REQ-MCP-002|
|INV-STDOUT-1|K|`stdout` gehört ausschließlich dem MCP-Protokoll; Logging nach `stderr`/Datei; `clippy::print_stdout` ist in allen Crates des MCP-Binärs `deny`.|I+C+G|REQ-MCP-001|
|INV-CANCEL-1|K|Anfrageabbruch (`notifications/cancelled`, Timeout, Disconnect) wirkt kooperativ bis in die Engine; vor dem WAL-Append bricht er ab, danach wird er ignoriert (Commit bleibt atomar).|I+C+G|REQ-MCP-007|
|INV-FORGET-GUARD-1|K|Breite Löschungen (Selektor, Collection, Tenant, > `forget_confirm_threshold` Dokumente) brauchen eine Bestätigung über einen Kanal **außerhalb des Tool-Aufrufs** (§13.3).|I+P|REQ-MCP-005|
|INV-TRUST-1|K|Jeder Eintrag trägt `trust ∈ {user, agent, external}` und `source`; Recall liefert Inhalt strukturiert als Daten mit Trust-Label, nie als Anweisung.|I+P|REQ-MCP-006|
|INV-LOG-1|K|Logs, Metriken, Fehlertexte und Panic-Meldungen enthalten keine Nutzinhalte, nur IDs und Zähler.|G+P|REQ-AUD-002|
|INV-RETENTION-1|K|Ranking-Abwertung löscht nie. Löschung entsteht ausschließlich durch ausdrückliche Aktion oder eine opt-in `RetentionPolicy` mit Dry-run, Karenzzeit und Audit (§11.6).|P+I|REQ-COG-003|
|INV-EVIDENCE-1|K|Jede Zusage ist ein Ziel, bis der Mechanismus sie maschinell belegt; ein nicht ausgeführtes oder nicht verfügbares Prüfwerkzeug ist rot.|G|REQ-QA-001|
|INV-ORACLE-1|K|Jede Kerngarantie RB1–RB4 hat ein unabhängig entstandenes Prüfmaß.|P|REQ-QA-002|
|INV-SCAN-1|K|Die öffentliche API enthält keine Funktion, keinen Typ und kein Feld, das Schlüsselmaterial (WAL-Integritätsschlüssel, DEK, KEK, Signierschlüssel) im Klartext zurückgibt; Test-Zugänge liegen hinter `cfg(test)` oder einem Test-Feature außerhalb jedes Release-/Default-Sets.|G|REQ-ARCH-014|
|INV-SIDECAR-1|K|Außer WAL, Manifest, SSTables, Blob-Segmenten und Indexdateien existiert persistenter Zustand nur in den Sidecar-Logs (§7.9). Jedes Sidecar trägt Magic und Version, eine HMAC-Kette mit eigenem Zweckschlüssel und eine Manifest-Marke. Wirkungen eines WAL-Commits stehen als `SidecarIntent` im selben Batch und werden idempotent angewendet; sicherheitskritische Wirkungen folgen „Sidecar vor Wirkung“.|G+C+I|REQ-DUR-016|
|INV-EMERGENCY-DEL-1|K|Ein Betroffener ist in jedem Betriebszustand mindestens **sperrbar**: in `ReadOnly` (ENOSPC) sind `forget`-Start und Lesesperre über eine vorab reservierte Notreserve ausführbar; in `Poisoned` ist der Sperr-only-Betrieb über unabhängige Sidecar-Handles zulässig, WAL-Schreiben nicht. Fehlt auch das, gilt eine prozesslokale Lesesperre (fail-closed) bis zum Neustart (§7.10).|C+I|REQ-DUR-017|
|INV-ASOF-ERASE-1|K|Historische Lesezugriffe (`as_of`, offene Snapshots, Checkpoint-Lesen) unterliegen der Lesesperre; ein gelöschter oder gesperrter Betroffener ist in keinem Snapshot lesbar, auch wenn die Altversion physisch existiert. Snapshots haben Höchstalter `snapshot_max_age`.|P+L|REQ-DEL-030|
|INV-BOOT-1|K|Der Start folgt der festen Phasenfolge B0–B9 (§13.5): Die Lesesperre (`ErasedSet`) ist armiert, bevor irgendein Lesepfad Verkehr annimmt; jede Phase kann den Start mit benanntem Grund verweigern; kein Teilstart.|I+C|REQ-FAC-005|
|INV-KEYPURPOSE-1|K|Jeder Schlüssel dient genau einem Zweck und entsteht aus dem Zweckregister (§12.3) mit eigenem HKDF-Label; ein Label ohne Registereintrag ist Gate-Fehler.|G+P|REQ-KEY-011|
|INV-RESTORE-HORIZON-1|K|`restore_checkpoint` und jeder Import verweigern fail-closed jedes **Schlüssel-Backup** (Spur K), dessen `valid_until` überschritten oder dessen Alter nicht bestimmbar ist; Daten-Backups (Spur D) sind ohne Schlüssel inert. Die Konfiguration erzwingt `key_backup_max_age ≤ erased_set_retention`. Ein `ErasedSet`-Eintrag bleibt mindestens `erased_set_retention` zuordenbar (Referenz-Epochen, §12.10).|I+C+G|REQ-DEL-032|
|INV-DEL-ORDER-1|K|Die Schlüsselvernichtung (Schritt 5) beginnt erst, wenn Klasse P (Siegel 2) für jede Ebene-A-Schicht der Einheit ausgestellt und das Pre-Destruction-Inventar durabel ist; einzige Ausnahme ist der Notpfad (§7.10).|I+C|REQ-DEL-N20|
|INV-NO-RESURRECTION-1|K|Beim Start (B6/B7) und nach jedem Restore steht für jede `ErasedSet`-Einheit ab `Frozen` kein lebender DocDek und kein lebender ArtKey in der KeyTable; Abweichung ⇒ Vernichtung wird wiederholt, Alarm (§12.8.4).|C+P|REQ-DEL-N21|
|INV-BACKUP-SPLIT-1|K|Kein Daten-Backup (Spur D) enthält Schlüsselmaterial; jedes Schlüssel-Backup (Spur K) trägt ein signiertes `valid_until` und wird danach verweigert (§12.17).|I+G|REQ-DEL-N22|
|INV-CLASS-MEASURED-1|K|Ein Bericht weist höchstens die Klasse aus (L, P, C*, C1–C3, M), deren Voraussetzungen gemessen sind; die Klasse ist nie höher als die Messung trägt (§12.5.1).|T+P|REQ-PRF-2|
|INV-CLAIM-BIND-1|K|Außentexte entstehen nur aus einer festen Satzbibliothek; jeder Satz ist an einen gemessenen Wert gebunden; ein Satz ohne Messwert bricht den Build (Gate `claim-generator`, §12.11).|G|REQ-DEL-035|
|INV-SURFACE-1|K|Jede persistente Datei- und Strukturklasse des Format-Registers (§7.11) steht in genau einer Zeile der Schichttabelle (§12.4) oder in den Ausschlüssen (§12.20); ein neues Format ohne Schichtzuordnung bricht den Build (Gate `surface-coverage`).|G|REQ-DEL-N23|
|INV-NO-DUMMY-PROOF-1|K|Kein Codepfad erzeugt einen `DeletionProof` ohne Scanner-Witness; `contextra_delete` und `contextra_drop_collection` erzeugen keinen Bericht außerhalb der Saga.|G+I|REQ-PRF-4|
|INV-EPOCH-SEAL-1|K|Die Klassen C1 bis C3 gelten erst nach dem ersten Epoch Seal, der die Löschung umfasst; bis dahin lautet die Klasse C\* mit `residuals: [R-KEKEPOCH until]` (§12.2.4).|C+P|REQ-DEL-N24|
|INV-ARTKEY-1|K|Jedes Index-Artefakt hat genau einen ArtKey; der alte ArtKey wird bei einem Neuschreiben ohne Löschbezug (Merge) sofort, bei einem Neuschreiben aus einer Löschung **nach Siegel 2** im Destroy-Protokoll (Schritt 5) vernichtet, damit Inventar und entschlüsselte Messung vor der Vernichtung möglich bleiben (§12.2.8, INV-DEL-ORDER-1).|P+C|REQ-DEL-N16|
|INV-SEAL-HARDENING-1|K|Der Bericht weist `flash_hardening ∈ {None, PassphraseRotated, ModuleBound, ProviderAttested}` aus; ein K1-Seal ohne Passphrasenwechsel ist `None` und stützt die Klasse C1 allein auf A-FLASH; die Angabe wird aus dem tatsächlichen Seal-Ablauf erzeugt, nie aus der Konfiguration (§12.2.4).|T+P|REQ-DEL-N40|
|INV-HEAP-ZERO-1|K|Klartext von Nutzdaten (entschlüsselte Blöcke, Query-Puffer, Memtable-Werte, Arena-Slots, Caches) liegt nur in `Sensitive`-Arenen der Insel `sys` und wird bei Freigabe zeroisiert; ein Heap-Canary-Test findet nach Klasse P keinen Scope-Inhalt im Prozessspeicher (§12.7.11).|T+G+C|REQ-DEL-N41|
|INV-UNITDIR-1|K|Jede persistente Datei trägt ein unter `K_eng` verschlüsseltes UnitDirectory (Einheit → Blöcke); die Messung stützt sich auf vollständiges Inventar; ein Block ohne Zuordnung ist ein Fehler (fail-closed); Einträge vernichteter Einheiten verschwinden beim Hygiene-Rewrite (§12.8.9).|P+G|REQ-DEL-N42|
|INV-ADMISSION-1|R|(`sandbox`) Ein `AdmittedModule` ist nur über `ModuleVerifier::verify` konstruierbar; SHA-256-Digest aus den Bytes, nie vom Aufrufer. Unsignierte Module nur mit `ApprovalRisk::Low`, im Audit als `unsigned`.|T+P+F|REQ-SBX-001|
|INV-APPROVAL-1|R|(`sandbox`) `High`-/`Elevated`-Module laufen nicht vor `ApprovalStatus::Approved`.|T+P+F|REQ-SBX-002|
|INV-WASM-DET-1|R|(`sandbox`) Gleiches Modul, Eingabe und Fuel liefern auf x86-64 und aarch64 bit-identische Ergebnisse; Wall-Clock-/Epoch-Unterbrechung ist nur Sicherheitsnetz und fail-closed.|T+P+F|REQ-SBX-003|
|INV-EGRESS-BUDGET-1|R|(`privacy`) Jede externe Wirkung ist budgetiert, auditiert und widerrufbar.|I+P|REQ-PRV-002|
|INV-LICENSE-1/-2/-3|E|(`license`) Verstoß/Ablauf/fehlende Aktivierung degradiert auf Basisfunktion, nie Datenverlust oder -sperre; niedrigste Klasse (`Fast`) immer `Ok`; falsche Installationsbindung ⇒ `NotActivated` ohne Informationsleck.|P|REQ-EXP-003|

**Zählung:** `[K]` 64 (53 der Fassung 1.3 minus `INV-SHRED-1` plus neun neue aus 2.3 und drei aus 2.4 [`INV-SEAL-HARDENING-1`, `INV-HEAP-ZERO-1`, `INV-UNITDIR-1`]; die 2.3-Neuen sind: `INV-DEL-ORDER-1`, `INV-NO-RESURRECTION-1`, `INV-BACKUP-SPLIT-1`, `INV-CLASS-MEASURED-1`, `INV-CLAIM-BIND-1`, `INV-SURFACE-1`, `INV-NO-DUMMY-PROOF-1`, `INV-EPOCH-SEAL-1`, `INV-ARTKEY-1`; die Summe ist vom Doc-Truth-Gate nachzuzählen), `[R]` 4, `[E]` 3 (als eine Zeile geführt). Das Doc-Truth-Gate prüft: jede `INV-*` einmal definiert, einmal im Mechanismus geführt, von mindestens einer Anforderung referenziert, IDs eindeutig (REQ-GOV-004).

### C-6. Nebenläufigkeitsmodell (E15, E29)

**Befund:** `store` hat 117–144 `async fn`, `engine` 162, `vector` 49, `text` 43 (Ring 0 gesamt 92 laut Scan); `StorageEngine` liefert `BoxFuture`. Das Gate `async-io` (Blockieren im async-Kontext) hat 18 Verstöße; das Gate `ring0-async` ist grün, prüft aber nur die `tokio`-Abhängigkeit. Ob die `async fn` in Ring 0 nur Port-Signaturen bedienen, ist nicht gemessen (Spike S-ASYNC).

**Entscheidung (gestufte Grenze):**

1. **Ring 0 bleibt Ziel laufzeitfrei und synchron** (`vector`, `text`, `rank`, `crypto`, `mvcc`, `types`: kein `async fn`, kein `tokio`, kein `BoxFuture`). Der **Umbau des Bestands ist zurückgestellt** (E29), in v0.1 und v1.0 nicht Pflicht. Es gilt ein **Ratchet:** keine neue `async fn`, keine neue `tokio`-Abhängigkeit in Ring 0; der Bestand darf nur sinken. Spike S-ASYNC ist reine Messung: Sind ≥ 70 % der `async fn` ohne Verhaltensänderung entfernbar, erfolgt der Umbau in Phase 4; sonst wird E15 in Ring 0 auf „Async-Fassade über synchronem Kern“ reduziert und neu entschieden.
2. **`store` (Ring 1)** darf async bleiben, aber ausschließlich an der Port-Grenze `StorageWrite`; lesende Pfade sind synchron (`StorageRead`). Blockierende Datei-I/O in `async fn` läuft über `spawn_blocking` oder einen dedizierten I/O-Thread (die 18 Verstöße werden behoben).
3. **`engine` (Ring 3)** hält den Compute-Pool: CPU-lastige Operationen (Embedding, Fusion, HNSW-Suche) laufen auf dem Blocking-Pool, nie auf dem Executor-Thread.
4. **Tokio nur an den Rändern:** Die Laufzeit wird im Composition Root bzw. `mcp` erzeugt; Bibliotheksnutzer ohne Async erhalten eine synchrone Fassade (`block_on` intern, dokumentiert).
5. **Abbruch (INV-CANCEL-1):** `RequestContext` trägt `CancelToken` und `Deadline`; synchrone Schleifen prüfen sie an definierten Punkten (HNSW-Suche je Ebene, Kompaktion je Block, Saga-Schritte). `forget` ist abbrechbar nur **zwischen** Saga-Schritten.
6. **Sperrenordnung (INV-LOCK-ORDER-1):** keine Locks über `await`; `clippy::await_holding_lock` auf `deny`; loom-Tests für `mvcc` und `engine/collection/kv_lock.rs` (§8.2).

|ID|Anforderung|
|---|---|
|REQ-ASYNC-1|Ratchet (Stufe B): `rg "async fn\|BoxFuture"` in den Ring-0-Crates steigt nie; `cargo tree -p <ring0-crate>` zeigt keine neue Kante zu `tokio` oder `futures-executor`. Endziel 0 (Ausnahmeliste ADR-pflichtig) nach Phase 4.|
|REQ-ASYNC-2|Gate `async-io` hat 0 Verstöße und prüft Blockieren im async-Kontext zeilenweise (heute heuristisch).|
|REQ-ASYNC-3|Jede `spawn`-Stelle hat eine Obergrenze (Backpressure); Gate `unbounded-spawn`.|

### C-7. Persistenz (`contextra-store`, `contextra-checkpoint`) `[K]`

#### C-7.1 Datenpfad und Blob-Segmente

`put/delete/merge` → TxBuffer → **WAL-Batch (Group Commit, ein Commit-Punkt)** → MemTable → Flush → SSTable (L0) → Kompaktion (L1…Ln). Werte ≥ `blob_threshold` (Start 64 KiB) gehen als **Blob-Segment** statt durch den WAL-/LSM-Wertpfad. Lesen: MemTable → immutable MemTables → SSTables (Bloom-gefiltert) mit Block-Cache. Alle Werte sind Dokument-Records (eine Verschlüsselungseinheit, §12.2).

**Blob-Segmente:** append-only Werte; LSM/WAL halten nur Zeiger + BLAKE3-Digest. **Reihenfolge:** Segment durabel flushen → WAL-Commit mit Zeiger → Ack. Orphan-Segmente (Absturz dazwischen) räumt Recovery per Manifest ab. Jeder Record im Segment liegt unter dem DocDek seiner Betroffenen-Einheit (§12.2.5); die Segmentdatei wird bei der Hygiene neu geschrieben (§12.8.2 Schritt 8).

#### C-7.2 Dateiformate

| Format | Magic | Inhalt |
| --- | --- | --- |
| WAL | `MFW3` (Ops `IndexIntent`/`IndexApplied`) | Records: Sequenznummer, Länge, `scope_id`, `key_id`, Nonce, AEAD-Ciphertext, **HMAC-SHA256-Kette** (jedes Record bindet den Vorgänger-Tag). Segmente vorab reserviert. |
| SSTable | `MFSX` (Legacy `MFST`, Entscheidung in WP 2.4) | Datenblöcke, Index-Block, Bloom-Filter, Footer mit Prüfsumme/Tag, AEAD je Block (Engine-Schlüssel); Werte darin je Record unter dem DocDek der Einheit. |
| Blob-Segment | `MFBL` | Append-only Werte, Kopf mit Digest; je Record ein Schlüsselbezug auf den DocDek der Einheit. |
| Manifest | `MFMN` | Versionierte Liste der Live-Dateien je Level, **High-Water-Mark** (höchste bestätigte WAL-Sequenz) gegen Truncation. |
| Keystore | `MFKS` | `KdfHeader` (Kennung `MFKD`), gewrappte Wurzelschlüssel; bei Betriebsart K0 der Integritätswurzelschlüssel im Klartext. |
| Sidecar-Logs | `MFKR`, `MFRV`, `MFAU`, `MFEJ`, `MFES` | `KeyRegistry`, `RevocationLog`, `AuditChain`, `ErasureJournal`, `ErasedSet` (§7.9). |

**Formatversionierung (INV-FORMAT-1):** Magic **und** Formatversion in jeder Datei; neuere als unterstützte Version ⇒ Öffnen verweigert (kein stilles Herabstufen); Downgrade-Angriffe (Ersetzen durch ältere Version) werden abgewiesen. Ältere Versionen öffnet nur ein expliziter, getesteter Migrationspfad: `contextra migrate` erzeugt zuvor ein verifiziertes Backup und bietet `--dry-run`. Pro Release liegt ein **Kompatibilitätskorpus** (Golden-Files je Format und Version) im Repository; die CI öffnet ihn mit dem aktuellen Code.

#### C-7.3 WAL-Garantien

- **Commit-Punkt = durabler Flush des Batches** (Primitive je Plattform, §7.7). Ack erst danach.
- **Group Commit:** Mehrere Transaktionen teilen sich einen Flush; Latenz/Durchsatz über `PerformanceProfile` (§13.7).
- **Kettenprüfung beim Recovery:** Bruch der HMAC-Kette ⇒ Abbruch am letzten gültigen Record (torn tail wird abgeschnitten; Korruption in der Mitte ⇒ fail-closed).
- **Anti-Truncation:** Manifest-High-Water-Mark > WAL-Ende ⇒ Manipulation/Datenverlust wird erkannt, nicht verschwiegen.
- **Index-Atomarität (INV-ATOMIC-IDX-1):** Ein Batch enthält Daten-Ops **und** `IndexIntent{index_id, op}`. `IndexApplied` ist ein **nicht-durabler Hinweis** (kein eigener Flush): Recovery wiederholt Intents idempotent; ein Commit kostet genau einen Flush.
- **CDC:** `WalObserver` erhält geordnete, deterministische Ereignisse mit dauerhaftem Cursor (at-least-once, idempotent konsumierbar). **Ereignisse tragen keine Nutzinhalte** (nur `op`, `scope_id`, opake IDs, `seq`), sodass Observer keine Löschkopien halten.
- **`fsync`-Fehler (INV-FSYNC-1)** und **volles Medium (INV-ENOSPC-1)** wie in §3.4. **Backpressure:** erreichen L0-Dateien oder MemTable-Bytes die Schwellen aus §7.4, werden Schreiber gedrosselt statt Speicher unbegrenzt zu belegen.

**7.3.1 Recovery-Algorithmus (Speicher-Teilfolge von B4–B7)**

1. DirLock erwerben. Kann das Dateisystem keine exklusive Sperre garantieren (Netzwerk-/Cloud-Sync-/unbekannte FUSE-Dateisysteme), wird das Öffnen im persistenten Betrieb verweigert; nur `MemoryOnly` läuft ohne Sperre.
2. Keystore und Manifest laden und prüfen.
3. WAL ab letztem Checkpoint kettenvalidieren.
4. Records in MemTable replayen; offene `IndexIntent`s vom Applier idempotent anwenden (INV-INDEX-APPLY-1); Embedding-/LLM-Werte stammen aus dem WAL (INV-NONDET-1).
5. High-Water-Mark abgleichen.
6. Sidecar-Logs validieren und gegen ihre Manifest-Marken abgleichen; ausstehende `SidecarIntent`s aus dem WAL idempotent anwenden.
7. `ErasedSet` laden und Lesesperre armieren (INV-BOOT-1); `ErasureJournal` replayen und offene Löschsagas zur Fortsetzung vormerken (§12.8).
8. Retention-Entscheidungen (WAL-Records, §11.6) in ihren `PendingDeletion`-Zustand überführen.
9. Erst dann Lese-/Schreibverkehr zulassen; Lesen eines durch den `ErasedSet` gesperrten Betroffenen bleibt auch danach verweigert.

Poison-Locks werden erkannt und über definierte Wiederherstellung behandelt. Die Checkpoint-Frequenz begrenzt die Recovery-Zeit auf ein je Tier festgelegtes Budget (§16).

#### C-7.4 Kompaktion

- **Tiered** und **adaptiv** (`CostBasedAdaptivePlanner`): minimiert Write-/Read-/Space-Amplification anhand gemessener Workload-Statistiken.
- Tombstones werden erst auf dem untersten Level bzw. nach Snapshot-Horizont verworfen (MVCC-sicher); heute nur bei Vollkompaktion mit `raw_seq < min_snapshot_seq` (`compaction/engine.rs:712–720`) – Fenster mit Restdaten bis zur Kompaktion ist belegt (Spike E1).
- **Hygiene (Saga-Schritt 8)** erzwingt bereichsweise **Purge-Kompaktion** betroffener SSTables (`compact_range(scope)`, §12.8.2). Sie ist nicht mehr Voraussetzung der Schlüsselvernichtung, aber Voraussetzung der Klasse M.
- Kompaktion ist budgetiert (I/O-Rate, CPU) und vom `SystemPressureMonitor` drosselbar.
- **Write-Stall-Policy:** ab `l0_slowdown` (Start 8) werden Schreiber gedrosselt; ab `l0_stop` (Start 20) warten sie höchstens `write_stall_timeout` (Start 5 s) und erhalten danach den retrybaren Fehler `Busy`. Rückstau ist **kein Betriebszustand**: er ändert weder Gesundheitszustand noch Garantien.

#### C-7.5 MergeOperator (IoC auf Schreibsemantik)

Port `MergeOperator { full_merge, partial_merge }`. Implementierungsarten: **Rust** (statisch, generisch) und **WASM** `[R]`. Verbindliche **CRDT-Gesetze** (Assoziativität, Kommutativität, Idempotenz soweit zutreffend), property-getestet für **LWW-Register**, **G-Counter**, **OR-Set**. Ein WASM-Operator wird nur registriert, wenn er die Gesetz-Suite besteht und die Determinismus-Konfiguration (Anhang A.1) erfüllt. Operatoren sind deterministisch (P28).

#### C-7.6 Caches, Druck, I/O-Regeln

- **Block-Cache:** ein Standard (**SIEVE**) hinter dem Port `BlockCache`; Alternativen sind Port-Implementierungen. Bloom-Filter je SSTable (konfigurierbare FPR); Prefix-Bloom für Range-Scans.
- **`SystemPressureMonitor`** (über `sys`): bei Speicherdruck werden Caches verkleinert und Kompaktion gedrosselt – kontrolliert, nie OOM-Kill.
- **I/O (INV-IO-1):** Standard ist `pread` mit Prüfsumme; entschlüsselte Blöcke leben als `Bytes` im Block-Cache (Zero-Copy ab dort). **`mmap`** nur für unveränderliche, vollständig prüfsummenverifizierte Dateien auf lokalem Volume (`doctor` prüft Dateisystemtyp); Verkürzen/Überschreiben durch Fremdprozesse wird durch DirLock und Rechte ausgeschlossen. Verschlüsselte Dateien werden nicht zero-copy gemappt (INV-ENCRYPT-REST-2).

#### C-7.7 Durability-Primitive je Plattform (INV-DURABILITY-PRIM-1)

| Plattform | Datei-Flush | Verzeichniseintrag (Create/Rename) |
| --- | --- | --- |
| Linux | `fdatasync` (Metadaten ändernd: `fsync`) | `fsync` auf das Elternverzeichnis |
| macOS | `fcntl(F_FULLFSYNC)` (reines `fsync` garantiert keine Durabilität) | `F_FULLFSYNC` auf das Verzeichnis |
| Windows | `FlushFileBuffers` | `MoveFileEx(MOVEFILE_WRITE_THROUGH)`/`ReplaceFile` |

Jede neue oder umbenannte Datei (WAL-Segment, SSTable, Manifest-Wechsel) gilt erst nach Flush des Elternverzeichnisses als existent. Sämtlicher Dateizugriff läuft über den Port `Vfs`; die Implementierung liegt in `store` (`fsync.rs`), die Fehlerinjektion `FaultVfs` in `testkit`; beide bilden diese Primitive ab.

#### C-7.8 Betriebszustände und Gesundheitsübergänge

| Zustand | Bedeutung | Eintritt | Verlassen |
| --- | --- | --- | --- |
| `Healthy` | alle Garantien aktiv | nach erfolgreicher Recovery | – |
| `Degraded` | nicht-kritischer Teil ausgefallen (Block-Cache, `[R]`-Feature, Hintergrundkompaktion); Kern-Garantien bleiben | Ausfall einer abgeleiteten Komponente | Komponente gesund oder Neustart |
| `ReadOnly` | Lesen erlaubt, Schreiben abgelehnt (retrybar); Löschen über die Notreserve zulässig (§7.10) | `ENOSPC` bei Rotation, `fsync`-Fehler auf einem Sidecar-Handle, Lock-Poisoning in Schreibstrukturen mit sicherer Wiederherstellung | Platz frei / Neustart |
| `Poisoned` | Durabilität unbestimmt: kein Ack, kein Schreiben | `fsync`-/`fdatasync`-Fehler am WAL (INV-FSYNC-1), unwiederherstellbarer Poison-Lock | ausschließlich Neustart mit Recovery-Replay |

Regeln: (1) Übergänge sind monoton in Richtung `Poisoned`, außer durch Neustart bzw. dokumentierte Selbstheilung `Degraded → Healthy`. (2) Jeder Übergang wird auditiert (ohne Nutzinhalte) und über `doctor` sichtbar. (3) Ein Poisoned-Lock ist ein typisierter Fehler mit definiertem Übergang, nie `unwrap`. (4) Unbekannte Zustände ⇒ fail-closed. (5) Prüfung: REQ-DUR-014.

#### C-7.9 Sidecar-Logs: die einzigen Persistenzdomänen neben dem WAL

Der WAL ist die einzige Wahrheit für Anwendungszustand (P24). Sicherheitszustand kann nicht aus dem WAL abgeleitet werden, weil S3 WAL-Segmente scrubbt, und muss Löschungen **überleben**. Er liegt deshalb in genau diesen Dateien; die Liste ist **abschließend** (INV-SIDECAR-1):

| Sidecar | Magic | Inhalt | Primärer Schreiber | Kettenschlüssel (§12.3) | Im Backup |
| --- | --- | --- | --- | --- | --- |
| `KeyRegistry` | `MFKR` | Schlüsselereignisse: Erzeugen, Wrap, Rotation | `KmsProvider` über `KeyManager` | `K_sc/keyreg` | ja (nur Ereignisse, keine Schlüssel) |
| `RevocationLog` | `MFRV` | Widerruf von Schlüsseln, Modulen, Berechtigungen | Autorisierung, Löschsaga | `K_sc/revoke` | ja |
| `AuditChain` | `MFAU` | Zugriffe, Löschungen, Berichte, Freigaben; tenant-getrennt | Applier (aus `SidecarIntent`) und Sicherheitspfade | `K_sc/audit` | ja |
| `ErasureJournal` | `MFEJ` | Saga-Übergänge (§12.8) | Orchestrator | `K_sc/journal` | nein (Betriebszustand) |
| `ErasedSet` | `MFES` | Referenz-Hashes gelöschter/gesperrter Betroffener und Einheiten samt Referenz-Epoche (§12.9, §12.10); wird an den `ErasedSetMirror` gespiegelt (§12.17.3) | Orchestrator | `K_sc/erased` | **nein** (§7.9 Regel 5) |
| Keystore | `MFKS` | `KdfHeader`, gewrappte Wurzelschlüssel | `KmsProvider` (lokal) | (Integritätswurzel) | **nein** (Schlüsselmaterial) |

Regeln:

1. **Form.** Jedes Sidecar ist append-only, trägt Magic und Formatversion, eine HMAC-Kette mit eigenem Zweckschlüssel und einen `fsync` je Eintrag über den Port `DurableLog`. Die **Manifest-Marke** (Name, letzte Sequenz, Kettenkopf) wird bei jedem Checkpoint und jeder Rotation aktualisiert; sie erkennt Tail-Truncation und Ersetzen durch eine ältere Version. Das Löschen eines Sidecars bei vorhandenen Daten ist Manipulation (fail-closed).
2. **Intent-Muster (WAL zuerst).** Wirkungen, die ein WAL-Commit auslöst (Audit-Einträge zu Datenoperationen, Lineage-Hinweise), stehen als `SidecarIntent{log, payload_hash}` im selben WAL-Batch (wie `IndexIntent`). Der Applier schreibt sie nach dem Commit idempotent über `(log, seq)`; Recovery wiederholt offene Intents (§7.3.1 Schritt 6). Ein Commit kostet weiterhin genau einen Flush.
3. **Sidecar vor Wirkung.** Sicherheitskritische Wirkungen, die nicht aus einem WAL-Commit folgen, werden zuerst im Sidecar durabel und erst danach wirksam: Autorisierungswiderruf (`RevocationLog`), Saga-Übergang (`ErasureJournal` **vor** Ausführung des Schritts), `ErasedSet`-Eintrag (**vor** der ersten Löschoperation).
4. **Autorität.** Sidecars sind nicht aus dem WAL rekonstruierbar und deshalb autoritativ; ein beschädigtes Sidecar in der Mitte der Kette ist fail-closed (Start verweigert, `doctor` benennt Datei und Sequenz).
5. **Backup.** Checkpoints enthalten `KeyRegistry`, `RevocationLog` und `AuditChain`, aber weder `MFKS` noch `ErasedSet`. Der `ErasedSet` gehört dem laufenden System: Der Restore-Guard liest den `ErasedSet` des **Zielsystems**.
6. **Ausfall.** Ein `fsync`-Fehler auf einem Sidecar-Handle vergiftet genau diesen Handle (INV-FSYNC-1) und führt gemäß §7.10 zu `ReadOnly` bzw. `Poisoned`.

#### C-7.10 Notlöschpfad und Poison-Umfang

Löschen ist eine Schutzfunktion und darf nicht an Betriebszuständen scheitern, die Löschen selbst auslösen (volles Medium, `fsync`-Fehler).

| Ereignis | Zustand | Löschen erlaubt? |
| --- | --- | --- |
| `ENOSPC` bei WAL-Rotation | `ReadOnly` | **Ja.** Tombstone-Batch, Journal-Einträge und `ErasedSet`-Eintrag nutzen die Notreserve (`emergency_reserve_bytes`, Start 64 MiB, mindestens zwei WAL-Segmente plus Journal, bei Start vorab reserviert). S2 (Purge-Kompaktion) braucht Platz: Schlägt sie mit `StorageFull` fehl, bleibt die Saga `Partial{outstanding: Lsm}` und setzt nach Freigabe fort; der Bericht weist das aus. |
| `fsync`-Fehler am WAL-Handle | `Poisoned` | **Nur Sperr-only:** `RevocationLog`, `ErasedSet` und `ErasureJournal` über ihre **eigenen, gesunden** Handles; keine WAL-Schreibzugriffe. Der Tombstone-Batch wird nach dem Neustart von der Saga aus dem Journal nachgeholt. |
| `fsync`-Fehler an einem Sidecar-Handle | `ReadOnly` (Funktion fail-closed) | Betroffen ist genau die abhängige Funktion: `RevocationLog` ⇒ keine Autorisierung neuer Schreibvorgänge; `ErasureJournal` ⇒ keine neue Saga, aber Lesesperre im RAM; `KeyRegistry` ⇒ keine Schlüsselerzeugung. |
| Ausfall aller Löschpfad-Handles | `Poisoned` | Prozesslokale Lesesperre für den angefragten Betroffenen (INV-EMERGENCY-DEL-1); Retry nach Neustart. |

Leseverkehr im Zustand `Poisoned` ist nur zulässig, wenn der `ErasedSet` armiert ist (INV-BOOT-1); andernfalls verweigert der Zustand auch das Lesen. Der Poison-Umfang ist pro Persistenzdomäne definiert und monoton wie in §7.8.

#### C-7.11 Format-Register und Regeln

Ein Register `docs/spec/FORMATS.md` führt jedes persistente Artefakt (Gate `format-registry`: jede Datei im Datenverzeichnis hat einen Eintrag). Soll-Stand (Spalte „Ist-Befund“ ist in WP 2.4 aus dem Code auszufüllen; „prüfen“ = nicht gelesen):

| Artefakt | Magic | Version | Prüfsumme | Ist-Befund |
| --- | --- | --- | --- | --- |
| SSTable | `MFSX` (Legacy `MFST`) | prüfen | prüfen | beide Magics im Code (`sstable/block_cache.rs:340–341`); Legacy-Pfad migrieren oder entfernen |
| Manifest | `MFMN` | prüfen | prüfen | `manifest/core.rs:17` |
| WAL | `MFW3` | prüfen | HMAC (`wal/hmac.rs`) | Panic `hmac.rs:194`, `wal/mod.rs:92` |
| Löschbericht (`DeletionProof`) | `MFDR` | V5 (explizites Layout, kein `serde`-Derive); V1–V3 nur lesbar; V4 bleibt unvergeben (E46) | Ed25519 | im Repo höchstens V3 (U-39); Golden-File Pflicht (REQ-PRF-5) |
| KeyTable | `MFKT` | prüfen | Blake3 | neu in 2.3; Ersetzen-Semantik, atomar (REQ-FMT-5), §12.2.9 |
| `ErasedSetMirror`-Eintrag | `MFEM` | prüfen | HMAC | neu in 2.3, §12.17.3 |
| `AuditChain`, `RevocationLog`, KV-Segment | prüfen | prüfen | prüfen | bincode (v1.3.3); Versionsfeld offen |
| HNSW-Snapshot, Text-Segmente | prüfen | prüfen | prüfen | nicht gelesen |
| Checkpoint-Manifest | – | – | Blake3, signiert | – |
| Wire (FlatBuffers) | – | Schema `schemas/contextra.fbs`, `flatbuffers 24.3` | – | generierter Code eingecheckt; `flatc` fehlt in der Analyseumgebung; Gate `check-flatbuffers-drift` existiert |

**Umfang in v1.0 (E30):** Pflicht für WAL, SSTable, Manifest, `AuditChain`, `RevocationLog`, Checkpoint-Manifest. HNSW-Snapshot, Text-Segmente, KV-Segment und Wire folgen, sobald ihre Leser gelesen sind (WP 2.4).

| ID | Regel |
| --- | --- |
| REQ-FMT-1 | Jedes persistente Artefakt beginnt mit Magic (4 Byte), Formatversion (u16), Flags, Längenfeld und trägt eine Prüfsumme (CRC32C für Blöcke, Blake3 für Manifeste/Segmente). |
| REQ-FMT-2 | Ein Leser verweigert **fail-closed** jede Version, die er nicht kennt; der Schreiber schreibt nur die aktuelle Version. |
| REQ-FMT-3 | Für jede unterstützte Version liegt ein **Golden-File** im Repo; Round-Trip-Tests laufen in CI. |
| REQ-FMT-4 | Serde-abgeleitete Layouts sind für signierte oder persistente Daten nur zulässig, wenn ein Golden-File sie festschreibt und die Serialisierungsbibliothek exakt gepinnt ist. |
| REQ-FMT-5 | Atomares Ersetzen: temporäre Datei, `fsync`, `rename`, `fsync` des Verzeichnisses (Linux `fdatasync`, macOS `F_FULLFSYNC`); Fehler werden propagiert. Das gilt auch für den Keystore. |
| REQ-FMT-6 | Migration läuft offline über `contextra migrate` mit Trockenlauf und vorheriger Sicherung; keine stille Migration beim Öffnen. |
| REQ-FMT-7 | Vor 1.0 sind inkompatible Formatwechsel erlaubt, wenn `migrate` sie abdeckt; ab 1.0 sind On-Disk-Formate stabil und nur über neue Versionsnummern erweiterbar. |

#### C-7.12 Datenverzeichnis

Beim Start (B1) wird das Dateisystem geprüft (INV-PLATFORM-1: unterstützt lokale ext4/xfs/btrfs/zfs/APFS/NTFS; **abgelehnt:** NFS/SMB, unbekannte FUSE, Cloud-Sync-Ordner wie OneDrive, Dropbox, iCloud Drive) und die Verzeichnissperre gesetzt (INV-IPC-1). `doctor` meldet Abweichungen mit benannter Ursache. Weder Netzwerk- noch Cloud-Sync-Dateisysteme werden im persistenten Betrieb zugelassen.

### C-8. MVCC und Serializable Snapshot Isolation (`contextra-mvcc`) `[K]`

#### C-8.1 Komponenten

- **SequenceLog:** monoton wachsende, WAL-gebundene Commit-Sequenz.
- **SnapshotRegistry:** aktive Snapshots; bestimmt GC-/Tombstone-Horizont. Ein Snapshot ist **immer ein konkreter Sequenzstand, nie `u64::MAX`**.
- **TxBuffer:** privater Schreibpuffer, Read-your-writes.
- **ReadSet:** erfasst Punkt-, **Prefix-/Range-Lesezugriffe (Phantomschutz)**; SSI-Validierung bei Commit erkennt gefährliche Strukturen (rw-Antidependenzen) ⇒ retrybarer `SerializationFailure`. Ob SSI tatsächlich aktiv ist, ist unbelegt (`supports_ssi_tracking` liefert konstant `true`, U-06); Nachweis durch T-SSI-1 oder Anspruch streichen.
- **Tenant:** aus der Transaktion abgeleitet; Schlüsselkonstruktion ohne Tenant-Kontext ist typbedingt unmöglich.
- **Kein Saga-/Kompensationsmodell** für Datenoperationen: der Normalfall ist atomar über den WAL-Batch; Fehler nach dem Commit-Punkt werden **vorwärts** behoben (Recovery wendet Intents erneut an), nie kompensiert. (Die Löschsaga, §12.8, ist ein eigener durabler Ablauf.)
- **Zeitreisen:** `as_of(seq | zeit)` liest historische Snapshots, solange nicht kompaktiert/gelöscht. **Löschung wirkt rückwirkend (INV-ASOF-ERASE-1):** Ein Betroffener im `ErasedSet` oder in `Frozen` ist auch in jedem historischen Snapshot unlesbar, selbst wenn S2/S3 noch `Pending` sind. Ein Snapshot hat das Höchstalter `snapshot_max_age` (Start 15 min); danach wird er als `SnapshotExpired` (final) ungültig, nicht stillschweigend verlängert (REQ-DEL-031).

| Lesetyp | Isolation |
| --- | --- |
| Punkt-/Prefix-/Range-Reads auf dem Keyspace | SSI (Phantomschutz) |
| ANN-, BM25-Suchen | Snapshot-Isolation via `born_seq`/`died_seq` (INV-INDEX-SNAPSHOT-1); approximative Suchen haben keinen definierbaren Phantomschutz |
| Optional `serializable_index_reads = true` | Index-Lesen registriert einen Collection-Marker im ReadSet; jeder konkurrierende Schreiber derselben Collection löst Konflikt aus (konservativ, kann Abbrüche erhöhen) |

#### C-8.2 Sperrenordnung (INV-LOCK-ORDER-1)

Alle Sperren der Engine (`collections`, Key-Sperren, Embedder, Hooks, Konfiguration, Uhr, Metriken) stehen in **einer** vollständigen, geordneten Hierarchie; nur diese Ordnung gilt beim Erwerb mehrerer Sperren. Mehrschlüssel-Operationen sperren Shards in aufsteigender Reihenfolge und deduplizieren sie; Shard-Hasher nutzen feste Seeds. Der Sperrtyp je Ebene (Mutex oder RwLock) ist dokumentiert und durch einen Test festgeschrieben. Guards von `std`/`parking_lot` DÜRFEN NICHT über `.await` gehalten werden (`await_holding_lock = deny`). Für jede nebenläufige Kernstruktur (SnapshotRegistry, TxBuffer, RevocationLog, HNSW-Arena, Group-Commit, Key-Sperren) existiert ein loom-Modell (§17.1).

### C-9. Vektor-Subsystem (`contextra-vector`) `[K]`

#### C-9.1 HNSW

Arena-Allokation, Backlinks, **adaptives ef**, Startwerte **m = 16, efConstruction = 200, efSearch = 64**, `MAX_SEARCH_K = 1000` (zu kalibrieren). Persistenz: WAL (`IndexIntent`) + Snapshot der Graphstruktur; Wiederaufbau deterministisch (INV-INDEX-APPLY-1: ein Applier in WAL-Reihenfolge, Level aus `hash(doc_id, seed)`; `Clock`/`Rng` über Ports, keine `Instant::now`-Aufrufe wie heute in `core_rebuild.rs`).

#### C-9.2 Mandantentrennung (INV-TENANT-IDX-1)

Je **(Tenant, Collection, Embedding-Modell-Hash)** existiert ein eigener Index (eigene Datei, eigener Schlüssel). Ein gemeinsamer Graph über Mandanten ist unzulässig (Kanten wären mandantenübergreifende Datenflüsse, VETO-F10). Der ACL-Bitset (§9.4) wirkt **innerhalb** eines Tenants (Rolle/Zeile). Suchlatenz und Recall eines Mandanten hängen nicht von anderen ab.

#### C-9.3 Löschung (VETO-F02) und Slot-Wiederverwendung

Nur **Tombstone + Pruning**; Knoten werden nicht „weggeflickt“ (**kein Rewiring der Nachbarn**). Stufe S1: der Vektor-Payload (Vollpräzision **und** SQ8-Code) des Knotens wird **sofort im Slot genullt** und der Index-Snapshot neu geschrieben; der Routing-Knoten bleibt (trägt keine Nutzdaten); Pruning entfernt tote Kanten beim Rebuild. Übersteigt der Tombstone-Anteil `tombstone_rebuild_ratio` (Start 0,2), läuft ein deterministischer Rebuild im Hintergrund.

**INV-HNSW-SLOT-1:** Ein Arena-Slot wird erst wiederverwendet, wenn keine eingehende Kante mehr auf ihn zeigt, oder jede Kante trägt eine Generationsmarkierung. Wird der Einstiegsknoten gelöscht, erfolgt die Neuwahl deterministisch. `partial_rebuild.rs` im Vektor-Crate ist daraufhin zu prüfen, ob es VETO-F02 widerspricht.

#### C-9.4 Gefilterte Suche, ACL und Planner

- **Planner:** Selektivität `s = |erlaubt| / |Index|`. Ist `|erlaubt| ≤ exact_scan_max` (Start 20 000), läuft ein **exakter SIMD-Scan** über die erlaubten IDs (kein Graph, kein Recall-Verlust). Sonst **ACORN-artige gefilterte Traversierung** (Prädikat während der Traversierung, erweiterte Nachbarschaft, ef skaliert mit 1/s bis zu einer Obergrenze).
- **ACL-Bitset (RLS):** pro Rolle ein Bitset berechtigter Vektor-IDs; **Prüfung vor Distanzberechnung**. Modus `strict`: nur berechtigte Knoten werden bewertet *und* expandiert; fällt dadurch die erreichbare Menge unter `k`, greift der exakte Scan statt einer stillen Unterlieferung. Ob das heute so umgesetzt ist, ist offen (O-5-Bezug, Review D2).
- **Seitenkanäle:** Ergebnisse enthalten keine Scores oder Counts nicht berechtigter Knoten; Latenz wird in Klassen gemessen, nicht als konstant zugesichert.

#### C-9.5 Metriken (IoC)

Generischer `Metric`-Trait mit Zero-Sized-Types (`Cosine`, `L2`, `InnerProduct`, `Hamming` …) ⇒ **monomorphisierter Hot-Path ohne dyn-Dispatch**. WASM-Metriken `[R]` dürfen ausschließlich im Top-k-Rerank laufen, nie in der Traversierung.

#### C-9.6 Quantisierung

**SQ8** (Skalar) ist die RAM-Vorstufe; die besten `ef` Kandidaten werden mit den Originalvektoren **exakt nachgerankt**. SIMD-Dispatch AVX-512 > AVX2 > NEON > skalar; skalar ist Orakel (Bit-/ULP-Toleranz getestet); in v1.0 ist der Skalarpfad Standard (E33). Das Ergebnis hängt nie vom Cache-Zustand ab. **RAM-Formel:** `N·D` Byte (SQ8) + Graph `N·R·4` Byte; Beispiel 1 Mio. × 1536 ≈ 1,5 GB SQ8 + Graph statt ≈ 6,1 GB fp32.

#### C-9.7 Skalierung über RAM hinaus `[A]`

RaBitQ (1-Bit-Codes) und ein DiskANN-artiger Modus (SSD-residente Vollpräzisionsvektoren per `mmap`) sind **archiviert** (`vector/diskann`) und erst relevant, wenn der Workload den Designpunkt (§2.1) überschreitet. Wiederaufnahme nur per ADR; Vorbedingungen: INV-ENCRYPT-REST-2, deterministisches Ergebnis unabhängig vom Hot-Set, `MAX_SEARCH_K` auch im Filterpfad, Filterfaktor `gamma = 0` ⇒ Ergebnis identisch zum ungefilterten Pfad, Tie-Breaker Score absteigend/DocId aufsteigend; kein Rewiring (VETO-F02).

#### C-9.8 Adaptiver Overfetch

Liefert ein selektiver Filter weniger als `k` Treffer, MUSS das System in deterministischen Stufen nachladen: `3·k → 6·k → 12·k`, bis `k` Treffer erreicht sind oder eine harte Scan-Obergrenze greift. Abbruch ausschließlich über deterministische Zähler, nie über Zeit-/Wall-Clock-Heuristiken (P28). **Abnahme:** Ein Filter mit 1 % Selektivität liefert exakt `k` Treffer oder nachweislich alle im Snapshot vorhandenen passenden Treffer.

### C-10. Text-Subsystem (`contextra-text`) `[K]`

#### C-10.1 BM25

**BM25** (Startwerte k1 = 1,5, b = 0,75, per Recall-Benchmark zu kalibrieren), **BM25F** mit Feldgewichten, **Block-Max-WAND**. **Statistiken (IDF, Dokumentlängen) werden je Tenant geführt** (INV-TENANT-IDX-1). Der invertierte Index ist segmentiert, über `IndexIntent` konsistent, je Tenant/Collection getrennt; Löschung per Segment-Neuschreibung mit neuem ArtKey, alter ArtKey nach Siegel 2 vernichtet (§12.2.8, §12.8.2 Schritt 5); Segmentdateien liegen unter dem ArtKey. Terme, die nur in gelöschten Dokumenten vorkamen, MÜSSEN aus dem Termwörterbuch verschwinden (sonst bleibt ein Inhaltsrest).

#### C-10.2 Analyzer

`Analyzer`-Port: Unicode-Normalisierung (NFKC), Kleinschreibung, Standard-Tokenisierung; Tokenizer ohne Panics. `GermanAnalyzer` `[R]` (Komposita-Zerlegung per Wörterbuch + Fugen-Heuristik, Lemmatisierung, Umlaute/ß) und Fachvokabulare `[R]` (Synonyme, Abkürzungen, Normbezüge, geschützte Begriffe, nie Stoppwörter) sind optional; der Vorteil für den DACH-Markt ist eine Marktannahme, nicht belegt (O-6). SPLADE `[E]` ist nicht Teil von v1.0.

#### C-10.3 Dokumentformat-Extraktoren `[R]`

PDF-, E-Mail- und DOCX-Ingestion MUSS, sobald aktiviert, mit Zip-Bomb-Schutz und Größenlimits abgesichert sein und in einem eigenen Adapter-Crate außerhalb des Kerns (`contextra-ingest`, §4.9) laufen; Formatparser in Ring 0–3 sind verboten; jeder Extraktor ist Fuzz-Ziel. Bis zur Aktivierung bleibt Ingestion im Kern auf Text/Markdown beschränkt.

### C-11. Embedding, Engine und Abfragepfad (`contextra-engine`, `contextra-rank`) `[K]`

#### C-11.1 Embedding-Identität (INV-EMBED-1/-2)

Es gibt keinen stillen Platzhalter-Embedder: Mock-/Fake-Embedder existieren nur hinter `cfg(test)` oder `test-utils` (Gate über `cargo metadata`); fehlende oder unbekannte Provider-Konfiguration ist ein Startfehler. `EmbeddingBackend ∈ {Candle, None}` in v1.0 (`None` = nur Textsuche; ONNX/Ollama archiviert). Jede Collection persistiert `embedding_model_hash` und Dimension im Collection-Manifest; Einfügen und Suche mit abweichendem Hash werden abgewiesen (`EmbedderMismatch`), auch bei gleicher Dimension; **Modellwechsel = Migration** mit Re-Embedding und vollständigem Neuaufbau des Vektorindex (neuer Index je INV-TENANT-IDX-1, atomarer Umschalt-Commit), nie stiller Mischbetrieb (T-EMB-1). Ein Semantik-Selbsttest („Hund“/„Welpe“ näher als „Hund“/„Steuererklärung“) ist festes CI-Gate je Backend. Die Modellkonfiguration wird aus dem Modellverzeichnis geladen, nie durch Defaults ersetzt.

**Modellidentität (REQ-IDX-015):** `embedding_model_hash` ist `BLAKE3` über die kanonische Kodierung von `(Gewichts-Digest, Tokenizer-Digest, Quantisierungskennung, Pooling-/Normalisierungskonfiguration, Dimension, Präfix-Konvention)`; der Typ `ModelIdentity` liegt in `types`, jedes Backend liefert ihn, die Fassade vergleicht ihn bei jedem `insert`/`search`. Zwei Modelle gleicher Dimension haben verschiedene Hashes; ein Backend ohne berechenbare Identität (z. B. Remote-Endpunkt ohne Gewichts-Digest) ist für persistente Collections unzulässig. **Embeddings sind personenbezogene Daten** (Text ist teilweise rekonstruierbar) und werden von den Löschschichten `VectorIndex` und `CachesAndVault` erfasst (§12.4).

#### C-11.2 Quelle der Wahrheit, Query-Builder, Planner

**Prinzip D1:** Store und WAL sind die einzige Wahrheit. Vektor- und Textindex sind **abgeleitet** und aus dem Store rekonstruierbar. Ein Index darf nie Daten enthalten, die der Store nicht hat; ein Löschvorgang ist erst abgeschlossen, wenn Store **und** Indizes gemessen sind (§12.4); nach Absturz werden Indizes aus dem Store abgeglichen.

- **HybridQueryBuilder:** deklarativ: Vektor + Text + `ScopeConstraint` (Tenant, Domäne, Projekt, Klassifikation) + `ValidityWindow` + `as_of` + Budget. `ScopeConstraint` wird in jede Modalität **vor** der Berechnung gepusht.
- **Planner** (deterministisch, regelbasiert): wählt Modalitäten und Parameter (Text-only, Hybrid) aus Anfrage, Budget und Index-Statistik; gleicher Zustand ⇒ gleicher Plan.
- **Ausführung:** Modalitätszweige im **Bounded-Compute-Pool**; Fusion über `rank/fusion`; kooperative Abbruchpunkte (INV-CANCEL-1).
- **Bi-temporal:** Gültigkeitszeit × Transaktionszeit („Was galt am Datum X, wie wir es am Datum Y wussten?“).
- **Provenienz:** Jedes Ergebnis trägt Lineage (WAL-Seq, Quelle, Chunk, Embedding-Version), `trust` und `source` (INV-TRUST-1).

#### C-11.3 `remember`

1. Validierung (Schema, Größe, Limits §11.8, Mandant aus `RequestContext`); `trust`/`source` setzen; `MemoryType` (Default `Semantic`).
2. Chunking nach §11.7.
3. Embedding über `EmbeddingProvider` **außerhalb von Sperren** (Blocking-Pool, §6; `max_embed_batch`, bricht bei Abbruch ab); das Ergebnis ist ein aufgezeichneter Wert (INV-NONDET-1). Große Werte: Blob-Segment vor dem Commit.
4. **Ein WAL-Batch:** Primärdaten + `IndexIntent`s (Vektor, Text) + Lineage + `SidecarIntent` (Audit) → durabler Flush (Commit-Punkt, `fsync`-Fehler propagieren; „WAL First“).
5. Der Applier wendet die Indexe an; bei Fehler gilt der Index als „nachzuziehen“ und wird beim nächsten Start abgeglichen.
6. Quittung `RememberReceipt` mit Dokument-ID, Sequenznummer (Provenienz-Anker), `embedder_id`.

#### C-11.4 `recall`

1. `RequestContext` und Snapshot-Sequenz festlegen; Planner; `ScopeConstraint`/ACL-Bitset.
2. Query-Embedding (Blocking-Pool).
3. Vektorsuche (mandantengetrennter Index, filterbewusst, ACL vor Score) **und** BM25 parallel am Snapshot, beide mit `max_results`-Grenze (Gate `check-max-results-unbound`).
4. Fusion über `rank/fusion` (RRF, Normalisierung, Provenienz), deterministisch bei gleichem Seed.
5. **Lesesperre:** Treffer, deren Dokument im `ErasedSet` steht, werden entfernt, **bevor** irgendetwas zurückgegeben wird (REQ-DEL-011).
6. Ausgabegrenze `max_response_tokens` (Start 4.000); Kürzung wird ausdrücklich gekennzeichnet; Ergebnis `{content, trust, source, lineage}`; Audit-Eintrag.

#### C-11.5 Fusion und Provenienz (`rank/fusion`)

**RRF** mit `k = 60`; deterministische Tie-Breaker: **Score absteigend, DocId aufsteigend** (REQ-IDX-009). Die Basis-Erklärung je Treffer (Modalität, Score, WAL-Seq, Quelle) ist die Grundlage von M4 und entsteht ausschließlich hier (Kantenregel 10, §4.4). Ohne `Calibrator` (Archiv) gelten rohe RRF-Scores und keine „Unsicher“-Ausgabe.

#### C-11.6 Retention-Reaper (INV-RETENTION-1)

Ein periodischer Hintergrund-Worker entfernt Einträge, die eine **opt-in `RetentionPolicy`** der Collection nach Karenzzeit (Standard 7 Tage im Zustand `PendingDeletion`, per `restore` rückholbar) erfasst. Batches sind begrenzt (Start 100 Dokumente/Tick), Herunterfahren per Cancellation. Jede Entscheidung wird **als Record ins WAL geschrieben** (Zeit aus `Clock`, Ursache, Policy-Version); die Entfernung folgt der Löschkette (§12.8). Eine Policy ohne Dry-run-Lauf und Audit-Eintrag ist nicht aktivierbar. Ranking-Abwertung (Decay) löscht nie. Ein storage-interner TTL-Mechanismus auf LSM-Ebene, falls vorhanden, ist davon abzugrenzen und zu dokumentieren. *Offene Zuordnung:* ob Reaper und `MemoryType` in `engine`/`types` leben (WP 3.2).

#### C-11.7 Chunking (MUSS)

Jedes Dokument wird vor Indexierung über einen `MarkdownChunker` zerlegt: (1) Zielgröße 512 und Obergrenze 1024 Token (Startwerte, per Recall-Benchmark zu kalibrieren); (2) Overlap bei überlangen Absätzen im Sliding-Window-Fallback 10–15 % der Zielgröße; (3) **atomare Blöcke:** Codeblöcke und Tabellen werden nicht mitten im Block getrennt; überschreitet ein Block die Obergrenze, wird er an Zeilen-/Zeilenblockgrenzen geteilt; (4) **Determinismus:** gleicher Text und gleiche Konfiguration ⇒ identische Chunk-Grenzen und Chunk-IDs (REQ-IDX-012); (5) jeder Chunk trägt `source_span` (Bytebereich im Quelldokument) für die Lineage. Ingestion ist ohne diese Konfiguration nicht aktivierbar. Das kontextuelle Chunk-Präfix (LLM-generiert) ist `[A]`/aus.

#### C-11.8 Limits und Quoten (DoS-Schutz, Startwerte)

`max_document_bytes` 16 MiB, `max_chunks_per_document` 4.096, `max_batch_ops` 10.000, `max_collections_per_tenant` 256, `max_in_flight_per_principal` 8. Überschreitung ⇒ strukturierter, nicht retrybarer Fehler `LimitExceeded`; einzige Ausnahme `max_in_flight_per_principal` ⇒ retrybares `Busy`. **Quoten (REQ-TEN-005):** `tenant_quota_bytes`, `tenant_quota_docs`; in `EnterpriseShared`/`EnterpriseRegulated` Pflichtwerte, sonst optional; Überschreitung ⇒ `QuotaExceeded` (final, bis Platz geschaffen ist); die Quote zählt freigegebenen Platz erst nach S2.

#### C-11.9 Determinismus-Versprechen (Reduktion)

Zugesichert wird: **gleiche Plattform, gleiche Eingabe, gleicher Seed → gleiche Ergebnisse**. Plattformübergreifende Bit-Identität wird **nicht** versprochen (SIMD-Rundung). Tests: T-DET-1 (HNSW-Aufbau), `check-recall-stability`, `check-nan-hot-loop`.

#### C-11.10 Offene Inhaltsfragen

Retention-Reaper/`MemoryType`-Ort (§11.6), BM25-Morphologie für deutsche Komposita (Marktannahme), Planner und Overfetch-Feinheiten (§9.4, §9.8). Sie werden in WP 3.2 beim Schneiden der Engine erfasst und entschieden.

### C-12. Kryptografie, Löschung, Löschbericht, Audit (`contextra-crypto` und Schicht-Crates) `[K]`

#### C-12.1 Rechtlicher und fachlicher Rahmen (keine Rechtsberatung)

> **Rechtshinweis.** Technische Spezifikation, keine Rechtsberatung. **Software kann eine Rechtswirkung weder herbeiführen noch garantieren.** Sie kann den technischen Zustand herstellen, den die vertretenen Rechtsauffassungen verlangen, diesen Zustand messen und belegen und ehrlich benennen, was sie nicht erreicht. Die rechtliche Bewertung trägt der Verantwortliche. Punkte „P-xx“ (§25.1) sind vor jeder Außenkommunikation fachanwaltlich zu prüfen. Das Ziel „rechtlich nicht anfechtbar“ wird deshalb **technisch** als „unter jeder plausiblen Auffassung bedient, gemessen, mit benannten Annahmen“ operationalisiert (§12.1.3).

**Belegstärke:** **[R]** verbindliches Recht · **[B]** Behördenauslegung (EDPB, DSK, Landesbehörden, Standard-Datenschutzmodell, nicht bindend) · **[P]** Fachliteratur/Praxis/Norm · **✔** an der Primärquelle geprüft · **◐** nur Sekundärquelle · **○** allgemeiner Kenntnisstand · **⚠** umstritten/offen.

##### C-12.1.1 Grundsatz

1. Art. 17 DSGVO verpflichtet zur Löschung **ohne unangemessene Verzögerung** (unverzüglich), auch ohne Antrag, definiert „Löschen“ aber nicht und schreibt keine Methode vor [R]. Löschen ist selbst Verarbeitung (Art. 5, 32). Das Standard-Datenschutzmodell (Baustein 60) versteht Löschen als **Unkenntlichmachen**: Der angeordnete Informationsverlust muss tatsächlich wirken [B ✔]. Technische Unmöglichkeit rechtfertigt keinen Verstoß (EDPB 02/2025 v2.0, Rn. 50) [B ✔].
2. Die Aufsichten fordern **Wirksamkeit und Nachweisbarkeit** (EDPB 02/2025 Rn. 63; CEF-Bericht 2025: Löschung verifizieren und nachweisen können) [B ✔]. Art. 5 Abs. 2, 24, 32 verlangen Rechenschaft [R].
3. Es ist **keine höchstrichterliche Entscheidung bekannt**, die Schlüsselvernichtung als Löschung anerkennt (P-01, P-02).
4. **Konstruktionsgrundsatz Verteidigung in der Tiefe.** Contextra stellt für jede erreichbare Datenschicht den Zustand her, den die **strengste** Auffassung verlangt (physische Bereinigung, Ebene A und Hygiene). Für nicht erreichbare Reste (Flash, Backups) kommt ein **zweiter, unabhängiger Mechanismus** hinzu (Schlüsselvernichtung mit Epoch Seal und Backup-Trennung, Ebene B). Jede Aussage im Bericht nennt ihre Voraussetzungen und Grenzen.
5. **Beide Mechanismen sind notwendig, keiner genügt allein.** Ohne Ebene A bleiben Klartext und invertierbare Suchstrukturen (Vektoren, Ghost Vectors). Ohne Ebene B bleiben Flash-Reste und Kopien lesbar, weil Überschreiben auf SSD nicht zuverlässig wirkt (Wear-Leveling). Ohne die Backup-Spur wären Backups eine ungelöschte Kopie.

##### C-12.1.2 Quellenlage

Die Spalte „Prüfung 2.3“ trennt, was bei der Quellenprüfung vor 2.3 geprüft wurde, von dem, was bei der Erstellung dieser Fassung am 26.10.2026 **zusätzlich** gegengeprüft wurde. Eine Quelle mit ◐ oder ○ DARF NICHT in Außentexte übernommen werden, bevor der Volltext geprüft ist (P-14).

|Quelle|Aussage (paraphrasiert)|Status|Prüfung vor 2.3|Prüfung 2.3|
|---|---|---|---|---|
|EDPB Guidelines 02/2025 Blockchain, v2.0 final (07.07.2026)|Rn. 51: Nach Schlüsselvernichtung sind verschlüsselte Daten unverständlich, solange der Algorithmus nicht gebrochen ist, die Entschlüsselung nicht fortschreitet und der Schlüssel nicht kompromittiert war; verschlüsselte Daten bleiben personenbezogen. Rn. 52: Hash mit Salt oder Schlüssel bleibt personenbezogen, nach Löschung des Schlüssels nicht mehr verknüpfbar, wenn dieser nicht kompromittiert war. Rn. 103–104: „effektive Anonymisierung“ durch Löschung der Off-chain-Daten. Rn. 85, 96, Empf. 13: Notfallplan für Algorithmenwechsel, periodische Neubewertung.|final; Analogie (Blockchain), keine unmittelbare Aussage zu RAG|✔|nicht erneut geöffnet|
|EDPB Guidelines 02/2026 Anonymisierung, Entwurf v1.0 (07.07.2026), Konsultation bis 30.10.2026|Kriterien No Record Isolation, No Linkage, No Inference; kontextueller Ansatz; Verschlüsselung ist reversibel und kein Ersatz für Anonymisierung; Anonymisierung ist selbst Verarbeitung.|Entwurf|◐|offen (P-14)|
|EDPB Guidelines 01/2025 Pseudonymisierung|keine Endfassung gefunden|Entwurf|◐|offen|
|EDPB CEF 2025 Recht auf Löschung|764 Verantwortliche, 32 Aufsichten, 9 Formalverfahren, sieben wiederkehrende Probleme (u. a. Backups, Anonymisierung). Issue 6: Löschung in Backups nicht immer ratsam, dann aber Verfahren, um Löschungen nachzuverfolgen und auf wiederhergestellten Systemen umzusetzen. Best Practice: Daten durch Zufallszeichen ersetzen. Empfehlung: anerkannte Standards befolgen, Löschung verifizieren und demonstrieren. Issue 7: einfache Pseudonymisierung oder Maskierung genügt nicht. Issue 4: Einschränkung nach Art. 18, wenn die Entscheidung aussteht.|Bericht|✔|**bestätigt**: angenommen 10.02.2026, veröffentlicht 18.02.2026 (Pressemitteilung und Berichts-PDF auf edpb.europa.eu gefunden)|
|NIST SP 800-88 Rev. 2 (26.09.2025), FAQ vom 16.07.2026|Cryptographic Erase (CE) nur bei: kein früherer Klartext, ausreichende Schlüssellänge und Entropie, Zeroization **aller** Zielschlüssel (DEK, Wrapping, Master); Backup- und Escrow-Kopien brauchen eigene Regeln; Teil-CE nur bei absoluter Sicherheit; Verification und Validation getrennt; Zertifikat mit Methode, Technik, Validierungsentscheidung, Concurrence. Rev. 1 ist zurückgezogen; technikspezifische Verfahren verweist Rev. 2 an IEEE 2883.|Norm/FAQ|✔ (FAQ), ◐ (Norm)|**bestätigt**: Veröffentlichung 26.09.2025, Rev. 1 zurückgezogen, CE-Vorbedingung „Verschlüsselung seit Bereitstellung aktiv, kein Klartext je geschrieben“ (Sekundärquellen); Volltext der Norm nicht geöffnet|
|ISO/IEC 27040:2024|alle Kopien der Zielschlüssel sind zu bereinigen, sonst gilt es nicht als CE|Norm|◐|offen|
|IEEE 2883:2022|technikspezifische Sanitisierungsverfahren je Medium|Norm|○|nur als Verweis von NIST Rev. 2 bestätigt|
|BayLfD 30. TB (2020) Nr. 12.5|Zeitversetzte Backup-Löschung zulässig bei technischer Unmöglichkeit, begründeter Löschfrequenz, Zugriff nur über Restore, Nachlöschung bei jedem Restore, Dokumentation|Aufsichtsauffassung|◐|in 2.4 am Primärtext nicht wiederholt geprüft (P-14); trägt keine konkrete Frist|
|SDM Baustein 60 V1.0a|Löschen als Unkenntlichmachen|Aufsichtsauffassung|✔ (Auszug)|–|
|EuGH C-413/23 P (EDPS/SRB), 04.09.2025|Relativer Personenbezug; für den Verantwortlichen bleiben pseudonyme Daten personenbezogen, für Empfänger ohne Mittel kann dies anders sein|Urteil|◐|offen|
|AG Lörrach, 03.03.2025, 3 C 1099/24|Kein Anspruch der betroffenen Person auf Löschnachweis ohne Streitfall|Instanzgericht|◐|offen|
|OLG Dresden, 15.10.2024, 4 U 940/24|Verantwortlicher muss vom Auftragsverarbeiter aussagekräftige Löschbestätigung einholen|Berufungsgericht|◐|offen|
|OLG Dresden, 03.02.2026, 4 U 292/25|Anonymisierung als „Minus“ zur Löschung auf Verlangen|Berufungsgericht|◐ nur Sekundärquelle|offen|
|DSB Österreich, Bescheid 05.12.2018|Unkenntlichmachung zulässig, völlige Irreversibilität nicht nötig|Behörde|◐|offen|
|eIDAS Art. 41, 42|Vermutung für Datum und Datenintegrität bei qualifiziertem Zeitstempel; § 371a ZPO betrifft qualifizierte Signaturen, nicht Zeitstempel unmittelbar|Recht|◐|offen|
|Ghost Vectors, arXiv 2606.18497|Soft-gelöschte HNSW-Vektoren bleiben rekonstruierbar; Ghost-Knoten verändern Treffermengen und erzeugen Timing-Kanal|Preprint|✔|–|

##### C-12.1.2a Quellenbereinigung und Begründungsbasis (2.4)

Mehrere in der Fachdiskussion kursierende Aussagen sind falsch zugeordnet oder nicht belegt. Das Design stützt sich **nicht** auf sie. Die Tabelle nennt je Aussage die Wertung und die Folge für das Design.

|Aussage|Wertung|Folge im Design|
|---|---|---|
|„OLG Dresden 4 U 1278/21 erkennt Anonymisierung als ‚Minus zur Löschung‘ an und stützt Crypto-Shredding“|**Falsch zugeordnet.** Die Entscheidung (14.12.2021) betrifft eine Inkasso-Streitigkeit mit namensgleicher Person und das Zusammenwirken von Aufbewahrungspflichten und Löschanspruch (Prüfung je Datum). Die „Minus“-Formel stammt aus OLG Dresden 4 U 292/25 (03.02.2026, Instanzgericht, ◐) und nennt Schlüsselvernichtung nicht.|Contextra stützt keine Außenaussage auf Anonymisierungsurteile; Löschung wird gemessen (Ebene A) und zusätzlich kryptografisch abgesichert (Ebene B), nicht rechtlich umgedeutet (§12.1.1 Punkt 4).|
|„Gerichte oder Aufsichten haben Crypto-Shredding als Löschung anerkannt“|**Nicht belegt.** Keine Entscheidung und keine Behördenaussage dazu gefunden; die einzige EDPB-Aussage (02/2025 Rn. 51, Blockchain) ist an Bedingungen geknüpft und eine Analogie.|Klassen mit benannten Annahmen (§12.5); „kryptografisch gelöscht“ nie ohne Custody und Annahmen (§12.11).|
|„BSI TR-02102 und ISO/IEC 27040 empfehlen oder akzeptieren kryptografisches Löschen“|**Nicht belegt.** TR-02102 regelt Algorithmen und Schlüssellängen, nicht Löschen; die Akzeptanz durch ISO/IEC 27040 ist nicht am Normtext geprüft.|TR-02102 nur für die Algorithmenwahl (§12.2.1); als Löschbeleg dienen Messung und NIST-Bedingungen (LA-3, LA-15).|
|„Aufsichten in Bayern, Baden-Württemberg und Berlin akzeptieren Backup-Rotation von 30, 60 oder 90 Tagen“|**Keine Primärquelle.** Belegt ist nur: Backups müssen nicht sofort überall verändert werden, nach einer Rücksicherung ist unmittelbar erneut zu löschen und der Vorgang zu überwachen (SDM Baustein 60, M60.S10, M60.P08; CEF 2025 Issue 6).|`key_backup_max_age` und `erased_set_retention` sind **eigene Setzungen** des Produkts, keine behördliche Vorgabe (§12.17, P-06). Die Wirkung hängt nicht an einer behördlichen Frist, sondern am Ablauf des Schlüssel-Backups.|
|„§ 35 BDSG analog rechtfertigt Nicht-Löschen in Backups und Vektorindizes“|**Trägt nicht.** § 35 Abs. 1 BDSG gilt nur bei nicht automatisierter Verarbeitung und nicht bei unrechtmäßiger Verarbeitung; Zweifel an der Unionsrechtskonformität.|Kein Ausnahmepfad „unverhältnismäßiger Aufwand“ im Design; granulare Löschbarkeit ist Konstruktionsziel (§12.2.5).|
|„EDPB 28/2024 stellt fest, dass Embeddings personenbezogen sind“|**Zu weit.** Die Stellungnahme prüft die Anonymität trainierter **KI-Modelle**, nicht eines Vektorindex. Die DSK-Orientierungshilfe RAG (10/2025) behandelt die Vektordatenbank als Verarbeitungsgegenstand mit Rechtsgrundlage, ohne Invertierbarkeitsprüfung; die Hamburger Gegenposition betrifft Tokens und Embeddings **im** Modell. Inversionsangriffe auf Text-Embeddings sind technisch belegt (Fachliteratur, ○).|Embeddings gelten **vorsorglich als personenbezogen** (P-05, LA-2); Schichten `VectorIndex` und `CachesAndVault` sind Pflicht (§12.4).|
|„Die DSK-Orientierungshilfe RAG nennt Crypto-Shredding“|**Falsch.** Sie hält Vektoreinträge für adressierbar und löschbar, erwähnt Crypto-Shredding nicht und nennt Betroffenenrechte im Sprachmodell „weitgehend ungelöst“.|Löschung im Modell und beim Modellanbieter ist Nicht-Gegenstand (§12.20, `ModelParameters`).|
|„AI Act Art. 12 verlangt die Protokollierung aller Ein- und Ausgaben“|**Überzeichnet.** Nur Hochrisiko-Systeme; verlangt Ereignisprotokolle; Art. 26 Abs. 6 stellt die Aufbewahrung unter den Vorbehalt des Unionsrechts, insbesondere des Datenschutzrechts. Hochrisiko-Fristen laut Digital Omnibus auf 02.12.2027 verschoben (◐, Amtsblatt prüfen).|Logs, Audit und Sidecars enthalten nur Referenz-Hashes und Zähler (INV-LOG-1, §12.10); kein Konflikt mit der Löschung.|
|„Buchungsbelege 10 Jahre, § 203 StGB als Aufbewahrungsnorm“|**Veraltet bzw. falsch.** Buchungsbelege 8 Jahre seit 01.01.2025 (§ 147 Abs. 3 AO, § 257 Abs. 4 HGB; Banken, Versicherungen weiter 10 Jahre); Bücher, Abschlüsse 10 Jahre; Handels- und Geschäftsbriefe 6 Jahre; Patientenakte 10 Jahre nach Behandlungsabschluss (§ 630f Abs. 3 BGB); § 203 StGB ist Schweigepflicht.|`Restricted` je Datum, Archiv getrennt vom Retrieval-Index (§12.8.6, P-09, P-22).|
|„Der Löschnachweis darf nicht aufbewahrt werden“ / „Nachweis ist nie geschuldet“|**Differenziert.** AG Lörrach (3 C 1099/24, erstinstanzlich, ◐): kein Anspruch der betroffenen Person auf individuellen Nachweis ohne Streitfall; OLG Dresden 4 U 940/24 (◐): der Verantwortliche muss sich die Löschung vom Auftragsverarbeiter nachweisen lassen; EDPB CEF 2025: Löschung verifizieren und nachweisen können.|Bericht datensparsam (Referenz-Hash statt ID, §12.10), Bescheinigung ohne Identifikatoren (§12.7.8).|

**Tragende Begründungen des Designs** (Quelle in Klammern):

1. *Löschen = Unkenntlichmachen, Kopien eingeschlossen* (SDM Baustein 60: vollzogen erst, wenn keine Kopie oder Replikation beim Verantwortlichen oder Auftragsverarbeiter besteht; Wipe unter Beachtung von Wear-Leveling) ⇒ zehn Schichten, Oberflächen-Inventar, Hygiene.
2. *Technische Unmöglichkeit entschuldigt nicht* (EDPB 02/2025 Rn. 50) ⇒ Löschbarkeit ist Architekturvorgabe (DocDek je Einheit), kein Betreiberproblem.
3. *Nachweisbarkeit schuldet der Verantwortliche* (Art. 5 Abs. 2, 24, 32 DSGVO; EuGH C-340/21; EDPB CEF 2025) ⇒ Bericht aus Messung, Verification und Validation getrennt (§12.7.7).
4. *Kryptografisches Löschen nur mit Zeroization aller Zielschlüssel und ohne früheren Klartext* (NIST SP 800-88 Rev. 2) ⇒ Nadelscan über alle Dateien, Epoch Seal, Heap-Canary, Härtung (§12.18).
5. *Restrisiko Algorithmus, Obsoleszenz, Quantencomputer, periodische Neubewertung, Notfallplan* (EDPB 02/2025 Rn. 85, 96) ⇒ Krypto-Agilität, `doctor crypto-drill` (§12.2.9).
6. *Hash mit Schlüssel bleibt personenbezogen, bis der Schlüssel vernichtet ist* (EDPB 02/2025 Rn. 52) ⇒ Referenzschlüssel mit Ablauf (§12.10).
7. *Relativer Personenbezug* (EuGH C-582/14, C-413/23 P) ⇒ Perspektivenmatrix (§12.1.4): das Urteil „unkenntlich“ gilt je Stelle und wird je Stelle ausgewiesen.

##### C-12.1.3 Rechtsstabilitätsmatrix

Contextra MUSS gleichzeitig unter allen plausiblen Auffassungen bestehen. Die Matrix ordnet jedem Maßstab den erfüllenden technischen Zustand zu (Klassen: §12.5).

|Auffassung|Maßstab|Erfüllender Zustand|Rest|
|---|---|---|---|
|T1 Strenge Vernichtung|alle Kopien im Einflussbereich physisch entfernt (SDM, EDPB Rn. 50, 63)|Klasse **M**: P plus Hygiene (physische Kompaktion, WAL-Scrub, Checkpoint-Rewrite, Index-Neuaufbau)|Flash-Reste, externe Backups (ausgewiesen)|
|T2 Unkenntlichmachung|niemand ist mit vertretbarem Aufwand zur Identifizierung in der Lage (SDM, DSB, OLG Dresden 4 U 292/25, Entwurf 02/2026)|Klasse **C** plus Restmetadaten-Selbsttest (§12.7.6) plus Perspektivenmatrix|Aufwandsbewertung ist Rechtsfrage (P-01, P-02)|
|T3 Cryptographic Erase|NIST-Bedingungen, ISO/IEC 27040 (EDPB Rn. 51 analog)|C2/C3 mit CE-Traceability; C1 mit Annahme A-FLASH|Hardware-Restrisiko bei K1|
|T4 Einschränkung/Aufbewahrung|Art. 18, Art. 17 Abs. 3 (CEF Issue 4)|Klasse **L**, `Restricted`, Legal Hold|Dauer ist Rechtsfrage (O-12)|
|T5 Backups|Löschungen nachverfolgen und beim Restore umsetzen (CEF Issue 6, BayLfD 12.5)|Zwei-Spuren-Backup, Restore-Guard, `ErasedSetMirror` (§12.17)|Externe Backups, die Contextra nicht sieht|

##### C-12.1.4 Perspektivenmatrix

Der Entwurf 02/2026 prüft Anonymität aus Sicht jeder Stelle mit realistischem Zugang. Der Bericht trägt deshalb ein Urteil **je Perspektive**, erzeugt aus den gemessenen Werten (nie statisch).

|Perspektive|Zugang|nach L|nach P|nach C|
|---|---|---|---|---|
|Q1 Leser (MCP-Client, API, `as_of`, Export)|Lesepfade|gesperrt|gesperrt|gesperrt|
|Q2 Betreiber mit Prozess- und Dateizugriff **und** Passphrase|Engine, Dateien, Keystore|Klartext nur über gesperrte Lesepfade|keine Klartextstruktur mit Scope-Inhalt|persistente Kopien unter vernichtetem DocDek nicht lesbar|
|Q3 Dateiangreifer ohne Passphrase|Datenverzeichnis|verschlüsselt|verschlüsselt|verschlüsselt|
|Q4 Medienangreifer mit Flash-Resten, ohne Passphrase|Chip-off, Controller-Reste|verschlüsselt|verschlüsselt|verschlüsselt|
|Q5 Medienangreifer mit Flash-Resten **und** Passphrase|wie Q4 plus Schlüssel|lesbar|lesbar (Chiffrat)|**C1: möglich** (A-FLASH). **C2/C3: nicht möglich**|
|Q6 Besitzer eines Daten-Backups ohne Schlüssel-Backup|Backup-Dateien|verschlüsselt|verschlüsselt|nicht lesbar|
|Q7 Besitzer eines Schlüssel-Backups älter als die Löschung|Schlüssel-Backup|lesbar bis Fensterende|lesbar bis Fensterende|unlesbar nach `key_backup_max_age`; Restore-Guard greift beim Einspielen|
|Q8 Empfänger/Dritte (Exporte, LLM-Anbieter, Auftragsverarbeiter)|außerhalb von Contextra|nicht erfasst|nicht erfasst|nicht erfasst (§12.20, `RecipientLedger` §12.19)|

##### C-12.1.5 Technische Anforderungen aus dem Rahmen

(Die Kennungen heißen in 2.3 `LA-n`, um die Kollision mit den Architekturregeln A1–A10 (§3.3) zu beseitigen; Fassung 2.2.1 führte beide als „A1…“.)

|ID|Anforderung|Rechtslage|Umsetzung|
|---|---|---|---|
|LA-1|Nach der Löschung keine Person identifizierbar, Informationsgehalt nicht mit vertretbarem Aufwand reproduzierbar; auch Kopien beim Verantwortlichen erfasst.|[B ✔]|§12.4, §12.5, §12.7, §12.16|
|LA-2|Alle Repräsentationen im Wirkungsbereich: Quelle, Chunk, Vektor, Volltextindex, Graph, Cache, Log, interne Checkpoints, Schlüsselmaterial.|[P]; Embeddings ⚠|§12.4 (zehn Schichten)|
|LA-3|Alle Kopien der Zielschlüssel sind erfasst (Wrap-Blobs, Epochen, Schlüssel-Backups).|[P ◐] NIST, ISO 27040|§12.2.4 (Epoch Seal), §12.17 (Spur K), Nadelscan §12.7.3|
|LA-4|Löschgranularität ohne Kollateralverlust.|[B ✔]|DocDek je Betroffenen-Einheit (§12.2.5, §12.15)|
|LA-5|Verbleibende Metadaten (Längen, Header, IDs, Zeitstempel) identifizieren nicht wieder; Pseudonyme laufen ab.|[P]; Entwurf 02/2026|Padmé, verschlüsselte Header (§12.2.6), §12.7.6, §12.10|
|LA-6|Löschung ist verifizierbar und nachweisbar; Verification und Validation getrennt.|[R/B ✔]|§12.7|
|LA-7|Bericht ist datensparsam.|⚠ umstritten|§12.10, §12.7.8|
|LA-8|Nach Rücksicherung unmittelbar erneut löschen; Überwachung; Rücksicherung nur innerhalb des Horizonts.|[B ✔]|§12.9, §12.17|
|LA-9|Aufbewahrungspflichten: Einschränkung statt Löschung.|[R] + ⚠|`Restricted` (§12.8.6)|
|LA-10|Krypto-Agilität: Austauschplan, periodische Neubewertung.|[B ✔ Analogie]|§12.2.9, REQ-DEL-028|
|LA-11|Unverzüglich; Monatsfrist; Alarm- und Eskalationsschwellen.|[R ○]|§12.8, §12.19, §15.5|
|LA-12|Ehrliche Produktaussagen; keine Rechtsbehauptung.|⚠|§2.6, §12.11|
|LA-13|Löschkonzept dokumentierbar (Löschinventar, Vorlage `doctor export-concept`).|[B ✔]|§12.14|
|LA-14|Empfänger werden informiert; Auftragsverarbeiter-Löschung nachweisbar.|[R ○/◐]|`RecipientLedger` (§12.19), Bescheinigung (§12.7.8)|
|LA-15|Prozessspeicher, Dumps, Swap sind adressiert; NIST-Vorbedingung „kein früherer Klartext auf dem Medium“.|[P ◐]|§12.18|
|LA-16|Wirkung ist unter Fehlern und Abbrüchen garantiert (Crash, ENOSPC, `fsync`-Fehler, Restore).|[B] (Rechenschaft)|§12.8.4, §7.10, §12.12|

**Pflichten des Betreibers (nicht ersetzbar durch Software).** `doctor export-concept` erzeugt eine **nicht rechtsverbindliche** Vorlage: Verzeichnis der Löschschichten und Löschfristen je Collection; Backup-Richtlinie (Daten-Backup, Schlüssel-Backup, `key_backup_max_age`, Ablageorte, `ErasedSetMirror`-Ziel); Textbaustein für Verzeichnis der Verarbeitungstätigkeiten und Auftragsverarbeitung (Löschbestätigung, OLG Dresden 4 U 940/24); Ablauf bei Restore. Ob und an wen Bescheinigungen ausgeliefert werden, entscheidet der Verantwortliche.

#### C-12.2 Primitive, Schlüsselhierarchie, Verwahrung

##### C-12.2.1 Primitive

**AES-256-GCM-SIV** (nonce-misuse-resistant AEAD), **Argon2id** (Passphrase-KDF, INV-KDF-1), **HKDF** (nur aus hochentropischem Material, nie aus Passphrasen), **HMAC-SHA256** (WAL-/Sidecar-Ketten, Referenz-Hashes), **Ed25519** (Berichtssignatur), **SHA-256/BLAKE3** (Hash). Zufall für Schlüssel, Nonces, Salts stammt aus dem System-RNG (P28-Ausnahme). Geheimnisse sind `zeroize`-geschützt (`Zeroizing<[u8; 32]>`, kein `Clone` ohne Zeroisierung, konstantzeitiger Vergleich). Kryptografische Primitive werden nie selbst gebaut. Schlüsselmaterial erscheint nie in `Debug`, Logs oder Fehlertexten.

##### C-12.2.2 Hierarchie

```
Wurzel (Custody-Slot K1 | K2 | K3)                    EffaceableKeySlot (§12.2.3)
 └─ KEK[coll, epoche]       je Collection und Epoche, gewrappt unter der Wurzel
     ├─ ShardKek[shard, gen]  je KeyTable-Shard (§12.2.4), gewrappt unter KEK
     │    └─ DocDek[einheit]  je Betroffenen-Einheit, gewrappt unter ShardKek
     ├─ ArtKey[artefakt]    je Index-Artefakt (Text-Segment, Graph-Snapshot), gewrappt unter KEK
     └─ K_ref[tenant, ref_epoche]   Referenzschlüssel (§12.10), zufällig, gewrappt unter KEK
K_eng                       Struktur und Integrität (keine Nutzdaten), HKDF aus der Wurzel
K_sc/*                      Zweckschlüssel der Sidecar-Ketten (HMAC), HKDF aus K_eng
```

Jeder Schlüssel hat genau einen Zweck und ein Label im Zweckregister (INV-KEYPURPOSE-1, §12.3). **Die Sidecar-Ketten und `K_eng` überleben jede Löschung von Nutzdaten und enthalten keine Nutzinhalte** (INV-LOG-1). Das ist Voraussetzung dafür, dass Berichte und Ketten nach der Löschung prüfbar bleiben.

##### C-12.2.3 Custody-Stufen und Port `EffaceableKeySlot`

|Stufe|Wurzel|Klasse|Status in v1.0|
|---|---|---|---|
|K0|keine Verschlüsselung (nur Entwicklung, `doctor` rot)|nur L, P|ausgeliefert|
|K1|Argon2id aus Passphrase (m ≥ 64 MiB, t ≥ 3, ≥ 250 ms), Software-Keystore|C1|ausgeliefert|
|K2|Siegelschlüssel in manipulationsresistentem, nicht exportierbarem Modul (z. B. TPM-2.0-Objekt, HSM); Eignung je Plattform ist Spike S-K2|C2|**Port vorgesehen**, nicht ausgeliefert bis Konformitätssuite besteht|
|K3|externer KMS/HSM mit Zeitfenster-Löschung und Vernichtungsattest|C3|wie K2|

Bis zur bestandenen Konformitätssuite liefern K2 und K3 `CapabilityUnsupported`, **nie** einen Stub, der Erfolg meldet (REQ-DEL-027). Hardware-Effaceable-Konzepte (z. B. Apple Effaceable Storage) sind Schnell-Löschen eines Volumenschlüssels und kein Mechanismus für beliebige Einzelschlüssel; eine Plattform gilt nur als K2, wenn die Suite nachweist, dass Schlüsselmaterial nach Vernichtung nicht aus Resten rekonstruierbar ist. Damit ist E27 (K2/K3 gestrichen) **teilweise zurückgenommen** (E44, §20).

```rust
pub trait EffaceableKeySlot: Send + Sync {
    fn custody(&self) -> Custody;                         // K1 | K2 | K3 (K0 hat keinen Slot)
    fn wrap(&self, material: &Zeroizing<[u8; 32]>, ad: &[u8]) -> Result<WrappedBlob, KeyError>;
    fn unwrap(&self, blob: &WrappedBlob, ad: &[u8]) -> Result<Zeroizing<[u8; 32]>, KeyError>;
    fn rotate(&self) -> Result<SlotGeneration, KeyError>; // neue Wurzel-Generation; die alte bleibt bis destroy
    fn destroy(&self, generation: SlotGeneration) -> Result<DestroyAttestation, KeyError>;
}
```

`DestroyAttestation` enthält Custody, Verfahren (Zeroization), Zeitpunkt (aus `Clock`) und einen Beleg der Implementierung. **Nur K1 liefert einen lokalen Beleg ohne Dritt-Attest** (Selbstauskunft, kein Beweis der Abwesenheit auf dem Medium). `KmsProvider` bleibt der Port für Erzeugen und Ableiten; er hat keinen Default und keinen Fallback (REQ-PORT-2).

##### C-12.2.4 KEK-Epochen und Epoch Seal

**Problem.** Wird nur der DocDek vernichtet, liegt sein gewrappter Blob als Rest auf dem Flash. Wer KEK und Passphrase besitzt, kann ihn entschlüsseln. NIST Rev. 2 zählt Wrapping-Schlüssel zu den CE-Zielschlüsseln, ISO/IEC 27040 verlangt die Bereinigung aller Schlüsselkopien.

**Epoch Seal.**

1. erzeugt `KEK[coll, e+1]`, gewrappt unter der Wurzel;
2. erzeugt für jeden **geänderten Shard** (Shard mit Vernichtung seit dem letzten Seal) eine neue `ShardKek`-Generation, wrappt dessen lebende DocDeks neu und vernichtet die alte Generation; wrappt außerdem **alle** ShardKeks, ArtKeys und `K_ref` unter `KEK[e+1]` neu (Re-Wrap, kein Datenrewrite);
3. schreibt die KeyTable atomar neu (REQ-FMT-5);
4. vernichtet `KEK[e]` und dessen Wrap-Kopie nach dem Vernichtungsprotokoll (§12.2.7);
5. bei K1 mit `rotate_passphrase_on_seal`: wechselt die Passphrase, sodass die alte Wurzel nicht mehr ableitbar ist, **wenn die alte Passphrase unbekannt ist**;
6. bei K2/K3: ruft `rotate()` und anschließend `destroy()` am Slot.

Nach dem Seal sind Flash-Reste gewrappter DocDeks früherer Epochen nur mit `KEK[e]` entschlüsselbar, und `KEK[e]` existiert nur noch als Rest unter der Wurzel. Bei K2 und K3 endet die Kette im Modul. Bei K1 endet sie in der Passphrase. Ein Seal **ohne** Passphrasenwechsel erhöht bei K1 den Schutz gegen einen Angreifer mit Flash-Resten **und** Passphrase nicht: Reste des alten KDF-Headers (Salt) und der alten Wurzel-Wraps bleiben mit derselben Passphrase wieder ableitbar. Der Bericht weist deshalb `flash_hardening ∈ {None, PassphraseRotated, ModuleBound, ProviderAttested}` aus (INV-SEAL-HARDENING-1): K1 ohne Wechsel ⇒ `None`, die Klasse C1 stützt sich dann allein auf **A-FLASH** (§12.5.2); K1 mit Wechsel ⇒ `PassphraseRotated`, wirksam gegen Angreifer, die nur die alte Passphrase kennen. Der Passphrasenwechsel ist ein vom Betreiber ausgelöster Vorgang (`seal-now --rotate-passphrase`), nie automatisch. Der Seal bleibt auch ohne Wechsel nötig, weil er die Schlüsselkette bereinigt (Nadelscan, Klasse C1 nach Siegel 3) und Voraussetzung von K2/K3 ist. Der einzige Weg, A-FLASH auf Laptops und Servern zu ersetzen, ist K2 (nicht exportierbares Modul); K2 ist deshalb der priorisierte Ausbau (Spike S-K2).

**KeyTable-Shards (2.4).** Die KeyTable ist in `n_shards` Shards (Startwert 256, nach `hash(unit_ref)`) geteilt; jeder Shard hat eine eigene `ShardKek`-Generation. Begründung: Der gewrappte Blob eines vernichteten DocDek liegt nur in den Dateigenerationen **seines** Shards. Es genügt, diesen Shard unter einer neuen `ShardKek` neu zu schreiben, die alte zu vernichten und die Kette darüber (`KEK` ⇒ ShardKeks) auf `KEK[e+1]` umzustellen. Die Kosten sinken von O(alle Schlüssel) auf O(Schlüssel der geänderten Shards + `n_shards`), und ein Seal kann binnen Minuten statt Tagen laufen. Die Wirkung ist unverändert: Reste im Flash enden bei `KEK[e]`, das nur noch als Rest unter der Wurzel existiert.

**Auslöser.** Zeitgesteuert (`kek_epoch_len`, Startwert 30 Tage), auf Anforderung (`seal-now`) und automatisch nach einer Löschung: Löschungen innerhalb von `seal_coalesce_window` (Startwert 15 min) teilen sich einen Seal; spätestens `seal_after_deletion_max` (Startwert 24 h; Profil `Maximum` 1 h; **in `EnterpriseRegulated` Pflichtwert ≤ 24 h**, H-07) nach der Löschung muss der Seal laufen. Der kürzere Standard folgt aus den Shard-Kosten (siehe unten): Ein Fenster von sieben Tagen verlängerte die Zeit, in der Flash-Reste des DocDek-Wraps unter `KEK[e]` stehen, ohne Kostenvorteil.

**Klassenwirkung (INV-EPOCH-SEAL-1).** C1 bis C3 gelten erst nach dem ersten Seal, der die Löschung umfasst. Zwischen Vernichtung und Seal weist der Bericht die Klasse `C*` (vorläufig) mit `residuals: [R-KEKEPOCH until]` aus.

**Crash-Sicherheit.** Der Seal ist ein eigener durabler Ablauf mit Journal. Ein Absturz mitten im Re-Wrap hinterlässt `KEK[e]` und `KEK[e+1]` nebeneinander und wird beim Start (B7) fortgesetzt. Der Seal ist idempotent.

**Kosten.** Re-Wrap ist O(Schlüssel der geänderten Shards + `n_shards`), nicht O(Datenmenge) und nicht O(alle Schlüssel). Die Messung gehört in `benchmarks/budgets.toml` (O-24, O-40).

##### C-12.2.5 DocDek-Regeln

1. Jede Betroffenen-Einheit (§12.15) erhält **genau einen** DocDek, erzeugt bei `remember`.
2. Jeder Record in WAL, SSTable und Blob-Segment wird mit dem DocDek **seiner Einheit** verschlüsselt. Ein Block mit Records mehrerer Einheiten enthält mehrere Records unter verschiedenen Schlüsseln; die Blockverschlüsselung unter `K_eng` bleibt als äußere Schicht. **Die Vernichtung eines DocDek macht deshalb keinen anderen Record unlesbar** (kein Kollateralverlust, LA-4).
3. Vektor-Payload (fp32 und SQ8) im persistenten Snapshot liegt je Slot unter dem DocDek.
4. Der gewrappte DocDek liegt in der **KeyTable** (Ersetzen-Semantik), nicht in einem Append-Log. Das `KeyRegistry`-Sidecar führt nur Ereignisse ohne Schlüsselmaterial und ohne Wrap-Blob (REQ-DEL-N5).
5. Der Key-Lock der Einheit wird über die **gesamte** Saga gehalten; ein konkurrierendes `remember` auf die Einheit wird abgewiesen (REQ-DEL-012).
6. **Entschlüsselte Schlüssel im Speicher** liegen ausschließlich in einem registrierten `KeyCache`, der bei Vernichtung **vor** dem Schreiben der neuen KeyTable invalidiert und zeroized wird (H-04). Kein Modul hält eine Kopie außerhalb des `KeyCache`.
7. Bestand ohne DocDek (Altformat, Scope-DEK) gilt als `LegacyUnkeyed`: höchstens Klasse P, bis `contextra migrate --rekey-units` die Records je Einheit neu verschlüsselt hat (H-12, REQ-DEL-N37). `doctor` weist den Anteil aus.

##### C-12.2.6 Record-Format und Restmetadaten

- AEAD: AES-256-GCM-SIV, Nonce 96 Bit zufällig, Tag 128 Bit; AD = `scope_id ‖ doc_key_id ‖ record_version`.
- **Padding:** Klartextlänge nach dem Padmé-Schema aufgefüllt (Leckage O(log log M), Overhead höchstens 12 %); Mindestlänge `min_record_bytes` (Startwert 256).
- **Header:** `doc_key_id` und Sequenzbezug stehen **innerhalb** des unter `K_eng` verschlüsselten Blockbereichs, nicht im Klartext des Records (REQ-DEL-N15).
- **Wrap der Schlüssel:** AEAD-Wrap (AES-256-GCM-SIV), zufällige 96-Bit-Nonce, AD = `key_id ‖ kek_epoch`. Die Nonce dient als **Fahndungsnadel** (§12.7.3). AES-KWP (SP 800-38F) ist per ADR wählbar, hat keine Nonce und braucht ein anderes Suchmuster.

##### C-12.2.7 Vernichtungsprotokoll (Destroy-Protokoll, H-03)

Die Schlüsselvernichtung ist ein durabler, idempotenter Ablauf mit festen Schritten. Jeder Schritt wird **vor** der Ausführung im `ErasureJournal` vermerkt (Sidecar vor Wirkung, §7.9 Regel 3).

|Schritt|Aktion|Durabilität / Ausfallverhalten|
|---|---|---|
|D1|Journal-Intent `destroy{key_ref, wrap_nonce_digest}`; Pre-Destruction-Inventar der Chiffratblöcke je `key_id` liegt vor (§12.7.3)|`fsync`; ohne Inventar kein D2|
|D2|Key-Lock halten; `KeyCache`-Eintrag invalidieren und zeroizen|im RAM; danach ist kein Lesen mit diesem Schlüssel mehr möglich|
|D3|Neue KeyTable-Generation **ohne** den Eintrag schreiben: temporäre Datei, `fsync`, `rename`, `fsync` des Verzeichnisses (REQ-FMT-5)|Ausfall hier: Schlüssel lebt, Einheit bleibt `Frozen`, Retry beim Start (kein „vernichtet“-Bericht)|
|D4|Alte KeyTable-Generation und alle Temp-/`.bak`-Kopien entfernen (`durable_remove`), Verzeichnis-`fsync`|Reste der alten Datei auf dem Medium sind Gegenstand des Epoch Seal (§12.2.4)|
|D5|`RevocationLog`-Eintrag `key_destroyed{key_ref_hash}`; `KeyRegistry`-Ereignis `destroyed`|`fsync`|
|D6|Journal-Done; Nadelscan (§12.7.3)|Bericht erst nach D6|

**Regeln.** (1) Ein Schlüssel gilt erst nach **D3** als vernichtet; der Bericht trägt `key_destroyed` erst nach D6. (2) Recovery wiederholt D2 bis D6 idempotent. (3) Die frühere Formulierung „Eintrag mit Nullen überschreiben, fsync, Eintrag entfernen“ ist **kein** Wirkmechanismus auf SSD und Copy-on-Write-Dateisystemen und wird nur als zusätzliche Hygiene beibehalten; die Wirkung trägt der Epoch Seal. (4) Bei `ENOSPC` nutzt D3 die Notreserve (§7.10); bei `fsync`-Fehler an der KeyTable gilt §7.10 (`Poisoned`, Sperr-only). (5) Nach D3 ist ein Wiederauftauchen des Schlüssels ausgeschlossen, weil ihn nur die KeyTable und Schlüssel-Backups der Spur K halten (§12.17); der Restore-Guard vernichtet ihn dort erneut.

##### C-12.2.8 Index-Artefakte (ArtKey)

Postings und Graph enthalten viele Dokumente und können nicht unter einem DocDek liegen.

|Artefakt|Schlüssel|Bereinigung|
|---|---|---|
|Vektor-Payload im Snapshot|DocDek|Vernichtung des DocDek|
|Graph-Snapshot (Topologie, Slot-Nummern)|ArtKey der Snapshot-Generation|Neuaufbau, danach Vernichtung des ArtKey (§12.16)|
|Text-Segment (Postings, Termwörterbuch, Längen)|ArtKey je Segment|Segment neu schreiben mit neuem ArtKey; alte Segmentdatei entfernen; alten ArtKey bei einer Löschung nach Siegel 2 vernichten (Schritt 5), bei einem Merge ohne Löschbezug sofort|

INV-ARTKEY-1: Jedes Index-Artefakt hat genau einen ArtKey; der alte wird bei Merge sofort, bei Löschung nach Siegel 2 vernichtet (REQ-DEL-N16). **Begründung:** Würde der alte ArtKey schon in Schritt 3 vernichtet, wären Pre-Destruction-Inventar und Messung der entschlüsselten Sicht (Schritt 4) für diese Artefakte unmöglich, und INV-DEL-ORDER-1 wäre unterlaufen. Die alte Segmentdatei ist ab Schritt 3 entfernt; bis Schritt 5 existiert nur noch ihr Flash-Rest, der unter dem (noch lebenden, nicht mehr referenzierten) ArtKey steht.

##### C-12.2.9 KeyTable, KeyRegistry, Krypto-Agilität, Keystore

- **KeyTable** (Magic `MFKT`, §7.2) ist ein eigener Speicherbereich mit Ersetzen-Semantik; in 2.4 in `n_shards` Dateien (Shards) geteilt (§12.2.4). Nach jeder Vernichtung und jedem Seal wird die Segmentdatei ohne vernichtete Einträge neu geschrieben (REQ-DEL-N4). Atomares Ersetzen nach REQ-FMT-5, auch für den Keystore.
- **`KeyRegistry`** (`MFKR`) führt Ereignisse (`created`, `destroyed`, `sealed`) mit Referenz-Hash, ohne Schlüsselmaterial und ohne Wrap-Blob; append-only, HMAC-verkettet, über `DurableLog` mit `fsync` je Eintrag. Ein gelöschtes Log samt Marker bei vorhandenen Daten gilt als Manipulation (fail-closed).
- **Krypto-Agilität (REQ-DEL-028, P-13).** Jedes persistierte kryptografische Objekt trägt Algorithmus- und Formatkennung. Ein Wechsel von AEAD, KDF, Hash oder Signatur ist ohne Datenverlust möglich (Re-Wrap bzw. Neuverschlüsselung per Kompaktion). Das Notfallverfahren bei Algorithmenversagen (EDPB Rn. 85, 96, Empf. 13) ist dokumentiert und wird halbjährlich per `doctor crypto-drill` geübt; die Eignung der Primitive wird periodisch neu bewertet.
- `mlock` über `sys`; scheitert die Sperrung, meldet `mem_lock` dies wahrheitsgemäß, und `doctor` weist es aus (§12.18).

#### C-12.3 Zweckregister und Domänentrennung (INV-KEYPURPOSE-1)

Jeder Schlüssel dient genau einem Zweck; HKDF-Label `contextra/v1/<zweck>`. Das Register `governance/key_purposes.toml` ist die einzige Quelle; das Gate vergleicht es mit dem Code (REQ-KEY-011). Die Datei **fehlt im Repo** (U-29, O-28) und ist Voraussetzung von WP L-2.

|Zweck|Label bzw. Herkunft|Ableitung|Wirkung der Vernichtung|
|---|---|---|---|
|Struktur-Schlüssel `K_eng`|`contextra/v1/eng`|HKDF aus der Wurzel|keine Nutzdaten; nie für Nutzdaten|
|WAL-Integrität|`contextra/v1/wal`|HKDF aus `K_eng`|keine|
|Sidecar-Ketten|`contextra/v1/sc/keyreg`, `…/revoke`, `…/audit`, `…/journal`, `…/erased`|HKDF aus `K_eng`|keine; Ketten sind keine Datenschlüssel|
|Keystore-Integritätswurzel|`contextra/v1/keystore-root`|bei K0 im Klartext, sonst gewrappt|keine|
|`KEK[coll, epoche]`|zufällig, 256 Bit, gewrappt|Wurzel (`EffaceableKeySlot`)|Epoch Seal; bei Collection-Löschung alle `KEK[coll,*]`|
|`DocDek[einheit]`|zufällig, 256 Bit, gewrappt|`KEK`|**Löschmaßnahme Ebene B** der Einheit|
|`ArtKey[artefakt]`|zufällig, 256 Bit, gewrappt|`KEK`|Löschmaßnahme für Index-Artefakte|
|`K_ref[tenant, ref_epoche]`|zufällig, 256 Bit, gewrappt; **bewusst nicht per HKDF ableitbar** (sonst nicht vernichtbar)|`KEK`|Pseudonym-Ablauf (§12.10)|
|Berichtssignatur|Signierschlüssel des `KeyManager`|`KmsProvider`|keine; Berichte bleiben prüfbar (INV-PROOF-KEY-1)|
|Surrogat-Vault `[R]`|`contextra/v1/vault`|HKDF aus `K_eng`|Löschschicht `CachesAndVault`|

Regeln: (1) Labels sind paarweise verschieden; Wiederverwendung eines Schlüssels für zwei Zwecke ist unzulässig. (2) Ein Label ohne Registereintrag ist Gate-Fehler. (3) Die Zwecke `DocDek`, `ArtKey`, `KEK-Epoche` und `K_ref` (statt `K_commit`) ersetzen die Streichung aus E27. (4) **Der Berichtssignaturschlüssel wird nie durch einen Seal oder eine Löschung vernichtet**; ein Wechsel erzeugt eine neue `signer_key_id`, die alte bleibt als Verifikationsschlüssel im `RevocationLog` nachvollziehbar (H-16).

#### C-12.4 Die zehn Löschschichten (`DeletionLayer`)

Der Typ `DeletionLayer` hat in 2.3 **zehn** Varianten (2.2.1: acht). Jede Schicht gehört einer Ebene an (**A** Klartext-Strukturen, die für die Suche Klartext brauchen und physisch bereinigt werden; **B** persistente Reste, die unter einem Schlüssel stehen) und hat **genau einen** Besitzer-Crate, der Scanner **und** Purger stellt (Kantenregel 5, §4.4).

|Schicht|Typname|Ebene|Besitzer (Scanner / Purger)|Maßnahme|Messung|Besonderheit|
|---|---|---|---|---|---|---|
|S1 Primärdaten (LSM, WAL)|`Lsm`|A (Memtable), B|`store` (`LsmScanner`)|Records unter DocDek; Memtable zeroizen; Hygiene: `compact_range(scope)`, WAL-Scrub (Segmente ohne Scope neu schreiben, alte entfernen, Verzeichnis-`fsync`)|Klartext, Chiffrat, Nadel|Memtable-Beleg erst nach `force_flush`; Tombstone-GC nur bei Vollkompaktion (§7.4)|
|S1b Blob-Segmente|`BlobSegments`|B|`store` (`LsmScanner`)|Records unter DocDek; Hygiene: Segment-Rewrite|wie `Lsm`|**neu in 2.3**; früher unter `Lsm` versteckt, daher Lücke im Oberflächen-Inventar|
|S2 Vektorindex|`VectorIndex`|A, B|`vector` (`VectorScanner`)|Arena-Slots (fp32, SQ8) nullen; Snapshot-Payload unter DocDek; Graph unter ArtKey, Neuaufbau (§12.16)|Slots mit Payload ≠ 0, Snapshot-Datei|kein Rewiring (VETO-F02); Slot-Nummern bis zum Neuaufbau nicht wiederverwendet (INV-HNSW-SLOT-1)|
|S3 Textindex|`TextIndex`|A, B|`text` (`TextScanner`)|Segmente neu schreiben (neuer ArtKey, alter ArtKey nach Siegel 2 vernichtet, §12.8.2 Schritt 5); Terme nur gelöschter Dokumente aus dem Wörterbuch; Statistik (IDF, Längen, Zähler) neu berechnen|Postings, Wörterbuch, Positionen|Terme müssen verschwinden, sonst Inhaltsrest|
|S4 Caches und Vault|`CachesAndVault`|A|`store`, `engine`, `privacy`|Block-, Embedding-, RRF-, Antwort-Cache und Surrogat-Vault invalidieren und zeroizen|`CacheScanner` (Enumeration nach Scope)|Antwort-Caches sind Pflicht (REQ-DEL-N7)|
|S5 Audit, Lineage, Sidecars|`AuditLineage`|–|`engine`, `crypto` (`SidecarScanner`, `LineageScanner`)|Ableitungen kaskadieren (INV-DERIVED-1); Sidecars enthalten nur Referenz-Hashes, keine Nutzinhalte, keine Schlüssel|Referenzen ohne Klartext|Ableitungen mit weiteren Quellen neu ableiten oder `stale`|
|S6 Interne Checkpoints|`Checkpoints`|B|`checkpoint` (`CheckpointScanner`)|Policy `Rewrite` (Standard), `ExpireNow`, `ExpireAfterHold` nur unter Hold (§12.5.4)|Rohsicht und entschlüsselte Sicht|Hardlink-Klone derselben SSTables liegen im Datenverzeichnis, also in der Kontrolle von Contextra|
|S7 Schlüsselmaterial|`KeyMaterial`|B|`crypto`, `store` (`KeyScanner`)|KeyTable, Keystore, Wrap-Blobs, Temp-/`.bak`-Kopien|Nadelscan (§12.7.3)|**neu in 2.3**; ohne diese Schicht fehlt der CE-Beleg (LA-3)|
|– Graph|`Graph`|–|– (`graph` `[A]`)|–|Start (B3) belegt „keine Daten dieses Typs“ ⇒ `FeatureDisabled`|sonst `NotMeasured(NoPhysicalScanner)`|
|– KV-Cache|`KvCache`|–|– (`kvcache` `[A]`)|–|wie `Graph`|wie `Graph`|
|S8 Externe Sicherungen|– (Ausschluss)|–|–|ausweisen (`ExternalBackups`); Daten-Backups der Spur D sind keine externen Backups, sofern Contextra sie erzeugt (§12.17)|keine|§12.20|

**Regeln.**

1. Jede Schicht stellt genau einen Witness-Typ aus. `FeatureDisabled` ist nur zulässig, wenn der Rand-Feature-Schalter aus ist **und** der Start den fehlenden Datenbestand belegt; sonst `NotMeasured(NoPhysicalScanner)`.
2. Eine Schicht, in der Daten liegen können und für die kein Scanner existiert, ist `NotMeasured`; der Bericht ist dann höchstens `Partial` (REQ-DEL-010, -020).
3. **Ableitungen (INV-DERIVED-1)**: `forget` kaskadiert über die Lineage; der Bericht führt sie als eigene Einträge.
4. **Embeddings und Vektorbytes sind personenbezogen** (vorsorglich, P-05): „Text gelöscht, Vektor im Index“ ist keine Löschung.
5. CDC-Ereignisse sind inhaltsfrei (§7.3).
6. Contextra führt keine eigenen Prompt- oder Gesprächshistorien außerhalb der Primärdaten und der AuditChain; Verläufe des Hosts stehen als `HostHistories` in `excluded` (A-HOSTHIST).
7. **Oberflächen-Inventar (INV-SURFACE-1, Gate `surface-coverage`).** Jede persistente Datei- und Strukturklasse des Format-Registers (§7.11) steht in genau einer Zeile dieser Tabelle oder in §12.20. Ein neues Format ohne Schichtzuordnung bricht den Build.
8. **Migration der Typen.** Der Code kennt sieben abweichende Varianten (`LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments`, `CsrGraph`, `KvCacheSegments`, `EmbeddingCache`; U-41). Die Abbildung auf die zehn Schichten steht in §21 (U-41) und ist Teil von WP L-1; Berichte V1 bis V3 bleiben lesbar und tragen `coverage_known = false`.

#### C-12.5 Löschwirkungsklassen, Annahmen, Checkpoint-Policy, aktive Snapshots

##### C-12.5.1 Klassen

Die Klassen bauen aufeinander auf. **Ein Bericht darf nur die höchste Klasse ausweisen, deren Voraussetzungen gemessen sind** (INV-CLASS-MEASURED-1). Die Stufen S0 bis S3 und SK der Fassung 2.2.1 gehen in den Klassen und der Saga (§12.8) auf.

|Klasse|Name|Aussage|Voraussetzungen (gemessen)|
|---|---|---|---|
|**L**|Gesperrt|Scope für alle Lesepfade gesperrt; rechtlich einer Einschränkung nach Art. 18 entsprechend|`ErasedSet`-Eintrag durabel (`fsync`), Lesepfade geprüft (inkl. `as_of`, Export, Checkpoint-Erzeugung)|
|**P**|Klartext bereinigt|In keiner Klartextstruktur, die Contextra beschreibt, ist ein Inhalt des Scopes auffindbar|Alle Ebene-A-Schichten gemessen, Positivkontrolle bestanden|
|**C***|Vorläufig kryptografisch|Schlüssel vernichtet (D3 vollzogen), Epoch Seal steht aus|Nadelscan, Inventar, Kanarien-Probe bestanden; `residuals: [R-KEKEPOCH until]`|
|**C1**|Reste unlesbar, Software-Schlüssel|Alle persistenten Kopien liegen nur als Chiffrat unter vernichteten Schlüsseln; **Annahme A-FLASH**|wie C*, plus Epoch Seal vollzogen, Custody K1|
|**C2**|wie C1, Wurzel im Modul|A-FLASH entfällt für die Schlüsselwurzel|wie C1, Custody K2 mit bestandener Konformitätssuite|
|**C3**|wie C1, externer KMS|A-FLASH ersetzt durch Anbieterattest (Prüfpflicht des Verantwortlichen, P-18)|wie C1, Custody K3|
|**M**|Messvollständig|P und C erreicht; alle Hygienemaßnahmen vollzogen (physische Kompaktion, Checkpoint-Policy, Graph-Neuaufbau); offen bleiben nur die Pflicht-Ausschlüsse|P, C1/C2/C3, Hygiene gemessen|

**Orthogonale Angaben im Bericht (immer vorhanden):** `legal_state ∈ {Eingeschränkt, TechnischGelöscht, BeschränktAufbewahrt}`; `backup_window ∈ {Closed, OpenUntil(date), ExternalUnmanaged}`; `assumptions[]`; `residuals[]` (zeitlich begrenzte Reste mit Frist, z. B. `R-GRAPH until`, `R-KEKEPOCH until`); `platform_guarantee`.

##### C-12.5.2 Annahmecodes

|Code|Annahme|
|---|---|
|A-FLASH|Flash-Reste gewrappter Schlüssel und die Passphrase kommen nicht zusammen in die Hand derselben Stelle (nur bei K1)|
|A-ALGO|Der Algorithmus (AES-256-GCM-SIV, Argon2id, HKDF) ist nicht gebrochen (EDPB Rn. 51)|
|A-KEYLEAK|Der Schlüssel war vor der Vernichtung nicht kompromittiert (EDPB Rn. 51)|
|A-RAM|Kein Klartext oder Schlüssel liegt in Swap, Dumps, Hibernation (P-03; §12.18)|
|A-EXTBACKUP|Backups außerhalb von Contextra sind nicht erfasst|
|A-HOSTHIST|Verläufe des Host-Systems (Prompt-Historien) sind nicht erfasst|
|A-CLOCK|Die Systemuhr ist nicht manipuliert (Horizont- und Fristentscheidungen, H-09)|

##### C-12.5.3 Wirkung je Plattform und Medium

Contextra behauptet **keine** Löschwirkung auf Medienebene. Auf SSD, Copy-on-Write-Dateisystemen (btrfs, zfs, APFS) und Journalen wirkt Überschreiben nicht zuverlässig; deshalb trägt auf Medienebene die Schlüsselvernichtung (Ebene B) und nicht das Überschreiben. Dateisystem-Snapshots, Time Machine und Journale gelten als externe Kopien und stehen im Bericht (§12.20).

##### C-12.5.4 Checkpoint-Policy

`checkpoint_policy` je Collection: `Rewrite` (**Standard**, ohne den Datensatz neu schreiben, Manifest und Prüfsumme neu ausstellen), `ExpireNow` (betroffene Checkpoints sofort verwerfen), `ExpireAfterHold` (nur unter `Restricted`/Legal Hold, höchstens `checkpoint_hold`, danach verwerfen). Der Standard wechselt gegenüber 2.2.1 von `ExpireAfterHold` auf `Rewrite` (O-15), weil Records nach der Vernichtung des DocDek in jedem Checkpoint unlesbar sind und ein Rewrite die Hygiene vollendet, ohne Wiederherstellungspunkte zu verlieren, die andere Einheiten betreffen. Bis zum Rewrite lautet das Ergebnis `Partial{outstanding: Checkpoints}` bzw. Klasse C ohne M, mit Anzahl, ältestem Alter und Ablaufdatum (`RetainedUnderRestoreGuard`). Ein gepinnter Checkpoint (`pin_checkpoint`) zählt zur Schicht `Checkpoints`. Interne Checkpoints sind **Spur D** (§12.17): Sie enthalten keine KeyTable.

##### C-12.5.5 Aktive Snapshots

`forget` ermittelt den ältesten aktiven Snapshot, der den Scope sieht, wartet bis `deadline` auf dessen Ende (Obergrenze `snapshot_max_age`) oder schließt mit `excluded = ActiveSnapshot{ids}` ab; der Bericht trägt dann einen eingeschränkten Geltungsbereich (`proof_scope`). Offene Snapshots verzögern die physische Hygiene (Status `Pending`), **nicht** die Schlüsselvernichtung: Ein offener Snapshot liest über den `KeyCache`, der bei D2 invalidiert wird; er erhält danach `SnapshotExpired` und liest den Scope nicht mehr (INV-ASOF-ERASE-1).

#### C-12.6 AuditChain und RevocationLog

**AuditChain:** append-only, hash-verkettet, signierte Checkpoints, tenant-getrennt; enthält Zugriffe, Löschungen, Berichte (alle vier Siegel), Freigaben, Validierungsentscheidungen, `Unfreeze`. **RevocationLog:** widerrufene Schlüssel/Module/Berechtigungen, `key_destroyed`-Einträge, Haltezustände (Legal Hold); wird bei **jeder** Autorisierung konsultiert (fail-closed). Beide sind durabel (Flush vor Wirksamkeit, Persistenz über `DurableLog`) und crash-sicher; Tail-Truncation ist über einen signierten Kettenkopf erkennbar. Beide Dekoder sind Fuzz-Ziele. Audit-Inhalte enthalten nur Referenz-Hashes (INV-LOG-1). Schreiben DARF den bestehenden Log nie zerstören (REQ-DEL-022). Löschereignisse stehen mit `subject_ref_hash`, nie mit einer ID. Ein externer Zeitanker ist optional (§12.7.9).

#### C-12.7 Messprinzip, Witness und Löschbericht

##### C-12.7.1 Messprinzip

1. **Messen vor Behaupten.** Ein Beleg entsteht nur aus einem Scanner der besitzenden Schicht (INV-WITNESS-MINT-1) und aus einer Messung am **Speicher**, nicht am Lesepfad: Sichtbarkeit (`scan_prefix`, Suche) belegt keine physische Abwesenheit.
2. Ein Messwert MUSS vom Scanner der Schicht erzeugt werden und trägt Methode, `units_scanned`, `residual_units`, einen Digest über Scan-Parameter und Dateiliste (kein Klartext) und einen Zeitstempel aus dem `Clock`-Port. `units_scanned = 0` ist nicht konstruierbar (Ausnahme: Schlüsselabwesenheit, §12.7.3).
3. Eine **Konstante oder ein vom Aufrufer gelieferter Zählerwert** DARF NIE als Messung dienen (REQ-DEL-008). **Es gibt keinen Fallback-, Standard- oder Testschlüssel für Berichte** (REQ-DEL-009).
4. Fehlt einer Schicht die Fähigkeit zu messen, ist sie `NotMeasured`; fehlende Fähigkeit ist `CapabilityUnsupported`, nie ein Erfolgsstub (REQ-DEL-010).
5. **Zwei unabhängige Messungen** je Pflichtschicht: Scanner der Schicht **und** Black-Box-Probe durch den Orchestrator über einen internen Prüfzugang.
6. **Positivkontrolle.** Jeder Scan-Lauf weist an einem absichtlich nicht gelöschten Kontroll-Datensatz nach, dass er anschlägt (AC-1); ein Lauf ohne bestandene Kontrolle ist **ungültig**.
7. **Verification und Validation sind getrennte Zustände** (NIST Rev. 2, §12.7.7).

##### C-12.7.2 Ebene A: Klartext-Scanner

1. Rohbyte-Scan **aller** Dateien des Datenverzeichnisses (WAL, SSTables, Blob-Segmente, Index-Artefakte, Checkpoints, Temp, `.bak`, Keystore, KeyTable, Journal, Sidecars) nach kanonischen Termen, Roh-IDs, Embedding-Bytes und Chunk-Prüfsummen des Scopes.
2. Dekodierte Sicht über die normalen Lesepfade mit **allen aktiven Schlüsseln**.
3. Positivkontrolle (AC-1) in beiden Sichten.

Der produktive Scanner arbeitet **strukturell und per Rohbyte-Suche über bekannte Muster**; der Canary-Byte-Scan (Spike S-E1: 2.000 → 0 Treffer) ist zusätzlich Test-Werkzeug (§17.9).

##### C-12.7.3 Ebene B: drei Nachweise (Schlüsselabwesenheit)

1. **Pre-Destruction-Inventar.** Vor der Vernichtung wird je `key_id` die Liste aller Chiffratblöcke (Datei, Offset, Länge) als Digest und Zähler festgehalten. Danach gilt: Jeder Block ist einem vernichteten oder lebenden Schlüssel zugeordnet; **Blöcke ohne Zuordnung sind ein Fehler (fail-closed)**. Das Inventar ist nur vor der Vernichtung möglich (deshalb D1).
2. **Nadelscan.** Gesucht wird die **Wrap-Nonce** (96 Bit, zufällig) des vernichteten Schlüssels, die vor der Vernichtung im Journal liegt, in allen Dateien: KeyTable, Keystore, Sidecars, Checkpoints, Temp, `.bak`. Ergebnis: **null Treffer**. Ein Hash des Blobs eignet sich nicht, weil die Rohbyte-Suche alle Offsets hashen müsste. Die Nadel wird nach Abschluss aus dem Journal gelöscht und nur als Digest geführt.
3. **Kanarien-Probe.** Ein Kanarien-Datensatz mit eigenem DocDek wird verschlüsselt und vom Scanner entschlüsselt, dann **über denselben Vernichtungspfad** (D1 bis D6) vernichtet; die Entschlüsselung muss scheitern. Die Probe belegt Scanner und Vernichtungsweg. Sie belegt **nicht** die Vernichtung des konkreten DocDek; dafür stehen Nadelscan und Inventar.

**Grenzen (stehen immer im Bericht):** Der Scanner sieht keine Flash-Remanenz unterhalb des Dateisystems, keinen Schlüssel in RAM, Swap, Dumps (A-RAM) und keine externen Backups.

##### C-12.7.4 Typen

```rust
/// Nur über `ResidualReport::from_scan` konstruierbar. Kein `Default`, kein `From<usize>`.
pub struct ResidualReport {
    layer: DeletionLayer,
    plane: Plane,                 // A | B
    method: ScanMethod,           // RawFileScan | DecryptedViewScan | IndexIntrospection | BlackBoxProbe | NeedleScan | InventoryDiff
    units_scanned: u64,           // > 0, außer NeedleScan (Nadelabwesenheit zählt gescannte Dateien)
    residual_units: u64,
    control_matches: u64,         // positive Kontrolle (AC-1); > 0
    evidence_digest: [u8; 32],    // Scan-Parameter, Dateiliste, Prüfsummen; kein Klartext
    measured_at: UnixNanos,       // aus `Clock`, nicht vom Aufrufer
}

/// Konstruktion nur im sealed `ScanSession`-Pfad (siehe unten). Kein Default, kein Deserialize.
pub struct LayerWitness { /* Felder privat */ }
impl LayerWitness {
    pub fn layer(&self) -> DeletionLayer;     // Enum, kein String (U-41)
    pub fn scanned(&self) -> u64;
    pub fn remaining(&self) -> u64;
    pub fn kind(&self) -> WitnessKind;        // Structural | NotMeasured(NotMeasuredReason) | FeatureDisabled
    pub fn digest(&self) -> [u8; 32];
}
```

**Mint-Härtung (H-02, INV-WITNESS-MINT-1).** Der Zustand im Code ist, dass `LayerWitness::issue_from_scanner` `pub` ist und nur über `#[doc(hidden)]` „geschützt“ wird (U-40); jeder Crate kann damit einen Witness mit `remaining = 0` erzeugen. 2.3 verlangt: (a) Konstruktoren sind `pub(crate)`; (b) ein Witness entsteht ausschließlich in `ScanSession::finish(self, clock) -> LayerWitness`, und `ScanSession` ist nur über `LayerScanner::begin_scan` erhältlich, das eine bestandene Positivkontrolle (`ControlToken`) verlangt; (c) die Trait-Methode `LayerScanner::scan` hat eine **Default-Implementierung**, die `begin_scan`, `scan_units`, `finish` in dieser Reihenfolge ruft; Scanner-Crates implementieren nur `scan_units`; (d) das Gate `witness-mint` lehnt jede Konstruktion außerhalb von `ScanSession` ab, auch in Tests des Produktionscodes (Ausnahme: `testkit`).

##### C-12.7.5 Bericht `DeletionProof` V5

**Format.** V5 ersetzt den in 2.2.1 eingefrorenen V4 (E26, ersetzt durch E46). **Der Repo-Stand kennt höchstens `SignatureVersion::V3`** (U-39); V4 existiert im Code nicht. 2.3 überspringt V4 nicht: V1 bis V3 bleiben lesbar (`coverage_known = false`), V5 ist das einzige schreibbare Format; die Nummer V4 bleibt unvergeben. Magic `MFDR` (§7.2), Golden-File, Migrationspfad, Fuzz-Ziel (REQ-FMT-1 bis -3). Serialisierung explizit spezifiziert (Feldreihenfolge, Endianness, Längen), nicht aus `serde`-Derives; `bincode` wird nicht für V5 verwendet (U-10 entfällt für V5).

**Pflichtfelder.** `class` (L, P, C*, C1, C2, C3, M), `seal_no` (1 bis 4), `legal_state`, `backup_window`, `assumptions[]`, `residuals[]`, je Schicht (`layer`, `plane`, `method`, `units_scanned`, `residual_units`, `control_matches`, `evidence_digest`, Ergebnis), `custody` und `DestroyAttestation`, Epoch-Seal-Referenz, `subject_ref_hash` (und `unit_ref_hash[]`), `scope`, `uncovered_layers[]` (`UncoveredLayer{layer, reason}`), `excluded[]` (**Pflicht, nie leer**), `perspectives[]` (je Q1 bis Q8 ein erzeugtes Urteil, §12.1.4), `verification`, `validation` (§12.7.7), `requested_at`/`completed_at` (aus `Clock`), `clock_source`, `platform_guarantee`, Verweis auf die AuditChain-Position, `signer_key_id`, Ed25519-Signatur (Schlüssel aus dem `KmsProvider`, INV-PROOF-KEY-1), Prüfsumme über alle Felder.

**NIST-Zertifikatsfelder.** Produkt und Version; Medium (Dateisystemtyp, Laufwerksklasse falls bekannt); Methode (Purge); Technik (Cryptographic Erase, Blockrewrite); Validierungsentscheidung; Concurrence; **CE-Traceability** (Algorithmen, Schlüsselstärken, Schlüsseltypen, Escrow-/Backup-Historie aus Spur K).

```rust
impl DeletionProof {
    /// Nur `ErasureOrchestrator` ruft `issue`; Aufrufer außerhalb von `engine` können keinen Bericht bilden.
    pub fn issue(ctx: &RequestContext, saga: &SagaRecord, witnesses: &[LayerWitness],
                 class: ReachedClass, assumptions: &[Assumption], residuals: &[Residual],
                 excluded: &[ExcludedScope], custody: &DestroyAttestation,
                 key: &SigningKeyHandle, clock: &dyn Clock) -> Result<DeletionProof, ProofError>;
    pub fn verify(&self, vk: &VerifyingKey) -> Result<VerifiedProof, ProofError>;
}
```

**Ist-Stand.** `DeletionProof::issue` ignoriert den Parameter `_custody` (U-42), kennt keine Klasse, keine Annahmen und keine Siegel. Das ist mit V5 zu beheben (WP L-9).

|ID|Regel|
|---|---|
|REQ-PRF-1|Der Stub `no_crypto_stubs` (`engine/src/lib.rs:17`) und die leeren Mandanten-Hooks werden **gelöscht**; `crypto` ist nicht-optionale Abhängigkeit der Engine. Der Stub akzeptiert heute **jede** Restzahl (`new_after_verified_empty` liefert immer `Ok`), das ist ein Verstoß gegen A1 (U-21).|
|REQ-PRF-2|`issue` liefert `Err`, wenn eine Pflichtschicht `remaining > 0` hat oder weder `Structural` noch `FeatureDisabled` ist, wenn die Klasse höher ist als die Messung trägt, wenn `excluded` leer ist oder wenn Annahmen fehlen, die die Klasse erfordert. `verify` mit falschem Schlüssel liefert `Err`.|
|REQ-PRF-3|Der Bericht nennt je Schicht Methode, Zahlen, Digest sowie `excluded`; die Außenformulierung wird aus dem Bericht erzeugt (INV-CLAIMS-1). Die `custody` wird für die Klasse **genutzt** (C1 bis C3).|
|REQ-PRF-4|`rg "default_test_deletion_proof_key"` im Produktionscode ergibt **0 Treffer** (heute `contextra-mcp/src/tools_crud.rs:206`, U-22). Der Schlüssel kommt aus dem `KmsProvider`. `contextra_delete` und `contextra_drop_collection` erzeugen keinen Bericht außerhalb der Saga (INV-NO-DUMMY-PROOF-1).|
|REQ-PRF-5|V5 mit Golden-File, Migrationspfad und Fuzz-Ziel (ersetzt „entfällt, E26“).|
|REQ-PRF-6|`doctor verify-report` prüft offline Signatur, Prüfsumme, Klasse, Siegelkette und Zeitanker (ersetzt „entfällt, E26“; REQ-DEL-N9). Ein unabhängiger Referenz-Prüfer in anderer Sprache dient **nur als Testorakel** (INV-ORACLE-1) und gehört nicht zum Produkt.|
|REQ-PRF-7|Jeder Bericht (alle vier Siegel) wird als Eintrag in die AuditChain geschrieben (Magic, Version, Prüfsumme, `fsync`) und ist für den Betreiber lesbar (`report(id)`).|

**Regression.** `review_stub_proof` bleibt dauerhaft im Repo: Bericht mit Restbestand → `Err`; falscher Schlüssel → `Err`; Bericht ohne Scanner-Witness → nicht kompilierbar.

##### C-12.7.6 Restmetadaten-Selbsttest

Nach der Vernichtung prüft `doctor` stichprobenartig, ob verbleibende Chiffrat-Records einer gelöschten Einheit (a) sich über Klartext-Header zuordnen lassen (**No Linkage**), (b) eine einzigartige Länge haben (**No Record Isolation**, Verteilung nach Padding), (c) aus Zählern, Statistik oder Zeitstempeln Inhalte ableiten lassen (**No Inference**). Zusätzlich laufen Entropietests auf Restdaten, die Chiffrat sein müssen; ein unerwartet niedriger Wert zeigt einen Chiffrierfehler an. Der Test liefert eine **Warnung, kein Rechtsurteil**.

##### C-12.7.7 Verification und Validation

|Zustand|Bedeutung|
|---|---|
|`Verified`|Alle Schichten gemessen, Positivkontrollen bestanden|
|`Validated`|Eine zuständige Person nimmt das Ergebnis ab (Validierungsentscheidung). **Im Regulated-Tier Pflicht, mit zweiter Signatur (Concurrence) einer anderen Person**|

Die Klasse im Bericht gilt für beide Zustände; `Validated` ist ein zusätzliches Feld. Ein Bericht ohne `Validated` im Regulated-Tier trägt den Vermerk `ValidationPending` und gilt nicht als Abschluss (REQ-DEL-N20).

##### C-12.7.8 Löschbescheinigung (abstrakt, datensparsam)

Aus dem Bericht wird eine einseitige Bescheinigung erzeugt (OLG Dresden 4 U 940/24, AG Lörrach 3 C 1099/24). Sie **enthält**: Datum, Art des Scopes (z. B. „Akte“), erreichte Klasse je Schichtgruppe, Annahmen, Ausschlüsse (zusammengefasst, nie gekürzt weitergegeben), Bericht-Digest, Signatur, Zeitanker-Status. Sie **enthält nicht**: IDs, Dateinamen, Inhalte, Embedding-Daten. Sie dient der Beantwortung betroffener Personen, der Auftragsverarbeitung und Aufsichtsanfragen; ob und an wen sie auszuliefern ist, entscheidet der Verantwortliche. Ein Anspruch der betroffenen Person auf diesen Nachweis ist nicht gesichert (AG Lörrach).

##### C-12.7.9 Zeitanker

Der Digest des Berichts wird optional über einen qualifizierten Zeitstempel (RFC 3161, eIDAS Art. 41, 42) verankert. Der Zeitstempel belegt, dass der Bericht zu diesem Zeitpunkt existierte (Datum und Datenintegrität), **nicht**, dass die Löschung stattgefunden hat. Offline-Betrieb: Anforderung, sobald eine Verbindung besteht (`anchor: pending`); Verifikation offline möglich, wenn Zertifikatskette und Vertrauensliste als Schnappschuss im Bericht liegen. Ein ausgestelltes Token ist nicht widerrufbar; der Zeitstempel überträgt nur den Digest. Anbieterwahl offen (O-34). Der Zeitanker ist ein Port `[R]` (`TimestampAuthority`), nicht Kern.

##### C-12.7.10 Aufbewahrung des Berichts

Der Bericht enthält nur Referenz-Hashes und gilt selbst als personenbezogen (EDPB Rn. 52). Aufbewahrung nach `report_retention` (Rechtsfrage P-07), danach pseudonym-ablaufend (§12.10). **Beweiskraft:** Der Bericht ist eine signierte Behauptung des löschenden Systems; die Signatur belegt nicht, dass Bytes fehlen. Unabhängige Aussagekraft entsteht aus gemessener Bereinigung, Nadelscan, Inventar und reproduzierbaren Canary-Läufen (§17.9).

##### C-12.7.11 Heap-Canary und Sensitive-Arenen (INV-HEAP-ZERO-1, 2.4)

**Problem.** Ebene A verspricht, dass Klartext-Strukturen bereinigt sind. Das Rohdatei-Scanning sieht nur Dateien, nicht den Prozessspeicher; `zeroize` auf einzelnen Typen belegt nicht, dass Kopien in Allokator-Resten, Puffern oder Caches verschwunden sind. Ohne Messung bliebe die wichtigste Klartextquelle (RAM) ein Behauptungsfeld.

**Regeln.**
1. Klartext von Nutzdaten (entschlüsselte Blöcke im Block-Cache, Query- und Antwortpuffer, Memtable-Werte, Arena-Slots, Embedding-Caches) liegt ausschließlich in `Sensitive`-Arenen der Insel `sys` (eigene `mmap`-Regionen, `MADV_DONTDUMP`, `mlock` soweit möglich, Zeroisierung bei Freigabe und bei Invalidierung). Ein Typ `Sensitive<T>` ist die einzige Hülle für solche Werte; Gate `plaintext-types` lehnt Klartextpuffer außerhalb der Hülle ab (Schnittstellen bleiben sicher, `unsafe` nur in `sys`, P2).
2. Der **Heap-Canary-Test** (Feature `test-hooks`, nie im Release) legt Canary-Daten an (≥ 32 Zufallsbyte, Embedding-Bytes, seltene Terme), führt Klasse P herbei und durchsucht den **Prozessspeicher** (Linux: `/proc/self/mem` mit `/proc/self/maps`; macOS: `vm_read`) nach den Needles. Erwartung: 0 Treffer außerhalb einer begründeten Allowlist (Testharness-eigene Kopien). Positivkontrolle: vor der Löschung muss der Scan die Needles finden.
3. Der Test läuft in jedem CI-Lauf (Tier 1 und 2) und im Profil `Maximum` als `doctor`-Selbstprobe.
4. **Grenze:** Der Test ist ein Regressionsdetektor für Contextras eigene Puffer, kein Beweis gegen Swap, Hibernation, Kernel- oder CPU-Caches und Fremdprozesse; diese bleiben A-RAM (§12.18, §12.20).

#### C-12.8 Löschsaga

##### C-12.8.1 Zustände

```
Requested → Authorized → Frozen → LogicallyDeleted → PlaintextPurged → PlaintextMeasured
          → KeyDestroyed → CryptoMeasured → (Sealed) → Hygiene → HygieneMeasured → Reported → Completed
              │             └─(Aufbewahrungspflicht/Legal Hold, ab Frozen)→ Restricted{basis, until}
              └→ Rejected                                   (jeder Zustand) → Partial{outstanding, reason} → Retry/Escalate
```

Weitere Zustände: `ReviewRequired` (Mixed-Segmente, §12.15), `Unfrozen` (Rücknahme vor `KeyDestroyed`, §12.8.3).

##### C-12.8.2 Reihenfolge (Fast-Path vor Hygiene, E42)

**Fast-Path** (Ziel: Minuten bis Stunden):

|Schritt|Zustand / Aktion|Durabilität|Klasse danach|
|---|---|---|---|
|1|`Frozen`: `ErasedSet`-Eintrag (mit `subject_ref_hash` und `unit_ref_hash[]`), Lesesperre, `legal_state = Eingeschränkt`; Spiegelung an den `ErasedSetMirror` (§12.17.3)|`fsync` **vor** jeder Änderung|**L** (Siegel 1)|
|2|`LogicallyDeleted`: Tombstone im WAL-Batch, Lineage-Kaskade|WAL-Commit|L|
|3|`PlaintextPurged`: Ebene A bereinigen (Arena-Slots nullen, Caches zeroizen, Postings im Speicher, Antwort-Cache); Text-Segmente neu schreiben (neuer ArtKey; alte Segmentdatei entfernt; alter ArtKey bleibt bis Schritt 5)|Journal-Eintrag vor jedem Schritt|–|
|4|`PlaintextMeasured`: Scanner Ebene A; **Pre-Destruction-Inventar** je `key_id`|Bericht mit Positivkontrolle|**P** (Siegel 2)|
|5|`KeyDestroyed`: DocDek **und alte ArtKeys** der Einheit vernichten nach Destroy-Protokoll D1 bis D5 (§12.2.7), nach optionaler `erasure_grace`|Journal-Intent vor Vernichtung|–|
|6|`CryptoMeasured`: Nadelscan, Kanarien-Probe, Chiffrat-Identität (D6)|Bericht|**C***|
|7|Epoch Seal (§12.2.4), koalesziert (`seal_coalesce_window`), spätestens nach `seal_after_deletion_max`, oder `seal-now`|Journal|**C1/C2/C3** (Siegel 3)|

**Hygiene** (terminiert, ≤ `purge_rebuild_max_days`, Startwert 14 Tage):

|Schritt|Aktion|Begründung|
|---|---|---|
|8|Physische Kompaktion der betroffenen SSTables (`compact_range(scope)`), WAL-Scrub, Blob-Rewrite, Checkpoint-Policy|T1; Platz|
|9|Graph-Neuaufbau (Generationswechsel), danach ArtKey der alten Generation vernichten (§12.16)|schließt `R-GRAPH`|
|10|`HygieneMeasured`, `Reported`, Siegel 4|**M**|

**Begründung der Reihenfolge.** Records sind je Einheit verschlüsselt (§12.2.5); eine gemeinsame Blockstruktur macht die Vernichtung deshalb nicht kollateral. Die Fassung v3.0 (Bereinigung vor Vernichtung) verzögerte die wirksamste Maßnahme um bis zu 30 Tage. Ein Messargument für „Bereinigung zuerst“ bleibt (Inventar und entschlüsselte Sicht sind nur vorher möglich); es ist in Schritt 4 abgedeckt. **Verworfen: „Key-first“** (Vernichtung unmittelbar nach `Frozen`): sie verhindert die Messung der entschlüsselten Sicht und schließt die Rücknahme bei Irrtum aus; die Karenz (§12.8.3) ersetzt sie.

**Ordnungsinvariante (INV-DEL-ORDER-1, H-05).** Schritt 5 DARF NICHT beginnen, bevor Siegel 2 (Klasse P) für **jede Ebene-A-Schicht** der Einheit ausgestellt und das Pre-Destruction-Inventar durabel ist. Ausnahme: Notpfad (§7.10), in dem nur L erreichbar ist.

##### C-12.8.3 Karenz und Rücknahme

Bis Schritt 5 ist die Löschung rücknehmbar (`Unfreeze`, Kategorie `Admin`, Begründung im Audit; **im Regulated-Tier zwei Personen**, H-10). `erasure_grace` (Startwert 0, Obergrenze 72 h, im Regulated-Tier Pflichtwert ohne Standard, REQ-DEL-N26) legt fest, wie lange nach Schritt 4 gewartet wird; währenddessen gilt `legal_state = Eingeschränkt`. Der Retention-Reaper (§11.6) nutzt die Karenz von 7 Tagen im Zustand `PendingDeletion`. Nach D3 gibt es **keine Rücknahme** (kein `Unfreeze`, nur Neuanlage aus Quellmaterial des Betreibers). Das Modell folgt dem Wartefenster üblicher KMS-Dienste (AWS 7 bis 30 Tage), ist aber auf Stunden verkürzt, um die Pflicht zur unverzüglichen Löschung nicht zu verzögern.

##### C-12.8.4 Crash-Verhalten

- Absturz vor Schritt 5: Recovery setzt bei `PlaintextPurged` fort; der Scope bleibt gesperrt.
- Absturz nach D3: Die Vernichtung ist vollzogen; Recovery wiederholt D4 bis D6 und misst nur noch. **Ein Bericht trägt nie Klasse C vor abgeschlossener Messung.**
- Intent ohne Ausführungsnachweis wird beim Start wiederholt; Vernichtung ist idempotent.
- Der Epoch Seal ist ein eigener durabler Ablauf (§12.2.4).
- **Keine Wiederauferstehung (INV-NO-RESURRECTION-1, H-06).** Beim Start (B6/B7) und nach jedem Restore prüft die Engine für **jeden** `ErasedSet`-Eintrag im Zustand ≥ `Frozen`, dass kein lebender DocDek und kein lebender ArtKey der betroffenen Einheiten in der KeyTable steht; Abweichung ⇒ Einheit bleibt gesperrt, Vernichtung wird wiederholt (`resurrection_repaired` im Journal), Alarm.
- **Uhr (H-09, A-CLOCK).** Entscheidungen, die von Zeit abhängen (Horizont, Fristen, Karenz, `valid_until`), schlagen fail-closed fehl, wenn die Uhr hinter den zuletzt in AuditChain oder Journal geschriebenen Zeitpunkt zurückspringt; `doctor` weist `ClockRegression` aus. Nur die Verkürzung einer Frist (strengere Wirkung) darf bei unsicherer Uhr weiterlaufen.

##### C-12.8.5 Siegel (gestufte Teilberichte)

|Siegel|Zeitpunkt|Inhalt|
|---|---|---|
|1|nach Schritt 1|Klasse L, `legal_state`|
|2|nach Schritt 4|Klasse P|
|3|nach Schritt 7|Klasse C1/C2/C3 mit Annahmen (bzw. C* mit `R-KEKEPOCH until`)|
|4|nach Schritt 10|Klasse M|

Jedes Siegel ist ein signierter Bericht, in die AuditChain eingetragen und mit dem vorherigen verkettet (REQ-DEL-N27). Der Betreiber kann damit nachweisen, **wann welcher Zustand erreicht war**.

##### C-12.8.6 `Restricted` und Legal Hold

Bei Aufbewahrungspflicht (Art. 17 Abs. 3) oder Legal Hold geht die Saga nach Schritt 1 in `Restricted{basis, until}`: **keine Löschoperation, keine Schlüsselvernichtung**, Lesesperre bleibt (AC-11). Nach Ablauf setzt die Saga bei Schritt 2 fort. `ExpireAfterHold` gilt nur für Checkpoints. Ob und wie lange: Rechtsfrage (O-12). Das Archiv ist vom aktiven Retrieval-Index zu trennen (P-09). Ein Hold wird im `RevocationLog` und Audit geführt und betrifft die Saga, nicht die Lesesperre.

##### C-12.8.7 Zustandsmodell: Regeln

|ID|Regel|
|---|---|
|REQ-FGT-1|Jeder Übergang wird **vor** der Ausführung ins durable Journal geschrieben (`fsync` mit Fehlerpropagation); jeder Schritt ist idempotent.|
|REQ-FGT-2|Beim Start (B7) setzt die Engine jede Saga ohne `Completed` fort oder meldet `Partial{outstanding}`.|
|REQ-FGT-3|`Frozen` schreibt den `ErasedSet`-Eintrag durabel **vor** der ersten Löschoperation; ab dann gilt die Lesesperre.|
|REQ-FGT-4|Fehler von `fsync`, `rename`, `write` in Purgern führen zu `Partial`, nie zu stillem Weiterlaufen.|
|REQ-FGT-5|Ein Scanner-Ergebnis mit `remaining > 0` hält die Saga in `PlaintextMeasured`/`HygieneMeasured` mit `ResidueRemaining{layer, count}`; es entsteht ein Bericht, kein höherer Klassenanspruch.|
|REQ-FGT-6|Die Saga ist abbrechbar nur **zwischen** Schritten; Abbruch hinterlässt `Partial`.|
|REQ-FGT-7|Der `RequestContext` (Mandant, Prinzipal) kommt vom Aufrufer; kein Fallback auf `TenantId::SYSTEM` und kein `unwrap_or(TxId::new(1))` (heute `tools_crud.rs:127,193`, `server_tools.rs:449`, U-23).|
|REQ-DEL-036|**Fristverfolgung:** Die Saga trägt `requested_at` und Schwellen (§12.19.1); jeder `Partial`-Zustand erzeugt je Schwelle genau einen Alarm, eine Eskalation bzw. eine Meldung (ohne Nutzinhalte). Kein `Partial` ohne ausstehende Schicht, Grund und Ablaufdatum. Contextra sendet nichts nach außen.|

##### C-12.8.8 `forget` aus Aufrufersicht

Autorisierung (Kategorie `Destructive`) und Forget-Guard (§13.3) → Schritt 1 (`Frozen`, durabel) → **sofortige Rückgabe** eines `DeletionReceipt` mit ehrlichem Teilbericht (Siegel 1, Klasse L) und `PendingDeletion`-Handle → Schritte 2 bis 7 (Minuten bis Stunden) → Hygiene → Bericht M. `report(id)` liefert den jeweils höchsten Stand. Idempotent und wiederaufnehmbar nach Absturz.

##### C-12.8.9 Planung, Effizienz und Profile (2.4)

Die Sicherheitsinvarianten sind unabhängig von Effizienz und Profil; Planung ändert nur **Zeitpunkt, Bündelung und Messtiefe**, nie die Reihenfolge nach INV-DEL-ORDER-1 und nie die Klassenvoraussetzungen (INV-CLASS-MEASURED-1).

**a) Planer.** Ein deterministischer Planer (Earliest-Deadline-First über die Fristen aus §12.19.1, Zeit aus `Clock`) ordnet die Schritte aller offenen Sagas. Schritte 1 und 2 (Sperre, Tombstone) und Messung Ebene A haben absoluten Vorrang vor Hygiene; Hygiene ist durch `SystemPressureMonitor` drosselbar, steigt aber mit näherndem `purge_rebuild_max_days` in der Priorität (kein Verhungern). Ein `dry_run` liefert Kostenvorschau (betroffene Dateien, Bytes, geschätzte Dauer) ohne Wirkung.

**b) UnitDirectory (INV-UNITDIR-1).** Jede persistente Datei (SSTable, Blob-Segment, WAL-Segment, Text-Segment, Index-Snapshot) trägt einen unter `K_eng` verschlüsselten Metadatenblock mit den `doc_key_id` und Blockbereichen der enthaltenen Einheiten. Begründung: Die Blöcke sind per DocDek verschlüsselt und deshalb ohne Schlüssel nicht zuordenbar; ohne Verzeichnis müsste jede Löschung alle Dateien entschlüsseln oder blind scannen. Mit Verzeichnis ist (1) das Pre-Destruction-Inventar O(Dateien + Blöcke der Einheit), (2) die Purge-Kompaktion `compact_range` auf die tatsächlich betroffenen Dateien begrenzt, (3) die Vollständigkeitsprüfung („jeder Block einem lebenden oder vernichteten Schlüssel zugeordnet“) maschinell möglich. Einträge vernichteter Einheiten verschwinden beim Hygiene-Rewrite (Schritt 8); bis dahin sind sie Zufalls-IDs ohne Personenbezug (§12.7.6).

**c) Zweistufige Messung (INV-UNITDIR-1, AC-35).** Der Rohbyte-Vollscan aller Dateien je Löschung wäre O(Datenmenge) und verhinderte häufige Löschungen. Er ist auch nicht nötig, weil ein Klartext-Needle in **verschlüsselten** Dateien nur bei einem Verschlüsselungsfehler trifft. Die Messung teilt sich deshalb:

|Stufe|Gegenstand|Tiefe je Löschung|
|---|---|---|
|M1 Inventar|alle Dateien: Zuordnung jedes Blocks zu einem Schlüssel über das UnitDirectory; unzugeordnete Blöcke ⇒ Fehler|vollständig (Metadaten)|
|M2 Klartext-fähige Bereiche|Memtable, Arenen, Caches, Temp, `.bak`, Journal, Sidecars, Logs, KeyTable-Dateien, Checkpoints im Rohzustand|vollständig, Multi-Needle (alle ausstehenden Needles einer Koaleszenz in einem Durchlauf)|
|M3 Verschlüsselte Dateien|Nadelscan der Wrap-Nonces, Entropiestichprobe, Stichprobe von Blöcken auf Klartext-Needle|Stichprobe; volle Tiefe alle `full_scan_interval` (Startwert 7 Tage) sowie in jedem CI- und Canary-Lauf|

Differenzbedingung: Der zweistufige Befund und der volle Rohscan liefern auf denselben Daten dasselbe Urteil; eine Abweichung ist ein Defekt des Scanners (AC-35). Klasse P stützt sich auf M1 und M2, Klasse C auf M1 bis M3.

**d) Koaleszenz.** Löschungen innerhalb von `measure_coalesce_window` (Startwert 60 s) teilen sich einen Messlauf; innerhalb von `seal_coalesce_window` einen Seal; mehrere Löschungen derselben Collection einen Graph-Neuaufbau (§12.16). Jede Löschung erhält trotzdem einen eigenen, signierten Bericht mit ihren eigenen Witnesses; Siegel 1 (Klasse L) wird **nie** verzögert (AC-37).

**e) Profile.** Das Profil `ErasureProfile` ändert nur Zeitfenster und Messtiefe:

|Parameter|`Standard` (Default)|`Maximum`|
|---|---|---|
|`erasure_grace`|0|0|
|`seal_after_deletion_max`|24 h|1 h|
|Passphrasenwechsel beim Seal (K1)|auf Anforderung|bei jedem Seal Pflichtvorgang, sonst `flash_hardening = None`|
|M3-Tiefe je Löschung|Stichprobe|vollständig|
|`ErasedSetMirror`|empfohlen|Pflicht|
|`checkpoint_policy`|`Rewrite`|`Rewrite`|
|Heap-Canary im Betrieb (`doctor`-Selbstprobe)|aus|bei jeder Löschung|

`EnterpriseRegulated` setzt mindestens `Maximum` für `erasure_grace`-Pflichtwert, Seal und Mirror (§14.2). Beide Profile erfüllen alle Invarianten; `Maximum` erkauft kürzere Restfenster mit höherer Last.

#### C-12.9 Lesesperre, Key-Lock und Restore-Guard

- **Lesesperre.** Mit `Frozen` trägt der persistente `ErasedSet` nur Referenz-Hashes. `get`, `search`, `hydrate`, `export` und Checkpoint-Erzeugung prüfen ihn **vor** Rückgabe (REQ-DEL-011); sie gilt auch für `as_of` (INV-ASOF-ERASE-1). Die Prüfung verwendet beide Hashes (`subject_ref_hash`, `unit_ref_hash`).
- **Key-Lock.** Das Löschen hält den Key-Lock der Einheit über die **gesamte** Saga (REQ-DEL-012); konkurrierendes Schreiben darf nichts wieder auftauchen lassen (loom).
- **`ErasedSet`-Eintrag (H-08).** `{entry_seq, scope_ref, subject_ref_hash?, unit_ref_hash[], ref_epoch, requested_at, state ∈ {Frozen, Restricted, KeyDestroyed, Hygiene, Done}, prev_mac}`. `unit_ref_hash = HMAC(K_ref, doc_key_id)` trägt Löschungen **ohne** Betroffenenbezug (einzelnes Dokument, Collection). Ohne `unit_ref_hash` könnte der Restore-Guard Einheiten ohne `subject_ref` nicht wiederfinden.
- **Restore-Guard.** Vor dem ersten Lesezugriff nach einem Restore: (1) `ErasedSet` des **Zielsystems** lesen (nicht aus dem Backup); (2) für jeden Eintrag Schritt 3 ff. erneut anwenden, wiederhergestellte DocDeks vernichten (Destroy-Protokoll); (3) Messung wie §12.7; (4) Journal-Ereignis `restore_reapplied`; (5) Dokumentation (Grund, Zeitpunkt) im Audit (BayLfD 12.5). Der Restore-Guard greift **vor** B9 (`INV-BOOT-1`).
- **Restore-Horizont (INV-RESTORE-HORIZON-1, neu gefasst).** Der Horizont gilt für **Schlüssel-Backups** (Spur K): Ein Schlüssel-Backup mit `valid_until` in der Vergangenheit oder unbestimmbarem Alter wird verweigert (`RestoreBeyondHorizon`, final). Daten-Backups (Spur D) unterliegen keinem Horizont, weil sie ohne Schlüssel inert sind. Die Konfiguration erzwingt `key_backup_max_age ≤ erased_set_retention` (Startfehler in B0). Ein `ErasedSet`-Eintrag bleibt mindestens `erased_set_retention` nach der Löschung zuordenbar (§12.10).
- **Unbekanntes Alter.** Ist das Erzeugungsdatum eines Schlüssel-Backups nicht aus dem signierten Manifest bestimmbar oder greift `ClockRegression`, wird verweigert.

|ID|Regel|
|---|---|
|REQ-RST-1|`ErasedSet` ist durabel, enthält nur Referenz-Hashes (keine Klartext-IDs) und wird in B6 vor Verkehr geladen (INV-BOOT-1).|
|REQ-RST-2|`restore_checkpoint` sperrt vor dem ersten Lesezugriff, wendet jeden Eintrag erneut an und schreibt `restore_reapplied` ins Journal (REQ-DEL-013).|
|REQ-RST-3|Schlüssel-Backups, deren signiertes `valid_until` überschritten oder deren Alter nicht bestimmbar ist, werden fail-closed verweigert; Start prüft `key_backup_max_age ≤ erased_set_retention`.|
|REQ-RST-4|Checkpoint-Erzeugung und Export prüfen den `ErasedSet` vor dem Schreiben.|
|REQ-RST-5|Backups außerhalb von Contextra stehen als `ExternalBackups`/`ExternalUnmanaged` im Bericht; die Rücksicherungsregel trägt der Verantwortliche.|

#### C-12.10 Referenzschlüssel, Datensparsamer Bericht, Pseudonym-Ablauf

**Zweck.** Berichte, `ErasedSet`, `KeyRegistry` und Journal müssen auf Betroffene verweisen, ohne sie zu identifizieren. **Terminologie (H-13).** Der Begriff „Commitment“ der Fassung 2.2.1 entfällt: `HMAC-SHA256` ist ein **Hash mit Schlüssel** im Sinne EDPB Rn. 52 und kein kryptografisches Commitment. Er heißt **Referenz-Hash**; der Schlüssel **Referenzschlüssel** `K_ref`. Ein echtes Commitment-Schema ist optional (O-36).

**Regeln.**

1. `subject_ref_hash = HMAC-SHA256(K_ref[tenant, ref_epoche], kanonische Betroffenen-Referenz)`. Der Hash bleibt **personenbezogen** (Rn. 52), solange `K_ref` existiert.
2. Die kanonische Referenz hat **Mindestentropie** (zufällige Betroffenen-ID, nicht Name oder E-Mail) und steht nur im Dokumentdatensatz. `unit_ref_hash` bildet über die zufällige 128-Bit-`doc_key_id`.
3. `K_ref` ist zufällig, nicht per HKDF ableitbar, unter `KEK` gewrappt, je (Tenant, Epoche); es wird vom Epoch Seal mitgenommen.
4. Nach `ref_epoch_len` (90 Tage) **plus** `erased_set_retention` wird `K_ref[tenant, epoche]` vernichtet (Destroy-Protokoll) und im `RevocationLog` eingetragen. Danach sind die Hashes dieser Epoche keiner Person mehr zuordenbar, soweit `K_ref` nicht in Schlüssel-Backups steht (Spur K ist befristet: `key_backup_max_age ≤ erased_set_retention`, daher unkritisch). Der Ablauf entfernt keine Kettenglieder, sondern die Zuordenbarkeit; die Ketten bleiben über ihre Köpfe prüfbar.
5. Sidecars enthalten keine Nutzinhalte und keine Schlüssel.
6. `erased_set_retention ≥ key_backup_max_age + Sicherheitszuschlag` (Startwert insgesamt 90 Tage; Obergrenze nach Rechtsprüfung P-06, P-07, P-16; die 365 Tage der Fassung 2.2.1 sind nicht mehr nötig, weil nur Schlüssel-Backups eine Rücksicherung gelöschter Inhalte ermöglichen). In `EnterpriseRegulated` Pflichtwert ohne Standard; nie unbegrenzt.

Kipptest: Will der Eigentümer auch die Referenz-Epochen streichen, werden `ErasedSet`-Einträge nach `erased_set_retention` per atomarem Rewrite entfernt (O-17); der Restore-Guard verliert dann die Wiedererkennung und `restore_unguarded` wird ausgewiesen.

#### C-12.11 Claim-Taxonomie und Außenformulierung

|Claim|Voraussetzung (gemessen)|
|---|---|
|`LogicalSuppression`|Klasse L: Lesesperre aktiv, Black-Box-Probe findet nichts|
|`PhysicalPurge{layer}`|Klasse P bzw. Hygiene: Segment neu geschrieben, Altdatei entfernt, Rohdateiscan ohne Canary, `units_scanned > 0`|
|`KeyDestroyed{custody}`|D1 bis D6 vollzogen, Nadelscan 0, Inventar vollständig|
|`CryptoErasure{C1\|C2\|C3}`|wie `KeyDestroyed`, Epoch Seal vollzogen, Annahmen benannt|
|`Restricted`|Aufbewahrungspflicht/Legal Hold; Lesesperre aktiv; keine Löschoperation|

**Verboten** (Gate `deletion-honesty`): „100 % gelöscht“, „unwiederbringlich“, „DSGVO-konform“, „Art.-17-konform“, „rechtskonform gelöscht“, „garantiert gelöscht“, „kryptografisch gelöscht“ ohne Custody und Annahmen, „Löschzertifikat“ ohne Scope und Klasse, „Löschung nach Art. 17“, jede Rechtsbehauptung.

**Satzbibliothek (INV-CLAIM-BIND-1).** Außentexte entstehen **nur** aus einer festen Satzbibliothek; es gibt keinen Freitext. Jeder Satz ist an eine gemessene Tatsache gebunden; ein Satz ohne gemessene Tatsache ist ein **Build-Fehler** (Gate `claim-generator`). Beispiele:

- **Klasse C1:** „Der Scope ist gesperrt. Klartextstrukturen sind bereinigt. Persistente Kopien sind nur als Chiffrat vorhanden. Die zugehörigen Schlüssel sind vernichtet. Dies gilt unter den Annahmen A-FLASH, A-ALGO, A-KEYLEAK, A-RAM. Folgende Ausschlüsse bestehen: …“
- **Klasse M:** wie C, ergänzt um die vollzogene physische Bereinigung und die Liste der Pflicht-Ausschlüsse.
- **Klasse L:** „Der Scope ist für alle Lesezugriffe gesperrt. Eine Bereinigung der Daten ist **noch nicht** gemessen.“

Das Gate `deletion-honesty` vergleicht jeden **statischen** Außentext (README, MCP-Werkzeugbeschreibungen, `doctor`-Texte) mit den Claims, die der Bericht der Standardkonfiguration (K1: höchstens C1) ausstellt; ein Text, der mehr behauptet, ist Gate-Fehler (REQ-DEL-035, AC-19, AC-24). Die Bezeichnung „Außenformulierung der Fassung 2.2.1“ („Der lokale Schlüssel wurde nicht als Löschmaßnahme vernichtet“) entfällt, weil sie der Wirklichkeit in 2.3 widerspricht.

#### C-12.12 Abnahmekriterien Löschung

Eine Löschung gilt erst als bewiesen, wenn alle gültigen Kriterien grün sind. AC-7, AC-12, AC-14, AC-15 bleiben **unbelegt** (Inhalt der Fassung 1.3 nicht rekonstruierbar, §25.3); die Nummern werden nicht neu vergeben. Die Epochen-Anforderung deckt AC-21 ab.

|ID|Kriterium|Methode|
|---|---|---|
|AC-1|Nach Löschung und Bericht enthält **keine** Datei des Datenverzeichnisses (WAL, SSTables, Blob-Segmente, Index-Artefakte, Checkpoints, Temp, `.bak`, Keystore, KeyTable, Journal) den Canary (≥ 32 Zufallsbyte als Text), dessen Embedding-Bytes, eindeutige Terms oder die Roh-ID. Geprüft in **zwei Sichten**: Rohbytes und entschlüsselte Sicht mit allen überlebenden Schlüsseln. **Positive Kontrolle** in beiden Sichten; sonst ist der Lauf ungültig.|Scanner im Test + unabhängiger `grep -r` + Positivkontrolle|
|AC-2|Wird eine Messung durch eine Konstante ersetzt, schlägt mindestens ein Test fehl.|Mutation|
|AC-3|Absturz an jedem Journalzustand und an jedem Schritt D1 bis D6: danach `Completed` mit vollständigen Messungen oder `Partial`, nie `Completed` ohne Messung.|Crash-Matrix (`FaultVfs`)|
|AC-4|Restore eines Checkpoints vor der Löschung: Betroffener gesperrt, danach erneut gelöscht, `restore_reapplied` im Journal.|Integration|
|AC-5|Zwischen `Frozen` und `Completed` liefert kein Lesepfad Daten des Betroffenen, auch parallel.|loom/Property|
|AC-6|Berichts-Negativsuite: Bit-Flip je Feld, Feldvertauschung, Versions-Downgrade, anderer Mandant, falscher Schlüssel, fehlendes `excluded`, Klasse ohne Annahmen, gefälschtes Siegel ⇒ Fehler mit benanntem Grund.|Unit + Fuzz|
|AC-8|Neustart nach Löschung: gesperrt/gelöscht, Journal konsistent.|Integration|
|AC-9|Mit `ManualClock` und festen Eingaben ist der Bericht bytegleich reproduzierbar.|Golden-Test|
|AC-10|Das Gate `deletion-honesty` erkennt konstante Messwerte, verworfene Prüfergebnisse, unbedingte `Ok(true)`-Verifikation und öffentliche Konstruktoren mit Zählerwerten in einem absichtlich fehlerhaften Beispiel.|Gate-Selbsttest|
|AC-11|Legal Hold ⇒ `Restricted`, keine Löschoperation, **keine Schlüsselvernichtung**, Lesesperre aktiv, `basis` und `until` im Bericht.|Integration|
|AC-13|Gleichzeitiges `put`/`delete` derselben ID ergibt nach Abschluss nie einen sichtbaren Zustand.|loom|
|AC-16|Checkpoint vor der Löschung: Bericht höchstens Klasse C ohne M mit `RetainedUnderRestoreGuard{count, oldest_age, expires_at}`, solange der Checkpoint die Einheit enthält; nach `Rewrite`/`ExpireNow`/Ablauf Klasse M.|Integration + Canary-Scan|
|AC-17|`restore_checkpoint` weist ein Schlüssel-Backup, das `valid_until` überschritten hat oder dessen Alter unbestimmbar ist, mit `RestoreBeyondHorizon` ab; eine Konfiguration mit `key_backup_max_age > erased_set_retention` startet nicht.|Integration + Chaos|
|AC-18|Nach Vernichtung von `K_ref[tenant, epoche]` ist kein Eintrag dieser Epoche in `ErasedSet`, `AuditChain` und Journal mehr einem Subjekt zuordenbar; die Ketten bleiben prüfbar.|Property|
|AC-19|Kein statischer Außentext enthält einen Claim, den der Bericht der Standardkonfiguration (K1) nicht ausstellt.|Gate-Selbsttest|
|AC-20|Mit `ManualClock`: ein `Partial`-Zustand erzeugt nach den drei Schwellen je genau einen Alarm, eine Eskalation bzw. eine Meldung, reproduzierbar, auch nach Absturz.|Integration + Chaos|
|AC-21|**Epoch Seal.** Nach dem Seal: Re-Wrap aller lebenden Schlüssel, `KEK[e]` vernichtet, keine Datei des Datenverzeichnisses enthält Wrap-Nonce oder Wrap-Blob eines vernichteten Schlüssels; Abbruch an jedem Teilschritt endet in Fortsetzung, idempotent.|Crash-Matrix + Property|
|AC-22|**Backup-Trennung.** Kein Daten-Backup enthält Schlüsselmaterial (Format- und Literalscan). Backup vor Löschung, Löschung, Restore mit altem und mit frischem Schlüssel-Backup: Scope bleibt gesperrt, kein lesbarer Rest; abgelaufenes Schlüssel-Backup wird verweigert.|Backup-/Restore-Test|
|AC-23|**Mixed-Segmente.** `BlockAll`, `RedactAndRekey`, `DeleteAll` verhalten sich nach §12.15; kein Segment wird automatisch freigegeben oder gelöscht.|Integration|
|AC-24|**Claim-Generator.** Jeder Satz der Satzbibliothek ist an einen Messwert gebunden; ein absichtlich überzogener Text lässt das Gate fehlschlagen; die Perspektivenurteile folgen den Messwerten.|Gate-Selbsttest|
|AC-25|**Nadelscan.** Eine absichtlich zurückgelassene Wrap-Nonce (Test-Mutation) wird gefunden; ohne Nonce 0 Treffer.|Mutation + Property|
|AC-26|**Kein Dummy-Bericht.** Kein Codepfad erzeugt einen `DeletionProof` ohne Scanner-Witness; `rg` nach `default_test_deletion_proof_key` und `new_after_verified_empty(…, 0)` im Produktionscode ergibt 0 Treffer; Mutation (Witness-Konstante) lässt Tests scheitern.|Gate + Mutation|
|AC-27|**Keine Wiederauferstehung.** Nach jedem Crash und jedem Restore steht kein lebender Schlüssel einer `ErasedSet`-Einheit in der KeyTable.|Crash-Matrix + Property|
|AC-28|**Destroy-Durabilität.** Stromausfall an jedem Punkt von D1 bis D6 (`FaultVfs`): Schlüssel lebt und Einheit bleibt `Frozen`, oder Schlüssel ist vernichtet; nie „vernichtet“ gemeldet bei vorhandenem Schlüssel.|Chaos|
|AC-29|**Mirror.** Restore nach Totalverlust: mit aktuellem `ErasedSetMirror` bleibt der Scope gesperrt; ohne aktuellen Mirror nur mit `--accept-unguarded`, `restore_unguarded` im Bericht.|Integration|
|AC-30|**Padding und Header.** Kein Record trägt `doc_key_id` im Klartext; die Längenverteilung bleibt nach Padmé unter dem Overhead von 12 %; der Restmetadaten-Selbsttest (§12.7.6) schlägt an einem absichtlich unpaddeten Beispiel an.|Format-Test + Property|
|AC-31|**Rücknahme.** `Unfreeze` nach D3 wird abgewiesen; im Regulated-Tier braucht `Unfreeze` zwei Personen.|Integration|
|AC-32|**Uhr.** Bei Uhrrücksprung verweigern Horizont-, Karenz- und `valid_until`-Entscheidungen fail-closed (`ClockRegression`).|Chaos (Uhrsprung)|
|AC-33|**Verification/Validation.** Im Regulated-Tier ist ein Bericht ohne Concurrence-Signatur `ValidationPending` und gilt nicht als Abschluss.|Typestate + Integration|
|AC-34|**Heap-Canary.** Nach Klasse P findet der Prozessspeicher-Scan keinen Canary, kein Embedding-Byte-Muster und keinen seltenen Term des Scopes; die Positivkontrolle vor der Löschung findet sie; eine absichtlich nicht zeroisierte Kopie (Test-Mutation) wird gefunden.|Heap-Scan + Mutation|
|AC-35|**Zweistufige Messung.** Auf 1.000 seed-reproduzierbaren Szenarien (mit Fehlerinjektion) liefern M1 bis M3 und der volle Rohscan dasselbe Urteil; ein Block ohne UnitDirectory-Zuordnung bricht die Messung (fail-closed).|Differentialtest + Property|
|AC-36|**Shard-Seal.** Nach dem Seal enthält keine Datei Wrap-Nonce oder -Blob eines vernichteten Schlüssels; nur geänderte Shards wurden neu geschrieben (Zähler); der Aufwand wächst mit der Zahl geänderter Shards, nicht mit der Gesamtzahl der Schlüssel; Abbruch an jedem Teilschritt endet in Fortsetzung.|Crash-Matrix + Benchmark|
|AC-37|**Koaleszenz.** n Löschungen in einem Fenster erzeugen n gültige Berichte mit einem Messlauf; keine Einheit erhält eine höhere Klasse, als ihre eigene Messung trägt; Siegel 1 erscheint unabhängig vom Fenster sofort.|Integration + `ManualClock`|
|AC-38|**Flash-Härtung.** `flash_hardening` im Bericht folgt dem tatsächlichen Ablauf: K1-Seal ohne Passphrasenwechsel ⇒ `None`; mit Wechsel ⇒ `PassphraseRotated`; ein konfigurierter, nicht ausgeführter Wechsel ändert nichts.|Typestate + Integration|

#### C-12.13 Löschpfad: Komponenten und Platzierung

**Entwurfsprinzipien:** (1) Messen vor Behaupten: Ein Claim existiert nur, wenn ein Typ, der nur aus einer Messung konstruierbar ist, ihn trägt. (2) Fail-Closed: keine Bescheinigung bei Teilausfall, keine Fallbacks. (3) Verteidigung in der Tiefe: Sperre → logische Löschung → Klartextbereinigung → Schlüsselvernichtung → Seal → Hygiene → Messung → Bericht. (4) Ehrliche Grenzen (§2.6, §12.20). (5) Kein Architekturverstoß: Ports in `ports`, reine Logik in `crypto`, Persistenz in `store`, Orchestrierung in `engine`, Handler dünn. (6) Datensparsame Evidenz. (7) Determinismus: Zeit nur über `Clock`, IDs aus Zählern und Hashes, Schlüsselmaterial nur aus dem System-RNG.

|Komponente|Crate|Ring|Inhalt|
|---|---|---|---|
|IDs, Fehler, Grundtypen: `ErasureId`, `SubjectRefHash`, `UnitRefHash`, `ErasureError`, `DeletionLayer` (zehn Varianten), `LayerWitness`-Grundtypen, `ReachedClass`, `Assumption`|`types`|0|reine Datentypen|
|Ports: `LayerScanner`, `Purger`, `ErasureJournal`, `Vfs`, `DurableLog`, `KmsProvider`, `EffaceableKeySlot`, `ErasedSet`-Zugriff, `ErasedSetMirror`, `RecipientLedger`, `TimestampAuthority` `[R]`|`ports`|0|nur Traits|
|Reine Krypto- und Kodierungslogik: Bericht V5 (Signatur, Verifikation, Satzbibliothek), `ResidualReport`, KeyTable- und Wrap-Logik, Padmé, Kettenlogik|`crypto`|0|synchron, I/O-frei, lock-frei|
|Persistenz: atomare Dateien, KeyTable-Datei, `DurableLog`-Impl, Journal-Dateien, Rohdatei-Scanner|`store`|1|`fsync`, Temp-Datei und Rename, Verzeichnis-`fsync`|
|Scanner/Purger je Schicht: LSM/WAL/Blob/KeyMaterial (`store`), HNSW (`vector`), Volltext (`text`), Checkpoint (`checkpoint`), Caches/Audit/Lineage (`store`/`engine`)|je Crate|0/1/3|implementieren `LayerScanner`/`Purger` (Default-`scan` über `ScanSession`)|
|Orchestrator: Saga, Lesesperre, Key-Lock, Epoch Seal, Restore-Guard, Legal-Hold-Gate, Fristen|`engine`|3|verdrahtet Scanner/Purger über Ports; `crypto` ist nicht optional|
|Werkzeug `memory_forget`, CLI (`doctor`, `confirm`, `verify-report`, `seal-now`)|`mcp` über `contextra`|4|nur Aufruf und JSON-Abbildung; **keine Berichtskonstruktion**|
|Test-Doubles (Fake-Uhr, Fehlerinjektion, Fake-Verwahrer)|`testkit`|–|nur dort|

Löschtypen existieren genau einmal (Kantenregel 5, Gate `deletion-types-once`). Ohne Löschpfad-Orchestrator liefert dessen Konstruktor `Err(CapabilityUnsupported)`, nie einen Stub, der Erfolg meldet (REQ-DEL-027). Die MCP-Antwort enthält `class`, `claims`, `excluded` und den Bericht (oder `null` bei `Partial`/`Restricted`), nie einen Nachweis ohne Messung.

#### C-12.14 Konformität, Risiken, Löschinventar

`crypto` und `engine` bleiben `#![forbid(unsafe_code)]`; Zeit über `Clock`; Schlüsselmaterial ausschließlich über den System-RNG; Produktionscode ohne `unwrap`/`expect`/`panic!`; Verschlüsselungstypen existieren unabhängig von Features (A1).

|Risiko|Gegenmaßnahme|
|---|---|
|Strukturelle Messung übersieht Reste|Nadelscan, Inventar, Canary-Byte-Scan, `excluded` im Bericht; Fachprüfung vor jeder Vermarktung|
|Flash-Reste gewrappter Schlüssel (K1)|Epoch Seal, Passphrasen-Rotation, Annahme A-FLASH ausgewiesen, K2/K3 per Port|
|Schlüssel-Backup enthält vernichteten Schlüssel|befristet (`key_backup_max_age`), Restore-Guard vernichtet beim Einspielen|
|Abbruch während Vernichtung|Destroy-Protokoll D1 bis D6, idempotent, Crash-Matrix|
|Wiederauferstehung durch Restore oder Fehler|INV-NO-RESURRECTION-1, Prüfung in B6/B7|
|Purge-Kompaktion/Segment-Rewrite kostet Latenz und Write-Amplification|Segmentgrößen begrenzen; Messung mit der Write-Amplification-Suite (§16)|
|Padding und Re-Wrap kosten Platz und Zeit|Padmé ≤ 12 %, Re-Wrap O(Schlüssel); Budgets in `budgets.toml` (O-24, O-40)|
|Ablauf der Referenzschlüssel schwächt den Restore-Guard|Horizont bezieht sich nur auf Schlüssel-Backups (§12.9)|
|Fehlbedienung (irrtümliche Löschung)|Karenz `erasure_grace`, Forget-Guard (§13.3), `Unfreeze` bis D3|
|Rechtswertung offen|kein Produkttext ohne fachkundige Freigabe (§25.1)|

**Löschinventar (`doctor`, REQ-DEL-037).** Die Software erzwingt keine Löschfristen, macht ihr Fehlen aber sichtbar. `doctor` weist je Collection ohne Nutzinhalte aus: (1) ob eine Löschfrist/`RetentionPolicy` gesetzt ist (nein = Warnung); (2) Verwahrungsstufe K0/K1/K2/K3; (3) Anteil `LegacyUnkeyed`; (4) interne Checkpoints (Anzahl, ältester, Policy); (5) `erased_set_retention`, `key_backup_max_age`, `kek_epoch_len`, Alter des letzten Seal, Alter der ältesten Schlüssel-Backups, Konsistenz; (6) offene `Partial`-Sagas mit Alter und ausstehender Schicht; (7) `ErasedSetMirror`-Zustand; (8) Härtungslücken (§12.18); (9) offene Residuale (`R-GRAPH`, `R-KEKEPOCH`) mit Frist. Das Inventar ist Beobachtung, keine Rechtsaussage.

#### C-12.15 Betroffenen-Einheiten, Mehrfachbezug, Mixed-Policy

**Einheit.** Eine Betroffenen-Einheit ist die kleinste Menge von Records, die ohne Kollateralverlust für einen Betroffenen gelöscht werden kann. Standard: ein Dokument mit genau einem Betroffenen.

**Referenzen.** Jedes Dokument trägt `subjects: [subject_ref]`; `subject_ref` ist eine zufällige ID mit Mindestentropie (§12.10), vergeben vom Aufrufer (oder durch eine optionale Erkennung `[R]`). Ohne Angabe gilt das Dokument als Einheit des Aufrufers.

**Mehrfachbezug.** Ein Dokument mit mehreren Betroffenen wird in Segmente zerlegt; jedes Segment erhält die Menge seiner Betroffenen; Segmente mit gleicher Menge teilen einen DocDek. Bei Löschung für Betroffenen A:

|Segmentart|Behandlung|
|---|---|
|Menge = {A}|DocDek vernichten|
|Menge enthält A und andere (Mixed)|Standard `BlockAll`: Segment bleibt gesperrt, `Partial{ReviewRequired}` mit Frist; der Betreiber entscheidet|
|Menge enthält A nicht|unberührt|

**Mixed-Policy** (je Collection): `BlockAll` (fail-closed, **Standard**), `RedactAndRekey` (Betreiber liefert redigierten Text; neuer DocDek, Re-Embedding, alter DocDek vernichtet), `DeleteAll` (Segment vollständig löschen; Betreiber trägt die Verantwortung für den Kollateralverlust). **Contextra entscheidet nie selbst**, ob ein Mixed-Segment nach Entfernung von A noch personenbezogen ist (Art. 17 Abs. 3, Rechtsfrage). `BlockAll` ist rechtlich eine Einschränkung (Klasse L für das Segment) und läuft in die Fristenverfolgung (§12.19).

**Ableitungen** (Zusammenfassungen, Chunk-Präfixe, Embedding-Caches) kaskadieren über die Lineage (INV-DERIVED-1); Ableitungen mit weiteren Quellen werden neu abgeleitet oder als `stale` markiert.

**Collection- und Tenant-Löschung (Akte).** Alle DocDeks, ArtKeys und `KEK[coll,*]` werden in einer Saga vernichtet; die Index-Dateien der Collection entfallen; bei Tenant-Löschung zusätzlich `K_ref` des Tenants nach Retention. Die Fassung 2.2.1 behandelte dies als „Hygiene ohne Claim“; in 2.3 ist es eine **Löschmaßnahme mit Bericht** (E27 ersetzt).

#### C-12.16 Index-Generationen und Restinformation

**`R-GRAPH`.** Im HNSW-Graph bleiben nach der Löschung Routing-Knoten ohne Nutzdaten (VETO-F02 verbietet Rewiring). Die Kanten enthalten nur interne Slot-Nummern; die Topologie verrät Nachbarschaftsbeziehungen zu gelöschten Vektoren (No Inference). Sie bleibt als `R-GRAPH` mit Frist im Bericht, bis der Neuaufbau erfolgt ist.

**Neuaufbau.** (1) Hintergrund-Neuaufbau erzeugt Generation g+1 ohne gelöschte Dokumente (deterministisch, INV-INDEX-APPLY-1); (2) atomarer Umschalt-Commit auf g+1; (3) Vernichtung des ArtKey der Generation g. Mehrere Löschungen teilen sich einen Neuaufbau. Auslöser: Tombstone-Anteil ≥ `tombstone_rebuild_ratio` (0,2), Fristablauf (`purge_rebuild_max_days`, Startwert 14), `seal-now`. **VETO-F02 bleibt unberührt:** Der Neuaufbau ist ein vollständiger Generationswechsel, kein partielles Rewiring.

**Restkanäle.** Treffermengen: Ghost-Knoten verändern in der Studie 95 % der Top-K-Mengen und erzeugen einen Timing-Unterschied von rund 4 %; der Neuaufbau beseitigt sie. Statistik (IDF, Dokumentlängen, Zähler) liegt im Text-Segment und wird mit der Segment-Neuschreibung neu berechnet.

**Einordnung.** Qdrant, Milvus, Weaviate und Chroma/FAISS behandeln Löschung als Wartung ohne Frist und ohne Nachweis (Sekundärquellen). Contextra erzwingt die Frist und misst.

#### C-12.17 Backup und Restore (Zwei-Spuren-Backup)

##### C-12.17.1 Spuren

|Spur|Inhalt|Lebensdauer|Schutz|
|---|---|---|---|
|**D** (Daten)|SSTables, WAL, Blob-Segmente, Index-Artefakte (Chiffrat), **ohne** KeyTable, **ohne** Keystore, ohne `ErasedSet`|beliebig (Betreiberentscheid)|ohne Schlüssel nicht lesbar|
|**K** (Schlüssel)|KeyTable-Snapshot (gewrappte DocDeks, ArtKeys, KEK-Epochen, `K_ref`)|**befristet** auf `key_backup_max_age` (Startwert 30 Tage), signiertes `valid_until`|verschlüsselt unter der Wurzel|

**Wirkung.** Nach Löschung und Ablauf aller Schlüssel-Backups, die den DocDek enthalten, existiert der Schlüssel nirgends mehr; Daten-Backups sind für diese Einheit unlesbar. Die Löschung wirkt in Backups spätestens nach `key_backup_max_age`, **ohne Eingriff in das Backup**. Bericht: `backup_window = OpenUntil(deletion_at + key_backup_max_age)` für von Contextra verwaltete Backups, `ExternalUnmanaged` für alles andere.

**Regeln.** INV-BACKUP-SPLIT-1: Kein Daten-Backup enthält Schlüsselmaterial; jedes Schlüssel-Backup trägt ein Ablaufdatum und wird danach verweigert. Der interne `checkpoint` ist Spur D (§12.5.4); `export` schreibt Spur D und optional Spur K nur über einen eigenen, protokollierten Befehl (`export-keys`), der `valid_until` setzt.

##### C-12.17.2 Restore-Guard und Horizont

Siehe §12.9. Zusätzlich: Ein Restore **ohne** Schlüssel-Backup lässt Daten inert (Q6). Ein Restore mit Schlüssel-Backup innerhalb `valid_until` stellt Schlüssel zurück, die der Guard für jede `ErasedSet`-Einheit sofort wieder vernichtet, **bevor** B9 Verkehr zulässt.

##### C-12.17.3 `ErasedSetMirror`

Nach Totalverlust fehlt der `ErasedSet` des Zielsystems. Der `ErasedSetMirror` (Port, Magic der Einträge `MFEM`) schreibt jeden `ErasedSet`-Eintrag (Referenz-Hashes, Zeitpunkt, Epoche) zusätzlich an ein vom Betreiber bestimmtes Ziel **außerhalb** des Datenverzeichnisses (zweiter Datenträger, WORM, Fremdsystem). Im Regulated-Tier ist er **Pflicht**. Ohne Mirror oder bei einem Mirror, der älter ist als das Schlüssel-Backup, ist ein Restore nur mit `--accept-unguarded` möglich; Bericht und `doctor` weisen `restore_unguarded` aus. **Schreibreihenfolge (H-11):** Ist der Mirror beim Einfrieren nicht erreichbar, blockiert das `Frozen` **nicht** (Löschen darf nicht an Fremdsystemen scheitern); der Eintrag wird `MirrorPending`, `doctor` alarmiert nach 24 h, und der Bericht trägt `residuals: [R-MIRROR until]`. Ein zentrales Löschregister (O-35) ist ein zulässiges Mirror-Ziel.

##### C-12.17.4 Verhältnis zu Aufsichtsauffassungen

CEF Issue 6 und BayLfD 12.5 verlangen Nachverfolgung und Nachlöschung beim Restore sowie Begründung, wenn Backups nicht sofort bereinigt werden; der Zwei-Spuren-Ansatz erfüllt dies mit einem festen, dokumentierten Zeitfenster. Ob ein Chiffrat ohne Schlüssel für den Besitzer des Daten-Backups keine personenbezogene Information mehr ist, bleibt P-02.

#### C-12.18 Härtung (Prozessspeicher, Dumps, Swap)

NIST verlangt für CE, dass vor der Verschlüsselung kein Klartext auf das Medium gelangt ist (LA-15). Contextra setzt:

1. **Core-Dumps aus:** `RLIMIT_CORE = 0`, Prozessattribut gegen Dumps; Schlüssel- und Klartext-Arenen sind von Dumps ausgenommen (`MADV_DONTDUMP` bzw. Plattformäquivalent).
2. **Schlüsselspeicher gesperrt:** `mlock` für Schlüsselpuffer; meldet die Sperrung Fehler, weist `doctor` dies aus.
3. **Swap:** `doctor` prüft, ob Swap aktiv und verschlüsselt ist; große Arenen lassen sich nicht sperren, daher Warnung bei unverschlüsseltem Swap.
4. **Temp-Dateien:** keine Klartext-Zwischenspeicher; Zwischenergebnisse liegen im Speicher oder verschlüsselt unter ArtKey/`K_eng`.
5. **Dateisystem:** unterstützte lokale Typen (INV-PLATFORM-1); Journal, Snapshots (APFS, btrfs, zfs) und Time Machine gelten als externe Kopien und stehen im Bericht.
6. **Logs:** nur IDs und Zähler (INV-LOG-1).
7. **Hibernation:** `doctor` weist auf aktive Hibernation hin (A-RAM).

Die plattformspezifischen Aufrufe sind in Spike S-HRD zu verifizieren (O-38). Bis dahin gilt: Fehlt ein Mittel, meldet `doctor` die Lücke, und A-RAM bleibt im Bericht. Im Regulated-Tier ist eine bekannte Härtungslücke eine **Startwarnung mit Bestätigungspflicht**, kein stiller Betrieb.

#### C-12.19 Fristen, Empfänger, Legal Hold

##### C-12.19.1 Fristen

|Frist|Quelle|Wert|
|---|---|---|
|Beantwortung|Art. 12 Abs. 3|ein Monat; Verlängerung um zwei Monate mit Mitteilung binnen eines Monats|
|`Frozen` (Klasse L)|intern|sofort nach Autorisierung|
|Klasse P|intern, Zielgröße|≤ 1 Stunde|
|Klasse C*|intern|unmittelbar nach P (zuzüglich `erasure_grace`)|
|Epoch Seal|intern|≤ `seal_after_deletion_max` (24 h; Profil `Maximum` 1 h; Regulated ≤ 24 h)|
|Hygiene (Klasse M)|intern, P-11|≤ `purge_rebuild_max_days` (14 Tage)|
|Alarm bei `Partial`|intern|24 h|
|Eskalation bei `Partial`|intern|14 Tage|
|Meldung Fristgefährdung|intern|25 Tage|

Die Werte sind Zielgrößen. **Contextra sendet nichts nach außen**; Alarme erscheinen in `doctor`, Metrik, CLI, MCP-Antwort und AuditChain. Ob eine Verzögerung zulässig ist, entscheidet die Technik nicht (P-11, P-19).

##### C-12.19.2 `RecipientLedger` (Art. 17 Abs. 2, Art. 19, P-12)

Je Dokument führt Contextra ein Register der Ausleitungen (Export, Egress-Kanal `[R]`, Checkpoint-Export) **ohne Inhalte**: Empfängerkennung, Zeitpunkt, Datenklasse. Bei einer Löschung liefert `doctor` die Empfängerliste. Die Mitteilung an Empfänger erfolgt durch den Verantwortlichen. Ausleitungen außerhalb von Contextra sind Ausschluss (§12.20).

#### C-12.20 Ausschlüsse (verpflichtend im Bericht) und Nicht-Gegenstand

**Ausschlüsse** (`excluded`, nie leer; der Text darf bei Weitergabe nicht gekürzt werden):

- `ExternalBackups`: Sicherungen außerhalb der Spur D/K (A-EXTBACKUP);
- `ProcessMemoryResidue`: Prozessspeicher, Swap, Dumps, Hibernation vor der Vernichtung (A-RAM);
- `HistoricalMediaStates`: hardwareinterne Reste (Wear-Leveling, Controller-Cache); Gegenmaßnahme Verschlüsselung plus Epoch Seal, Restannahme A-FLASH bei K1;
- `ModelParameters`, `ExfiltratedData`: Trainings- und Fine-Tuning-Daten externer Modelle, Kopien beim Auftragsverarbeiter, Modellanbieter-Logs, Prompt-Caches;
- `HostHistories`: Host-Verläufe (A-HOSTHIST);
- `DerivedOutsideLineage`: Ableitungen außerhalb der Lineage (vom Betreiber angelegte Exporte);
- `FilesystemSnapshots`: Dateisystem-Snapshots und Journale;
- `ActiveSnapshot{ids}` (§12.5.5).

(Der Ausschluss `KeyMaterialBackups` der Fassung 2.2.1 entfällt: Schlüssel-Backups sind Spur K und erfasst; Schlüsselkopien **außerhalb** von Spur K stehen unter `ExternalBackups`.)

**Nicht Gegenstand:** maschinelles Verlernen in Modellgewichten; Löschung beim LLM-Anbieter; Löschung in Backup-Medien außerhalb der Spuren D/K; rechtliche Wertung der Löschwirkung.

### C-13. Schnittstellen, Laufzeit und Plattform

Dieser Abschnitt legt die Außenflächen fest, über die Contextra betrieben wird: die Rust-API, den MCP-Server, die CLI und den lokalen IPC-Proxy. Er ergänzt die Verdrahtungs- und Persistenzregeln aus §5, §7 und §12.

#### C-13.1 Zugänge und Betriebsmodell

- **Rust-API:** `contextra` (Fassade, Composition Root) und `contextra-db` (Collections, Transaktionen). Öffentliche Funktionen: `Contextra::open`, `db()`, `remember`, `recall`, `forget`, `report(id)`, `restore_checkpoint`, `pin_checkpoint`, `as_of`, `shutdown`.
- **MCP-Server (`contextra-mcp`):** Transport ist ausschließlich **stdio, JSON-RPC 2.0**. `stdout` gehört dem Protokoll (INV-STDOUT-1); Logs gehen nach `stderr` oder in eine Datei. Es gibt keinen TCP- oder UDP-Listener (INV-IPC-1).
- **Primärprozess und Proxy (INV-IPC-1):** Genau ein Primärprozess hält das Datenverzeichnis und die Engine. Jeder weitere MCP-Host-Start erkennt den Primär über Lock- und Endpoint-Datei und arbeitet als **stdio↔IPC-Proxy**. IPC ist ein Unix-Domain-Socket (Verzeichnis `0700`, Socket `0600`) bzw. eine Named Pipe mit ACL. Peer-Credentials (`SO_PEERCRED`, `getpeereid`, `GetNamedPipeClientProcessId`) werden geprüft. Der Prinzipal besteht aus OS-Nutzer und Client-Name und wird über die Konfiguration einem Tenant zugeordnet. Der Primärprozess endet nach `idle_timeout` (Startwert 15 min) ohne Clients; fällt er aus, übernimmt der nächste Client nach Recovery. Alle Clients teilen dieselben Garantien.
- **CLI:** `init`, `doctor` (inkl. `verify-report`, `export-concept`, `crypto-drill`), `status`, `confirm <token>`, `migrate`, `seal-now` (optional `--rotate-passphrase`), `export-keys`. Ein MCP-Statuswerkzeug ist in v1.0 nicht enthalten (O-21).

#### C-13.2 Werkzeugoberfläche (MCP)

In v1.0 stehen genau **drei Agentenwerkzeuge** bereit: `memory_remember`, `memory_recall`, `memory_forget`. Defaults für Tenant, Scope und Budget kommen aus der Konfiguration, nie aus dem Aufruf (INV-TENANT-1). Jedes Werkzeug trägt MCP-Annotationen (`readOnlyHint`, `destructiveHint`), die aus seiner Kategorie erzeugt werden: `memory_recall` ist `Read`, `memory_remember` ist `Write`, `memory_forget` ist `Destructive`.

Der Vollwerkzeugsatz `contextra_*` der Vorgängerfassung (Kategorien `Read`, `Write`, `Destructive`, `Admin`, `Egress`, `Plugin`) entfällt für v1.0 (E36, §0.5). Administrative Funktionen laufen über CLI und Fassade. Änderungen an dieser Festlegung sind ADR-pflichtig (§18.6).

**Sicherheitspipeline (fünf Stufen je Aufruf, REQ-MCP-004):**
1. Schema- und Größenvalidierung (RPC-Größe, Argumentgrenzen; fuzz-geprüft, REQ-MCP-008).
2. Authentifizierung und Prinzipal (§13.1).
3. Autorisierung gegen Kategorie, Scope und `RevocationLog` (fail-closed).
4. Budget-, Rate- und Ausleitungsprüfung (`max_in_flight_per_principal`, `max_response_tokens`).
5. Audit-Eintrag vor und nach der Ausführung (ohne Nutzinhalte, INV-LOG-1).

#### C-13.3 Forget-Guard (INV-FORGET-GUARD-1)

`memory_forget` mit **einer** ID und `dry_run` als Standard ist ohne weitere Bestätigung zulässig. Folgende Fälle sind bestätigungspflichtig:

- Selektor-Löschungen,
- Löschungen auf Collection- oder Tenant-Ebene,
- Löschungen von mehr als `forget_confirm_threshold` Dokumenten (Startwert 10).

Das Werkzeug antwortet dann mit `ConfirmationRequired { token, preview }`. Der Token ist **nur über einen Kanal außerhalb des Tool-Aufrufs** einlösbar, z. B. `contextra confirm <token>` oder ein Dialog des Betriebssystems. Das LLM selbst kann den Token nicht einlösen. Damit ist eine durch Prompt-Injection ausgelöste Massenlöschung ausgeschlossen (REQ-MCP-005).

#### C-13.4 Fassade und Composition Root

Die Fassade `contextra` ist der **einzige** Ort, an dem konkrete Implementierungen gewählt und verdrahtet werden (P29, REQ-FAC-001). `mcp` und `py` rufen nur ihre API auf. Die Fassade:

1. liest Konfiguration, Tier und Profil (§14, §13.7);
2. instanziiert die Ports in Ringreihenfolge: `Vfs`-Implementierung aus `store`, dann `KmsProvider` aus `crypto`, dann `store`, dann die Indexe `vector` und `text`, dann `engine`, dann `checkpoint` und `db`. Rand-Module werden nur bei aktiviertem Feature eingebunden (§4.7, Anhang A);
3. registriert Observer (WAL → Indexe, Audit, CDC) und injiziert Embedder und Reranker als Ports (§5.2);
4. führt beim Start die Phasen B0–B9 aus (§13.5), einschließlich eines **Verdrahtungs-Selbsttests**: jeder Sicherheitsport hat eine reale Implementierung (INV-WIRING-2, REQ-FAC-002);
5. stellt die öffentliche Rust-API bereit (§13.1).

Ohne Löschpfad-Orchestrator liefert dessen Konstruktor `Err(CapabilityUnsupported)` und nie einen Stub, der Erfolg meldet (REQ-DEL-027, §12.13).

#### C-13.5 Startsequenz B0–B9 (INV-BOOT-1, REQ-FAC-005)

Der Start folgt der festen Phasenfolge. Jede Phase kann den Start mit benanntem Grund verweigern. Teilstarts sind unzulässig.

| Phase | Schritt | Verweigert den Start bei |
|---|---|---|
| B0 | Konfiguration, Tier und Profil prüfen (§14). Insbesondere gilt `key_backup_max_age ≤ erased_set_retention` (§12.9); im Regulated-Tier sind `erasure_grace`, `seal_after_deletion_max`, `erased_set_retention` und ein `ErasedSetMirror`-Ziel Pflichtwerte. | ungültiger Konfiguration, Tier-Verletzung |
| B1 | Datenverzeichnis: Dateisystemtyp (INV-PLATFORM-1), Verzeichnissperre (INV-IPC-1) | nicht unterstütztem Dateisystem (NFS/SMB, unbekanntes FUSE, Cloud-Sync-Ordner), belegter Sperre |
| B2 | Schlüssel: `KmsProvider` und `K_eng` | fehlendem Keystore oder nicht entschlüsselbarem Keystore |
| B3 | Sidecar-Logs validieren (Magic, Kette, Manifest-Marke, §7.9) **und** Schichtbelegung prüfen: `Graph` und `KvCache` sind nur dann `FeatureDisabled`, wenn keine Daten dieses Typs vorhanden sind (§12.4) | gebrochener Kette, fehlender Sidecar-Datei bei vorhandenen Daten, Daten eines deaktivierten Schichttyps |
| B4 | WAL- und Manifest-Recovery (§7.3.1, Schritte 1–5) | Lücke in der WAL-Kette (INV-FAILCLOSED-1) |
| B5 | Ausstehende `SidecarIntent`s idempotent anwenden (§7.3.1, Schritt 6) | dauerhaft scheiterndem Intent |
| B6 | `ErasedSet` laden und Lesesperre armieren (INV-BOOT-1, REQ-RST-1) | unlesbarem oder manipuliertem `ErasedSet` |
| B7 | Löschsagas aus dem `ErasureJournal` fortsetzen oder als `Partial` melden (REQ-FGT-2); unterbrochene Destroy-Protokolle (D1–D6) und Epoch Seals fortsetzen; **Wiederauferstehungsprüfung** (INV-NO-RESURRECTION-1); Retention-Entscheidungen in `PendingDeletion` überführen (§11.6) | gebrochener Journal-Kette |
| B8 | Verdrahtungs-Selbsttest der Fassade (§13.4, Punkt 4) | fehlender Sicherheits-Port-Implementierung (INV-WIRING-2) |
| B9 | Verkehr zulassen | — |

**Regeln:** (1) Lesepfade sind erst nach B6 zugelassen; B9 setzt B6 voraus. (2) Die Reihenfolge wird durch ein Gate geprüft (`boot-order`, §17.3). (3) Die Zuordnung der Schichtbelegung zu B3 ist eine Festlegung dieser Fassung; die Vorgängerfassung hatte sie in B3 nicht vorgesehen.

#### C-13.6 Fehlermodell (REQ-MCP-010, A6)

Alle Crates münden in `ContextraError`. FFI und MCP geben stabile Fehlercodes aus. Jeder Fehler ist **retrybar** oder **final**.

| Klasse | Codes |
|---|---|
| retrybar | `SerializationFailure` (SSI-Konflikt), `Busy` (Schreibstau, `max_in_flight_per_principal`), `StorageFull` (ENOSPC mit Notreserve) |
| final | `LimitExceeded`, `QuotaExceeded`, `Denied`, `Corrupt`, `ConfirmationRequired`, `CapabilityUnsupported`, `EmbedderMismatch`, `RestoreBeyondHorizon`, `SnapshotExpired`, `TenantErased`, `ResidueRemaining` (nur als Sagazustand), `Poisoned` (nur Neustart hilft) |

Ein Fehler darf keine Nutzinhalte enthalten (INV-LOG-1).

#### C-13.7 Plattform, Garantiebereich und Performance-Profile (INV-PLATFORM-1)

**Garantieplattformen.** Durabilitäts- und Löschgarantien gelten nur auf den Plattformen der folgenden Matrix mit Tier 1 und Tier 2.

| Plattform | Tier | Besonderheiten |
|---|---|---|
| Linux | 1 | Referenz; `fdatasync`; `mlock` wirksam |
| macOS | 2 | `F_FULLFSYNC` Pflicht; `mlock` begrenzt; APFS-Snapshots und Time Machine sind externe Kopien (deklariert) |
| Windows | offen (O-18) | offene oder gemappte Dateien sind nicht löschbar (Handles freigeben, `FILE_SHARE_DELETE`); ACL über `sys`; Virenscanner-Ausschluss für das Datenverzeichnis wird empfohlen (`doctor`-Hinweis) |

**Dateisysteme.** Unterstützt sind lokale ext4, xfs, btrfs, zfs, APFS und NTFS. Abgelehnt im persistenten Betrieb sind Netzwerk-Dateisysteme (NFS, SMB), unbekannte FUSE-Mounts und Cloud-Sync-Ordner (z. B. OneDrive, Dropbox, iCloud Drive). `doctor` erkennt sie, und der Start verweigert sie (B1, §7.12). Jede Garantieplattform führt CI-Läufe für die Durability-Primitive, die Stufen S2 und S3 sowie `mmap` aus. Auf anderen Plattformen ist der Betrieb Best Effort: `doctor` weist `BestEffort` aus, und Berichte tragen `platform_guarantee = false` (REQ-PLT-001). Die Aufnahme von Windows als Garantieplattform für den Server und den MCP-Betrieb ist in v1.0 ausgeschlossen (§0.5). Ob Windows als Bibliothek Best Effort unterstützt wird, ist offen (O-Punkt, §25).

**Durability-Primitive.** Der Commit-Punkt nutzt die plattformgerechte Primitive (§7.7). Löschlogik stützt sich nie auf die Unix-Unlink-Semantik offener Dateien (REQ-PLT-003).

**PerformanceProfile.** Die Profile legen Gruppierung und Prefetch fest, nie die Korrektheitsinvarianten:

- `Compliance`: maximale Durabilität (Flush je Commit oder kleine Gruppen), Audit synchron.
- `Balanced`: Group Commit, Standard-Audit.
- `BareMetal`: größere Gruppen, aggressives Prefetch.

**DurabilityMode.** Es gibt genau zwei Modi: `Full` (WAL, HMAC-Kette und AEAD; Standard und einziger Modus für `Compliance`) und `MemoryOnly` (keine Persistenz, keine Verzeichnissperre). Einen Modus ohne Integritätskette gibt es nicht (`WalNoHmac` entfällt, REQ-DUR-009). In `Full` sind DirLock und Recovery-Prüfungen nicht abschaltbar.

#### C-13.8 Checkpoints, Backup und Restore

- **Erzeugung:** Ein Checkpoint ist ein konsistenter Snapshot an einem Sequenzstand. Seine Erzeugungszeit steht im signierten Manifest (`Clock`-Port). Checkpoint-Erzeugung und Export prüfen den `ErasedSet` vor dem Schreiben (REQ-RST-4).
- **Export:** verschlüsselt und **ohne Keystore und ohne KeyTable** (Spur D). Schlüssel werden nur über den eigenen Befehl `export-keys` als Spur K mit signiertem `valid_until` exportiert (§12.17).
- **Restore:** `restore_checkpoint` verifiziert Kette, Manifest und Format, sperrt vor dem ersten Lesezugriff, liest den `ErasedSet` des **Zielsystems** und wendet jede bekannte Löschung erneut an. Jede Wiederanwendung erzeugt im Journal ein Ereignis `restore_reapplied` (REQ-DEL-013, REQ-RST-2).
- **Restore-Horizont (INV-RESTORE-HORIZON-1):** Schlüssel-Backups (Spur K), deren `valid_until` überschritten oder deren Alter nicht bestimmbar ist, werden mit `RestoreBeyondHorizon` abgewiesen (REQ-RST-3, §12.9, §12.17). Daten-Backups (Spur D) enthalten keine Schlüssel und unterliegen keinem Horizont.
- **Checkpoint-Policy:** je Collection `Rewrite` (Standard), `ExpireNow` oder `ExpireAfterHold` (nur unter `Restricted`/Legal Hold, §12.5.4). Interne Checkpoints sind Spur D und enthalten keine KeyTable. Ein gepinnter Checkpoint (`pin_checkpoint`) zählt zur Schicht `Checkpoints`.

#### C-13.9 Embedded-API und Collections

`contextra-db` stellt Collection-Handles und Transaktionen bereit. Die Logik liegt in `engine`; `db` enthält keine eigene Geschäftslogik (§4.3). Jede Collection persistiert `embedding_model_hash` und Dimension (INV-EMBED-2). Die Collection ist die Produkt-Einheit „Akte“ (Annahme O-14, §25). `drop_collection` ist eine Löschung im Sinne von §12.8 und läuft durch den Forget-Guard (§13.3).

### C-15. Startwerte und Kalibrierung

Werte mit **„fest“** sind unveränderlich. Werte mit **„Startwert“** sind zu kalibrieren; die Kalibrierung läuft über die Benchmarks aus §16 und verändert den Code nur über ein Gate. Jede Konstante hat genau eine Quelle im Code oder in der Konfiguration (INV-DOC-SINGLE-SOURCE-1).

#### C-15.1 Formate und Kryptografie (fest)

| Parameter | Wert |
|---|---|
| Magics | WAL `MFW3`, SSTable `MFSX` (Legacy `MFST` entscheidet WP 2.4), Blob `MFBL`, Manifest `MFMN`, Keystore `MFKS`, Sidecars `MFKR`, `MFRV`, `MFAU`, `MFEJ`, `MFES`; jede Datei trägt zusätzlich eine Formatversion (INV-FORMAT-1) |
| WAL-Integrität | HMAC-SHA256-Kette (Vorgänger-Tag im Record) und AEAD |
| AEAD | AES-256-GCM-SIV; Nonce 96 Bit zufällig; Tag 128 Bit (28 Byte Overhead je Record); AD = `scope_id ‖ key_id ‖ doc_id_hash` |
| Schlüssel | Wurzel (`EffaceableKeySlot`, K1: Argon2id); `KEK[coll,epoche]`, `DocDek`, `ArtKey`, `K_ref` je 256 Bit zufällig, gewrappt; `K_eng` per HKDF(Label) (§12.2.2, §12.3) |
| KDF-Untergrenze | Argon2id m ≥ 64 MiB, t ≥ 3, Kalibrierung auf ≥ 250 ms (INV-KDF-1) |
| Signatur, Hash | Ed25519; SHA-256 (Merkle, Digest); BLAKE3 (Blob-Digest, Manifest) |
| Löschbericht | `DeletionProof` Formatversion 5 (Magic `MFDR`); V4 bleibt unvergeben (E46) |
| Unsafe-Inseln | genau drei: `simd`, `sys`, `wire` (§4.6) |

#### C-15.2 Storage

| Parameter | Startwert |
|---|---|
| `blob_threshold` | 64 KiB |
| `l0_slowdown` / `l0_stop` | 8 / 20 L0-Dateien |
| `write_stall_timeout` | 5 s, danach retrybar `Busy` |
| Block-Cache | SIEVE hinter Port `BlockCache` |
| `emergency_reserve_bytes` | 64 MiB (mindestens zwei WAL-Segmente plus Journal; bei Start vorab reserviert, §7.10) |
| Recovery-Zeitbudget | je Tier in `benchmarks/budgets.toml` (§16) |

#### C-15.3 Index und Retrieval

| Parameter | Startwert |
|---|---|
| HNSW `m` / `efConstruction` / `efSearch` | 16 / 200 / 64 |
| `MAX_SEARCH_K` | 1000 (auch im Filterpfad) |
| `exact_scan_max` | 20 000 erlaubte IDs |
| `tombstone_rebuild_ratio` | 0,2 |
| Overfetch | `3·k → 6·k → 12·k`, harte Scan-Obergrenze |
| BM25 `k1` / `b` | 1,5 / 0,75 |
| RRF `k` | 60 (Tie-Breaker: Score absteigend, DocId aufsteigend) |
| Designpunkt | ≤ 10⁷ Vektoren und ≤ 10² Mandanten je Knoten (fest, §2.1) |

#### C-15.4 Chunking, Limits, Betrieb

| Parameter | Startwert |
|---|---|
| Chunk-Ziel / Obergrenze | 512 / 1024 Token; Overlap 10–15 % |
| `max_document_bytes` | 16 MiB |
| `max_chunks_per_document` | 4 096 |
| `max_batch_ops` | 10 000 |
| `max_collections_per_tenant` | 256 |
| `max_in_flight_per_principal` | 8 |
| `max_response_tokens` | 4 000 |
| `forget_confirm_threshold` | 10 Dokumente |
| `idle_timeout` (IPC) | 15 min |
| Retention-Karenzzeit / Reaper-Batch | 7 Tage / 100 Dokumente je Tick |
| `snapshot_max_age` | 15 min (REQ-DEL-031) |

#### C-15.5 Löschpfad

| Parameter | Startwert | Herleitung |
|---|---|---|
| `erased_set_retention` | 90 Tage (≥ `key_backup_max_age` + Zuschlag; Pflichtwert ohne Standard in `EnterpriseRegulated`) | Rechtsgrundlage und Frist offen (P-07, P-16); Untergrenze `key_backup_max_age` (§12.10) |
| `key_backup_max_age` | 30 Tage (≤ `erased_set_retention`) | Lebensdauer eines Schlüssel-Backups (Spur K, §12.17); Frist offen (P-06) |
| `kek_epoch_len` | 30 Tage | Epoch Seal zeitgesteuert (§12.2.4) |
| `seal_after_deletion_max` | 24 h (`Maximum`: 1 h; Regulated: ≤ 24 h) | Epoch Seal nach Löschung (H-07); kurz, weil Seal nur geänderte KeyTable-Shards neu schreibt (§12.2.4) |
| `seal_coalesce_window` | 15 min | Löschungen teilen sich einen Seal |
| `measure_coalesce_window` | 60 s | Löschungen teilen sich einen Messlauf (§12.8.9) |
| `full_scan_interval` | 7 Tage | volle Tiefe der Messung M3 (§12.8.9) |
| `n_shards` | 256 | KeyTable-Shards; per Benchmark kalibrieren (O-42) |
| `erasure_grace` | 0 (Obergrenze 72 h; Regulated: Pflichtwert ohne Standard) | Karenz zwischen Klasse P und Schlüsselvernichtung (§12.8.3) |
| `purge_rebuild_max_days` | 14 Tage | Hygiene und Graph-Neuaufbau (§12.8.2, §12.16) |
| `min_record_bytes` | 256 | Padmé-Padding (§12.2.6) |
| `ref_epoch_len` | 90 Tage | Länge einer Referenz-Epoche (§12.10) |
| `checkpoint_policy` | `Rewrite` | Standard (§12.5.4; O-15) |
| `checkpoint_hold` | 25 Tage | nur für `ExpireAfterHold` unter `Restricted`; interne Zielgröße unter der Monatsfrist (P-11, P-15) |
| `partial_alarm_after` | 24 Stunden | Interne Zielgröße (P-11, P-19) |
| `partial_escalate_after` | 14 Tage | dito |
| `partial_deadline_report` | 25 Tage | dito; Monatsfrist Art. 12 Abs. 3 DSGVO bleibt die Obergrenze |
| Retry-Backoff der Saga | exponentiell, Start 1 s, Deckel 5 min, Zeit über `Clock` | deterministisch testbar mit `ManualClock` |
| Canary | mindestens 32 Zufallsbyte als Text | Eindeutigkeit gegen Zufallstreffer (AC-1) |
| `K_ref`-Lebensdauer | Epochenende + `erased_set_retention` | §12.10 |

**KEK-Epochen** (§12.2.4) und **Referenz-Epochen** (§12.10) sind in 2.3 Bestandteil dieser Tabelle; die Lösch-Epochen der Fassung 1.3 (Rotation je Dokument mit Batch-Fenster, `N_epoch`, `L_wrap`) bleiben gestrichen, weil der Epoch Seal sie ersetzt.

#### C-15.6 Kognition und Rand

Die Parameter der Vorgängerfassung für Decay, Working-TTL, `VolatileContextVault`, `ContextPrefixEngine`, `GaspValidator` und `MultiStepEngine` gehören zu archivierten oder randständigen Crates (Anhang B). Sie sind nicht normativ, solange das Feature nicht aktiviert und ein ADR nicht erteilt ist.

#### C-17.4 Chaos und Fehlerinjektion (Jepsen-Stil)

`FaultVfs` (in `testkit`) injiziert seed-reproduzierbar an **jedem** Schreib- und Durability-Punkt: Crash, Torn Write, `fsync`-Fehler (INV-FSYNC-1), ENOSPC (INV-ENOSPC-1), `EMFILE`, Bit-Flips, Umordnung von Schreibvorgängen vor dem Flush, Rechtefehler (REQ-QA-006). Weitere Szenarien: Uhrsprünge über den `Clock`-Port, Absturz des IPC-Primärprozesses mit Übernahme, konkurrierende Clients, Handle-Semantik bei Windows, Abbruch während Restore und Löschkette.

Nach jedem Szenario laufen Recovery und der Invarianten-Checker: keine verlorenen Bestätigungen, Kette gültig, Indexe konsistent, SSI-Historie serialisierbar, nach `forget` ist nichts Gelöschtes lesbar (Scanner je Schicht), und der Sidecar-Zustand ist konsistent. Fehlschläge sind mit Seed exakt replizierbar.

#### C-17.5 Pflichttests (T-*)

Die Pflichttests der Garantien G1–G6 (§2.4) sind in dieser Fassung benannt. Die Nummern sind neu festgelegt; jeder Test hat einen Mechanismus nach §1.4.

| Test | Garantie | Prüfung | Mechanismus |
|---|---|---|---|
| T-DEL-1 | G1 | nach `Completed` keine Klartextreste in LSM, Vektor-, Textindex und Checkpoints (Canary-Scan) | C + I |
| T-DEL-2 | G2 | Bericht entsteht nur aus gemessenen Restzahlen 0 mit der gemessenen Klasse (kein höherer Anspruch) | T + P |
| T-DEL-3 | G2 | Signatur verifiziert mit falschem Schlüssel nicht | P |
| T-DEL-4 | G3 | Absturz an jeder Stelle der Saga führt zu Fortsetzung oder `Partial` | C (Crash-Matrix) |
| T-DEL-5 | G3 | kein Bericht ohne Löschung bei Absturz | C |
| T-DEL-7 | G4 | ab `Frozen` liefert kein Lesepfad Daten des Scopes | L + P |
| T-DEL-8 | G4 | Restore stellt gelöschte Daten nicht wieder her | I |
| T-DEL-9 | G4 | Lesesperre gilt für `as_of` und offene Snapshots (INV-ASOF-ERASE-1) | P + L |
| T-TEN-1 | G5 | kein Mandant liest, findet oder löscht Daten eines anderen | P + F |
| T-TEN-2 | G5 | keine Cross-Tenant-Statistik, -Kante oder -Posting (INV-TENANT-IDX-1) | P + G |
| T-REC-2 | G6 | Zustand nach hartem Abbruch aus dem WAL rekonstruierbar | C + P |
| T-SSI-1 | RB4 | SSI aktiv; Read-Sets mit Phantomschutz validiert (INV-SSI-ACTIVE-1, U-06) | P + C + L |
| T-EMB-1 | — | Modellwechsel wird abgewiesen; Migration nur mit Neuaufbau (§11.1) | I + G |
| T-DET-1 | RB1 | HNSW-Aufbau mit festem Seed ist bit-identisch (INV-INDEX-APPLY-1) | P |

Ergänzend gilt: AC-1 bis AC-38 aus §12.12 sind Abnahmekriterien für die Löschung. AC-7, AC-12, AC-14 und AC-15 bleiben unbelegt (Inhalt der Fassung 1.3 nicht rekonstruierbar, §25.3); die Nummern werden nicht neu vergeben. Neue Pflichttests zu G3/G4/G7: T-DEL-15 (Lösch-Mutationskatalog, §17.11), T-DEL-10 (Destroy-Protokoll, AC-28), T-DEL-11 (Epoch Seal, AC-21), T-DEL-12 (Backup-Trennung, AC-22), T-DEL-13 (Nadelscan, AC-25), T-DEL-14 (keine Wiederauferstehung, AC-27).

#### C-17.8 Orakel und deterministische Simulation (INV-ORACLE-1)

Jede Kerngarantie RB1–RB4 (§2.5) hat ein Prüfmaß, das nicht aus denselben Datenstrukturen stammt und nicht vom selben Erzeuger wie der geprüfte Pfad (EP15):

| Garantie | Orakel |
|---|---|
| RB1 Crash und WAL | Referenzmodell der WAL-Semantik; Crash-Simulation mit `FaultVfs` (seed-reproduzierbar) |
| RB2 Löschung | Canary-Scanner (§17.9), Heap-Canary (§12.7.11) und ein Referenz-Prüfer in anderer Sprache **nur als Testorakel** (nicht Produktbestandteil, REQ-PRF-6) |
| RB3 Audit und Provenienz | unabhängige Referenzimplementierung der HMAC-Kette |
| RB4 Isolation | Brute-Force-Referenz für ANN und ACL; History-Checker für SSI |

Eine Simulation ist deterministisch: gleicher Seed ergibt gleiche Fehlerfolge.

#### C-17.11 Lösch-Mutationskatalog (2.4, G7)

Die Löschzusage ist nur so stark wie der schwächste Test. Das Gate `del-mutation` (nächtlich) fügt je Zeile **einen** Fehler in den Löschpfad ein (Mutant); **jeder** Mutant MUSS von mindestens einem benannten Test getötet werden, sonst ist das Gate rot. Neue Löschschritte erhalten vor dem Merge eine Zeile (A10).

|ID|Mutant|tötender Test|
|---|---|---|
|M-DEL-01|Memtable-Werte werden nicht zeroisiert|AC-1, AC-34|
|M-DEL-02|Antwort- oder RRF-Cache wird nicht invalidiert|AC-1 (entschlüsselte Sicht), AC-5|
|M-DEL-03|SQ8-Code im Arena-Slot wird nicht genullt|`VectorScanner`-Test, AC-1|
|M-DEL-04|Term bleibt im Wörterbuch|`TextScanner`-Test|
|M-DEL-05|alte Text-Segmentdatei bleibt|AC-1, AC-35|
|M-DEL-06|WAL-Scrub lässt Altsegment|AC-1, AC-3|
|M-DEL-07|Blob-Rewrite entfällt|AC-35 (Inventar)|
|M-DEL-08|D4 entfällt (alte KeyTable-Generation bleibt)|AC-25, AC-36|
|M-DEL-09|`KeyCache` wird bei D2 nicht invalidiert|AC-5, AC-28|
|M-DEL-10|D3 ohne Verzeichnis-`fsync`|AC-28 (Crash-Matrix)|
|M-DEL-11|Schritt 5 beginnt vor Siegel 2|AC-31, Typestate (INV-DEL-ORDER-1)|
|M-DEL-12|Scanner liefert konstant 0|AC-2, AC-26|
|M-DEL-13|Positivkontrolle entfällt|AC-1|
|M-DEL-14|Scanner liest nur Teilmenge der Dateien|Dateilisten-Digest gegen Verzeichnis (AC-35)|
|M-DEL-15|`KEK[e]` überlebt den Seal|AC-21, AC-25|
|M-DEL-16|geänderter Shard wird nicht neu geschrieben|AC-36|
|M-DEL-17|Restore-Guard entfällt|AC-4, AC-27|
|M-DEL-18|Lesesperre prüft nur `subject_ref_hash`, nicht `unit_ref_hash`|AC-5|
|M-DEL-19|`as_of` umgeht die Lesesperre|T-DEL-9|
|M-DEL-20|Klasse C1 ohne Epoch Seal ausgestellt|AC-6, INV-CLASS-MEASURED-1|
|M-DEL-21|Satz ohne Messwert in der Satzbibliothek|AC-24|
|M-DEL-22|Padding abgeschaltet|AC-30|
|M-DEL-23|Mirror-Ausfall blockiert `Frozen`|Chaos (AC-29)|
|M-DEL-24|Uhrrücksprung wird ignoriert|AC-32|
|M-DEL-25|alter ArtKey schon in Schritt 3 vernichtet|AC-35 (Inventar vor Vernichtung), INV-DEL-ORDER-1|
|M-DEL-26|`flash_hardening` aus Konfiguration statt aus Ablauf|AC-38|
|M-DEL-27|Koaleszenz hebt die Klasse einer Einheit über ihre Messung|AC-37|
|M-DEL-28|zweistufige Messung weicht vom Vollscan ab (Scanner übersieht Datei)|AC-35|

### C-24. Datenlebenszyklus

#### C-24.1 `remember`

Eingabe, Prinzipal und Tenant aus der Transaktion, Limits und Ausleitungsprüfung, `trust` und `source`, `MemoryType` (Standard `Semantic`), Chunking (§11.7), Embedding über den Port als aufgezeichneter Wert (INV-NONDET-1). Große Werte werden vor dem Commit als Blob-Segment geschrieben (§7.1). Dann folgt **ein** WAL-Batch mit Primärdaten, `IndexIntent`s für Vektor und Text, Lineage und `SidecarIntent` für Audit. Nach dem durablen Flush (Commit-Punkt) wendet der Applier die Indexe an. Die Quittung `RememberReceipt` enthält Dokument-ID, WAL-Sequenz als Provenienz-Anker und `embedder_id`.

#### C-24.2 `recall`

Anfrage, Planner, Snapshot-Sequenz, `ScopeConstraint` und ACL-Bitset. Dann Query-Embedding im Blocking-Pool, Vektorsuche und BM25 parallel am Snapshot mit `max_results`. Die Fusion erfolgt über RRF in `rank/fusion`. Die Lesesperre entfernt Treffer gesperrter Betroffener **vor** der Rückgabe (REQ-DEL-011). Das Ergebnis wird auf `max_response_tokens` begrenzt; eine Kürzung wird ausdrücklich gekennzeichnet. Jeder Treffer trägt `{content, trust, source, lineage}`. Inhalte sind Daten und nie Anweisungen (INV-TRUST-1).

#### C-24.3 `forget`

Autorisierung (Kategorie `Destructive`) und Forget-Guard (§13.3). Danach läuft die Saga nach §12.8.2:

1. **Fast-Path.** `Frozen` (`ErasedSet`-Eintrag durabel, Lesesperre, Spiegelung an den Mirror; Siegel 1, Klasse L), sofortige Rückgabe eines `DeletionReceipt` mit Siegel 1 und `PendingDeletion`-Handle (REQ-DEL-021); Tombstone-Batch und Lineage-Kaskade (INV-DERIVED-1); Ebene-A-Bereinigung und Messung (Siegel 2, Klasse P); nach optionaler `erasure_grace` Schlüsselvernichtung D1 bis D6 (INV-DEL-ORDER-1); Epoch Seal, soweit fällig (Siegel 3, Klasse C\* bzw. C1 bis C3).
2. **Hygiene** (terminiert, ≤ `purge_rebuild_max_days`): Purge-Kompaktion, WAL-Scrub, Blob-Rewrite, Checkpoint-Policy, Graph-Neuaufbau; Messung, Siegel 4 (Klasse M), verankert in der `AuditChain`.

`report(id)` liefert den jeweils höchsten erreichten Stand. Solange offene Snapshots die Hygiene verzögern, lautet der Status `Pending` (§12.5.5). Die Saga ist idempotent und nach einem Absturz wiederaufnehmbar; ihr Zustand steht im `ErasureJournal`. Die Außenformulierung entsteht aus der Satzbibliothek (INV-CLAIM-BIND-1, §12.11).

#### C-24.4 Checkpoint und Backup

Siehe §13.8. Export und Restore unterliegen dem Restore-Horizont (INV-RESTORE-HORIZON-1). Checkpoints im Datenverzeichnis sind interne Kopien (Spur D, ohne KeyTable) und eine eigene Löschschicht (§12.4); Schlüssel-Backups (Spur K) sind befristet (§12.17). Backups außerhalb von Contextra sind im Bericht als `ExternalBackups` ausgewiesen (§12.9, REQ-RST-5).

#### C-24.5 Tenant-Lebenszyklus (REQ-TEN-004)

Ein Tenant durchläuft die Zustände `Aktiv`, `Gesperrt` und `Gelöscht`.

- **Aktiv → Gesperrt** (Kategorie `Admin`): keine Schreibzugriffe; Lesen nur für Betreiber mit Protokollierung; Schlüssel bleiben erhalten. Die Rückkehr nach `Aktiv` ist nur bei unveränderten Schlüsseln zulässig.
- **Gesperrt → Gelöscht** (`erase_tenant`, Kategorie `Destructive`): Der Tenant wird im `ErasedSet` eingetragen, bevor die Saga beginnt (INV-BOOT-1). Jede Collection durchläuft die Saga nach §12.8.2 (Klassen L bis M). Je Scope entstehen die Siegel 1 bis 4.
- **Gelöscht** ist endgültig. Die Tenant-ID wird nie wiederverwendet. Schreibversuche liefern `TenantErased` (final).
