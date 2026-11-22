//! # Redo-Log Modul
//!
//! Segmentiertes Wiederholungsprotokoll mit Absturzsicherheit als Sans-I/O-Zustandsmaschine
//! nach SPEC §9.4, §10.2, §10.5, §13.1, §13.2, §16.6 und §18.

pub mod driver;
pub mod error;
pub mod frame;
pub mod header;
pub mod op;
pub mod replay;
pub mod replay_sm;
pub mod writer_sm;

pub use driver::{segment_file_name, RedoWriter, DEFAULT_SEGMENT_BYTES};
pub use error::{check_redo_hwm, RedoError};
pub use frame::{Durability, RedoFrame, RedoIntegrityMode, FLAG_SEAL, SEAL_OP_COUNT};
pub use header::{RedoHeader, FORMAT_VERSION, HEADER_SIZE, MAGIC};
pub use op::{RedoOp, RedoOpKind, OPFLAG_DOMAIN, OPFLAG_SEALED, OPFLAG_UNBOUNDED_END};
pub use replay::{parse_segment_id, recover_redo, ReplayResult};
pub use replay_sm::{ReplayEffect, ReplayEvent, ReplayStateMachine};
pub use writer_sm::{WriterEffect, WriterEvent, WriterState, WriterStateMachine};
