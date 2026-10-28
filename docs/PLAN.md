# Contextra – Umsetzungsplan 3.1 (IST → SOLL, Baukasten-Fassung)

```
Dokument:   Umsetzungsplan zur SPEC 3.1 (docs/SPEC.md)
Fassung:    3.1 (Entwurf zur Freigabe); überarbeitet 3.0 um Baseline-, Metrik- und Scan-Befunde (U-15 bis U-27)
Stand:      28.10.2026
Belegbasis: drei Commit-Stände (SPEC §1.2): Klon HEAD 21a09ad (statisch gelesen), Baseline-Lauf auf 8f8fa7cd (E2, nur Summaries,
            Toolchain 1.89.0), Skript-Scan auf 52d3227 (Heuristik). In dieser Überarbeitung wurde NICHTS kompiliert oder ausgeführt.
Ersetzt:    PLAN 3.0 (Entwurf) und PLAN 2.5 (Phasenplan „Kern schließen, Rest archivieren“, 1.120 Zeilen, Phasen 0–9)
Ziel:       robuste, datenschutzkonforme, entkoppelte Rust-Bibliothek als Fundament für lokale
            LLM-Lösungen im Mittelstand in Deutschland und Europa; jedes Modul einzeln
            auditierbar, verifizierbar und leistungsmessbar
```

> **Rangfolge.** Bei Widerspruch gilt die SPEC. Jedes Arbeitspaket (WP) nennt sein SPEC-Ziel, seinen Schnitt, seine Abnahme (Gate oder Test) und seine Abhängigkeiten. **Ein PR = ein WP = höchstens ein Crate-Schnitt**; nach jedem Schnitt laufen Build, `module-audit` des betroffenen Crates und die Zeilenzählung.

> **Aufwandsangaben** (S ≈ ≤ 2 Tage, M ≈ ≤ 1 Woche, L ≈ ≤ 3 Wochen, XL > 3 Wochen, je eine Entwicklerin/ein Entwickler mit LLM-Unterstützung) sind **Schätzungen ohne Messung** und dienen nur der Reihenfolge.

---

## 0. Ausgangslage in einer Seite

Der Code ist **weiter und zugleich weniger belegt**, als der Audit-Bericht vom 28.10.2026 behauptet. Die Baseline (E2, teilweise) liefert erstmals Messwerte: **`cargo check -p <crate>` ist für alle 35 Mitglieder grün**, aber `cargo check --workspace --all-targets` ist rot (drei defekte Test-Targets, darunter der Löschbericht-Test), **`clippy -D warnings` besteht nur bei 2 von 35 Crates** (Ursache unbelegt) und bei den Tests stehen **20 grün, 2 Compile-Fehler und 13 Timeouts** im 45-s-Harness. Der Code hat (E1) eine saubere Ringrichtung ohne Ringverletzung, klare Unsafe-Inseln, ein umfangreiches Gate-Gerüst (58 Gates) und ein vorhandenes Scanner-/Bericht-Fundament für die Löschkette. Er hat **nicht**: einen vollständig grünen Build-/Test-Beleg, wirkliche Entkopplung (10 Crates über Transitiv-Budget, 8 Crates plus Fassade mit harten Kanten außerhalb §4.4), isolierte Modulprüfung, ein Netz-Standard „aus“ (ONNX lädt aus dem Netz), eine einzige Messquelle (drei widersprüchliche Zählungen), eine Typfamilie je Konzept (35 doppelte Typnamen) und den größeren Teil der spezifizierten Löschkette (Schlüsselhierarchie, Sperre, Saga, fünf Schichten).

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
|U-15|Drei Test-Targets kompilieren nicht (`db`, `crypto`, `mvcc`); Workspace `--all-targets` rot|0.5|0|
|U-16|Clippy `-D warnings`: 33 von 35 rot, Ursache unbelegt|0.6|0|
|U-17|13 Crates im Test-Timeout, Statistik deckt 15 Crates nicht ab|0.7|0|
|U-18|Drei widersprüchliche Messungen (Panics, Zeilen, Zeitaufrufe)|0.4|0|
|U-19|Direkte Kanten außerhalb §4.4 (`db` 10, `py` 6, `mcp` 5, Fassade → `license`, …)|2.3, 2.6, 2.7, 2.14|2|
|U-20|35 doppelt definierte Typnamen (u. a. `DeletionProof`, `TenantId`, KV-Typen)|2.13 (Teil: 2.5, 5.1)|2|
|U-21|44 direkte Zeitaufrufe in Produktion (`mvcc` 14 in Ring 0)|2.12|2|
|U-22|Undefinierte `cfg(feature)` in `store` (Sicherheitscode wird nie kompiliert); 21 tote Features|1.5, 2.9|1|
|U-23|15 Stellen blockierendes I/O in `async fn`|1.7|1|
|U-24|Test-Hooks und Mocks in der Produktions-API (`py`, `store`, `mcp`)|1.6|1|
|U-25|Unverdrahtete Ports und Null-/Test-only-Implementierungen (u. a. `KmsProvider`)|3.4, 5.3|3|
|U-26|`unsafe` in Testcode außerhalb der Inseln; `wire`-Generat ohne `// SAFETY:`|0.4, 1.4|0|
|U-27|`capabilities.toml` unvollständig, 5 Dateien > 800 Zeilen, veraltete Doku-Stempel|0.2, 2.1, 3.8|0|

### 0.2 Leitplanken des Plans

1. **Wahrheit zuerst.** Phase 0 erzeugt eine belegte Baseline, bevor etwas umgebaut wird.
2. **Sofortschäden vor Architektur.** Produktions-Panics und Netz-Download sind klein und werden zuerst beseitigt.
3. **Entkopplung vor Löschkette.** Die Löschkette braucht saubere Ports und das neue Crate `erasure`; sie wird erst nach Phase 2 und 3 gebaut.
4. **Messbar je Schritt.** Jedes WP hat ein Gate oder einen Test als Abnahme; „fertig“ heißt Gate grün auf CI, nicht „Code geschrieben“.
5. **Nichts löschen, was nicht ersetzt ist.** Bestehende Tests wandern mit dem Code; ein Test wird nur mit Begründung entfernt (Gate `test-integrity`).
6. **Rückbau ist erlaubt.** Fähigkeiten ohne Benchmark-Beleg oder Verdrahtung (EP3, EP12) werden zurückgebaut oder auf `experimental` gesetzt.
7. **Ursache vor Behebung.** Ein Fehlerbild (Clippy, Timeout, Test-Compile) wird an den **rohen Logs** erklärt, bevor Code geändert wird; Ursachenaussagen aus Berichten sind Hypothesen (SPEC §1.2a).
8. **Timeout ist rot.** Ein nicht ausgeführter Test ist kein bestandener Test (ADR-D11); Baseline heißt je Crate **und** Workspace, alle Targets.
9. **Tests folgen dem Vertrag.** Ein defekter Test wird an die gültige API angepasst oder mit Begründung ersetzt; ein entfernter Typ wird nicht wieder eingeführt, nur um einen Test grün zu machen.
10. **Zuerst, was Sicherheit oder Mandantentrennung betrifft** (U-20 `TenantId`, U-22 `cfg`-Features), dann Architektur.

---

## 1. Phasenübersicht

|Phase|Ziel|WPs|Abnahme (Kern)|Voraussetzung|Aufwand|
|---|---|---|---|---|---|
|**0 Baseline und Wahrheit**|belegter Ist-Stand je Crate **und** Workspace; ehrliche Reife; eine Messquelle|0.1 – 0.7|`--all-targets` grün; Clippy-Ursache belegt; kein Test-Timeout; Baseline-Protokoll E2; `claims-consistency` grün|–|M–L|
|**1 Sofortschäden**|keine Produktions-Panics; Allow-Register; keine toten Sicherheits-`cfg`; keine Test-Hooks in der API; kein blockierendes I/O|1.1 – 1.7|`panic-inventory` Produktion = 0 (`text`, `wire`); `undefined-cfg-feature`, `test-hooks-private`, `async-blocking-io` grün|0.1, 0.5|M|
|**2 Entkopplung**|Soll-Graph (SPEC §4.4/§4.5), keine Zyklen, Netz „aus“|2.1 – 2.14|`transitive-budget`, `no-cycles`, `egress-inventory`, `duplicate-types`, `determinism` grün|0.1, 1.x|XL|
|**3 Modul-Infrastruktur**|isolierte Prüfung, Evidence-Pack, Benchmarks je Crate|3.1 – 3.9|`module-audit <crate>` läuft für alle Crates; Wartbarkeitsregeln W1–W11 als Gates|2.1|L–XL|
|**4 Persistenz-Fundament**|`DurableLog`, Format-Register, Crash-Matrix je Modul|4.1 – 4.4|Crash-Matrix grün; `DurableLog` implementiert|3.2|L|
|**5 Löschkette**|Baustein nach SPEC §9 bis `beta`|5.1 – 5.10|AC-Matrix, Mutationskatalog, Chaos, Rechtsmatrix|2.4, 3.x, 4.x|XL|
|**6 Modul-Härtung F1–F5**|Zielreife je Modul (Fuzz, Differential, Bench)|6.1 – 6.5|Evidence-Pack E3 je Modul|3.x|XL|
|**7 Profile, Beispiele, Doku**|Search, Local-RAG, Agent, Compliance lauffähig und geprüft|7.1 – 7.5|`profile-audit` grün|6.x (teilweise)|L|
|**8 Abnahme und Release**|Release-Kriterien SPEC §13.4|8.1 – 8.3|alle Kriterien grün; externer Review|alle|M–L|

**Kritischer Pfad:** 0.5 → 0.1 → 2.1 → 3.2 → 4.2 → 5.1 → 5.3 → 5.4 → 5.5 → 5.6 → 8.x (0.5 zuerst: ohne lauffähige Löschbericht-Tests ist G2 unbelegt). **Parallel möglich:** Phase 1 neben Phase 0; Phase 2 (unabhängige Entkopplungen 2.2/2.3/2.5/2.8/2.10 je Crate); Phase 6 je Modulfamilie; Phase 7 je Profil, sobald dessen Module `beta` erreichen.

---

## 2. Phase 0 – Baseline und Wahrheit

### WP 0.1 – Belegte Build-/Test-/Clippy-Baseline (U-10)

**Stand 3.1:** Teilweise erfüllt. Vorliegend: Exit-Code und Dauer je Crate für `check`, `clippy`, `test` sowie zwei Standalone-Builds und vier Workspace-Läufe (SPEC §1.2a), **ohne rohe Logs** und auf einem Commit (8f8fa7cd), der im flachen Klon nicht auflösbar ist. Offen: Logs, Zielcommit, zweiter Runner, vollständige Tests.

**Ziel:** Erstes **E2-Protokoll** für alle 35 Workspace-Mitglieder, vollständig und auf **einem** Zielcommit.

**Schnitt:** CI-Konfiguration und `evidence/baseline/` (kein Produktionscode).

**Aufgaben**

1. Toolchain 1.89.0 auf einem sauberen Runner; Lauf von `cargo build -p <c> --no-default-features` **je Mitglied einzeln** (auch die neun, die nicht in `default-members` stehen: `infer-candle`, `sandbox`, `py`, `privacy`, `license`, `audit-export`, `avv-generator`, `bench`, `xtask`).
2. `cargo test -p <c>`, `cargo clippy -p <c> --all-targets --locked`, `cargo deny check`, `cargo audit`, `cargo fmt --check`.
3. Ergebnis je Crate als `evidence/baseline/<crate>.json` (Befehl, Commit, Toolchain, Exit-Code, Dauer, Testanzahl, rote Tests, Warnungen).
4. Rote oder nicht kompilierende Crates erhalten ein Ticket und werden in `capabilities.toml` auf `experimental` gesetzt, bis sie grün sind.

5. Ausgangspunkt ist `evidence/baseline-a547a42/` (älterer Commit, dort rot: `contextra-engine` fand `contextra_graph` nicht). Dieses Protokoll gilt **nicht** für den heutigen HEAD; es wird als `historisch` markiert und durch `evidence/baseline/` ersetzt.
6. **Zielcommit festlegen** (O-12) und den flachen Klon vertiefen (`git fetch --unshallow`); alle Zahlen aus 8f8fa7cd, 52d3227 und 21a09ad werden auf diesen Commit neu erhoben, bis dahin nicht addiert.
7. Zusätzlich zu den Einzelläufen: `cargo check --workspace --all-targets --locked`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo test --workspace --no-fail-fast --locked` (Referenz: ≈ 6,2 min) mit **rohen Logs je Befehl und je Crate** unter `evidence/baseline/logs/`.
8. Testlimit je Crate ≥ Referenzlaufzeit oder `cargo nextest`; **Timeout wird als rot protokolliert** (ADR-D11), nicht als „grün (0)“.
9. Die Ursachenspalte in `BASELINE.md` (z. B. „unkonfigurierte Warnings“, „`LayerCleanupProof` liegt in `contextra_crypto` statt `contextra_engine`“) wird als **Hypothese** gekennzeichnet; Klon-Befund: der Typ ist nirgends definiert.

**Abnahme:** Protokoll mit rohen Logs auf dem Zielcommit vorhanden und von einem zweiten Runner reproduziert (E3-Grundlage); Exit-Codes je Crate stimmen mit WP 0.5 bis 0.7 überein. **Aufwand:** M.

### WP 0.2 – Reife und Berichte bereinigen (U-01)

**Ziel:** Keine Aussage in Repository oder Berichten, die `capabilities.toml` widerspricht.

**Aufgaben**

1. `SYSTEMATIC_CRATE_AUDIT.md`: Kopf „**Hypothese – nicht belegt**“, Verweis auf SPEC §1.3; Noten (x/10) und Formulierungen „Produktionsreif“ entfernen oder durch Verweis auf Evidence-Pack ersetzen.
2. `capabilities.toml` auf die Felder des Modulvertrags erweitern (SPEC §4.3): `family`, `class`, `may_depend_on`, `transitive_budget`, `provides`, `requires`, `egress`, `bench`, `non_goals`, `evidence`.
3. Reife nach Baseline (0.1) setzen: nur `stable`, wenn E3 und V1–V6 belegt; sonst `beta`/`experimental` (Erwartung: viele Crates fallen von `stable` auf `beta`).
4. Gate `claims-consistency` (neu): vergleicht Berichte, README, `MODULE.md` mit `capabilities.toml`.
5. Baseline-Aussagen: „alle Crates kompilieren fehlerfrei“ wird nur noch mit Angabe des Prüfumfangs akzeptiert (`check -p` vs. `--all-targets`); `claims-consistency` prüft das gegen ADR-D11.

**Abnahme:** Gate grün; keine Fundstelle von „produktionsreif“ ohne Evidence-Pack. **Aufwand:** S–M.

### WP 0.3 – SPEC 3.1 und PLAN 3.1 ablegen, Dokumente konsolidieren

**Aufgaben**

1. `docs/SPEC.md` und `docs/PLAN.md` durch die Fassungen 3.1 ersetzen; die Fassungen 2.4 und 3.0 unter `docs/spec/archive/` ablegen.
2. ADR-D01 bis ADR-D16 (SPEC §15) in `docs/decisions/` anlegen (heute genau eine Datei dort: `0001-gemessene-loeschung-scanner-vertrag-v1.md`; Nummerierung fortsetzen, Format der Datei 0001 übernehmen).
3. Anforderungskatalog (alt §19) nach `docs/spec/requirements.toml` überführen, je Anforderung `module`, `test`, `status`.
4. Anhang C der SPEC schrittweise in `crates/<c>/MODULE.md` und Fachdokumente überführen, dann streichen.
5. Veraltete Dokumente (`docs/analysis`, `docs/reports`) mit Datum und „historisch“ kennzeichnen.

**Abnahme:** Gate `doc-truth` und `adr-index` grün. **Aufwand:** M.

### WP 0.4 – Reproduzierbarer Mess-Befehl

**Ziel:** Die Kennzahlen aus SPEC §5.2 (Code-/Testzeilen, `unsafe`, Panic-Muster, transitive Last, Features) sind **maschinell reproduzierbar**.

**Aufgaben:** `cargo xtask metrics --json` (aus `cargo metadata` und `syn`/AST statt Regex), Ausgabe nach `evidence/metrics.json`; Gate `metrics-ratchet` (nur besser werden). **Zusatz 3.1 (ADR-D12, U-18):** die Messung ist die **einzige** Zahlenquelle für SPEC und PLAN und ersetzt Scan-Skript, `ratchet.toml`-Altwerte und Handzählungen; sie weist Kanten nach Art (normal, optional, dev) aus `cargo metadata` aus, trennt Produktions-/Test-/generierten Code, zählt Panics, `allow`, `unsafe` (Block, nicht Muster-String), Zeit-/Netzaufrufe, Doppeldefinitionen und tote/undefinierte Features; die Methode steht im Kopf der Ausgabedatei.

**Abnahme:** Zwei Läufe auf demselben Commit liefern identische Werte. **Aufwand:** M.

### WP 0.5 – Defekte Test-Targets reparieren (U-15, höchste Priorität)

**Schnitt:** je ein PR je Crate (`db`, `crypto`, `mvcc`), nur Test-Code.

**Ist:** `db/tests/deletion_proof_integration.rs` und `crypto/tests/audit_chain_tests.rs` importieren `LayerCleanupProof` (im Klon **nirgends definiert**, Verwendung z. B. `LayerCleanupProof::for_test(...)`); `mvcc/tests/ssi_metrics_observability.rs` importiert `MetricEvent`, `TestMetricsSink` (Baseline: aus `ports::metrics`; Klon: aus `testkit`, dort in `mocks.rs` definiert).

**Aufgaben**

1. Am Zielcommit die **rustc-Meldungen im Rohlog** lesen; Abweichungen zwischen Baseline-Text und Klon-Befund dokumentieren (SPEC §1.2a Punkt 2).
2. `LayerCleanupProof`: Entscheidung E-0.5 (Test an den gültigen V5-Vertrag anpassen oder, falls der Vertrag den Typ vorsieht, genau eine Definition in `crypto` einführen). **Nicht** einen Hilfstyp nur für den Test zurückbringen.
3. `mvcc`: Importpfad und `dev-dependencies` (`testkit`) prüfen.
4. Gate `baseline-all-targets` in die PR-Stufe aufnehmen.
5. Keine Tests löschen (`test-integrity`); eine Anpassung belegt, dass dieselbe Garantie (G2, G6) weiter geprüft wird.

**Abnahme:** `cargo check --workspace --all-targets --locked` Exit 0; `cargo test -p contextra-crypto -p contextra-db -p contextra-mvcc --no-run` Exit 0. **Aufwand:** S–M.

### WP 0.6 – Clippy: Ursache belegen, dann beheben (U-16)

**Aufgaben**

1. Aus den Rohlogs je Crate: erste Fehlermeldung, Lint-Namen, Fundzahl; gruppieren nach **Ursache** (workspace-weites Lint, gemeinsame Abhängigkeit, echte Lints je Crate). Die nahezu gleiche Laufzeit von ≈ 2,2 s bei 31 Crates ist ein Hinweis auf eine gemeinsame Ursache, aber kein Beleg.
2. Je Gruppe: beheben (Standard) oder Lint-Konfiguration mit ADR. **Kein** Absenken von `-D warnings`, **keine** pauschalen `allow` (EP16).
3. Reihenfolge entlang der Abhängigkeiten (`types`, `ports`, dann Ring 0 → 4).
4. Gate `clippy-baseline` (je Mitglied mit `--all-targets`).

**Abnahme:** `cargo clippy -p <c> --all-targets --locked -- -D warnings` Exit 0 für alle 35 und `--workspace`. **Aufwand:** M–L (Schätzung ohne Messung; hängt von Schritt 1 ab).

### WP 0.7 – Testlaufzeit und Timeouts (U-17)

**Aufgaben**

1. Für die 13 Crates (`store`, `graph`, `vector`, `db`, `cognition`, `engine`, `text`, `infer-candle`, `mcp`, `contextra`, `agent`, `bench`, `xtask`) Laufzeit je Test messen (nextest, JUnit-Ausgabe).
2. Ursache klassifizieren: echte Last, Fixture-Aufbau, blockierendes I/O (WP 1.7), Wartezeit mit Echtzeit statt `ManualClock` (WP 2.12), Deadlock.
3. Laufzeitbudget je Crate (SPEC W10); langsame Tests hinter `slow-tests` bzw. nextest-Profil; `quick`-Profil je Crate < 120 s.
4. Vollständigen Testlauf **ohne 45-s-Limit** protokollieren und die Teststatistik (bisher 1.259 bestanden aus 20 Crates) für alle 35 Mitglieder neu erheben.
5. Gate `test-runtime-budget` (nightly).

**Abnahme:** kein Crate im Timeout; Statistik für alle Mitglieder; ursächliche Tests benannt. **Aufwand:** M.

---

## 3. Phase 1 – Sofortschäden

### WP 1.1 – `panic!` im Tokenizer (U-02)

**Schnitt:** `contextra-text`, `src/tokenizer.rs`.

**Aufgaben:** `get_protected_regex()` darf nicht panicken. Muster einmalig beim Start kompilieren und **Fehler als `ContextraError`** an den Konstruktor des Tokenizers melden (Fail-Closed) oder auf eine **konstante, bewiesen gültige** Regex-Konstruktion über `regex::Regex::new` in einem Test absichern und im Fehlerfall den Tokenizer ohne geschützte Muster betreiben (dokumentierter Degradationspfad). Entscheidung vor dem WP (E-1.1, §13). `#[allow(clippy::panic)]` entfernen.

**Abnahme:** `panic-inventory` zeigt für `text` Produktion = 0; Test „ungültiges Muster erzeugt Fehler, keinen Abbruch“; Fuzz-Smoke des Tokenizers grün. **Aufwand:** S.

### WP 1.2 – Allow-Register (EP16)

**Ausgangszahl (E1, zeilenbasierte Zählung, methodenabhängig):** 136 bis 158 Allow-Direktiven auf `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic` außerhalb von `tests/`-Verzeichnissen (zwei Zählmuster am selben Stand ergeben 136 bzw. 158; verbindlich ist erst der AST-Wert aus WP 0.4); Schwerpunkte `types` 27, `mcp` 18, `infer-ollama` 18, `mvcc` 14, `router` 13, `checkpoint` 12, `privacy` 12, `crypto` 10. Ein Teil steht in `#[cfg(test)]`-Modulen. **3.1:** Die Metrik-Datei nennt Produktions-Panics in `db` (8 `expect`, 2 `panic`), `store` (2/4), `text` (6/1), `vector` (1 `unwrap`); der Scan nennt `store` 2/1, `text` 6/1 und nichts für `db`. Der Widerspruch wird in WP 0.4 aufgelöst; bis dahin gilt je Crate die **größere** Zahl als Arbeitsumfang.

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

### WP 1.5 – Undefinierte `cfg(feature)` in `store` (U-22, sicherheitsrelevant)

**Schnitt:** `contextra-store` (und Manifest).

**Ist:** `cfg(feature = "encryption-at-rest")` und `cfg(feature = "wal-integrity")` ohne Definition im Manifest: der davon abhängige Code wird **nie kompiliert**, also nie getestet. Das widerspricht A1 („Sicherheit ist kein Feature-Schalter“). Dazu 21 definierte, im Code nicht genutzte Features (Scan; reine Weiterleitungen im Manifest sind Fehlalarme zu prüfen).

**Aufgaben:** (1) Stellen lokalisieren und prüfen, ob der Code durch WAL v3 (HMAC-Kette) und Record-Verschlüsselung überholt ist; (2) Entscheidung E-1.5: Code entfernen, **unbedingt** kompilieren (Sicherheitscode ohne Schalter) oder Feature definieren und in der `feature-matrix` testen; (3) Gate `undefined-cfg-feature`; (4) `unexpected_cfgs = "deny"` in `store` (zusammen mit WP 2.9).

**Abnahme:** Gate grün; kein `cfg(feature)` ohne Definition im Workspace. **Aufwand:** S.

### WP 1.6 – Test-Hooks und Mocks aus der Produktions-API (U-24)

**Schnitt:** `py`, `store`, `mcp`, `checkpoint` (ein PR je Crate).

**Aufgaben:** (1) `py`: `_trigger_panic_for_test` (3 Stellen) hinter Feature `test-hooks`, nicht im Release-Set; (2) `store`: `simulate_wal_append_failure_for_test` nur unter `cfg(any(test, feature = "test-hooks"))`; (3) `mcp/config.rs`: `MockEmbedder`/`MockLlmGenerator` nach `testkit` oder `cfg`; ist ein Demo-Backend gewollt, umbenennen, dokumentieren und nie stiller Standard (INV-FAILCLOSED-1); (4) `LoomFile`/`LoomMetadata`/`LoomReadDir` (Konstanten-Stubs in `store`, `checkpoint`): `cfg(loom)` prüfen, Doppelt `LoomMetadata` zusammenführen (WP 2.13); (5) `mint_witness_for_tests` nur als `dev-dependency` erreichbar; (6) Gate `test-hooks-private`.

**Abnahme:** Gate grün; `cargo public-api` zeigt keine `*_for_test`-Items im Release-Feature-Set. **Aufwand:** S–M.

### WP 1.7 – Blockierendes I/O in `async` (U-23)

**Schnitt:** je Crate ein PR: `store` (`wal/mod.rs`, `wal/replay.rs`, `sstable/reader.rs`), `vector` (`diskann/build.rs`, `hnsw/core_rebuild.rs`: `std::thread::sleep`), `infer-onnx` (`lib.rs`, siehe auch WP 2.7), `mcp` (Binary-Start).

**Aufgaben:** Dateizugriff über `Vfs` oder `spawn_blocking`; Wartezeiten über `Clock`/`tokio::time`; Gate `async-blocking-io`; Test, dass ein Ein-Thread-Executor während Replay und Rebuild nicht länger als ein Budget blockiert.

**Abnahme:** Gate grün; Test grün; die Zeitmessung aus WP 0.7 zeigt keine Verschlechterung. **Aufwand:** M.

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

### WP 2.12 – Determinismus: `Clock`-Port durchsetzen (U-21)

**Ist (Scan, 44 Stellen):** `mvcc` 14 (Ring 0), `engine` 7, `store` 7 (Metrik: 14), `kvcache` 3, `ports` 3, `router` 3, `checkpoint` 2, `sandbox` 2, `adapt`/`db`/`privacy` je 1; `clippy.toml` ohne `disallowed_methods`/`disallowed_types`.

**Aufgaben:** (1) Je Stelle `Clock` einsetzen oder mit Begründung in `determinism-allow.toml` eintragen (nur `sys`, Werkzeug, Messung); (2) `clippy.toml`: `disallowed_methods` für `SystemTime::now`, `Instant::now`, `thread::sleep`; (3) Reihenfolge: `mvcc` (SSI- und Recovery-Tests), `store`, `engine`, dann Rest; (4) Tests mit `ManualClock`.

**Abnahme:** Gate `determinism` ohne ungedeckte Stelle; T-DET-1 und T-REC-2 grün. **Aufwand:** M.

### WP 2.13 – Doppeldefinitionen beseitigen (A2, U-20)

**Methode je Name:** gleiches Konzept ⇒ **zusammenführen** im besitzenden Crate; verschiedene Konzepte ⇒ **umbenennen** (Domänenpräfix). Register `governance/duplicate-types.toml` mit Frist für verbleibende Fälle; vor jeder Zusammenführung eines serialisierten Typs gilt Golden-File-Test (Formate nicht brechen, R15). Zahlen stammen aus dem Scan (52d3227) und werden am Zielcommit mit WP 0.4 neu gemessen.

|Gruppe|Namen|Entscheidung (Empfehlung)|Zusammenhang|
|---|---|---|---|
|**Mandant (zuerst)**|`TenantId` in `privacy/processing_registry.rs` und `types`|Duplikat entfernen, `types::TenantId` nutzen; VETO-F10-Review (Typverwechslung darf Mandantentrennung nie umgehen)|sofort, Aufwand S|
|Löschbeweis|`DeletionProof` (`crypto`, `graph/cascade.rs`), `DeletionProofKeyPair` (doppelt in `crypto`)|je **eine** Definition in `crypto`; `graph` nutzt Witness-Typen/Port|WP 5.1|
|KV-Cache|`KvSegment`, `EncryptedSegmentPayload`, `EvictionWorker`, `SpillHandler`, `TenantIsolatedKvStore` (`crypto`, `kvcache`)|`kvcache` besitzt, `crypto` verliert|WP 2.5|
|Zerfallsregler|`AdaptiveDecayController`, `DecayControllerConfig`, `DecaySignalInputs` (`adapt`, `engine`)|`adapt` besitzt; `engine` konsumiert|WP 2.6|
|Graph-Typen|`HyperEdgeId`, `RoleId`, `RoleBinding`, `GraphMutationError` (`graph`, `ports`), `Edge` (`graph`, `types`)|`graph` besitzt; `ports` führt graph-freie Signatur|WP 2.2, 2.4|
|Egress und AVV|`EgressGuard` (`mcp`, `privacy`), `AvvContext` (`avv-generator`, `privacy`)|`privacy` besitzt Guard; `avv-generator` besitzt `AvvContext`|WP 2.7|
|Statistik/Fusion|`DriftDetector` (`adapt`, `rank`), `ConformalCalibrator` (`rank`, `router`), `BoundedTopK` (`rank`, `text`), `Signal`, `SignalKind`, `HybridQueryBuilder`, `TlHfdParams` (`types`, andere)|je Fall prüfen: gemeinsames Konzept nach `types`/`ports`, sonst umbenennen|je Crate|
|Übrige|`EntityExtractor`, `EntityExtractionConfig`, `CommunityDetectionConfig`, `PercolationConfig`, `ConsolidationNodesGuard`, `CompactionStrategy`, `SynthesisPhaseResult`, `ContextEditAuditRecord`, `LoomMetadata`, `BoxFuture`|umbenennen oder zusammenführen|je Crate|

**Abnahme:** Gate `duplicate-types` grün (Register mit Fristen leer oder begründet); Golden-Files unverändert. **Aufwand:** L (ein PR je Gruppe).

### WP 2.14 – Verbleibende Kanten außerhalb SPEC §4.4 (U-19)

**Ist (Metrik-Datei, harte Kanten gegen §4.4 gerechnet):**

|Crate|Kante (hart bzw. optional)|Soll|Mittel|
|---|---|---|---|
|`db`|`cognition`, `graph`, `adapt` (→ WP 2.3), zusätzlich direkt `sys`, `crypto`, `store`, `vector`, `text`, `checkpoint`, `rank`|nur `types`, `ports`, `engine`|Typen und Funktionen über die `engine`-API oder Ports; Fassade verdrahtet|
|`engine`|`sys` hart; `sandbox` optional|`sys` entfällt (über `store`/Port); `sandbox` über Port|Port `Sandbox` oder Feature der Fassade|
|`contextra` (Fassade)|`license`, `core` hart|`license` optional (P30), `core` entfällt|WP 2.8; Port `LicenseGate` (E-2.14)|
|`mcp`|`license`, `rank`, `crypto`, `privacy`, `router` hart; `infer-candle` optional|nur über Fassade, optional|WP 2.7|
|`py`|`router`, `rank`, `adapt`, `core`, `store`, `db` hart|`types`, Fassade|WP 2.7|
|`router`|`privacy`, `wire` hart|optional|Feature oder Port|
|`infer-onnx`|`infer-candle` optional (Rand → Rand ohne Eintrag)|entfernen oder deklarieren|Reranker-Double statt Kante|
|`checkpoint`, `vector`, `cognition`, `agent`|`core`; `crypto`; `engine`/`store`/`vector`; `db`/`store`|siehe WP 2.8, 2.10, 2.6|–|

**Aufgaben:** je Zeile Kante entfernen oder als befristete Ausnahme in `capabilities.toml` führen (Ablauf-WP); danach Budgets in `transitive-budget` auf Soll senken.

**Abnahme:** Gate `may-depend-on` ohne Ausnahme; Soll-Budgets SPEC §4.5 erreicht oder per ADR begründet. **Aufwand:** M–L.

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
4. **Unverdrahtete Ports (U-25)** je Fund entscheiden: verdrahten, Nutzen mit Integrationstest belegen oder entfernen (EP3). Betroffen laut Scan: `TextGenerator`, `Snapshot`, `KmsProvider` (UNWIRED); `MetricsSink`, `ReplayProgressSink`, `EntityRecognizer` (nur Null-Implementierung); `KvBridgeStorage`, `PluginManifest`, `SemanticEmbedder`, `AgentTool` (nur Tests). Ein Sicherheitsport mit nur Null-Implementierung macht sein Crate `experimental` (SPEC §7.5).

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
5. Das externe Skript `contextra_scan.py` wird **nicht** Gate (SPEC §17 F-03, prüft gegen Nummern der Spezifikation 2.x); seine nützlichen Prüfungen (`duplicate-types`, `undefined-cfg-feature`, `test-hooks-private`, `async-blocking-io`, Zeitaufrufe) werden als AST-Gates in `xtask` neu geschrieben und per Meta-Test auf Wirksamkeit geprüft.

**Abnahme:** Inventar vollständig; Rückbau-Liste umgesetzt; Gate-Laufzeit gesamt im Protokoll (Ziel: PR-Stufe < 15 Minuten, Startwert, nicht gemessen). **Aufwand:** L.

### WP 3.7 – Profil-Gerüst `profile-audit`

**Aufgaben:** `cargo xtask profile-audit <search|local-rag|agent|compliance>` baut das Profil, prüft transitive Last gegen SPEC §11, führt `egress-inventory` und einen Smoke-Benchmark aus; Fassaden-Features (`embedded`, `candle`, `onnx`, `ollama`, `agents`, `graph`, `cognition`, `router`, `privacy`, `mcp`, `sandbox`, `compliance`, `license`) nach SPEC §4.7 neu schneiden; `contextra-core` und `license` sind **nie** Pflicht.

**Abnahme:** Gerüst läuft für alle vier Profile (Beispiele folgen in Phase 7). **Aufwand:** M.

### WP 3.8 – Wartbarkeitsregeln W1–W11 als Gates (SPEC §4.9)

**Aufgaben:** je Regel Gate und Startwert mit Ratchet: `file-size` (W1, Start: 5 Dateien über 800 Zeilen), `public-api`/`unwired-pub` (W2, Start: 89), `#[non_exhaustive]`-Liste und `semver` (W3), `-D deprecated` und Migration von `TenantId::new` (W4), Algorithmus-ID in jedem signierten/verschlüsselten Format (W5, Inventar aus WP 4.1), `duplicate-types` (W6, WP 2.13), `rust-version` je Crate = Pin (W7), `default-features = false` und `deny`-Regeln (W8, mit WP 2.9), `MODULE.md`-Pflicht und Doctests (W9, mit WP 3.1), `test-runtime-budget` (W10, mit WP 0.7).

**Abnahme:** jedes Gate hat Meta-Test (absichtliche Verletzung scheitert) und Startwert im Protokoll. **Aufwand:** M–L.

### WP 3.9 – Schnittstudie großer Crates (W11)

**Ziel:** begründete Entscheidung, ob `store` (13.521 Zeilen: WAL, LSM, SSTable, Blob, Scanner), `engine` (9.372), `graph` (7.796), `vector` (7.878) und `xtask` (32.572) geteilt werden, **bevor** die Löschkette weitere Scanner und Purger in `store` und `vector` ablegt (WP 5.2).

**Aufgaben:** Änderungsgrund-Analyse (Git-Hotspots, Kopplung der Module innerhalb des Crates, öffentliche API je Teil), Variantenvergleich (Modulgrenzen im Crate, Feature, eigenes Crate), Kosten (transitive Last, API-Budget); Ergebnis als ADR je Crate. **Keine Teilung ohne ADR** und ohne grüne Baseline des betroffenen Crates.

**Abnahme:** ADRs vorhanden; Entscheidung dokumentiert, auch wenn sie „nicht teilen“ lautet. **Aufwand:** M.

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

**Zusatz (3.1):** `graph/src/cascade.rs` definiert einen eigenen `DeletionProof`, und `crypto` hält `DeletionProofKeyPair` doppelt (`deletion_proof.rs`, `ed25519_proof.rs`): je eine Definition (WP 2.13). `LayerCleanupProof` (nirgends definiert, in zwei Tests importiert) wird in WP 0.5 entschieden, nicht hier.

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

**Zusatz (3.1):** `KmsProvider` (Trait in `crypto/wal_crypto.rs`, laut Scan unverdrahtet) wird verdrahtet, sein Besitz geklärt (SPEC §17 F-05); eine Null- oder Test-Implementierung allein macht `crypto` `experimental` (SPEC §7.5).

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
4. Verdrahtung nur in der Fassade (EP7); bestehende Saga-Reste in `engine`, `mcp`, `crypto` hierhin verlagern. **Schnittstellenvertrag und Pseudocode: SPEC §9.11 (E-1 bis E-7)**; `ErasureBuilder::build()` scheitert, wenn eine Schicht, in der Daten liegen können, weder Scanner noch dokumentierten Ausschluss hat (Fail-Closed, E-6); ein Profil ohne `erasure` behauptet nichts über Löschung (E-7).

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

### WP 5.10 – Rechtsmatrix (SPEC §9.12)

**Ziel:** Die gesetzlichen Vorgaben, die die Löschkette und ihre Nachbarn tragen, sind **maschinenlesbar** auf Baustein, Test und Status abgebildet; eine Rechtsänderung hat genau eine Eintrittsstelle.

**Aufgaben**

1. `docs/spec/legal-matrix.toml` aus SPEC §9.12 mit den Feldern `vorgabe`, `anforderung`, `baustein`, `test`, `status` (`umgesetzt`, `teilweise`, `offen`, `nicht-ziel`), `rechtsfrage` (Verweis O-01 bis O-06, O-11).
2. Gate `legal-matrix-consistency`: eine Zeile mit Status `umgesetzt` nennt einen existierenden Test; README und Berichte dürfen nichts als erfüllt ausweisen, was `offen` oder `teilweise` ist.
3. Externe Rechtsprüfung (O-01, O-02, O-05) liefert Eingaben; die Matrix ist **keine** Rechtsauskunft und keine Zertifizierung.
4. Änderungsverfahren: Rechtsänderung ⇒ Zeile, Ticket, betroffener Baustein, Test; Fristen und Legal-Hold-Kataloge bleiben **Konfiguration** (O-02).

**Abnahme:** Gate grün; jede Zeile hat Test oder ausgewiesenes Nicht-Ziel. **Aufwand:** S–M. **Voraussetzung:** 5.8 (für Fristzeilen), sonst Status `offen`.

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
|`baseline-all-targets`, `clippy-baseline`|neu|0.5, 0.6|
|`test-runtime-budget` (nightly)|neu|0.7|
|`measure-single-source`|neu|0.4|
|`undefined-cfg-feature`|neu|1.5|
|`test-hooks-private`|neu|1.6|
|`async-blocking-io`|neu|1.7|
|`determinism` (mit `disallowed_methods`)|vorhanden, schärfen|2.12|
|`duplicate-types`|neu|2.13|
|`file-size`, `unwired-pub` (rot ab Phase 6)|neu / vorhanden|3.8|
|`legal-matrix-consistency`|neu|5.10|
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
|T-BASE-1|`check`, `clippy -D warnings`, `test` je Crate **und** `--workspace --all-targets` grün, kein Timeout (ADR-D11)|0.5, 0.6, 0.7|
|T-DET-1|Zeit, Zufall, IDs nur über Ports; deterministische Wiederholung|2.12|
|T-ASYNC-1|Ein-Thread-Executor wird in Replay/Rebuild nicht über Budget blockiert|1.7|
|T-LEGAL-1|jede Zeile `umgesetzt` der Rechtsmatrix hat einen existierenden Test|5.10|
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
|E-0.5|`LayerCleanupProof`: Test an V5-Vertrag anpassen oder Typ im Vertrag führen?|Test anpassen / Typ einführen|Test anpassen; Typ nur, wenn der Löschbericht-Vertrag ihn braucht (genau eine Definition in `crypto`)|0.5|
|E-1.5|`store`: undefinierte Features `encryption-at-rest`, `wal-integrity`|Code entfernen / unbedingt kompilieren / Feature definieren|entfernen oder unbedingt kompilieren (A1); Feature nur mit Matrix-Test|1.5|
|E-2.14|Lizenzprüfung ohne harte Kante|Port `LicenseGate` in Fassade / Feature|Port `LicenseGate`; `license` nie Pflicht, nie harte Kante von Kern oder Fassade|2.14|
|E-3.8|Dokumentationssprache|`rustdoc` englisch + Leitfäden deutsch / alles deutsch|`rustdoc` englisch (Ökosystem), Integrations- und Betriebsleitfäden deutsch|3.8|

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
|R12|Drei Commit-Stände (Baseline, Scan, Klon) werden vermischt|Falsche Zahlen, nicht reproduzierbare Aussagen|Zielcommit festlegen (WP 0.1, O-12); rohe Logs archivieren; Zahlen nur mit Quelle und Commit|
|R13|Clippy-Rot wird per Sammel-`allow` oder gesenkter Strenge „behoben“|Verdeckte Defekte, Verstoß gegen EP16|Ursache zuerst (WP 0.6); Allow-Register; kein Absenken von `-D warnings`|
|R14|Timeouts werden als grün gelesen|Unentdeckte Testfehler in 13 Crates|Timeout = rot (ADR-D11); WP 0.7|
|R15|Zusammenführen doppelter Typen bricht Formate oder Löschbericht|Datenverlust, ungültige Berichte|Golden-Files V1–V5 vor Zusammenführung; ein Typ je PR (WP 2.13)|
|R16|Rechts- und Regulierungsänderungen (Aufsichtspraxis, KI-Verordnung, Cyber Resilience Act)|Bausteine veralten|Rechtsmatrix (WP 5.10); Fristen und Kataloge als Konfiguration; O-06, O-11|

---

## 15. Definition of Done je Phase

|Phase|Fertig, wenn|
|---|---|
|0|`evidence/baseline/` mit rohen Logs für alle 35 Mitglieder auf **einem** Zielcommit, auf zweitem Runner reproduziert; `--all-targets` grün; Clippy-Ursache belegt und behoben; kein Test-Timeout; eine Messquelle; `claims-consistency` grün; SPEC/PLAN 3.1 liegen unter `docs/`|
|1|Produktions-Panics = 0 in `text`, `wire`-Handcode; Allow-Register aktiv (≤ 20 befristete Einträge); Panic-Inventar getrennt; keine undefinierten `cfg(feature)`, keine öffentlichen Test-Hooks, kein blockierendes I/O in `async`|
|2|`no-cycles` ohne Ausnahme; `duplicate-types` und `determinism` ohne ungedeckte Stelle; `may-depend-on` ohne Ausnahme; Budgets aus SPEC §4.5 erreicht oder mit ADR begründet; Egress-Tabelle vom Gate bestätigt; `contextra-core` entfernt|
|3|`module-audit <crate>` für alle 33 Crates; Evidence-Pack und Reife ableitbar; Meta-Test der Gates grün|
|4|Crash-Matrix, Format-Register und `DurableLog` grün; `store`/`mvcc`/`checkpoint` isoliert auditierbar|
|5|Löschkette `beta` mit Chaos- und Mutationsnachweis; AC-Matrix vollständig; Rechtsmatrix vollständig und konsistent|
|6|Evidence-Pack E3 je Modul gemäß Zielreife|
|7|Vier Profile bestehen `profile-audit`; deutsche Anleitung und lauffähige Beispiele|
|8|Alle Zeilen aus SPEC §13.4 belegt; externer Review beauftragt (Ergebnis für `certified-ready`)|

---

## 16. Nächste Schritte (erste zwei Wochen)

**Woche 1 – Wahrheit und Sicherheit zuerst**

1. **0.1 (Beginn):** Zielcommit festlegen, Klon vertiefen, Rohlogs sichern (O-12). Ohne Logs bleiben Ursachenaussagen Hypothese.
2. **0.5:** die drei defekten Test-Targets reparieren (`db`, `crypto`, `mvcc`); danach `cargo check --workspace --all-targets --locked` grün. Kritischer Pfad, weil G2 (Löschbericht) sonst ungetestet bleibt.
3. **1.5 und TenantId-Teil von 2.13:** undefinierte `cfg(feature)` in `store` klären; `privacy`-Duplikat von `TenantId` entfernen (VETO-F10). Beide klein (S) und sicherheitsrelevant.
4. **0.6 Schritt 1:** Clippy-Rohlogs auswerten (nur Analyse, noch keine Änderungen).
5. **1.1 und 1.4:** Tokenizer-`panic!`, `wire`-Generat.

**Woche 2 – Messbarkeit**

6. **0.7:** Testlaufzeit je Crate messen; vollständigen Lauf ohne 45-s-Limit protokollieren.
7. **0.6 Behebung** nach Ursachengruppen; **1.6** Test-Hooks und Mocks; **1.7** blockierendes I/O (`store`, `vector`).
8. **0.2 und 0.3:** Berichte auf „Hypothese“, `capabilities.toml` auf Schema 3, SPEC/PLAN 3.1 und ADR-D01 bis ADR-D16 ablegen.
9. **0.4 und 2.1:** AST-Messbefehl und Kanten-/Budget-Gates mit Ist-Werten (Startpunkt für alle Entkopplungen).

**Bewusst nicht in den ersten zwei Wochen:** Löschkette (Phase 5) und Entkopplungs-Umbauten (Phase 2, außer den genannten Kleinstpaketen). Voraussetzung dafür sind grüne G2-Tests (0.5) und eine belegte Baseline (0.1).

---

## Anhang P1. Abbildung PLAN 2.5 → PLAN 3.1

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
|**3.1**|**28.10.2026**|Baseline (E2, teilweise), Metrik-Datei und Scan eingearbeitet (U-15 bis U-27); WP 0.5–0.7 (defekte Tests, Clippy-Ursache, Testlaufzeit), 1.5–1.7 (cfg-Features, Test-Hooks, blockierendes I/O), 2.12–2.14 (Clock, Doppeldefinitionen, Restkanten), 3.8–3.9 (Wartbarkeit, Schnittstudie), 5.10 (Rechtsmatrix); Leitplanken 7–10; Gate-Plan, Testmatrix, Entscheidungen E-0.5/E-1.5/E-2.14/E-3.8, Risiken R12–R16, Definition of Done und nächste Schritte angepasst|
