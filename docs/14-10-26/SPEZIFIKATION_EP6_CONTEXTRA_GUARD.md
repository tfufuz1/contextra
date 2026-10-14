# Contextra — Implementierungsspezifikation EP6: Contextra Guard (Privacy, MCP & Sandbox)

**Dokumenten-ID:** `DOC-SPEC-EP6-GUARD-2026-10-14`  
**Crate-Scope (Streng Disjunktiv):**
- `contextra-privacy` (Ring 2 — Egress Vault, PII Masking, Hash-Chained Audit Trace)
- `contextra-sandbox` (Ring 3 — WASM Plugin Host, Module Admission, Fuel Limits)
- `contextra-mcp` (Ring 3 — Model Context Protocol Server & Tool Classification)
- `contextra-py` (Ring 4 — Python CFFI Bindings & Integration)

**Status:** Verbindliche normative Spezifikation  
**Verantwortlich:** Principal Senior Rust Architect

---

## 1. Übersicht & Zielsetzung

EP6 schützt das Gesamtsystem an seinen äußeren Rändern:
1. **Egress Firewall & DLP:** Fail-Closed Egress Gateway zur Erkennung und Maskierung von PII (Kreditkarten nach Luhn-Algorithmus, IBAN, API-Keys).
2. **Hash-Verketteter Audit-Trail (INV-EGRESS-AUDIT-1):** BLAKE3-Kryptokette aller ausgehenden Anfragen ohne Speicherung sensibler Klartextdaten.
3. **Behebung von B1, B4 & B9 im MCP-Gateway:** Striktes Delegieren aller Operationen an `ContextraDb`; Blockieren von Cloud-Queries ohne aktiven Guard; Rückgabe valider Löschnachweise.
4. **WASM-Sandbox-Isolation:** Begrenzung von Custom Merge-Operators durch deterministisches CPU-Fuel (10.000.000 Ticks) und 5.000 ms Timeout mit `ZeroizeOnDrop` für Outputs.

---

## 2. Befund- und Korrekturmatrix EP6

| ID | Datei | Zeile(n) | Problem & Befund | Exakte Korrekturmaßnahme |
|---|---|---|---|---|
| **B1** | `crates/contextra-mcp/src/server.rs` | 145–170 | `contextra_cloud_query` ist registriert, selbst wenn kein `EgressGuard` konfiguriert ist (Fail-Open Risiko). | Vor Aufruf prüfen: Wenn `egress_guard.is_none()`, Anfrage strikt mit Fehler abweisen (Fail-Closed). |
| **B9** | `crates/contextra-mcp/src/server.rs` | 310–390 | `remember` und `recall` implementieren eigene Teillogik statt Delegation an `ContextraDb`. | Duplizierte Pfade löschen; direkte Delegation an `ContextraDb::remember` und `ContextraDb::recall`. |
| **SANDBOX-RING** | `crates/contextra-sandbox/Cargo.toml` | 15–30 | Unerlaubte Dev- und Build-Kanten verletzen Ring-Hierarchie. | Sandbox strikt in Ring 3 positionieren; `WasmMergeOperator` hinter Port-Trait `MergeOperator` kapseln. |
| **ORPHAN-PRIV** | `crates/contextra-privacy/src/egress_gateway.rs` | 260 | `handle_cloud_query_scoped_with_bulk_detector` unverdrahtet. | Im MCP-Cloud-Query-Dispatcher als Standard-Evaluator registrieren. |
| **SURROGATE-DEF** | `crates/contextra-privacy/src/egress_vault.rs` | 135 | `SurrogateVault` speichert Mapping im RAM ohne Scrubbing bei Drop. | `ZeroizeOnDrop` für Vault-Einträge erzwingen. |

---

## 3. Exakte Schnittstellen & Implementierungen

### 3.1 Behebung B1 & B9 im MCP-Server
```rust
// crates/contextra-mcp/src/server.rs

use contextra_types::{Result, ContextraError, TenantId};
use serde_json::{Value, json};

impl McpServer {
    /// Behebung B1: Egress Guard Pflichtprüfung
    pub async fn handle_cloud_query(&self, tenant_id: TenantId, query: Value) -> Result<Value> {
        let guard = self.egress_guard.as_ref().ok_or_else(|| {
            ContextraError::security_violation(
                "Cloud egress rejected: EgressGuard is not active or unconfigured (Fail-Closed B1)"
            )
        })?;

        // 1. DLP & Sanitization
        let sanitized = guard.sanitize_outbound_payload(tenant_id, &query)?;

        // 2. Ausführung über sicheren Port
        let response = self.cloud_client.execute(sanitized).await?;

        // 3. Rehydrierung
        let rehydrated = guard.rehydrate_response(tenant_id, response)?;
        Ok(rehydrated)
    }

    /// Behebung B9: Direkte Delegation an ContextraDb
    pub async fn handle_remember(&self, tenant_id: TenantId, doc: Value) -> Result<Value> {
        let memory_entry = serde_json::from_value(doc)
            .map_err(|e| ContextraError::invalid_input(format!("Invalid document JSON: {e}")))?;
        
        let doc_id = self.db.remember(tenant_id, memory_entry).await?;
        Ok(json!({ "status": "stored", "doc_id": doc_id }))
    }
}
```

### 3.2 Hash-Chained Egress Audit Trace (INV-EGRESS-AUDIT-1)
```rust
// crates/contextra-privacy/src/audit_trace.rs

use contextra_types::TenantId;

#[derive(Debug, Clone)]
pub struct EgressTraceChain {
    current_tail_hash: [u8; 32],
}

impl EgressTraceChain {
    pub fn new(genesis_hash: [u8; 32]) -> Self {
        Self { current_tail_hash: genesis_hash }
    }

    pub fn append_event(
        &mut self,
        tenant_id: TenantId,
        destination_url_hash: [u8; 32],
        payload_hash: [u8; 32],
        timestamp_utc: u64,
    ) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();
        hasher.update(&self.current_tail_hash);
        hasher.update(&tenant_id.0.to_le_bytes());
        hasher.update(&destination_url_hash);
        hasher.update(&payload_hash);
        hasher.update(&timestamp_utc.to_le_bytes());

        let new_hash = *hasher.finalize().as_bytes();
        self.current_tail_hash = new_hash;
        new_hash
    }

    pub fn current_hash(&self) -> [u8; 32] {
        self.current_tail_hash
    }
}
```

### 3.3 WASM Module Admission Control & Fuel Limits
```rust
// crates/contextra-sandbox/src/executor.rs

use contextra_types::{Result, ContextraError};

pub const DEFAULT_MAX_FUEL: u64 = 10_000_000; // Deterministisches CPU-Limit
pub const DEFAULT_TIMEOUT_MS: u64 = 5_000;     // Wall-Clock Timeout

pub struct WasmExecutionPolicy {
    pub max_fuel: u64,
    pub timeout_ms: u64,
    pub max_memory_pages: u32,
}

impl Default for WasmExecutionPolicy {
    fn default() -> Self {
        Self {
            max_fuel: DEFAULT_MAX_FUEL,
            timeout_ms: DEFAULT_TIMEOUT_MS,
            max_memory_pages: 16, // 1 MB
        }
    }
}
```

---

## 4. Akzeptanztests & Verifikationskommandos
1. `cargo test -p contextra-mcp --test egress_guard_fail_closed` (Prüft B1)
2. `cargo test -p contextra-mcp --test mcp_facade_delegation` (Prüft B9)
3. `cargo test -p contextra-privacy --test egress_audit_chain_integrity` (Prüft INV-EGRESS-AUDIT-1)
4. `cargo test -p contextra-sandbox --test wasm_fuel_exhaustion` (Prüft CPU-Tick-Limit)
