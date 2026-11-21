# contextra-store

**Zweck:** LSM-Tree-Speicher-Engine: WAL, SST, Manifest, KV-Familien. Größtes Crate des Workspace. Spec: `G-STORE`, `CMP-4`, Kapitel 5 (Datenmodell, Segmente).

**Ebene:** E1

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod error
pub mod traits
pub use error::{StoreError, StoreResult}
pub mod budget
pub mod stats
pub mod compaction
pub mod fsync
pub mod key_source
pub mod lsm
pub mod store_layer
pub mod manifest
pub mod memtable
pub mod sstable
pub mod system_pressure
pub mod tenant_codec
pub mod util
pub mod wal
pub use util::{RecordAuthenticator, RecordSealer, SealedEnvelope}
pub use compaction::
pub use key_source::{KeyMaterialSource, OsKeyMaterialSource, StaticKeyMaterialSource}
pub use lsm::
pub use store_layer::StoreLayer
pub use manifest::{Manifest, ManifestEntry}
pub use system_pressure::{PressureLevel, SystemPressure, SystemPressureMonitor}
pub use tenant_codec::{TenantKeyCodec, TenantScopedStorage}
pub mod io_seam
pub use io_seam::{RealFs, RealFsFile, StoreIo, Vfs, VfsFile, VfsOpenOptions}
```

## Features (`Cargo.toml`)

`default`, `workspace`, `test-hooks`, `inspect`, `memory-only-storage`, `fault-injection`, `block-cache-v2`, `sieve-cache`, `docid-128`, `legacy-wal-key`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

