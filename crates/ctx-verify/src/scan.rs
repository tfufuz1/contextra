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
    execute_field_scan_with_control(data_dir, target, passphrase_fd, None)
}

/// Executes independent field scan with positive control pattern verification.
/// ZERO-TRUST: Ein Negativbefund ohne bestandene Positivkontrolle ist ungültig.
pub fn execute_field_scan_with_control(
    data_dir: &Path,
    target: &str,
    passphrase_fd: i32,
    control_needle: Option<&[u8]>,
) -> Result<(), InvalidCode> {
    let mut passphrase = read_passphrase_from_fd(passphrase_fd)?;
    let res = execute_scan_internal(data_dir, target, &passphrase, control_needle);
    passphrase.zeroize();
    res
}

fn execute_scan_internal(
    data_dir: &Path,
    target: &str,
    _passphrase: &[u8],
    control_needle: Option<&[u8]>,
) -> Result<(), InvalidCode> {
    if !data_dir.exists() || !data_dir.is_dir() {
        return Err(InvalidCode::IoError);
    }

    let target_bytes = hex::decode(target).map_err(|_| InvalidCode::Mk1StructureResidue)?;
    if target_bytes.len() != 16 {
        return Err(InvalidCode::Mk1StructureResidue);
    }
    let mut target_arr = [0u8; 16];
    target_arr.copy_from_slice(&target_bytes);

    // Positivkontrolle: Falls ein Kontrollmuster vorgegeben ist, MUSS es im Verzeichnis nachweisbar sein.
    // Wenn kein Muster vorgegeben ist, führen wir eine automatische Positivkontrolle auf Datenkonsistenz/Vorhandensein durch (Zero-Trust Z3).
    if let Some(needle) = control_needle {
        if needle.is_empty() {
            return Err(InvalidCode::MissingPositiveControl);
        }
        let mut control_found = false;
        check_control_needle(data_dir, needle, &mut control_found)?;
        if !control_found {
            return Err(InvalidCode::MissingPositiveControl);
        }
    } else {
        // Fallback-Positivkontrolle: Das Verzeichnis muss lesbar sein und mindestens eine gültige Datei enthalten
        let mut has_files = false;
        check_has_files(data_dir, &mut has_files)?;
        if !has_files {
            return Err(InvalidCode::MissingPositiveControl);
        }
    }

    // Check directory recursively using structured parsers
    scan_dir_recursive(data_dir, &target_arr)
}

fn check_has_files(dir: &Path, has_files: &mut bool) -> Result<(), InvalidCode> {
    if *has_files {
        return Ok(());
    }
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
            check_has_files(&path, has_files)?;
            if *has_files {
                return Ok(());
            }
        } else if path.is_file() {
            *has_files = true;
            return Ok(());
        }
    }
    Ok(())
}

fn check_control_needle(dir: &Path, needle: &[u8], found: &mut bool) -> Result<(), InvalidCode> {
    if *found || needle.is_empty() {
        return Ok(());
    }
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
            check_control_needle(&path, needle, found)?;
            if *found {
                return Ok(());
            }
        } else if path.is_file() {
            let content = match std::fs::read(&path) {
                Ok(c) => c,
                Err(_) => return Err(InvalidCode::IoError),
            };
            if content.windows(needle.len()).any(|w| w == needle) {
                *found = true;
                return Ok(());
            }
        }
    }
    Ok(())
}

fn scan_dir_recursive(dir: &Path, target_tag: &[u8; 16]) -> Result<(), InvalidCode> {
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
            scan_dir_recursive(&path, target_tag)?;
        } else if path.is_file() {
            let content = match std::fs::read(&path) {
                Ok(c) => c,
                Err(_) => return Err(InvalidCode::IoError),
            };

            // Check if file name indicates vault shard (<nn>.vlt per SPEC F.3/D-111) or legacy vault.dat
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let is_vault = filename == "vault.dat"
                || (filename.ends_with(".vlt")
                    && filename.len() == 6
                    && filename[..2].chars().all(|c| c.is_ascii_digit()));

            if is_vault {
                // MK3: Exact structural verification of vault slots per SPEC F.3 & D-111
                if parse_vault_has_active_slot(&content, target_tag)? {
                    return Err(InvalidCode::Mk3KeyPresent);
                }
            }

            // MK1 & MK2 scan: Check for raw residue of target tag
            if content
                .windows(target_tag.len())
                .any(|w| w == target_tag.as_slice())
            {
                return Err(InvalidCode::Mk2RawResidue);
            }
        }
    }

    Ok(())
}

/// Parses vault file according to SPEC F.3 layout and inspects each 64-byte slot.
fn parse_vault_has_active_slot(
    vault_data: &[u8],
    target_tag: &[u8; 16],
) -> Result<bool, InvalidCode> {
    const VAULT_HEADER_SIZE: usize = 64;
    const VAULT_METABLOCK_SIZE: usize = 64;
    const VAULT_BASE_OFFSET: usize = VAULT_HEADER_SIZE + VAULT_METABLOCK_SIZE;
    const VAULT_SLOT_SIZE: usize = 64;

    if vault_data.len() < VAULT_BASE_OFFSET {
        return Ok(false);
    }

    // Inspect slots
    let slots_data = match vault_data.get(VAULT_BASE_OFFSET..) {
        Some(s) => s,
        None => return Ok(false),
    };

    for chunk in slots_data.chunks_exact(VAULT_SLOT_SIZE) {
        let slot_type = chunk.first().copied().unwrap_or(0);
        let status = chunk.get(1).copied().unwrap_or(0);
        let tag = match chunk.get(8..24) {
            Some(t) => t,
            None => continue,
        };

        // If slot status is active (0x01) and tag matches target_tag, target key is still present (MK3 failure)
        if status == 0x01 && tag == target_tag.as_slice() {
            return Ok(true);
        }

        // If slot status is tombstone (0x02) for target_tag:
        // In SPEC F.3: Tombstone zeroes all bytes except status (byte 1 = 0x02).
        // That means slot_type (byte 0) must be 0, gen (2..8) and payload (24..56) are zero.
        if status == 0x02 && tag == target_tag.as_slice() {
            if slot_type != 0x00 {
                return Err(InvalidCode::Mk1StructureResidue);
            }
            if let Some(payload) = chunk.get(24..56) {
                if payload.iter().any(|&b| b != 0) {
                    return Err(InvalidCode::Mk1StructureResidue);
                }
            }
        }
    }

    Ok(false)
}
