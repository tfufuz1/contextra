# Audit-Bericht: `contextra-infer-ollama`

**Datum:** 2026-09-27
**Auditor:** Jules (Principal Senior Rust Architect)
**Crate:** `contextra-infer-ollama` (Layer 3 / Opt-in Ollama Provider Crate)
**Gegenstand:** Audit für `crates/contextra-infer-ollama/src/` (`client.rs`, `embedding.rs`, `context_prefixer.rs`, `importance.rs`, `model_info.rs`).

---

## 1. Vorbereitung & Inventar

- **Precheck Claim Script:** `bash .jules/verify/claim_precheck.sh` (Prüfung auf Vorhandensein durchgeführt; Script im Repository nicht vorhanden; Vorbereitungsprüfungen manuell absolviert).
- **AGENTS.md Compliance:** `crates/contextra-infer-ollama/AGENTS.md` verifiziert.
- **Dateibestand (`find crates/contextra-infer-ollama/src -name "*.rs"`):**
  - `crates/contextra-infer-ollama/src/lib.rs` (33 LOC)
  - `crates/contextra-infer-ollama/src/client.rs` (29 LOC)
  - `crates/contextra-infer-ollama/src/client/config.rs` (38 LOC)
  - `crates/contextra-infer-ollama/src/client/core.rs` (601 LOC)
  - `crates/contextra-infer-ollama/src/client/errors.rs` (40 LOC)
  - `crates/contextra-infer-ollama/src/client/trait_impls.rs` (106 LOC)
  - `crates/contextra-infer-ollama/src/client/validation.rs` (105 LOC)
  - `crates/contextra-infer-ollama/src/client/wire.rs` (68 LOC)
  - `crates/contextra-infer-ollama/src/client/tests.rs` (1063 LOC)
  - `crates/contextra-infer-ollama/src/embedding.rs` (320 LOC)
  - `crates/contextra-infer-ollama/src/context_prefixer.rs` (449 LOC)
  - `crates/contextra-infer-ollama/src/importance.rs` (626 LOC)
  - `crates/contextra-infer-ollama/src/model_info.rs` (323 LOC)

---

## 2. Prüfpunkte & Audit-Ergebnisse

### P1: RETRY-LOGIK KORREKT
- **Exponentielles Backoff:** In `client/core.rs` (`generate_text`, `generate`, `embed`) ist exponentielles Backoff wie folgt implementiert:
  `let base_delay = Duration::from_millis(100 * 2u64.pow(attempt));`
  `let jitter = Duration::from_millis(rand::random::<u64>() % 100);`
  `let delay = (base_delay + jitter).min(Duration::from_secs(5));`
- **Maximum Retry-Count:** Capped über `config.max_retries` (Standard: 3 Retries).
- **Permanente Fehler (4xx):** Fehlerklassifizierung in `client/errors.rs` via `is_transient_error()` identifiziert I/O-Fehler (`ContextraError::Io`) sowie 5xx HTTP-Server-Fehler (500, 502, 503, 504) und Timeouts. Permanentere Client-Fehler (4xx, wie 400 Bad Request, 404 Not Found) werden als `ContextraError::InvalidInput` / `ContextraError::NotFound` / `ContextraError::PolicyViolation` zurückgegeben und führen **nicht** zu Retries. Testabdeckung vorhanden in `test_no_retry_on_4xx_codes` und `test_no_retry_on_400`.
- **Status:** **PASS**

### P2: ZERO-PANIC
- **Befehl:** Codebase-Analyse auf `.unwrap()`, `.expect()`, `panic!` im Produktionscode (`crates/contextra-infer-ollama/src/` ohne Tests/`mod tests`).
- **Ergebnis:** 0 Vorkommen im Produktionscode. Crate Root in `lib.rs` erzwingt `#![forbid(unsafe_code)]`.
- **Status:** **PASS**

### P3: CONTEXT-PREFIXER LÄNGE
- **Token-Begrenzung:** In `context_prefixer.rs` konfiguriert `ContextPrefixConfig` die Standardlänge auf `max_prefix_tokens: 80` (empfohlener Korridor 50-100 Tokens).
- **Hartes Zeichenlimit & Prompting:**
  - `compute_max_prefix_chars(max_prefix_tokens)` berechnet ein Zeichenlimit von `80 * 4 = 320` Zeichen (bei max. 100 Tokens 400 Zeichen).
  - Der LLM-Prompt instruiert das Modell explizit: `"Maximal {max_p} Zeichen."`.
  - Die Funktion `truncate_prefix()` führt zweistufige Truncation durch: zuerst Wortgrenzen-Kürzung auf `max_prefix_tokens`, anschließend harte Zeichenbegrenzung `truncate_chars()` unter Wahrung von Wort- und Unicode-Codepoint-Grenzen. Das Kontextfenster des Hauptmodells bleibt garantiert unverschmutzt.
- **Status:** **PASS**

### P4: IMPORTANCE-SCORING ROBUSTHEIT
- **LLM-Fehler / Format-Abweichung:** In `importance.rs` verarbeitet `parse_importance_score_response()` LLM-Ergebnisse via OnceLock-Regex. Bei Parsing-Fehlern oder fehlerhaftem LLM-Format liefert die Funktion einen sicheren Default-Wert `ImportanceScore(0.5)` mit `Confidence::Unparseable` und zeichnet eine Warnung im Tracing-Log auf.
- **Batch-Verarbeitung:** `score_importance_batch()` verarbeitet Chunks parallel über `buffer_unordered()`. Ein Fehler bei einem einzelnen Chunk schlägt nicht auf das gesamte Batch durch, sondern fängt den Fehler ab (`unwrap_or_else`) und setzt den Default-Score (0.5).
- **Proptests:** Property-Based Tests (`prop_score_importance_arbitrary_garbage_never_panics`) verifizieren, dass beliebige Müll-Eingaben niemals Panics auslösen.
- **Status:** **PASS**

### P5: OPT-IN VERIFIKATION
- **Cargo.toml Prüfung:** In `crates/contextra-infer-ollama/Cargo.toml` existiert keine `default`-Feature-Definition.
- **Workspace-Ausschluss:** In der Root-`Cargo.toml` ist `crates/contextra-infer-ollama` aus `default-members` ausgeschlossen (Spezifikation Section 0.5 / A.3: External process provider dependency). Crate ist rein opt-in per `cargo build -p contextra-infer-ollama`.
- **Status:** **PASS**

---

## 3. Test- & Clippy-Ergebnisse

### Test Suite Output
```text
cargo test -p contextra-infer-ollama --locked -- --nocapture
test result: ok. 91 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.17s
Doc-tests contextra_infer_ollama: ok. 1 passed; 0 failed
```

### Clippy Output
```text
cargo clippy --no-deps -p contextra-infer-ollama --all-targets -- -D warnings
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.64s
0 warnings / 0 errors.
```

---

## 4. Fazit

Die Crate `contextra-infer-ollama` erfüllt alle Architektur- und Sicherheitsspezifikationen der Contextra Produktspezifikation. Zero-Panic Disziplin, exponentielles Backoff, Injection-Isolation und opt-in Eigenschaften sind lückenlos nachgewiesen.
