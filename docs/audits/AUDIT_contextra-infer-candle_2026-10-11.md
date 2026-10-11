# Audit-Bericht: contextra-infer-candle & KV-Cache-Bridge

**Datum:** 2026-10-11
**Auditor:** Senior Rust ML-Engineer — Candle Inferenz, GGUF Quantisierung & KV-Cache-Bridge
**Crate:** `contextra-infer-candle` (Ring 2)
**Safety Status:** `#![forbid(unsafe_code)]` enforced

---

## 1. Zero-Panic-Inventar & Fallback-Doktrin (P1 & AGENTS.md §3)

Gemäß der Contextra Zero-Panic-Doktrin (P7 / AGENTS.md §6) und den Crate-Invarianten (AGENTS.md §3) wurden alle Produktionspfade überprüft.

### Codebase Scan Resultate
Analyse der Suchtreffer aus `grep -rn "\.unwrap()\|\.expect(\|panic!" crates/contextra-infer-candle/src/`:

| Dateipfad | Typ | Klassifikation | Begründung |
|---|---|---|---|
| `src/gguf_loader.rs` | `panic!` | Test-Code | Innerhalb `#[cfg(test)] mod tests` |
| `src/kv_bridge.rs` | `.unwrap()`, `.expect()` | Test-Code | Innerhalb `#[cfg(test)] mod tests` |
| `src/embedding.rs` | `.unwrap()`, `panic!` | Test-Code / Doc-Comments | Innerhalb `#[cfg(test)] mod tests` oder unwrap_or_else |
| `src/gasp.rs` | `.unwrap()`, `panic!` | Test-Code | Innerhalb `#[cfg(test)] mod tests` |

### Fallback bei fehlenden Modellgewicht-Dateien (AGENTS.md §3)
- **Behebung / Befund:** `CandleEmbedClient::from_dir` wurde angepasst, so dass bei fehlender `model.safetensors`-Datei in einem bestehenden Modellverzeichnis nicht mit einem `Err` abgebrochen wird, sondern – analog zu `CandleLlmClient::from_dir` – auf das Safe-Mock-Modell `DefaultCandleEmbedModel { dim: 384 }` zurückgegriffen wird.
- **Invariante verifiziert:** Alle Unittests und Integrationstests (u.a. `tests/j05-contextra-infer-candle_wiring_test.rs` und `tests/embedding_config_loading.rs`) wurden auf diesen Soll-Zustand hin verifiziert und bestanden.

---

## 2. KV-Cache-Bridge (Retrieval↔Inferenz) Tiefen-Audit (fail-open & fingerprint-isolation)

Das Modul `src/kv_bridge.rs` verbindet Retrieval-Chunks mit dem mandantenisolierten, verschlüsselten KV-Cache (RAM Tier 1 + LSM Tier 2 Spill via `KvBridgeStorage`).

### Prüf-Invarianten & Ergebnisse

1. **Fail-Open Invariante (`try_get_cached_segment` & `try_get_cached_segment_async`):**
   - Jede Entschlüsselungs-, Deserialisierungs- oder Store-Ausnahme fängt Fehler ab und gibt `None` zurück.
   - Ein Fehler oder Cache-Miss schlägt niemals als `Err` bis zum Aufrufer durch, sondern veranlasst transparent einen vollständigen Prefill der Inferenz.

2. **Fingerprint- & RoPE-Offset-Isolationsprüfung (`validate_payload`):**
   - `KvCacheKey` kapselt `chunk_id`, `ModelFingerprint` und `rope_offset`.
   - Bei Modellwechsel oder abweichendem `rope_offset` liefert `validate_payload` sofort `None`, um fehlerhafte Tensor-Injektionen zu verhindern.

3. **Mandanten-Separation (`test_tenant_isolation`):**
   - Segmente von Tenant A (`tenant_a`) können unter keinen Umständen von Tenant B (`tenant_b`) abgerufen werden.

4. **Lock Contention & Concurrency:**
   - Keine blockierenden Mutexes über `.await`-Punkte gehalten.
   - Inferenz-Execution nutzt Semaphoren-Backpressure (`DEFAULT_MAX_CONCURRENT_INFERENCES = 4`, `DEFAULT_MAX_CONCURRENT_EMBEDDINGS = 8`) und `tokio::task::spawn_blocking`.

---

## 3. VERDICT

```markdown
## VERDICT
- STATUS: APPROVED
- CRATE: contextra-infer-candle
- RING: 2
- UNSAFE: #![forbid(unsafe_code)]
- FEATURES VERIFIED: default, candle, kv-bridge, kv-stage-b
```
