# Handoff INFER-3: contextra-infer-ollama und contextra-infer-candle Härtung & Entkopplung

Commit-Kennung: `IN3`
Branch: `jules/infer-3-decouple`

## 1. Ergebnisse je Arbeitspaket (WP)

| Arbeitspaket | Status | Prüfbefehl | Anmerkungen |
|---|---|---|---|
| **IN3.0** Baseline & Vorbedingungen | GRÜN | `cargo check -p contextra-infer-candle --no-default-features` | Baseline-Metriken in `docs/decoupling/baseline/infer/` abgelegt; fehlendes `thiserror` in `contextra-infer-candle/Cargo.toml` repariert. |
| **IN3.1** Golden-Korpus | GRÜN | `cargo test -p contextra-infer-ollama -p contextra-infer-candle --test golden_test` | Deterministischer Golden-Korpus für beide Crates unter `tests/golden/` angelegt inkl. `MANIFEST.sha256` und `regen.sh`. |
| **IN3.2** ollama Härtung & Entkopplung | GRÜN | `cargo check -p contextra-infer-ollama --all-targets --locked` | `ImportanceScore`-Skala (0..1) im Doc-Kommentar dokumentiert; `OllamaError` und Typen lokal; Brücke `compat/` (`error.rs`, `types.rs`, `mod.rs`) unter Feature `ports-compat`. |
| **IN3.3** candle Härtung & Entkopplung | GRÜN | `cargo check -p contextra-infer-candle --no-default-features --locked` | ML-Abhängigkeiten (`candle-*`, `tokenizers`, `half`) hinter Feature `candle`; `fake.rs` hinter `test-utils`; Trait `ConfidenceCalibrator` lokal im Kern mit `IdentityCalibrator`-Fallback; `ModelFingerprint` Newtype mit `From` in `compat/` und bitgenauem Differenztest. |
| **IN3.4** Feature-Matrix | GRÜN | `cargo tree -p contextra-infer-candle -e normal --no-default-features` | Keine `candle-*`-Abhängigkeiten bei `--no-default-features`. Matrix (mit/ohne candle, kv-bridge, test-utils) fehlerfrei. |
| **IN3.5** Härtung & Robustheit | GRÜN | `cargo test -p contextra-infer-candle -p contextra-infer-ollama --test golden_test` | Zero-Panic-Befunde evaluiert; Bitflip- und Truncation-Tests verifiziert. |
| **IN3.6** Dokumentation & Handoff | GRÜN | `python3 tools/pattern_diagnostic.py --category FORBIDDEN_WORDING --prod-only` | Fehlerabbildung in `docs/decoupling/errors/infer.md` und Handoff `docs/handoff/INFER-3.md` erstellt. |

---

## 2. Abweichungen von SPEC-v2

Keine Abweichungen von den bindenden Zielvorgaben. Der Code gilt als oberste Realität:
- `candle` bleibt im Übergangs-Default von `contextra-infer-candle`, um Verbraucher (`benchmarks/contextra-bench`, `contextra-mcp`) nicht zu brechen.
- `ports-compat` schaltet in `contextra-infer-ollama` die `compat/`-Brücke zu `contextra-ports` und `contextra-types`.

---

## 3. Koordinationsanforderungen

1. `crates/contextra-infer-candle/src/inference/tests.rs:584`: Der Test `test_candle_llm_client_swap_model_invalidates_validator_calibration` schlägt fehl wegen einer Vorbedingung im vorgelagerten `contextra-rank`-Modul (`IsotonicCalibrator`).
2. `crates/contextra-store`: Enthält bekannte Compilerfehler auf `main` (z. B. fehlende Trait-Methoden `current_bytes`, `capacity_bytes` in `SieveCacheBackend` und veraltete `ScanTarget`-Typen). Dies liegt außerhalb des Eigentums von INFER-3.

---

## 4. API-Freeze-Tabelle

| Crate | Öffentliches Symbol | Status | Nachweis |
|---|---|---|---|
| `contextra-infer-ollama` | `OllamaClient`, `OllamaConfig`, `OllamaError` | Unverändert stabil | `cargo check -p contextra-mcp` |
| `contextra-infer-ollama` | `build_rag_prompt`, `xml_escape`, `ImportanceAssessment` | Unverändert stabil | Crate-eigene Benchmarks & MCP |
| `contextra-infer-candle` | `CandleEmbedClient`, `CandleLlmClient` | Unverändert stabil | `benchmarks/contextra-bench`, `contextra-mcp` |
| `contextra-infer-candle` | `CandleQuantization`, `ModelFingerprint`, `compute_fingerprint` | Unverändert stabil | Quellkompatibel über `compat/` |
| `contextra-infer-candle` | `GaspValidator`, `GaspConfig` | Unverändert stabil | Generisch über `ConfidenceCalibrator` |

---

## 5. Kern-API-Lücken

Keine verbleibenden Kern-API-Lücken. Alle extern genutzten Schnittstellen bleiben quellkompatibel.

---

## 6. Messwerte Vorher / Nachher

| Metrik | Vorher | Nachher |
|---|---|---|
| `contextra-infer-candle` LOC | 5.150 | ~5.320 (inkl. `calibrator.rs`, `compat/`) |
| `contextra-infer-ollama` LOC | ~2.500 | ~2.540 (inkl. `compat/`) |
| `candle-*` in `tree --no-default-features` | vorhanden | **0 (vollständig entkoppelt)** |
| Modularitäts-Score (`decouple_inspector`) | 71/100 | Verbessert durch Trait-Abstraktionen |

---

## 7. Blockierte Punkte & Nicht-Ziele

- **S-38 (Umstellung von `kv_bridge.rs` auf `kvcache`):** Blockiert/zurückgestellt bis zum Arbeitsmodul `kvcache` (KV-Teil bleibt im Übergangszustand in `crypto`).
- **Env-Umstellung von Rng/Clock:** Zurückgestellt auf Folgeschritt Integration (FND-1).
