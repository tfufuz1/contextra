# P0 Befundbericht: Workspace-Resolution & Dependency-Audit

**PR-Status:** Befund, nicht abgeschlossen

---

## 1. Erstbefund `cargo metadata --locked`

**Befehl:** `cargo metadata --locked --format-version 1`
**Exit-Code vor Behebung:** 101
**Exit-Code nach Behebung:** 0

### Ursprüngliche Fehlermeldung
```
error: failed to load manifest for workspace member `/app/xtask-heavy`
referenced by workspace at `/app/Cargo.toml`

Caused by:
  failed to load manifest for dependency `contextra-router`

Caused by:
  failed to parse manifest at `/app/crates/contextra-router/Cargo.toml`

Caused by:
  error inheriting `contextra-adapt` from workspace root manifest's `workspace.dependencies.contextra-adapt`

Caused by:
  `dependency.contextra-adapt` was not found in `workspace.dependencies`
```

### Ursachenanalyse & durchgeführte Manifest-Anpassungen
1. **Wurzel `Cargo.toml`:**
   - `"xtask-heavy"` wurde aus `[workspace] members` und `[workspace] default-members` entfernt.
   - `xtask-heavy` hing von `contextra-router` ab (kein Workspace-Mitglied), welches wiederum via `workspace = true` nicht in der Wurzel-`Cargo.toml` deklarierte Abhängigkeiten (`contextra-adapt`, `contextra-graph`) erbte.
2. **`crates/contextra-vector/Cargo.toml`:**
   - Nicht-existierender Benchmark-Eintrag `flush_threshold_amplification` wurde entfernt.
3. **`crates/contextra-engine/Cargo.toml`:**
   - Redundante/nicht-existierende Feature-Anforderung `experimental-diskann` unter `dev-dependencies` entfernt.
4. **`Cargo.lock`:**
   - Lockfile an die geänderten Workspace-Manifeste angepasst.

---

## 2. Quelltext-Importe von Nicht-Workspace-Mitgliedern (Aufgabe d)

Die folgenden Stellen in Workspace-Mitgliedern importieren Quelltext aus Nicht-Workspace-Mitgliedern. Gemäß Aufgabe d wurde der Quelltext **NICHT** gelöscht:

### `xtask-heavy` -> `contextra-router`
- **Datei:** `xtask-heavy/src/check_bandit_latency_budget.rs`
- **Zeile 4:** `use contextra_router::BanditProfileState;` (Importierte Struktur: `BanditProfileState`)
- **Zeile 104:** `contextra_router::BanditImplementation::DiagonalApproximation` (Importiertes Enum-Variant: `BanditImplementation::DiagonalApproximation`)

---

## 3. Verbleibende Befunde `cargo check --locked -p contextra-mcp`

**Befehl:** `cargo check --locked -p contextra-mcp`
**Exit-Code:** 101

### Ursache
`contextra-mcp` hängt ab von `contextra`, welches transitiv von `contextra-engine` abhängt.
`contextra-engine` weist nach Auslagerung von Graph-Komponenten aus dem Workspace 40 Kompilierungsfehler auf:

1. **Fehlende Graph-Felder auf `Collection`:**
   - `crates/contextra-engine/src/transaction/compensating_actions.rs:239`: `no field graph_index on type collection::Collection`
   - `crates/contextra-engine/src/transaction/compensating_actions.rs:250`: `no field graph_index on type collection::Collection`
   - `crates/contextra-engine/src/transaction/compensating_actions.rs:372`: `no field graph_index on type collection::Collection`
   - `crates/contextra-engine/src/transaction/db_transaction.rs:235`: `no field graph_index on type collection::Collection`
2. **Fehlende Methoden auf `Collection`:**
   - `crates/contextra-engine/src/contextra_impl/collections.rs:270`: `no method named set_community_detection_trigger_threshold`
   - `crates/contextra-engine/src/contextra_impl/collections.rs:276`: `no method named set_auto_extraction_config`
   - `crates/contextra-engine/src/contextra_impl/crud.rs:297`: `no method named relate`
   - `crates/contextra-engine/src/contextra_impl/crud.rs:303`: `no method named relate_bidirectional`
3. **Fehlende Variants in `ContextraError`:**
   - `crates/contextra-engine/src/collection/mod.rs:513`: `no variant or associated item named TenantIsolationViolation found for enum contextra_types::ContextraError`
