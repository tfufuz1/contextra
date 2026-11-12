# Frozen Surface (Schnittstellenvertrag STORE-MOD-G)

Die folgenden Symbole und Pfade bilden die Außenfläche von `contextra-store` und sind fest eingefroren.

## 1. Re-Exporte in `src/lib.rs`
- `StoreError`, `StoreResult` aus `crate::error`
- `budget`: `ResourceBudget`, `ResourceTracker`
- `stats`: `StorageStats`
- `traits`: `StoreFile`, `StoreFileOpenOptions`, `StoreFs`
- `compat`: Brücken zu `contextra-types` und `contextra-ports`
- `lsm`: `LsmStorage`, `LsmConfig`
- `manifest`: `Manifest`, `ManifestEntry`
- `sstable`: `SstableBuilder`, `SstableReader`
- `wal`: `Wal`, `WalOp`, `WalEntry`
- `tenant_codec`: `TenantKeyCodec`

## 2. Querschnittliche Basismodule
- `src/error.rs`: `StoreError` (`#[non_exhaustive]`), `StoreResult<T>`
- `src/key_source.rs`: `KeySource`, `StaticKeySource`
- `src/util.rs`: `RecordSealer`, `RecordAuthenticator`, `SealedEnvelope`
- `src/io_seam.rs`: `Vfs`, `VfsFile`, `VfsOpenOptions`, `VerifiedVia`, `RealFs`, `RealFsFile`, `StoreIo`
- `src/traits.rs`: `StoreFile`, `StoreFs`, `StoreFileOpenOptions`
- `src/fsync.rs`: Durability primitives (`durable_flush_file`, `durable_flush_dir`)
- `src/tenant_codec.rs`: Tenant-scoped keys, prefixes, and codec helpers
