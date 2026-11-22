//! # Golden A.4 Test für FORMAT (Plain, 51 Bytes)
//!
//! SPEC §9.2, Anhang A.4:
//! Eingabe: version = 1, required = 0, optional = 0, integrity_mode = 0,
//! instance_id = 00 01 … 0f, created_ms = 1_700_000_000_000.
//! Byte-genaue Übereinstimmung mit Anhang A.4.

use contextra_storage::catalog::format::{
    FormatHeader, IntegrityMode, FORMAT_MAGIC, FORMAT_PLAIN_LEN,
};

#[test]
fn test_golden_a4_format_plain_byte_exact() {
    let mut instance_id = [0u8; 16];
    for i in 0..16 {
        instance_id[i] = i as u8;
    }

    let header = FormatHeader::new(
        IntegrityMode::Plain,
        instance_id,
        1_700_000_000_000,
        0,
        0,
    );

    let encoded = header.encode_plain();
    assert_eq!(encoded.len(), FORMAT_PLAIN_LEN);
    assert_eq!(encoded.len(), 51);

    // Byte-genauer Vergleich mit Golden A.4 aus SPEC.md:
    // 4354585301000000000000000000000000000000000000000102030405060708090a0b0c0d0e0f0068e5cf8b0100005663b886
    let hex_encoded: String = encoded.iter().map(|b| format!("{b:02x}")).collect();
    let expected_hex = "4354585301000000000000000000000000000000000000000102030405060708090a0b0c0d0e0f0068e5cf8b0100005663b886";
    assert_eq!(hex_encoded, expected_hex);

    // Kontrolle der Feld-Offsets
    assert_eq!(&encoded[0..4], FORMAT_MAGIC);
    assert_eq!(encoded[22], 0); // Offset 22: integrity_mode = Plain (0)

    // Dekodierung prüfen
    let decoded = FormatHeader::decode(&encoded, Some(IntegrityMode::Plain), None)
        .expect("Decode von Golden A.4 muss erfolgreich sein");

    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.required_features, 0);
    assert_eq!(decoded.optional_features, 0);
    assert_eq!(decoded.integrity_mode, IntegrityMode::Plain);
    assert_eq!(decoded.instance_id, instance_id);
    assert_eq!(decoded.created_ms, 1_700_000_000_000);
}
