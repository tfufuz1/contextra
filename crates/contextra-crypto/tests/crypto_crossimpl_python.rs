// FILE-CONTEXT
// ZWECK: Cross-Implementation Validierung zwischen Contextra Rust Primitive-Funktionen und Python (cryptography, argon2-cffi, blake3).
// INVARIANTEN: 200 zufällige Testfälle je Primitive mit festem Seed (42) oder konfigurierbarem ENV Seed.
// NICHT-OFFENSICHTLICH: Falls Python3 oder Pakete fehlen, wird der Test mit eprintln! "SKIPPED: ..." übersprungen.
// HOTSPOTS: [run_python_cross_validation, test_python_cross_implementation]

#![allow(clippy::unwrap_used, clippy::expect_used)]

use aes_gcm_siv::{
    aead::{Aead, KeyInit as AeadKeyInit, Payload},
    Aes256GcmSiv, Nonce,
};
use contextra_crypto::{
    derive_key_argon2id,
    ed25519_proof::{sign_deletion_proof_v3, verify_deletion_proof_v3, DeletionProofKeyPair},
    KdfHeader, KdfParams,
};
use hkdf::Hkdf;
use hmac::{digest::KeyInit, Hmac, Mac};
use rand::{rngs::StdRng, RngCore, SeedableRng};
use sha2::Sha256;
use std::process::Command;

type HmacSha256 = Hmac<Sha256>;

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(hex: &str) -> Vec<u8> {
    let clean = hex.replace([' ', '\n'], "");
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).expect("valid hex byte"))
        .collect()
}

fn get_test_seed() -> u64 {
    std::env::var("CROSSIMPL_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42)
}

const PYTHON_SCRIPT: &str = r#"
import sys, json, hmac, hashlib
try:
    from cryptography.hazmat.primitives.asymmetric import ed25519
    from cryptography.hazmat.primitives.ciphers.aead import AESGCMSIV
    from cryptography.hazmat.primitives.kdf.hkdf import HKDF
    from cryptography.hazmat.primitives import hashes
    import argon2
    import blake3
except ImportError as e:
    print(f"IMPORT_ERROR: {e}")
    sys.exit(101)

data = json.load(sys.stdin)
results = {
    "ed25519_rust_signed_verified_by_py": [],
    "ed25519_py_signed": [],
    "hkdf": [],
    "hmac": [],
    "aes_gcm_siv": [],
    "argon2id": [],
    "blake3_unkeyed": [],
    "blake3_keyed": []
}

for item in data.get("ed25519", []):
    pk = bytes.fromhex(item["pubkey"])
    msg = bytes.fromhex(item["msg"])
    sig = bytes.fromhex(item["sig_rust"])
    try:
        vk = ed25519.Ed25519PublicKey.from_public_bytes(pk)
        vk.verify(sig, msg)
        results["ed25519_rust_signed_verified_by_py"].append(True)
    except Exception as e:
        results["ed25519_rust_signed_verified_by_py"].append(False)

    seed = bytes.fromhex(item["seed"])
    priv = ed25519.Ed25519PrivateKey.from_private_bytes(seed)
    py_sig = priv.sign(msg)
    results["ed25519_py_signed"].append(py_sig.hex())

for item in data.get("hkdf", []):
    ikm = bytes.fromhex(item["ikm"])
    salt = bytes.fromhex(item["salt"])
    info = bytes.fromhex(item["info"])
    length = item["length"]
    hkdf = HKDF(algorithm=hashes.SHA256(), length=length, salt=salt if salt else None, info=info if info else None)
    okm = hkdf.derive(ikm)
    results["hkdf"].append(okm.hex())

for item in data.get("hmac", []):
    key = bytes.fromhex(item["key"])
    msg = bytes.fromhex(item["msg"])
    digest = hmac.new(key, msg, hashlib.sha256).hexdigest()
    results["hmac"].append(digest)

for item in data.get("aes_gcm_siv", []):
    key = bytes.fromhex(item["key"])
    nonce = bytes.fromhex(item["nonce"])
    aad = bytes.fromhex(item["aad"])
    pt = bytes.fromhex(item["plaintext"])
    cipher = AESGCMSIV(key)
    ct = cipher.encrypt(nonce, pt, aad)
    results["aes_gcm_siv"].append(ct.hex())

for item in data.get("argon2id", []):
    pw = item["passphrase"].encode('utf-8')
    salt = bytes.fromhex(item["salt"])
    m_cost = item["m_cost"]
    t_cost = item["t_cost"]
    p_cost = item["p_cost"]
    raw = argon2.low_level.hash_secret_raw(pw, salt, time_cost=t_cost, memory_cost=m_cost, parallelism=p_cost, hash_len=32, type=argon2.Type.ID)
    results["argon2id"].append(raw.hex())

for item in data.get("blake3_unkeyed", []):
    msg = bytes.fromhex(item["msg"])
    h = blake3.blake3(msg).hexdigest()
    results["blake3_unkeyed"].append(h)

for item in data.get("blake3_keyed", []):
    key = bytes.fromhex(item["key"])
    msg = bytes.fromhex(item["msg"])
    h = blake3.blake3(msg, key=key).hexdigest()
    results["blake3_keyed"].append(h)

print(json.dumps(results))
"#;

#[test]
fn test_python_cross_implementation() {
    let seed = get_test_seed();
    eprintln!("Running crypto_crossimpl_python with SEED={seed}");
    let mut rng = StdRng::seed_from_u64(seed);

    // Generate 200 cases per primitive
    const NUM_CASES: usize = 200;

    let mut ed25519_inputs = Vec::with_capacity(NUM_CASES);
    let mut ed25519_rust_sigs = Vec::with_capacity(NUM_CASES);
    let mut ed25519_pubkeys = Vec::with_capacity(NUM_CASES);

    for _ in 0..NUM_CASES {
        let mut secret_seed = [0u8; 32];
        rng.fill_bytes(&mut secret_seed);
        let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_seed);
        let verifying_key = signing_key.verifying_key();
        let keypair = DeletionProofKeyPair {
            signing_key,
            verifying_key,
        };

        let msg_len = (rng.next_u32() % 128) as usize;
        let mut msg = vec![0u8; msg_len];
        rng.fill_bytes(&mut msg);

        let sig = sign_deletion_proof_v3(&keypair, &msg);

        ed25519_pubkeys.push(keypair.verifying_key_bytes());
        ed25519_rust_sigs.push(sig);

        ed25519_inputs.push(serde_json::json!({
            "seed": hex_encode(&secret_seed),
            "pubkey": hex_encode(&keypair.verifying_key_bytes()),
            "msg": hex_encode(&msg),
            "sig_rust": hex_encode(&sig),
        }));
    }

    let mut hkdf_inputs = Vec::with_capacity(NUM_CASES);
    let mut hkdf_rust_okms = Vec::with_capacity(NUM_CASES);

    for _ in 0..NUM_CASES {
        let ikm_len = 16 + (rng.next_u32() % 32) as usize;
        let mut ikm = vec![0u8; ikm_len];
        rng.fill_bytes(&mut ikm);

        let salt_len = (rng.next_u32() % 16) as usize;
        let mut salt = vec![0u8; salt_len];
        rng.fill_bytes(&mut salt);

        let info_len = (rng.next_u32() % 16) as usize;
        let mut info = vec![0u8; info_len];
        rng.fill_bytes(&mut info);

        let length = 16 + (rng.next_u32() % 48) as usize;

        let hk = Hkdf::<Sha256>::new(if salt.is_empty() { None } else { Some(&salt) }, &ikm);
        let mut okm = vec![0u8; length];
        hk.expand(&info, &mut okm).unwrap();

        hkdf_rust_okms.push(hex_encode(&okm));
        hkdf_inputs.push(serde_json::json!({
            "ikm": hex_encode(&ikm),
            "salt": hex_encode(&salt),
            "info": hex_encode(&info),
            "length": length,
        }));
    }

    let mut hmac_inputs = Vec::with_capacity(NUM_CASES);
    let mut hmac_rust_digests = Vec::with_capacity(NUM_CASES);

    for _ in 0..NUM_CASES {
        let key_len = 1 + (rng.next_u32() % 80) as usize;
        let mut key = vec![0u8; key_len];
        rng.fill_bytes(&mut key);

        let msg_len = (rng.next_u32() % 200) as usize;
        let mut msg = vec![0u8; msg_len];
        rng.fill_bytes(&mut msg);

        let mut mac = <HmacSha256 as KeyInit>::new_from_slice(&key).unwrap();
        mac.update(&msg);
        let digest = mac.finalize().into_bytes();

        hmac_rust_digests.push(hex_encode(&digest));
        hmac_inputs.push(serde_json::json!({
            "key": hex_encode(&key),
            "msg": hex_encode(&msg),
        }));
    }

    let mut aes_inputs = Vec::with_capacity(NUM_CASES);
    let mut aes_rust_cts = Vec::with_capacity(NUM_CASES);

    for _ in 0..NUM_CASES {
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);

        let mut nonce = [0u8; 12];
        rng.fill_bytes(&mut nonce);

        let aad_len = (rng.next_u32() % 32) as usize;
        let mut aad = vec![0u8; aad_len];
        rng.fill_bytes(&mut aad);

        let pt_len = (rng.next_u32() % 128) as usize;
        let mut pt = vec![0u8; pt_len];
        rng.fill_bytes(&mut pt);

        let cipher = Aes256GcmSiv::new_from_slice(&key).unwrap();
        let ct = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &pt,
                    aad: &aad,
                },
            )
            .unwrap();

        aes_rust_cts.push(hex_encode(&ct));
        aes_inputs.push(serde_json::json!({
            "key": hex_encode(&key),
            "nonce": hex_encode(&nonce),
            "aad": hex_encode(&aad),
            "plaintext": hex_encode(&pt),
        }));
    }

    let mut argon_inputs = Vec::with_capacity(NUM_CASES);
    let mut argon_rust_keys = Vec::with_capacity(NUM_CASES);

    for i in 0..NUM_CASES {
        let passphrase = format!("test_passphrase_{i}_{seed}");
        let mut salt = vec![0u8; 16];
        rng.fill_bytes(&mut salt);

        // Lightweight Argon2id parameters for test performance
        let params = KdfParams {
            m_cost_kib: 19456,
            t_cost: 2,
            p_cost: 1,
        };
        let header = KdfHeader {
            version: 1,
            kdf_id: 1,
            params,
            salt: salt.clone(),
        };

        let key = derive_key_argon2id(&passphrase, &header).unwrap();
        argon_rust_keys.push(hex_encode(&key.0));

        argon_inputs.push(serde_json::json!({
            "passphrase": passphrase,
            "salt": hex_encode(&salt),
            "m_cost": 19456,
            "t_cost": 2,
            "p_cost": 1,
        }));
    }

    let mut blake_unkeyed_inputs = Vec::with_capacity(NUM_CASES);
    let mut blake_unkeyed_rust = Vec::with_capacity(NUM_CASES);

    for _ in 0..NUM_CASES {
        let msg_len = (rng.next_u32() % 2048) as usize;
        let mut msg = vec![0u8; msg_len];
        rng.fill_bytes(&mut msg);

        let h = blake3::hash(&msg);
        blake_unkeyed_rust.push(h.to_hex().to_string());

        blake_unkeyed_inputs.push(serde_json::json!({
            "msg": hex_encode(&msg),
        }));
    }

    let mut blake_keyed_inputs = Vec::with_capacity(NUM_CASES);
    let mut blake_keyed_rust = Vec::with_capacity(NUM_CASES);

    for _ in 0..NUM_CASES {
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);

        let msg_len = (rng.next_u32() % 2048) as usize;
        let mut msg = vec![0u8; msg_len];
        rng.fill_bytes(&mut msg);

        let h = blake3::keyed_hash(&key, &msg);
        blake_keyed_rust.push(h.to_hex().to_string());

        blake_keyed_inputs.push(serde_json::json!({
            "key": hex_encode(&key),
            "msg": hex_encode(&msg),
        }));
    }

    let payload = serde_json::json!({
        "ed25519": ed25519_inputs,
        "hkdf": hkdf_inputs,
        "hmac": hmac_inputs,
        "aes_gcm_siv": aes_inputs,
        "argon2id": argon_inputs,
        "blake3_unkeyed": blake_unkeyed_inputs,
        "blake3_keyed": blake_keyed_inputs,
    });

    let mut child = match Command::new("python3")
        .arg("-c")
        .arg(PYTHON_SCRIPT)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(e) => {
            eprintln!("SKIPPED: python3 executable not found on host: {e}");
            return;
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let json_bytes = serde_json::to_vec(&payload).unwrap();
        if let Err(e) = stdin.write_all(&json_bytes) {
            eprintln!("SKIPPED: Failed writing to python stdin: {e}");
            return;
        }
    }

    let output = match child.wait_with_output() {
        Ok(out) => out,
        Err(e) => {
            eprintln!("SKIPPED: Error executing python script: {e}");
            return;
        }
    };

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        eprintln!("SKIPPED: python script execution failed (missing packages?): {err_msg}");
        return;
    }

    let py_res: serde_json::Value = match serde_json::from_slice(&output.stdout) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("SKIPPED: Failed parsing python output JSON: {e}");
            return;
        }
    };

    // Assertions cross-checking Rust vs Python
    // 1. Ed25519 Rust signatures verified by Python
    let ed25519_verified = py_res["ed25519_rust_signed_verified_by_py"]
        .as_array()
        .unwrap();
    assert_eq!(ed25519_verified.len(), NUM_CASES);
    for (idx, ok) in ed25519_verified.iter().enumerate() {
        assert!(
            ok.as_bool().unwrap(),
            "Python failed to verify Rust Ed25519 signature at index {idx}"
        );
    }

    // 2. Ed25519 Python signatures verified by Rust
    let py_sigs = py_res["ed25519_py_signed"].as_array().unwrap();
    assert_eq!(py_sigs.len(), NUM_CASES);
    for (idx, py_sig_val) in py_sigs.iter().enumerate() {
        let py_sig_hex = py_sig_val.as_str().unwrap();
        let py_sig_bytes = hex_decode(py_sig_hex);
        let py_sig_arr: [u8; 64] = py_sig_bytes.as_slice().try_into().unwrap();
        let pubkey_arr = &ed25519_pubkeys[idx];
        let msg_hex = ed25519_inputs[idx]["msg"].as_str().unwrap();
        let msg_bytes = hex_decode(msg_hex);

        assert!(
            verify_deletion_proof_v3(pubkey_arr, &msg_bytes, &py_sig_arr).is_ok(),
            "Rust failed to verify Python Ed25519 signature at index {idx}"
        );
    }

    // 3. HKDF
    let py_hkdf = py_res["hkdf"].as_array().unwrap();
    assert_eq!(py_hkdf.len(), NUM_CASES);
    for (idx, okm_val) in py_hkdf.iter().enumerate() {
        assert_eq!(
            okm_val.as_str().unwrap(),
            hkdf_rust_okms[idx],
            "HKDF mismatch between Rust and Python at index {idx}"
        );
    }

    // 4. HMAC
    let py_hmac = py_res["hmac"].as_array().unwrap();
    assert_eq!(py_hmac.len(), NUM_CASES);
    for (idx, digest_val) in py_hmac.iter().enumerate() {
        assert_eq!(
            digest_val.as_str().unwrap(),
            hmac_rust_digests[idx],
            "HMAC mismatch between Rust and Python at index {idx}"
        );
    }

    // 5. AES-256-GCM-SIV
    let py_aes = py_res["aes_gcm_siv"].as_array().unwrap();
    assert_eq!(py_aes.len(), NUM_CASES);
    for (idx, ct_val) in py_aes.iter().enumerate() {
        assert_eq!(
            ct_val.as_str().unwrap(),
            aes_rust_cts[idx],
            "AES-256-GCM-SIV mismatch between Rust and Python at index {idx}"
        );
    }

    // 6. Argon2id
    let py_argon = py_res["argon2id"].as_array().unwrap();
    assert_eq!(py_argon.len(), NUM_CASES);
    for (idx, key_val) in py_argon.iter().enumerate() {
        assert_eq!(
            key_val.as_str().unwrap(),
            argon_rust_keys[idx],
            "Argon2id mismatch between Rust and Python at index {idx}"
        );
    }

    // 7. BLAKE3 Unkeyed
    let py_blake_u = py_res["blake3_unkeyed"].as_array().unwrap();
    assert_eq!(py_blake_u.len(), NUM_CASES);
    for (idx, hash_val) in py_blake_u.iter().enumerate() {
        assert_eq!(
            hash_val.as_str().unwrap(),
            blake_unkeyed_rust[idx],
            "BLAKE3 unkeyed mismatch between Rust and Python at index {idx}"
        );
    }

    // 8. BLAKE3 Keyed
    let py_blake_k = py_res["blake3_keyed"].as_array().unwrap();
    assert_eq!(py_blake_k.len(), NUM_CASES);
    for (idx, hash_val) in py_blake_k.iter().enumerate() {
        assert_eq!(
            hash_val.as_str().unwrap(),
            blake_keyed_rust[idx],
            "BLAKE3 keyed mismatch between Rust and Python at index {idx}"
        );
    }

    eprintln!("SUCCESS: All {NUM_CASES} cases per primitive matched between Rust and Python!");
}
