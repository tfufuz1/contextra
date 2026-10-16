---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "10"
---
## 10. Sicherheits- und Datenschutzmodell

### 10.1 Kryptographische Grundlagen

AES-256-GCM-SIV für Daten at rest, WAL mit race-freier HMAC-Kette, `DeletionProof` für DSGVO-Art.-17-Nachweise.

```rust
pub struct DeletionProof {
    pub key_hash: [u8; 32],
    pub hmac_chain_entry: [u8; 32],
    pub prev_hmac: [u8; 32],
    pub timestamp: i64,
}

pub struct HmacChain {
    last: std::sync::Mutex<[u8; 32]>,
}

impl HmacChain {
    /// MUSS atomar gegenüber gleichzeitigen `append`-Aufrufen sein.
    pub fn append(&self, payload: &[u8]) -> Result<[u8; 32], CryptoError>;
    pub fn verify_chain(entries: &[[u8; 32]]) -> Result<(), CryptoError>;
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("hmac chain fork at index {0}")]
    ChainFork(usize),
    #[error("aead operation failed")]
    AeadFailure,
}
```

### 10.2 WASM-Sandbox (🟢)

```rust
pub struct WasmCapabilities {
    pub max_fuel: Option<u64>,
    pub max_wall_clock_ms: u64, // Default 5000; 0 = unbegrenzt
    pub allow_cloud_egress: bool, // Default false
}

pub struct SandboxExecutor {
    engine: wasmtime::Engine,
}

impl SandboxExecutor {
    pub fn execute(&self, module: &[u8], caps: &WasmCapabilities) -> Result<Vec<u8>, SandboxError> {
        let mut store = wasmtime::Store::new(&self.engine, ());
        if let Some(fuel) = caps.max_fuel { store.set_fuel(fuel).map_err(SandboxError::from)?; }
        // Wall-Clock unabhängig von Fuel via separatem Timeout-Task (P23).
        unimplemented!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SandboxError {
    #[error("fuel budget exhausted")]
    FuelExhausted,
    #[error("wall clock budget of {0}ms exceeded")]
    WallClockExceeded(u64),
}
```

`fd_write`-WASI-Stub MUSS `iovs` korrekt parsen.

### 10.3 Prompt-Injection-Schutz

Eingaben aus dem Agenten-Kontext werden nicht ungeprüft als Steuerbefehle interpretiert; der Lesepfad ist
durchgehend Zero-Copy, um unnötige Pufferkopien sensibler Daten zu vermeiden.

### 10.4 Cloud-Egress Privacy Gateway (🟢, 5-Schichten-Architektur)

1. **Token-Vaulting/Pattern-Matching (`EgressVault`):** `RegexSet`-Klassifikation, Payload-Deckel.
2. **Vorabstraktion.**
3. **Graph-Generalisierung.**
4. **Bulk-Exfiltration-Detektor:** Großvolumige Anfragemuster erkennen.
5. **Re-Hydration:** `CloudResponseRehydrator::rehydrate` — Round-Trip-sicher, Multibyte-UTF-8-panic-sicher.

```rust
pub struct EgressVault {
    pattern_matcher: regex::RegexSet,
    surrogate_map: scc::HashMap<SurrogateToken, OriginalEntity>,
}

impl EgressVault {
    pub fn generate_surrogate(&self, entity: &OriginalEntity, session: SessionId) -> SurrogateToken;
    pub fn get_entity(&self, token: &SurrogateToken) -> Option<OriginalEntity>;
}

pub struct BulkExfiltrationDetector {
    pub max_bytes_per_window: usize,
    pub window: std::time::Duration,
}

pub struct CloudResponseRehydrator;
impl CloudResponseRehydrator {
    pub fn rehydrate(&self, response: &str, vault: &EgressVault) -> String { unimplemented!() }
}
```

**⚠️ Opus-Optimierung 0.4 — Egress-Klassifizierung (Stufe 0, gering):**
Eigene, restriktivere Policy-Kategorie für Cloud-Egress-Methoden statt gleiche wie lokale Lesezugriffe.

---

<a id="11-betrieb"></a>
