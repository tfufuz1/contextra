// FILE-CONTEXT
// STAND:       2026-09-10T19:25:24Z (SESSION: ae8c2fb9)
// ZWECK:       MCP Sandbox & Zero-Trust Tool Isolation Layer
// INVARIANTEN: Opt-In Security; volatile_results nutzt Single-Lock (parking_lot::Mutex); keine geschachtelten Locks
// HOTSPOTS:    validate_tool_call(), store_volatile(), execute_with_timeout()
// SIEHE AUCH:  ADR-010, rules/detect_nested_locks.yml

// contextra-mcp/src/sandbox.rs
// MCP Tool Isolation Layer (Contextra Volatile-Output Isolation)

use contextra_core::{ContextraError, Result};
use parking_lot::Mutex;
use serde_json::Value;
use std::collections::HashMap;

/// Maximale Anzahl von volatilen Ergebnissen pro Sandbox-Session.
pub const MAX_VOLATILE_RESULTS: usize = 1_000;
/// Maximale Länge eines Volatile-Result-Schlüssels in Bytes.
pub const MAX_VOLATILE_KEY_BYTES: usize = 256;
/// Maximale Größe einer volatilen Tool-Ausgabe in Bytes (16 MB).
pub const MAX_VOLATILE_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

/// Erlaubte MCP-Tool-Kategorien (Whitelist-Prinzip).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCategory {
    /// Read-only Datenbankoperationen (immer erlaubt).
    DatabaseRead,
    /// Schreibende Datenbankoperationen (erfordern explizite Freigabe).
    DatabaseWrite,
    /// Externe Code-Ausführung (höchste Risikostufe, standardmäßig gesperrt).
    CodeExecution,
    /// Externe Cloud-Anfragen / Egress (standardmäßig gesperrt; erfordert explizites Opt-in).
    CloudEgress,
}

/// Konfiguration der Sandbox-Policy.
#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    pub allow_db_reads: bool,
    pub allow_db_writes: bool,
    pub allow_code_execution: bool,
    /// Cloud-Egress erlaubt? Standardmäßig `false` (additiv zu Layer 1/4 EgressGuard Bulk-Exfiltrationsprüfungen).
    pub allow_cloud_egress: bool,
    /// Maximale Ausführungszeit pro Tool-Call in Millisekunden.
    pub max_execution_ms: u64,
}

impl Default for SandboxPolicy {
    fn default() -> Self {
        Self {
            allow_db_reads: true,
            allow_db_writes: false,      // Schreibzugriff explizit opt-in
            allow_code_execution: false, // Code-Ausführung standardmäßig gesperrt
            allow_cloud_egress: false, // Cloud-Egress gesperrt (erfordert explizites Opt-in via allow_cloud_egress)
            max_execution_ms: 5_000,
        }
    }
}

/// Volatiler Tool-Output-Speicher mit Zeroize-Garantie bei Drop.
///
/// Tool-Ergebnisse werden verschlüsselt im Arbeitsspeicher gehalten.
/// Beim Drop wird der Speicher via Zeroize bereinigt.
pub struct VolatileToolResult {
    /// Verschlüsselte Ausgabe (AES-256-GCM-SIV via contextra-security).
    encrypted: zeroize::Zeroizing<Vec<u8>>,
    /// Klartext-Nonce (nicht sensitiv).
    nonce: Vec<u8>,
}

impl VolatileToolResult {
    /// Speichert einen Tool-Output verschlüsselt.
    pub fn encrypt(plaintext: &[u8], key: &contextra_crypto::CryptoKey) -> Result<Self> {
        let (encrypted, nonce) = key
            .encrypt_auto_nonce(plaintext)
            .map_err(|e| ContextraError::Internal(format!("Sandbox encrypt: {e}")))?;
        Ok(Self {
            encrypted: zeroize::Zeroizing::new(encrypted),
            nonce: nonce.to_vec(),
        })
    }

    /// Entschlüsselt und gibt den Klartext zeroized zurück.
    pub fn decrypt(&self, key: &contextra_crypto::CryptoKey) -> Result<zeroize::Zeroizing<Vec<u8>>> {
        if self.nonce.len() != 12 {
            return Err(ContextraError::Internal(
                "Sandbox decrypt: Invalid nonce length".into(),
            ));
        }
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes.copy_from_slice(&self.nonce);
        let decrypted = key
            .decrypt_auto_nonce(&self.encrypted, &nonce_bytes)
            .map_err(|e| ContextraError::Internal(format!("Sandbox decrypt: {e}")))?;
        Ok(zeroize::Zeroizing::new(decrypted))
    }
}

/// MCP Sandbox: validiert Tool-Calls und verwaltet volatile Ergebnisse.
pub struct McpSandbox {
    policy: SandboxPolicy,
    /// Session-lokale volatile Ergebnisse (werden bei Session-Ende gedropt).
    volatile_results: Mutex<HashMap<String, VolatileToolResult>>,
    /// Session-Schlüssel (wird bei Drop zeroized).
    session_key: contextra_crypto::CryptoKey,
}

impl McpSandbox {
    /// Erstellt eine neue Sandbox-Instanz mit frischem Sitzungsschlüssel.
    pub fn new(policy: SandboxPolicy) -> Result<Self> {
        use rand::RngCore;
        let mut salt = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut salt);
        let mut passphrase = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut passphrase);
        let key =
            contextra_crypto::CryptoKey::try_new(&hex::encode(passphrase), &salt).map_err(|e| {
                ContextraError::Internal(format!("McpSandbox: CryptoKey initialization failed: {e}"))
            })?;

        Ok(Self {
            policy,
            volatile_results: Mutex::new(HashMap::new()),
            session_key: key,
        })
    }

    /// Gibt die aktuell konfigurierte SandboxPolicy zurück.
    pub fn policy(&self) -> &SandboxPolicy {
        &self.policy
    }

    /// Validiert ob ein Tool-Call erlaubt ist.
    ///
    /// HINWEIS: Die Prüfung von `ToolCategory::CloudEgress` ist **additiv** zur
    /// Egress-Guard- / Egress-Vault-Prüfung (`EgressClassifier`/`EgressGuardCheck` in `egress_gateway.rs`).
    /// Die Sandbox-Policy entscheidet, ob die Methode überhaupt aufgerufen werden darf,
    /// während der Egress-Guard zusätzlich prüft, ob der konkrete Payload bei erlaubtem Aufruf
    /// unbedenklich ist (z. B. Schutz vor Bulk-Exfiltration).
    pub fn validate_tool_call(&self, method: &str, _params: &Value) -> Result<()> {
        if method.is_empty() || method.len() > 256 {
            return Err(ContextraError::InvalidInput(format!(
                "Sandbox: Tool call method name length invalid: {}",
                method.len()
            )));
        }
        let category = Self::classify_method(method);
        match category {
            ToolCategory::DatabaseRead => {
                if !self.policy.allow_db_reads {
                    return Err(ContextraError::InvalidInput(format!(
                        "Sandbox: DB-Lesezugriff gesperrt für '{method}'"
                    )));
                }
            }
            ToolCategory::DatabaseWrite => {
                if !self.policy.allow_db_writes {
                    return Err(ContextraError::InvalidInput(format!(
                        "Sandbox: DB-Schreibzugriff gesperrt für '{method}'"
                    )));
                }
            }
            ToolCategory::CodeExecution => {
                if !self.policy.allow_code_execution {
                    return Err(ContextraError::InvalidInput(
                        "Sandbox: Code-Ausführung ist gesperrt (SandboxPolicy)".into(),
                    ));
                }
            }
            ToolCategory::CloudEgress => {
                if !self.policy.allow_cloud_egress {
                    return Err(ContextraError::InvalidInput(format!(
                        "Sandbox: Cloud-Egress gesperrt für '{method}' — erfordert explizites Opt-in via `allow_cloud_egress`"
                    )));
                }
            }
        }
        Ok(())
    }

    /// Klassifiziert die MCP-Methode bzw. den Tool-Namen in eine `ToolCategory`.
    pub fn classify_method(method: &str) -> ToolCategory {
        match method {
            "contextra_search" | "contextra_get" | "contextra_collections" => ToolCategory::DatabaseRead,
            "contextra_cloud_query" => ToolCategory::CloudEgress,
            "contextra_insert"
            | "contextra_delete"
            | "contextra_forget"
            | "contextra_upsert"
            | "contextra_relate"
            | "contextra_relate_n_ary"
            | "contextra_create_collection"
            | "contextra_drop_collection"
            | "contextra_consolidate" => ToolCategory::DatabaseWrite,
            _ => ToolCategory::CodeExecution,
        }
    }

    /// Führt eine Asynchrone Tool-Future mit Timeout gemäß SandboxPolicy aus.
    pub async fn execute_with_timeout<F, T, E>(
        &self,
        tool_name: &str,
        fut: F,
    ) -> std::result::Result<T, E>
    where
        F: std::future::Future<Output = std::result::Result<T, E>>,
        E: From<ContextraError>,
    {
        let duration = std::time::Duration::from_millis(self.policy.max_execution_ms);
        tokio::time::timeout(duration, fut).await.map_err(|_| {
            E::from(ContextraError::Internal(format!(
                "Tool '{}' überschritt Timeout {}ms",
                tool_name, self.policy.max_execution_ms
            )))
        })?
    }

    /// Speichert einen Tool-Output verschlüsselt in der Session.
    pub fn store_volatile(&self, key: &str, output: &[u8]) -> Result<()> {
        if key.is_empty() || key.len() > MAX_VOLATILE_KEY_BYTES {
            return Err(ContextraError::InvalidInput(format!(
                "Sandbox: Volatile result key length invalid: {} (max {})",
                key.len(),
                MAX_VOLATILE_KEY_BYTES
            )));
        }
        if output.len() > MAX_VOLATILE_OUTPUT_BYTES {
            return Err(ContextraError::InvalidInput(format!(
                "Sandbox: Volatile result output size exceeded: {} bytes (max {})",
                output.len(),
                MAX_VOLATILE_OUTPUT_BYTES
            )));
        }
        let mut results = self.volatile_results.lock();
        if !results.contains_key(key) && results.len() >= MAX_VOLATILE_RESULTS {
            return Err(ContextraError::InvalidInput(format!(
                "Sandbox: Volatile result limit reached ({MAX_VOLATILE_RESULTS})"
            )));
        }
        let volatile_res = VolatileToolResult::encrypt(output, &self.session_key)?;
        results.insert(key.to_string(), volatile_res);
        Ok(())
    }

    /// Ruft einen verschlüsselten Tool-Output ab und entschlüsselt ihn.
    pub fn get_volatile(&self, key: &str) -> Result<Option<zeroize::Zeroizing<Vec<u8>>>> {
        let results = self.volatile_results.lock();
        if let Some(res) = results.get(key) {
            let plaintext = res.decrypt(&self.session_key)?;
            Ok(Some(plaintext))
        } else {
            Ok(None)
        }
    }
}

impl Drop for McpSandbox {
    fn drop(&mut self) {
        // volatile_results wird gedroppt → Zeroizing<Vec<u8>> nullt den Speicher
        self.session_key.emergency_wipe();
        tracing::debug!("McpSandbox dropped: volatile tool results zeroized");
    }
}

// simple hex encoder helper to avoid external crate dependency overhead if needed, or use std formatting
mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Deckt OPUS-0.4 ab: Cloud-Egress ("contextra_cloud_query") ist in eigener ToolCategory::CloudEgress
    /// und im Default-Zustand (allow_cloud_egress: false) gesperrt.
    #[test]
    fn test_cloud_egress_classification_and_policy() {
        // 1. Classification check
        assert_eq!(
            McpSandbox::classify_method("contextra_cloud_query"),
            ToolCategory::CloudEgress
        );
        assert_eq!(
            McpSandbox::classify_method("contextra_search"),
            ToolCategory::DatabaseRead
        );

        // 2. Default policy: allow_cloud_egress is false
        let default_sandbox = McpSandbox::new(SandboxPolicy::default()).unwrap(); // unwrap
        let default_res = default_sandbox.validate_tool_call("contextra_cloud_query", &Value::Null);
        assert!(default_res.is_err());
        let err_msg = default_res.unwrap_err().to_string();
        assert!(
            err_msg.contains("Cloud-Egress gesperrt"),
            "Expected 'Cloud-Egress gesperrt' in error message, got: {err_msg}"
        );

        // 3. Permitting policy: allow_cloud_egress is true
        let policy = SandboxPolicy {
            allow_db_reads: true,
            allow_db_writes: false,
            allow_code_execution: false,
            allow_cloud_egress: true,
            max_execution_ms: 5_000,
        };
        let permitting_sandbox = McpSandbox::new(policy).unwrap(); // unwrap
        assert!(permitting_sandbox
            .validate_tool_call("contextra_cloud_query", &Value::Null)
            .is_ok());
    }

    /// Deckt R-01 ab: Schreibzugriff ist im Default gesperrt (Read-Only Safety Policy).
    #[test]
    fn test_sandbox_default_policy() {
        let sandbox = McpSandbox::new(SandboxPolicy::default()).unwrap(); // unwrap

        assert!(sandbox
            .validate_tool_call("contextra_search", &Value::Null)
            .is_ok());
        assert!(sandbox
            .validate_tool_call("contextra_get", &Value::Null)
            .is_ok());
        assert!(sandbox
            .validate_tool_call("contextra_collections", &Value::Null)
            .is_ok());

        assert!(sandbox
            .validate_tool_call("contextra_insert", &Value::Null)
            .is_err());
        assert!(sandbox
            .validate_tool_call("contextra_delete", &Value::Null)
            .is_err());
        assert!(sandbox
            .validate_tool_call("contextra_forget", &Value::Null)
            .is_err());
        assert!(sandbox
            .validate_tool_call("unknown_code_tool", &Value::Null)
            .is_err());
        assert!(sandbox
            .validate_tool_call("contextra_cloud_query", &Value::Null)
            .is_err());
    }

    /// Deckt R-01 ab: Schreibzugriff ist erlaubt, wenn `allow_db_writes` explizit aktiviert ist.
    #[test]
    fn test_sandbox_permitting_policy() {
        let policy = SandboxPolicy {
            allow_db_reads: true,
            allow_db_writes: true,
            allow_code_execution: true,
            allow_cloud_egress: true,
            max_execution_ms: 5_000,
        };
        let sandbox = McpSandbox::new(policy).unwrap(); // unwrap

        assert!(sandbox
            .validate_tool_call("contextra_search", &Value::Null)
            .is_ok());
        assert!(sandbox
            .validate_tool_call("contextra_insert", &Value::Null)
            .is_ok());
        assert!(sandbox
            .validate_tool_call("contextra_delete", &Value::Null)
            .is_ok());
        assert!(sandbox
            .validate_tool_call("contextra_forget", &Value::Null)
            .is_ok());
        assert!(sandbox
            .validate_tool_call("some_custom_tool", &Value::Null)
            .is_ok());
    }

    #[test]
    fn test_volatile_result_encryption_roundtrip() {
        let sandbox = McpSandbox::new(SandboxPolicy::default()).unwrap(); // unwrap
        let data = b"Top secret volatile tool result data";

        sandbox.store_volatile("res1", data).expect("store"); // expect

        let retrieved = sandbox.get_volatile("res1").expect("get").expect("exists"); // expect
        assert_eq!(retrieved.as_slice(), data);
    }

    #[test]
    fn volatile_tool_result_roundtrip() {
        let key =
            contextra_crypto::CryptoKey::try_new("0123456789abcdef0123456789abcdef", b"salt1234")
                .unwrap(); // unwrap
        let plaintext = b"tool output data";
        let result = VolatileToolResult::encrypt(plaintext, &key).unwrap(); // unwrap
        let decrypted = result.decrypt(&key).unwrap(); // unwrap
        assert_eq!(decrypted.as_slice(), plaintext);
    }

    #[test]
    fn test_volatile_result_error_path_zeroizes_intermediate_data() {
        let key =
            contextra_crypto::CryptoKey::try_new("0123456789abcdef0123456789abcdef", b"salt1234")
                .unwrap(); // unwrap
        let plaintext = b"sensitive payload that will drop early";
        let result = VolatileToolResult::encrypt(plaintext, &key).unwrap(); // unwrap

        let simulate_aborted_processing = || -> Result<()> {
            let decrypted = result.decrypt(&key)?;
            assert_eq!(decrypted.as_slice(), plaintext);
            // Simulate early return / error before processing finishes
            Err(ContextraError::Internal("Simulated pipeline failure".into()))
        };

        let err = simulate_aborted_processing();
        assert!(err.is_err());
        // The decrypted Zeroizing<Vec<u8>> dropped upon early exit, zeroizing memory.
    }

    #[tokio::test]
    async fn test_sandbox_execution_timeout() {
        let policy = SandboxPolicy {
            allow_db_reads: true,
            allow_db_writes: true,
            allow_code_execution: true,
            allow_cloud_egress: true,
            max_execution_ms: 50,
        };
        let sandbox = McpSandbox::new(policy).unwrap(); // unwrap

        let slow_fut = async {
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
            Ok::<(), ContextraError>(())
        };

        let res = sandbox.execute_with_timeout("slow_tool", slow_fut).await;
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("slow_tool"));
        assert!(err_msg.contains("überschritt Timeout 50ms"));
    }

    #[test]
    fn test_sandbox_validate_tool_call_method_length_checks() {
        let sandbox = McpSandbox::new(SandboxPolicy::default()).unwrap(); // unwrap

        // Empty method name
        let res_empty = sandbox.validate_tool_call("", &Value::Null);
        assert!(res_empty.is_err());
        assert!(res_empty
            .unwrap_err()
            .to_string()
            .contains("length invalid"));

        // Oversized method name
        let oversized = "a".repeat(257);
        let res_oversized = sandbox.validate_tool_call(&oversized, &Value::Null);
        assert!(res_oversized.is_err());
        assert!(res_oversized
            .unwrap_err()
            .to_string()
            .contains("length invalid"));
    }

    #[test]
    fn test_volatile_storage_boundary_guards() {
        let sandbox = McpSandbox::new(SandboxPolicy::default()).unwrap(); // unwrap

        // Empty key
        let res_empty_key = sandbox.store_volatile("", b"data");
        assert!(res_empty_key.is_err());

        // Oversized key (>256 bytes)
        let long_key = "k".repeat(257);
        let res_long_key = sandbox.store_volatile(&long_key, b"data");
        assert!(res_long_key.is_err());

        // Oversized output payload (>16MB)
        let huge_payload = vec![0u8; MAX_VOLATILE_OUTPUT_BYTES + 1];
        let res_huge = sandbox.store_volatile("huge", &huge_payload);
        assert!(res_huge.is_err());
        assert!(res_huge.unwrap_err().to_string().contains("exceeded"));
    }

    #[test]
    fn test_volatile_storage_capacity_limit() {
        let sandbox = McpSandbox::new(SandboxPolicy::default()).unwrap(); // unwrap

        for i in 0..MAX_VOLATILE_RESULTS {
            let key = format!("item_{i}");
            sandbox.store_volatile(&key, b"value").unwrap(); // unwrap
        }

        // Inserting item #1001 should fail
        let res_overflow = sandbox.store_volatile("overflow_key", b"value");
        assert!(res_overflow.is_err());
        assert!(res_overflow
            .unwrap_err()
            .to_string()
            .contains("limit reached"));
    }
}
