//! Makro-Shim fuer optionale Tracing-Instrumentierung (SPEC §4.2, Minimal-Default M5).
//!
//! Bei aktivem `tracing`-Feature werden alle Makros direkt an `tracing` delegiert.
//! Bei deaktiviertem Feature (`--no-default-features`) expandieren sie spurlos zu
//! leeren Statements/Ausdruecken, ohne Allokationen, Laufzeitkosten oder Seiteneffekte.

#[cfg(feature = "tracing")]
#[doc(hidden)]
pub use tracing::{debug, error, info, trace, warn};

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! _store_trace {
    ($($arg:tt)*) => {
        ()
    };
}

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! _store_debug {
    ($($arg:tt)*) => {
        ()
    };
}

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! _store_info {
    ($($arg:tt)*) => {
        ()
    };
}

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! _store_warn {
    ($($arg:tt)*) => {
        ()
    };
}

#[cfg(not(feature = "tracing"))]
#[doc(hidden)]
#[macro_export]
macro_rules! _store_error {
    ($($arg:tt)*) => {
        ()
    };
}

#[cfg(not(feature = "tracing"))]
pub use _store_debug as debug;
#[cfg(not(feature = "tracing"))]
pub use _store_error as error;
#[cfg(not(feature = "tracing"))]
pub use _store_info as info;
#[cfg(not(feature = "tracing"))]
pub use _store_trace as trace;
#[cfg(not(feature = "tracing"))]
pub use _store_warn as warn;
