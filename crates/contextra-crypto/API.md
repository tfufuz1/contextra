# API: contextra-crypto

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 343 öffentliche Items, 6 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- contextra-ports
- contextra-types

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/aead.rs:14  pub const REC_AAD: &[u8] = b"ctx/rec/v2"
src/aead.rs:18  pub fn seal_record_with_nonce(
src/aead.rs:49  pub fn seal_record_with_rng<R: RngCore>(
src/aead.rs:62  pub fn seal_record(dek: &Dek, doc: [u8; 16], key_bytes: &[u8], plain: &[u8]) -> Vec<u8>
src/aead.rs:68  pub fn open_record(dek: &Dek, key_bytes: &[u8], env: &[u8]) -> Result<Zeroizing<Vec<u8>>>
src/anti_tamper.rs:24  pub struct VolatileEncryptionKey
src/anti_tamper.rs:30  pub fn new(raw: [u8; 32]) -> Self
src/anti_tamper.rs:38  pub fn emergency_wipe(&mut self)
src/anti_tamper.rs:43  pub fn as_bytes(&self) -> &[u8]
src/anti_tamper.rs:49  pub fn inspect_key_bytes_for_test(&self) -> &[u8; 32]
src/anti_tamper.rs:77  pub fn constant_time_compare(a: &[u8], b: &[u8]) -> bool
src/anti_tamper.rs:86  pub fn verify_tag_constant_time(expected: &[u8], actual: &[u8]) -> Result<()>
src/anti_tamper.rs:96  pub struct Blake3Chain
src/anti_tamper.rs:103  pub fn new(genesis: [u8; 32]) -> Self
src/anti_tamper.rs:111  pub fn extend(&mut self, context: &[u8], data: &[u8]) -> [u8; 32]
src/anti_tamper.rs:127  pub fn current_hash(&self) -> &[u8; 32]
src/anti_tamper.rs:132  pub fn step_count(&self) -> u64
src/anti_tamper.rs:137  pub fn verify_current_hash(&self, expected: &[u8; 32]) -> Result<()>
src/audit_chain.rs:37  pub enum DataClass
src/audit_chain.rs:56  pub struct AuditChainEntry
src/audit_chain.rs:80  pub fn compute_hash(
src/audit_chain.rs:120  pub struct AuditChainHeadSignature
src/audit_chain.rs:131  pub struct AuditHeadAnchor
src/audit_chain.rs:142  pub fn new(seq_no: u64, head_hash: [u8; 32]) -> Self
src/audit_chain.rs:151  pub fn with_signature(
src/audit_chain.rs:166  pub struct EncryptedCommitmentSalt
src/audit_chain.rs:178  pub fn encrypt(
src/audit_chain.rs:193  pub fn decrypt(&self, registry: &KeyRegistry) -> Result<Vec<u8>>
src/audit_chain.rs:199  pub fn compute_record_commitment(salt: &[u8], attr: &[u8]) -> [u8; 32]
src/audit_chain.rs:210  pub struct AuditChain
src/audit_chain.rs:218  pub fn new() -> Self
src/audit_chain.rs:227  pub fn append(
src/audit_chain.rs:271  pub fn len(&self) -> usize
src/audit_chain.rs:276  pub fn is_empty(&self) -> bool
src/audit_chain.rs:281  pub fn entries(&self) -> &[AuditChainEntry]
src/audit_chain.rs:286  pub fn head_hash(&self) -> [u8; 32]
src/audit_chain.rs:291  pub fn sign_head(
src/audit_chain.rs:322  pub fn verify_head_signature(
src/audit_chain.rs:348  pub fn verify_chain(&self) -> Result<bool>
src/audit_chain.rs:396  pub fn verify_chain_with_anchor(
src/crypto.rs:65  pub struct KeyManager
src/crypto.rs:88  pub fn try_new(passphrase: &str, salt: &[u8]) -> Result<Self>
src/crypto.rs:130  pub fn try_new_with_kdf(passphrase: &str, header: &KdfHeader) -> Result<Self>
src/crypto.rs:151  pub fn try_new_random_salt(passphrase: &str) -> Result<(Self, [u8; 32])>
src/crypto.rs:160  pub fn derive_file_key(&self, file_id: &[u8]) -> Result<Self>
src/crypto.rs:206  pub fn derive_segment_key(&self, info: &str) -> Result<Self>
src/crypto.rs:238  pub fn derive_kv_key(
src/crypto.rs:293  pub fn derive_kv_key_scoped(
src/crypto.rs:305  pub fn cipher_for(&self, tenant_id: contextra_types::TenantId) -> Result<Self>
src/crypto.rs:316  pub fn cipher_for_scoped<T>(
src/crypto.rs:328  pub fn integrity_key(&self) -> Result<[u8; 32]>
src/crypto.rs:339  pub fn derive_deletion_proof_key(&self) -> Result<[u8; 32]>
src/crypto.rs:364  pub fn encrypt_auto_nonce(&self, data: &[u8]) -> Result<(Vec<u8>, [u8; 12])>
src/crypto.rs:395  pub fn decrypt_auto_nonce(&self, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>>
src/crypto.rs:404  pub fn emergency_wipe(&mut self)
src/crypto.rs:411  pub fn export_dek(&self) -> Zeroizing<Vec<u8>>
src/crypto.rs:417  pub fn inspect_key_bytes_for_test(&self) -> &[u8; 32]
src/dek.rs:15  pub struct UnitKey(pub SecretKey<32>)
src/dek.rs:18  pub fn new(bytes: [u8; 32]) -> Self
src/dek.rs:22  pub fn expose_secret(&self) -> &[u8; 32]
src/dek.rs:29  pub struct Dek(pub SecretKey<32>)
src/dek.rs:32  pub fn new(bytes: [u8; 32]) -> Self
src/dek.rs:36  pub fn expose_secret(&self) -> &[u8; 32]
src/dek.rs:41  pub type Salt = [u8; 32]
src/dek.rs:42  pub type DocTag = [u8; 16]
src/dek.rs:45  pub struct AncestorSalts
src/dek.rs:52  pub const DEK_LABEL: &[u8] = b"ctx/dek/v2"
src/dek.rs:53  pub const DEK_SALT_LABEL: &[u8] = b"ctx/dek-salt/v2"
src/dek.rs:54  pub const WRAP_AAD: &[u8] = b"ctx/dek-wrap/v1"
src/dek.rs:57  pub fn derive_dek(unit: &UnitKey, a: &AncestorSalts, doc: DocTag) -> Dek
src/dek.rs:81  pub fn wrap_dek(kek: &UnitKey, dek: &Dek, nonce: [u8; 12]) -> Result<Vec<u8>>
src/dek.rs:98  pub fn unwrap_dek(kek: &UnitKey, wrapped: &[u8]) -> Result<Dek>
src/deletion_proof/chain.rs:12  pub fn verify_proof_sequence(proofs: &[DeletionProof]) -> Result<bool>
src/deletion_proof/creation.rs:18  pub fn issue<'a>(
src/deletion_proof/creation.rs:215  pub fn create(
src/deletion_proof/creation.rs:235  pub fn create_with_wal_receipt(
src/deletion_proof/creation.rs:300  pub fn create_v3(
src/deletion_proof/creation.rs:325  pub fn create_v3_with_audit_position(
src/deletion_proof/creation.rs:352  pub fn create_with_wal_receipt_v3(
src/deletion_proof/creation.rs:379  pub fn create_full_v3(
src/deletion_proof/creation.rs:540  pub fn compute_wal_delete_receipt(
src/deletion_proof/key_derivation.rs:11  pub const DELETION_PROOF_KEY_PURPOSE: &[u8] = b"Contextra-v1-DeletionProof-Key-Derivation"
src/deletion_proof/key_derivation.rs:14  pub fn derive_deletion_proof_key_for_tenant(
src/deletion_proof/mod.rs:21  pub mod chain
src/deletion_proof/mod.rs:22  pub mod creation
src/deletion_proof/mod.rs:23  pub mod key_derivation
src/deletion_proof/mod.rs:24  pub mod types
src/deletion_proof/mod.rs:25  pub mod verification
src/deletion_proof/mod.rs:26  pub mod verifier
src/deletion_proof/mod.rs:28  pub use chain::*
src/deletion_proof/mod.rs:29  pub use creation::*
src/deletion_proof/mod.rs:30  pub use key_derivation::*
src/deletion_proof/mod.rs:31  pub use types::*
src/deletion_proof/mod.rs:32  pub use verification::*
src/deletion_proof/mod.rs:33  pub use verifier::*
src/deletion_proof/mod.rs:35  pub use crate::ed25519_proof::DeletionProofKeyPair
src/deletion_proof/types.rs:19  pub trait LayerWitnessExt
src/deletion_proof/types.rs:45  pub fn map_witness_layer_to_deletion_layer(
src/deletion_proof/types.rs:76  pub struct GraphRepairAttestation
src/deletion_proof/types.rs:86  pub fn hash_deleted_keys_length_prefixed(deleted_keys: &[Vec<u8>]) -> [u8; 32]
src/deletion_proof/types.rs:97  pub enum CustodyLevel
src/deletion_proof/types.rs:108  pub enum SigningKeyHandle<'a>
src/deletion_proof/types.rs:135  pub enum VerificationKey<'a>
src/deletion_proof/types.rs:168  pub struct LayerCleanupProof
src/deletion_proof/types.rs:175  pub fn for_test(layer: DeletionLayer) -> Self
src/deletion_proof/types.rs:183  pub fn layer(&self) -> &DeletionLayer
src/deletion_proof/types.rs:190  pub enum DeletionLayer
src/deletion_proof/types.rs:214  pub struct UncoveredLayer
src/deletion_proof/types.rs:235  pub fn serialize<S>(tech: &ScanTechnique, serializer: S) -> Result<S::Ok, S::Error>
src/deletion_proof/types.rs:250  pub fn deserialize<'de, D>(deserializer: D) -> Result<ScanTechnique, D::Error>
src/deletion_proof/types.rs:268  pub struct LayerWitnessMeasurement
src/deletion_proof/types.rs:286  pub enum ExcludedScope
src/deletion_proof/types.rs:295  pub enum DeletionScope
src/deletion_proof/types.rs:321  pub struct DeletionProof
src/deletion_proof/verification.rs:19  pub fn signature_version_typed(
src/deletion_proof/verification.rs:26  pub fn tenant_id(&self) -> TenantId
src/deletion_proof/verification.rs:35  pub fn verify<'a>(&self, key: impl Into<VerificationKey<'a>>) -> Result<bool>
src/deletion_proof/verification.rs:112  pub fn verify_external(
src/deletion_proof/verification.rs:136  pub fn export_for_audit(&self) -> Result<String>
src/deletion_proof/verification.rs:164  pub fn verify_wal_delete_receipt(
src/deletion_proof/verifier.rs:14  pub enum VerificationErrorReason
src/deletion_proof/verifier.rs:55  pub struct VerificationResult
src/deletion_proof/verifier.rs:64  pub fn success() -> Self
src/deletion_proof/verifier.rs:72  pub fn failure(reason: VerificationErrorReason) -> Self
src/deletion_proof/verifier.rs:111  pub struct DeletionProofVerifier
src/deletion_proof/verifier.rs:115  pub fn verify_standalone(
src/deletion_proof/verifier.rs:144  pub fn verify_standalone_for_tenant(
src/deletion_proof/verifier.rs:159  pub fn verify_audit_chain_and_head(
src/ed25519_proof.rs:25  pub use crate::deletion_proof::hash_deleted_keys_length_prefixed
src/ed25519_proof.rs:29  pub enum DeletionProofError
src/ed25519_proof.rs:47  pub struct DeletionProofKeyPair
src/ed25519_proof.rs:57  pub fn generate() -> Self
src/ed25519_proof.rs:63  pub fn generate_with_rng(rng: &mut impl RngCore) -> Self
src/ed25519_proof.rs:75  pub fn from_signing_key(signing_key: ed25519_dalek::SigningKey) -> Self
src/ed25519_proof.rs:84  pub fn verifying_key_bytes(&self) -> [u8; 32]
src/ed25519_proof.rs:89  pub fn signing_key(&self) -> &ed25519_dalek::SigningKey
src/ed25519_proof.rs:94  pub fn verifying_key(&self) -> &ed25519_dalek::VerifyingKey
src/ed25519_proof.rs:102  pub fn sign_deletion_proof_v3(keypair: &DeletionProofKeyPair, proof_payload: &[u8]) -> [u8; 64]
src/ed25519_proof.rs:117  pub fn verify_deletion_proof_v3(
src/ed25519_proof.rs:139  pub enum SignatureVersion
src/ed25519_proof.rs:152  pub fn as_u8(&self) -> u8
src/error.rs:16  pub type Result<T> = std::result::Result<T, CryptoError>
src/error.rs:21  pub enum CryptoError
src/error.rs:80  pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self
src/kdf.rs:24  pub const KDF_HEADER_MAGIC: &[u8; 4] = b"MFKD"
src/kdf.rs:27  pub const KDF_HEADER_VERSION_1: u8 = 1
src/kdf.rs:30  pub const KDF_ID_ARGON2ID: u8 = 1
src/kdf.rs:33  pub const MIN_SALT_LEN: usize = 16
src/kdf.rs:36  pub const MIN_M_COST_KIB: u32 = 19456; // ~19 MiB
src/kdf.rs:38  pub const MIN_T_COST: u32 = 2
src/kdf.rs:40  pub const MIN_P_COST: u32 = 1
src/kdf.rs:43  pub const DEFAULT_M_COST_KIB: u32 = 65536; // 64 MiB
src/kdf.rs:45  pub const DEFAULT_T_COST: u32 = 3
src/kdf.rs:47  pub const DEFAULT_P_COST: u32 = 1
src/kdf.rs:51  pub struct KdfParams
src/kdf.rs:69  pub fn new(m_cost_kib: u32, t_cost: u32, p_cost: u32) -> Result<Self>
src/kdf.rs:94  pub fn new_for_test(m_cost_kib: u32, t_cost: u32, p_cost: u32) -> Self
src/kdf.rs:123  pub struct KdfHeader
src/kdf.rs:132  pub fn generate_default() -> Result<Self>
src/kdf.rs:144  pub fn new(params: KdfParams, salt: Vec<u8>) -> Result<Self>
src/kdf.rs:170  pub fn to_bytes(&self) -> Vec<u8>
src/kdf.rs:186  pub fn from_bytes(bytes: &[u8]) -> Result<Self>
src/kdf.rs:274  pub struct DerivedKey(pub [u8; 32])
src/kdf.rs:285  pub fn derive_key_argon2id(passphrase: &str, header: &KdfHeader) -> Result<DerivedKey>
src/kdf.rs:311  pub fn hkdf_expand(prk: &[u8], info: &[u8], out: &mut [u8]) -> Result<()>
src/kv_cipher.rs:42  pub use contextra_types::ModelFingerprint
src/kv_cipher.rs:46  pub const CURRENT_KV_FORMAT_VERSION: u8 = 2
src/kv_cipher.rs:54  pub struct EncryptedKvLayer
src/kv_cipher.rs:71  pub trait KvCipher: Send + Sync
src/kv_cipher.rs:115  pub struct KvSegmentCipher
src/kv_cipher.rs:123  pub fn new(key_manager: KeyManager) -> Self
src/kv_cipher.rs:131  pub fn registry(&self) -> &KeyRegistry
src/kv_cipher.rs:139  pub fn encrypt(
src/kv_cipher.rs:165  pub fn decrypt(&self, encrypted: &EncryptedKvLayer) -> Result<Vec<u8>>
src/kv_cipher.rs:180  pub fn encrypt_with_version(
src/kv_cipher.rs:202  pub fn decrypt_with_version(
src/kv_segment/delete_mode.rs:22  pub enum KvDeleteMode
src/kv_segment/eviction_worker.rs:40  pub struct EvictionWorker
src/kv_segment/eviction_worker.rs:47  pub fn spawn(store: Arc<TenantIsolatedKvStore>) -> Self
src/kv_segment/eviction_worker.rs:78  pub fn trigger_eviction(&self, target_free_bytes: usize)
src/kv_segment/eviction_worker.rs:86  pub fn shutdown(&self)
src/kv_segment/eviction_worker.rs:103  pub fn emergency_wipe(store: &TenantIsolatedKvStore)
src/kv_segment/manager.rs:22  pub struct KvSegmentConfig
src/kv_segment/manager.rs:29  pub struct KvSegmentPayload
src/kv_segment/manager.rs:46  pub struct KvSegmentManager
src/kv_segment/manager.rs:54  pub fn new(
src/kv_segment/manager.rs:67  pub fn delete_mode(&self) -> KvDeleteMode
src/kv_segment/manager.rs:72  pub fn write_segment(
src/kv_segment/manager.rs:106  pub fn read_segment(&self, segment: &KvSegmentPayload) -> Result<Vec<u8>, CryptoError>
src/kv_segment/manager.rs:141  pub fn delete_segment(&self, group_id: u64) -> Result<bool, CryptoError>
src/kv_segment/manager.rs:149  pub fn generate_deletion_proof(&self, group_id: u64) -> Result<bool, CryptoError>
src/kv_segment/mod.rs:1  pub mod delete_mode
src/kv_segment/mod.rs:2  pub mod eviction_worker
src/kv_segment/mod.rs:3  pub mod manager
src/kv_segment/mod.rs:4  pub mod segment
src/kv_segment/mod.rs:5  pub mod store
src/kv_segment/mod.rs:7  pub use delete_mode::KvDeleteMode
src/kv_segment/mod.rs:8  pub use eviction_worker::{emergency_wipe, EvictionWorker}
src/kv_segment/mod.rs:9  pub use manager::{KvSegmentConfig, KvSegmentManager, KvSegmentPayload}
src/kv_segment/mod.rs:10  pub use segment::{KvSegment, CURRENT_KV_KEY_DERIVATION_VERSION}
src/kv_segment/mod.rs:11  pub use store::TenantIsolatedKvStore
src/kv_segment/segment.rs:21  pub const CURRENT_KV_KEY_DERIVATION_VERSION: u8 = 1
src/kv_segment/segment.rs:26  pub struct EncryptedSegmentPayload
src/kv_segment/segment.rs:49  pub struct KvSegment
src/kv_segment/segment.rs:74  pub fn new(tenant_id: TenantId, segment_id: u64, data: Vec<u8>) -> Self
src/kv_segment/segment.rs:90  pub fn new_with_metadata(
src/kv_segment/segment.rs:113  pub fn new_encrypted(
src/kv_segment/segment.rs:147  pub fn decrypt_data(&self, cipher: &KvSegmentCipher) -> Result<Vec<u8>, crate::CryptoError>
src/kv_segment/segment.rs:178  pub fn to_spill_bytes(&self) -> Vec<u8>
src/kv_segment/segment.rs:189  pub fn as_bytes(&self) -> &[u8]
src/kv_segment/segment.rs:194  pub fn len(&self) -> usize
src/kv_segment/segment.rs:199  pub fn is_empty(&self) -> bool
src/kv_segment/store.rs:26  pub type SpillHandler = Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>
src/kv_segment/store.rs:121  pub struct TenantIsolatedKvStore
src/kv_segment/store.rs:136  pub const DEFAULT_SHARD_COUNT: usize = 32
src/kv_segment/store.rs:137  pub const DEFAULT_SEGMENT_CAPACITY_PER_TENANT: usize = 256
src/kv_segment/store.rs:140  pub fn new() -> Self
src/kv_segment/store.rs:146  pub fn with_shard_count(n: usize) -> Self
src/kv_segment/store.rs:168  pub fn with_capacity(segment_capacity_per_tenant: usize) -> Self
src/kv_segment/store.rs:176  pub fn set_spill_handler(&self, handler: SpillHandler)
src/kv_segment/store.rs:187  pub fn insert_segment(&self, tenant: TenantId, segment: KvSegment)
src/kv_segment/store.rs:210  pub fn on_rollback(&self, tenant: TenantId, chunk_ids: &[u64])
src/kv_segment/store.rs:219  pub fn remove_segments_for_rollback(&self, tenant: TenantId, segment_ids: &[u64])
src/kv_segment/store.rs:247  pub fn remove_segment(&self, tenant: TenantId, segment_id: u64)
src/kv_segment/store.rs:252  pub fn get_segment_bytes(&self, tenant: TenantId, segment_id: u64) -> Option<Vec<u8>>
src/kv_segment/store.rs:263  pub fn insert_encrypted_segment(
src/kv_segment/store.rs:286  pub fn get_segments(&self, tenant: TenantId) -> Vec<u64>
src/kv_segment/store.rs:298  pub fn get_decrypted_segment(
src/kv_segment/store.rs:316  pub fn get_tenant_segment_len(&self, tenant: TenantId) -> usize
src/kv_segment/store.rs:395  pub fn evict_lru_fair(&self, target_free_bytes: usize) -> usize
src/kv_segment/store.rs:407  pub fn evict_lru_fair_with_hook<F>(&self, target_free_bytes: usize, mut hook: F) -> usize
src/kv_segment/store.rs:510  pub fn clear_all(&self)
src/kv_shredding.rs:34  pub const DEFAULT_SHRED_KEY_GROUP_SIZE: usize = 64
src/kv_shredding.rs:38  pub struct SubKey(pub [u8; 32])
src/kv_shredding.rs:56  pub struct GroupKek(pub [u8; 32])
src/kv_shredding.rs:68  pub struct RecordDek(pub [u8; 32])
src/kv_shredding.rs:81  pub struct EncryptedRecordPayload
src/kv_shredding.rs:112  pub fn derive_subkey(
src/kv_shredding.rs:122  pub struct KeyRegistry
src/kv_shredding.rs:130  pub fn new() -> Self
src/kv_shredding.rs:140  pub fn with_revocation_log(mut self, log: std::sync::Arc<RevocationLog>) -> Self
src/kv_shredding.rs:162  pub fn get_wrapped_kek(&self, group_id: u64) -> Option<(Vec<u8>, [u8; 12])>
src/kv_shredding.rs:172  pub fn get_wrapped_dek(&self, group_id: u64, record_id: u64) -> Option<(Vec<u8>, [u8; 12])>
src/kv_shredding.rs:186  pub fn is_group_revoked(&self, group_id: u64) -> bool
src/kv_shredding.rs:201  pub fn get_or_derive(&self, master_key: &KeyManager, group_id: u64) -> Result<SubKey>
src/kv_shredding.rs:261  pub fn revoke_group(&self, group_id: u64) -> Result<bool>
src/kv_shredding.rs:300  pub fn revoke_subkey(&self, group_id: u64) -> Result<bool>
src/kv_shredding.rs:306  pub fn revoke_record(&self, group_id: u64, record_id: u64) -> bool
src/kv_shredding.rs:326  pub fn is_group_active(&self, group_id: u64) -> bool
src/kv_shredding.rs:336  pub fn is_key_active(&self, group_id: u64) -> bool
src/kv_shredding.rs:341  pub fn is_record_active(&self, group_id: u64, record_id: u64) -> bool
src/kv_shredding.rs:356  pub fn encrypt_record(
src/kv_shredding.rs:411  pub fn decrypt_record(
src/kv_shredding.rs:466  pub fn encrypt_with_group(
src/kv_shredding.rs:494  pub fn decrypt_with_group(
src/lib.rs:43  pub mod aead
src/lib.rs:44  pub mod anti_tamper
src/lib.rs:45  pub mod audit_chain
src/lib.rs:46  pub mod crypto
src/lib.rs:47  pub mod dek
src/lib.rs:53  pub mod deletion_proof
src/lib.rs:59  pub mod ed25519_proof
src/lib.rs:60  pub mod error
src/lib.rs:61  pub mod kdf
src/lib.rs:62  pub mod kv_cipher
src/lib.rs:63  pub mod kv_segment
src/lib.rs:69  pub mod kv_shredding
src/lib.rs:75  pub mod revocation_log
src/lib.rs:76  pub mod secret
src/lib.rs:77  pub mod wal_completeness
src/lib.rs:78  pub mod wal_crypto
src/lib.rs:80  pub use aead::*
src/lib.rs:81  pub use dek::{derive_dek, AncestorSalts, Dek, UnitKey}
src/lib.rs:82  pub use kdf::hkdf_expand
src/lib.rs:83  pub use secret::SecretKey
src/lib.rs:85  pub use audit_chain::
src/lib.rs:89  pub use crypto::KeyManager as CryptoKey
src/lib.rs:90  pub use crypto::KeyManager
src/lib.rs:93  pub use deletion_proof::
src/lib.rs:99  pub use ed25519_proof::{DeletionProofError, SignatureVersion}
src/lib.rs:100  pub use error::{CryptoError, Result}
src/lib.rs:101  pub use kdf::{derive_key_argon2id, KdfHeader, KdfParams}
src/lib.rs:103  pub use kv_cipher::KvSegmentCipher
src/lib.rs:104  pub use kv_cipher::{EncryptedKvLayer, KvCipher, ModelFingerprint}
src/lib.rs:105  pub use kv_segment::
src/lib.rs:115  pub use kv_shredding::{derive_subkey, KeyRegistry, SubKey, DEFAULT_SHRED_KEY_GROUP_SIZE}
src/lib.rs:118  pub use revocation_log::{RevocationEntry, RevocationLog, RevocationTarget}
src/lib.rs:119  pub use wal_completeness::verify_wal_chain_completeness
src/lib.rs:121  pub use wal_crypto::CloudEgressGuard
src/lib.rs:122  pub use wal_crypto::
src/revocation_log.rs:44  pub enum RevocationTarget
src/revocation_log.rs:57  pub fn canonical_bytes(&self) -> Vec<u8>
src/revocation_log.rs:69  pub struct RevocationEntry
src/revocation_log.rs:88  pub fn serialize<S>(bytes: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
src/revocation_log.rs:95  pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
src/revocation_log.rs:107  pub fn compute_entry_hash(
src/revocation_log.rs:124  pub struct RevocationLog
src/revocation_log.rs:135  pub fn new_in_memory(
src/revocation_log.rs:155  pub fn open_or_create(
src/revocation_log.rs:195  pub fn verify_integrity(&self) -> Result<()>
src/revocation_log.rs:242  pub fn append(&self, target: RevocationTarget) -> Result<RevocationEntry>
src/revocation_log.rs:292  pub fn revoked_targets(&self) -> AHashSet<RevocationTarget>
src/revocation_log.rs:297  pub fn is_revoked(&self, target: &RevocationTarget) -> bool
src/revocation_log.rs:303  pub fn len(&self) -> usize
src/revocation_log.rs:308  pub fn is_empty(&self) -> bool
src/secret.rs:8  pub struct SecretKey<const N: usize>
src/secret.rs:14  pub fn new(bytes: [u8; N]) -> Self
src/secret.rs:20  pub fn expose_secret(&self) -> &[u8; N]
src/secret.rs:26  pub fn as_bytes(&self) -> &[u8; N]
src/secret.rs:59  pub struct Secret<T: Zeroize>
src/secret.rs:65  pub fn new(inner: T) -> Self
src/secret.rs:71  pub fn expose_secret(&self) -> &T
src/secret.rs:77  pub fn expose_secret_mut(&mut self) -> &mut T
src/wal_completeness.rs:45  pub fn verify_wal_chain_completeness(
src/wal_crypto.rs:32  pub trait KmsProvider: Send + Sync
src/wal_crypto.rs:55  pub struct LocalKeyManagerKms
src/wal_crypto.rs:63  pub fn new(key_manager: KeyManager, key_id: impl Into<String>) -> Self
src/wal_crypto.rs:72  pub fn key_manager(&self) -> &KeyManager
src/wal_crypto.rs:146  pub struct CloudEgressGuard
src/wal_crypto.rs:160  pub fn new() -> Self
src/wal_crypto.rs:167  pub fn with_max_bytes(max_egress_bytes: usize) -> Self
src/wal_crypto.rs:172  pub fn validate_egress_payload(&self, payload: &[u8]) -> Result<()>
src/wal_crypto.rs:203  pub struct EncryptedWal
src/wal_crypto.rs:216  pub fn new(key_manager: KeyManager, file_id: &[u8]) -> Result<Self>
src/wal_crypto.rs:235  pub fn with_kms(kms: Arc<dyn KmsProvider>) -> Self
src/wal_crypto.rs:240  pub fn kms(&self) -> &Arc<dyn KmsProvider>
src/wal_crypto.rs:246  pub fn encrypt_chunk(&self, payload: &[u8]) -> Result<Vec<u8>>
src/wal_crypto.rs:258  pub fn decrypt_chunk(&self, data: &[u8]) -> Result<Vec<u8>>
src/wal_crypto.rs:291  pub struct WalHmac
src/wal_crypto.rs:296  pub fn new(integrity_key: &[u8]) -> Result<Self>
src/wal_crypto.rs:313  pub fn update(&mut self, data: &[u8])
src/wal_crypto.rs:318  pub fn finalize(self) -> [u8; 32]
src/wal_crypto.rs:333  pub struct WalEntrySnapshot
src/wal_crypto.rs:361  pub struct IntegrityVerifier
src/wal_crypto.rs:368  pub fn new(integrity_key: &[u8]) -> Self
src/wal_crypto.rs:382  pub fn set_last_hmac(&mut self, hmac: [u8; 32])
src/wal_crypto.rs:387  pub fn last_hmac_snapshot(&self) -> [u8; 32]
src/wal_crypto.rs:392  pub fn set_last_seq_no(&mut self, seq_no: Option<u64>)
src/wal_crypto.rs:397  pub fn last_seq_no_snapshot(&self) -> Option<u64>
src/wal_crypto.rs:402  pub fn verify_and_update_v3(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>
src/wal_crypto.rs:491  pub fn verify_and_update_v2(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>
src/wal_crypto.rs:560  pub fn skip_hmac_verify_legacy(&mut self, entry: &WalEntrySnapshot)
src/wal_crypto.rs:566  pub fn verify_and_update(&mut self, entry: &WalEntrySnapshot, offset: u64) -> Result<()>
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/deletion_proof/creation.rs:443  pub(crate) fn construct_v5_payload(&self) -> std::result::Result<Vec<u8>, CryptoError>
src/deletion_proof/creation.rs:488  pub(crate) fn construct_v3_payload(&self) -> std::result::Result<Vec<u8>, CryptoError>
src/deletion_proof/creation.rs:551  pub(crate) fn compute_hmac_sha256(key: &[u8], data_parts: &[&[u8]]) -> Result<[u8; 32]>
src/deletion_proof/types.rs:169  pub(crate) layer: DeletionLayer,
src/deletion_proof/types.rs:170  pub(crate) _private: (),
src/kv_segment/store.rs:328  pub(crate) fn evict_lru_global(&self, target_free_bytes: usize) -> usize
```
