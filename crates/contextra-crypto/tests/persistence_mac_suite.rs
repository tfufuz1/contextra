//! `mac_suite`: BLAKE3-keyed-MAC als `Mac` (SPEC §4; algo_set 1; IMPL WP-2.4).
//!
//! Testvektoren: BLAKE3-team/BLAKE3, `test_vectors/test_vectors.json`, Feld `keyed_hash`
//! (erste 16 Byte, da der Port 16 Byte liefert). Eingabe: Byte `i mod 251` für
//! `i < input_len`, Schlüssel: der Testschlüssel der Datei (32 Byte).

use std::error::Error;

use contextra_crypto::persistence::Blake3Mac;
use contextra_ports::persistence::Mac;

type TestResult = Result<(), Box<dyn Error>>;

const KEY: &[u8; 32] = b"whats the Elvish word for friend";

/// (input_len, keyed_hash[0..16] als Hex)
const CASES: &[(usize, &str)] = &[
    (0, "92b2b75604ed3c761f9d6f62392c8a92"),
    (1, "6d7878dfff2f485635d39013278ae14f"),
    (2, "5392ddae0e0a69d5f40160462cbd9bd8"),
    (3, "39e67b76b5a007d4921969779fe666da"),
    (4, "7671dde590c95d5ac9616651ff5aa0a2"),
    (5, "73ac69eecf286894d8102018a6fc729f"),
    (6, "82d3199d0013035682cc7f2a399d4c21"),
    (7, "af0a7ec382aedc0cfd626e49e7628bc7"),
    (8, "be2f5495c61cba1bb348a34948c00404"),
    (63, "bb1eb5d4afa793c1ebdd9fb08def6c36"),
    (64, "ba8ced36f327700d213f120b1a207a3b"),
    (65, "c0a4edefa2d2accb9277c371ac12fcdb"),
    (127, "c64200ae7dfaf35577ac5a9521c47863"),
    (128, "b04fe15577457267ff3b6f3c947d93be"),
    (129, "d4a64dae6cdccbac1e5287f54f17c5f9"),
    (1023, "c951ecdf03288d0fcc96ee3413563d8a"),
    (1024, "75c46f6f3d9eb4f55ecaaee480db732e"),
    (1025, "357dc55de0c7e382c900fd6e320acc04"),
    (2048, "879cf1fa2ea0e79126cb1063617a05b6"),
    (2049, "9f29700902f7c86e514ddc4df1e3049f"),
    (3072, "044a0e7b172a312dc02a4c9a818c036f"),
    (3073, "68dede9bef00ba89e43f31a6825f4cf4"),
    (4096, "befc660aea2f1718884cd8deb9902811"),
    (4097, "00df940cd36bb9fa7cbbc3556744e0db"),
    (5120, "2c493e48e9b9bf31e0553a22b23503c0"),
    (5121, "6ccf1c34753e7a044db80798ecd0782a"),
    (6144, "3d6b6d21281d0ade5b2b016ae4034c5d"),
    (6145, "9ac301e9e39e45e3250a7e3b3df701aa"),
    (7168, "b42835e40e9d4a7f42ad8cc04f85a963"),
    (7169, "ed9b1a922c046fdb3d423ae34e143b05"),
    (8192, "dc9637c8845a770b4cbf76b8daec0eeb"),
    (8193, "954a2a75420c8d6547e3ba5b98d963e6"),
    (16384, "9e9fc4eb7cf081ea7c47d1807790ed21"),
    (31744, "efa53b389ab67c593dba624d898d0f73"),
    (102400, "1c35d1a5811083fd7119f5d5d1ba027b"),
];

fn hex(s: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = s.as_bytes();
    if !bytes.len().is_multiple_of(2) {
        return Err("ungerade Hex-Länge".into());
    }
    bytes
        .chunks(2)
        .map(|pair| -> Result<u8, Box<dyn Error>> {
            let txt = std::str::from_utf8(pair)?;
            Ok(u8::from_str_radix(txt, 16)?)
        })
        .collect()
}

fn input(len: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    (0..len)
        .map(|i| u8::try_from(i % 251))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(Into::into)
}

#[test]
fn blake3_keyed_vectors_prefix_16() -> TestResult {
    let mac = Blake3Mac;
    for (len, expected_hex) in CASES {
        let data = input(*len)?;
        let expected = <[u8; 16]>::try_from(hex(expected_hex)?.as_slice())?;
        let (a, b) = data.split_at(data.len() / 2);
        assert_eq!(mac.mac16(KEY, &[data.as_slice()]), expected, "len {len}");
        assert_eq!(
            mac.mac16(KEY, &[a, b]),
            expected,
            "geteilte Teile, len {len}"
        );
    }
    Ok(())
}

#[test]
fn verify_accepts_correct_and_rejects_altered_inputs() -> TestResult {
    let mac = Blake3Mac;
    let msg = b"geheime Nachricht";
    let tag = mac.mac16(KEY, &[msg]);
    assert!(mac.verify16(KEY, &[msg], &tag));

    let mut bad_tag = tag;
    if let Some(b) = bad_tag.first_mut() {
        *b ^= 0x01;
    }
    assert!(!mac.verify16(KEY, &[msg], &bad_tag), "veränderter Tag");

    let other_key = [0u8; 32];
    assert!(!mac.verify16(&other_key, &[msg], &tag), "anderer Schlüssel");

    let altered = b"geheime NachrichT";
    assert!(!mac.verify16(KEY, &[altered], &tag), "veränderte Nachricht");
    Ok(())
}

#[test]
fn tag_depends_on_every_part() -> TestResult {
    let mac = Blake3Mac;
    let tag = mac.mac16(KEY, &[b"a", b"b"]);
    assert_ne!(tag, mac.mac16(KEY, &[b"a", b"c"]));
    assert_ne!(tag, mac.mac16(KEY, &[b"a"]));
    Ok(())
}
