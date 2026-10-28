# Contextra – Umsetzungsplan 3.0 (IST → SOLL, Baukasten-Fassung)

```
Dokument:   Umsetzungsplan zur SPEC 3.0 (docs/SPEC.md)
Fassung:    3.0 (Entwurf zur Freigabe)
Stand:      28.10.2026
Belegbasis: Repo-Klon HEAD 52d3227f0 (flach, 28.10.2026 22:23); statische Prüfung (SPEC §1.2);
            KEIN Build-, Test- oder Clippy-Lauf (Toolchain 1.89.0 in der Prüfumgebung nicht ladbar)
Ersetzt:    PLAN 2.5 (Phasenplan „Kern schließen, Rest archivieren“, 1.120 Zeilen, Phasen 0–9)
Ziel:       robuste, datenschutzkonforme, entkoppelte Rust-Bibliothek als Fundament für lokale
            LLM-Lösungen im Mittelstand in Deutschland und Europa; jedes Modul einzeln
            auditierbar, verifizierbar und leistungsmessbar
```

> **Rangfolge.** Bei Widerspruch gilt die SPEC. Jedes Arbeitspaket (WP) nennt sein SPEC-Ziel, seinen Schnitt, seine Abnahme (Gate oder Test) und seine Abhängigkeiten. **Ein PR = ein WP = höchstens ein Crate-Schnitt**; nach jedem Schnitt laufen Build, `module-audit` des betroffenen Crates und die Zeilenzählung.

> **Aufwandsangaben** (S ≈ ≤ 2 Tage, M ≈ ≤ 1 Woche, L ≈ ≤ 3 Wochen, XL > 3 Wochen, je eine Entwicklerin/ein Entwickler mit LLM-Unterstützung) sind **Schätzungen ohne Messung** und dienen nur der Reihenfolge.

---

## 0. Ausgangslage in einer Seite

Der Code ist **weiter und zugleich weniger belegt**, als der Audit-Bericht vom 28.10.2026 behauptet. Er hat (E1) eine saubere Ringrichtung ohne Ringverletzung, klare Unsafe-Inseln, ein umfangreiches Gate-Gerüst (58 Gates) und ein vorhandenes Scanner-/Bericht-Fundament für die Löschkette. Er hat **nicht**: einen belegten Build-/Testlauf, wirkliche Entkopplung (hohe transitive Last bei `mcp`, `agent`, `py`, `db`, `engine`, `cognition`), isolierte Modulprüfung, ein Netz-Standard „aus“ (ONNX lädt aus dem Netz) und den größeren Teil der spezifizierten Löschkette (Schlüsselhierarchie, Sperre, Saga, fünf Schichten).

### 0.1 Befunde → Arbeitspakete

|Befund (SPEC §1.3)|Kurz|Arbeitspaket|Phase|
|---|---|---|---|
|U-01|Reifegrade im Audit ≠ `capabilities.toml`|0.2|0|
|U-02|`panic!` in `text/tokenizer.rs:30`|1.1|1|
|U-03|`VectorScanner` fehlt, `HnswScanner` teilweise|5.2|5|
|U-04|Löschkette teilweise umgesetzt|5.1 – 5.9|5|
|U-05|`contextra-core` mit 61 Vorkommen in 32 Dateien|2.8|2|
|U-06|Scheinbare Entkopplung (`engine`→`graph`, `db`→`cognition`, `mcp`→`router`, Dev-Zyklus)|2.1 – 2.6, 2.10, 2.11|2|
|U-07|ONNX-Download, Ollama-HTTP|2.7|2|
|U-08|`wire`-Generat mit `unwrap`, `deny` statt `forbid`|1.4|1|
|U-09|`tokio full`, tote Features|2.9|2|
|U-10|Kompilierung nicht belegt, 9 Mitglieder nicht in `default-members`|0.1|0|
|U-11|KV-Kryptografie in `crypto`|2.5|2|
|U-12|`xtask` 32.572 Zeilen, Wirksamkeit unbekannt|3.6|3|
|U-13|Dev-Kanten brechen isolierte Testbarkeit (`router`, `cognition`, `infer-ollama`, `rank`, `checkpoint`, `agent`, …)|2.10, 2.11|2|
|U-14|Baseline im Repo (`evidence/baseline-a547a42/`) ist rot und stammt von altem Commit|0.1|0|

### 0.2 Leitplanken des Plans

1. **Wahrheit zuerst.** Phase 0 erzeugt eine belegte Baseline, bevor etwas umgebaut wird.
2. **Sofortschäden vor Architektur.** Produktions-Panics und Netz-Download sind klein und werden zuerst beseitigt.
3. **Entkopplung vor Löschkette.** Die Löschkette braucht saubere Ports und das neue Crate `erasure`; sie wird erst nach Phase 2 und 3 gebaut.
4. **Messbar je Schritt.** Jedes WP hat ein Gate oder einen Test als Abnahme; „fertig“ heißt Gate grün auf CI, nicht „Code geschrieben“.
5. **Nichts löschen, was nicht ersetzt ist.** Bestehende Tests wandern mit dem Code; ein Test wird nur mit Begründung entfernt (Gate `test-integrity`).
6. **Rückbau ist erlaubt.** Fähigkeiten ohne Benchmark-Beleg oder Verdrahtung (EP3, EP12) werden zurückgebaut oder auf `experimental` gesetzt.

---

## 1. Phasenübersicht

|Phase|Ziel|WPs|Abnahme (Kern)|Voraussetzung|Aufwand|
|---|---|---|---|---|---|
|**0 Baseline und Wahrheit**|belegter Ist-Stand; ehrliche Reife|0.1 – 0.4|Baseline-Protokoll E2; `claims-consistency` grün|–|M|
|**1 Sofortschäden**|keine Produktions-Panics; Allow-Register|1.1 – 1.4|`panic-inventory` Produktion = 0 (`text`, `wire`)|0.1|S–M|
|**2 Entkopplung**|Soll-Graph (SPEC §4.4/§4.5), keine Zyklen, Netz „aus“|2.1 – 2.11|`transitive-budget`, `no-cycles`, `egress-inventory` grün|0.1, 1.x|XL|
|**3 Modul-Infrastruktur**|isolierte Prüfung, Evidence-Pack, Benchmarks je Crate|3.1 – 3.7|`module-audit <crate>` läuft für alle Crates|2.1|L–XL|
|**4 Persistenz-Fundament**|`DurableLog`, Format-Register, Crash-Matrix je Modul|4.1 – 4.4|Crash-Matrix grün; `DurableLog` implementiert|3.2|L|
|**5 Löschkette**|Baustein nach SPEC §9 bis `beta`|5.1 – 5.9|AC-Matrix, Mutationskatalog, Chaos|2.4, 3.x, 4.x|XL|
|**6 Modul-Härtung F1–F5**|Zielreife je Modul (Fuzz, Differential, Bench)|6.1 – 6.5|Evidence-Pack E3 je Modul|3.x|XL|
|**7 Profile, Beispiele, Doku**|Search, Local-RAG, Agent, Compliance lauffähig und geprüft|7.1 – 7.5|`profile-audit` grün|6.x (teilweise)|L|
|**8 Abnahme und Release**|Release-Kriterien SPEC §13.4|8.1 – 8.3|alle Kriterien grün; externer Review|alle|M–L|

**Kritischer Pfad:** 0.1 → 2.1 → 3.2 → 4.2 → 5.1 → 5.3 → 5.4 → 5.5 → 5.6 → 8.x. **Parallel möglich:** Phase 1 neben Phase 0; Phase 2 (unabhängige Entkopplungen 2.2/2.3/2.5/2.8/2.10 je Crate); Phase 6 je Modulfamilie; Phase 7 je Profil, sobald dessen Module `beta` erreichen.

---

## 2. Phase 0 – Baseline und Wahrheit

### WP 0.1 – Belegte Build-/Test-/Clippy-Baseline (U-10)

**Ziel:** Erstes **E2-Protokoll** für alle 35 Workspace-Mitglieder.

**Schnitt:** CI-Konfiguration und `evidence/baseline/` (kein Produktionscode).

**Aufgaben**

1. Toolchain 1.89.0 auf einem sauberen Runner; Lauf von `cargo build -p <c> --no-default-features` **je Mitglied einzeln** (auch die neun, die nicht in `default-members` stehen: `infer-candle`, `sandbox`, `py`, `privacy`, `license`, `audit-export`, `avv-generator`, `bench`, `xtask`).
2. `cargo test -p <c>`, `cargo clippy -p <c> --all-targets --locked`, `cargo deny check`, `cargo audit`, `cargo fmt --check`.
3. Ergebnis je Crate als `evidence/baseline/<crate>.json` (Befehl, Commit, Toolchain, Exit-Code, Dauer, Testanzahl, rote Tests, Warnungen).
4. Rote oder nicht kompilierende Crates erhalten ein Ticket und werden in `capabilities.toml` auf `experimental` gesetzt, bis sie grün sind.

5. Ausgangspunkt ist `evidence/baseline-a547a42/` (älterer Commit, dort rot: `contextra-engine` fand `contextra_graph` nicht). Dieses Protokoll gilt **nicht** für den heutigen HEAD; es wird als `historisch` markiert und durch `evidence/baseline/` ersetzt.

**Abnahme:** Protokoll vorhanden und von einem zweiten Runner reproduziert (E3-Grundlage). **Aufwand:** M.

### WP 0.2 – Reife und Berichte bereinigen (U-01)

**Ziel:** Keine Aussage in Repository oder Berichten, die `capabilities.toml` widerspricht.

**Aufgaben**

1. `SYSTEMATIC_CRATE_AUDIT.md`: Kopf „**Hypothese – nicht belegt**“, Verweis auf SPEC §1.3; Noten (x/10) und Formulierungen „Produktionsreif“ entfernen oder durch Verweis auf Evidence-Pack ersetzen.
2. `capabilities.toml` auf die Felder des Modulvertrags erweitern (SPEC §4.3): `family`, `class`, `may_depend_on`, `transitive_budget`, `provides`, `requires`, `egress`, `bench`, `non_goals`, `evidence`.
3. Reife nach Baseline (0.1) setzen: nur `stable`, wenn E3 und V1–V6 belegt; sonst `beta`/`experimental` (Erwartung: viele Crates fallen von `stable` auf `beta`).
4. Gate `claims-consistency` (neu): vergleicht Berichte, README, `MODULE.md` mit `capabilities.toml`.

**Abnahme:** Gate grün; keine Fundstelle von „produktionsreif“ ohne Evidence-Pack. **Aufwand:** S–M.

### WP 0.3 – SPEC 3.0 und PLAN 3.0 ablegen, Dokumente konsolidieren

**Aufgaben**

1. `docs/SPEC.md` und `docs/PLAN.md` durch die Fassungen 3.0 ersetzen; die Fassungen 2.4 unter `docs/spec/archive/` ablegen.
2. ADR-D01 bis ADR-D10 (SPEC §15) in `docs/decisions/` anlegen (heute genau eine Datei dort: `0001-gemessene-loeschung-scanner-vertrag-v1.md`; Nummerierung fortsetzen, Format der Datei 0001 übernehmen).
3. Anforderungskatalog (alt §19) nach `docs/spec/requirements.toml` überführen, je Anforderung `module`, `test`, `status`.
4. Anhang C der SPEC schrittweise in `crates/<c>/MODULE.md` und Fachdokumente überführen, dann streichen.
5. Veraltete Dokumente (`docs/analysis`, `docs/reports`) mit Datum und „historisch“ kennzeichnen.

**Abnahme:** Gate `doc-truth` und `adr-index` grün. **Aufwand:** M.

### WP 0.4 – Reproduzierbarer Mess-Befehl

**Ziel:** Die Kennzahlen aus SPEC §5.2 (Code-/Testzeilen, `unsafe`, Panic-Muster, transitive Last, Features) sind **maschinell reproduzierbar**.

**Aufgaben:** `cargo xtask metrics --json` (aus `cargo metadata` und `syn`/AST statt Regex), Ausgabe nach `evidence/metrics.json`; Gate `metrics-ratchet` (nur besser werden).

**Abnahme:** Zwei Läufe auf demselben Commit liefern identische Werte. **Aufwand:** M.

---

## 3. Phase 1 – Sofortschäden

### WP 1.1 – `panic!` im Tokenizer (U-02)

**Schnitt:** `contextra-text`, `src/tokenizer.rs`.

**Aufgaben:** `get_protected_regex()` darf nicht panicken. Muster einmalig beim Start kompilieren und **Fehler als `ContextraError`** an den Konstruktor des Tokenizers melden (Fail-Closed) oder auf eine **konstante, bewiesen gültige** Regex-Konstruktion über `regex::Regex::new` in einem Test absichern und im Fehlerfall den Tokenizer ohne geschützte Muster betreiben (dokumentierter Degradationspfad). Entscheidung vor dem WP (E-1.1, §13). `#[allow(clippy::panic)]` entfernen.

**Abnahme:** `panic-inventory` zeigt für `text` Produktion = 0; Test „ungültiges Muster erzeugt Fehler, keinen Abbruch“; Fuzz-Smoke des Tokenizers grün. **Aufwand:** S.

### WP 1.2 – Allow-Register (EP16)

**Ausgangszahl (E1, zeilenbasierte Zählung, methodenabhängig):** 136 bis 158 Allow-Direktiven auf `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic` außerhalb von `tests/`-Verzeichnissen (zwei Zählmuster am selben Stand ergeben 136 bzw. 158; verbindlich ist erst der AST-Wert aus WP 0.4); Schwerpunkte `types` 27, `mcp` 18, `infer-ollama` 18, `mvcc` 14, `router` 13, `checkpoint` 12, `privacy` 12, `crypto` 10. Ein Teil steht in `#[cfg(test)]`-Modulen.

**Aufgaben**

1. Alle Allows in Produktionscode von Testcode trennen (AST-basiert, WP 0.4).
2. Für jeden Produktions-Allow: entfernen (Fehler propagieren) **oder** in `governance/allow-register.toml` mit Begründung, Invariante und Frist eintragen.
3. Gate `allow-register` (neu).

**Abnahme:** Gate grün; Produktions-Allows ≤ 20 und alle befristet. **Aufwand:** M.

### WP 1.3 – Produktions- und Testcode getrennt zählen

**Aufgaben:** `panic-inventory` trennt `src/` (Produktion), `#[cfg(test)]`, `tests/`, `benches/`, Generat. Bericht in `evidence/<crate>/panic.json`.

**Abnahme:** Inventar je Crate vorhanden; Ratchet nur sinkend. **Aufwand:** S.

### WP 1.4 – `wire`: Generat isolieren (U-08)

**Aufgaben:** `contextra_generated.rs` (FlatBuffers) in ein eigenes Modul mit gekapseltem `allow` für das Generat; handgeschriebener Code in `wire` unter `forbid`. `flatc`-Version pinnen; Panic-Muster im Generat gegen den **Verifier** absichern (jede Eingabe wird vor Zugriff verifiziert); Fuzz-Ziel für den Verifier.

**Abnahme:** `unsafe-inventory` weist Generat und Handcode getrennt aus; Fuzz-Lauf ohne Absturz; `check_flatbuffers_drift` grün. **Aufwand:** M.

---

## 4. Phase 2 – Entkopplung

**Ziel der Phase:** Der Ist-Graph (SPEC §4.5) erreicht die Soll-Budgets. **Methode je WP:** (1) Port definieren, (2) Referenzimplementierung hinter dem Port, (3) harte Kante entfernen, (4) Test-Double in `testkit`, (5) `transitive-budget` senken, (6) Konformitätstest des Ports.

### WP 2.1 – Kanten deklarieren, Zyklen verbieten

**Schnitt:** `capabilities.toml`, `xtask`.

**Aufgaben:** `may_depend_on` aus dem Ist-Graph erzeugen und gegen SPEC §4.4 abgleichen; Abweichungen als **geplante Ausnahmen mit Ablauf-WP** eintragen; Gates `may-depend-on`, `no-cycles`, `transitive-budget` (mit Ist-Werten als Startbudget, nur sinkend). Dev-Zyklus `graph` ↔ `store` als Ausnahme bis WP 2.10.

**Abnahme:** Gates grün mit Ist-Budgets; jede Ausnahme hat ein Ablaufdatum. **Aufwand:** M.

### WP 2.2 – `engine` ↔ `graph` über Port `RelationIndex` (U-06)

**Ist:** `engine/src/collection/relate.rs` nutzt `contextra_graph::hyperedge::{HyperEdgeId, RoleId, RoleBinding, HyperEdge}` direkt.

**Aufgaben:** Port `RelationIndex` im besitzenden Crate (`graph`) bzw. in `ports` (Entscheidung E-2.2); `engine` fordert den Port, die Fassade verdrahtet; Funktion `relate` wird optional (nicht vorhanden, wenn kein Port). `engine` verliert die Abhängigkeit `contextra-graph`.

**Abnahme:** `cargo tree -p contextra-engine` enthält `contextra-graph` nicht; `transitive-budget(engine)` ≤ 14, Ziel 11; Relate-Tests laufen mit `graph` in der Fassade. **Aufwand:** M.

### WP 2.3 – `db` ohne `cognition` und `adapt` (U-06)

**Ist:** `db` re-exportiert `contextra_cognition` und `execute_sleep_cycle`, nutzt `adapt::PidController` und Homeostat-Typen.

**Aufgaben:** Re-Exports in die Fassade verlegen; `homeostat.rs`/`multistep.rs` nach `adapt` oder `engine` verschieben, soweit sie Kernlogik sind, sonst hinter Feature; `db` hängt nur von `types`, `ports`, `engine`.

**Abnahme:** `transitive-budget(db)` ≤ 12; Fassaden-API unverändert nutzbar (Kompatibilitäts-Tests in `contextra`). **Aufwand:** M.

### WP 2.4 – Domänen-Ports in besitzende Crates (ADR-D05)

**Ist:** `ports` enthält `graph.rs`, `graph_index.rs`, `kv.rs`, `kv_bridge_port.rs`, `attention.rs`. Nutzer (Dateizahl): `graph` 31, `kvcache` 18, `crypto` 17, `infer-candle` 11, `engine` 8, `db` 5, `store` 4.

**Aufgaben:** Nutzungsanalyse; Typen, die nur Graph bzw. KV betreffen, nach `graph` bzw. `kvcache` verlagern; echte Querschnitts-Ports (Embedding, Vector, Text, Storage) bleiben. `ports` wird kleiner und stabil. SemVer-Hinweis für Abhängige.

**Abnahme:** `ports` enthält keine graph-/kv-spezifischen Typen; `public-api`-Diff geprüft; Budget `ports` ≤ 1 hart. **Aufwand:** L.

### WP 2.5 – KV-Kryptografie aus `crypto` (U-11)

**Aufgaben:** `kv_cipher.rs`, `kv_segment/`, `kv_shredding.rs` und Features `kv-encryption`, `cloud-egress-guard` nach `kvcache` (bzw. `privacy` für den Egress-Guard) verlagern; `crypto` bleibt I/O-, Lock- und domänenfrei. Mandanten- und Modellbindung (INV-KVTENANT-1) dort testen.

**Abnahme:** `crypto` ohne `kv_*`; Tests migriert; Zeroize-Test per Prozessspeichersonde grün. **Aufwand:** M.

### WP 2.6 – `cognition` und `agent` über Ports (U-06)

**Ist:** `cognition` nutzt `engine::collection::{Collection, StoredDocument, …}` und `AdaptiveDecayController`; `agent` zieht über `db` → `engine` → alle Kerne (transitiv 20).

**Aufgaben:** Ports `DocumentStore`, `VectorIndex`, `RelationIndex`, `CheckpointPort` nutzen; `AdaptiveDecayController` nach `cognition`/`adapt` verlegen; `agent` verliert `db`, `store`; Tests laufen mit `testkit`-Doubles.

**Abnahme:** `transitive-budget(cognition)` ≤ 6, `agent` ≤ 10; beide Crates bestehen `module-audit` ohne `engine`/`store` im Graph. **Aufwand:** L.

### WP 2.7 – Netz „aus“: ONNX, Ollama, `mcp`, `py` (U-07)

**Aufgaben**

1. `infer-onnx`: automatischen Modell-Download (`huggingface.co`) **entfernen** oder hinter Feature `model-download` plus SHA-256-Prüfsumme stellen; Standard: Modelle müssen lokal vorliegen.
2. `infer-ollama`: Standard **nur Loopback**; weitere Hosts nur per Allowlist; Hinweis: die in der Vorfassung gemeldete `RING-LAYER-EXCEPTION` (eigener PRNG) ist am HEAD 52d3227 **nicht mehr auffindbar**; das vorhandene Gate `determinism` bestätigt, dass `infer-ollama` weder `SystemTime`/`Instant` noch eigene Zufallsquellen nutzt, andernfalls `Rng`-/`Clock`-Port.
3. `egress-inventory`-Gate (statisch) und Test T-EGR-1 (Netzwerk-Namespace ohne Route).
4. `mcp`: `router`, `privacy`, `crypto`, `rank` nur optional über Fassaden-Re-Export; `py`: Abhängigkeiten auf `types` und Fassade reduzieren.

**Abnahme:** Egress-Tabelle (SPEC §10.6) wird vom Gate bestätigt; T-EGR-1 grün; `transitive-budget(mcp)` ≤ 16, `py` ≤ 12. **Aufwand:** L.

### WP 2.8 – `contextra-core` entfernen (U-05, ADR-D06)

**Aufgaben:** 61 Vorkommen (32 Dateien; zeilenbasierte Zählung, Wert aus WP 0.4 maßgeblich) auf `types`/`ports`/`mvcc`/`wire` umstellen (`checkpoint`, `py`, `bench`, Dev-Kanten von `engine`); `deprecated` markieren; ein Release warten; Crate entfernen und aus `Cargo.toml` streichen.

**Abnahme:** keine Verwendung mehr; Workspace baut ohne `core`. **Aufwand:** M.

### WP 2.9 – Features und `tokio` (U-09, ADR-D10)

**Aufgaben:** Workspace-`tokio` ohne `full`; je Crate benötigte Features; tote und doppelte Features löschen; `unexpected_cfgs = "deny"` (auch in `store`); `feature-matrix` je Modul.

**Abnahme:** `feature-matrix` grün; Bauzeit-Messung vor/nachher im Protokoll. **Aufwand:** M.

### WP 2.10 – Letzte Kanten: `vector`→`crypto`, `graph`↔`store`

**Aufgaben:** `vector`: `contextra-crypto` nur hinter `experimental-diskann` (nur dort genutzt); `graph`: Dev-Kante auf `store` entfernen (Tests mit `testkit` bzw. Port-Double); `store`: Dev-Kanten auf `graph`, `vector`, `sandbox` verlagern.

**Abnahme:** `no-cycles` ohne Ausnahme; `transitive-budget(vector)` ≤ 5 hart. **Aufwand:** M.
### WP 2.11 – Aufwärts- und Quer-Dev-Kanten (neu, aus der Nachmessung am HEAD 52d3227)

**Ist (E1, Manifeste):** Neben `graph` ↔ `store` bestehen weitere Dev-Kanten, die Isolation (M2) verletzen, weil Tests eines Crates ein anderes Rand- oder Kern-Crate bauen müssen:

|Crate|Dev-Kanten (Ist)|Soll|
|---|---|---|
|`router`|`db`, `graph`, `text`, `vector`|nur `testkit`; Provider-Doubles über Ports|
|`cognition`|`infer-candle`|`EmbeddingProvider`-Double aus `testkit`|
|`infer-ollama`|`infer-onnx`|entfernen (Reranker-Double)|
|`rank`|`adapt`|entfernen oder Feature `dibud`|
|`checkpoint`|`store`|`StorageEngine`-Double|
|`infer-candle`, `privacy`|`crypto`|`KmsProvider`-/Zeroize-Double|
|`agent`|`text`, `vector`|`VectorIndex`-/`TextIndex`-Doubles|
|`db`, `engine`|`vector`, (`engine`: `core`)|zulässig bis WP 2.8; `vector` bleibt als Dev-Kante erlaubt (gleicher Ring)|

Selbst-Dev-Kanten (`store`, `crypto` auf sich selbst, z. B. zur Feature-Aktivierung in Tests) sind **keine Zyklen** und werden vom Gate `no-cycles` ausgenommen, aber im Evidence-Pack ausgewiesen.

**Aufgaben:** je Zeile Test-Double in `testkit` bereitstellen, Tests umstellen, Dev-Kante entfernen; Ausnahmen mit Ablauf-WP in `capabilities.toml`.

**Abnahme:** Gate `module-isolation` (WP 3.2) baut und testet jedes der genannten Crates ohne die entfernten Dev-Abhängigkeiten. **Aufwand:** M–L.

---

## 5. Phase 3 – Modul-Infrastruktur (isoliert prüfen, belegen, messen)

**Ziel der Phase:** Für **jedes** Crate gilt `cargo xtask module-audit <crate>` (SPEC §7.1), erzeugt ein Evidence-Pack (SPEC §7.2) und bestimmt daraus die Reife. Phase 3 liefert **Werkzeug und Verträge**, noch keine Fachhärtung (das ist Phase 6).

### WP 3.1 – Modulvertrag und `MODULE.md`

**Schnitt:** `capabilities.toml` (Schema 3), `crates/<c>/MODULE.md` (33 Dateien; heute 0 vorhanden), Vorlage `docs/templates/MODULE.md`.

**Aufgaben**

1. Schema 3 der `capabilities.toml` mit den Pflichtfeldern aus SPEC §4.3; ein Schema-Test (`xtask`) lehnt fehlende Felder ab.
2. `MODULE.md` aus Manifest und Quelltext **generieren** (Zweck, Ports, Invarianten, Annahmen, Nicht-Ziele, Prüfkommando, Benchmarks); handgeschrieben sind nur Zweck, Annahmen und Nicht-Ziele; der Rest wird von `cargo xtask gen-module-docs` erzeugt und im Gate `doc-truth` gegen den Stand geprüft.
3. Länge ≤ 120 Zeilen (Gate-Prüfung).

**Abnahme:** 33 `MODULE.md` vorhanden, generierte Teile aktuell, `claims-consistency` grün. **Aufwand:** M.

### WP 3.2 – `cargo xtask module-audit <crate>` (zentral)

**Schnitt:** `xtask` (neues Untermodul `module_audit/`), kein Produktionscode.

**Aufgaben:** die neun Schritte aus SPEC §7.1 (Graph, Build, Lint, Inventare, Tests, Tiefe, Egress, Bench, Pack) als **ein** Befehl mit den Profilen `quick` (Schritte 1–5, < 5 Minuten je Crate), `full` (alle Schritte), `release` (`full` plus Mutationen und Fuzz-Laufzeit). Neue Gates: `module-isolation`, `egress-inventory`, `may-depend-on` (aus WP 2.1 übernommen). Schritt 2 baut **jede zulässige Feature-Kombination** mit `--no-default-features` und prüft zusätzlich, dass das Crate **ohne** seine nicht deklarierten Workspace-Nachbarn kompiliert (Build in einem Workspace-Ausschnitt per `cargo metadata`-Filter und `--package`-Isolation, nicht über den Gesamt-Workspace). Das Ergebnis ist maschinenlesbar (`evidence/<c>/manifest.json`) und der Exit-Code ist ungleich null, sobald ein Pflichtschritt scheitert.

**Selbsttest (Pflicht):** Ein Meta-Test erzeugt absichtlich ein Crate mit (a) nicht deklarierter Kante, (b) `panic!` in Produktion, (c) `reqwest`-Abhängigkeit ohne Egress-Eintrag, (d) Zyklus; `module-audit` MUSS jeweils scheitern (EP9, O-08).

**Abnahme:** `module-audit quick` läuft für alle 33 Crates; Meta-Test grün; Laufzeiten im Protokoll. **Aufwand:** L.

### WP 3.3 – Evidence-Pack, Reifeberechnung, API-Budget

**Aufgaben**

1. Generator für `evidence/<c>/{manifest,deps,api,unsafe,panic,tests,depth,bench,claims}.json` (SPEC §7.2) aus den Schritten von WP 3.2; Dateien enthalten Commit, Toolchain und Befehl, Wiederholung auf demselben Commit liefert identische Inhalte (außer Zeit- und Messfeldern, die als solche markiert sind).
2. **Reifeberechnung** `cargo xtask maturity`: leitet `experimental|beta|stable` aus dem Pack ab (SPEC §7.5) und vergleicht mit `capabilities.toml`; Abweichung = Fehler (Herabstufung automatisch vorgeschlagen, Heraufstufung nur per PR).
3. API-Budget (SPEC M5): `cargo public-api` je Crate; Rand-Crates ≤ 40, Fassaden ≤ 20 Elemente (Startwerte); Überschreitung nur mit ADR.
4. `cargo-semver-checks` für alle `[K]`-Crates (Gate `semver`).

**Abnahme:** Gates `evidence-pack`, `public-api`, `semver` grün; die Reifespalte in `capabilities.toml` ist vollständig aus dem Pack ableitbar. **Aufwand:** M–L.

### WP 3.4 – Port-Konformität und Test-Doubles (M7)

**Aufgaben**

1. Konformitätstest-Suiten je Port in `testkit::conformance::<port>` (`StorageEngine`, `VectorIndex`, `TextIndex`, `EmbeddingProvider`, `LlmTextGenerator`, `Reranker`, `KmsProvider`, `LayerScanner`, `Purger`, `Clock`, `Rng`, `IdGen`, `Vfs`, `EgressPolicy`).
2. Je Port ein Test-Double (`Fake*`, deterministisch, ohne Netz) in `testkit` (Feature `testkit`, nie im Release-Set).
3. Jede Referenzimplementierung und jedes Double besteht dieselbe Suite (Gate `ports-conformance`); Port-Fehlertypen sind `ContextraError`-Varianten (SPEC §6.3).

**Abnahme:** Gate grün; je Port mindestens eine Referenzimplementierung und ein Double; Ring-0-Rechenports sind synchron (Gate `ring0-async`). **Aufwand:** L.

### WP 3.5 – Benchmarks je Crate (M4)

**Ist (E1):** `benches/`-Verzeichnisse existieren bereits in `checkpoint`, `crypto`, `graph`, `infer-candle`, `infer-onnx`, `mvcc`, `router`, `simd`, `store`, `vector`; `benchmarks/` enthält Ende-zu-Ende-Läufe (`competitive_bench.rs`, `scale_bench.rs`, `rrf_scale_bench.rs`, `relate_bench.rs`, `migration_benchmarks.rs`) und `contextra-bench` (Vergleich mit Fremdsystemen).

**Aufgaben**

1. Benchmark-Matrix je Crate nach SPEC §7.4; fehlende Benchmarks (z. B. `text`, `rank`, `engine`, `kvcache`, `agent`, `privacy`, F2) ergänzen, vorhandene auf Fixture, Metrik und Budget bringen.
2. `crates/<c>/benches/budgets.toml` (p50/p99/p99.9 mit Konfidenzintervall, Hardwareklasse „KMU-Mini-PC“); Messregeln: Warm-up, ≥ 30 Wiederholungen, Commit und Toolchain im Ergebnis.
3. Fremde Baselines nur für Vergleichsaussagen (EP4): HNSW vs. Brute-Force und einer externen Referenz, BM25 vs. naive Implementierung.
4. Gate `module-bench` (nightly): bricht bei Regression über Budget; Ergebnisse nach `evidence/<c>/bench.json`.
5. Hardwareklasse festschreiben (O-07); **bis dahin keine absoluten Leistungsversprechen** in Dokumenten.

**Abnahme:** Jedes Modul mit Leistungsversprechen hat Benchmark-ID, Fixture, Budget; Nightly-Lauf grün. **Aufwand:** L.

### WP 3.6 – `xtask` inventarisieren und zurückbauen (U-12, ADR-D09)

**Ist (E1):** `evidence/xtask-inventory/` existiert bereits; `xtask/src` enthält 87 Einträge, darunter Prozess- und Agentenprüfungen (`check_agents_*`, `jules_*`, `session_history`, `claim`, `commit_health`), die für die Bibliotheksqualität nicht zentral sind, und regexbasierte Prüfungen (z. B. `check_duplicate_core_primitives`).

**Aufgaben**

1. Das vorhandene Inventar fortschreiben: je Gate/Prüfung **Zweck, Eingabe, Laufzeit, Fehlalarmquote, letzter wirksamer Treffer**; Wirksamkeit per Mutationstest (Meta-Test, SPEC O-08): Eine Prüfung ohne nachgewiesenen Treffer auf einer absichtlichen Verletzung gilt als **unwirksam**.
2. Drei Klassen: **Pflicht** (qualitätsrelevant, bleibt im Release-Set), **Prozess** (Agenten-/Commit-Workflows, aus dem Pflichtpfad entfernt, optional), **Rückbau** (unwirksam oder doppelt, entfernt).
3. Regexprüfungen, die Rust-Syntax beurteilen, auf AST (`syn`) umstellen (gemeinsam mit WP 0.4).
4. `xtask` bekommt ein eigenes Budget (Zeilen und Abhängigkeiten) und **darf kein `unsafe`** enthalten (SPEC §4.6).

**Abnahme:** Inventar vollständig; Rückbau-Liste umgesetzt; Gate-Laufzeit gesamt im Protokoll (Ziel: PR-Stufe < 15 Minuten, Startwert, nicht gemessen). **Aufwand:** L.

### WP 3.7 – Profil-Gerüst `profile-audit`

**Aufgaben:** `cargo xtask profile-audit <search|local-rag|agent|compliance>` baut das Profil, prüft transitive Last gegen SPEC §11, führt `egress-inventory` und einen Smoke-Benchmark aus; Fassaden-Features (`embedded`, `candle`, `onnx`, `ollama`, `agents`, `graph`, `cognition`, `router`, `privacy`, `mcp`, `sandbox`, `compliance`, `license`) nach SPEC §4.7 neu schneiden; `contextra-core` und `license` sind **nie** Pflicht.

**Abnahme:** Gerüst läuft für alle vier Profile (Beispiele folgen in Phase 7). **Aufwand:** M.

---

## 6. Phase 4 – Persistenz-Fundament

**Ziel:** `store`, `mvcc`, `checkpoint` sind **allein** (ohne `vector`, `text`, `engine`) prüfbar und tragen die Sidecar-Logs, auf die die Löschkette aufbaut (SPEC §8).

### WP 4.1 – Format-Register

**Aufgaben:** je persistente Struktur (WAL `MFW3`, SSTable `MFSX`/`MFST`, Blob `MFBL`, Manifest `MFMN`, Keystore `MFKS`, Checkpoint, Sidecar-Logs) Magic, Versionsfeld, Prüfsumme und **Golden-File** (`tests/golden/`); unbekannte Version ⇒ Fail-Closed mit benanntem Fehler; zentrale Tabelle `docs/spec/formats.toml` mit Schichtzuordnung (Grundlage für Gate `surface-coverage`, INV-SURFACE-1).

**Abnahme:** Golden-Tests grün; Gate `surface-coverage` kennt jede Dateiklasse. **Aufwand:** M.

### WP 4.2 – Port `DurableLog` und Sidecar-Journale

**Aufgaben:** Port `DurableLog` in `ports` (append, fsync-bestätigt, Replay, Truncation-Schutz durch High-Water-Mark); Implementierung in `store` (Löschjournal, Audit, `RevocationLog`); `crypto` nutzt den Port statt eigener Dateizugriffe (`crypto` bleibt I/O-frei, SPEC §4.4). Konformitätssuite (WP 3.4) und `FaultVfs`-Tests (Teilschreibung, `ENOSPC`, `EIO`).

**Abnahme:** `crypto` enthält keinen Dateizugriff mehr (Gate `ring0-async` und `may-depend-on` grün); Replay-Test nach hartem Abbruch grün. **Aufwand:** L. **Voraussetzung:** 3.2.

### WP 4.3 – Crash-Matrix und Fehlerpropagation

**Aufgaben:** `crash_kill_durability` auf alle Schreibpfade ausweiten (WAL-Commit, Flush, Kompaktion, Manifest-Ersetzung, Blob-Segment, Checkpoint); jede `fsync`-/`rename`-Stelle propagiert Fehler (Gate `result-dropped-on-io`); atomare Ersetzung nach SPEC §8.2 Regel 3 testen; WAL-Replay-Fuzz.

**Abnahme:** Crash-Matrix grün; keine verworfenen I/O-Fehler; Fuzz-Lauf ohne Fund. **Aufwand:** L.

### WP 4.4 – Orakel, SSI, Nebenläufigkeit

**Aufgaben:** Differentialtest `store` gegen `ModelStore` (unabhängiges Orakel, INV-ORACLE-1); loom-Modelle für Lock-Protokolle in `store` und für den SSI-Validator in `mvcc` (Write-Skew-Tests); Determinismustest der Recovery (G6, T-REC-2).

**Abnahme:** `loom`- und Orakel-Läufe grün; `mvcc` besteht `module-audit full` ohne Nachbarn. **Aufwand:** M–L.

---

## 7. Phase 5 – Löschkette (Baustein nach SPEC §9)

**Reihenfolge begründet:** Typen (5.1) → Messung (5.2) → Schlüssel (5.3, 5.4) → Orchestrierung (5.5) → Bericht (5.6) → Randbedingungen (5.7, 5.8) → Beweisführung (5.9). **Jedes WP besteht `module-audit` des betroffenen Crates**, bevor das nächste beginnt. Bis Phase 5 abgeschlossen ist, bleibt die Löschkette `experimental` (SPEC §9.10).

### WP 5.1 – Löschtypen vereinheitlichen (D1)

**Ist (E1):** `DeletionLayer` liegt in `crypto/src/deletion_proof.rs`, `WitnessLayer` in `ports/src/scanner/layer.rs` mit den alten sieben Varianten plus `Custom`; der Doku-Kommentar nennt V3 als aktuell, der Code erzeugt `SignatureVersion::V5`.

**Aufgaben:** **eine** Typfamilie (A2): `DeletionLayer` mit zehn Varianten (SPEC §9.4: `Lsm`, `BlobSegments`, `VectorIndex`, `TextIndex`, `CachesAndVault`, `AuditLineage`, `Checkpoints`, `KeyMaterial`, `Graph`, `KvCache`), `LayerWitness`, `ReachedClass`; `WitnessLayer` entfällt bzw. wird Alias mit Deprecation; Doku-Kommentare auf V5; Gate `deletion-types-once`.

**Abnahme:** genau eine Definition je Löschtyp; alle Scanner-Crates kompilieren gegen die neue Familie; Golden-Dateien V1–V5 verifizieren unverändert. **Aufwand:** M.

### WP 5.2 – Scanner und Purger je Schicht (D4, D5)

**Ist:** vorhanden `MemtableScanner`, `SstableScanner`, `WalScanner` (`store`), `HnswScanner` (`vector`), `TextScanner` (`text`); Ports `LayerScanner`, `Purger`.

**Aufgaben (je Schicht im besitzenden Crate)**

|Schicht|Besitzer|Neu|
|---|---|---|
|S1b Blob|`store`|`BlobScanner`, Segment-Rewrite-Purger|
|S2 Vektor|`vector`|`HnswScanner` erweitern um Snapshot-Datei und SQ8-Codes (Umbenennung/Alias `VectorScanner` nur, wenn alle drei Strukturen gemessen werden); Slot-Nullung belegt (INV-HNSW-SLOT-1)|
|S4 Caches/Vault|`store`, `engine`, `privacy`|`CacheScanner`, Invalidierung und Zeroize|
|S5 Lineage|`engine`, `crypto`|`LineageScanner`, Kaskade (INV-DERIVED-1)|
|S6 Checkpoints|`checkpoint`|`CheckpointScanner`, Policy `Rewrite`/`ExpireNow`/`ExpireAfterHold`|
|S7 Schlüssel|`crypto`, `store`|`KeyScanner` (Nadelscan; Keystore, Wrap-Blobs, Temp-/`.bak`)|
|S9 Graph|`graph`|`GraphScanner`, Entfernen und Neuaufbau|
|S10 KV|`kvcache`|`KvScanner`, Zeroize der Blöcke je Mandant/Quelle|

Jeder Scanner besteht: Positivkontrolle, Negativkontrolle, **Differenz gegen einen Brute-Force-Scan der Rohdateien** (SPEC §9.8). Eine Schicht, in der Daten liegen können und die keinen Scanner hat, liefert `NotMeasured` (nie stilles `Clean`).

**Abnahme:** je Schicht Scanner **und** Purger; Differenztest grün; Vorher/Nachher-Test mit `FaultVfs`-Abbrüchen. **Aufwand:** XL (teilbar je Schicht, ein PR je Schicht).

### WP 5.3 – Schlüsselhierarchie (D2)

**Aufgaben:** Keystore (`MFKS`), `KeyTable` mit Shards, `DocDek` je Betroffenen-Einheit, `ArtKey` je Artefakt, KEK-Epochen; Port `EffaceableKeySlot` (K0/K1 Software-Schlüssel; K2/K3 nur Port, SPEC §2.4); Record-Verschlüsselung unter dem `DocDek` im WAL-/LSM-/Blob-Pfad (`store`); Schlüsselmaterial nur `Zeroizing`; Constant-Time-Vergleiche; `KeyScanner` aus WP 5.2.

**Abnahme:** Konformitätstest `EffaceableKeySlot`; Zeroize-Test per Prozessspeichersonde (`testkit`, `/proc/self/mem`); kein Schlüsselbyte in Logs oder `Debug`. **Aufwand:** L. **Voraussetzung:** 4.2.

### WP 5.4 – Destroy-Protokoll D1–D6 und Epoch Seal (D3)

**Aufgaben:** Protokoll nach SPEC §9.6 (Intent vor Wirkung, Key-Lock, neue KeyTable-Generation ohne Eintrag per tmp → `fsync` → `rename` → Verzeichnis-`fsync`, Entfernen alter Generationen und Temp-/`.bak`-Kopien, `RevocationLog`, Journal-Done und Nadelscan); Epoch Seal (KEK-Wechsel) für SSD/Copy-on-Write; Idempotenz.

**Abnahme:** Crash-Matrix D1–D6 mit `FaultVfs`: nach jedem Wiederanlauf **kein lebender Schlüssel** der Einheit (INV-NO-RESURRECTION-1); Bericht trägt `key_destroyed` erst nach D6. **Aufwand:** L.

### WP 5.5 – Crate `erasure`: Saga, `ErasedSet`, Lesesperre (D6, D7)

**Schnitt:** neues Ring-3-Crate `contextra-erasure`, hängt **nur** von `types`, `ports` (und `crypto` für Berichtstypen); keine Abhängigkeit von `engine`, `db`, `store`.

**Aufgaben**

1. Zustandsmaschine nach SPEC §9.6 (`Requested → … → Completed`, `Restricted`, `Rejected`, `Partial{outstanding, reason}`), Journal über `DurableLog`, Recovery, Idempotenz, Notpfad (nur Klasse L).
2. `ErasedSet` durabel; Port `ErasedSetPort`/`ControlToken`; **Lesesperre ab `Frozen` in allen Lesepfaden** von `engine`/`db` (inkl. `as_of`, Export, Checkpoint-Erzeugung) über den Port – kein Lesepfad ohne Prüfung (Compilezeit-Typestate, wo möglich).
3. INV-DEL-ORDER-1: `KeyDestroyed` erst nach gemessener Klasse P aller Ebene-A-Schichten und durablem Pre-Destruction-Inventar.
4. Verdrahtung nur in der Fassade (EP7); bestehende Saga-Reste in `engine`, `mcp`, `crypto` hierhin verlagern.

**Abnahme:** Zustandsmaschinen-Modelltest, loom-Modell der Saga, Chaos mit `FaultVfs` an jeder Stelle (G3); `module-audit erasure` ohne `engine`/`store` im Graph. **Aufwand:** XL.

### WP 5.6 – Bericht V5, Klassenbildung, Claim-Generator (D8)

**Aufgaben:** Klassenbildung nur aus Witnesses (INV-CLASS-MEASURED-1); Pflichtangaben (`legal_state`, `backup_window`, `assumptions[]`, `residuals[]` mit Frist, `platform_guarantee`, `excluded[]`, SPEC §9.5); **Typestate:** Bericht ohne Witness je Schicht ist nicht kompilierbar; Claim-Generator erzeugt den Berichtstext nur aus gemessenen Feldern (keine freie Formulierung); Golden V1–V5; Fuzz des Verifiers; falscher Schlüssel ⇒ `Err`.

**Abnahme:** T-DEL-2/3 grün; Compile-fail-Test (`trybuild`) für den Bericht ohne Witness. **Aufwand:** M–L.

### WP 5.7 – Backup/Restore und Restore-Guard (D9)

**Aufgaben:** Zwei-Spuren-Backup (Spur D Daten mit Restore-Guard, Spur K Schlüssel getrennt); `ErasedSetMirror` an die Sicherungsspur; Restore stellt Gelöschtes nicht wieder her (G4, T-DEL-7/8/9); `checkpoint` greift nur über `CheckpointPort` zu (nach WP 2.8).

**Abnahme:** Restore-Tests nach Löschung grün; `CheckpointScanner` belegt Policy. **Aufwand:** L.

### WP 5.8 – Fristen, Empfänger, Legal Hold (D10)

**Aufgaben:** `RecipientLedger` (Art. 17 Abs. 2, Art. 19), Fristen mit Standard und Obergrenzen (kalibrierbar, Anhang C-12), Legal Hold und Mixed-Policy ⇒ `Restricted{basis, until}`; **Aufbewahrungsfristen und Legal-Hold-Katalog sind Rechtsfrage (O-02)**: Contextra liefert den Mechanismus, **keine** Standardwerte mit Rechtsanspruch.

**Abnahme:** Zustandsübergänge `Frozen → Restricted → Deleted` getestet; Bericht weist `BeschränktAufbewahrt` korrekt aus. **Aufwand:** M.

### WP 5.9 – Lösch-Mutationskatalog und Abnahme (D11, G7)

**Aufgaben:** Mutationskatalog (Scanner meldet 0, Purger überspringt Segment, Schlüssel nicht vernichtet, Bericht ohne Messung, Sperre fehlt in einem Lesepfad, Restore überschreibt `ErasedSet`) als automatisierte Injektionen; jede MUSS mindestens einen Test scheitern lassen; `/proc/self/mem`-Sonde (INV-HEAP-ZERO-1); AC-Matrix (Anforderung → Test) vollständig; Abgleich mit `mutants-guarantees.toml`.

**Abnahme:** Mutation-Wirksamkeit 100 % auf dem Katalog; Löschkette `beta` mit E3 je Teil (SPEC §9.10). **Aufwand:** L.

---

## 8. Phase 6 – Modul-Härtung F1–F5

Jedes WP erhebt die Crates der Familie auf **Zielreife** (SPEC §5.3, Spalte „Pflicht bis Zielreife“): Evidence-Pack E3, Verifikationsleiter V1–V6 (V7 für `[K]`), Benchmark gegen Budget. Reihenfolge innerhalb einer Phase nach transitiver Last (leichte Crates zuerst).

|WP|Familie|Crates|Kernaufgaben (zusätzlich zu Phase 2 bis 5)|
|---|---|---|---|
|6.1|F1 Speicher/Suche (Kern)|`mvcc`, `store`, `checkpoint`, `vector`, `text`, `rank`|Differentialtests (HNSW vs. Brute-Force; BM25 vs. naive Referenz); Tokenizer-Fuzz; Recall@10 und Latenz gegen fremde Baseline; SQ8-Recall; `rank`: Kernanteil `fusion/` bewerten|
|6.2|F1 Orchestrierung|`engine`, `db`|Mandanten-Isolationstests T-TEN-1/2 (ACL vor Distanzberechnung, IDF je Mandant); `remember`/`recall`-TTC-Benchmark; Planner-Property-Tests|
|6.3|F3 Inferenz|`infer-candle`, `infer-onnx`, `infer-ollama`, `kvcache`, `adapt`|Determinismus mit festem Seed; INV-KVTENANT-1; Abbruch/Streaming gibt Ressourcen frei; ONNX-Prüfsummenzwang; Ollama-Loopback; Latenz/Speicher je 3B/7B/14B; `adapt`: Nutzen belegen oder zurückbauen (EP12)|
|6.4|F4 Agenten|`graph`, `cognition`, `router`, `agent`|Mandanten-Isolation und Löschkaskade (S5, S9); `cognition` mit `ManualClock`, nie im Anfragepfad; Router-Fallback bei Drift; Agent: Crash-/Replay-Tests, Budget-Race, Werkzeug-Allowlist je Mandant, Prompt-Injection-Red-Team-Korpus|
|6.5|F5 Schutz/Schnittstellen|`privacy`, `mcp`, `py`, `sandbox`, `license`, `audit-export`, `avv-generator`|Golden-Korpus deutscher PII (IBAN, Steuer-ID, Telefon, Kfz-Kennzeichen) mit ausgewiesener Falsch-Negativ-/Falsch-Positiv-Rate (**keine Vollständigkeitszusage**); JSON-RPC-Fuzz; FFI-Panic-Grenze; Wasmtime-Escape-/Fuel-Tests; Lizenzprüfung offline; Formatverträge (JSON-Schema) und Golden-Files für `audit-export`|

**Abnahme je WP:** `module-audit full` grün für jedes Crate der Zeile; `capabilities.toml` setzt die Reife nur gemäß Pack (WP 3.3). Crates, die ihr Ziel nicht erreichen, bleiben `beta`/`experimental` und werden **nicht** in ein Profil aufgenommen, das `stable` verlangt. **Aufwand:** je XL, parallelisierbar je Familie.

---

## 9. Phase 7 – Profile, Beispiele, Dokumentation

|WP|Profil|Inhalt|Abnahme|
|---|---|---|---|
|7.1|**Search**|`examples/search/` (Speisekarte, Produktkatalog): Ingest, Hybridsuche, Mandantentrennung; Betriebsanleitung (deutsch)|`profile-audit search` grün; transitive Last ≤ 14|
|7.2|**Local-RAG**|Search + `infer-candle` oder `infer-ollama` + `privacy`; optional `mcp`; Beispiel „Produktberater“ und „Reservierungsassistent“; Modellbeschaffung und Prüfsumme dokumentiert|`profile-audit local-rag` grün; T-EGR-1 grün; ≤ 17|
|7.3|**Agent**|Local-RAG + `checkpoint`, `graph`, `router`, `agent`, `kvcache`, `adapt`; Beispiel Mehrschritt-Workflow mit Wiederanlauf|`profile-audit agent` grün (mindestens `beta`); ≤ 22|
|7.4|**Compliance**|Search + `checkpoint`, `erasure`, `audit-export`, `avv-generator`; Beispiel Löschung samt Bericht, Verzeichnis (Art. 30), AVV-Entwurf; deutsche Hinweise zu Grenzen (SPEC §9.9)|`profile-audit compliance` grün; ≤ 18|
|7.5|**Doku**|Deutsche Integrationsanleitung je Profil (Betrieb, Backup, Löschung, Grenzen, Update); SBOM je Crate; Modelllizenzen im SBOM-Anhang (Herkunft, Lizenz, SHA-256); `doctor`-Handbuch|`doc-truth` grün; jedes Beispiel läuft in CI|

**Aufwand:** L gesamt. **Voraussetzung:** Module des Profils mindestens `beta` (Compliance zusätzlich Phase 5).

---

## 10. Phase 8 – Abnahme und Release

### WP 8.1 – Release-Kriterien-Audit

Jede Zeile aus SPEC §13.4 wird mit Nachweisdatei, Commit und Gate-Lauf abgehakt (`evidence/release/v1.0/`). Fehlende Kriterien blockieren das Release; sie werden **nicht** umformuliert.

### WP 8.2 – Externer Review

Beauftragung eines unabhängigen Reviews für Krypto (`crypto`, Schlüsselhierarchie, Destroy-Protokoll) und Löschkette (V8, SPEC §7.3); Befunde als Tickets mit Frist; Ergebnis im Evidence-Pack. **Voraussetzung für `certified-ready`**, nicht für `stable`/`beta`.

### WP 8.3 – Versionierung und Veröffentlichung

SemVer je Crate (`cargo-semver-checks` für `[K]`), Änderungsprotokoll, reproduzierbarer Build (`reproducible_build`), SBOM, Lizenzbericht (`cargo deny`), Tag. Ein Release „Contextra 1.0“ setzt voraus, dass **jedes im Profil enthaltene Modul** seine Zielreife erreicht hat (SPEC §13.3); einzelne Module dürfen früher freigegeben werden.

**Aufwand Phase 8:** M–L (ohne Wartezeit des externen Reviews).

---

## 11. Gate-Plan: Gate → Phase, die es grün macht

|Gate|Status am HEAD|WP|
|---|---|---|
|`claims-consistency`|neu|0.2|
|`metrics-ratchet`|neu|0.4|
|`panic-inventory` (Produktion/Test getrennt)|vorhanden, anpassen|1.3|
|`allow-register`|neu|1.2|
|`may-depend-on`, `no-cycles`, `transitive-budget`|neu|2.1 (Budgets sinken bis 2.11)|
|`egress-inventory`|neu|2.7 / 3.2|
|`feature-matrix`|vorhanden, je Modul|2.9|
|`module-isolation`|neu|3.2|
|`evidence-pack`, `public-api`, `semver`|neu / vorhanden|3.3|
|`ports-conformance`|neu|3.4|
|`module-bench`|neu (nightly)|3.5|
|`profile-audit`|neu (merge)|3.7, 7.x|
|`surface-coverage`|neu|4.1|
|`result-dropped-on-io`|vorhanden|4.3|
|`deletion-types-once`, `deletion-honesty`|vorhanden, anpassen|5.1, 5.6|
|`unsafe-inventory`, `miri`, `fuzz-matrix`, `fuzz-regression`, `determinism`, `test-integrity`, `deny`, `audit`, `sbom`|vorhanden (58 Gates insgesamt, Wirksamkeit ungemessen)|Wirksamkeit in 3.6 prüfen|

---

## 12. Testmatrix: SPEC-Anforderung → Test → Arbeitspaket

|Test-ID|Prüft|Paket|
|---|---|---|
|T-ISO-1|jedes Crate baut/testet isoliert, inkl. Dev-Kanten (G9, INV-ISO-1)|3.2, 2.11|
|T-KV-1|KV-Cache je Mandant, Zeroize (INV-KVTENANT-1)|6.3|
|T-MOD-1|Berichte/README dürfen Reife nicht überhöhen (M8)|0.2|
|T-EGR-1|Laufzeit ohne Netzroute besteht (G8, INV-NET-1)|2.7|
|T-TEN-1/2|Mandantenisolation Lesen/Finden/Löschen (G5)|6.2, 6.4|
|T-REC-2|Recovery deterministisch (G6)|4.4|
|T-DEL-1|Byte-Scan Klasse P/M (G1)|5.2, 5.9|
|T-DEL-2/3|Bericht nur gemessene Klasse; falscher Schlüssel ⇒ `Err` (G2)|5.6|
|T-DEL-4/5|Absturz in der Saga ⇒ Fortsetzung oder `Partial`; kein lebender Schlüssel (G3)|5.4, 5.5|
|T-DEL-7/8/9|Lesesperre, Restore stellt nichts wieder her (G4)|5.5, 5.7|
|T-DEL-15|Lösch-Mutationskatalog (G7)|5.9|
|INV-KVTENANT-1|KV-Cache je Mandant und Modell|6.3|
|INV-HEAP-ZERO-1|keine Klartext-Reste im Prozessspeicher|5.3, 5.9|

---

## 13. Entscheidungspunkte (vor dem jeweiligen WP zu klären)

|ID|Frage|Optionen|Empfehlung|Vor WP|
|---|---|---|---|---|
|E-1.1|Tokenizer: Muster-Fehler fail-closed oder Degradationspfad?|Konstruktorfehler / Betrieb ohne geschützte Muster|Konstruktorfehler (INV-FAILCLOSED-1); Muster zusätzlich per Test als gültig bewiesen|1.1|
|E-2.2|Wo liegt `RelationIndex`?|in `graph` / in `ports`|in `ports`, solange `graph`-freie Signatur; sonst `graph` (ADR-D05)|2.2|
|E-2.4|Welche Ports bleiben in `ports`?|nach Nutzungsanalyse|nur Querschnitts-Ports|2.4|
|E-3.1|`xtask` behalten oder auf `cargo-make`/`just` auslagern?|behalten, bereinigt / ersetzen|behalten, Budget setzen (WP 3.6)|3.6|
|E-4.1|SSTable-Magic `MFST` (Legacy) weiter lesen?|lesen+migrieren / ablehnen|lesen, nur einmalig migrieren, danach Fail-Closed|4.1|
|E-5.1|Schlüsselverwahrung K1 im Modul oder nur Port?|K1 Software / nur Port|K1 Software als Referenz, K2/K3 nur Port (SPEC §2.4)|5.3|
|E-5.2|Standard-Policy für interne Checkpoints|`Rewrite` / `ExpireNow`|`Rewrite` (SPEC §9.4)|5.7|
|E-6.1|`adapt` zurückbauen, wenn Benchmark keinen Nutzen zeigt?|behalten / Archiv-Tag|zurückbauen (EP12)|6.3|
|E-6.2|DiskANN/RaBitQ im Designpunkt ≤ 10⁷ behalten? (O-09)|behalten / entfernen|nach Benchmark entscheiden; bis dahin hinter Feature|6.1|
|E-7.1|Plattformen: macOS/Windows als Edge unterstützen? (O-10)|ja / nein|macOS ohne Garantieaussage, Windows nicht in v1.0|7.x|

Rechts- und Marktfragen (O-01 bis O-06) sind **nicht** durch Entwicklung lösbar und stehen in SPEC §16.

---

## 14. Risiken

|Nr.|Risiko|Wirkung|Gegenmaßnahme|
|---|---|---|---|
|R1|Baseline (0.1) ist rot oder instabil|Phase 2 baut auf unbekanntem Stand|Phase 0 hat Vorrang; rote Crates auf `experimental`; kein Umbau ohne grünen Ausgangsstand des betroffenen Crates|
|R2|Entkopplung (Phase 2) bricht öffentliche APIs|Abhängige Crates, Python-Bindings, Beispiele|Fassade bleibt API-kompatibel; `semver-checks`, Kompatibilitätstests, Deprecation vor Entfernen|
|R3|Löschkette wird zu groß für einen Zuschnitt|Verzug im kritischen Pfad|ein PR je Schicht/Baustein; `beta` ist das Ziel für v1.0, nicht `stable`|
|R4|Rechtliche Erwartungen überschreiten technische Aussage|Haftung, Fehlvertrauen|Bericht weist Grenzen verpflichtend aus (SPEC §9.9); keine Wörter „rechtssicher“/„zertifiziert“; O-01 bis O-06 extern|
|R5|Gate-Inflation (58 Gates, `xtask` > Store)|Wartung, Fehlalarme, lange CI|WP 3.6; Meta-Tests; Budget für `xtask`|
|R6|Benchmarks auf falscher Hardware|Falsche Leistungsaussagen|Hardwareklasse festschreiben (O-07); keine absoluten Versprechen bis dahin|
|R7|Audit-Berichte (LLM/Agenten) werden wieder als Wahrheit gelesen|Reife-Inflation|EP17, Gate `claims-consistency`, Berichtskopf „Hypothese“|
|R8|Modelllizenzen (Qwen, Llama u. a.) schränken Einsatz ein|Rechtsrisiko für Integratoren|Lizenz je Modell im SBOM-Anhang; Beschaffung ist Integratorpflicht (SPEC §10.2)|
|R9|Netzzugriff schleicht sich wieder ein (neue Abhängigkeiten)|Verstoß VETO-NET1|`egress-inventory` statisch + T-EGR-1; `cargo deny` sperrt Netz-Crates außerhalb der Egress-Tabelle|
|R10|Prompt-Injection über Dokumentinhalte|Datenabfluss, fremde Werkzeugaufrufe|Werkzeug-Allowlist, `privacy`/`mcp`-Filter, Red-Team-Korpus (WP 6.4/6.5)|
|R11|Einzelner Entwickler (Bus-Faktor) und LLM-Unterstützung|Qualitätsstreuung|Pflicht-Gates, Evidence-Pack, `module-audit` als objektives Kriterium|

---

## 15. Definition of Done je Phase

|Phase|Fertig, wenn|
|---|---|
|0|`evidence/baseline/` für alle 35 Mitglieder, auf zweitem Runner reproduziert; `claims-consistency` grün; SPEC/PLAN 3.0 liegen unter `docs/`|
|1|Produktions-Panics = 0 in `text`, `wire`-Handcode; Allow-Register aktiv (≤ 20 befristete Einträge); Panic-Inventar getrennt|
|2|`no-cycles` ohne Ausnahme; Budgets aus SPEC §4.5 erreicht oder mit ADR begründet; Egress-Tabelle vom Gate bestätigt; `contextra-core` entfernt|
|3|`module-audit <crate>` für alle 33 Crates; Evidence-Pack und Reife ableitbar; Meta-Test der Gates grün|
|4|Crash-Matrix, Format-Register und `DurableLog` grün; `store`/`mvcc`/`checkpoint` isoliert auditierbar|
|5|Löschkette `beta` mit Chaos- und Mutationsnachweis; AC-Matrix vollständig|
|6|Evidence-Pack E3 je Modul gemäß Zielreife|
|7|Vier Profile bestehen `profile-audit`; deutsche Anleitung und lauffähige Beispiele|
|8|Alle Zeilen aus SPEC §13.4 belegt; externer Review beauftragt (Ergebnis für `certified-ready`)|

---

## 16. Nächste Schritte (erste zwei Wochen)

1. **0.1** Baseline auf sauberem CI-Runner (Toolchain 1.89.0, alle 35 Mitglieder einzeln); ohne dieses Ergebnis ist jede weitere Zahl in SPEC und PLAN E1.
2. **0.2 + 0.3** Berichte auf „Hypothese“ setzen, `capabilities.toml` auf Schema 3, SPEC/PLAN 3.0 nach `docs/` und ADR-D01 bis ADR-D10 anlegen.
3. **1.1** Tokenizer-`panic!` (S), parallel **1.4** `wire`-Generat.
4. **2.1** Kanten und Budgets als Gates mit Ist-Werten (Startpunkt für alle Entkopplungen).
5. **0.4** Mess-Befehl (AST-basiert) – danach ersetzen seine Werte die E1-Handzählungen dieser Dokumente.

---

## Anhang P1. Abbildung PLAN 2.5 → PLAN 3.0

|PLAN 2.5 (Repo, `docs/PLAN.md`)|PLAN 3.0|
|---|---|
|Phase 0 Baseline und Netz|Phase 0 (Baseline) + WP 2.7 (Netz)|
|Phase 1 Stubs und Sofortschäden|Phase 1 (Stubs sind am HEAD nicht mehr vorhanden, SPEC U-04)|
|Phase 2 Archivierung und Entkopplung|Phase 2 (**ohne Archivierung**, ADR-D01)|
|Phase 3 Fundament|WP 4.1, 4.2 und Phase 3 (Modulwerkzeug)|
|Phase 4 Store|Phase 4|
|Phase 5 Schlüssel|WP 5.3, 5.4|
|Phase 6 Scanner und Purger|WP 5.1, 5.2|
|Phase 7 Orchestrator|WP 5.5, 5.6, 5.7, 5.8|
|Phase 8 Fassade, Kernhärtung, Schnittstellen|Phase 6, WP 3.7, Phase 7|
|Phase 9 Abnahme|WP 5.9, Phase 8|

## Anhang P2. Änderungsverlauf

|Fassung|Datum|Änderung|
|---|---|---|
|2.5|27.10.2026|Phasenplan „Kern schließen, Rest archivieren“|
|**3.0**|**28.10.2026**|Baukasten-Zuschnitt; Phase 3 (Modulwerkzeug), Phase 6 (Härtung je Familie), Phase 7 (Profile); Löschkette als Baustein (Phase 5, neues Crate `erasure`); WP 2.11 (Dev-Kanten); Nachmessung am HEAD 52d3227; Archivierungsentscheidung aufgehoben|
