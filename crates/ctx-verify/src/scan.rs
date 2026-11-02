//! Independent field scanner module per SPEC §8.17, §12.1, D-72.
//!
//! Provides MK1 (structural non-presence), MK2 (raw pattern / needle absence),
//! and MK3 (key destruction) verification.
//!
//! Invariants:
//! - HR-4: No plaintext file paths in logs or outputs, only BLAKE3 hashes.
//! - Safe passphrase read exclusively via file descriptor.
//! - Zero secrets in outputs.

use crate::codes::InvalidCode;
use blake3::Hasher;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zeroize::Zeroize;

/// Field scan options.
#[derive(Debug)]
pub struct ScanOptions {
    pub target_tag: [u8; 16],
    pub passphrase: Vec<u8>,
}

/// Safely reads the passphrase from a file descriptor into a zeroizable buffer.
/// On Linux/Unix, reads from `/proc/self/fd/<fd>` without requiring raw memory access.
pub fn read_passphrase_from_fd(fd: i32) -> Result<Vec<u8>, InvalidCode> {
    if fd < 0 {
        return Err(InvalidCode::PassphraseFdMissing);
    }

    let fd_path = format!("/proc/self/fd/{}", fd);
    let mut file = File::open(&fd_path).map_err(|_| InvalidCode::PassphraseFdMissing)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|_| InvalidCode::PassphraseFdMissing)?;

    if buf.is_empty() {
        return Err(InvalidCode::PassphraseFdMissing);
    }

    Ok(buf)
}

/// Computes an HR-4 compliant hash representation of a path.
pub fn hash_path(path: &Path) -> String {
    let mut hasher = Hasher::new();
    hasher.update(path.to_string_lossy().as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Executes independent field scan on data directory.
pub fn execute_field_scan(
    data_dir: &Path,
    target: &str,
    passphrase_fd: i32,
) -> Result<(), InvalidCode> {
    let mut passphrase = read_passphrase_from_fd(passphrase_fd)?;
    let res = execute_scan_internal(data_dir, target, &passphrase);
    passphrase.zeroize();
    res
}

fn execute_scan_internal(
    data_dir: &Path,
    target: &str,
    _passphrase: &[u8],
) -> Result<(), InvalidCode> {
    if !data_dir.exists() || !data_dir.is_dir() {
        return Err(InvalidCode::IoError);
    }

    let target_bytes = hex::decode(target).map_err(|_| InvalidCode::Mk1StructureResidue)?;
    if target_bytes.is_empty() {
        return Err(InvalidCode::Mk1StructureResidue);
    }

    // Check directory recursively
    scan_dir_recursive(data_dir, &target_bytes)
}

fn scan_dir_recursive(dir: &Path, target_bytes: &[u8]) -> Result<(), InvalidCode> {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Err(InvalidCode::IoError),
    };

    for entry_res in entries {
        let entry = match entry_res {
            Ok(e) => e,
            Err(_) => return Err(InvalidCode::IoError),
        };
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(&path, target_bytes)?;
        } else if path.is_file() {
            let content = match std::fs::read(&path) {
                Ok(c) => c,
                Err(_) => return Err(InvalidCode::IoError),
            };

            // Check if file name indicates vault
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if filename == "vault.dat" {
                // If vault contains active slot for target, MK3 fails
                // F.3 slot layout check
                if contains_active_vault_slot(&content, target_bytes) {
                    return Err(InvalidCode::Mk3KeyPresent);
                }
            }

            // MK1 / MK2 scan: check for target bytes
            if content
                .windows(target_bytes.len())
                .any(|w| w == target_bytes)
            {
                return Err(InvalidCode::Mk2RawResidue);
            }
        }
    }

    Ok(())
}

fn contains_active_vault_slot(vault_data: &[u8], target_tag: &[u8]) -> bool {
    if target_tag.len() != 16 {
        return false;
    }
    // F.3: header (128) + metablock (64) = 192, then 80-byte slots
    // slot: 0: slot_type (1), 1..17: tag (16), 17..49: payload (32), 49: status (1: 0x01 aktiv, 0x02 tombstone)
    if vault_data.len() < 192 {
        return false;
    }

    let slots_data = match vault_data.get(192..) {
        Some(s) => s,
        None => return false,
    };

    for chunk in slots_data.chunks_exact(80) {
        let tag = match chunk.get(1..17) {
            Some(t) => t,
            None => continue,
        };
        let status = match chunk.get(49) {
            Some(s) => *s,
            None => continue,
        };
        if tag == target_tag && status == 0x01 {
            return true;
        }
    }

    false
}
