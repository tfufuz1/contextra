# contextra-ids

**Zweck:** Kanonische Identitäts- und Tenant-Typen. Laut Spec (ADR-33, ADR-52) die **einzige Definition von Limits** (`MAX_SEARCH_K` etc. gehören hierher) und Aufnahmeort der Typen aus `contextra-types` (REM-30).

**Ebene:** E0

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub const ID_WIDTH_BITS: u32 = if cfg!(feature = "docid-128") { 128 } else { 64 }
pub const TOMBSTONE_BIT: u64 = 1 << 63
pub const MAX_SEARCH_K: usize = 1_000
pub const MAX_REF_LEN: usize = 256
pub enum IdError
pub type Result<T> = core::result::Result<T, IdError>
pub struct TenantId(pub u64)
pub struct CollectionId(pub u64)
pub struct DocId(pub u64)
pub struct DocId(pub u128)
pub struct EntityId(pub u64)
pub struct TxId(pub u64)
pub struct FileId(pub [u8; 16])
pub struct StoreId(pub [u8; 16])
pub struct Seq(pub u64)
pub struct Timestamp(pub u64)
pub struct SubjectRef([u8; 16])
pub struct PersonalRef([u8; 16])
pub struct ErasureId(pub [u8; 16])
pub struct GateId(pub [u8; 16])
pub struct DocRef
pub enum TenantScopeViolation
pub struct TenantScoped<T>
```

## Features (`Cargo.toml`)

`default`, `std`, `docid-128`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

