# Audit-Skript-Validierung & Befund-Auswertung (`scripts/contextra_full_audit.py`)

## 1. Executive Summary & Zielsetzung

Das Python-Audit-Skript `scripts/contextra_full_audit.py` (v6) dient der statischen und dynamischen Überprüfung der Contextra-Codebase. Die Ausführung im Protokoll `evidence/audit-20261101T184545Z-722d45aa8/summary.md` ergab:
- **FAIL**: 84
- **INFO**: 6
- **PASS**: 64
- **PENDING**: 3

Als Principal Senior Rust Entwickler wurde das Skript im Detail gegen die maßgebliche Zielarchitektur (`docs/SPEC.md` v2.0.0 / Stand Release 1.0.0) analysiert.
Ergebnis: **Das Skript enthält fundamentale Fehlannahmen und veraltete Modellannahmen aus einer früheren Phase vor dem Scope-Schnitt (Rückbau von 37 auf 14 Kern-Crates laut `docs/SPEC.md §5.1`).** Gleichzeitig deckt das Skript jedoch eine Reihe **hochkritischer, realer Defekte** im Quellcode auf.

---

## 2. Detaillierte Validierung des Audit-Skripts (`scripts/contextra_full_audit.py`)

### 2.1 Skript-Fehlannahmen (False Positives / Veraltete Soll-Mengen)

| Skript-Stelle | Konfiguration im Skript | Widerspruch zu `docs/SPEC.md` | Auswirkung im Audit-Lauf |
|---|---|---|---|
| Z. 42–56 | `BENCHES` listet Benches für `contextra-checkpoint`, `contextra-router`, `contextra-graph`, `contextra-infer-candle`, `contextra-bench`. | Laut `docs/SPEC.md §1.4, §3.1, §5.1` sind Checkpoints, Router, Graph, Candle und Benchmarks rückgebaut/archiviert (`archive/v0-full`). | Schritt `befund_bench_fuzz_dateien` schlägt fehl (FAIL); Benchmarks für diese Crates können nicht existieren. |
| Z. 57 | `FUZZ_CRATES = ["ingest", "mcp", "text", "db", "crypto", "vector", "store", "wire"]` | Laut `docs/SPEC.md §5.1` wurden `ingest`, `mcp`, `db` und `wire` vollständig entfernt. | Schritt `befund_bench_fuzz_dateien` und Fuzzing-Schritte melden FAIL wegen fehlender Fuzz-Targets entfernter Crates. |
| Z. 60–62 | `PRODUKT = ["contextra-types", "contextra-ports", "contextra-rank", "contextra-ingest", "contextra-privacy", "contextra-infer-ollama", "contextra-audit-export", "contextra-testkit", "contextra-avv-generator"]` | **Kardinaler Modellfehler im Skript:** Hier wird eine völlig veraltete Definition der "Produktmenge" zugrunde gelegt. Laut `docs/SPEC.md §5.1` umfasst der Kern 14 Crates (`types`, `ports`, `sys`, `simd`, `durable-fs`, `crypto`, `mvcc`, `store`, `text`, `vector`, `rank`, `erasure`, `engine`, `contextra`). Crates wie `ingest`, `privacy`, `infer-ollama`, `audit-export`, `avv-generator` sind explizit Nicht-Ziele (SPEC §1.4). | Schritt `produkt_huelle_ohne_kern` liefert FAIL (5 Crates mit Befund), weil diese Crates entweder gar nicht mehr existieren oder falsche Hüllen prüfen. |
| Z. 64–67 | `VERBOTEN_IM_PRODUKT` verbietet `engine`, `store`, `vector`, `text`, `mvcc` etc. | Diese Crates sind das **Herzstück** des Speicherkerns von Contextra v1! Das Skript geht fälschlicherweise davon aus, dass Contextra nur ein zustandsloser Wrapper ohne Speicher ist. | Völlige Fehlbewertung der Architektur im Schritt `produkt_huelle_ohne_kern`. |
| Z. 320–335 | `befund_maturity_claims` vergleicht `capabilities.toml` mit Mitgliedern | `capabilities.toml` enthält noch 21 Einträge von alten Crates (`contextra-agent`, `contextra-graph`, `contextra-router` etc.). | `befund_maturity_claims` meldet FAIL: `stable=15 beta=12 Eintraege ohne Crate=21`. Dies ist kein Code-Bug, sondern eine nicht synchronisierte Konfigurationsdatei. |

### 2.2 Korrekte und berechtigte Befunde des Audit-Skripts

Das Skript deckt jedoch substanzielle, reale Mängel in der bestehenden Codebasis auf:
1. **`befund_cfg_features_undefiniert` (FAIL):** Undefinierte Feature-Flags im Code (K1/K11), die durch unvorsichtiges `#![allow(unexpected_cfgs)]` maskiert werden.
2. **`befund_safe_fn_mit_unsafe_inhalt` (FAIL):** `pub fn` ohne `unsafe`-Signatur in `contextra-sys`, die intern `unsafe {}` nutzen (z. B. `mmap_readonly`, `reopen_and_dup2`).
3. **`befund_from_entropy` (FAIL):** Direkter Entropieaufruf im Produktivcode statt CSPRNG/Port-Abstraktion (K14).
4. **`fmt_check` & `doc` (FAIL):** Formatierungsfehler und defekte Rustdoc-Dokumentation (z. B. ungelöste Links / Warnungen als Fehler).
5. **Clippy-Deny-Lints (FAIL):** Massive Verstöße gegen Workspace-Lints (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`).
6. **Compile-Fehler in Tests:** Viele Integrations- und Unit-Tests kompilieren nicht mehr, weil sie alte Signaturen aufrufen oder auf gelöschte Crates zugreifen.

---

## 3. Klassifikation der 84 FAIL-Meldungen aus dem Audit-Protokoll

Die 84 Fehlschläge aus `audit-20261101T184545Z-722d45aa8` teilen sich in folgende Root-Cause-Kategorien:

| Kategorie | Betroffene Schritte (Anzahl) | Ursache | Bewertung & Behebung |
|---|---|---|---|
| **A. Skript-Artefakte / Veraltetes Soll-Modell** | ca. 22 Schritte (`befund_bench_fuzz_dateien`, `produkt_huelle_ohne_kern`, diverse `fuzz__*`, `bench__*` von entfernten Crates) | Das Skript testet Crates, die nach SPEC §5.1 gar nicht mehr im Workspace existieren (`contextra-checkpoint`, `router`, `graph`, `candle`, `bench`, `ingest`, `mcp`, `wire`). | **Skript korrigieren:** Bench- und Fuzz-Listen auf die 14 Kern-Crates synchronisieren. |
| **B. Fehlende Synchronisation der Governance-Metadaten** | 2 Schritte (`befund_maturity_claims`, `deny`) | `capabilities.toml` und `deny.toml` enthalten veraltete Crate-Namen und ungeprüfte Advisory-Ausnahmen. | Metadaten an den 14-Crate-Kern anpassen. |
| **C. Kompilierungsfehler in `contextra-engine` & Tests** | ca. 25 Schritte (`check_alle_targets`, `test_lib__*`, `test_integration__*`, `nextest__*`) | Verbliebene Referenzen auf gelöschte Typen/Methoden (`graph_index`, `relate`, `get_community`) in `engine` und Testsuites. Da `engine` und abhängige Tests nicht bauen, fallen alle Test-Schritte durch. | Verwaiste Graph- und Community-Aufrufe in `engine` und Testsuites entfernen bzw. bereinigen. |
| **D. Clippy- & Code-Hygiene-Verstöße** | ca. 20 Schritte (`clippy_lib__*`, `clippy_alle_targets__*`, `panic_flaeche_lib`) | Nichteinhaltung von P6 (keine Panics, keine unhandled Unwraps, striktes Indexing). | Systematischer Umbau auf typisierte `Result`/`Option`-Behandlung und sicheres Slicing (`.get()`). |
| **E. Statische Sicherheits-Gates (`spec_gates_statisch`)** | 8 Schritte (`S-SEC-2`, `S-SEC-3`, `S-SEC-4`, `S-SEC-5`, `S-SEC-7`, `S-SEC-8`, `S-PAN-1`, `S-LINT-1`) | Reale Sicherheitsmängel im Kerncode: Argon2id < 64 MiB, Verschlüsselung optional, Passphrase als String, verbotene Löschaussagen. | Ursachenbehebung in den jeweiligen Kern-Crates (siehe `ROOT_CAUSE_ANALYSIS.md`). |

---

## 4. Fazit zur Skript-Validierung

Das Audit-Skript `scripts/contextra_full_audit.py` ist als Prüfwerkzeug wertvoll, **darf aber nicht ungeprüft als absolute Wahrheit interpretiert werden**, da sein eigener Quelltext noch auf einem veralteten Architekturstand vor dem Scope-Schnitt basiert.
Zur Erreichung der Produktionsreife muss:
1. Das Skript auf das normative Daten- und Crate-Modell aus `docs/SPEC.md §5.1` aktualisiert werden (Entfernung von Geister-Crates in `BENCHES`, `FUZZ_CRATES`, Neudefinition von `PRODUKT`).
2. Die identifizierten realen Quellcode-Defekte (Kompilierfehler in `engine`, Sicherheitslücken in `crypto`/`store`/`mvcc`, Lints) müssen an ihrer Wurzel behoben werden.
