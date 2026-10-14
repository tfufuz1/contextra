# Contextra — Mikrofeingranulare Implementierungsspezifikation EP6: Contextra Guard & Sandbox

**Dokumenten-ID:** `DOC-SPEC-EP6-GUARD-2026-10-14`  
**Version:** 1.0 · **Datum:** 14. Oktober 2026  
**Autor:** Principal Senior Rust Architect für Contextra  
**Status:** Verbindliche, normative Implementierungsanweisung für Egress-Schutz, Sandbox & Lizenzen  
**Geltungsbereich (Disjunktive Crates):**  
- **Ring 0 (Datenschutz):** [`contextra-privacy`](file:///home/freddy/Projekte/contextra/crates/contextra-privacy)  
- **Ring 2 (WASM-Sandbox):** [`contextra-sandbox`](file:///home/freddy/Projekte/contextra/crates/contextra-sandbox)  
- **Ring 0 (Lizenz-Gating):** [`contextra-license`](file:///home/freddy/Projekte/contextra/crates/contextra-license)  
- **Ring 4 (MCP Guard Wiring):** [`contextra-mcp`](file:///home/freddy/Projekte/contextra/crates/contextra-mcp) (Gating von `contextra_cloud_query`)

---

## 1. Architektonische Grenzziehung & Disjunktion

Contextra Guard & Sandbox (EP6) stellt die Sicherheits- und Isolationshülle dar: Cloud-Egress-Filterung mit DLP-Pseudonymisierung, sichere WASM-Merge-Operatoren mit Fuel- und Timeout-Schranken sowie kryptografische Lizenzprüfung.

**Disjunktionsregeln für EP6:**
1. **Keine direkte Ring-Verletzung:** `contextra-store` (Ring 1) und `contextra-engine` (Ring 3) dürfen **nicht** direkt von `contextra-sandbox` (Ring 2) abhängen. Die Koppelung erfolgt ausschließlich über den Port-Trait `MergeOperatorFactory` in `contextra-ports`.
2. **Fail-Closed bei Egress:** Tritt im Egress-Gateway ein Timeout, ein Regex-Fehler oder ein Klassifikations-Fehlzustand auf, muss die Anfrage strikt blockiert werden (`EgressClassification::Block`). Es existiert kein Fail-Open.
3. **Sicherheitsgrenze MCP-Tool-Registrierung:** `contextra_cloud_query` darf im MCP-Server nur dann registriert werden, wenn der `EgressGuard` konfiguriert und aktiv ist (Behebung B1 & INV-MCP-CLASSIFY-1).

---

## 2. Modul- und Datei-Feinspezifikationen

### 2.1 Komponente 1: Egress-Schutz & DLP Surrogate Vault (`contextra-privacy`)

#### Architektur
Vor jedem ausgehenden Cloud-Aufruf werden sensible Daten (Kreditkarten nach Luhn, IBAN, E-Mails, API-Keys) im Klartext abgefangen, durch Surrogat-Token (`[USER_ENTITY_<16-hex>]`) pseudonymisiert und im `SurrogateVault` gehalten. Antworten werden rehydriert.

#### Datei: `crates/contextra-privacy/src/egress_vault.rs`
```rust
use contextra_types::Result;

pub struct EgressVault {
    surrogates: SurrogateVault,
}

impl EgressVault {
    /// Pseudonymisiert sensible Daten im Payload (Fail-Closed)
    pub fn sanitize_and_vault(&self, payload: &str) -> Result<SanitizedPayload, EgressError> {
        let mut sanitized = payload.to_string();

        // 1. Kreditkartenprüfung mit Luhn-Algorithmus
        sanitized = self.mask_credit_cards(&sanitized)?;

        // 2. IBAN Validierung
        sanitized = self.mask_ibans(&sanitized)?;

        // 3. PII & API-Key Erkennung
        sanitized = self.mask_api_keys(&sanitized)?;

        Ok(SanitizedPayload {
            content: sanitized,
            has_surrogates: true,
        })
    }
}
```

---

### 2.2 Komponente 2: MCP Gating von `contextra_cloud_query` (Behebung B1)

#### Problem
In `crates/contextra-mcp/src/sandbox.rs` war `contextra_cloud_query` standardmäßig registriert, selbst wenn kein Egress-Guard konfiguriert war.

#### Datei: `crates/contextra-mcp/src/sandbox.rs`
```rust
impl McpSandbox {
    pub fn register_tools(&mut self, has_egress_guard: bool) {
        // Lokale Core-Tools — immer registriert
        self.register_core_tools();

        // Egress-Tool ausschließlich bei aktivem Schutz registrieren
        #[cfg(feature = "guard")]
        if has_egress_guard {
            self.registry.register(
                "contextra_cloud_query",
                ToolClassification::EgressExternal,
            );
        }
    }
}
```

---

### 2.3 Komponente 3: Entkopplung der WASM-Sandbox über Port-Traits

#### Problem
Die direkte Abhängigkeit von `engine -> sandbox` und `store -> sandbox` verletzte die Ring-Layering-Invariante (P5).

#### Datei: `crates/contextra-ports/src/merge_operator.rs`
```rust
pub trait MergeOperatorFactory: Send + Sync + 'static {
    fn create_operator(&self, module_bytes: &[u8]) -> Result<Box<dyn MergeOperator>, ContextraError>;
}

pub trait MergeOperator: Send + Sync {
    fn merge(&self, existing_val: Option<&[u8]>, operand: &[u8]) -> Result<Vec<u8>, ContextraError>;
}
```

#### Datei: `crates/contextra-sandbox/src/executor.rs`
`contextra-sandbox` implementiert diesen Port; `store` und `engine` kennen nur den Trait aus `contextra-ports`:
```rust
impl contextra_ports::MergeOperatorFactory for WasmExecutor {
    fn create_operator(&self, module_bytes: &[u8]) -> Result<Box<dyn contextra_ports::MergeOperator>, ContextraError> {
        let admitted = self.validate_and_admit(module_bytes)?;
        Ok(Box::new(WasmMergeOperator::new(admitted)))
    }
}
```

---

### 2.4 Komponente 4: Ed25519 Lizenz-Gating (`contextra-license`)

#### Datei: `crates/contextra-license/src/signed_gate.rs`
```rust
pub struct SignedLicenseGate {
    verifying_key: ed25519_dalek::VerifyingKey,
}

impl SignedLicenseGate {
    /// 7-Stufen Evaluation: Signatur, Gültigkeitszeitraum, Machine-Fingerprint, Tier-Limits
    pub fn evaluate_license(
        &self,
        raw_token: &[u8],
        current_fingerprint: &str,
        now_unix: u64,
    ) -> Result<LicenseFeatures, LicenseError> {
        // 1. Signatur validieren
        let payload = self.verify_ed25519_signature(raw_token)?;

        // 2. Zeitstempel prüfen
        if now_unix < payload.valid_from || now_unix > payload.valid_to {
            return Err(LicenseError::Expired);
        }

        // 3. Machine-Binding prüfen
        if payload.fingerprint != current_fingerprint {
            return Err(LicenseError::MachineMismatch);
        }

        Ok(payload.features)
    }
}
```

---

## 3. Akzeptanzkriterien & Qualitätsgates (EP6)

1. **Fail-Closed Test:**
   `cargo test -p contextra-privacy --test fail_closed` verifiziert, dass Timeouts und ReDoS-Versuche ausnahmslos mit `Err(EgressError::Blocked)` enden.
2. **Ring-Layering Prüfung:**
   `cargo xtask check-ring-layering` muss nach Entfernung der direkten Sandbox-Abhängigkeiten in Ring 1 und Ring 3 mit 0 Verstößen durchlaufen.
3. **MCP Tool-Klassifikation:**
   `cargo test -p contextra-mcp --test tool_classification` stellt sicher, dass `contextra_cloud_query` ohne aktiven Egress-Guard unregistriert bleibt und Abrufe mit `MethodNotFound` scheitern.
