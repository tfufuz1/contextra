//! # Zeiger CURRENT auf die aktive Katalog-Datei
//!
//! SPEC §8.1 und §9.7: CURRENT enthält catalog_no als Text und crc32c.
//! Die Aktualisierung erfolgt atomar per write_atomic.

use crate::catalog::durable::{write_atomic_synced, Synced};
use crate::catalog::types::CatalogError;
use crate::codec::crc32c::crc32c;
use contextra_storage_ports::{Fs, OpenMode};
use std::io;
use std::path::Path;

/// Name der Zeigerdatei.
pub const CURRENT_FILE_NAME: &str = "CURRENT";

/// Schreibt die CURRENT-Zeigerdatei atomar per `write_atomic` (SPEC §9.7).
pub fn write_current(fs: &dyn Fs, root: &Path, catalog_no: u64) -> io::Result<Synced<u64>> {
    let current_path = root.join(CURRENT_FILE_NAME);
    let no_str = catalog_no.to_string();
    let chk = crc32c(0, no_str.as_bytes());

    let payload = format!("{no_str} {chk:08x}\n");
    write_atomic_synced(fs, &current_path, payload.as_bytes(), catalog_no)
}

/// Liest die CURRENT-Zeigerdatei und verifiziert die Prüfsumme.
pub fn read_current(fs: &dyn Fs, root: &Path) -> Result<u64, CatalogError> {
    let current_path = root.join(CURRENT_FILE_NAME);
    let file = fs
        .open(&current_path, OpenMode::Read)
        .map_err(|e| CatalogError::Io(format!("CURRENT konnte nicht geöffnet werden: {e}")))?;

    let len = file.len().map_err(CatalogError::from)?;
    let len_usize = usize::try_from(len).map_err(|_| CatalogError::InvalidFormat("Längenüberlauf"))?;
    if len_usize > 1024 {
        return Err(CatalogError::Corrupt("CURRENT-Datei ungewöhnlich groß"));
    }

    let mut buf = vec![0u8; len_usize];
    file.read_at(&mut buf, 0).map_err(CatalogError::from)?;

    let text = std::str::from_utf8(&buf)
        .map_err(|_| CatalogError::Corrupt("CURRENT ist kein gültiges UTF-8"))?;

    let mut parts = text.split_whitespace();
    let no_part = parts
        .next()
        .ok_or(CatalogError::Corrupt("Fehlende catalog_no in CURRENT"))?;
    let chk_part = parts
        .next()
        .ok_or(CatalogError::Corrupt("Fehlende Prüfsumme in CURRENT"))?;

    let catalog_no: u64 = no_part
        .parse()
        .map_err(|_| CatalogError::Corrupt("Ungültige Zahl in CURRENT"))?;

    let expected_chk = u32::from_str_radix(chk_part, 16)
        .map_err(|_| CatalogError::Corrupt("Ungültige Hex-Prüfsumme in CURRENT"))?;

    let actual_chk = crc32c(0, no_part.as_bytes());
    if actual_chk != expected_chk {
        return Err(CatalogError::Corrupt("Prüfsummenfehler in CURRENT-Datei"));
    }

    Ok(catalog_no)
}
