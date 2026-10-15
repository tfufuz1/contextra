> **SUPERSEDED:** superseded by docs/spec/CONTEXTRA_ENDPRODUKT_SPEZIFIKATION_v1_4.md

# Contextra — Mikrofeingranulare Implementierungsspezifikation EP6: Contextra Guard & Sandbox

**Dokumenten-ID:** `DOC-SPEC-EP6-GUARD-2026-10-14`  
**Version:** 2.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Egress-Schutz, Sandbox & Lizenzen  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (Datenschutz & DLP):** [`contextra-privacy`](../crates/contextra-privacy)
- **Ring 2 (WASM-Sandbox):** [`contextra-sandbox`](../crates/contextra-sandbox)
- **Ring 0 (Lizenz-Gating):** [`contextra-license`](../crates/contextra-license)
- **Ring 4 (MCP Guard Wiring):** [`contextra-mcp`](../crates/contextra-mcp) (Gating von `contextra_cloud_query`)
- **Ring 0 (Port-Verträge):** [`contextra-ports`](../crates/contextra-ports) (Modul `src/merge_operator.rs`)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Guard & Sandbox (EP6) stellt die Sicherheits- und Isolationshülle dar: Cloud-Egress-Filterung mit DLP-Pseudonymisierung, sichere WASM-Merge-Operatoren mit Fuel- und Timeout-Schranken sowie kryptografische Lizenzprüfung.

**Disjunktionsregeln für EP6:**
1. **Keine Ring-DAG-Verletzung (P5):** Weder `contextra-store` (Ring 1) noch `contextra-engine` (Ring 3) dürfen direkt von `contextra-sandbox` (Ring 2) abhängen. Die Koppelung erfolgt ausschließlich über den Port-Trait `MergeOperatorFactory` in `contextra-ports`.
2. **Fail-Closed bei Egress:** Tritt im Egress-Gateway ein Timeout, ein Regex-Fehler oder ein Klassifikations-Fehlzustand auf, muss die Anfrage strikt blockiert werden (`EgressClassification::Block`). Es existiert kein Fail-Open.
3. **Sicherheitsgrenze MCP-Tool-Registrierung (B1 & INV-MCP-CLASSIFY-1):** `contextra_cloud_query` darf im MCP-Server nur dann registriert werden, wenn der `EgressGuard` konfiguriert und aktiv ist.

---

## 2. Detaillierte Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: Egress-Schutz & DLP Surrogate Vault (`contextra-privacy`)

#### Architektur & Algorithmus
Vor jedem externen Cloud-Call wird der Payload untersucht:
1. **Kreditkartenprüfung:** Luhn-Algorithmus (MOD 10) über 13–19-stellige Ziffernfolgen.
2. **IBAN-Prüfung:** MOD 97-10 Prüfziffern-Validierung nach ISO 7064.
3. **PII & API-Keys:** Regex-Matching für E-Mails, IPv4/IPv6, Bearer-Tokens (`sk-[A-Za-z0-9]{32,}`).
4. **Pseudonymisierung:** Ersetzung durch Surrogat-Token `[USER_ENTITY_<16-hex-hash>]` und Ablage im `SurrogateVault`.

#### Datei: `crates/contextra-privacy/src/egress_vault.rs`
**IST-Zustand (Zeilen 15–70):** Luhn- und IBAN-Routinen vorhanden, jedoch fehlte einheitlicher Struct-Durchstich mit Fail-Closed Rückgabe.  
**SOLL-Blueprint:**
```rust
// In crates/contextra-privacy/src/egress_vault.rs
use ahash::AHashMap;
use contextra_types::{ContextraError, Result};
use std::sync::RwLock;

pub struct SurrogateVault {
    // Map: Surrogat-Token -> Originaltext
    surrogates: RwLock<AHashMap<String, String>>,
}

impl SurrogateVault {
    pub fn new() -> Self {
        Self {
            surrogates: RwLock::new(AHashMap::new()),
        }
    }

    pub fn insert_surrogate(&self, original: &str) -> String {
        let hash = blake3::hash(original.as_bytes());
        let token = format!("[USER_ENTITY_{}]", &hash.to_hex()[..16]);
        let mut map = self.surrogates.write().unwrap();
        map.insert(token.clone(), original.to_string());
        token
    }

    pub fn rehydrate(&self, text: &str) -> String {
        let map = self.surrogates.read().unwrap();
        let mut rehydrated = text.to_string();
        for (token, original) in map.iter() {
            if rehydrated.contains(token) {
                rehydrated = rehydrated.replace(token, original);
            }
        }
        rehydrated
    }
}

pub struct EgressVault {
    pub surrogates: SurrogateVault,
}

impl EgressVault {
    pub fn new() -> Self {
        Self {
            surrogates: SurrogateVault::new(),
        }
    }

    /// Validiert Kreditkartennummern nach dem Luhn-Algorithmus (MOD 10)
    pub fn is_luhn_valid(digits: &str) -> bool {
        let cleaned: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
        if cleaned.len() < 13 || cleaned.len() > 19 {
            return false;
        }

        let mut sum = 0;
        let mut alternate = false;
        for &d in cleaned.iter().rev() {
            let mut val = d;
            if alternate {
                val *= 2;
                if val > 9 {
                    val -= 9;
                }
            }
            sum += val;
            alternate = !alternate;
        }

        sum % 10 == 0
    }

    /// Führt die Pseudonymisierung durch (Fail-Closed)
    pub fn sanitize_payload(&self, payload: &str) -> Result<String, ContextraError> {
        let mut sanitized = payload.to_string();

        // 1. Kreditkarten erkennen und maskieren
        // ... (Regex Matcher auf \b(?:\d[ -]*?){13,19}\b)
        
        // 2. API-Schlüssel maskieren
        if sanitized.contains("sk-") {
            // Surrogatisieren
        }

        Ok(sanitized)
    }
}
```

---

### 2.2 Komponente 2: MCP Gating von `contextra_cloud_query` (Behebung B1)

#### Problem
In [`crates/contextra-mcp/src/sandbox.rs`](../crates/contextra-mcp/src/sandbox.rs) war `contextra_cloud_query` registriert, selbst wenn kein Egress-Guard konfiguriert war. Dadurch konnte ein Agent Daten ohne Egress-Filterung an externe Clouds senden.

#### Datei: `crates/contextra-mcp/src/sandbox.rs`
**SOLL-Blueprint:**
```rust
// In crates/contextra-mcp/src/sandbox.rs (Zeile 40-75)
use crate::protocol::ToolClassification;

impl McpSandbox {
    /// Registriert verfügbare MCP-Tools unter strikter Berücksichtigung konfigurierter Schutzeinrichtungen.
    pub fn register_tools(&mut self, egress_guard_active: bool) {
        // Lokale Core-Tools — immer registriert
        self.registry.register("contextra_search", ToolClassification::ReadOnly);
        self.registry.register("contextra_remember", ToolClassification::WriteLocal);
        self.registry.register("contextra_forget", ToolClassification::WriteLocal);

        // B1-Fix & INV-MCP-CLASSIFY-1:
        // contextra_cloud_query DARF NUR DANN registriert werden, wenn der EgressGuard aktiv ist
        #[cfg(feature = "guard")]
        if egress_guard_active {
            self.registry.register(
                "contextra_cloud_query",
                ToolClassification::EgressExternal,
            );
        }
    }
}
```

---

### 2.3 Komponente 3: Entkopplung der WASM-Sandbox via Port-Traits (Behebung Ring-Layering)

#### Problem
`contextra-store` (Ring 1) und `contextra-engine` (Ring 3) banden `contextra-sandbox` (Ring 2) direkt in den `Cargo.toml`-Manifesten ein. Dies erzeugte verbotene Rückwärtskanten im Ring-Graphen (`AUDIT_governance_harness_DEEP_2026-10-13.md`).

#### Datei 1: `crates/contextra-ports/src/merge_operator.rs` (NEU)
```rust
// In crates/contextra-ports/src/merge_operator.rs
use contextra_types::{ContextraError, Result};

pub trait MergeOperator: Send + Sync {
    /// Führt die Zusammenführung des bisherigen Werts und des neuen Operanden durch.
    fn merge(&self, existing: Option<&[u8]>, operand: &[u8]) -> Result<Vec<u8>>;
}

pub trait MergeOperatorFactory: Send + Sync + 'static {
    /// Instanziiert einen MergeOperator aus WASM-Bytecode oder statischer Definition.
    fn create_operator(&self, module_bytes: &[u8]) -> Result<Box<dyn MergeOperator>>;
}
```

#### Datei 2: `crates/contextra-sandbox/src/executor.rs`
```rust
// In crates/contextra-sandbox/src/executor.rs
use contextra_ports::merge_operator::{MergeOperator, MergeOperatorFactory};
use contextra_types::{ContextraError, Result};

pub struct WasmExecutor {
    engine: wasmtime::Engine,
    max_fuel: u64,
}

impl MergeOperatorFactory for WasmExecutor {
    fn create_operator(&self, module_bytes: &[u8]) -> Result<Box<dyn MergeOperator>> {
        let admitted = self.validate_and_admit_module(module_bytes)?;
        Ok(Box::new(WasmMergeInstance {
            engine: self.engine.clone(),
            module: admitted,
            max_fuel: self.max_fuel,
        }))
    }
}

pub struct WasmMergeInstance {
    engine: wasmtime::Engine,
    module: crate::admission::AdmittedModule,
    max_fuel: u64,
}

impl MergeOperator for WasmMergeInstance {
    fn merge(&self, existing: Option<&[u8]>, operand: &[u8]) -> Result<Vec<u8>> {
        // Sichere WASI Preview1 Ausführung mit Fuel- und Timeout-Budget (Fail-Closed)
        // ...
        Ok(Vec::new())
    }
}
```

---

### 2.4 Komponente 4: Ed25519 Lizenz-Gating (`contextra-license`)

#### Datei: `crates/contextra-license/src/signed_gate.rs`
**Ziel:** 7-Stufen Evaluation mit Machine-Binding.
```rust
// In crates/contextra-license/src/signed_gate.rs (Zeile 160-220)
use ed25519_dalek::{Signature, VerifyingKey};
use contextra_types::{ContextraError, Result};

pub struct SignedLicenseGate {
    public_key: VerifyingKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LicensePayload {
    pub customer_id: String,
    pub tier: String,
    pub valid_from_unix: u64,
    pub valid_until_unix: u64,
    pub machine_hash: String,
}

impl SignedLicenseGate {
    pub fn verify_and_admit(
        &self,
        token_bytes: &[u8],
        expected_machine_hash: &str,
        current_unix_time: u64,
    ) -> Result<LicensePayload> {
        if token_bytes.len() < 64 {
            return Err(ContextraError::PermissionDenied("Invalid license token length".to_string()));
        }

        let split_pos = token_bytes.len() - 64;
        let payload_bytes = &token_bytes[..split_pos];
        let sig_bytes: [u8; 64] = token_bytes[split_pos..].try_into().unwrap();
        let sig = Signature::from_bytes(&sig_bytes);

        // 1. Mathematische Signaturprüfung
        self.public_key.verify(payload_bytes, &sig)
            .map_err(|_| ContextraError::PermissionDenied("License signature invalid".to_string()))?;

        // 2. JSON-Payload deserialisieren
        let payload: LicensePayload = serde_json::from_slice(payload_bytes)
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;

        // 3. Zeitstempel prüfen
        if current_unix_time < payload.valid_from_unix || current_unix_time > payload.valid_until_unix {
            return Err(ContextraError::PermissionDenied("License has expired".to_string()));
        }

        // 4. Machine-Binding prüfen
        if payload.machine_hash != expected_machine_hash {
            return Err(ContextraError::PermissionDenied("License bound to another machine".to_string()));
        }

        Ok(payload)
    }
}
```

---

## 3. Akzeptanzkriterien, Testsuite & Qualitätsgates (EP6)

### 3.1 Unit-Test: Fail-Closed Egress Timeout & ReDoS Test
**Datei:** `crates/contextra-privacy/tests/fail_closed_egress_test.rs`
```rust
use contextra_privacy::egress_gateway::handle_cloud_query_scoped_with_guard;

#[tokio::test]
async fn test_egress_timeout_fails_closed() {
    // Simuliert einen Classifier-Timeout
    let res = handle_cloud_query_scoped_with_guard("sensitiver text", /* timeout_immediate */ true).await;
    assert!(res.is_err(), "Egress must strictly fail closed on timeout");
}
```

### 3.2 Verbindliche Acceptance-Kommandos
```bash
# 1. Ring-Layering Prüfung: Keine direkte Abhängigkeit von store/engine auf sandbox
cargo xtask check-ring-layering

# 2. MCP Tool Classification Verification
cargo test -p contextra-mcp --test mcp_tool_classification

# 3. Privacy & DLP Testsuite
cargo test -p contextra-privacy --locked
```
**Erwartetes Ergebnis:** Alle Tests müssen mit Exit-Code 0 terminieren; 0 Ring-Layering-Verletzungen.
