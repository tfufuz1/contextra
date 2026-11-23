//! # Golden-Test FORMAT (Plain, 53 Byte) gegen den Codec
//!
//! SPEC §10.2, §19.3, §19.4. Die Goldens stammen aus `tests/format/goldens_plain.json`
//! (erzeugt von `tools/reference/generator.py`, unabhängig vom Kern). Dieser Test
//! **liest** die Datei und vergleicht sie Byte für Byte mit der Codec-Ausgabe.
//! Er ändert nie ein Golden. Stimmt eines nicht, ist das ein Befund für WP-2.1, nicht
//! eine Anpassung hier.
//!
//! Der frühere 51-Byte-Test (Altlayout, §19.3) ist durch diesen Test ersetzt.

use contextra_storage::codec::crc32c::crc32c;
use contextra_storage::codec::format_header::{
    decode_format_plain, encode_format_plain, FormatFields, ALGO_SET_XCHACHA_BLAKE3,
    FORMAT_PLAIN_LEN, INTEGRITY_PLAIN,
};

const GOLDENS: &str = include_str!("../../../../tests/format/goldens_plain.json");

/// Liest `"hex"` des Abschnitts `section` aus dem Golden-JSON.
/// Das JSON ist klein und fest formatiert; eine Abhängigkeit für den Parser ist nicht nötig.
fn golden_hex(section: &str) -> String {
    let start = GOLDENS
        .find(&format!("\"{section}\""))
        .expect("Abschnitt im Golden-JSON vorhanden");
    let rest = GOLDENS.get(start..).expect("Abschnittsende");
    let key = "\"hex\": \"";
    let from = rest.find(key).expect("hex-Feld vorhanden") + key.len();
    let to = rest.get(from..).and_then(|s| s.find('"')).expect("hex endet");
    rest.get(from..from + to).expect("hex lesbar").to_string()
}

/// Liest `"laenge"` des Abschnitts `section` aus dem Golden-JSON.
fn golden_laenge(section: &str) -> usize {
    let start = GOLDENS
        .find(&format!("\"{section}\""))
        .expect("Abschnitt im Golden-JSON vorhanden");
    let rest = GOLDENS.get(start..).expect("Abschnittsende");
    let key = "\"laenge\": ";
    let from = rest.find(key).expect("laenge-Feld vorhanden") + key.len();
    let digits: String = rest
        .get(from..)
        .expect("laenge lesbar")
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().expect("laenge ist eine Zahl")
}

/// Eingabe des Goldens `format_plain` (SPEC §10.2 und goldens_plain.json).
fn golden_input() -> FormatFields {
    FormatFields {
        feature_req: 0,
        feature_opt: 0,
        integrity_mode: INTEGRITY_PLAIN,
        instance_id: core::array::from_fn(|i| u8::try_from(i).unwrap_or(0)),
        algo_set: ALGO_SET_XCHACHA_BLAKE3,
        created_ms: 72_623_859_790_382_856,
    }
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn golden_format_plain_eingabe_stimmt_mit_json_ueberein() {
    // Die im Test verwendete Eingabe muss der Eingabe im Golden entsprechen.
    assert!(GOLDENS.contains("\"instance_id\": \"000102030405060708090a0b0c0d0e0f\""));
    assert!(GOLDENS.contains("\"created_ms\": 72623859790382856"));
    assert!(GOLDENS.contains("\"algo_set\": 1"));
}

#[test]
fn golden_format_plain_laenge_53() {
    assert_eq!(golden_laenge("format_plain"), 53);
    assert_eq!(FORMAT_PLAIN_LEN, 53);
}

#[test]
fn golden_format_plain_byte_genau_gegen_codec() {
    let encoded = encode_format_plain(&golden_input(), &crc32c).expect("Plain kodierbar");
    assert_eq!(encoded.len(), golden_laenge("format_plain"));
    assert_eq!(to_hex(&encoded), golden_hex("format_plain"));
}

#[test]
fn golden_format_plain_dekodiert_zu_eingabe() {
    let golden = golden_hex("format_plain");
    let bytes: Vec<u8> = (0..golden.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(golden.get(i..i + 2).expect("Hex-Paar"), 16).expect("Hex"))
        .collect();
    assert_eq!(decode_format_plain(&bytes, &crc32c), Ok(golden_input()));
}

#[test]
fn golden_format_plain_chk_ist_crc32c_seed_0_ueber_49_bytes() {
    // Annahme A-1: chk = CRC-32C mit Seed 0 über die 49 Byte vor chk.
    let encoded = encode_format_plain(&golden_input(), &crc32c).expect("Plain kodierbar");
    let (body, chk) = encoded.split_at(49);
    assert_eq!(body.len(), 49);
    assert_eq!(chk, crc32c(0, body).to_le_bytes());
}
