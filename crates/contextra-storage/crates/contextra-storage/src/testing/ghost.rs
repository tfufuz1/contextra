//! Ghost journal inspection for O-Device / historical-write verification (SP-ENV §3.4).

use super::canary::find_canary_in_slice;
use crate::env::faultfs::FaultFs;
use contextra_storage_ports::traits::Fs;
use contextra_storage_ports::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GhostReport {
    pub entries: usize,
    pub total_bytes: usize,
}

pub fn summarize_ghost_journal<F: Fs>(fs: &FaultFs<F>) -> Result<GhostReport, Error> {
    let journal = fs.ghost_journal()?;
    let total_bytes = journal.iter().map(Vec::len).sum();
    Ok(GhostReport {
        entries: journal.len(),
        total_bytes,
    })
}

pub fn verify_no_canary_in_ghost_journal<F: Fs>(
    fs: &FaultFs<F>,
    canaries: &[Vec<u8>],
) -> Result<bool, Error> {
    let journal = fs.ghost_journal()?;
    for entry in journal {
        for needle in canaries {
            if find_canary_in_slice(&entry, needle).is_some() {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
