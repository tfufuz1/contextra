# contextra-env

**Zweck:** Injizierbare Umgebung: `Clock`, `SeededRng`, `MetricsSink`, ID-Erzeugung. **Einziger Ort**, an dem `Instant::now` und `SystemTime::now` aufgerufen werden dürfen (R10, G-12).

**Ebene:** E0

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod clock
pub mod env
pub mod id_gen
pub mod metrics
pub mod rng
pub mod seq_log
pub mod manual_clock
pub use clock::{Clock, SystemClock}
pub use env::*
pub use id_gen::{IdGen, SequentialIdGen}
pub use metrics::
pub use rng::{Rng, SeededRng}
pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog}
pub use manual_clock::ManualClock
pub fn unix_ms(c: &dyn Clock) -> u64
```

## Features (`Cargo.toml`)

`default`, `std`, `test-utils`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

