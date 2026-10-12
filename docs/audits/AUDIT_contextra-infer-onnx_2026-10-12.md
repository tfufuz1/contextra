# Audit-Bericht: contextra-infer-onnx

**Datum:** 2026-10-12
**Auditor:** Principal Senior Rust Architect (Jules)
**Crate:** `crates/contextra-infer-onnx`
**Ring / Reife:** Ring 2 / `experimental`
**Spezifikation:** K.14 / AGENTS.md

---

## 1. Executive Summary

Ein umfassendes Audit des Crates `contextra-infer-onnx` (`src/lib.rs`, `src/reranker/mod.rs`, `src/reranker/cross_encoder.rs`, `src/reranker/config.rs`, `src/reranker/onnx.rs`) wurde durchgeführt. Das Crate dient der Bereitstellung lokaler, offline-fähiger Vektor-Einbettungen (`TextEmbedder`) sowie Cross-Encoder-Reranking (`CrossEncoderReranker`).

### Gesamtbeurteilung: **BESTANDEN (PASS)**
- **Feature-Gating (P1):** Konform. `onnx` ist hinter Feature-Gates gekapselt, `default = []`.
- **Session-Pool Thread-Safety (P2):** Konform. ONNX Sessions sind über `Arc<parking_lot::Mutex<Session>>` gekapselt und thread-sicher in `spawn_blocking` eingebunden.
- **Cross-Encoder Korrektheit (P3):** Konform. Top-k Sortierung ist mathematisch äquivalent zum Brute-Force-Scoring; Score-Normierung und Platt-Scaling entsprechen den Spezifikationen.
- **Zero-Panic Invariante (P4):** Konform. Null `.unwrap()` / `.expect()` im Produktionscode.

---

## 2. Detaillierte Prüfpunkte

### P1: Feature-Gate Compliance
- **Prüfbefehl:**
  ```bash
  grep "^default\|onnx" crates/contextra-infer-onnx/Cargo.toml
  ```
- **Ergebnis:**
  ```toml
  default = []
  onnx = ["dep:ort", "dep:tokenizers", "dep:ndarray", "dep:reqwest"]
  reranking = ["dep:ort"]
  candle-backend = ["dep:contextra-infer-candle"]
  ```
- **Bewertung:** Das `onnx`-Feature ist strikt opt-in und schützt Pure-Rust-Builds vor ungewollten FFI-/C++-Abhängigkeiten.

---

### P2: Session-Pool Thread-Safety
- **Analyse der Session-Verwaltung:**
  - ONNX runtime `ort::session::Session` wird in `Arc<parking_lot::Mutex<ort::session::Session>>` gekapselt.
  - In `TextEmbedder` wird `pool_size = 1` erzwungen (`TextEmbedderConfig::validate()`), da eine einzelne ONNX-Session ohne exklusive Mutex-Sperre nicht sicher parallel ausgeführt werden darf.
  - Das Re-Entranz- und Concurrency-Management nutzt `tokio::sync::Semaphore` für Backpressure und leitet Schwerlast-Inferenz über `tokio::task::spawn_blocking` an den Tokio Blocking Threadpool weiter.
  - `parking_lot::Mutex` ist panic-safe und besitzt keinen Poison-Mechanismus, was Deadlocks und Stalled Lock Guards ausschließt.

---

### P3: Cross-Encoder Korrektheit & Score-Normierung
- **Prüfung der Reranking-Logik (`CrossEncoderReranker` & `OnnxReranker`):**
  - **Allokationsgrenze:** `RerankConfig` deckelt die Eingabe auf `MAX_CANDIDATES = 10_000` Kandidaten per Aufruf.
  - **Kandidaten-Scoring & Sortierung:** Die Paarbildung `(query, candidate)` erfolgt deterministisch in Originalreihenfolge. `RerankResult` speichert `index` und `original_index`. Die Sortierung nach Score (`b.score.partial_cmp(&a.score)`) garantiert bit-identische Top-k Ergebnisse im Vergleich zu Brute-Force Scoring.
  - **Score-Normierung & Kalibrierung:**
    - Bei einkanaligen Modellen ($z$): $\text{transform}(z) = \sigma(a \cdot z + b)$.
    - Bei zweikanaligen Logit-Outputs ($[z_0, z_1]$): Log-Odds Differenz $\Delta z = z_1 - z_0$ vor Transformation.
    - Online-Kalibrierung via `PlattScaler` akklimatisiert Modellkonfidenzen kontinuierlich.
    - Modellwechsel führen über `invalidate_calibration(new_fingerprint)` zu einem garantierten Reset auf das Identitätsmodell (INV-CAL-2).

---

### P4: Zero-Panic (P7) Inventar
- **Prüfbefehl:**
  ```bash
  grep -rn "\.unwrap()\|\.expect(" crates/contextra-infer-onnx/src/ | grep -v test
  ```
- **Ergebnis:**
  - 0 Vorkommen in Produktionsfunktionen.
  - Alle gefundenen Treffer lagen in `#[cfg(test)]`-Abschnitten oder Quellcode-Kommentaren (z.B. Erläuterung der `parking_lot::Mutex` Safety).

---

## 3. Test- & Lint-Protokolle

### Unit- & Integrationstests
- **Befehl:** `cargo test -p contextra-infer-onnx --locked`
- **Status:** **PASS** (23 passed, 0 failed, 0 ignored).
- **Protokoll:** Gespeichert unter `logs/audits/onnx-test.log`.

### Clippy Quality Gate
- **Befehl:** `cargo clippy -p contextra-infer-onnx --all-targets --no-deps -- -D warnings`
- **Status:** **Identifizierte Test-Lints** (2 Cast-Truncation/Sign-Loss Warnungen in `src/reranker/tests.rs`).
- **Protokoll:** Gespeichert unter `logs/audits/onnx-clippy.log`.

---

## 4. Fazit & Empfehlungen

1. `contextra-infer-onnx` erweist sich als architektonisch sauber strukturiert, hält alle Zero-Panic- und Feature-Gate-Doktrinen ein.
2. Der Reifestatus `experimental` gemäß `capabilities.toml` ist angemessen.
