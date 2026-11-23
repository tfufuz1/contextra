//! Lösch-Ticket und monotone Fortschrittsprojektion (SP-ERASE §9, §13).
//!
//! Implementierergrenze: Tickets referenzieren den durable Ledger-Intent und
//! leiten ihren Status ausschließlich aus validierten Ledger-/Catalog-/Owner-
//! Belegen ab. RAM-Zustand ist Cache und darf nach Recovery rekonstruiert werden.
//! Statusabfolge darf niemals zurücklaufen; `Blocked` ist fortsetzbar, `Failed`
//! terminal. `CompleteWithResidual` darf keinen fehlenden Pflichtbeleg kaschieren.
//! Persistenz-/Sequencerlogik gehört in `protocol.rs`, kanonische Bytes in
//! `receipt.rs`, Hashkette und Records in SP-LEDGER.
//!
//! Kein synthetischer Erfolg und kein `Done` ohne finalen Purge-Edit, leeren
//! Graveyard, alle erforderlichen Owner-Atteste und erfolgreiches Verify.
