//! Durable high-water reservations. Ranges are abandoned on reopen.

use super::ops::CatalogOp;
use contextra_storage_ports::{Error, Stamp, STAMP_MAX, SYSTEM_SPACE_ID_MAX};

pub const STAMP_RESERVATION: Stamp = 1 << 20;
pub const ID_RESERVATION: u64 = 1 << 16;
pub const SPACE_ID_RESERVATION: u64 = 1 << 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reservation { pub first: u64, pub through: u64 }

impl Reservation {
    pub fn contains(&self, value: u64) -> bool { value >= self.first && value <= self.through }
}

/// Prepare a monotone reservation edit; the caller must durably commit the
/// returned op before using any ID in the range.
pub fn next_stamp_reservation(current_limit: Stamp, amount: Stamp) -> Result<(CatalogOp, Reservation), Error> {
    let amount = amount.max(1);
    let through = current_limit.checked_add(amount).filter(|v| *v <= STAMP_MAX)
        .ok_or(Error::Limit { limit: "stamp space exhausted" })?;
    let first = current_limit.checked_add(1).ok_or(Error::Limit { limit: "stamp space exhausted" })?;
    Ok((CatalogOp::ReserveStamps { through }, Reservation { first, through }))
}

pub fn next_id_reservation(current_limit: u64, amount: u64) -> Result<(CatalogOp, Reservation), Error> {
    let through = current_limit.checked_add(amount.max(1)).ok_or(Error::Limit { limit: "ID space exhausted" })?;
    let first = current_limit.checked_add(1).ok_or(Error::Limit { limit: "ID space exhausted" })?;
    Ok((CatalogOp::ReserveIds { through }, Reservation { first, through }))
}

/// User-space IDs start after the reserved system range. The high-water mark
/// is persisted before any ID in the returned interval may be issued.
pub fn next_space_id_reservation(current_limit: u64, amount: u64) -> Result<(CatalogOp, Reservation), Error> {
    let first_available = current_limit.max(SYSTEM_SPACE_ID_MAX).checked_add(1).ok_or(Error::Exhausted)?;
    let through = first_available.checked_add(amount.max(1) - 1).ok_or(Error::Exhausted)?;
    Ok((CatalogOp::ReserveSpaceIds { through }, Reservation { first: first_available, through }))
}

/// Recovery starts past the entire persisted range, preventing stamp reuse
/// even when only a prefix of that range reached Redo before the crash.
pub fn after_reopen(stamp_limit: Stamp, id_limit: u64, space_id_limit: u64) -> Result<(Stamp, u64, u64), Error> {
    let stamp = stamp_limit.checked_add(1).filter(|v| *v <= STAMP_MAX)
        .ok_or(Error::Exhausted)?;
    let id = id_limit.checked_add(1).ok_or(Error::Exhausted)?;
    let space_id = space_id_limit.max(SYSTEM_SPACE_ID_MAX).checked_add(1).ok_or(Error::Exhausted)?;
    Ok((stamp, id, space_id))
}
