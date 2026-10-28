# Contextra – Gesamtspezifikation 3.0 (Baukasten-Fassung, Zielzustand v1.0)

```
Dokument:   Normative Gesamtspezifikation (Architektur, Verhalten, Module, Prüfverträge)
Fassung:    3.0 (Entwurf zur Freigabe), baut auf 2.4 auf und ersetzt deren Produktzuschnitt
Stand:      28.10.2026
Belegbasis: Repo-Klon, HEAD e2f7573f2 (Merge PR #5360); 35 Workspace-Mitglieder (33 Crates unter `crates/`, `benchmarks/contextra-bench`, `xtask`)
Ablage:     docs/SPEC.md
Ersetzt:    SPEC 2.4 (Produktzuschnitt, Archivierungsentscheidung, Kern-Definition); der
            Löschpfad (alt §12) bleibt inhaltlich erhalten und wird in §9 als Baustein neu gefasst
Zielgruppe: Menschen und LLM-Agenten, die Contextra bauen, prüfen oder in ein KMU-Produkt einbetten
```

> **Leitidee der Fassung 3.0.** Contextra ist **keine Komplettlösung** mehr, sondern eine **robuste, datenschutzkonforme und entkoppelte Rust-Bibliothek (Baukasten) als Fundament für lokale LLM-Lösungen im Mittelstand in Deutschland und Europa**. Jedes Modul MUSS **einzeln gebaut, auditiert, verifiziert und leistungsgemessen** werden können. Die gesetzlich begründete Löschung samt Löschkette ist ein **eigenständiger Baustein** dieses Baukastens, kein Gesamtversprechen des Systems.

> **Selbständigkeit.** Dieses Dokument ist die einzige Quelle der Wahrheit für Architektur, Modulverträge, Konstanten, Gates und Plan. Jedes `§x` meint einen Abschnitt dieses Dokuments. Fehlt ein Detail, gilt: **nicht raten**, Fail-Closed wählen (INV-FAILCLOSED-1) und den Punkt in §16 melden. Der Umsetzungsplan steht in `PLAN.md` und darf dieser Spezifikation nicht widersprechen.

---

## 0. Schnellstart für Menschen und LLM-Agenten

### 0.1 Was Contextra ist

Contextra ist ein **Baukasten aus Rust-Crates** für lokale (On-Premise, ohne Cloud-Zwang) Wissens-, Such- und LLM-Anwendungen. Er besteht aus fünf **Baustein-Familien** (§5): Speicher und Suche, Löschung und Nachweis, Inferenz und Embedding, Agenten und Gedächtnis, Schutz und Schnittstellen. Ein Integrator wählt **nur die Crates, die er braucht** (z. B. nur Hybrid-Suche + LSM-Store, oder nur MCP-Server + Inferenz + Agenten-Pipeline), und kann jedes gewählte Crate **für sich** prüfen.

Typische Einsatzfälle (Hypothesen, §16): Speisekarten- und Reservierungsassistent in der Gastronomie, lokaler Produktberater im Online-Shop, DSGVO-konforme Vor-Ort-Verarbeitung in Kommunen, Jobcentern und Agenturen, interne Wissensdatenbanken von KMU.

### 0.2 Die vierzehn Regeln für jede Änderung

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
|Repo-Klon, HEAD e2f7573f2, 28.10.2026: Workspace-`Cargo.toml`, alle Crate-Manifeste, `capabilities.toml`, Quelltexte (Stichproben und maschinelle Zählung)|gelesen / statisch gemessen (E1)|
|Statische Kennzahlen (§5.2): eigener Zähler, nicht ausgerichtet am Gate `loc-budget`; Zeilen = nicht leere Nicht-Kommentarzeilen, Testcode (`tests/`, `#[cfg(test)]`, `*_tests.rs`) getrennt|E1, nicht vergleichbar mit Altwerten|
|Abhängigkeitsgraph aus den Manifesten (Regex-Auswertung, `[dependencies]`/`[dev-dependencies]`)|E1|
|`SYSTEMATIC_CRATE_AUDIT.md` (Fassung 2.3, 28.10.2026)|Hypothese (E0), in §1.3 gegen Code geprüft|

**Nicht geprüft in dieser Fassung:** Kompilierung, Testsuite, Clippy, Miri, Fuzz, Benchmarks, CI-Stand. In der Prüfumgebung war keine Rust-Toolchain verfügbar. Jede Zeile „Testergebnis: Pass“ des Audit-Berichts ist daher **unbelegt**, bis WP 0.1 (PLAN.md) die Baseline auf einem CI-Runner erzeugt.

### 1.3 Abweichungen zwischen Audit-Bericht und Code (Befunde U-01 bis U-12)

|ID|Aussage im Audit-Bericht|Befund am Code (E1)|Folge|
|---|---|---|---|
|U-01|35 Crates „Stable / Produktionsreif“, Mehrzahl 9–10/10|`capabilities.toml` führt `crypto`, `db`, `engine`, `mcp`, `store`, `text`, `vector` als **`beta`**, `adapt`, `audit-export`, `avv-generator`, `infer-onnx`, `license` als **`experimental`**|Audit-Noten sind **ungültig**; maßgeblich ist `capabilities.toml` und §5.3|
|U-02|`text`: „Panic-freie Regex-Initialisierung (`LazyLock<Option<Regex>>`)“|`tokenizer.rs:30` enthält `panic!` hinter `#[allow(clippy::panic)]`; Initialisierung über `OnceLock`|Verstoß gegen Regel 3; WP 1.1|
|U-03|`vector`: „`VectorScanner` für Audits“|Der Typ `VectorScanner` existiert nicht; vorhanden ist `HnswScanner` (`vector/src/hnsw/scanner.rs`). Gemessen werden nur HNSW-Slots; Snapshot-Datei und Quantisierungscodes sind nicht belegt|Schicht S2 teilweise gemessen; WP 5.2|
|U-04|Löschpfad (alt §12) als umgesetzt vorausgesetzt|**Vorhanden (E1):** `LayerScanner`-/`Purger`-Port; `MemtableScanner`, `SstableScanner`, `WalScanner` (`store`); `HnswScanner` (`vector`); `TextScanner` (`text`); `DeletionProof` mit `SignatureVersion` bis **V5** und Feldern für Annahmen, Reste, Ausschlüsse (`crypto`, 49 Treffer); `AuditChain`, `RevocationLog`, `KmsProvider`. Die in der Vorfassung gemeldeten Stubs (`no_crypto_stubs`, `default_test_deletion_proof_key`) sind **nicht mehr vorhanden**. **Keine Treffer** für `ErasedSet`, `EffaceableKeySlot`, `DurableLog`, `RecipientLedger`, `CacheScanner`, `CheckpointScanner`, `KeyScanner`, Blob-/Lineage-Scanner, Löschsaga. `WitnessLayer` kennt noch die alten sieben Varianten plus `Custom`; der Doku-Kommentar in `deletion_proof.rs` nennt V3 als „aktuell“|Löschkette **teilweise** umgesetzt: Messung und Bericht ja, Schlüsselhierarchie, Sperre, Saga und fünf von zehn Schichten nein; Reife `experimental` (§9.10)|
|U-05|`contextra-core` „9/10, Kompatibilitätsfassade“|41 Verwendungen von `contextra_core` außerhalb des Crates; hängt von `mvcc` und `wire`|Fassade ist nicht trivial; WP 2.8|
|U-06|„Entkoppelt“, „vollständig entkoppelt“ (`agent`, `graph`, `cognition`)|`engine` → `graph` (hart, `collection/relate.rs`); `db` → `cognition`, `adapt` (hart); `mcp` → `router`, `privacy`, `crypto`, `rank` (hart); `cognition` → `engine`; Dev-Zyklus `graph` ↔ `store`; `ports` enthält `graph.rs`, `graph_index.rs`, `kv.rs`, `attention.rs`|Echte Entkopplung fehlt (§4.5); Phase 2|
|U-07|`mcp`: „kein HTTP/Axum“|Bestätigt für `mcp`. **Aber:** `infer-onnx` (Feature `onnx`) lädt Modell und Tokenizer per `reqwest` von `huggingface.co` herunter (`src/lib.rs`, ca. Z. 90–135); `infer-ollama` nutzt `reqwest` gegen konfigurierbare URL (Beispiele `http://localhost:11434`)|Verstoß gegen VETO-NET1/INV-NET-1: Download nur nach ausdrücklicher Freigabe, sonst Offline-Modell mit Prüfsumme; Ollama nur Loopback per Default; WP 2.7|
|U-08|„Drei Unsafe-Inseln, nur dort `unsafe`“|Bestätigt: `unsafe` nur in `simd` (310 Zeilen), `sys` (42), `wire` (47, überwiegend generiert). `xtask` zählt nur Muster-Strings. `wire` hat `deny(unsafe_code)` statt `forbid` und generierten Code mit `.unwrap()`|Inseln korrekt, aber `wire`-Generat braucht Panic-Prüfung; WP 1.4|
|U-09|Tokio, Features, Zero-Overhead|Workspace-`tokio` mit `features = ["full"]`; Fassade hat `default = ["embedded"]` (gut), aber `candle`, `compliance` als Features|`tokio full` entfernen; WP 2.9|
|U-10|„Alle 35 Crates kompilieren fehlerfrei“|Nicht belegt. `default-members` enthält 26 von 35 Mitgliedern; **nicht** darin: `infer-candle`, `sandbox`, `py`, `privacy`, `license`, `audit-export`, `avv-generator`, `bench`, `xtask`|Baseline nötig (WP 0.1); CI muss alle Mitglieder je einzeln bauen|
|U-11|`crypto`: „AEAD, Löschbericht, WAL-Integrität“ (Fachlichkeit Löschung)|`crypto/src` enthält zusätzlich `kv_cipher.rs`, `kv_segment/`, `kv_shredding.rs` und die Features `kv-encryption`, `cloud-egress-guard`; KV-Cache-Typen (`KvSegment`) werden in 17 `crypto`-, 18 `kvcache`-, 11 `infer-candle`-Dateien genutzt|Fachfremde KV-Cache-Logik im Löschkern; Verlagerung nach `kvcache` (hinter Port), `crypto` bleibt I/O- und domänenfrei; WP 2.5|
|U-12|`xtask` „10/10, Quality Gates“|`xtask` hat ~32.600 Code-Zeilen und ist damit **mehr als doppelt so groß wie `store`** (13.500); mindestens ein Teil der Prüfungen ist regex-basiert (z. B. `check_duplicate_core_primitives.rs`), daneben stehen Agenten-/Prozess-Prüfungen (`check_agents_*`, `jules_*`), die für die Bibliotheksqualität nicht zentral sind|Wartungsrisiko; Gate-Inventar, Meta-Tests und Rückbau nach Wirksamkeit; WP 3.6|

Zählung: 33 Crates unter `crates/`, dazu `contextra-bench` und `xtask` = 35 Mitglieder; der Audit-Bericht zählt `xtask` und `bench` als „Crates“ mit.

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
|Öffentliche Stellen (Kommunen, Jobcenter)|Vor-Ort-Verarbeitung, nachvollziehbahre Löschung, AVV-Unterlagen|Familie 1 + 2 + 5 (Profil „Compliance“)|
|Entwickler von Agenten|deterministischer, budgetierter, wiederanlauffähiger Agentenlauf|Familie 4 (Profil „Agent“)|

**Designpunkt (Annahme, per Benchmark zu belegen):** ≤ 10⁷ Vektoren, ≤ 10² Mandanten je Knoten, Single-Node, ein Prozess hält ein Datenverzeichnis. Kein Sharding, keine Replikation, kein Netz-Listener, keine Telemetrie nach außen.

### 2.2 Modulprinzipien M1–M8 (neu)

|ID|Prinzip|Prüfmechanismus|
|---|---|---|
|M1|**Isoliert baubar:** `cargo build -p <crate> --no-default-features` gelingt mit ausschließlich den **hart deklarierten** Abhängigkeiten|Gate `module-isolation`|
|M2|**Isoliert testbar:** Alle Tests des Crates laufen ohne andere Rand-Crates; Fremdverhalten kommt über Ports und `testkit`|Gate `module-isolation`|
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
6. **Aufwärts-Dev-Kanten** sind nur als deklarierte Ausnahme in `capabilities.toml` erlaubt; Ziel: keine.
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
|alle übrigen|≤ 3|≤ 3|–|

Das Gate `transitive-budget` bricht, wenn ein Crate sein Budget überschreitet; Budgets dürfen nur sinken (Ratchet).

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

Methode: nicht leere Nicht-Kommentarzeilen aus `*.rs`; „Test-Z.“ = `tests/`, `benches/`, `#[cfg(test)]`-Blöcke und `*_tests.rs`. `wire` enthält ~1.200 generierte Zeilen. Werte sind E1 und **nicht** mit dem Altgate `loc-budget` vergleichbar. Reife Ist = `capabilities.toml`; Reife Ziel = Zielwert für v1.0 nach §1.4 (`stable` verlangt E3). „Hart abh.“ = transitive harte Workspace-Abhängigkeiten.

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
|`store`|LSM, WAL v3 mit HMAC-Kette, Manifest, Kompaktion, Memtable/SSTable-Scanner|`beta`; 13.500 Code-Z., 28.000 Test-Z.; fremde Dev-Kanten (`graph`, `vector`, `sandbox`)|Crash-Matrix (`crash_kill_durability`), Fehlerpropagation von `fsync`, Dev-Kanten entfernen, Blob-/Key-Scanner (F2)|
|`checkpoint`|Checkpoints, Pin-/Orphan-Handling, Manifest-Verifikation|hängt von `contextra-core`|`core` entfernen; Restore-Guard und `CheckpointScanner` (F2)|
|`vector`|HNSW, SQ8, ACL-Bitset, `HnswScanner`; DiskANN/RaBitQ experimentell|`beta`; `crypto` nur im DiskANN-Pfad genutzt|`crypto` hinter Feature; Slot-Nullung belegt (INV-HNSW-SLOT-1); Recall-Benchmark vs. fremde Baseline|
|`text`|BM25/BM25F, Block-Max-WAND, Tokenizer mit deutscher Komposita-Zerlegung, `TextScanner`|`beta`; **`panic!` in `tokenizer.rs:30`**|Panic entfernen (WP 1.1); Recall-/Latenz-Benchmark; Fuzz des Tokenizers|
|`rank`|RRF-Fusion, Kalibrierung, Coherence-Bonus|Feature `dibud`; 2.972 Code-Z.|Aufteilung `fusion/` vs. Rest prüfen (Kernanteil minimal halten)|
|`engine`|Planner, Collections, Chunking, Retention, Bounded-Compute-Pool, SSI-Gate|`beta`; 9.372 Code-Z.; harte Kante auf `graph`|`graph` hinter Port; Saga nach `erasure` verlagern; Mandanten-Isolationstests|
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
|`infer-candle`|lokale GGUF-Inferenz (Q4_K_M/Q8_0), Embeddings, ChatML; Features `kv-bridge`, `kv-stage-b`, `cuda`|laut Capabilities `stable` ohne belegtes E3; 2.423 Code-Z.|Determinismustest (festes Seed), Mandanten-KV-Test (INV-KVTENANT-1), Abbruch-/Stream-Tests, Latenz- und Speicher-Benchmark je Modellgröße|
|`infer-onnx`|ONNX-Embeddings (`ort`), Reranking|Feature `onnx` zieht `reqwest`, `tokenizers`, `ndarray`; `experimental`|Modell-Download von `huggingface.co` **entfernen oder hinter Egress-Freigabe**; Prüfsummen der Modelldateien erzwingen (U-07)|
|`infer-ollama`|REST-Client für lokalen Ollama-Dienst|`reqwest`; behält lokalen PRNG mit `RING-LAYER-EXCEPTION`|Nur Loopback standardmäßig; TLS-/Host-Allowlist; `Rng`-Port statt Ausnahme|
|`kvcache`|quantisierter KV-Cache, Eviction, Zeroize beim Drop|`forbid(unsafe_code)` in 4 Dateien; Features `kv-encryption`, `content-addressed-kv-cache`|Löschkaskade (INV-DERIVED-1, `CacheScanner`); Zeroize-Test per Prozessspeichersonde|
|`adapt`|PID-Latenzregelung, Sherman-Morrison, Catoni, Bandit|`experimental`; 2.193 Code-Z.; Re-Exports|Nur eigenständig nutzbar halten; numerische Stabilitätstests; Nutzen per Benchmark belegen oder zurückbauen|

#### F4 – Agenten und Gedächtnis

|Crate|Verantwortung|Befund (E1)|Pflicht bis Zielreife|
|---|---|---|---|
|`graph`|CSR-Graph, PageRank/PPR, Hyperkanten, Sitzungsbaum|7.796 Code-Z., 14.600 Test-Z.; Zyklus mit `store` (Dev); Mandantenbezug prüfen|Mandanten-Isolation und Löschkaskade (INV-DERIVED-1, VETO-F10); `RelationIndex`-Port|
|`cognition`|Konsolidierung, Wartungsplaner, Sleep-Cycle|3.187 Code-Z.; hängt an `engine`-Typen|Auf Ports umstellen; Löschkaskade; Determinismus mit `ManualClock`|
|`router`|Kontextueller Bandit, Drift-Erkennung, Konformale Kalibrierung|Abhängigkeiten `privacy`, `wire`|Sicherer Fallback bei Drift testen; Provider-Ports; Reproduzierbarkeit mit `Rng`-Port|
|`agent`|Orchestrator, State-Graph, DLQ, Checkpointing, `TokenBudget`|transitive Last 20|Entkoppeln (WP 2.6); Crash-/Replay-Tests; Budget-Race-Tests; Berechtigungsmodell für Werkzeuge|

#### F5 – Schutz und Schnittstellen

|Crate|Verantwortung|Befund (E1)|Pflicht bis Zielreife|
|---|---|---|---|
|`privacy`|PII-Klassifikation (E-Mail, IBAN, API-Key …), Schwärzung, Egress-Gateway|`stable` laut Capabilities; deutsche Formate (IBAN, Steuer-ID, Telefon) als Prüfliste|Golden-Corpus deutscher PII; Falsch-Negativ-Rate messen und ausweisen (keine Vollständigkeitszusage)|
|`mcp`|MCP-Server über stdio, Prompt-Injection-Guard, Exfiltrations-Detektor|`beta`; kein HTTP; transitive Last 22|Fuzz der JSON-RPC-Schicht; Werkzeug-Allowlist; Last verringern|
|`py`|PyO3-Bindings|`stable` laut Capabilities, 460 Test-Z. bei 1.415 Code-Z.|Zielreife `beta`; FFI-Panic-Grenze (xtask-Prüfung `check_ffi_panic_boundary`); Python-Tests|
|`sandbox`|Wasmtime-Sandbox, Fuel, Timeout|`stable` laut Capabilities, keine Abhängigkeiten|Escape-Tests, Fuel-Determinismus; Reife `beta`|
|`license`|Ed25519-Lizenzprüfung|`experimental`; **nicht** Teil des Pflichtbaus|Über `LicenseGate`-Port; Offline-Prüfung; keine Netzverbindung|

### 5.4 Entfallende Crates

`contextra-core` (106 Code-Z.) ist eine Kompatibilitätsfassade mit Re-Exports von `types`, `ports`, `mvcc`, `wire`; sie hat **41 Verwendungen** im Workspace. Sie entfällt in zwei Schritten: (1) als `deprecated` markieren, (2) Verwendungen auf direkte Crates umstellen, dann entfernen (ADR-D06, WP 2.8).

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
|D1|Löschtypen (`DeletionLayer` zehn Varianten, `LayerWitness`, `ReachedClass`)|`types`/`ports`/`crypto`|`WitnessLayer` mit sieben Alt-Varianten; `DeletionLayer` in `crypto` vorhanden|WP 5.1|
|D2|Schlüsselhierarchie: Keystore, KeyTable, `DocDek`, `ArtKey`, KEK-Epochen|`crypto` (Logik), `store` (Persistenz)|nicht gefunden (`KeyScanner`, `EffaceableKeySlot` fehlen)|WP 5.3|
|D3|Destroy-Protokoll D1–D6, Epoch Seal|`crypto` + `store`|nicht gefunden|WP 5.4|
|D4|Scanner Ebene A/B je Schicht|`store`, `vector`, `text`, `checkpoint`, `kvcache`, `graph`, `engine`|Memtable, SSTable, WAL, HNSW, Text vorhanden; Blob, Cache, Checkpoint, Key, Lineage fehlen|WP 5.2|
|D5|Purger je Schicht (`compact_range(scope)`, Slot-Nullung, Segment-Rewrite)|besitzende Crates|`Purger`-Port vorhanden; Implementierungen teilweise|WP 5.2|
|D6|`ErasedSet`, Lesesperre, Key-Lock|`erasure` + Lesepfade|nicht gefunden|WP 5.5|
|D7|Löschsaga (Zustandsmaschine, Journal, Recovery)|`erasure`|nicht gefunden|WP 5.5|
|D8|Bericht `DeletionProof` V5, Siegel, Claim-Generator|`crypto` (Format), `erasure` (Bildung)|V5-Format im Code; Klassen-/Siegelbildung unklar (E0)|WP 5.6|
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

**Lösch-Mutationskatalog (G7):** jede bewusste Fehlerinjektion (Scanner meldet 0, Purger übersprügt Segment, Schlüssel nicht vernichtet, Bericht ohne Messung) MUSS mindestens einen Test scheitern lassen.

### 9.9 Ehrliche Grenzen (im Bericht verpflichtend)

Nicht durch Contextra kontrollierbar und deshalb **ausgewiesen**, nie behauptet: externe Sicherungen des Betreibers, Datenträger-Firmware (A-FLASH, Wear-Leveling, Überprovisionierung), Cloud-/Anbieter-Replikation, Host-Prompt-Historien, Daten bereits an Dritte übermittelt, Auslagerung (Swap) und Core-Dumps außerhalb des gehärteten Prozesses. Das Ergebnis ist ein **technischer Befund über das lokale Datenverzeichnis**, kein Rechtsnachweis und keine Zertifizierung. Externe Zertifizierung (z. B. nach Medienlöschvorgaben) ist Aufgabe des Betreibers (O-04).

### 9.10 Reife der Löschkette

|Teil|Reife Ist|Reife Ziel v1.0|Voraussetzung|
|---|---|---|---|
|Scanner-Port und vorhandene Scanner|`beta` (E1–E2)|`stable`|Differentialtests, Chaos|
|`DeletionProof` V5 (Format, Verifikation)|`beta`|`stable`|Golden, Fuzz, externer Krypto-Review|
|Schlüsselhierarchie, Destroy, Seal|`experimental` (nicht umgesetzt)|`beta` (v1.0), `stable` danach|WP 5.3–5.4|
|Saga, `ErasedSet`, Lesesperre|`experimental` (nicht umgesetzt)|`beta`|WP 5.5|
|Backup/Restore-Guard|`experimental`|`beta`|WP 5.7|
|Gesamtbaustein „Löschkette“|**`experimental`**|`beta` mit Evidenz E3 je Teil|Phase 5 des PLAN.md|

Bis Phase 5 abgeschlossen ist, DARF kein Produkt, README oder Bericht die Löschkette als „produktionsreif“ oder „rechtssicher“ bezeichnen (INV-CLAIMS-1).

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
|§6 Nebenläufigkeit, §7 Persistenz, §8 MVCC|§8 (Rahmen) + Anhang C (C-6, C-7, C-8)|
|§9 Vektor, §10 Text, §11 Engine/Abfragepfad|§5.3 (Steckbriefe) + Anhang C (C-9 bis C-11)|
|§12 Löschung|§9 (Baustein) + Anhang C (C-12)|
|§13 Schnittstellen, §14 Konfiguration, §15 Startwerte|§10.6, §10.7, §11, §13 + Anhang C (C-13, C-15)|
|§16/§17 Benchmarks, QS|§7, §12|
|§18 Governance|§12, §13, §15|
|§19–§26 Kataloge|ersetzt durch §1.3, §15, §16; REQ-Katalog wird in WP 0.3 nach `docs/spec/requirements.toml` überführt|

## Anhang B. Änderungsverlauf

|Fassung|Datum|Änderung|
|---|---|---|
|2.4|27.10.2026|Löschpfad optimiert, Quellenbereinigung|
|**3.0**|**28.10.2026**|Produktzuschnitt auf entkoppelten LLM-Baukasten; Modulvertrag und isolierte Prüfbarkeit; Befunde U-01–U-12 aus dem Abgleich Audit ↔ Code; ADR-D01–D10; Löschkette als Baustein (§9); Aufhebung der Archivierungsentscheidung|

## Anhang C. Übernommener Detailtext aus 2.4 (nachrangig)

Die folgenden Abschnitte sind **unverändert aus Fassung 2.4** übernommen (Persistenz, MVCC, Löschpfad). Ihre `§`-Verweise beziehen sich auf die **Nummerierung von 2.4** (hier „C-§“). **Bei Widerspruch gelten §1 bis §16 dieses Dokuments.** Insbesondere gelten die Aussagen zu „Archiv“, „`FeatureDisabled` für Graph und KV-Cache“, „Kern ist geschlossen“ und zu den Crate-Budgets **nicht mehr** (ADR-D01, ADR-D02). Nach WP 0.3 werden diese Texte in `MODULE.md` und Fachkapitel der jeweiligen Crates überführt und hier gestrichen.
