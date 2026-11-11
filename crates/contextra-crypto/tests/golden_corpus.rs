//! Golden-Korpus Generator und Prüftest für `contextra-crypto`.
//!
//! Inhalt des Golden-Korpus:
//! 1. WAL-HMAC-Kette (`wal_hmac_chain.bin`)
//! 2. MFKD-Schlüsselableitung (`mfkd_keys.bin`)
//! 3. AEAD-Vektoren mit festem Schlüssel und fester Nonce (`aead_vectors.bin`)
//! 4. KV-Segment v2 Format (`kv_segment_v2.bin`)
//! 5. WAL-Completeness Prüfungsausgaben (`wal_completeness.bin`)
//! 6. DEK-Wrapping Vektoren (`dek_wrapping.bin`)

use contextra_crypto::aead::{open_record, seal_record_with_nonce};
use contextra_crypto::dek::{derive_dek, unwrap_dek, wrap_dek, AncestorSalts, Dek, UnitKey};
use contextra_crypto::kdf::hkdf_expand;
use contextra_crypto::kv_cipher::{EncryptedKvLayer, ModelFingerprint};
use contextra_crypto::wal_completeness::verify_wal_chain_completeness;
use contextra_crypto::wal_crypto::WalHmac;
use contextra_types::TenantId;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const GOLDEN_DIR: &str = "tests/golden";

fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn generate_wal_hmac_data() -> Vec<u8> {
    let key = [0x42u8; 32];
    let mut current_hmac = [0u8; 32];
    let mut out = Vec::new();

    for i in 0..10u64 {
        let entry_key = format!("test_key_{i}").into_bytes();
        let entry_val = format!("test_val_{i}").into_bytes();
        let mut mac = WalHmac::new(&key).expect("wal hmac new");
        mac.update(&current_hmac);
        mac.update(&i.to_le_bytes());
        mac.update(&(entry_key.len() as u32).to_le_bytes());
        mac.update(&entry_key);
        mac.update(&(entry_val.len() as u32).to_le_bytes());
        mac.update(&entry_val);
        let next_hmac = mac.finalize();
        out.extend_from_slice(&next_hmac);
        current_hmac = next_hmac;
    }
    out
}

fn generate_mfkd_data() -> Vec<u8> {
    let prk = [0x55u8; 32];
    let mut out = Vec::new();
    let labels: [&[u8]; 4] = [b"file_key", b"segment_key", b"index_key", b"recovery_key"];
    for label in labels {
        let mut sub_key = [0u8; 32];
        hkdf_expand(&prk, label, &mut sub_key).expect("hkdf expansion");
        out.extend_from_slice(&sub_key);
    }
    out
}

fn generate_aead_data() -> Vec<u8> {
    let dek = Dek::new([0x77u8; 32]);
    let doc = [0x11u8; 16];
    let key_bytes = b"sample_record_key";
    let plain = b"Hello, this is a deterministic AEAD payload test string for golden corpus!";
    let nonce = [0x12u8; 12];
    let ciphertext =
        seal_record_with_nonce(&dek, doc, key_bytes, plain, nonce).expect("seal record");

    let mut out = Vec::new();
    out.extend_from_slice(&(ciphertext.len() as u32).to_le_bytes());
    out.extend_from_slice(&ciphertext);
    out
}

fn generate_kv_segment_data() -> Vec<u8> {
    let layer = EncryptedKvLayer {
        format_version: 2,
        tenant_id: TenantId::try_new(1001).unwrap(),
        model_fingerprint: ModelFingerprint::new([0x33u8; 32], "llama-3-8b", "Q4_K_M"),
        nonce: [0x88u8; 12],
        ciphertext: b"deterministic encrypted kv segment ciphertext payload".to_vec(),
    };
    bincode::serialize(&layer).expect("serialize kv layer")
}

fn generate_wal_completeness_data() -> Vec<u8> {
    let mut out = Vec::new();
    let hmac = [0xAAu8; 32];
    let mut hwm = [0xAAu8; 32];
    let ok_res = verify_wal_chain_completeness(&hmac, &hwm).is_ok();
    out.push(ok_res as u8);

    hwm[0] ^= 0xFF;
    let err_res = verify_wal_chain_completeness(&hmac, &hwm).is_err();
    out.push(err_res as u8);
    out
}

fn generate_dek_data() -> Vec<u8> {
    let parent_key = UnitKey::new([0x99u8; 32]);
    let salts = AncestorSalts {
        t: [0x11u8; 32],
        c: [0x22u8; 32],
        s: Some([0x33u8; 32]),
        d: [0x44u8; 32],
    };
    let doc = [0x55u8; 16];
    let derived = derive_dek(&parent_key, &salts, doc);

    let nonce = [0x66u8; 12];
    let wrapped = wrap_dek(&parent_key, &derived, nonce).expect("wrap dek");

    let mut out = Vec::new();
    out.extend_from_slice(derived.expose_secret());
    out.extend_from_slice(&wrapped);
    out
}

#[test]
fn test_golden_corpus_files_exist_and_match_manifest() {
    let manifest_path = format!("{GOLDEN_DIR}/MANIFEST.sha256");
    assert!(
        Path::new(&manifest_path).exists(),
        "Golden MANIFEST.sha256 does not exist!"
    );

    let manifest_content = fs::read_to_string(&manifest_path).expect("read manifest");
    for line in manifest_content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Malformed manifest line: {line}");
        let expected_sha256 = parts[0];
        let filename = parts[1];
        let file_path = format!("{GOLDEN_DIR}/{filename}");

        let file_bytes = fs::read(&file_path)
            .expect("Could not read golden file");
        let actual_sha256 = compute_sha256(&file_bytes);

        assert_eq!(
            expected_sha256, actual_sha256,
            "Hash mismatch for golden file {filename}!"
        );
    }
}

#[test]
fn test_golden_corpus_backward_reading_without_panics() {
    // 1. Read WAL HMAC
    let wal_hmac_bytes =
        fs::read(format!("{GOLDEN_DIR}/wal_hmac_chain.bin")).expect("read wal hmac");
    assert_eq!(wal_hmac_bytes.len(), 32 * 10);

    // 2. Read MFKD keys
    let mfkd_bytes = fs::read(format!("{GOLDEN_DIR}/mfkd_keys.bin")).expect("read mfkd");
    assert_eq!(mfkd_bytes.len(), 32 * 4);

    // 3. Read AEAD vectors and decrypt
    let aead_bytes = fs::read(format!("{GOLDEN_DIR}/aead_vectors.bin")).expect("read aead");
    let ct_len = u32::from_le_bytes(aead_bytes[..4].try_into().unwrap()) as usize;
    let ciphertext = &aead_bytes[4..4 + ct_len];
    let dek = Dek::new([0x77u8; 32]);
    let key_bytes = b"sample_record_key";
    let pt = open_record(&dek, key_bytes, ciphertext).expect("decrypt golden aead");
    assert_eq!(
        pt.as_slice(),
        b"Hello, this is a deterministic AEAD payload test string for golden corpus!"
    );

    // 4. Read KV Segment v2
    let kv_bytes = fs::read(format!("{GOLDEN_DIR}/kv_segment_v2.bin")).expect("read kv");
    let layer: EncryptedKvLayer = bincode::deserialize(&kv_bytes).expect("deserialize kv layer");
    assert_eq!(layer.format_version, 2);
    assert_eq!(layer.tenant_id, TenantId::try_new(1001).unwrap());

    // 5. Read WAL completeness
    let wal_comp_bytes =
        fs::read(format!("{GOLDEN_DIR}/wal_completeness.bin")).expect("read wal comp");
    assert_eq!(wal_comp_bytes, vec![1, 1]);

    // 6. Read DEK wrapping
    let dek_bytes = fs::read(format!("{GOLDEN_DIR}/dek_wrapping.bin")).expect("read dek");
    let original_dek = &dek_bytes[..32];
    let wrapped = &dek_bytes[32..];
    let parent_key = UnitKey::new([0x99u8; 32]);
    let unwrapped = unwrap_dek(&parent_key, wrapped).expect("unwrap golden dek");
    assert_eq!(unwrapped.expose_secret(), original_dek);
}

#[test]
fn test_golden_corpus_bitflip_and_truncation_rejection() {
    let aead_bytes = fs::read(format!("{GOLDEN_DIR}/aead_vectors.bin")).expect("read aead");
    let ct_len = u32::from_le_bytes(aead_bytes[..4].try_into().unwrap()) as usize;
    let ciphertext = &aead_bytes[4..4 + ct_len];
    let dek = Dek::new([0x77u8; 32]);
    let key_bytes = b"sample_record_key";

    // 1. Bitflip at each byte position must yield Err and never panic
    for pos in 0..ciphertext.len() {
        let mut corrupted = ciphertext.to_vec();
        corrupted[pos] ^= 0x01;
        let res = open_record(&dek, key_bytes, &corrupted);
        assert!(res.is_err(), "Bitflip at position {pos} must be rejected with Err");
    }

    // 2. Truncation at deterministic lengths must yield Err and never panic
    for trunc_len in 0..ciphertext.len() {
        let truncated = &ciphertext[..trunc_len];
        let res = open_record(&dek, key_bytes, truncated);
        assert!(res.is_err(), "Truncation at length {trunc_len} must be rejected with Err");
    }

    // 3. Bitflip in DEK wrapping must yield Err and never panic
    let dek_bytes = fs::read(format!("{GOLDEN_DIR}/dek_wrapping.bin")).expect("read dek");
    let wrapped = &dek_bytes[32..];
    let parent_key = UnitKey::new([0x99u8; 32]);

    for pos in 0..wrapped.len() {
        let mut corrupted = wrapped.to_vec();
        corrupted[pos] ^= 0x01;
        let res = unwrap_dek(&parent_key, &corrupted);
        assert!(res.is_err(), "Bitflip in wrapped DEK at position {pos} must be rejected with Err");
    }

    // 4. Truncation in wrapped DEK must yield Err and never panic
    for trunc_len in 0..wrapped.len() {
        let truncated = &wrapped[..trunc_len];
        let res = unwrap_dek(&parent_key, truncated);
        assert!(res.is_err(), "Truncated wrapped DEK at length {trunc_len} must be rejected with Err");
    }
}

#[test]
#[ignore]
fn generate_golden_files() {
    fs::create_dir_all(GOLDEN_DIR).expect("create golden dir");

    let files: [(&str, Vec<u8>); 6] = [
        ("wal_hmac_chain.bin", generate_wal_hmac_data()),
        ("mfkd_keys.bin", generate_mfkd_data()),
        ("aead_vectors.bin", generate_aead_data()),
        ("kv_segment_v2.bin", generate_kv_segment_data()),
        ("wal_completeness.bin", generate_wal_completeness_data()),
        ("dek_wrapping.bin", generate_dek_data()),
    ];

    let mut manifest_lines = Vec::new();

    for (name, data) in &files {
        let path = format!("{GOLDEN_DIR}/{name}");
        fs::write(&path, data).expect("write golden file");
        let hash = compute_sha256(data);
        manifest_lines.push(format!("{hash}  {name}"));
    }

    let manifest_path = format!("{GOLDEN_DIR}/MANIFEST.sha256");
    fs::write(&manifest_path, manifest_lines.join("\n") + "\n").expect("write manifest");
    println!("Golden files regenerated successfully with SHA-256.");
}
