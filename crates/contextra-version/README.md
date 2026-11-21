<<<<<<< HEAD:crates/contextra-version/README.md
# contextra-version

**Zweck:** Multi-Version Concurrency Control: Sequenz-Log, Transaktionspuffer, Versionskatalog. **Ist-Name `contextra-version`.** Laut Spec (ADR-06) wird das Crate zu **`contextra-version`** umbenannt, mit Versionskatalog, Snapshots und Aufbewahrung.
=======
# contextra-mvcc

**Zweck:** Multi-Version Concurrency Control: Sequenz-Log, Transaktionspuffer, Versionskatalog. **Ist-Name `contextra-mvcc`.** Laut Spec (ADR-06) wird das Crate zu **`contextra-version`** umbenannt, mit Versionskatalog, Snapshots und Aufbewahrung.
>>>>>>> a9b569ed6 (Shell-Commit):crates/contextra-mvcc/README.md

**Ebene:** E1 (Ist-Name, Ziel contextra-version, ADR-06)

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub use crate::mvcc_error::{MvccError, Result as MvccResult}
pub use contextra_ids::{DocId, TenantId, TxId, TOMBSTONE_BIT}
pub mod types
pub mod compat
pub use crate::compat::{workspace_error as error, ContextraError, Result}
pub use crate::mvcc_error::Result
pub mod mvcc_error
pub use contextra_env::seq_log
pub mod snapshot
pub mod ssi
pub mod tombstone
pub mod tx_buffer
pub use contextra_env::seq_log::{SeqLogChange, SeqLogEntry, SequenceLog}
pub use snapshot::{SnapshotGuard, SnapshotRegistry}
pub use ssi::{PruningBlockerInfo, ReadSet, SequenceLogSsiValidator, SsiValidator}
pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck}
pub use tx_buffer::{IndexOp, TxBuffer}
```

## Features (`Cargo.toml`)

`default`, `workspace`, `types-compat`, `ports-compat`, `docid-128`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

