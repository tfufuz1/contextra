//! Serialisierung und Deserialisierung von Scope-Dateien (SPEC §11.1, Anhang F.1, ER-17).
//!
//! Scope-Dateien haben kind = 9, zufälligen Dateischlüssel (F-Slot im Tresor).
//! Kleine Umfänge werden direkt in den Journal-Eintrag eingebettet.
//!
//! Binäres Format:
//! - Magic: "CTXSCOM\0" (8 Bytes)
//! - scope_count: u32 (Big-Endian)
//! - Je ResolvedLayerScope:
//!   - layer_id: [u8; 16]
//!   - unit_count: u64 (Big-Endian)
//!   - units: [u64; unit_count] (Big-Endian)

use super::set::{LayerScopeSet, ResolvedLayerScope};
use crate::error::ErasureError;

const SCOPE_MAGIC: &[u8; 8] = b"CTXSCOM\0";

/// Serialisiert einen `LayerScopeSet` in ein festes Binärformat (Big-Endian).
pub fn encode_scope(scope: &LayerScopeSet) -> Result<Vec<u8>, ErasureError> {
    let mut out = Vec::new();
    out.extend_from_slice(SCOPE_MAGIC);

    let count = u32::try_from(scope.scopes.len())
        .map_err(|_| ErasureError::LimitExceeded("Too many layer scopes".to_string()))?;
    out.extend_from_slice(&count.to_be_bytes());

    for s in &scope.scopes {
        out.extend_from_slice(&s.layer_id);
        out.extend_from_slice(&s.unit_count.to_be_bytes());
        for &u in &s.units {
            out.extend_from_slice(&u.to_be_bytes());
        }
    }

    Ok(out)
}

/// Deserialisiert einen `LayerScopeSet` aus dem festen Binärformat.
pub fn decode_scope(data: &[u8]) -> Result<LayerScopeSet, ErasureError> {
    if data.len() < 12 {
        return Err(ErasureError::Corrupt);
    }

    let magic = data.get(0..8).ok_or(ErasureError::Corrupt)?;
    if magic != SCOPE_MAGIC {
        return Err(ErasureError::Corrupt);
    }

    let count_bytes = data.get(8..12).ok_or(ErasureError::Corrupt)?;
    let mut count_arr = [0u8; 4];
    count_arr.copy_from_slice(count_bytes);
    let count =
        usize::try_from(u32::from_be_bytes(count_arr)).map_err(|_| ErasureError::Corrupt)?;

    let mut cursor = data.get(12..).ok_or(ErasureError::Corrupt)?;
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

    Ok(LayerScopeSet { scopes })
}
