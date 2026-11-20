## contextra-ids

Pfad: `crates/contextra-ids`

Abhängigkeiten (direkt): blake3, serde, thiserror, rand, zeroize

### Module

- `/src/lib.rs`
- `/src/tests.rs`

### Öffentliche Typen

- **CollectionId** (struct) – src/lib.rs:161
- **DocId** (struct) – src/lib.rs:237
- **DocRef** (struct) – src/lib.rs:888
- **EntityId** (struct) – src/lib.rs:393
- **ErasureId** (struct) – src/lib.rs:806
- **FileId** (struct) – src/lib.rs:560
- **GateId** (struct) – src/lib.rs:852
- **IdError** (enum) – src/lib.rs:44
- **PersonalRef** (struct) – src/lib.rs:766
- **Result** (type) – src/lib.rs:77
- **Seq** (struct) – src/lib.rs:614
- **StoreId** (struct) – src/lib.rs:586
- **SubjectRef** (struct) – src/lib.rs:716
- **TenantId** (struct) – src/lib.rs:92
- **TenantScopeViolation** (enum) – src/lib.rs:899
- **TenantScoped** (struct) – src/lib.rs:916
- **Timestamp** (struct) – src/lib.rs:640
- **TxId** (struct) – src/lib.rs:469

### Öffentliche Funktionen

- `pub fn as_bytes(&self) -> &[u8; 16] `  – src/lib.rs:743
- `pub fn as_bytes(&self) -> &[u8] `  – src/lib.rs:679
- `pub fn as_bytes(&self) -> Vec<u8> `  – src/lib.rs:415
- `pub fn doc_id_from_key_128(key: &str) -> Result<u128, &'static str> `  – tests/legacy_reference/types_legacy.rs:27
- `pub fn doc_id_from_key_64(key: &str) -> Result<u64, &'static str> `  – tests/legacy_reference/types_legacy.rs:20
- `pub fn from_doc_id(doc_id: DocId) -> Self `  – src/lib.rs:421
- `pub fn from_doc_id(doc_id: DocId) -> Self `  – src/lib.rs:427
- `pub fn from_key(key: &str) -> Result<Self> `  – src/lib.rs:259
- `pub fn from_key(key: &str) -> Result<Self> `  – src/lib.rs:317
- `pub fn from_key(key: &str) -> Result<Self> `  – src/lib.rs:432
- `pub fn generate() -> Result<Self> `  – src/lib.rs:720
- `pub fn generate() -> Result<Self> `  – src/lib.rs:770
- `pub fn generate() -> Result<Self> `  – src/lib.rs:810
- `pub fn hash_key_u128(key: &str) -> u128 `  – tests/legacy_reference/types_legacy.rs:13
- `pub fn hash_key_u64(key: &str) -> u64 `  – tests/legacy_reference/types_legacy.rs:6
- `pub fn inner(&self) -> &T `  – src/lib.rs:953
- `pub fn into_inner(self) -> Vec<u8> `  – src/lib.rs:685
- `pub fn into_inner_checked(`  – src/lib.rs:958
- `pub fn is_valid_origin(&self) -> bool `  – src/lib.rs:516
- `pub fn map<U>(self, f: impl FnOnce(T) -> U) -> TenantScoped<U> `  – src/lib.rs:973
- `pub fn new(tenant_id: TenantId, value: T) -> Self `  – src/lib.rs:935
- `pub fn tenant_id(&self) -> &TenantId `  – src/lib.rs:941
- `pub fn try_from_internal_offset(offset: u64) -> Result<Self> `  – src/lib.rs:507
- `pub fn try_new(`  – src/lib.rs:923
- `pub fn try_new(bytes: Vec<u8>) -> Result<Self> `  – src/lib.rs:667
- `pub fn try_new(id: u64) -> Result<Self> `  – src/lib.rs:110
- `pub fn try_new(id: u64) -> Result<Self> `  – src/lib.rs:176
- `pub fn value(&self) -> &T `  – src/lib.rs:947
- `pub fn vector_doc_id_from_key(key: &str) -> Result<u64, &'static str> `  – tests/legacy_reference/vector_legacy.rs:6

---
