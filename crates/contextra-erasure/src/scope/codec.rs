//! Serialisierung und Deserialisierung von Scope-Dateien (SPEC §11.1, Anhang F.1, ER-17).
//!
//! Scope-Dateien haben kind = 9, zufälligen Dateischlüssel (F-Slot im Tresor).
//! Kleine Umfänge werden direkt in den Journal-Eintrag eingebettet.
//!
//! Binäres Format (V2 gehärtet):
//! - Magic: "CTXSCOM\0" (8 Bytes)
//! - version: u16 Big-Endian (aktuell: 1 oder 2)
//! - flags: u16 Big-Endian (Reserviert, aktuell 0)
//! - scope_count: u32 Big-Endian
//! - BLAKE3-Prüfsumme der Scopes: [u8; 32]
//! - Je ResolvedLayerScope:
//!   - layer_id: [u8; 16]
//!   - unit_count: u64 (Big-Endian)
//!   - units: [u64; unit_count] (Big-Endian)
//!
//! Abwärtskompatibilität: V1-Format ohne Prüfsumme (12 Byte Kopf) wird beim Dekodieren
//! weiterhin unterstützt, wenn die Bytes exakt validiert werden können.

// Herkunft: 4b98cf91b:crates/contextra-store/src/lsm/purge/scope_file.rs:43

use super::set::{LayerScopeSet, ResolvedLayerScope};
use crate::error::ErasureError;

const SCOPE_MAGIC: &[u8; 8] = b"CTXSCOM\0";
const SCOPE_VERSION_V2: u16 = 2;

/// Serialisiert einen `LayerScopeSet` in ein festes Binärformat mit Versionsfeld und Prüfsumme (Big-Endian).
pub fn encode_scope(scope: &LayerScopeSet) -> Result<Vec<u8>, ErasureError> {
    let mut payload = Vec::new();

    let count = u32::try_from(scope.scopes.len())
        .map_err(|_| ErasureError::LimitExceeded("Too many layer scopes".to_string()))?;

    for s in &scope.scopes {
        payload.extend_from_slice(&s.layer_id);
        payload.extend_from_slice(&s.unit_count.to_be_bytes());
        for &u in &s.units {
            payload.extend_from_slice(&u.to_be_bytes());
        }
    }

    // Berechne Prüfsumme über Nutzdaten
    let checksum = blake3::hash(&payload);

    let mut out = Vec::with_capacity(8 + 2 + 2 + 4 + 32 + payload.len());
    out.extend_from_slice(SCOPE_MAGIC);
    out.extend_from_slice(&SCOPE_VERSION_V2.to_be_bytes());
    out.extend_from_slice(&0u16.to_be_bytes()); // flags
    out.extend_from_slice(&count.to_be_bytes());
    out.extend_from_slice(checksum.as_bytes());
    out.extend_from_slice(&payload);

    Ok(out)
}

/// Deserialisiert einen `LayerScopeSet` aus dem festen Binärformat.
///
/// Fail-Closed: Bei falschem Magic, unbekannter Version, Trunkierung,
/// ungültigen Zählern oder Prüfsummen-Diskrepanz schlägt die Deserialisierung
/// zwingend mit `ErasureError::Corrupt` fehl und gibt niemals still leere Daten zurück.
pub fn decode_scope(data: &[u8]) -> Result<LayerScopeSet, ErasureError> {
    if data.len() < 12 {
        return Err(ErasureError::Corrupt);
    }

    let magic = data.get(0..8).ok_or(ErasureError::Corrupt)?;
    if magic != SCOPE_MAGIC {
        return Err(ErasureError::Corrupt);
    }

    // Prüfe V2 Format (Header: Magic(8) + Version(2) + Flags(2) + Count(4) + Checksum(32) = 48 Bytes)
    let version_bytes = data.get(8..10).ok_or(ErasureError::Corrupt)?;
    let mut version_arr = [0u8; 2];
    version_arr.copy_from_slice(version_bytes);
    let version = u16::from_be_bytes(version_arr);

    if version == SCOPE_VERSION_V2 {
        if data.len() < 48 {
            return Err(ErasureError::Corrupt);
        }

        let count_bytes = data.get(12..16).ok_or(ErasureError::Corrupt)?;
        let mut count_arr = [0u8; 4];
        count_arr.copy_from_slice(count_bytes);
        let count =
            usize::try_from(u32::from_be_bytes(count_arr)).map_err(|_| ErasureError::Corrupt)?;

        let expected_checksum = data.get(16..48).ok_or(ErasureError::Corrupt)?;
        let payload = data.get(48..).ok_or(ErasureError::Corrupt)?;

        // Prüfe BLAKE3 Prüfsumme
        let computed_checksum = blake3::hash(payload);
        if computed_checksum.as_bytes() != expected_checksum {
            return Err(ErasureError::Corrupt);
        }

        return parse_scopes_payload(payload, count);
    }

    // Abwärtskompatibles V1-Format (Magic(8) + Count(4) = 12 Bytes)
    let count_bytes = data.get(8..12).ok_or(ErasureError::Corrupt)?;
    let mut count_arr = [0u8; 4];
    count_arr.copy_from_slice(count_bytes);
    let count =
        usize::try_from(u32::from_be_bytes(count_arr)).map_err(|_| ErasureError::Corrupt)?;

    let payload = data.get(12..).ok_or(ErasureError::Corrupt)?;
    parse_scopes_payload(payload, count)
}

fn parse_scopes_payload(data: &[u8], count: usize) -> Result<LayerScopeSet, ErasureError> {
    let mut cursor = data;
    let mut scopes = Vec::with_capacity(count);

    for _ in 0..count {
        if cursor.len() < 16 + 8 {
            return Err(ErasureError::Corrupt);
        }
        let layer_bytes = cursor.get(0..16).ok_or(ErasureError::Corrupt)?;
        let mut layer_id = [0u8; 16];
        layer_id.copy_from_slice(layer_bytes);

        let unit_count_bytes = cursor.get(16..24).ok_or(ErasureError::Corrupt)?;
        let mut unit_count_arr = [0u8; 8];
        unit_count_arr.copy_from_slice(unit_count_bytes);
        let unit_count = u64::from_be_bytes(unit_count_arr);

        let unit_count_usize = usize::try_from(unit_count).map_err(|_| ErasureError::Corrupt)?;
        let unit_bytes_len = unit_count_usize
            .checked_mul(8)
            .ok_or(ErasureError::Corrupt)?;

        let rest = cursor.get(24..).ok_or(ErasureError::Corrupt)?;
        if rest.len() < unit_bytes_len {
            return Err(ErasureError::Corrupt);
        }

        let units_bytes = rest.get(0..unit_bytes_len).ok_or(ErasureError::Corrupt)?;
        let mut units = Vec::with_capacity(unit_count_usize);

        for chunk in units_bytes.chunks_exact(8) {
            let mut u_arr = [0u8; 8];
            u_arr.copy_from_slice(chunk);
            units.push(u64::from_be_bytes(u_arr));
        }

        scopes.push(ResolvedLayerScope {
            layer_id,
            unit_count,
            units,
        });

        cursor = rest.get(unit_bytes_len..).ok_or(ErasureError::Corrupt)?;
    }

    // Wenn nach dem Parsen aller Scopes noch unerwartete Restbytes vorhanden sind -> Korruption
    if !cursor.is_empty() {
        return Err(ErasureError::Corrupt);
    }

    Ok(LayerScopeSet { scopes })
}
