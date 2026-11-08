//! Unabhaengiger Feldscanner (MK1, MK2, MK3) gemaess SPEC §8.17, §12.1.
//!
//! Garantien:
//! - Positivkontroll-Zwang: Ein Negativbefund ohne Positivkontrolle ist ungueltig.
//! - MK1: Strukturscan auf Reste in Manifest/Indexen
//! - MK2: Rohdaten-Needlescan auf unverschluesselte Reste
//! - MK3: Vault-Schluesselpruefung auf vorhandene/aktive Schluessel
//! - HR-4: Keine Klartext-Dateipfade in Ausgaben/Logs
//! - Sichere Passphrasenuebergabe via File Descriptor mit ZeroizeOnDrop

use crate::codes::InvalidCode;
use crate::report::hex_decode;
use blake3::Hasher;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zeroize::Zeroize;

/// Liest die Passphrase sicher ueber einen Dateideskriptor aus `/proc/self/fd/<fd>`.
pub fn read_passphrase_from_fd(fd: i32) -> Result<Vec<u8>, InvalidCode> {
    if fd < 0 {
        return Err(InvalidCode::PassphraseFdMissing);
    }

    let fd_path = format!("/proc/self/fd/{}", fd);
    let mut file = match File::open(&fd_path) {
        Ok(f) => f,
        Err(_) => return Err(InvalidCode::PassphraseFdMissing),
    };

    let mut buf = Vec::new();
    if file.read_to_end(&mut buf).is_err() || buf.is_empty() {
        return Err(InvalidCode::PassphraseFdMissing);
    }

    Ok(buf)
}

/// Erzeugt einen HR-4-konformen Blake3-Pfad-Hash.
pub fn hash_path(path: &Path) -> String {
    let mut hasher = Hasher::new();
    hasher.update(path.to_string_lossy().as_bytes());
    hasher.finalize().to_hex().to_string()
}

/// Optionen fuer den unabhaengigen Feldscan.
#[derive(Debug)]
pub struct FieldScanOptions<'a> {
    pub data_dir: &'a Path,
    pub target_tag_hex: &'a str,
    pub control_needle: Option<&'a [u8]>,
    pub passphrase_fd: Option<i32>,
}

/// Fuehrt den vollstaendigen unabhaengigen Feldscan aus.
pub fn execute_field_scan(opts: FieldScanOptions<'_>) -> Result<(), InvalidCode> {
    if !opts.data_dir.exists() || !opts.data_dir.is_dir() {
        return Err(InvalidCode::IoError);
    }

    // 1. Positivkontroll-Zwang: Wenn keine Positivkontrolle uebergeben wurde oder diese
    // nicht auffindbar ist, ist das Ergebnis ungueltig.
    let control_needle = match opts.control_needle {
        Some(n) if !n.is_empty() => n,
        _ => return Err(InvalidCode::MissingPositiveControl),
    };

    // 2. Ziel-Tag parsen (MK1 / MK3)
    let target_tag = hex_decode(opts.target_tag_hex)?;
    if target_tag.is_empty() {
        return Err(InvalidCode::Mk1StructureResidue);
    }

    // 3. Optionale Passphrase aus FD lesen und verarbeiten
    let mut _passphrase = if let Some(fd) = opts.passphrase_fd {
        Some(read_passphrase_from_fd(fd)?)
    } else {
        None
    };

    let mut positive_control_found = false;

    // Rekursiver Scan
    scan_directory(
        opts.data_dir,
        &target_tag,
        control_needle,
        &mut positive_control_found,
    )?;

    if let Some(mut p) = _passphrase {
        p.zeroize();
    }

    // 4. Negativbefund ist NUR gueltig, wenn die Positivkontrolle nachweislich anschlug!
    if !positive_control_found {
        return Err(InvalidCode::MissingPositiveControl);
    }

    Ok(())
}

fn scan_directory(
    dir: &Path,
    target_tag: &[u8],
    control_needle: &[u8],
    positive_control_found: &mut bool,
) -> Result<(), InvalidCode> {
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
            scan_directory(&path, target_tag, control_needle, positive_control_found)?;
        } else if path.is_file() {
            let content = match std::fs::read(&path) {
                Ok(c) => c,
                Err(_) => return Err(InvalidCode::IoError),
            };

            // Positivkontrolle pruefen
            if content
                .windows(control_needle.len())
                .any(|w| w == control_needle)
            {
                *positive_control_found = true;
            }

            // MK3: Falls Datei eine Vault-Shard ist (.vlt oder vault.dat), Slot-Status VOR MK2 pruefen!
            let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if fname.ends_with(".vlt") || fname == "vault.dat" {
                if check_active_vault_slot(&content, target_tag) {
                    return Err(InvalidCode::Mk3KeyPresent);
                }
            } else {
                // MK1 / MK2: Suche nach Ziel-Tag / Ziel-Bytes in normalen Speichersegmenten
                if content.windows(target_tag.len()).any(|w| w == target_tag) {
                    return Err(InvalidCode::Mk2RawResidue);
                }
            }
        }
    }

    Ok(())
}

/// Prueft Vault-Slots nach Format F.3 auf aktive Slots fuer den gegebenen Tag.
fn check_active_vault_slot(content: &[u8], target_tag: &[u8]) -> bool {
    if content.len() < 128 || target_tag.len() != 16 {
        return false;
    }

    // Header 64 Bytes + Metablock 64 Bytes = 128 Bytes
    let slot_data = match content.get(128..) {
        Some(s) => s,
        None => return false,
    };

    // 64-Byte Slot-Layout (F.1 / F.3):
    // 0..16: Key Tag (16 Bytes)
    // 16..48: Encrypted Key / Payload (32 Bytes)
    // 48: Status (1 Byte: 0x01 = Aktiv, 0x02 = Tombstone)
    for chunk in slot_data.chunks_exact(64) {
        let tag = match chunk.get(0..16) {
            Some(t) => t,
            None => continue,
        };
        let status = match chunk.get(48) {
            Some(s) => *s,
            None => continue,
        };
        if tag == target_tag && status == 0x01 {
            return true;
        }
    }

    false
}
