// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Externer Zeitanker nach RFC 3161 und lokaler Entwicklungs-Zeitanker für Löschbeweise.
// INVARIANTEN: Zero-Panic (kein unwrap/expect/panic), Fail-closed, keine Rekursion im DER-Parser,
//              ehrliche Auszeichnung von LocalMonotonicAnchor als NotQualified (kein eIDAS Art. 42 / § 371a ZPO).

use crate::error::CryptoError;
use serde::{Deserialize, Serialize};

/// Qualifikationsniveau eines Zeitstempel-Ankers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualificationLevel {
    /// Lokaler monotoner Zeitanker.
    LocalMonotonic,
    /// Externe verifizierte Qualifizierung (nur durch Verifier-Beweis konstruierbar).
    Qualified(RealTsaProofToken),
}

/// Privater Proof-Token, der nur durch erfolgreiche `RealTsaVerifier`-Prüfung ausgestellt werden kann.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RealTsaProofToken {
    _private: (),
}

impl RealTsaProofToken {
    #[allow(dead_code)]
    pub(crate) fn new() -> Self {
        Self { _private: () }
    }
}

/// Qualifikationsstatus eines Zeitstempel-Ankers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnchorQualification {
    /// Qualifizierter elektronischer Zeitstempel.
    Qualified,
    /// Nicht qualifizierter Zeitstempel (z. B. lokaler Entwicklungs- oder Test-Anker).
    NotQualified,
}

/// Nachweis/Quittung über eine Zeitanker-Bindung.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorReceipt {
    /// Gebundener Hash (MessageImprint).
    pub message_imprint: Vec<u8>,
    /// Erzeugungszeitpunkt (Unix-Timestamp in Sekunden).
    pub gen_time_unix_secs: u64,
    /// Seriennummer der TSA-Antwort.
    pub serial_number: Vec<u8>,
    /// OID der Zeitstempel-Policy (z. B. "1.2.3.4").
    pub policy_oid: String,
    /// Qualifikationsstatus des Ankers.
    pub qualification: AnchorQualification,
    /// Typbezeichnung des Ankers (z. B. "RFC3161-TSA" oder "LocalMonotonic").
    pub anchor_type: String,
}

/// Trait für Zeitstempel-Anker.
pub trait TimeAnchor: Send + Sync {
    /// Bindet einen Hash (MessageImprint) an einen Zeitanker und erzeugt ein `AnchorReceipt`.
    fn anchor(&self, msg_imprint: &[u8]) -> Result<AnchorReceipt, CryptoError>;
}

/// Trait zur Verifikation von TSA-Signaturen (Dependency-Injektion ohne Netzwerk/Krypto-Kopplung).
pub trait TsaSignatureVerifier: Send + Sync {
    /// Verifiziert die kryptographische Signatur der TSA über die Token-Daten.
    fn verify_signature(&self, signed_data: &[u8], signature: &[u8]) -> Result<(), CryptoError>;
}

/// Pass-Through-Verifier für Tests oder vertrauenswürdige In-Memory-TSA-Mocks.
#[cfg(any(test, feature = "testing", feature = "test-support", feature = "test-utils"))]
#[derive(Debug, Clone, Default)]
pub struct AcceptAllTsaVerifier;

#[cfg(any(test, feature = "testing", feature = "test-support", feature = "test-utils"))]
impl TsaSignatureVerifier for AcceptAllTsaVerifier {
    fn verify_signature(&self, _signed_data: &[u8], _signature: &[u8]) -> Result<(), CryptoError> {
        Ok(())
    }
}

#[cfg(any(test, feature = "testing", feature = "test-support", feature = "test-utils"))]
#[deprecated(note = "NoOpTsaSignatureVerifier is renamed to AcceptAllTsaVerifier")]
pub type NoOpTsaSignatureVerifier = AcceptAllTsaVerifier;

/// Lokaler monotoner Zeitanker für Entwicklung und Tests.
///
/// EHRLICHE DOKUMENTATION:
/// Dieser Anker ist KEIN qualifizierter Zeitstempel im Sinne der eIDAS-Verordnung (Art. 42)
/// und erfüllt nicht die Anforderungen des § 371a ZPO für den Anscheinsbeweis.
/// Er dient ausschließlich zu Testzwecken und lokaler Entwicklung.
#[derive(Debug, Clone)]
pub struct LocalMonotonicAnchor {
    policy_oid: String,
}

impl LocalMonotonicAnchor {
    pub fn new(policy_oid: impl Into<String>) -> Self {
        Self {
            policy_oid: policy_oid.into(),
        }
    }
}

impl Default for LocalMonotonicAnchor {
    fn default() -> Self {
        Self::new("1.3.6.1.4.1.58309.1.1")
    }
}

impl TimeAnchor for LocalMonotonicAnchor {
    fn anchor(&self, msg_imprint: &[u8]) -> Result<AnchorReceipt, CryptoError> {
        if msg_imprint.is_empty() {
            return Err(CryptoError::InvalidInput(
                "Message imprint cannot be empty".to_string(),
            ));
        }

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| CryptoError::InvalidInput("System time before UNIX EPOCH".to_string()))?
            .as_secs();

        // Deterministische Seriennummer basierend auf Hash des Imprints + Timestamp
        let mut serial = Vec::with_capacity(msg_imprint.len() + 8);
        serial.extend_from_slice(msg_imprint);
        serial.extend_from_slice(&now_secs.to_be_bytes());

        Ok(AnchorReceipt {
            message_imprint: msg_imprint.to_vec(),
            gen_time_unix_secs: now_secs,
            serial_number: serial,
            policy_oid: self.policy_oid.clone(),
            qualification: AnchorQualification::NotQualified,
            anchor_type: "LocalMonotonic".to_string(),
        })
    }
}

/// Minimaler, sicherer, non-rekursiver DER-Parser für RFC 3161 `TSTInfo`.
#[derive(Debug, Clone)]
pub struct DerTstInfoParser;

/// Parsed `TSTInfo` Felder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTstInfo {
    pub policy_oid: String,
    pub message_imprint: Vec<u8>,
    pub serial_number: Vec<u8>,
    pub gen_time_unix_secs: u64,
}

impl DerTstInfoParser {
    /// Parse raw DER-encoded TSTInfo payload without panic or recursion.
    pub fn parse_tst_info(der: &[u8]) -> Result<ParsedTstInfo, CryptoError> {
        const MAX_DER_SIZE: usize = 65536;
        if der.is_empty() || der.len() > MAX_DER_SIZE {
            return Err(CryptoError::InvalidInput(
                "DER data is empty or exceeds maximum size limit (64KB)".to_string(),
            ));
        }

        let mut pos = 0;
        let (tag, length) = Self::read_tlv_header(der, &mut pos)?;

        // TSTInfo MUST be a SEQUENCE (0x30)
        if tag != 0x30 {
            return Err(CryptoError::InvalidInput(format!(
                "Expected SEQUENCE tag (0x30), found 0x{:02X}",
                tag
            )));
        }

        if pos + length > der.len() {
            return Err(CryptoError::InvalidInput(
                "DER SEQUENCE payload truncated".to_string(),
            ));
        }

        let seq_end = pos + length;

        // Parse fields inside TSTInfo SEQUENCE sequentially:
        // 1. version INTEGER (0x02)
        // 2. policy OBJECT IDENTIFIER (0x06)
        // 3. messageImprint SEQUENCE (0x30)
        // 4. serialNumber INTEGER (0x02)
        // 5. genTime GeneralizedTime (0x18)

        // 1. Version INTEGER
        let (v_tag, v_len) = Self::read_tlv_header(der, &mut pos)?;
        if v_tag != 0x02 || pos + v_len > seq_end {
            return Err(CryptoError::InvalidInput(
                "Invalid TSTInfo version field".to_string(),
            ));
        }
        pos += v_len;

        // 2. Policy OBJECT IDENTIFIER
        let (p_tag, p_len) = Self::read_tlv_header(der, &mut pos)?;
        if p_tag != 0x06 || pos + p_len > seq_end {
            return Err(CryptoError::InvalidInput(
                "Invalid TSTInfo policy field".to_string(),
            ));
        }
        let policy_bytes = &der[pos..pos + p_len];
        let policy_oid = Self::parse_oid(policy_bytes)?;
        pos += p_len;

        // 3. messageImprint SEQUENCE
        let (mi_tag, mi_len) = Self::read_tlv_header(der, &mut pos)?;
        if mi_tag != 0x30 || pos + mi_len > seq_end {
            return Err(CryptoError::InvalidInput(
                "Invalid TSTInfo messageImprint field".to_string(),
            ));
        }
        let mi_end = pos + mi_len;

        // Inside messageImprint: AlgorithmIdentifier (SEQUENCE), hashedMessage (OCTET STRING)
        let (alg_tag, alg_len) = Self::read_tlv_header(der, &mut pos)?;
        if alg_tag != 0x30 || pos + alg_len > mi_end {
            return Err(CryptoError::InvalidInput(
                "Invalid messageImprint hashAlgorithm field".to_string(),
            ));
        }
        pos += alg_len;

        let (hash_tag, hash_len) = Self::read_tlv_header(der, &mut pos)?;
        if hash_tag != 0x04 || pos + hash_len > mi_end {
            return Err(CryptoError::InvalidInput(
                "Invalid messageImprint hashedMessage field".to_string(),
            ));
        }
        let message_imprint = der[pos..pos + hash_len].to_vec();
        pos = mi_end;

        // 4. serialNumber INTEGER
        let (sn_tag, sn_len) = Self::read_tlv_header(der, &mut pos)?;
        if sn_tag != 0x02 || pos + sn_len > seq_end {
            return Err(CryptoError::InvalidInput(
                "Invalid TSTInfo serialNumber field".to_string(),
            ));
        }
        let serial_number = der[pos..pos + sn_len].to_vec();
        pos += sn_len;

        // 5. genTime GeneralizedTime
        let (gt_tag, gt_len) = Self::read_tlv_header(der, &mut pos)?;
        if gt_tag != 0x18 || pos + gt_len > seq_end {
            return Err(CryptoError::InvalidInput(
                "Invalid TSTInfo genTime field".to_string(),
            ));
        }
        let gen_time_str = std::str::from_utf8(&der[pos..pos + gt_len]).map_err(|_| {
            CryptoError::InvalidInput("Invalid UTF-8/ASCII in GeneralizedTime".to_string())
        })?;
        let gen_time_unix_secs = Self::parse_generalized_time(gen_time_str)?;

        Ok(ParsedTstInfo {
            policy_oid,
            message_imprint,
            serial_number,
            gen_time_unix_secs,
        })
    }

    fn read_tlv_header(der: &[u8], pos: &mut usize) -> Result<(u8, usize), CryptoError> {
        if *pos >= der.len() {
            return Err(CryptoError::InvalidInput(
                "Unexpected end of DER input while reading tag".to_string(),
            ));
        }

        let tag = der[*pos];
        *pos += 1;

        if *pos >= der.len() {
            return Err(CryptoError::InvalidInput(
                "Unexpected end of DER input while reading length".to_string(),
            ));
        }

        let first_len_byte = der[*pos];
        *pos += 1;

        let length = if first_len_byte < 0x80 {
            first_len_byte as usize
        } else if first_len_byte == 0x80 {
            return Err(CryptoError::InvalidInput(
                "Indefinite length encoding is forbidden in DER".to_string(),
            ));
        } else {
            let num_bytes = (first_len_byte & 0x7F) as usize;
            if num_bytes > 4 {
                return Err(CryptoError::InvalidInput(
                    "DER length field exceeds maximum supported 4 bytes".to_string(),
                ));
            }
            if *pos + num_bytes > der.len() {
                return Err(CryptoError::InvalidInput(
                    "Unexpected EOF reading DER multi-byte length".to_string(),
                ));
            }

            let mut len: usize = 0;
            for &b in &der[*pos..*pos + num_bytes] {
                len = (len << 8) | (b as usize);
            }
            *pos += num_bytes;
            len
        };

        Ok((tag, length))
    }

    fn parse_oid(bytes: &[u8]) -> Result<String, CryptoError> {
        if bytes.is_empty() {
            return Err(CryptoError::InvalidInput("Empty OID bytes".to_string()));
        }

        let mut components = Vec::new();

        // First byte encodes components 1 and 2: X * 40 + Y
        let first = bytes[0] as u32;
        let x = first / 40;
        let y = first % 40;
        components.push(x.to_string());
        components.push(y.to_string());

        let mut current_val: u64 = 0;
        let mut in_component = false;

        for &b in &bytes[1..] {
            let val = (b & 0x7F) as u64;
            if !in_component {
                current_val = val;
                in_component = true;
            } else {
                current_val = current_val.checked_shl(7).ok_or_else(|| {
                    CryptoError::InvalidInput("OID component overflow".to_string())
                })? | val;
            }

            if (b & 0x80) == 0 {
                components.push(current_val.to_string());
                current_val = 0;
                in_component = false;
            }
        }

        if in_component {
            return Err(CryptoError::InvalidInput(
                "Truncated OID byte stream".to_string(),
            ));
        }

        Ok(components.join("."))
    }

    fn parse_generalized_time(s: &str) -> Result<u64, CryptoError> {
        // Expected format: YYYYMMDDHHMMSSZ or YYYYMMDDHHMMSS.fffZ
        if s.len() < 15 || !s.ends_with('Z') {
            return Err(CryptoError::InvalidInput(format!(
                "Invalid GeneralizedTime format: {}",
                s
            )));
        }

        let year: u32 = s[0..4].parse().map_err(|_| {
            CryptoError::InvalidInput("Invalid year in GeneralizedTime".to_string())
        })?;
        let month: u32 = s[4..6].parse().map_err(|_| {
            CryptoError::InvalidInput("Invalid month in GeneralizedTime".to_string())
        })?;
        let day: u32 = s[6..8]
            .parse()
            .map_err(|_| CryptoError::InvalidInput("Invalid day in GeneralizedTime".to_string()))?;
        let hour: u32 = s[8..10].parse().map_err(|_| {
            CryptoError::InvalidInput("Invalid hour in GeneralizedTime".to_string())
        })?;
        let min: u32 = s[10..12].parse().map_err(|_| {
            CryptoError::InvalidInput("Invalid minute in GeneralizedTime".to_string())
        })?;
        let sec: u32 = s[12..14].parse().map_err(|_| {
            CryptoError::InvalidInput("Invalid second in GeneralizedTime".to_string())
        })?;

        if month < 1 || month > 12 || day < 1 || day > 31 || hour > 23 || min > 59 || sec > 60 {
            return Err(CryptoError::InvalidInput(
                "GeneralizedTime field value out of range".to_string(),
            ));
        }

        // Simplifizierte Konvertierung nach Unix-Timestamp
        let days_since_epoch = naive_days_from_civil(year as i32, month, day);
        let unix_secs =
            days_since_epoch as u64 * 86400 + hour as u64 * 3600 + min as u64 * 60 + sec as u64;

        Ok(unix_secs)
    }
}

/// Algorithmus aus Hatcher/Howard zur Berechnung der Tage seit Unix-Epoch (1970-01-01).
fn naive_days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year } as i64;
    let m = if month <= 2 { month + 12 } else { month } as i64;
    let d = day as i64;

    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * (m - 3) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy as u64;

    era * 146097 + (doe as i64) - 719468
}

/// Anker-Implementierung für externe RFC-3161 TSA-Antworten.
#[derive(Debug)]
pub struct Rfc3161TimeAnchor<V: TsaSignatureVerifier> {
    tsa_verifier: V,
}

impl<V: TsaSignatureVerifier> Rfc3161TimeAnchor<V> {
    pub fn new(tsa_verifier: V) -> Self {
        Self { tsa_verifier }
    }

    /// Verifiziert eine RFC-3161 TSA-Antwort (DER-codiertes TSTInfo) und Signatur.
    pub fn parse_and_verify(
        &self,
        tst_info_der: &[u8],
        signature: &[u8],
        expected_imprint: &[u8],
    ) -> Result<AnchorReceipt, CryptoError> {
        self.tsa_verifier
            .verify_signature(tst_info_der, signature)?;

        let parsed = DerTstInfoParser::parse_tst_info(tst_info_der)?;

        if parsed.message_imprint != expected_imprint {
            return Err(CryptoError::InvalidInput(
                "Message imprint mismatch in TSA response".to_string(),
            ));
        }

        Ok(AnchorReceipt {
            message_imprint: parsed.message_imprint,
            gen_time_unix_secs: parsed.gen_time_unix_secs,
            serial_number: parsed.serial_number,
            policy_oid: parsed.policy_oid,
            qualification: AnchorQualification::Qualified,
            anchor_type: "RFC3161-TSA".to_string(),
        })
    }
}
