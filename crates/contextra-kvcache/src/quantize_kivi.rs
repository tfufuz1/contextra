//! # KIVI 2-Bit Quantisierung und Kompression
//!
//! ANCHOR[contextra-kvcache:KIVI-001]
//!
//! Dieses Modul implementiert asymmetrische 2-Bit-Quantisierung nach dem KIVI-Verfahren
//! (arXiv:2402.02750: *KIVI: A Tuning-Free Asymmetric 2-bit Quantization for KV Cache*).
//!
//! ## Was ist KIVI?
//! Bei langen Text-Kontexten wächst der KV-Cache im RAM massiv an. Übliche FP16/BF16-Tensoren
//! belegen 16 Bit pro Skalarwert. KIVI reduziert diesen Speicherbedarf drastisch auf **2 Bit**:
//! - **Keys (Schlüssel)**: Werden *per-channel* (kanalweise) in Gruppen (`key_group_size`, z. B. 16) quantisiert.
//! - **Values (Werte)**: Werden *per-token* quantisiert.
//!
//! ## INV-KIVI-AEAD-ORDER
//! Für das Tiering auf Sekundärspeicher gilt zwingend folgende Reihenfolge:
//! 1. **Quantisierung** (Datenvolumen um ~75% reduzieren).
//! 2. **Kompression** (Run-Length-Encoding / Packen).
//! 3. **AEAD-Verschlüsselung** (Authentifizierte Verschlüsselung mit Nonce).
//!
//! Beim Lesen gilt exakt die umgekehrte Reihenfolge:
//! 1. **AEAD-Entschlüsselung**
//! 2. **Dekomprimierung**
//! 3. **Dequantisierung**

#![forbid(unsafe_code)]

use contextra_types::{ContextraError, Result};
use serde::{Deserialize, Serialize};

// ANCHOR[contextra-kvcache:KIVI-002]
/// Konfiguration für die KIVI-Quantisierungsparameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KiviQuantizeConfig {
    /// Gruppengröße für die per-channel Key-Quantisierung (Standard: 16).
    pub key_group_size: usize,
    /// Gibt an, ob Values ebenfalls quantisiert werden sollen (Standard: true).
    pub quantize_values: bool,
}

impl Default for KiviQuantizeConfig {
    fn default() -> Self {
        Self {
            key_group_size: 16,
            quantize_values: true,
        }
    }
}

// ANCHOR[contextra-kvcache:KIVI-003]
/// Sicht (View) auf unquantisierte Roh-Tensoren für Keys und Values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KvTensorView {
    /// Key-Tensor als Fließkommazahlen in `[num_tokens, num_channels]` Row-Major-Ordnung.
    pub keys: Vec<f32>,
    /// Value-Tensor als Fließkommazahlen in `[num_tokens, num_channels]` Row-Major-Ordnung.
    pub values: Vec<f32>,
    /// Anzahl encodierter Tokens.
    pub num_tokens: usize,
    /// Anzahl Kanäle pro Head/Token.
    pub num_channels: usize,
}

impl KvTensorView {
    /// Erstellt einen neuen `KvTensorView` und prüft die Dimensionen gegen die Elementanzahl.
    ///
    /// # Fehler
    /// Liefert [`ContextraError::InvalidInput`], falls `keys.len()` oder `values.len()` nicht `num_tokens * num_channels` entsprechen.
    pub fn new(
        keys: Vec<f32>,
        values: Vec<f32>,
        num_tokens: usize,
        num_channels: usize,
    ) -> Result<Self> {
        let expected = num_tokens * num_channels;
        if keys.len() != expected {
            return Err(ContextraError::InvalidInput(format!(
                "Keys length {} does not match num_tokens ({}) * num_channels ({}) = {}",
                keys.len(),
                num_tokens,
                num_channels,
                expected
            )));
        }
        if values.len() != expected {
            return Err(ContextraError::InvalidInput(format!(
                "Values length {} does not match num_tokens ({}) * num_channels ({}) = {}",
                values.len(),
                num_tokens,
                num_channels,
                expected
            )));
        }
        Ok(Self {
            keys,
            values,
            num_tokens,
            num_channels,
        })
    }

    /// Serialisiert die Tensor-Sicht in Bytes via Bincode.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| ContextraError::Serialization(e.to_string()))
    }

    /// Deserialisiert eine Tensor-Sicht aus Bytes via Bincode.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| ContextraError::Serialization(e.to_string()))
    }
}

// ANCHOR[contextra-kvcache:KIVI-004]
/// Metadaten zur Dequantisierung eines 2-Bit KIVI-Blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KiviBlockMeta {
    /// Anzahl Bits pro Skalarwert (für KIVI strikt 2).
    pub bits_per_value: u8,
    /// Skalierungsfaktor (Scale) für lineare Min/Max-Quantisierung.
    pub scale: f32,
    /// Nullpunktverschiebung (Zero Point) für asymmetrische Quantisierung.
    pub zero_point: f32,
    /// Ursprüngliche Elementanzahl der Fließkommawerte im Block.
    pub original_len: usize,

    /// Anzahl Tokens.
    #[serde(default)]
    pub num_tokens: usize,
    /// Anzahl Kanäle.
    #[serde(default)]
    pub num_channels: usize,
    /// Key-Quantisierungs-Gruppengröße.
    #[serde(default)]
    pub key_group_size: usize,
    /// Gibt an, ob Values quantisiert wurden.
    #[serde(default)]
    pub quantize_values: bool,
    /// Skalierungsfaktoren pro Kanalgruppe für Keys.
    #[serde(default)]
    pub key_scales: Vec<f32>,
    /// Nullpunkte pro Kanalgruppe für Keys.
    #[serde(default)]
    pub key_zero_points: Vec<f32>,
    /// Skalierungsfaktoren pro Token für Values.
    #[serde(default)]
    pub value_scales: Vec<f32>,
    /// Nullpunkte pro Token für Values.
    #[serde(default)]
    pub value_zero_points: Vec<f32>,
}

// ANCHOR[contextra-kvcache:KIVI-005]
/// 2-Bit-quantisierter Byte-Block mit zugehörigen Dequantisierungs-Metadaten.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KiviQuantizedBlock {
    /// Dequantisierungs-Metadaten.
    pub meta: KiviBlockMeta,
    /// Gepackte 2-Bit-Skalarwerte (4 Werte pro Byte gepackt, LSB-first).
    pub packed: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct KiviPackedPayload {
    packed_keys: Vec<u8>,
    packed_values: Vec<u8>,
}

// ANCHOR[contextra-kvcache:KIVI-006]
/// Quantisiert einen unkomprimierten [`KvTensorView`] in einen 2-Bit [`KiviQuantizedBlock`].
///
/// # Ablauf
/// 1. Dimensionen prüfen und Gruppengröße festlegen.
/// 2. **Keys quantisieren**: Für jede Kanalgruppe Min- und Max-Werte ermitteln,
///    Scale und Zero-Point berechnen und die Werte auf 2 Bit (0..=3) packen (4 Werte pro Byte).
/// 3. **Values quantisieren**: Pro Token Min/Max berechnen und in 2 Bit packen
///    (bzw. unquantisiert serialisieren, falls `quantize_values == false`).
/// 4. Metadaten und gepackte Byte-Payload in [`KiviQuantizedBlock`] assemblieren.
pub fn kivi_quantize(raw: &KvTensorView, config: KiviQuantizeConfig) -> Result<KiviQuantizedBlock> {
    let num_tokens = raw.num_tokens;
    let num_channels = raw.num_channels;
    let group_size = if config.key_group_size == 0 {
        16
    } else {
        config.key_group_size
    };

    if num_tokens == 0 || num_channels == 0 {
        let meta = KiviBlockMeta {
            bits_per_value: 2,
            scale: 0.0,
            zero_point: 0.0,
            original_len: 0,
            num_tokens,
            num_channels,
            key_group_size: group_size,
            quantize_values: config.quantize_values,
            key_scales: Vec::new(),
            key_zero_points: Vec::new(),
            value_scales: Vec::new(),
            value_zero_points: Vec::new(),
        };
        let payload = KiviPackedPayload {
            packed_keys: Vec::new(),
            packed_values: Vec::new(),
        };
        let packed_bytes = bincode::serialize(&payload)
            .map_err(|e| ContextraError::Serialization(e.to_string()))?;
        return Ok(KiviQuantizedBlock {
            meta,
            packed: packed_bytes,
        });
    }

    let num_groups = num_tokens.div_ceil(group_size);
    let key_entries = num_groups * num_channels;
    let mut key_scales = Vec::with_capacity(key_entries);
    let mut key_zero_points = Vec::with_capacity(key_entries);

    let key_packed_len = (num_tokens * num_channels).div_ceil(4);
    let mut packed_keys = vec![0u8; key_packed_len];

    // Compute key group scales and zero points
    for g in 0..num_groups {
        let t_start = g * group_size;
        let t_end = (t_start + group_size).min(num_tokens);

        for c in 0..num_channels {
            let mut min_v = f32::INFINITY;
            let mut max_v = f32::NEG_INFINITY;

            for t in t_start..t_end {
                let idx = t * num_channels + c;
                let v = raw.keys[idx];
                if v.is_finite() {
                    min_v = min_v.min(v);
                    max_v = max_v.max(v);
                }
            }

            if !min_v.is_finite() {
                min_v = 0.0;
                max_v = 0.0;
            }

            let (sc, zp) = if (max_v - min_v).abs() < f32::EPSILON {
                (0.0, min_v)
            } else {
                ((max_v - min_v) / 3.0, min_v)
            };

            key_scales.push(sc);
            key_zero_points.push(zp);
        }
    }

    // Quantize keys into row-major 2-bit packed array
    for t in 0..num_tokens {
        let g = t / group_size;
        for c in 0..num_channels {
            let orig_idx = t * num_channels + c;
            let v = raw.keys[orig_idx];
            let meta_idx = g * num_channels + c;
            let sc = key_scales[meta_idx];
            let zp = key_zero_points[meta_idx];

            let code = if sc > 0.0 && v.is_finite() {
                let norm = ((v - zp) / sc).round();
                norm.clamp(0.0, 3.0) as u8
            } else {
                0u8
            };

            let byte_idx = orig_idx / 4;
            let bit_shift = (orig_idx % 4) * 2;
            if let Some(byte) = packed_keys.get_mut(byte_idx) {
                *byte |= (code & 0x03) << bit_shift;
            }
        }
    }

    // Quantize Values per-token
    let mut value_scales = Vec::new();
    let mut value_zero_points = Vec::new();
    let packed_values = if config.quantize_values {
        value_scales.reserve(num_tokens);
        value_zero_points.reserve(num_tokens);

        let val_packed_len = (num_tokens * num_channels).div_ceil(4);
        let mut p_vals = vec![0u8; val_packed_len];

        for t in 0..num_tokens {
            let mut min_v = f32::INFINITY;
            let mut max_v = f32::NEG_INFINITY;

            for c in 0..num_channels {
                let idx = t * num_channels + c;
                let v = raw.values[idx];
                if v.is_finite() {
                    min_v = min_v.min(v);
                    max_v = max_v.max(v);
                }
            }

            if !min_v.is_finite() {
                min_v = 0.0;
                max_v = 0.0;
            }

            let (sc, zp) = if (max_v - min_v).abs() < f32::EPSILON {
                (0.0, min_v)
            } else {
                ((max_v - min_v) / 3.0, min_v)
            };

            value_scales.push(sc);
            value_zero_points.push(zp);

            for c in 0..num_channels {
                let elem_idx = t * num_channels + c;
                let v = raw.values[elem_idx];

                let code = if sc > 0.0 && v.is_finite() {
                    let norm = ((v - zp) / sc).round();
                    norm.clamp(0.0, 3.0) as u8
                } else {
                    0u8
                };

                let byte_idx = elem_idx / 4;
                let bit_shift = (elem_idx % 4) * 2;
                if let Some(byte) = p_vals.get_mut(byte_idx) {
                    *byte |= (code & 0x03) << bit_shift;
                }
            }
        }
        p_vals
    } else {
        bincode::serialize(&raw.values).map_err(|e| ContextraError::Serialization(e.to_string()))?
    };

    let meta = KiviBlockMeta {
        bits_per_value: 2,
        scale: key_scales.first().copied().unwrap_or(0.0),
        zero_point: key_zero_points.first().copied().unwrap_or(0.0),
        original_len: num_tokens * num_channels * 2,
        num_tokens,
        num_channels,
        key_group_size: group_size,
        quantize_values: config.quantize_values,
        key_scales,
        key_zero_points,
        value_scales,
        value_zero_points,
    };

    let payload = KiviPackedPayload {
        packed_keys,
        packed_values,
    };
    let packed =
        bincode::serialize(&payload).map_err(|e| ContextraError::Serialization(e.to_string()))?;

    Ok(KiviQuantizedBlock { meta, packed })
}

// ANCHOR[contextra-kvcache:KIVI-007]
/// Dequantisiert gepackte 2-Bit-Bytes zurück in einen [`KvTensorView`].
///
/// # Ablauf
/// 1. Metadaten validieren (`bits_per_value == 2`, Skalen und Nullpunkte müssen endlich sein).
/// 2. Payload deserialisieren.
/// 3. 2-Bit-Codes entpacken und mit `v = zp + code * sc` in f32 zurückrechnen.
pub fn kivi_dequantize(bytes: &[u8], meta: &KiviBlockMeta) -> Result<KvTensorView> {
    if meta.bits_per_value != 2 {
        return Err(ContextraError::kv_quantization(format!(
            "KIVI dequantization requires bits_per_value == 2, got {}",
            meta.bits_per_value
        )));
    }

    let (active_meta, packed_bytes) = match bincode::deserialize::<KiviQuantizedBlock>(bytes) {
        Ok(block) => (block.meta, block.packed),
        Err(_) => (meta.clone(), bytes.to_vec()),
    };

    if !active_meta.scale.is_finite() || !active_meta.zero_point.is_finite() {
        return Err(ContextraError::kv_quantization(
            "Non-finite key scale or zero_point encountered in metadata".to_string(),
        ));
    }

    let num_tokens = active_meta.num_tokens;
    let num_channels = active_meta.num_channels;

    if num_tokens == 0 || num_channels == 0 {
        return KvTensorView::new(Vec::new(), Vec::new(), num_tokens, num_channels);
    }

    let payload: KiviPackedPayload = match bincode::deserialize(&packed_bytes) {
        Ok(p) => p,
        Err(e) => {
            return Err(ContextraError::kv_quantization(format!(
                "Failed to deserialize KIVI payload bytes: {}",
                e
            )));
        }
    };

    let group_size = if active_meta.key_group_size == 0 {
        16
    } else {
        active_meta.key_group_size
    };

    for sc in &active_meta.key_scales {
        if !sc.is_finite() {
            return Err(ContextraError::kv_quantization(
                "Non-finite key scale encountered in metadata".to_string(),
            ));
        }
    }
    for zp in &active_meta.key_zero_points {
        if !zp.is_finite() {
            return Err(ContextraError::kv_quantization(
                "Non-finite key zero_point encountered in metadata".to_string(),
            ));
        }
    }

    let total_elements = num_tokens * num_channels;
    let mut keys = vec![0.0f32; total_elements];

    for t in 0..num_tokens {
        let g = t / group_size;
        for c in 0..num_channels {
            let orig_idx = t * num_channels + c;
            let meta_idx = g * num_channels + c;
            let sc = active_meta
                .key_scales
                .get(meta_idx)
                .copied()
                .unwrap_or(active_meta.scale);
            let zp = active_meta
                .key_zero_points
                .get(meta_idx)
                .copied()
                .unwrap_or(active_meta.zero_point);

            let byte_idx = orig_idx / 4;
            let bit_shift = (orig_idx % 4) * 2;

            let byte = match payload.packed_keys.get(byte_idx) {
                Some(&b) => b,
                None => {
                    return Err(ContextraError::kv_quantization(format!(
                        "Packed key byte index {} out of bounds (len {})",
                        byte_idx,
                        payload.packed_keys.len()
                    )));
                }
            };

            let code = (byte >> bit_shift) & 0x03;
            keys[orig_idx] = zp + (code as f32) * sc;
        }
    }

    let values = if active_meta.quantize_values {
        let mut vals = vec![0.0f32; total_elements];
        for t in 0..num_tokens {
            let sc = active_meta
                .value_scales
                .get(t)
                .copied()
                .unwrap_or(active_meta.scale);
            let zp = active_meta
                .value_zero_points
                .get(t)
                .copied()
                .unwrap_or(active_meta.zero_point);

            for c in 0..num_channels {
                let elem_idx = t * num_channels + c;
                let byte_idx = elem_idx / 4;
                let bit_shift = (elem_idx % 4) * 2;

                let byte = match payload.packed_values.get(byte_idx) {
                    Some(&b) => b,
                    None => {
                        return Err(ContextraError::kv_quantization(format!(
                            "Packed value byte index {} out of bounds (len {})",
                            byte_idx,
                            payload.packed_values.len()
                        )));
                    }
                };

                let code = (byte >> bit_shift) & 0x03;
                let v = zp + (code as f32) * sc;
                vals[elem_idx] = v;
            }
        }
        vals
    } else {
        match bincode::deserialize::<Vec<f32>>(&payload.packed_values) {
            Ok(v) => v,
            Err(e) => {
                return Err(ContextraError::kv_quantization(format!(
                    "Failed to deserialize raw value floats: {}",
                    e
                )));
            }
        }
    };

    KvTensorView::new(keys, values, num_tokens, num_channels)
}

// ANCHOR[contextra-kvcache:KIVI-008]
/// Entpackt einen einzelnen linearen 2-Bit-Block zurück in ein `Vec<f32>`.
pub fn unpack_kivi_block(block: &KiviQuantizedBlock) -> Result<Vec<f32>> {
    if block.meta.bits_per_value != 2 {
        return Err(ContextraError::InvalidInput(format!(
            "KIVI dequantization requires bits_per_value == 2, got {}",
            block.meta.bits_per_value
        )));
    }

    let expected_packed_len = block.meta.original_len.div_ceil(4);
    if block.packed.len() != expected_packed_len {
        return Err(ContextraError::InvalidInput(format!(
            "Packed byte length mismatch: expected {} bytes for {} original elements, got {}",
            expected_packed_len,
            block.meta.original_len,
            block.packed.len()
        )));
    }

    if !block.meta.scale.is_finite() || !block.meta.zero_point.is_finite() {
        return Err(ContextraError::InvalidInput(
            "KIVI metadata scale and zero_point must be finite floats".to_string(),
        ));
    }

    let mut result = Vec::with_capacity(block.meta.original_len);
    for i in 0..block.meta.original_len {
        let byte_idx = i / 4;
        let bit_shift = (i % 4) * 2;
        let byte = match block.packed.get(byte_idx) {
            Some(&b) => b,
            None => {
                return Err(ContextraError::InvalidInput(format!(
                    "Unexpected out-of-bounds byte access at index {}",
                    byte_idx
                )));
            }
        };

        let code = (byte >> bit_shift) & 0x03;
        let val = block.meta.zero_point + (code as f32) * block.meta.scale;
        result.push(val);
    }

    Ok(result)
}

// ANCHOR[contextra-kvcache:KIVI-009]
/// Referenz-Implementierung für lineare 2-Bit Min/Max-Quantisierung eines Fließkomma-Slices.
pub fn pack_kivi_block(values: &[f32]) -> KiviQuantizedBlock {
    if values.is_empty() {
        return KiviQuantizedBlock {
            meta: KiviBlockMeta {
                bits_per_value: 2,
                scale: 0.0,
                zero_point: 0.0,
                original_len: 0,
                num_tokens: 0,
                num_channels: 0,
                key_group_size: 16,
                quantize_values: true,
                key_scales: Vec::new(),
                key_zero_points: Vec::new(),
                value_scales: Vec::new(),
                value_zero_points: Vec::new(),
            },
            packed: Vec::new(),
        };
    }

    let finite_vals: Vec<f32> = values.iter().copied().filter(|v| v.is_finite()).collect();
    let (min_v, max_v) = if !finite_vals.is_empty() {
        let min_val = finite_vals.iter().copied().fold(f32::INFINITY, f32::min);
        let max_val = finite_vals
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        (min_val, max_val)
    } else {
        (0.0, 0.0)
    };

    let (scale, zero_point) = if (max_v - min_v).abs() < f32::EPSILON {
        (0.0, min_v)
    } else {
        let sc = (max_v - min_v) / 3.0;
        (sc, min_v)
    };

    let packed_len = values.len().div_ceil(4);
    let mut packed = vec![0u8; packed_len];

    for (i, &v) in values.iter().enumerate() {
        let byte_idx = i / 4;
        let bit_shift = (i % 4) * 2;

        let code = if scale > 0.0 && v.is_finite() {
            let norm = ((v - zero_point) / scale).round();
            norm.clamp(0.0, 3.0) as u8
        } else {
            0u8
        };

        if let Some(byte) = packed.get_mut(byte_idx) {
            *byte |= (code & 0x03) << bit_shift;
        }
    }

    KiviQuantizedBlock {
        meta: KiviBlockMeta {
            bits_per_value: 2,
            scale,
            zero_point,
            original_len: values.len(),
            num_tokens: 1,
            num_channels: values.len(),
            key_group_size: 16,
            quantize_values: true,
            key_scales: vec![scale],
            key_zero_points: vec![zero_point],
            value_scales: Vec::new(),
            value_zero_points: Vec::new(),
        },
        packed,
    }
}

// ANCHOR[contextra-kvcache:KIVI-010]
/// Komprimiert ein Byte-Slice verlustfrei für Tier-2 Auslagerungen (LSM-Spill).
///
/// Nutzt ein einfaches Run-Length-Encoding (RLE) mit Header `0x43` ('C' für Compressed)
/// bzw. Fallback `0x55` ('U' für Uncompressed), falls RLE keine Ersparnis bringt.
pub fn compress_bytes(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return vec![0x55];
    }
    let mut compressed = Vec::with_capacity(input.len());
    compressed.push(0x43); // 'C' header

    let mut i = 0;
    while i < input.len() {
        let byte = input[i];
        let mut run_len = 1u8;
        while i + (run_len as usize) < input.len()
            && input[i + (run_len as usize)] == byte
            && run_len < 255
        {
            run_len += 1;
        }
        compressed.push(run_len);
        compressed.push(byte);
        i += run_len as usize;
    }

    if compressed.len() >= input.len() {
        let mut raw = Vec::with_capacity(1 + input.len());
        raw.push(0x55); // 'U' header
        raw.extend_from_slice(input);
        raw
    } else {
        compressed
    }
}

// ANCHOR[contextra-kvcache:KIVI-011]
/// Dekomprimiert ein zuvor mit [`compress_bytes`] komprimiertes Byte-Slice.
///
/// # Fehler
/// Liefert [`ContextraError::kv_quantization`], falls die Eingabe leer ist oder der Header ungültig ist.
pub fn decompress_bytes(input: &[u8]) -> Result<Vec<u8>> {
    if input.is_empty() {
        return Err(ContextraError::kv_quantization(
            "Empty compression payload".to_string(),
        ));
    }
    match input[0] {
        0x55 => Ok(input[1..].to_vec()),
        0x43 => {
            let mut decompressed = Vec::new();
            let mut i = 1;
            while i + 1 < input.len() {
                let count = input[i] as usize;
                let byte = input[i + 1];
                decompressed.resize(decompressed.len() + count, byte);
                i += 2;
            }
            Ok(decompressed)
        }
        other => Err(ContextraError::kv_quantization(format!(
            "Invalid compression header byte 0x{:02x}",
            other
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // Prüft, dass ungültige Metadaten (z. B. bits_per_value != 2) sauber mit Fehler quittiert werden ohne Panic.
    #[test]
    fn test_kivi_metadata_validation_errors_no_panic() {
        let meta_wrong_bits = KiviBlockMeta {
            bits_per_value: 4,
            scale: 1.0,
            zero_point: 0.0,
            original_len: 4,
            num_tokens: 1,
            num_channels: 4,
            key_group_size: 16,
            quantize_values: true,
            key_scales: vec![1.0],
            key_zero_points: vec![0.0],
            value_scales: vec![],
            value_zero_points: vec![],
        };
        let block_wrong_bits = KiviQuantizedBlock {
            meta: meta_wrong_bits,
            packed: vec![0x00],
        };
        assert!(unpack_kivi_block(&block_wrong_bits).is_err());
    }

    // Prüft grundlegendes Quantisieren und Dequantisieren eines einfachen KvTensorViews.
    #[test]
    fn test_kivi_quantize_dequantize_roundtrip_simple() {
        let raw = KvTensorView::new(vec![1.0, 2.0, 3.0, 4.0], vec![10.0, 20.0, 30.0, 40.0], 2, 2)
            .unwrap();

        let config = KiviQuantizeConfig {
            key_group_size: 16,
            quantize_values: true,
        };

        let block = kivi_quantize(&raw, config).unwrap();
        let decompressed = kivi_dequantize(&block.packed, &block.meta).unwrap();

        assert_eq!(decompressed.num_tokens, 2);
        assert_eq!(decompressed.num_channels, 2);
        assert_eq!(decompressed.keys.len(), 4);
        assert_eq!(decompressed.values.len(), 4);
    }

    // Prüft verlustfreien Roundtrip von Kompression und Dekompression.
    #[test]
    fn test_compress_decompress_bytes_roundtrip() {
        let sample = vec![0x00, 0x00, 0x00, 0x00, 0xAA, 0xBB, 0xCC, 0xDD];
        let compressed = compress_bytes(&sample);
        let decompressed = decompress_bytes(&compressed).unwrap();
        assert_eq!(sample, decompressed);
    }

    // Prüft die Rekonstruktionsgenauigkeit: Kosinus-Ähnlichkeit zwischen Original und Rekonstruktion muss > 0.99 sein.
    #[test]
    fn test_v3_kivi_quantization_cosine_similarity_reconstruction() {
        // Generate a synthetic attention KV tensor view [num_tokens: 32, num_channels: 64]
        let num_tokens = 32;
        let num_channels = 64;
        let total_elems = num_tokens * num_channels;

        let mut keys = Vec::with_capacity(total_elems);
        let mut values = Vec::with_capacity(total_elems);

        for t in 0..num_tokens {
            for c in 0..num_channels {
                // Keys: per-channel grouping (key_group_size 16)
                let k_val = ((t as f32 * 0.1).sin() + (c as f32 * 0.05).cos()) * 2.5;
                keys.push(k_val);

                // Values: per-token quantization in KIVI. Generate values with smooth token-wise variation
                let v_val = (t as f32 * 0.2).sin() + 1.5 + (c as f32 * 0.01);
                values.push(v_val);
            }
        }

        let raw =
            KvTensorView::new(keys.clone(), values.clone(), num_tokens, num_channels).unwrap();

        let config = KiviQuantizeConfig {
            key_group_size: 16,
            quantize_values: true,
        };

        let block = kivi_quantize(&raw, config).unwrap();
        let decompressed = kivi_dequantize(&block.packed, &block.meta).unwrap();

        // Calculate Cosine Similarity between original raw and reconstructed keys/values
        fn compute_cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
            let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
            let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
            let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm_a == 0.0 || norm_b == 0.0 {
                0.0
            } else {
                dot / (norm_a * norm_b)
            }
        }

        let key_cos_sim = compute_cosine_similarity(&keys, &decompressed.keys);
        let val_cos_sim = compute_cosine_similarity(&values, &decompressed.values);

        assert!(
            key_cos_sim > 0.99,
            "Key reconstruction Cosine Similarity must be > 0.99, got {:.6}",
            key_cos_sim
        );
        assert!(
            val_cos_sim > 0.99,
            "Value reconstruction Cosine Similarity must be > 0.99, got {:.6}",
            val_cos_sim
        );
    }

    proptest! {
        // Eigenschaftstest: Beliebige Byte-Vektoren bleiben nach Kompression und Dekompression unverändert.
        #[test]
        fn prop_compression_roundtrip(vec in proptest::collection::vec(0u8..255u8, 0..100)) {
            let compressed = compress_bytes(&vec);
            let decompressed = decompress_bytes(&compressed).unwrap();
            prop_assert_eq!(vec, decompressed);
        }
    }
}
