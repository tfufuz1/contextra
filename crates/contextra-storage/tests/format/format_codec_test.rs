//! # Negativtests des FORMAT-Kopfes (SPEC §10.2, INV-FMT-1, F-15)
//!
//! Jede Manipulation wird mit einem neu berechneten `chk` eingebracht, damit der jeweils
//! gemeinte Fehler greift und nicht der Prüfwert. Die Reihenfolge der Prüfungen ist Teil
//! des Vertrags (siehe Modul `codec::format_header`).

use contextra_storage::codec::crc32c::crc32c;
use contextra_storage::codec::format_header::{
    decode_format_plain, encode_format_plain, FormatCodecError, FormatFields,
    ALGO_SET_XCHACHA_BLAKE3, FEATURE_ENCRYPTED, INTEGRITY_PLAIN,
};

fn valid() -> Vec<u8> {
    let f = FormatFields {
        feature_req: FEATURE_ENCRYPTED,
        feature_opt: 0,
        integrity_mode: INTEGRITY_PLAIN,
        instance_id: [0x5a; 16],
        algo_set: ALGO_SET_XCHACHA_BLAKE3,
        created_ms: 1_000,
    };
    encode_format_plain(&f, &crc32c).expect("Plain kodierbar")
}

/// Setzt `chk` über die aktuellen 49 Byte neu.
fn resign(mut bytes: Vec<u8>) -> Vec<u8> {
    let chk = crc32c(0, bytes.get(..49).expect("49 Byte Körper"));
    bytes.truncate(49);
    bytes.extend_from_slice(&chk.to_le_bytes());
    bytes
}

fn set(mut bytes: Vec<u8>, off: usize, val: &[u8]) -> Vec<u8> {
    for (i, b) in val.iter().enumerate() {
        if let Some(slot) = bytes.get_mut(off + i) {
            *slot = *b;
        }
    }
    bytes
}

#[test]
fn gueltiger_kopf_wird_akzeptiert() {
    assert!(decode_format_plain(&valid(), &crc32c).is_ok());
}

#[test]
fn falsches_magic_ist_badmagic() {
    let bytes = resign(set(valid(), 0, b"CTXX"));
    assert_eq!(
        decode_format_plain(&bytes, &crc32c),
        Err(FormatCodecError::BadMagic)
    );
}

#[test]
fn modus_abweichung_wird_vor_pruefwert_gemeldet() {
    // F-15: integrity_mode 1 ohne neu berechneten chk -> Unsupported, nicht ChecksumMismatch.
    let bytes = set(valid(), 22, &[1]);
    assert_eq!(
        decode_format_plain(&bytes, &crc32c),
        Err(FormatCodecError::UnsupportedIntegrityMode(1))
    );
}

#[test]
fn manipulierter_koerper_ist_checksum_fehler() {
    let mut bytes = valid();
    if let Some(b) = bytes.get_mut(30) {
        *b ^= 0x01;
    }
    assert!(matches!(
        decode_format_plain(&bytes, &crc32c),
        Err(FormatCodecError::ChecksumMismatch { .. })
    ));
}

#[test]
fn unbekannte_version_ist_unsupported() {
    let bytes = resign(set(valid(), 4, &2u16.to_le_bytes()));
    assert_eq!(
        decode_format_plain(&bytes, &crc32c),
        Err(FormatCodecError::UnsupportedVersion(2))
    );
}

#[test]
fn unbekanntes_pflicht_bit_ist_unsupported() {
    let unknown = 1u64 << 4;
    let bytes = resign(set(
        valid(),
        6,
        &(FEATURE_ENCRYPTED | unknown).to_le_bytes(),
    ));
    assert_eq!(
        decode_format_plain(&bytes, &crc32c),
        Err(FormatCodecError::UnsupportedRequiredFeature(unknown))
    );
}

#[test]
fn unbekanntes_optionales_bit_wird_ignoriert() {
    let opt = 1u64 << 40;
    let bytes = resign(set(valid(), 14, &opt.to_le_bytes()));
    let f = decode_format_plain(&bytes, &crc32c).expect("optionales Bit ignorieren");
    assert_eq!(f.feature_opt, opt);
}

#[test]
fn unbekanntes_algo_set_ist_unsupported() {
    let bytes = resign(set(valid(), 39, &2u16.to_le_bytes()));
    assert_eq!(
        decode_format_plain(&bytes, &crc32c),
        Err(FormatCodecError::UnsupportedAlgoSet(2))
    );
}

#[test]
fn zu_kurz_ist_truncated() {
    let mut bytes = valid();
    bytes.truncate(52);
    assert_eq!(
        decode_format_plain(&bytes, &crc32c),
        Err(FormatCodecError::Truncated {
            expected: 53,
            actual: 52
        })
    );
}

#[test]
fn restbytes_sind_fehler() {
    let mut bytes = valid();
    bytes.push(0);
    assert_eq!(
        decode_format_plain(&bytes, &crc32c),
        Err(FormatCodecError::TrailingBytes { extra: 1 })
    );
}
