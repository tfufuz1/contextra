# Principal Rust Engineer Leitfaden: Lokales Testen, Messen, Benchmarking & Auditieren

> **Status:** Referenzdokumentation für Ingenieure und KI-Agenten (zsh-optimiert)
> **Gültigkeit:** Workspace Contextra & alle 39 Crates unter `crates/`
> **Ziel:** Exaktes, feingranulares lokales Kompilieren, Testen, Benchmarking und Auditieren mit reproduzierbaren Shell-Befehlen, Crate-internen Tools und token-schonenden Ausgabeformaten.

---

## 1. Quick Start & Executive Summary (zsh)

### Umgebungsvoraussetzungen (zsh)
Stelle sicher, dass Rust (Nightly oder Stable gem. `rust-toolchain.toml`) und Python 3 vorhanden sind:
```zsh
# Rust Version & Toolchain prüfen
rustc --version
cargo --version

# Python 3 Version prüfen (Standardbibliothek ausreichend)
python3 --version

# Repository-Root als Basisvariable definieren
export CONTEXTRA_ROOT="$(pwd)"
```

### Die 3 goldenen Befehle für Crate-spezifisches Arbeiten
Jedes Crate in `crates/<crate>/` verfügt über einen lokalen Ordner `tools/runner.sh`:
```zsh
# 1. Schneller Test (feingranular):
./crates/contextra-crypto/tools/runner.sh test-unit aead

# 2. Token-schonende Zusammenfassung für KI-Prompts / Jules:
./crates/contextra-crypto/tools/runner.sh summary

# 3. Tiefenprüfung (Panic, Unsafe, Determinismus, Entkopplung):
./crates/contextra-crypto/tools/runner.sh audit
```

---

## 2. Workspace-weites Kompilieren, Testen & Prüfen

### 2.1 Workspace Compilation & Type-Checking
```zsh
# Schneller Syntax- und Typ-Check für den gesamten Workspace
cargo check --workspace

# Check ohne Default-Features
cargo check --workspace --no-default-features

# Check mit allen Features
cargo check --workspace --all-features

# Release-Build zur Validierung von Optimierungen und Linker-Stufen
cargo build --workspace --release
```

### 2.2 Workspace Test-Execution
```zsh
# Alle Tests im Workspace ausführen
cargo test --workspace

# Alle Tests mit allen aktivierten Features
cargo test --workspace --all-features

# Nur Dokumentationstests ausführen
cargo test --workspace --doc

# Tests sequentiell ausführen (bei geteilten Hardware-/I/O-Ressourcen)
cargo test --workspace -- --test-threads=1
```

### 2.3 Workspace Linting & Static Analysis
```zsh
# Strikter Clippy-Durchlauf (Treat Warnings as Errors)
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Formatierungs-Check
cargo fmt --all -- --check
```

---

## 3. Crate-spezifisches feingranulares Arbeiten (Cargo-Ebene)

### 3.1 Die 39 Crates im Überblick
Die Crates gliedern sich in Ringe / Bausteine:
- **Ring 0 (Foundations / Micro-Cores):** `contextra-ids`, `contextra-env`, `contextra-types`, `contextra-ports`, `contextra-sys`, `contextra-simd`, `contextra-wire`, `contextra-durable-fs`, `contextra-mvcc`, `contextra-crypto`
- **Ring 1 (Engine / Storage / Retrieval):** `contextra-store`, `contextra-text`, `contextra-vector`, `contextra-rank`
- **Ring 2 & 3 (Services, Cognition, Erase):** `contextra-erasure`, `contextra-vault`, `contextra-engine`, `contextra-db`, `contextra-graph`, `contextra-cognition`, `contextra-privacy`, `contextra-agent`, `contextra-router`, `contextra-kvcache`, `contextra-license`, `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx`, `contextra-prompt-guard`, `contextra-sandbox`, `contextra-ingest`, `contextra-checkpoint`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-adapt`, `contextra-mcp`, `contextra-py`
- **Ring 4 & Tooling:** `contextra`, `contextra-testkit`

### 3.2 Gezieltes Crate-Testing via Cargo
```zsh
# Syntax-Check eines spezifischen Crates
cargo check -p contextra-crypto

# Check mit expliziter Feature-Kombination
cargo check -p contextra-mvcc --no-default-features
cargo check -p contextra-mvcc --all-features

# Tests eines spezifischen Crates
cargo test -p contextra-crypto

# Nur Unit-Tests eines Crates (--lib)
cargo test -p contextra-crypto --lib

# Nur Integrations-Tests eines Crates
cargo test -p contextra-crypto --test '*'

# Spezifischen Integrations-Test binary ausführen
cargo test -p contextra-crypto --test kv_segment_concurrency

# Clippy für ein Crate isoliert
cargo clippy -p contextra-crypto -- -D warnings
```

---

## 4. Modul- und Funktions-Ebene: Feinstgranular testen & debuggen

### 4.1 Filterung nach Test-Name und Pfad
```zsh
# 1. Alle Tests ausführen, deren Name 'aead' enthält
cargo test -p contextra-crypto aead

# 2. Exakter Testname
cargo test -p contextra-crypto test_aead_seal_open_roundtrip -- --exact

# 3. Modul-Pfad eingrenzen (z. B. kv_segment)
cargo test -p contextra-crypto kv_segment::

# 4. Ausgabe von `println!` und Tracing im Test anzeigen (--nocapture)
cargo test -p contextra-crypto test_aead -- --nocapture

# 5. Fehlgeschlagene Tests sofort abbrechen (-- --fail-fast)
cargo test -p contextra-crypto -- --fail-fast
```

### 4.2 Ignorierte Tests ausführen
```zsh
# Nur ignorierte Tests ausführen (z. B. teure Stresstests)
cargo test -p contextra-crypto -- --ignored

# Sowohl reguläre als auch ignorierte Tests ausführen
cargo test -p contextra-crypto -- --include-ignored
```

---

## 5. Lokale Crate-Tools (`crates/<crate>/tools/runner.sh`)

In jedem Crate existiert das standardisierte Skript `crates/<crate>/tools/runner.sh`. Es abstrahiert wiederkehrende Crate-Befehle und koppelt lokale Aktionen direkt mit den Workspace-Diagnosetools.

### 5.1 Verfügbare Befehle
| Befehl | Zweck | Beispiel |
|---|---|---|
| `summary` | **Token-optimierte** Metriken (LOC, Panics, Unsafe, Determinismus) | `./crates/contextra-crypto/tools/runner.sh summary` |
| `check` | Schneller Syntax- & Typ-Check | `./crates/contextra-crypto/tools/runner.sh check` |
| `check-all-features` | Check mit allen Features | `./crates/contextra-crypto/tools/runner.sh check-all-features` |
| `check-no-default` | Check ohne Default-Features | `./crates/contextra-crypto/tools/runner.sh check-no-default` |
| `test [FILTER]` | Tests mit optionalem Namensfilter | `./crates/contextra-crypto/tools/runner.sh test kdf` |
| `test-unit [FILTER]` | Nur Unit-Tests (`--lib`) | `./crates/contextra-crypto/tools/runner.sh test-unit secret` |
| `test-doc` | Dokumentationstests | `./crates/contextra-crypto/tools/runner.sh test-doc` |
| `clippy` | Crate-spezifisches Clippy (`-D warnings`) | `./crates/contextra-crypto/tools/runner.sh clippy` |
| `bench [NAME]` | Benchmarks des Crates | `./crates/contextra-crypto/tools/runner.sh bench` |
| `audit` | Vollständiges JSON-Audit des Crates | `./crates/contextra-crypto/tools/runner.sh audit` |
| `loc` | LOC-Zählung des Crates | `./crates/contextra-crypto/tools/runner.sh loc` |

### 5.2 Direkte Ausführung aus dem Crate-Verzeichnis
```zsh
cd crates/contextra-mvcc
./tools/runner.sh check
./tools/runner.sh test
./tools/runner.sh summary
```

---

## 6. Workspace-Diagnosetools (`tools/`) im Detail

Die Python-Diagnosetools unter `tools/` analysieren ASTs, Code-Invarianten, Isolation und Entkopplung. Sie bieten stets Optionen zur Filterung (`--crate`) und Maschinenlesbarkeit (`--json`).

### 6.1 `tools/zero_panic_analyzer.py` (Zero-Panic & Invarianten)
Prüft auf unerlaubte `unwrap()`, `expect()`, `panic!()`, `todo!()`, `assert!()` im Produktionscode.
```zsh
# Gesamten Workspace prüfen
python3 tools/zero_panic_analyzer.py

# Feingranular auf ein Crate beschränken (JSON-Output)
python3 tools/zero_panic_analyzer.py --crate contextra-crypto --json

# Bei Produktions-Verstößen mit Exit-Code 1 abbrechen (für CI/Pre-Commit)
python3 tools/zero_panic_analyzer.py --crate contextra-crypto --fail-on-prod
```

### 6.2 `tools/unsafe_boundary_auditor.py` (Unsafe-Inseln & SAFETY-Docs)
Auditert `unsafe`-Blöcke, `#![forbid(unsafe_code)]` und prüft, ob jeder `unsafe`-Block mit `// SAFETY:` dokumentiert ist.
```zsh
# Alle Crates analysieren
python3 tools/unsafe_boundary_auditor.py

# Spezifisches Crate im JSON-Format
python3 tools/unsafe_boundary_auditor.py --crate contextra-simd --json
```

### 6.3 `tools/determinism_guard.py` (P28 Determinismus-Prüfung)
Erkennt Nicht-Determinismus im Produktionscode (z.B. direkte `Instant::now()`, `SystemTime::now()`, `rand::thread_rng()`, oder `HashMap`/`HashSet` ohne festen Hasher).
```zsh
# Gesamter Workspace
python3 tools/determinism_guard.py

# Crate-spezifisch mit JSON-Ausgabe
python3 tools/determinism_guard.py --crate contextra-privacy --json
```

### 6.4 `tools/decouple_inspector.py` (Entkopplung, Schnittstellen & Leaks)
Analysiert Crate-Kopplungen, exportierte Schnittstellen, Dead-Code-Kandidaten und DSGVO/Löschleaks.
```zsh
# Einzelnes Crate analysieren
python3 tools/decouple_inspector.py --crate contextra-text

# Nur nach Löschketten-/DSGVO-Relikten suchen (Erasing leaks)
python3 tools/decouple_inspector.py --crate contextra-graph --erasure-leak-check

# Detailliertes Schnittstelleninventar (Traits, Structs, Enums) als JSON
python3 tools/decouple_inspector.py --crate contextra-mvcc --interfaces --json

# Funktionsinventar anzeigen
python3 tools/decouple_inspector.py --crate contextra-crypto --functions --json
```

### 6.5 `tools/loc.py` (Code-Größen & Budget-Tracking)
Zählt produktive Rust-Zeilen (ohne Kommentare und Leerzeilen) und vergleicht mit Governance-Budgets.
```zsh
# Workspace-Gesamtauswertung
python3 tools/loc.py

# Crate-spezifisch als JSON
python3 tools/loc.py --crate contextra-store --json
```

### 6.6 `tools/isolation_proof.py` (G-L4 Autarkie-Beweis)
Validiert, dass ein Crate unabhängig von internen Monorepo-Verflechtungen baubar und testbar ist.
```zsh
# Isolation prüfen für no-default, default und individual features
python3 tools/isolation_proof.py --crate contextra-crypto --modes no-default,default
```

### 6.7 `tools/dead_code_pruner.py` & `tools/dead_code_microprobe.py`
Identifiziert ungenutzte Symbole und toten Code.
```zsh
# Crate analysieren
python3 tools/dead_code_pruner.py --crate contextra-store --json
```

### 6.8 Weitere Spezialwerkzeuge
- `tools/fsync_durability_tracer.py`: Prüft auf INV-FSYNC-1 Konformität (`python3 tools/fsync_durability_tracer.py --json`).
- `tools/error_topology_mapper.py`: Bildet Fehler-Enums ab und verhindert Datenlecks in Log-Strings (`python3 tools/error_topology_mapper.py --json`).
- `tools/port_contract_auditor.py`: Auditert M3/M7 Port-Verträge und API-Leckagen (`python3 tools/port_contract_auditor.py --json`).
- `tools/type_registry_sync.py`: Erkennt Typ-Kollisionen über Crate-Grenzen hinweg (`python3 tools/type_registry_sync.py --json`).
- `tools/async_boundary_scanner.py`: Scannt nach Halten von Locks über `.await`-Punkte (`python3 tools/async_boundary_scanner.py --json`).
- `tools/crate_coupling_report.py`: Erzeugt Abhängigkeitsmatrizen und Mermaid-Diagramme (`python3 tools/crate_coupling_report.py -o coupling.md --mermaid`).
- `tools/orchestrate_diagnostics.py`: Führt alle Diagnostiken sequentiell aus und erzeugt `docs/diagnostics/DIAGNOSTIC_SUMMARY.md`.

---

## 7. Benchmarks & Performance-Messung

### 7.1 Criterion Micro-Benchmarks
Zahlreiche Crates besitzen dedizierte Criterion-Benchmarks in `benches/`.
```zsh
# Alle Benchmarks eines Crates ausführen
cargo bench -p contextra-simd

# Spezifischen Benchmark filtern
cargo bench -p contextra-simd --bench bench_simd -- cosine_similarity

# Schneller Durchlauf ohne statistische Exzess-Iterationen (Quick run)
cargo bench -p contextra-simd -- --quick

# Baselines speichern und vergleichen
cargo bench -p contextra-simd -- --save-baseline before_optimization
# (nach Code-Änderung:)
cargo bench -p contextra-simd -- --baseline before_optimization
```

### 7.2 System- und Makro-Benchmarks (`benchmarks/`)
Im Verzeichnis `benchmarks/` liegen übergreifende Benchmark-Suites:
```zsh
# Kompetitiver Benchmark
cargo bench --bench competitive_bench

# Scale & Durchsatz Benchmark
cargo bench --bench scale_bench
```

---

## 8. Erweiterte Verifikation: Sanitizer, Loom & Coverage

### 8.1 Loom Concurrency Verification
Für deterministische Concurrency- und Race-Condition-Prüfungen:
```zsh
# Crate mit Loom-Konfiguration testen (z. B. contextra-mvcc)
RUSTFLAGS="--cfg loom" cargo test -p contextra-mvcc --test loom_snapshot_registry -- --nocapture
```

### 8.2 LLVM Source-Based Code Coverage (`cargo-llvm-cov`)
```zsh
# Crate-Coverage erfassen und Zusammenfassung im Terminal ausgeben
cargo llvm-cov -p contextra-crypto --summary-only

# Coverage als lcov.info zur Weitergabe exportieren
cargo llvm-cov -p contextra-crypto --lcov --output-path coverage.lcov
```

### 8.3 Miri (Undefined Behavior Detection)
```zsh
# Miri auf Core-Modulen ohne externe I/O ausführen
cargo miri test -p contextra-ids
cargo miri test -p contextra-env
```

---

## 9. Token-schonende Ausgabe-Rezepte für KI & Jules

Um Kontextfenster und Token zu sparen, sollten Test- und Audit-Ergebnisse komprimiert übermittelt werden.

### 9.1 Crate Summary (Empfohlen für Status-Updates)
Nutze direkt:
```zsh
./crates/contextra-crypto/tools/runner.sh summary
```
Ausgabebeispiel (nur ~4 Zeilen):
```text
=== [CRATE SUMMARY: contextra-crypto] ===
LOC: 4905 across 27 files
Panic: prod_unwrap=0, prod_expect=0, prod_assert=1
Unsafe: occurrences=0, forbid_unsafe=True
Determinism: total_prod_issues=0
=====================================
```

### 9.2 Nur fehlgeschlagene Tests übermitteln
Statt 1000 erfolgreichen Testzeilen nur die Failures erfassen:
```zsh
cargo test -p contextra-crypto 2>&1 | grep -E "FAILED|failures:|error\[" -A 10
```

### 9.3 Kompakte JSON-Extraktion mit `jq` oder Python
```zsh
# Zero-Panic Zusammenfassung in einer Zeile
python3 tools/zero_panic_analyzer.py --crate contextra-crypto --json | python3 -c '
import sys, json; data = json.load(sys.stdin)
print("Summary:", data[0].get("summary") if data else "N/A")
'
```

### 9.4 Kompakte Compilation-Errors
```zsh
# Nur Fehlermeldungen ohne Compiler-Warnungsrauschen
cargo check -p contextra-crypto --message-format=short 2>&1 | grep "error:"
```

---

## 10. Checkliste für den lokalen Entwicklungszyklus

1. **Vor der Änderung:**
   - `./crates/<crate>/tools/runner.sh summary` zur Baseline-Erfassung.
2. **Während der Änderung:**
   - `./crates/<crate>/tools/runner.sh test-unit <modul>` für sofortiges Feedback im TDD-Stil.
3. **Nach der Änderung:**
   - `./crates/<crate>/tools/runner.sh clippy`
   - `./crates/<crate>/tools/runner.sh audit`
   - `./crates/<crate>/tools/runner.sh summary`
4. **Ergebnisse teilen:**
   - Sende nur die kompakte `summary` oder gezielte Fehlerauszüge an Jules / KI-Agenten.
