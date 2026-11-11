# DECOUPLING & CONFORMANCE REPORT — CONTEXTRA-TESTKIT (WP-TK-1)

Stand: 2026-11-11
Rolle: Principal Rust Engineer

---

## 1. Gate-Matrix

| Gate | Geprüfter Vertrag / Bedingung | Exakter Befehl | Status |
|---|---|---|---|
| G-B1 | Standalone-Kompilierung (mit & ohne Default-Features) | `cargo check -p contextra-testkit --all-targets --offline && cargo check -p contextra-testkit --no-default-features --offline` | Grün |
| G-B2 | Unit-, Integration- und Doc-Tests | `cargo test -p contextra-testkit --offline && cargo test -p contextra-testkit --all-features --offline && cargo test -p contextra-testkit --doc --offline` | Grün (88 Tests bestanden) |
| G-B3 | Clippy (Zero Warnings) | `cargo clippy -p contextra-testkit --all-targets --no-deps --offline -- -D warnings` | Grün (0 Warnungen) |
| G-L1 | Layering / Keine Zyklen | `cargo tree -p contextra-testkit --offline --prefix none \| grep '^contextra-' \| sort -u` | Grün: nur `ports`, `types`, `ids`, `env` (kein `kvcache`, kein `router`) |
| G-D  | Determinismus (keine Systemzeit, kein Zufall, kein HashMap) | `grep -rEn 'Instant::now\|SystemTime::now\|HashMap\|thread_rng\|rand::' crates/contextra-testkit/src/conformance/kv_ports.rs crates/contextra-testkit/src/conformance/routing_ports.rs` | Grün (leer) |
| G-N  | Negativtests (Mutanten-Erkennung) | Jede der 8 Suiten verfügt über mindestens einen negativen Mutanten-Testfall, der zuverlässig als `Err(ConformanceFailure)` abgewiesen wird | Grün |
| G-S  | Scope-Beschränkung (nur `crates/contextra-testkit/**`) | `git status -s` zeigt ausschließlich `crates/contextra-testkit/**` | Grün |

---

## 2. Baseline- und Messwerte (WP-TK-1.0 & WP-TK-1.5)

- **Main-Zustand (Gate-0):**
  - Defekt A (`contextra-infer-candle/Cargo.toml` thiserror-Duplikat): Behoben (1 Zeile).
  - Defekt B (`contextra-ports/src/lib.rs` KeyMaterialSource-Duplikat): Behoben (1 Zeile).
  - Kein lokales Patch-Overlay erforderlich.
- **Compiler & Toolchain:**
  - `cargo 1.89.0`
  - `python 3.12.13`
- **Testzahlen Vorher vs. Nachher:**
  - `src/lib.rs`: 16 -> 16 Tests
  - `tests/conformance_selftest.rs`: 16 -> 32 Tests (+16 Konformitäts- & Mutantentests)
  - `tests/in_memory_doubles.rs`: 32 -> 40 Tests (+8 Fake-Integrationstests)
  - Doc-Tests: 2 ignoriert (bestehend)
  - **Gesamt:** 64 Tests (vorher) -> 88 Tests (nachher, +24 neue Tests)
- **Erstellte & erweiterte Typen:**
  - `conformance/kv_ports.rs`: `run_kv_prefix_store_suite`, `run_kv_bridge_storage_suite`, `run_attention_exporter_suite`, `assert_*_conformance`.
  - `conformance/routing_ports.rs`: `run_hybrid_search_provider_suite`, `run_community_resolver_suite`, `run_context_preparer_suite`, `run_drift_status_provider_suite`, `run_egress_policy_suite`, `assert_*_conformance`.
  - `mocks.rs`: `InMemoryKvPrefixStore`, `InMemoryKvBridgeStorage`, `ScriptedAttentionExporter`, `ScriptedHybridSearchProvider`, `StaticCommunityResolver`, `BudgetTruncatingContextPreparer`, `StaticDriftStatusProvider`, `ScriptedEgressPolicy`.
  - `lib.rs`: Re-Exports unter `doubles` und flach.

---

## 3. Vertragsmatrix & Analyse der Ports (WP-TK-1.1)

### 3.1 KV-Ports

| Port | Methode(n) | Dokumentierte Vor- & Nachbedingungen | Konformitätsprüfungen (Suite) |
|---|---|---|---|
| `KvPrefixStore` (`kv.rs:151`) | `lookup(tenant, key, tokens) -> Option<KvPrefixHit>`<br>`insert(tenant, key, tokens, blocks) -> Result<()>`<br>`evict(tenant, key) -> Result<u64>` | - `lookup`: Liefert längsten exakten Token-Match; `None` falls kein Match.<br>- Mandanten- und Modell-Isolation: Kein Cross-Tenant oder Cross-Model Hit.<br>- `insert`: Speichert Blöcke für gegebenes Präfix; idempotente Aktualisierung / Overwrite.<br>- `evict`: Entfernt Einträge für Mandant & Key, liefert Anzahl entfernter Blöcke, idempotent.<br>- Determinismus: Wiederholte Aufrufe mit gleichem State liefern identisches Ergebnis. | 1. `roundtrip_and_exact_match`<br>2. `isolation_tenant_and_key`<br>3. `overwrite_updates_data`<br>4. `evict_clears_data`<br>5. `deterministic_repeated_lookup` |
| `KvBridgeStorage` (`kv_bridge_port.rs:14`) | `get(key) -> BoxFuture<Result<Option<Bytes>>>`<br>`put(tx_id, key, value) -> BoxFuture<Result<()>>`<br>`commit(tx_id) -> BoxFuture<Result<()>>` | - `get`: Unbekannter Schlüssel liefert `Ok(None)`.<br>- Transaktionsisolation: Uncommitted `put` ist vor `commit` für Leser nicht sichtbar (oder gemäß Engine-Isolation); `commit` macht Änderungen dauerhaft sichtbar.<br>- Multi-Tx-Isolation: Mehrere Transaktionen schreiben isoliert voneinander.<br>- `put` überschreibt bestehenden Wert innerhalb der Tx. | 1. `get_empty_key`<br>2. `uncommitted_put_invisible`<br>3. `commit_makes_visible`<br>4. `multi_tx_isolation`<br>5. `overwrite_key` |
| `AttentionExporter` (`attention.rs:11`) | `export_attention_weights(RequestId) -> Option<Vec<f32>>` | - Unbekannte `RequestId` liefert `None`.<br>- Bekannte `RequestId` liefert `Some(Vec<f32>)`.<br>- Wiederholter Aufruf für dieselbe RequestId liefert identische Vektoren gleicher Länge. | 1. `unknown_request_none`<br>2. `known_request_weights`<br>3. `deterministic_repeated_export` |

### 3.2 Routing-Ports

| Port | Methode(n) | Dokumentierte Vor- & Nachbedingungen | Konformitätsprüfungen (Suite) |
|---|---|---|---|
| `HybridSearchProvider` (`vector_index.rs:228`) | `search_hybrid(query_text, query_embedding, top_k) -> BoxFuture<Result<Vec<ContextChunk>>>` | - `top_k == 0` liefert leere Ergebnisliste.<br>- Ergebnisliste hat höchstens `top_k` Chunks (`len <= top_k`).<br>- Deterministische Reihenfolge bei identischem Input (Tie-Break nach `DocId` aufsteigend).<br>- Leere Query (`query_text = ""`) führt nicht zu einer Panic. | 1. `zero_top_k_empty`<br>2. `top_k_respected`<br>3. `deterministic_tie_breaking`<br>4. `empty_query_no_panic` |
| `CommunityResolver` (`graph_index.rs:325`) | `get_community(EntityId) -> BoxFuture<Result<Option<u64>>>` | - Unbekannte Entity liefert `Ok(None)`.<br>- Bekannte Entity liefert `Ok(Some(community_id))`.<br>- Stabile / deterministische Antwort bei wiederholter Abfrage. | 1. `unknown_entity_none`<br>2. `known_entity_some`<br>3. `stable_repeated_resolution` |
| `ContextPreparer` (`lifecycle.rs:60`) | `prepare_context(chunks, budget, relevance_threshold) -> Result<ContextWindow>` | - Aggregierte Tokens (`total_tokens`) im zurückgegebenen `ContextWindow` dürfen das `TokenBudget` nicht überschreiten (`<= budget.available()` bzw. `effective_limit`).<br>- Chunks mit `relevance < relevance_threshold` werden herausgefiltert.<br>- Leere Eingabe (`chunks.is_empty()`) liefert ein leeres `ContextWindow` (`chunks.is_empty()`, `total_tokens == 0`). | 1. `empty_input_empty_window`<br>2. `relevance_threshold_filtering`<br>3. `budget_never_exceeded` |
| `DriftStatusProvider` (`observability.rs:10`) | `overall_drift_status() -> String` | - Rückgabewert ist exakt einer der 4 dokumentierten Strings: `"stabil"`, `"warnung"`, `"kritisch"`, `"unbekannt"`. | 1. `valid_status_enum_values` |
| `EgressPolicy` (`policy.rs:28`) | `evaluate(ctx, payload) -> EgressDecision` | - Jede Entscheidung ist eine der drei Varianten: `Allow`, `Redact`, `Deny`.<br>- `Redact(reason)` und `Deny(reason)` tragen eine nicht-leere Begründung (`!reason.trim().is_empty()`).<br>- Determinismus: Gleiche Eingabe (`ctx`, `payload`) liefert gleiche Entscheidung. | 1. `valid_decision_variants`<br>2. `non_empty_reasons_on_deny_and_redact`<br>3. `deterministic_evaluation` |

---

## 4. Getroffene Architekturentscheidungen

- **Keine Zyklen:** Das Testkit bindet weder `contextra-kvcache` noch `contextra-router` ein. Alle Suiten sind generisch über die Trait-Definitionen aus `contextra-ports`.
- **Deterministische Datenstrukturen:** Verbot von `std::collections::HashMap` und `HashSet` in Suiten und neuen Fakes; stattdessen Einsatz von `std::collections::BTreeMap` / `BTreeSet` und `contextra_types::DocId` aufsteigendem Tie-Break.
- **Fail-Closed Result-Pattern:** Suiten geben `ConformanceResult` zurück (`Result<(), ConformanceFailure>`), ohne selbst bei Testfehlschlägen unkontrolliert zu panicken. Dadurch können Adoptierende und Negativtests (Mutanten) Fehler deterministisch abfangen.
- **SPEC R2 / INV-EMBED-1 Beachtung:** Deutlicher Hinweis im README und Moduldokumentation, dass Fakes und Testkit-Mocks ausschließlich für `[dev-dependencies]` bestimmt sind und keinesfalls in Produktionscode eingebunden werden dürfen.

---

## 5. Übergaben an andere Eigentümer & Vertragslücken

### 5.1 Vertragslücken in `contextra-ports` (an den ports-Eigentümer)
1. **`crates/contextra-ports/src/kv.rs:158` (`KvPrefixStore`):**
   - *Lücke:* Es ist nicht spezifiziert, wie `tokens.is_empty()` behandelt werden soll (Match mit 0 Tokens oder Ablehnung via `None`/`Err`).
   - *Empfehlung:* Dokumentation ergänzen: `tokens.is_empty() -> None`.
2. **`crates/contextra-ports/src/kv_bridge_port.rs:14` (`KvBridgeStorage`):**
   - *Lücke:* Trait spezifiziert kein Rollback-Interface (`rollback(tx_id)`). Transaktionen können committet werden, aber explizites Verwerfen ohne Commit ist nicht Teil des Traits (nur implizit durch Nicht-Aufruf von `commit`).
   - *Empfehlung:* Dokumentieren, dass uncommittete Transaktionen beim Verwerfen des Handles oder nach Timeout verworfen werden.
3. **`crates/contextra-ports/src/observability.rs:14` (`DriftStatusProvider`):**
   - *Lücke:* Rückgabetyp ist ein freier `String` statt eines typsicheren Enums (`enum DriftStatus { Stabil, Warnung, Kritisch, Unbekannt }`).
   - *Empfehlung:* Einführung eines Enums `DriftStatus` in `contextra-types` oder `contextra-ports`, um String-Tippfehler zur Compile-Zeit auszuschließen.
4. **`crates/contextra-ports/src/vector_index.rs:230` (`HybridSearchProvider`):**
   - *Lücke:* Verhalten bei negativen oder `NaN`-Werten in `query_embedding` ist für `search_hybrid` nicht dokumentiert.
   - *Empfehlung:* Explizite `INV-HYBRID-NAN`-Validierung wie in `VectorIndex` vorschreiben.

### 5.2 Übergaben an P04 (kvcache & router Verdrahtung)
Folgende Konformitätssuiten aus `contextra-testkit::conformance` stehen zur Verdrahtung bereit:
- **Für `contextra-kvcache`:**
  - `run_kv_prefix_store_suite` / `assert_kv_prefix_store_conformance`
  - `run_kv_bridge_storage_suite` / `assert_kv_bridge_storage_conformance`
  - `run_attention_exporter_suite` / `assert_attention_exporter_conformance`
- **Für `contextra-router`:**
  - `run_hybrid_search_provider_suite` / `assert_hybrid_search_provider_conformance`
  - `run_community_resolver_suite` / `assert_community_resolver_conformance`
  - `run_context_preparer_suite` / `assert_context_preparer_conformance`
  - `run_drift_status_provider_suite` / `assert_drift_status_provider_conformance`
  - `run_egress_policy_suite` / `assert_egress_policy_conformance`
