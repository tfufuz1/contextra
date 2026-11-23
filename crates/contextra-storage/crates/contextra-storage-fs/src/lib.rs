//! RealFs (SP-ENV §3.5). Einziger Crate mit unsafe-Erlaubnis; Budget per cargo-geiger.
//! Kein Lint-Vererben: unsafe bleibt hier gesperrt, bis WP-03.3 entscheidet (rustix bevorzugt).
pub mod realfs;
