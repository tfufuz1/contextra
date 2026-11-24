# Schnittstellenspezifikation: `contextra-crypto`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-crypto` |
| Version | workspace |
| Pfad | `crates/contextra-crypto` |
| Rolle | Krypto Legacy (Passphrase, AES-GCM-SIV) |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # Kryptographie-Kern für Contextra (Ring 0)
>
> Dieses Modul bildet die kryptographische Basis des gesamten Contextra-Systems.
> Als Ring-0-Komponente (Kern) stellt es grundlegende Sicherheitsprimitive bereit,
> ohne selbst von höheren Schichten abzuhängen.
>
> ## Ring-Zugehörigkeit und Einbindung
> - **Ring:** Ring 0 (Kern / Foundation)
> - **Abhängigkeiten (ausgehend):** `contextra-ids`, `contextra-env`
> - **Wird eingebunden von:** `contextra`, `contextra-db`, `contextra-engine`,
> `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-privacy`,
> `contextra-store`, `contextra-vector`.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `aes-gcm-siv` | normal | registry | nein |  |
| `ahash` | normal | workspace | nein |  |
| `argon2` | normal | registry | nein |  |
| `bincode` | normal | workspace | nein |  |
| `blake3` | normal | workspace | nein |  |
| `contextra-crypto` | dev | path | nein | contextra-crypto, contextra-crypto-fuzz, contextra-infer-candle, contextra-kvcache, contextra-mcp, contextra-store |
| `contextra-env` | dev | path | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-infer-candle, contextra-kvcache, contextra-mvcc, contextra-ports, contextra-store, contextra-text, contextra-vector |
| `contextra-ids` | normal | path | nein | contextra-checkpoint, contextra-crypto, contextra-graph, contextra-mvcc, contextra-store, contextra-text, contextra-types, contextra-vector |
| `contextra-ports` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-rank, contextra-router, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vector, contextra-vector-fuzz |
| `contextra-types` | dev | path | nein | contextra, contextra-adapt, contextra-agent, contextra-bench, contextra-checkpoint, contextra-cognition, contextra-crypto, contextra-crypto-fuzz, contextra-db, contextra-engine, contextra-graph, contextra-graph-fuzz, contextra-infer-candle, contextra-infer-ollama, contextra-infer-onnx, contextra-ingest, contextra-kvcache, contextra-mcp, contextra-mvcc, contextra-ports, contextra-py, contextra-rank, contextra-router, contextra-simd, contextra-store, contextra-store-fuzz, contextra-testkit, contextra-text, contextra-vault, contextra-vector, contextra-vector-fuzz |
| `criterion` | dev | workspace | nein |  |
| `hkdf` | normal | registry | nein |  |
| `hmac` | normal | registry | nein |  |
| `loom` | dev | registry | nein |  |
| `lru` | normal | registry | nein |  |
| `parking_lot` | normal | workspace | nein |  |
| `proptest` | dev | registry | nein |  |
| `rand` | normal | workspace | nein |  |
| `regex` | normal | workspace | nein |  |
| `serde` | normal | workspace | nein |  |
| `serde_json` | normal | workspace | nein |  |
| `sha2` | normal | registry | nein |  |
| `subtle` | normal | registry | nein |  |
| `tempfile` | dev | workspace | nein |  |
| `thiserror` | normal | workspace | nein |  |
| `tokio` | dev | workspace | nein |  |
| `tracing` | normal | workspace | nein |  |
| `zeroize` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-crypto`, `contextra-crypto-fuzz`, `contextra-infer-candle`, `contextra-kvcache`, `contextra-mcp`, `contextra-store`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `cloud-egress-guard` | – |
| `default` | `kv-encryption` |
| `kv-encryption` | – |
| `ports-compat` | `workspace` |
| `test-utils` | – |
| `types-compat` | `workspace` |
| `workspace` | `dep:contextra-types`, `dep:contextra-ports` |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 22 |
| Quellzeilen | 6290 |
| Öffentliche Items (Modul-Ebene) | 96 |
| davon `fn` | 14 |
| davon `struct` | 34 |
| davon `enum` | 3 |
| davon `trait` | 4 |
| davon `type` | 4 |
| davon `const` | 16 |
| davon `mod` | 21 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 21 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 0 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 25 |
| `#[deprecated]` | 5 |
| `#[cfg(feature…)]`-Gates | 29 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 218 / 218 = 100 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `aead` | `crates/contextra-crypto/src/aead.rs` | 207 |
| `anti_tamper` | `crates/contextra-crypto/src/anti_tamper.rs` | 210 |
| `audit_chain` | `crates/contextra-crypto/src/audit_chain.rs` | 374 |
| `compat::error` | `crates/contextra-crypto/src/compat/error.rs` | 165 |
| `compat` | `crates/contextra-crypto/src/compat/mod.rs` | 7 |
| `compat::model_fingerprint` | `crates/contextra-crypto/src/compat/model_fingerprint.rs` | 44 |
| `contract` | `crates/contextra-crypto/src/contract.rs` | 154 |
| `crypto` | `crates/contextra-crypto/src/crypto.rs` | 1041 |
| `dek` | `crates/contextra-crypto/src/dek.rs` | 170 |
| `error` | `crates/contextra-crypto/src/error.rs` | 96 |
| `kdf` | `crates/contextra-crypto/src/kdf.rs` | 319 |
| `kv_cipher` | `crates/contextra-crypto/src/kv_cipher.rs` | 365 |
| `kv_segment::delete_mode` | `crates/contextra-crypto/src/kv_segment/delete_mode.rs` | 55 |
| `kv_segment::eviction_worker` | `crates/contextra-crypto/src/kv_segment/eviction_worker.rs` | 220 |
| `kv_segment::manager` | `crates/contextra-crypto/src/kv_segment/manager.rs` | 141 |
| `kv_segment` | `crates/contextra-crypto/src/kv_segment/mod.rs` | 11 |
| `kv_segment::segment` | `crates/contextra-crypto/src/kv_segment/segment.rs` | 356 |
| `kv_segment::store` | `crates/contextra-crypto/src/kv_segment/store.rs` | 859 |
| `crate` | `crates/contextra-crypto/src/lib.rs` | 94 |
| `secret` | `crates/contextra-crypto/src/secret.rs` | 162 |
| `wal_completeness` | `crates/contextra-crypto/src/wal_completeness.rs` | 77 |
| `wal_crypto` | `crates/contextra-crypto/src/wal_crypto.rs` | 1163 |

## 6. Re-Exports (`pub use`)

- `pub use delete_mode::KvDeleteMode` — `crates/contextra-crypto/src/kv_segment/mod.rs:7` (Modul `kv_segment`)
- `pub use eviction_worker::` — `crates/contextra-crypto/src/kv_segment/mod.rs:8` (Modul `kv_segment`)
- `pub use manager::` — `crates/contextra-crypto/src/kv_segment/mod.rs:9` (Modul `kv_segment`)
- `pub use segment::` — `crates/contextra-crypto/src/kv_segment/mod.rs:10` (Modul `kv_segment`)
- `pub use store::TenantIsolatedKvStore` — `crates/contextra-crypto/src/kv_segment/mod.rs:11` (Modul `kv_segment`)
- `pub use aead::*` — `crates/contextra-crypto/src/lib.rs:63` (Modul `crate`)
- `pub use contract::*` — `crates/contextra-crypto/src/lib.rs:64` (Modul `crate`)
- `pub use dek::` — `crates/contextra-crypto/src/lib.rs:65` (Modul `crate`)
- `pub use kdf::hkdf_expand` — `crates/contextra-crypto/src/lib.rs:66` (Modul `crate`)
- `pub use secret::SecretKey` — `crates/contextra-crypto/src/lib.rs:67` (Modul `crate`)
- `pub use audit_chain::` — `crates/contextra-crypto/src/lib.rs:69` (Modul `crate`)
- `pub use crypto::KeyManager as CryptoKey` — `crates/contextra-crypto/src/lib.rs:73` (Modul `crate`)
- `pub use crypto::KeyManager` — `crates/contextra-crypto/src/lib.rs:74` (Modul `crate`)
- `pub use error::` — `crates/contextra-crypto/src/lib.rs:75` (Modul `crate`)
- `pub use kdf::` — `crates/contextra-crypto/src/lib.rs:76` (Modul `crate`)
- `pub use kv_cipher::KvSegmentCipher` — `crates/contextra-crypto/src/lib.rs:78` (Modul `crate`)
- `pub use kv_cipher::` — `crates/contextra-crypto/src/lib.rs:79` (Modul `crate`)
- `pub use kv_segment::` — `crates/contextra-crypto/src/lib.rs:80` (Modul `crate`)
- `pub use wal_completeness::verify_wal_chain_completeness` — `crates/contextra-crypto/src/lib.rs:88` (Modul `crate`)
- `pub use wal_crypto::CloudEgressGuard` — `crates/contextra-crypto/src/lib.rs:90` (Modul `crate`)
- `pub use wal_crypto::` — `crates/contextra-crypto/src/lib.rs:91` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `aead`

#### `fn open_record`
- Ort: `crates/contextra-crypto/src/aead.rs:81`
```rust
pub fn open_record(dek: &Dek, key_bytes: &[u8], env: &[u8]) -> Result<Zeroizing<Vec<u8>>>
```
Dokumentation: Opens an encrypted record envelope, verifying the associated data and authentication tag. Fails with a uniform `CryptoError::Aead` on any authentication or corruption issue without leaking oracle details.

#### `fn seal_record`
- Ort: `crates/contextra-crypto/src/aead.rs:70`
```rust
pub fn seal_record(dek: &Dek, doc: [u8
```

#### `fn seal_record_with_nonce`
- Ort: `crates/contextra-crypto/src/aead.rs:24`
```rust
pub fn seal_record_with_nonce( dek: &Dek, doc: [u8
```
Dokumentation: Seals a plaintext record into an envelope using an explicit, caller-provided 12-byte nonce. Binds `doc` and `key_bytes` into the Associated Data (AAD) preventing replay/reordering across documents.

#### `fn seal_record_with_rng`
- Ort: `crates/contextra-crypto/src/aead.rs:56`
```rust
pub fn seal_record_with_rng<R: RngCore>( dek: &Dek, doc: [u8
```

#### `const REC_AAD`
- Ort: `crates/contextra-crypto/src/aead.rs:15`
```rust
pub const REC_AAD: &[u8] = b""
```

### Modul `anti_tamper`

#### `struct Blake3Chain`
- Ort: `crates/contextra-crypto/src/anti_tamper.rs:111`
```rust
pub struct Blake3Chain
```
- Impl-Block `crates/contextra-crypto/src/anti_tamper.rs:116`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(genesis: [u8` | `crates/contextra-crypto/src/anti_tamper.rs:118` | Initialisiert eine neue BLAKE3-Kette mit einem Seed/Genesis-Hash. |
| `pub fn extend(&mut self, context: &[u8], data: &[u8]) -> [u8` | `crates/contextra-crypto/src/anti_tamper.rs:126` | Aktualisiert den Zustand der Kette mit einem neuen Block und Kontext in AAD-Disziplin. |
| `pub fn current_hash(&self) -> &[u8` | `crates/contextra-crypto/src/anti_tamper.rs:142` | Gibt den aktuellen Ketten-Hash zurück. |
| `pub fn step_count(&self) -> u64` | `crates/contextra-crypto/src/anti_tamper.rs:147` | Gibt die bisherige Anzahl an Kettengliedern zurück. |
| `pub fn verify_current_hash(&self, expected: &[u8` | `crates/contextra-crypto/src/anti_tamper.rs:152` | Verifiziert in konstanter Zeit, ob der aktuelle Hash mit einem erwarteten Wert übereinstimmt. |

#### `struct VolatileEncryptionKey`
- Ort: `crates/contextra-crypto/src/anti_tamper.rs:30`
```rust
pub struct VolatileEncryptionKey
```
- Impl-Block `crates/contextra-crypto/src/anti_tamper.rs:34`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-crypto/src/anti_tamper.rs:60`
- Impl-Block (Trait `PartialEq`) `crates/contextra-crypto/src/anti_tamper.rs:68`
- Impl-Block (Trait `Eq`) `crates/contextra-crypto/src/anti_tamper.rs:77`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(raw: [u8` | `crates/contextra-crypto/src/anti_tamper.rs:36` | Creates a new volatile key from a raw 32-byte array. |
| `pub fn emergency_wipe(&mut self)` | `crates/contextra-crypto/src/anti_tamper.rs:44` | Emergency Trigger: Explicitly wipes the key from memory. This is to be hooked into hardware sensors (e.g. crash/voltage). |
| `pub fn as_bytes(&self) -> &[u8]` | `crates/contextra-crypto/src/anti_tamper.rs:49` | Accessor for cryptographic operations. |
| `pub fn inspect_key_bytes_for_test(&self) -> &[u8` | `crates/contextra-crypto/src/anti_tamper.rs:55` | Test-only method to inspect key bytes. |

#### `fn constant_time_compare`
- Ort: `crates/contextra-crypto/src/anti_tamper.rs:87`
```rust
pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool
```
Dokumentation: Konstantzeit-Verifikation zweier Byte-Slices (`INV-CRYPTO-CONSTANT-TIME-1`). Verhindert Timing-Orakel bei MAC-, Tag- und Prüfsummen-Validierungen.

#### `fn verify_tag_constant_time`
- Ort: `crates/contextra-crypto/src/anti_tamper.rs:97`
```rust
pub fn verify_tag_constant_time(expected: &[u8], actual: &[u8]) -> Result<()>
```

### Modul `audit_chain`

#### `struct AuditChain`
- Ort: `crates/contextra-crypto/src/audit_chain.rs:185`
```rust
pub struct AuditChain
```
Dokumentation: Manipulationssichere Audit-Kette mit BLAKE3-Hash-Verkettung.
- Impl-Block `crates/contextra-crypto/src/audit_chain.rs:191`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `entries` | `Vec<AuditChainEntry>` | `crates/contextra-crypto/src/audit_chain.rs:187` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-crypto/src/audit_chain.rs:193` | Erzeugt eine neue, leere Audit-Kette. |
| `pub fn append( &mut self, schema_id: impl Into<String>, rules_hash: [u8` | `crates/contextra-crypto/src/audit_chain.rs:202` | Fügt einen neuen Eintrag zur Audit-Kette hinzu und aktualisiert den Kettenkopf-Hash. |
| `pub fn len(&self) -> usize` | `crates/contextra-crypto/src/audit_chain.rs:246` | Liefert die Anzahl der Einträge in der Kette. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-crypto/src/audit_chain.rs:251` | Prüft, ob die Kette leer ist. |
| `pub fn entries(&self) -> &[AuditChainEntry]` | `crates/contextra-crypto/src/audit_chain.rs:256` | Gibt eine Referenz auf alle Einträge der Kette zurück. |
| `pub fn head_hash(&self) -> [u8` | `crates/contextra-crypto/src/audit_chain.rs:261` | Liefert den aktuellen Kettenkopf-Hash. |
| `pub fn sign_head_with_bytes( &self, signature_bytes: Vec<u8>, ) -> Result<AuditChainHeadSignature>` | `crates/contextra-crypto/src/audit_chain.rs:266` | Periodischer Aufruf: Erzeugt eine Kettenkopf-Signatur / Snapshot. |
| `pub fn verify_chain(&self) -> Result<bool>` | `crates/contextra-crypto/src/audit_chain.rs:298` | Verifiziert die vollständige Kette auf Integrität. # Warnhinweis / Security Warning Diese Methode prüft lediglich die **interne kryptographische Hash-Verkettung |
| `pub fn verify_chain_with_anchor( &self, anchor: &AuditHeadAnchor, ) -> Result<bool>` | `crates/contextra-crypto/src/audit_chain.rs:346` | Verifiziert die Kette gegen einen externen Kopf-Anker ([`AuditHeadAnchor`]). Schlägt fehl (`Ok(false)`), wenn: 1. Die Kette kürzer ist als vom Anker gefordert ( |

#### `struct AuditChainEntry`
- Ort: `crates/contextra-crypto/src/audit_chain.rs:52`
```rust
pub struct AuditChainEntry
```
Dokumentation: Einzelner Eintrag in der manipulationssicheren BLAKE3-Audit-Hash-Kette.
- Impl-Block `crates/contextra-crypto/src/audit_chain.rs:73`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `index` | `u64` | `crates/contextra-crypto/src/audit_chain.rs:54` |
| `schema_id` | `String` | `crates/contextra-crypto/src/audit_chain.rs:56` |
| `rules_hash` | `[u8; 32]` | `crates/contextra-crypto/src/audit_chain.rs:58` |
| `data_class` | `DataClass` | `crates/contextra-crypto/src/audit_chain.rs:60` |
| `doc_id` | `DocId` | `crates/contextra-crypto/src/audit_chain.rs:62` |
| `tx_id` | `TxId` | `crates/contextra-crypto/src/audit_chain.rs:64` |
| `prev_hash` | `[u8; 32]` | `crates/contextra-crypto/src/audit_chain.rs:66` |
| `commitment` | `Option<[u8; 32]>` | `crates/contextra-crypto/src/audit_chain.rs:68` |
| `entry_hash` | `[u8; 32]` | `crates/contextra-crypto/src/audit_chain.rs:70` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn compute_hash( index: u64, schema_id: &str, rules_hash: &[u8` | `crates/contextra-crypto/src/audit_chain.rs:76` | Berechnet den Blake3-Hash für die Felder eines Audit-Ketten-Eintrags. |

#### `struct AuditChainHeadSignature`
- Ort: `crates/contextra-crypto/src/audit_chain.rs:116`
```rust
pub struct AuditChainHeadSignature
```
Dokumentation: Periodisch erzeugte Signatur über den aktuellen Kettenkopf.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `chain_index` | `u64` | `crates/contextra-crypto/src/audit_chain.rs:118` |
| `head_hash` | `[u8; 32]` | `crates/contextra-crypto/src/audit_chain.rs:120` |
| `signature` | `Vec<u8>` | `crates/contextra-crypto/src/audit_chain.rs:122` |

#### `struct AuditHeadAnchor`
- Ort: `crates/contextra-crypto/src/audit_chain.rs:127`
```rust
pub struct AuditHeadAnchor
```
Dokumentation: Externer Kopf-Anker zur Rollback-Erkennung für die Audit-Hash-Kette.
- Impl-Block `crates/contextra-crypto/src/audit_chain.rs:136`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `seq_no` | `u64` | `crates/contextra-crypto/src/audit_chain.rs:129` |
| `head_hash` | `[u8; 32]` | `crates/contextra-crypto/src/audit_chain.rs:131` |
| `head_signature` | `Option<AuditChainHeadSignature>` | `crates/contextra-crypto/src/audit_chain.rs:133` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(seq_no: u64, head_hash: [u8` | `crates/contextra-crypto/src/audit_chain.rs:138` | Erstellt einen neuen nicht signierten `AuditHeadAnchor`. |
| `pub fn with_signature( seq_no: u64, head_hash: [u8` | `crates/contextra-crypto/src/audit_chain.rs:147` | Erstellt einen neuen signierten `AuditHeadAnchor`. |

#### `struct EncryptedCommitmentSalt`
- Ort: `crates/contextra-crypto/src/audit_chain.rs:162`
```rust
pub struct EncryptedCommitmentSalt
```
Dokumentation: Verschlüsselter Salt-Wert für ein Record-Commitment.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `group_id` | `u64` | `crates/contextra-crypto/src/audit_chain.rs:164` |
| `ciphertext` | `Vec<u8>` | `crates/contextra-crypto/src/audit_chain.rs:166` |
| `nonce` | `[u8; 12]` | `crates/contextra-crypto/src/audit_chain.rs:168` |

#### `enum DataClass`
- Ort: `crates/contextra-crypto/src/audit_chain.rs:33`
```rust
pub enum DataClass
```
Dokumentation: Datenklassifikation für Audit-Einträge (keine Freitext-Strings).

Varianten:

- `Public` — `crates/contextra-crypto/src/audit_chain.rs:35`
- `Internal` — `crates/contextra-crypto/src/audit_chain.rs:37`
- `Confidential` — `crates/contextra-crypto/src/audit_chain.rs:39`
- `Restricted` — `crates/contextra-crypto/src/audit_chain.rs:41`
- `Personal` — `crates/contextra-crypto/src/audit_chain.rs:43`
- `SpecialCategoryPersonal` — `crates/contextra-crypto/src/audit_chain.rs:45`

#### `fn compute_record_commitment`
- Ort: `crates/contextra-crypto/src/audit_chain.rs:174`
```rust
pub fn compute_record_commitment(salt: &[u8], attr: &[u8]) -> [u8
```
Dokumentation: Berechnet das Commitment `Blake3(salt || attr)`.

### Modul `compat`

#### `mod error`
- Ort: `crates/contextra-crypto/src/compat/mod.rs:6`
```rust
pub mod error
```

#### `mod model_fingerprint`
- Ort: `crates/contextra-crypto/src/compat/mod.rs:7`
```rust
pub mod model_fingerprint
```

### Modul `contract`

#### `struct DeterministicEntropy`
- Ort: `crates/contextra-crypto/src/contract.rs:25`
```rust
pub struct DeterministicEntropy
```
Dokumentation: Standard-Implementierung von [`CryptoEntropy`] basierend auf einem deterministischen Generator.
- Impl-Block `crates/contextra-crypto/src/contract.rs:29`
- Impl-Block (Trait `CryptoEntropy`) `crates/contextra-crypto/src/contract.rs:38`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(seed: u64) -> Self` | `crates/contextra-crypto/src/contract.rs:31` | Erzeugt eine neue deterministische Entropiequelle mit festem Startwert (Seed). |

#### `struct InMemoryKms`
- Ort: `crates/contextra-crypto/src/contract.rs:70`
```rust
pub struct InMemoryKms
```
Dokumentation: I/O-freie In-Memory Referenzimplementierung von [`KmsContract`].
- Impl-Block `crates/contextra-crypto/src/contract.rs:75`
- Impl-Block (Trait `KmsContract`) `crates/contextra-crypto/src/contract.rs:85`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(active_id: impl Into<String>, master_key: [u8` | `crates/contextra-crypto/src/contract.rs:77` | Erzeugt eine neue `InMemoryKms`-Instanz mit einem 32-Byte Master-Schlüssel. |

#### `trait CryptoEntropy`
- Ort: `crates/contextra-crypto/src/contract.rs:18`
```rust
pub trait CryptoEntropy: Send + Sync
```
Dokumentation: Trait zur Bereitstellung kryptographischer Entropie (Zufallsbytes). Ermöglicht die Injektion deterministischer Test-Zufallsquellen oder sicherer CSPRNGs.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn fill_bytes(&mut self, dest: &mut [u8])` | `crates/contextra-crypto/src/contract.rs:20` | Füllt den übergebenen Puffer mit pseudozufälligen Bytes. |

#### `trait KmsContract`
- Ort: `crates/contextra-crypto/src/contract.rs:52`
```rust
pub trait KmsContract: Send + Sync
```
Dokumentation: Schnittstelle für Key Management Services (KMS) zur sicheren Schlüsselableitung und Hüllkurven-Verschlüsselung.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get_key(&self) -> Result<Zeroizing<Vec<u8>>>` | `crates/contextra-crypto/src/contract.rs:54` | Liefert den aktiven Datenverschlüsselungsschlüssel (DEK), genullt bei Freigabe. |
| `fn active_key_id(&self) -> &str` | `crates/contextra-crypto/src/contract.rs:57` | Liefert die Kennung (ID) des aktiven Schlüssels. |
| `fn derive_data_key(&self, key_id: &[u8]) -> Result<Zeroizing<Vec<u8>>>` | `crates/contextra-crypto/src/contract.rs:60` | Leitet einen abgeleiteten Schlüssel anhand einer `key_id` ab. |
| `fn wrap_key(&self, key_id: &[u8], plaintext: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-crypto/src/contract.rs:63` | Wickelt (verschlüsselt) Daten mit dem angegebenen Schlüssel ein. |
| `fn unwrap_key(&self, key_id: &[u8], wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>>` | `crates/contextra-crypto/src/contract.rs:66` | Entpackt (entschlüsselt) Daten mit dem angegebenen Schlüssel. |

### Modul `crate`

#### `mod aead`
- Ort: `crates/contextra-crypto/src/lib.rs:43`
```rust
pub mod aead
```

#### `mod anti_tamper`
- Ort: `crates/contextra-crypto/src/lib.rs:44`
```rust
pub mod anti_tamper
```

#### `mod audit_chain`
- Ort: `crates/contextra-crypto/src/lib.rs:45`
```rust
pub mod audit_chain
```

#### `mod compat`
- Ort: `crates/contextra-crypto/src/lib.rs:48`
```rust
pub mod compat
```
- Gate: `#[cfg(feature = "workspace")]`

#### `mod contract`
- Ort: `crates/contextra-crypto/src/lib.rs:46`
```rust
pub mod contract
```

#### `mod crypto`
- Ort: `crates/contextra-crypto/src/lib.rs:49`
```rust
pub mod crypto
```

#### `mod dek`
- Ort: `crates/contextra-crypto/src/lib.rs:50`
```rust
pub mod dek
```

#### `mod error`
- Ort: `crates/contextra-crypto/src/lib.rs:51`
```rust
pub mod error
```

#### `mod kdf`
- Ort: `crates/contextra-crypto/src/lib.rs:52`
```rust
pub mod kdf
```

#### `mod kv_cipher`
- Ort: `crates/contextra-crypto/src/lib.rs:53`
```rust
pub mod kv_cipher
```

#### `mod kv_segment`
- Ort: `crates/contextra-crypto/src/lib.rs:54`
```rust
pub mod kv_segment
```

#### `mod secret`
- Ort: `crates/contextra-crypto/src/lib.rs:55`
```rust
pub mod secret
```

#### `mod wal_completeness`
- Ort: `crates/contextra-crypto/src/lib.rs:56`
```rust
pub mod wal_completeness
```

#### `mod wal_crypto`
- Ort: `crates/contextra-crypto/src/lib.rs:57`
```rust
pub mod wal_crypto
```

### Modul `crypto`

#### `struct KeyManager`
- Ort: `crates/contextra-crypto/src/crypto.rs:65`
```rust
pub struct KeyManager
```
Dokumentation: Zentraler kryptographischer Schlüsselmanager für symmetrische Verschlüsselung. Hält den Master-Key im RAM, leitet per HKDF-Expand datei- und mandantenspezifische Teilschlüssel ab und kapselt symmetrische AEAD-Ver- und Entschlüsselung.
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-crypto/src/crypto.rs:72`
- Impl-Block `crates/contextra-crypto/src/crypto.rs:81`
- Impl-Block (Trait `KvCipher`) `crates/contextra-crypto/src/kv_cipher.rs:101`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self>` | `crates/contextra-crypto/src/crypto.rs:88` | Creates a new KeyManager by deriving a key from a passphrase using HKDF-SHA256. # WARNHINWEIS Diese Funktion nutzt HKDF-SHA256 und setzt voraus, dass `passphras |
| `pub fn try_new_with_kdf(passphrase: &str, header: &KdfHeader) -> Result<Self>` | `crates/contextra-crypto/src/crypto.rs:130` | Creates a new KeyManager using Argon2id password-based key derivation. Nutzt Argon2id mit den im [`KdfHeader`] definierten Kostenparametern und Salt, um eine si |
| `pub fn try_new_random_salt(passphrase: &str) -> Result<(Self, [u8` | `crates/contextra-crypto/src/crypto.rs:151` | Creates a new KeyManager with a cryptographically secure random salt. |
| `pub fn derive_file_key(&self, file_id: &[u8]) -> Result<Self>` | `crates/contextra-crypto/src/crypto.rs:160` | Derives a sub-key for a specific file or logical stream. This prevents nonce-reuse when multiple files use the same master key. |
| `pub fn derive_segment_key(&self, info: &str) -> Result<Self>` | `crates/contextra-crypto/src/crypto.rs:206` | Derives a sub-key specifically for a KV-cache segment based on a versioned info string. |
| `pub fn derive_kv_key_bytes( &self, tenant_id: contextra_ids::TenantId, fingerprint: &[u8` | `crates/contextra-crypto/src/crypto.rs:261` | Derives a sub-key for KV-cache segment encryption using raw 32-byte model fingerprint. |
| `pub fn derive_kv_key_scoped_bytes( &self, scoped_fingerprint: contextra_ids::TenantScoped<&[u8` | `crates/contextra-crypto/src/crypto.rs:275` | Derives a sub-key for KV-cache segment encryption with strict compile-time tenant scope enforcement using raw 32-byte model fingerprint. |
| `pub fn cipher_for_bytes( &self, tenant_id: contextra_ids::TenantId, fingerprint: &[u8` | `crates/contextra-crypto/src/crypto.rs:287` | Derives a tenant-isolated `KeyManager` instance for a specific `TenantId` with raw 32-byte fingerprint. |
| `pub fn cipher_for_scoped_bytes<T>( &self, scoped_key: contextra_ids::TenantScoped<T>, expected_tenant_id: &contextra_ids::TenantId, fingerprint: &[u8` | `crates/contextra-crypto/src/crypto.rs:297` | Derives a tenant-isolated `KeyManager` instance enforcing compile-time `TenantScoped` verification with raw 32-byte fingerprint. |
| `pub fn derive_kv_key( &self, tenant_id: contextra_ids::TenantId, model_fingerprint: &crate::kv_cipher::ModelFingerprint, ) -> Result<Self>` | `crates/contextra-crypto/src/crypto.rs:312` | All KV-cache segments encrypted before v2 are cryptographically unreadable under v2 keys and will fail at the `format_version` check in `KvSegmentCipher::decryp |
| `pub fn derive_kv_key_scoped( &self, scoped_fingerprint: contextra_ids::TenantScoped<&crate::kv_cipher::ModelFingerprint>, expected_tenant_id: &contextra_ids::TenantId, ) -> Result<Self>` | `crates/contextra-crypto/src/crypto.rs:347` | Derives a sub-key for KV-cache segment encryption with strict compile-time tenant scope enforcement. Unpacks `scoped_fingerprint` only if the inner bound `Tenan |
| `pub fn cipher_for(&self, tenant_id: contextra_ids::TenantId) -> Result<Self>` | `crates/contextra-crypto/src/crypto.rs:359` | Derives a tenant-isolated `KeyManager` instance for a specific `TenantId`. |
| `pub fn cipher_for_scoped<T>( &self, scoped_key: contextra_ids::TenantScoped<T>, expected_tenant_id: &contextra_ids::TenantId, ) -> Result<Self>` | `crates/contextra-crypto/src/crypto.rs:370` | Derives a tenant-isolated `KeyManager` instance enforcing compile-time `TenantScoped` verification. Unpacks `scoped_key` only if the bound `TenantId` matches `e |
| `pub fn integrity_key(&self) -> Result<[u8` | `crates/contextra-crypto/src/crypto.rs:382` | Derives an integrity key for HMAC-SHA256. |
| `pub fn encrypt_auto_nonce(&self, data: &[u8]) -> Result<(Vec<u8>, [u8` | `crates/contextra-crypto/src/crypto.rs:406` | Encrypts a block of data with a deterministically generated 12-byte nonce. Returns the ciphertext and the full 12-byte nonce used. |
| `pub fn decrypt_auto_nonce(&self, ciphertext: &[u8], nonce_bytes: &[u8` | `crates/contextra-crypto/src/crypto.rs:437` | Decrypts a block of data using a full 12-byte nonce. |
| `pub fn emergency_wipe(&mut self)` | `crates/contextra-crypto/src/crypto.rs:446` | Emergency Trigger: Explicitly wipes the key from memory and clears cipher cache. |
| `pub fn export_dek(&self) -> Zeroizing<Vec<u8>>` | `crates/contextra-crypto/src/crypto.rs:453` | Exports the 32-byte raw Data Encryption Key (DEK) wrapped in [`Zeroizing<Vec<u8>>`]. |
| `pub fn inspect_key_bytes_for_test(&self) -> &[u8` | `crates/contextra-crypto/src/crypto.rs:459` | Provides access to the key bytes ONLY during testing. |

### Modul `dek`

#### `struct AncestorSalts`
- Ort: `crates/contextra-crypto/src/dek.rs:51`
```rust
pub struct AncestorSalts
```

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `t` | `Salt` | `crates/contextra-crypto/src/dek.rs:52` |
| `c` | `Salt` | `crates/contextra-crypto/src/dek.rs:53` |
| `s` | `Option<Salt>` | `crates/contextra-crypto/src/dek.rs:54` |
| `d` | `Salt` | `crates/contextra-crypto/src/dek.rs:55` |

#### `struct Dek`
- Ort: `crates/contextra-crypto/src/dek.rs:35`
```rust
pub struct Dek(pub SecretKey<32>)
```
- Impl-Block `crates/contextra-crypto/src/dek.rs:37`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(bytes: [u8` | `crates/contextra-crypto/src/dek.rs:38` |  |
| `pub fn expose_secret(&self) -> &[u8` | `crates/contextra-crypto/src/dek.rs:42` |  |

#### `struct UnitKey`
- Ort: `crates/contextra-crypto/src/dek.rs:20`
```rust
pub struct UnitKey(pub SecretKey<32>)
```
- Impl-Block `crates/contextra-crypto/src/dek.rs:22`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(bytes: [u8` | `crates/contextra-crypto/src/dek.rs:23` |  |
| `pub fn expose_secret(&self) -> &[u8` | `crates/contextra-crypto/src/dek.rs:27` |  |

#### `fn derive_dek`
- Ort: `crates/contextra-crypto/src/dek.rs:68`
```rust
pub fn derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: DocTag) -> Dek
```
Dokumentation: Derives a DEK using AncestorSalts and DocTag.

#### `fn unwrap_dek`
- Ort: `crates/contextra-crypto/src/dek.rs:111`
```rust
pub fn unwrap_dek(kek: &UnitKey, wrapped: &[u8]) -> Result<Dek>
```

#### `fn wrap_dek`
- Ort: `crates/contextra-crypto/src/dek.rs:93`
```rust
pub fn wrap_dek(kek: &UnitKey, dek: &Dek, nonce: [u8
```

#### `type DocTag`
- Ort: `crates/contextra-crypto/src/dek.rs:48`
```rust
pub type DocTag = [u8
```

#### `type Salt`
- Ort: `crates/contextra-crypto/src/dek.rs:47`
```rust
pub type Salt = [u8
```

#### `const DEK_LABEL`
- Ort: `crates/contextra-crypto/src/dek.rs:58`
```rust
pub const DEK_LABEL: &[u8] = b""
```

#### `const DEK_SALT_LABEL`
- Ort: `crates/contextra-crypto/src/dek.rs:59`
```rust
pub const DEK_SALT_LABEL: &[u8] = b""
```

#### `const WRAP_AAD`
- Ort: `crates/contextra-crypto/src/dek.rs:60`
```rust
pub const WRAP_AAD: &[u8] = b""
```

### Modul `error`

#### `enum CryptoError`
- Ort: `crates/contextra-crypto/src/error.rs:27`
```rust
pub enum CryptoError
```
- Impl-Block (Trait `From<ContextraError>`) `crates/contextra-crypto/src/compat/error.rs:22`
- Impl-Block `crates/contextra-crypto/src/error.rs:82`

Varianten:

- `KeyDerivation(String)` — `crates/contextra-crypto/src/error.rs:29`
- `Encryption(String)` — `crates/contextra-crypto/src/error.rs:32`
- `Decryption(String)` — `crates/contextra-crypto/src/error.rs:35`
- `Aead(String)` — `crates/contextra-crypto/src/error.rs:38`
- `InvalidLength(String)` — `crates/contextra-crypto/src/error.rs:41`
- `IntegrityViolation` — `crates/contextra-crypto/src/error.rs:44`
- `InvalidInput(String)` — `crates/contextra-crypto/src/error.rs:47`
- `Crypto(String)` — `crates/contextra-crypto/src/error.rs:50`
- `KvFormatVersionMismatch{` — `crates/contextra-crypto/src/error.rs:53`
- `WalCorruption{` — `crates/contextra-crypto/src/error.rs:61`
- `WalTruncationDetected` — `crates/contextra-crypto/src/error.rs:70`
- `KeyRevoked(String)` — `crates/contextra-crypto/src/error.rs:73`
- `Storage(String)` — `crates/contextra-crypto/src/error.rs:76`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self` | `crates/contextra-crypto/src/error.rs:84` | Helper to construct a `WalCorruption` error. |
| `pub fn storage(reason: impl Into<String>) -> Self` | `crates/contextra-crypto/src/error.rs:92` | Helper to construct a `Storage` error. |

#### `type Result`
- Ort: `crates/contextra-crypto/src/error.rs:17`
```rust
pub type Result<T> = std::result::Result<T, CryptoError>
```
Dokumentation: Result type alias for cryptographic operations in `contextra-crypto`.

### Modul `kdf`

#### `struct DerivedKey`
- Ort: `crates/contextra-crypto/src/kdf.rs:274`
```rust
pub struct DerivedKey(pub [u8
```
Dokumentation: Wrapper für abgeleitetes 256-Bit Schlüsselmaterial, das bei Drop automatisch aus dem Speicher gelöscht wird.

#### `struct KdfHeader`
- Ort: `crates/contextra-crypto/src/kdf.rs:123`
```rust
pub struct KdfHeader
```
Dokumentation: Versionierter KDF-Metadaten-Header zur deterministischen Reproduktion von Schlüsseln. Speichert die Argon2id-Kostenparameter (m_cost, t_cost, p_cost) und den Salt, um verschlüsselte Datenbankdateien auch nach Server-Neustarts mit identischen Parametern entsperren zu können. ### Binär-Layout (Big-Endian): | Offset | Länge | Feld | Beschreibung | |---|---|---|---| | 0..4 | 4B | Magic | "MFKD" (Contextra Key Derivation) | | 4 | 1B | Version | Versionsnummer (aktuell 1) | | 5 | 1B | KDF-ID | 1 = Argon2id | | 6..10 | 4B | m_cost_kib | Speicherkosten in KiB (>= 19456) | | 10..14 | 4B | t_cost | Iter
- Impl-Block `crates/contextra-crypto/src/kdf.rs:130`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `version` | `u8` | `crates/contextra-crypto/src/kdf.rs:124` |
| `kdf_id` | `u8` | `crates/contextra-crypto/src/kdf.rs:125` |
| `params` | `KdfParams` | `crates/contextra-crypto/src/kdf.rs:126` |
| `salt` | `Vec<u8>` | `crates/contextra-crypto/src/kdf.rs:127` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn generate_default() -> Result<Self>` | `crates/contextra-crypto/src/kdf.rs:132` | Erstellt einen neuen `KdfHeader` v1 mit zufälligem 32-Byte-Salt und OWASP-Standardparametern. |
| `pub fn new(params: KdfParams, salt: Vec<u8>) -> Result<Self>` | `crates/contextra-crypto/src/kdf.rs:144` | Erstellt einen `KdfHeader` v1 mit vorgegebenen Parametern und Salt. |
| `pub fn to_bytes(&self) -> Vec<u8>` | `crates/contextra-crypto/src/kdf.rs:170` | Serialisiert den `KdfHeader` in Binärform. Layout: `magic (4B "MFKD") \| version (1B = 1) \| kdf_id (1B = 1 Argon2id) \| m_cost_kib (4B BE) \| t_cost (4B BE) \| p_co |
| `pub fn from_bytes(bytes: &[u8]) -> Result<Self>` | `crates/contextra-crypto/src/kdf.rs:186` | Parst einen `KdfHeader` aus Binärdaten. Panic-Freiheit (total function) bei beliebigen Byte-Eingaben. |

#### `struct KdfParams`
- Ort: `crates/contextra-crypto/src/kdf.rs:51`
```rust
pub struct KdfParams
```
Dokumentation: Konfiguration der Argon2id KDF-Kostenparameter.
- Impl-Block (Trait `Default`) `crates/contextra-crypto/src/kdf.rs:57`
- Impl-Block `crates/contextra-crypto/src/kdf.rs:67`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `m_cost_kib` | `u32` | `crates/contextra-crypto/src/kdf.rs:52` |
| `t_cost` | `u32` | `crates/contextra-crypto/src/kdf.rs:53` |
| `p_cost` | `u32` | `crates/contextra-crypto/src/kdf.rs:54` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(m_cost_kib: u32, t_cost: u32, p_cost: u32) -> Result<Self>` | `crates/contextra-crypto/src/kdf.rs:69` | Erstellt neue `KdfParams` mit Validierung der Untergrenzen. |
| `pub fn new_for_test(m_cost_kib: u32, t_cost: u32, p_cost: u32) -> Self` | `crates/contextra-crypto/src/kdf.rs:94` | Erstellt `KdfParams` mit kleineren Kostenparametern ausschließlich für Testzwecke. |

#### `fn derive_key_argon2id`
- Ort: `crates/contextra-crypto/src/kdf.rs:285`
```rust
pub fn derive_key_argon2id(passphrase: &str, header: &KdfHeader) -> Result<DerivedKey>
```
Dokumentation: Leitet einen kryptographisch starken Schlüssel mittels Argon2id aus einer Passphrase ab. # Ablauf der Argon2id-Schlüsselableitung: 1. Passphrase und Salt auf Mindestlänge prüfen. 2. Kostenparameter gegen OWASP-Mindestgrenzen validieren. 3. Argon2id-Instanz initialisieren und 32-Byte-Schlüssel ableiten. 4. Temporäre Puffer löschen und Ergebnis im geschützten KeyManager kapseln.

#### `fn hkdf_expand`
- Ort: `crates/contextra-crypto/src/kdf.rs:311`
```rust
pub fn hkdf_expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> Result<()>
```
Dokumentation: HKDF-SHA256 expand.

#### `const DEFAULT_M_COST_KIB`
- Ort: `crates/contextra-crypto/src/kdf.rs:43`
```rust
pub const DEFAULT_M_COST_KIB: u32 = 65536
```
Dokumentation: OWASP Standardempfehlungen für Argon2id

#### `const DEFAULT_P_COST`
- Ort: `crates/contextra-crypto/src/kdf.rs:47`
```rust
pub const DEFAULT_P_COST: u32 = 1
```
Dokumentation: Standard-Parallelitätsgrad für Argon2id.

#### `const DEFAULT_T_COST`
- Ort: `crates/contextra-crypto/src/kdf.rs:45`
```rust
pub const DEFAULT_T_COST: u32 = 3
```
Dokumentation: Standard-Zeitkosten (Iterationen) für Argon2id.

#### `const KDF_HEADER_MAGIC`
- Ort: `crates/contextra-crypto/src/kdf.rs:24`
```rust
pub const KDF_HEADER_MAGIC: &[u8
```
Dokumentation: Magischer Header-Marker "MFKD" (Contextra Key Derivation)

#### `const KDF_HEADER_VERSION_1`
- Ort: `crates/contextra-crypto/src/kdf.rs:27`
```rust
pub const KDF_HEADER_VERSION_1: u8 = 1
```
Dokumentation: Aktuelle KDF Header Version

#### `const KDF_ID_ARGON2ID`
- Ort: `crates/contextra-crypto/src/kdf.rs:30`
```rust
pub const KDF_ID_ARGON2ID: u8 = 1
```
Dokumentation: Identifikator für Argon2id

#### `const MIN_M_COST_KIB`
- Ort: `crates/contextra-crypto/src/kdf.rs:36`
```rust
pub const MIN_M_COST_KIB: u32 = 19456
```
Dokumentation: OWASP Mindestanforderungen für Argon2id

#### `const MIN_P_COST`
- Ort: `crates/contextra-crypto/src/kdf.rs:40`
```rust
pub const MIN_P_COST: u32 = 1
```
Dokumentation: Minimaler Parallelitätsgrad für Argon2id gemäß OWASP.

#### `const MIN_SALT_LEN`
- Ort: `crates/contextra-crypto/src/kdf.rs:33`
```rust
pub const MIN_SALT_LEN: usize = 16
```
Dokumentation: Min. empfohlener Salt-Umfang (16 Bytes)

#### `const MIN_T_COST`
- Ort: `crates/contextra-crypto/src/kdf.rs:38`
```rust
pub const MIN_T_COST: u32 = 2
```
Dokumentation: Minimale Anzahl an Iterationen (Zeitkosten) für Argon2id gemäß OWASP.

### Modul `kv_cipher`

#### `struct EncryptedKvLayer`
- Ort: `crates/contextra-crypto/src/kv_cipher.rs:77`
```rust
pub struct EncryptedKvLayer
```
Dokumentation: Container for an encrypted KV-cache segment layer. Plaintext data and nonce are zeroized on drop. Public metadata (`tenant_id`, `model_fingerprint`, `format_version`) is explicitly skipped during zeroization (`#[zeroize(skip)]`).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `format_version` | `u8` | `crates/contextra-crypto/src/kv_cipher.rs:80` |
| `ciphertext` | `Vec<u8>` | `crates/contextra-crypto/src/kv_cipher.rs:82` |
| `nonce` | `[u8; 12]` | `crates/contextra-crypto/src/kv_cipher.rs:84` |
| `tenant_id` | `TenantId` | `crates/contextra-crypto/src/kv_cipher.rs:87` |
| `model_fingerprint` | `ModelFingerprint` | `crates/contextra-crypto/src/kv_cipher.rs:90` |

#### `struct KvSegmentCipher`
- Ort: `crates/contextra-crypto/src/kv_cipher.rs:132`
```rust
pub struct KvSegmentCipher
```
Dokumentation: Chiffre für segmentweise verschlüsselte KV-Einträge.
- Gate: `#[cfg(feature = "kv-encryption")]`
- Impl-Block `crates/contextra-crypto/src/kv_cipher.rs:137`
- Impl-Block (Trait `KvCipher`) `crates/contextra-crypto/src/kv_cipher.rs:213`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(key_manager: KeyManager) -> Self` | `crates/contextra-crypto/src/kv_cipher.rs:139` | Creates a new `KvSegmentCipher` wrapping the workspace master `KeyManager`. |
| `pub fn encrypt( &self, tenant_id: TenantId, model_fingerprint: ModelFingerprint, plaintext: &[u8], ) -> Result<EncryptedKvLayer>` | `crates/contextra-crypto/src/kv_cipher.rs:149` | Encrypts KV-cache plaintext payload for a given `(tenant_id, model_fingerprint)` pair. Derives an isolated sub-key via HKDF-SHA256 through `KeyManager::derive_k |
| `pub fn decrypt(&self, encrypted: &EncryptedKvLayer) -> Result<Vec<u8>>` | `crates/contextra-crypto/src/kv_cipher.rs:175` | Decrypts an `EncryptedKvLayer` back to its original plaintext. Checks that `encrypted.format_version == CURRENT_KV_FORMAT_VERSION`. Derives the exact sub-key fo |
| `pub fn encrypt_with_version( &self, tenant_id: TenantId, _segment_id: u64, _version: u8, model_fingerprint: ModelFingerprint, plaintext: &[u8], ) -> Result<EncryptedKvLayer>` | `crates/contextra-crypto/src/kv_cipher.rs:190` | Encrypts KV-cache plaintext payload for a given tenant, segment, and model fingerprint. |
| `pub fn decrypt_with_version( &self, encrypted: &EncryptedKvLayer, _segment_id: u64, _version: u8, ) -> Result<Vec<u8>>` | `crates/contextra-crypto/src/kv_cipher.rs:202` | Decrypts an `EncryptedKvLayer` for a given segment and version. |

#### `struct ModelFingerprint`
- Ort: `crates/contextra-crypto/src/kv_cipher.rs:43`
```rust
pub struct ModelFingerprint
```
Dokumentation: Uniquely identifies a model weight file and its quantization tier.
- Impl-Block `crates/contextra-crypto/src/kv_cipher.rs:52`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `hash` | `[u8; 32]` | `crates/contextra-crypto/src/kv_cipher.rs:45` |
| `model_id` | `String` | `crates/contextra-crypto/src/kv_cipher.rs:47` |
| `quantization` | `String` | `crates/contextra-crypto/src/kv_cipher.rs:49` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( hash: [u8` | `crates/contextra-crypto/src/kv_cipher.rs:54` | Creates a new `ModelFingerprint`. |

#### `trait KvCipher`
- Ort: `crates/contextra-crypto/src/kv_cipher.rs:94`
```rust
pub trait KvCipher: Send + Sync
```
Dokumentation: Abstract cipher trait for KV block encryption and seal/open operations.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-crypto/src/kv_cipher.rs:96` | Encrypts (seals) plaintext byte payload into authenticated ciphertext bytes. |
| `fn open(&self, ciphertext: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-crypto/src/kv_cipher.rs:98` | Decrypts (opens) authenticated ciphertext bytes back to plaintext byte payload. |

#### `const CURRENT_KV_FORMAT_VERSION`
- Ort: `crates/contextra-crypto/src/kv_cipher.rs:69`
```rust
pub const CURRENT_KV_FORMAT_VERSION: u8 = 2
```
Dokumentation: Current serialized format version for [`EncryptedKvLayer`]. Increment when the serialized layout changes in a breaking way.

### Modul `kv_segment`

#### `mod delete_mode`
- Ort: `crates/contextra-crypto/src/kv_segment/mod.rs:1`
```rust
pub mod delete_mode
```

#### `mod eviction_worker`
- Ort: `crates/contextra-crypto/src/kv_segment/mod.rs:2`
```rust
pub mod eviction_worker
```

#### `mod manager`
- Ort: `crates/contextra-crypto/src/kv_segment/mod.rs:3`
```rust
pub mod manager
```

#### `mod segment`
- Ort: `crates/contextra-crypto/src/kv_segment/mod.rs:4`
```rust
pub mod segment
```

#### `mod store`
- Ort: `crates/contextra-crypto/src/kv_segment/mod.rs:5`
```rust
pub mod store
```

### Modul `kv_segment::delete_mode`

#### `enum KvDeleteMode`
- Ort: `crates/contextra-crypto/src/kv_segment/delete_mode.rs:22`
```rust
pub enum KvDeleteMode
```
- Impl-Block (Trait `Default`) `crates/contextra-crypto/src/kv_segment/delete_mode.rs:27`

Varianten:

- `TombstoneOnly` — `crates/contextra-crypto/src/kv_segment/delete_mode.rs:23`
- `CryptoShred` — `crates/contextra-crypto/src/kv_segment/delete_mode.rs:24`

### Modul `kv_segment::eviction_worker`

#### `struct EvictionWorker`
- Ort: `crates/contextra-crypto/src/kv_segment/eviction_worker.rs:40`
```rust
pub struct EvictionWorker
```
Dokumentation: Hintergrund-Worker für die Speicherfreigabe nach dem Least-Recently-Used-Prinzip (LRU).
- Impl-Block `crates/contextra-crypto/src/kv_segment/eviction_worker.rs:45`
- Impl-Block (Trait `Drop`) `crates/contextra-crypto/src/kv_segment/eviction_worker.rs:94`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn spawn(store: Arc<TenantIsolatedKvStore>) -> Self` | `crates/contextra-crypto/src/kv_segment/eviction_worker.rs:47` | Spawnt den Eviction-Worker auf einem dedizierten OS-Thread. |
| `pub fn trigger_eviction(&self, target_free_bytes: usize)` | `crates/contextra-crypto/src/kv_segment/eviction_worker.rs:78` | Nicht-blockierender Trigger vom Hot-Path aus. |
| `pub fn shutdown(&self)` | `crates/contextra-crypto/src/kv_segment/eviction_worker.rs:86` | Beendet den Worker-Thread geordnet. |

#### `fn emergency_wipe`
- Ort: `crates/contextra-crypto/src/kv_segment/eviction_worker.rs:103`
```rust
pub fn emergency_wipe(store: &TenantIsolatedKvStore)
```
Dokumentation: SEPARATER Pfad für Notfall-Bereinigung (z.B. Prozess-Shutdown, expliziter Sicherheits-Trigger). Synchron, blockierend, vor Rückkehr abgeschlossen -- bewusst ANDERS als der reguläre Worker-Pfad.

### Modul `kv_segment::manager`

#### `struct KvSegmentConfig`
- Ort: `crates/contextra-crypto/src/kv_segment/manager.rs:19`
```rust
pub struct KvSegmentConfig
```
Dokumentation: Configuration for KV segment storage behavior.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `delete_mode` | `KvDeleteMode` | `crates/contextra-crypto/src/kv_segment/manager.rs:21` |

#### `struct KvSegmentManager`
- Ort: `crates/contextra-crypto/src/kv_segment/manager.rs:43`
```rust
pub struct KvSegmentManager
```
- Impl-Block `crates/contextra-crypto/src/kv_segment/manager.rs:48`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( config: KvSegmentConfig, master_key: Option<Arc<KeyManager>>, ) -> Self` | `crates/contextra-crypto/src/kv_segment/manager.rs:50` | Creates a new manager with specified config and master key. |
| `pub fn delete_mode(&self) -> KvDeleteMode` | `crates/contextra-crypto/src/kv_segment/manager.rs:61` | Returns the current delete mode. |
| `pub fn write_segment( &self, group_id: u64, plaintext: &[u8], ) -> Result<KvSegmentPayload, CryptoError>` | `crates/contextra-crypto/src/kv_segment/manager.rs:66` | Writes a KV segment payload according to the configured mode. |
| `pub fn read_segment(&self, segment: &KvSegmentPayload) -> Result<Vec<u8>, CryptoError>` | `crates/contextra-crypto/src/kv_segment/manager.rs:99` | Reads/decrypts a KV segment payload. |
| `pub fn delete_segment(&self, _group_id: u64) -> Result<bool, CryptoError>` | `crates/contextra-crypto/src/kv_segment/manager.rs:137` | Deletes a KV segment. |

#### `struct KvSegmentPayload`
- Ort: `crates/contextra-crypto/src/kv_segment/manager.rs:26`
```rust
pub struct KvSegmentPayload
```
Dokumentation: Representation of a written KV segment (encrypted or raw payload with group_id and nonce).

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `group_id` | `u64` | `crates/contextra-crypto/src/kv_segment/manager.rs:28` |
| `payload` | `Vec<u8>` | `crates/contextra-crypto/src/kv_segment/manager.rs:30` |
| `nonce` | `Option<[u8; 12]>` | `crates/contextra-crypto/src/kv_segment/manager.rs:32` |
| `is_encrypted` | `bool` | `crates/contextra-crypto/src/kv_segment/manager.rs:34` |

### Modul `kv_segment::segment`

#### `struct EncryptedSegmentPayload`
- Ort: `crates/contextra-crypto/src/kv_segment/segment.rs:26`
```rust
pub struct EncryptedSegmentPayload
```
Dokumentation: Encrypted layer representation stored inside a KvSegment when encryption is active.
- Gate: `#[cfg(feature = "kv-encryption")]`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `layer` | `EncryptedKvLayer` | `crates/contextra-crypto/src/kv_segment/segment.rs:28` |

#### `struct KvSegment`
- Ort: `crates/contextra-crypto/src/kv_segment/segment.rs:49`
```rust
pub struct KvSegment
```
- Impl-Block `crates/contextra-crypto/src/kv_segment/segment.rs:72`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-crypto/src/kv_segment/segment.rs:204`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tenant_id` | `TenantId` | `crates/contextra-crypto/src/kv_segment/segment.rs:51` |
| `segment_id` | `u64` | `crates/contextra-crypto/src/kv_segment/segment.rs:53` |
| `key_derivation_version` | `u8` | `crates/contextra-crypto/src/kv_segment/segment.rs:58` |
| `encrypted` | `bool` | `crates/contextra-crypto/src/kv_segment/segment.rs:60` |
| `model_fingerprint` | `Option<ModelFingerprint>` | `crates/contextra-crypto/src/kv_segment/segment.rs:63` |
| `rope_offset` | `Option<usize>` | `crates/contextra-crypto/src/kv_segment/segment.rs:65` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(tenant_id: TenantId, segment_id: u64, data: Vec<u8>) -> Self` | `crates/contextra-crypto/src/kv_segment/segment.rs:74` | Erstellt ein neues Klartext-KV-Cache-Segment (Default / Zero-Config, P12-konform). |
| `pub fn new_with_metadata( tenant_id: TenantId, segment_id: u64, data: Vec<u8>, #[cfg(feature = "")] model_fingerprint: Option<ModelFingerprint>, rope_offset: Option<usize>, ) -> Self` | `crates/contextra-crypto/src/kv_segment/segment.rs:90` | Erstellt ein neues Klartext-KV-Cache-Segment mit optionalen Metadaten. |
| `pub fn new_encrypted( cipher: &KvSegmentCipher, tenant_id: TenantId, segment_id: u64, model_fingerprint: ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> Result<Self, crate::CryptoError>` | `crates/contextra-crypto/src/kv_segment/segment.rs:113` | Erstellt ein verschlüsseltes KV-Cache-Segment aus einem Klartext-Tensor. |
| `pub fn decrypt_data(&self, cipher: &KvSegmentCipher) -> Result<Vec<u8>, crate::CryptoError>` | `crates/contextra-crypto/src/kv_segment/segment.rs:147` | Entschlüsselt die Daten des Segments, falls es verschlüsselt ist. Gibt bei Klartext-Segmenten direkt einen Klon der `data`-Bytes zurück. |
| `pub fn to_spill_bytes(&self) -> Vec<u8>` | `crates/contextra-crypto/src/kv_segment/segment.rs:178` | Serialisiert das Segment bzw. dessen verschlüsselte Layer für Tier-2-LSM-Spill. |
| `pub fn as_bytes(&self) -> &[u8]` | `crates/contextra-crypto/src/kv_segment/segment.rs:189` | Read-Only-Zugriff. Kein Klartext-Export nach außen ohne expliziten Call. |
| `pub fn len(&self) -> usize` | `crates/contextra-crypto/src/kv_segment/segment.rs:194` | Länge der Tensor-Bytes in Bytes. |
| `pub fn is_empty(&self) -> bool` | `crates/contextra-crypto/src/kv_segment/segment.rs:199` | Prüft ob das Segment leer ist. |

#### `const CURRENT_KV_KEY_DERIVATION_VERSION`
- Ort: `crates/contextra-crypto/src/kv_segment/segment.rs:21`
```rust
pub const CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1
```
Dokumentation: Aktuelle Version der KV-Segment-Schlüsselableitung. Erhöhe diesen Wert, wenn sich der HKDF-Info-String oder der Salt-Aufbau ändert.

### Modul `kv_segment::store`

#### `struct TenantIsolatedKvStore`
- Ort: `crates/contextra-crypto/src/kv_segment/store.rs:121`
```rust
pub struct TenantIsolatedKvStore
```
- Impl-Block `crates/contextra-crypto/src/kv_segment/store.rs:135`
- Impl-Block (Trait `Default`) `crates/contextra-crypto/src/kv_segment/store.rs:525`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-crypto/src/kv_segment/store.rs:140` | Erstellt einen neuen tenant-isolierten KV-Store mit Standard-Shard-Anzahl (32). |
| `pub fn with_shard_count(n: usize) -> Self` | `crates/contextra-crypto/src/kv_segment/store.rs:146` | Erstellt einen Store mit angegebener Shard-Anzahl. Die Shard-Anzahl wird automatisch auf die nächste Zweierpotenz aufgerundet. |
| `pub fn with_capacity(segment_capacity_per_tenant: usize) -> Self` | `crates/contextra-crypto/src/kv_segment/store.rs:168` | Erstellt einen Store mit konfigurierter Segment-Kapazität pro Tenant. |
| `pub fn set_spill_handler(&self, handler: SpillHandler)` | `crates/contextra-crypto/src/kv_segment/store.rs:176` | Registriert einen Spill-Handler für evictierte Segmente. |
| `pub fn insert_segment(&self, tenant: TenantId, segment: KvSegment)` | `crates/contextra-crypto/src/kv_segment/store.rs:187` | Fügt ein Segment für einen bestimmten Tenant ein. |
| `pub fn on_rollback(&self, tenant: TenantId, chunk_ids: &[u64])` | `crates/contextra-crypto/src/kv_segment/store.rs:210` | Bereinigt assoziierte KV-Segmente bei einem transaktionalen Rollback. Delegiert direkt an `remove_segments_for_rollback`, um verwaiste KV-Segmente (Karteileiche |
| `pub fn remove_segments_for_rollback(&self, tenant: TenantId, segment_ids: &[u64])` | `crates/contextra-crypto/src/kv_segment/store.rs:219` | Entfernt alle Segmente mit den angegebenen `segment_ids` für den gegebenen Tenant. Muss bei transaktionalem Rollback aufgerufen werden, wenn ein `DbTransaction: |
| `pub fn remove_segment(&self, tenant: TenantId, segment_id: u64)` | `crates/contextra-crypto/src/kv_segment/store.rs:247` | Entfernt ein einzelnes Segment. Convenience-Wrapper um `remove_segments_for_rollback`. |
| `pub fn get_segment_bytes(&self, tenant: TenantId, segment_id: u64) -> Option<Vec<u8>>` | `crates/contextra-crypto/src/kv_segment/store.rs:252` | Liefert unverschlüsselte Segment-Bytes für einen Tenant (Klartext-Modus). |
| `pub fn insert_encrypted_segment( &self, cipher: &crate::KvSegmentCipher, tenant: TenantId, segment_id: u64, model_fingerprint: crate::ModelFingerprint, rope_offset: Option<usize>, plaintext: &[u8], ) -> Result<(), crate::CryptoError>` | `crates/contextra-crypto/src/kv_segment/store.rs:263` | Verschlüsselt einen Klartext-Tensor und fügt ein verschlüsseltes Segment ein. |
| `pub fn get_segments(&self, tenant: TenantId) -> Vec<u64>` | `crates/contextra-crypto/src/kv_segment/store.rs:286` | INV-TENANT-Analogon für KV-Bridge: Ein Tenant kann niemals Segmente eines anderen Tenants lesen. Strukturell erzwungen durch getrennte Maps. |
| `pub fn get_decrypted_segment( &self, cipher: &crate::KvSegmentCipher, tenant: TenantId, segment_id: u64, ) -> Result<Option<Vec<u8>>, crate::CryptoError>` | `crates/contextra-crypto/src/kv_segment/store.rs:298` | Liest ein bestimmtes Segment eines Mandanten und entschlüsselt es falls nötig. |
| `pub fn get_tenant_segment_len(&self, tenant: TenantId) -> usize` | `crates/contextra-crypto/src/kv_segment/store.rs:316` | Gibt die Anzahl der gespeicherten Segmente für einen bestimmten Tenant zurück. |
| `pub fn evict_lru_fair(&self, target_free_bytes: usize) -> usize` | `crates/contextra-crypto/src/kv_segment/store.rs:395` | Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness via Round-Robin über alle aktiven Tenants. Im Gegensatz zu `evict_lru_global()` verhindert diese Metho |
| `pub fn evict_lru_fair_with_hook<F>(&self, target_free_bytes: usize, mut hook: F) -> usize where F: FnMut(),` | `crates/contextra-crypto/src/kv_segment/store.rs:407` |  |
| `pub fn clear_all(&self)` | `crates/contextra-crypto/src/kv_segment/store.rs:510` | Leert alle Segmente aus allen Shards vollständig und löscht deren Speicher via Zeroize. |

Assoziierte Konstanten/Typen:

- `pub const DEFAULT_SHARD_COUNT: usize = 32` — `crates/contextra-crypto/src/kv_segment/store.rs:136`
- `pub const DEFAULT_SEGMENT_CAPACITY_PER_TENANT: usize = 256` — `crates/contextra-crypto/src/kv_segment/store.rs:137`

#### `type SpillHandler`
- Ort: `crates/contextra-crypto/src/kv_segment/store.rs:26`
```rust
pub type SpillHandler = Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>
```
Dokumentation: Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache.

### Modul `secret`

#### `struct Secret`
- Ort: `crates/contextra-crypto/src/secret.rs:69`
```rust
pub struct Secret<T: Zeroize>
```
- Impl-Block `crates/contextra-crypto/src/secret.rs:73`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-crypto/src/secret.rs:92`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-crypto/src/secret.rs:98`
- Impl-Block (Trait `serde::Serialize`) `crates/contextra-crypto/src/secret.rs:118`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(inner: T) -> Self` | `crates/contextra-crypto/src/secret.rs:75` | Creates a new `Secret` wrapping sensitive data. |
| `pub fn expose_secret(&self) -> &T` | `crates/contextra-crypto/src/secret.rs:81` | Exposes a reference to the inner secret. |
| `pub fn expose_secret_mut(&mut self) -> &mut T` | `crates/contextra-crypto/src/secret.rs:87` | Exposes a mutable reference to the inner secret. |

#### `struct SecretKey`
- Ort: `crates/contextra-crypto/src/secret.rs:14`
```rust
pub struct SecretKey<const N: usize>
```
- Impl-Block `crates/contextra-crypto/src/secret.rs:18`
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-crypto/src/secret.rs:37`
- Impl-Block (Trait `std::fmt::Display`) `crates/contextra-crypto/src/secret.rs:43`
- Impl-Block (Trait `ConstantTimeEq`) `crates/contextra-crypto/src/secret.rs:49`
- Impl-Block (Trait `PartialEq`) `crates/contextra-crypto/src/secret.rs:55`
- Impl-Block (Trait `Eq`) `crates/contextra-crypto/src/secret.rs:61`
- Impl-Block (Trait `serde::Serialize`) `crates/contextra-crypto/src/secret.rs:108`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(bytes: [u8` | `crates/contextra-crypto/src/secret.rs:20` | Creates a new secret key wrapping raw bytes. |
| `pub fn expose_secret(&self) -> &[u8` | `crates/contextra-crypto/src/secret.rs:26` | Exposes a reference to the secret bytes. |
| `pub fn as_bytes(&self) -> &[u8` | `crates/contextra-crypto/src/secret.rs:32` | Exposes a reference to the secret bytes (compatibility alias). |

### Modul `wal_completeness`

#### `fn verify_wal_chain_completeness`
- Ort: `crates/contextra-crypto/src/wal_completeness.rs:43`
```rust
pub fn verify_wal_chain_completeness( wal_tail_hmac: &[u8
```
Dokumentation: Verifiziert, dass der WAL-Tail nicht abgeschnitten wurde, indem der letzte bekannte WAL-HMAC mit der im Checkpoint-Manifest persistierten High-Water-Mark verglichen wird. # Semantik & Bedrohungsmodell - Die Verifikation prüft die exakte Byteweise Gleichheit (`subtle::ConstantTimeEq`) zwischen dem Ende der WAL-HMAC-Kette (`wal_tail_hmac`) und der im Checkpoint-Manifest aufgezeichneten High-Water-Mark. - Ein fehlgeschlagener Vergleich zeigt an, dass nach dem letzten Checkpoint geschriebene WAL-Einträge durch Truncation, Dateisystem-Fehler oder bösartige Rollback-Versuche verloren gegangen oder v

### Modul `wal_crypto`

#### `struct CloudEgressGuard`
- Ort: `crates/contextra-crypto/src/wal_crypto.rs:146`
```rust
pub struct CloudEgressGuard
```
Dokumentation: Egress guard for inspecting and sanitizing encrypted payload blocks prior to cloud egress.
- Gate: `#[cfg(feature = "cloud-egress-guard")]`
- Impl-Block (Trait `Default`) `crates/contextra-crypto/src/wal_crypto.rs:151`
- Impl-Block `crates/contextra-crypto/src/wal_crypto.rs:158`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-crypto/src/wal_crypto.rs:160` | Creates a new `CloudEgressGuard` with a default maximum payload limit of 10 MB. |
| `pub fn with_max_bytes(max_egress_bytes: usize) -> Self` | `crates/contextra-crypto/src/wal_crypto.rs:167` | Creates a custom `CloudEgressGuard` with a specified maximum payload size. |
| `pub fn validate_egress_payload(&self, payload: &[u8]) -> Result<()>` | `crates/contextra-crypto/src/wal_crypto.rs:172` | Validates that an egress payload meets size and structural integrity bounds. |

#### `struct EncryptedWal`
- Ort: `crates/contextra-crypto/src/wal_crypto.rs:203`
```rust
pub struct EncryptedWal
```
Dokumentation: Encrypted WAL chunk provider that handles transparent encryption/decryption of WAL payloads via KMS. # Invariants - Delegates encryption and key management to a [`KmsProvider`]. - Prepends a unique 12-byte random nonce to every encrypted chunk output. # Usage Use when writing or reading encrypted WAL logs at rest. Call `encrypt_chunk` to wrap plain payload bytes into an encrypted chunk with prepended nonce, and `decrypt_chunk` to recover the original payload. # Errors Emits `CryptoError::Crypto` if encryption/decryption fails, key derivation fails, or if chunk length is less than 12 bytes duri
- Impl-Block `crates/contextra-crypto/src/wal_crypto.rs:213`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(key_manager: KeyManager, file_id: &[u8]) -> Result<Self>` | `crates/contextra-crypto/src/wal_crypto.rs:216` | Creates a new EncryptedWal with per-file key derivation via LocalKeyManagerKms to prevent nonce-reuse. `file_id` (e.g., filename) is used to derive a unique sub |
| `pub fn with_kms(kms: Arc<dyn KmsProvider>) -> Self` | `crates/contextra-crypto/src/wal_crypto.rs:235` | Creates a new EncryptedWal using an arbitrary KmsProvider. |
| `pub fn kms(&self) -> &Arc<dyn KmsProvider>` | `crates/contextra-crypto/src/wal_crypto.rs:240` | Returns a reference to the inner KMS provider. |
| `pub fn encrypt_chunk(&self, payload: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-crypto/src/wal_crypto.rs:246` | Wraps the internal WAL chunk in AES-256-GCM-SIV stream. Prepends the 12-byte nonce to the encrypted ciphertext. |
| `pub fn decrypt_chunk(&self, data: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-crypto/src/wal_crypto.rs:258` | Decrypts the WAL chunk by extracting the prepended 12-byte nonce from the data. |

#### `struct IntegrityVerifier`
- Ort: `crates/contextra-crypto/src/wal_crypto.rs:361`
```rust
pub struct IntegrityVerifier
```
Dokumentation: Stateful verifier for WAL HMAC-SHA256 hash chains. # Invariants - Maintains running `last_hmac` state across entries to enforce sequential hash-chain continuity. - Performs constant-time comparison (`subtle::ConstantTimeEq`) of checksums and `prev_hmac` to prevent timing side-channel attacks. - Zeroizes key material on drop to prevent secret key leakage from memory. - Any HMAC discrepancy or broken chain link immediately halts verification. # Usage Instantiate with the WAL's integrity key at the start of replay and call `verify_and_update()` sequentially for each deserialized WAL entry. # Erro
- Impl-Block `crates/contextra-crypto/src/wal_crypto.rs:367`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(integrity_key: &[u8]) -> Self` | `crates/contextra-crypto/src/wal_crypto.rs:368` |  |
| `pub fn set_last_hmac(&mut self, hmac: [u8` | `crates/contextra-crypto/src/wal_crypto.rs:382` | Übernimmt einen bereits bekannten `last_hmac`-Kettenzustand, z. B. beim Wechsel von einem Normal- auf einen Legacy-Key-Verifier MITTEN in einem Batch, damit die |
| `pub fn last_hmac_snapshot(&self) -> [u8` | `crates/contextra-crypto/src/wal_crypto.rs:387` | Gibt den aktuellen `last_hmac`-Kettenzustand zurück (für Verifier-Handoff). |
| `pub fn set_last_seq_no(&mut self, seq_no: Option<u64>)` | `crates/contextra-crypto/src/wal_crypto.rs:392` | Übernimmt eine bereits bekannte `last_seq_no` für den Verifier-Handoff. |
| `pub fn last_seq_no_snapshot(&self) -> Option<u64>` | `crates/contextra-crypto/src/wal_crypto.rs:397` | Gibt die aktuell bekannte `last_seq_no` zurück (für Verifier-Handoff). |
| `pub fn verify_and_update_v3(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>` | `crates/contextra-crypto/src/wal_crypto.rs:402` | Verifies a V3 entry (with tx_id and length prefixes) and updates the chain state. |
| `pub fn verify_and_update_v2(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>` | `crates/contextra-crypto/src/wal_crypto.rs:491` | Verifiziert und aktualisiert die HMAC-Kette im V2-Format (Rückwärtskompatibilität). |
| `pub fn skip_hmac_verify_legacy(&mut self, entry: &WalEntrySnapshot)` | `crates/contextra-crypto/src/wal_crypto.rs:560` | Updates the chain state for legacy V1 entries without HMAC verification. |
| `pub fn verify_and_update(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>` | `crates/contextra-crypto/src/wal_crypto.rs:566` | Default verification delegating to V3 verification. |

#### `struct LocalKeyManagerKms`
- Ort: `crates/contextra-crypto/src/wal_crypto.rs:55`
```rust
pub struct LocalKeyManagerKms
```
Dokumentation: Standard local implementation of [`KmsProvider`] wrapping [`KeyManager`]. Provides thread-safe, zeroized key derivation, wrapping, unwrapping, and rotation.
- Impl-Block `crates/contextra-crypto/src/wal_crypto.rs:61`
- Impl-Block (Trait `KmsProvider`) `crates/contextra-crypto/src/wal_crypto.rs:77`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(key_manager: KeyManager, key_id: impl Into<String>) -> Self` | `crates/contextra-crypto/src/wal_crypto.rs:63` | Creates a new `LocalKeyManagerKms` wrapping an existing `KeyManager`. |
| `pub fn key_manager(&self) -> &KeyManager` | `crates/contextra-crypto/src/wal_crypto.rs:72` | Accesses the underlying `KeyManager`. |

#### `struct WalEntrySnapshot`
- Ort: `crates/contextra-crypto/src/wal_crypto.rs:333`
```rust
pub struct WalEntrySnapshot
```
Dokumentation: Immutable snapshot of a WAL entry used for cryptographic integrity verification. # Invariants - Captures entry fields (`seq_no`, `op_type`, `key`, `value`, `checksum`, `prev_hmac`) required to recompute the entry's HMAC-SHA256 checksum and verify hash-chain continuity. # Usage Passed to `IntegrityVerifier::verify_and_update()` during WAL replay or recovery.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `tx_id` | `u64` | `crates/contextra-crypto/src/wal_crypto.rs:334` |
| `seq_no` | `u64` | `crates/contextra-crypto/src/wal_crypto.rs:335` |
| `op_type` | `u8` | `crates/contextra-crypto/src/wal_crypto.rs:336` |
| `key` | `Vec<u8>` | `crates/contextra-crypto/src/wal_crypto.rs:337` |
| `value` | `Vec<u8>` | `crates/contextra-crypto/src/wal_crypto.rs:338` |
| `checksum` | `[u8; 32]` | `crates/contextra-crypto/src/wal_crypto.rs:339` |
| `prev_hmac` | `[u8; 32]` | `crates/contextra-crypto/src/wal_crypto.rs:340` |

#### `struct WalHmac`
- Ort: `crates/contextra-crypto/src/wal_crypto.rs:291`
```rust
pub struct WalHmac
```
Dokumentation: Stateful wrapper around HMAC-SHA256 initialized with WAL domain separation. # Invariants - Initialized with `b"contextra-wal-v1"` domain separation string as the first updated block, preventing cross-context HMAC collisions if the integrity key is reused elsewhere. # Usage Use to compute deterministic 32-byte HMAC-SHA256 checksums over WAL entry fields and previous chain links during WAL appends or verification. # Errors Emits `CryptoError::Crypto` if key initialization fails.
- Impl-Block `crates/contextra-crypto/src/wal_crypto.rs:295`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(integrity_key: &[u8]) -> Result<Self>` | `crates/contextra-crypto/src/wal_crypto.rs:296` |  |
| `pub fn update(&mut self, data: &[u8])` | `crates/contextra-crypto/src/wal_crypto.rs:313` | Führt neue Daten in die laufende HMAC-Berechnung ein. |
| `pub fn finalize(self) -> [u8` | `crates/contextra-crypto/src/wal_crypto.rs:318` | Schließt die HMAC-Berechnung ab und liefert den 32-Byte-Digest. |

#### `trait KmsProvider`
- Ort: `crates/contextra-crypto/src/wal_crypto.rs:32`
```rust
pub trait KmsProvider: Send + Sync
```
Dokumentation: Key Management Strategy interface for data key derivation, key wrapping/unwrapping, and key rotation.

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn get_key(&self) -> Result<Zeroizing<Vec<u8>>>` | `crates/contextra-crypto/src/wal_crypto.rs:34` | Retrieves the Data Encryption Key (DEK) zeroized on drop. |
| `fn active_key_id(&self) -> &str` | `crates/contextra-crypto/src/wal_crypto.rs:37` | Returns the active key identifier or version alias. |
| `fn derive_data_key(&self, key_id: &[u8]) -> Result<Zeroizing<Vec<u8>>>` | `crates/contextra-crypto/src/wal_crypto.rs:40` | Derives a raw data key bound to the specified `key_id`. |
| `fn wrap_key(&self, key_id: &[u8], plaintext: &[u8]) -> Result<Vec<u8>>` | `crates/contextra-crypto/src/wal_crypto.rs:43` | Wraps (encrypts) plaintext data or a raw key using the key bound to `key_id`. |
| `fn unwrap_key(&self, key_id: &[u8], wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>>` | `crates/contextra-crypto/src/wal_crypto.rs:46` | Unwraps (decrypts) wrapped ciphertext data or key using the key bound to `key_id`. |
| `fn rotate_key(&mut self) -> Result<String>` | `crates/contextra-crypto/src/wal_crypto.rs:49` | Rotates the active master key or key version and returns the new active key ID. |

## 9. Impl-Blöcke ohne Typ im selben Crate

- `impl From<CryptoError> for ContextraError` — `crates/contextra-crypto/src/compat/error.rs:8` (0 pub Methoden)
- `impl From<TypesFingerprint> for LocalFingerprint` — `crates/contextra-crypto/src/compat/model_fingerprint.rs:9` (0 pub Methoden)
- `impl From<LocalFingerprint> for TypesFingerprint` — `crates/contextra-crypto/src/compat/model_fingerprint.rs:19` (0 pub Methoden)
