//! Standalone-Beispiel fuer den Einsatz von contextra-erasure-verify (Baukasten-Vertrag M10).
//!
//! Zeigt die unabhaengige Pruefung eines Loeschberichts sowie den unabhaengigen Feldscan
//! mit Positivkontrolle.

use contextra_erasure_verify::codes::InvalidCode;
use contextra_erasure_verify::report::{hex_encode, parse_public_key, verify_report_bytes};
use contextra_erasure_verify::scan::{execute_field_scan, FieldScanOptions};
use ed25519_dalek::{Signer, SigningKey};
use rand::rngs::OsRng;
use std::fs::File;
use std::io::Write;
use tempfile::tempdir;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Contextra Erasure Verify Standalone Example ===");

    // 1. Schluesselpaar generieren
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    let vk_bytes = verifying_key.to_bytes();
    let parsed_vk = parse_public_key(&vk_bytes)?;

    // 2. Einen minimalen, kanonischen Loeschbericht erstellen
    let fp = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(verifying_key.as_bytes());
        let d = h.finalize();
        hex_encode(&d[0..16])
    };

    let unsigned_val = serde_json::json!({
        "bytes_rewritten": 0,
        "classes": { "lsm": "Purged" },
        "commitment": "c0ffee",
        "completed_at": 1000,
        "erasure_id": "era-1",
        "field_measurement": [{
            "class": "M",
            "control_ok": true,
            "duration_us": 12,
            "hash": "abc",
            "layer": "store",
            "matches": 0,
            "method_id": "raw_scan",
            "scanned_bytes": 1024,
            "scanned_files": 1
        }],
        "gated_at": 950,
        "key_coverage": { "store": "Covered" },
        "key_destroyed_at": 980,
        "lab_evidence": "verified",
        "legal_notice": "no-legal-advice",
        "process_hardening": "Enabled",
        "pub_fingerprint": fp,
        "report_version": 5,
        "requested_at": 900,
        "retention_code": "RET-01",
        "residual_risks": [
            "FlashRemanence",
            "FreedBlockRemanence",
            "ReplicaWindow",
            "ExternalCopies",
            "ExternalProcessors",
            "PseudonymCommitment",
            "KeyCompromise"
        ],
        "segments_rewritten": 0,
        "signature_scope_notice": "scope",
        "sla_us": 100000,
        "sla_violation": false,
        "store_id": "store-1",
        "target_type": "Document"
    });

    let unsigned_canonical =
        contextra_erasure_verify::canonical_json::to_canonical_json_string(&unsigned_val)?;

    let mut message = Vec::new();
    message.extend_from_slice(b"ctx/report/v1");
    message.extend_from_slice(unsigned_canonical.as_bytes());
    let sig = signing_key.sign(&message);
    let sig_hex = hex_encode(&sig.to_bytes());

    let mut signed_map = unsigned_val.as_object().unwrap().clone();
    signed_map.insert("signature".to_string(), serde_json::Value::String(sig_hex));
    let signed_val = serde_json::Value::Object(signed_map);
    let signed_canonical =
        contextra_erasure_verify::canonical_json::to_canonical_json_string(&signed_val)?;

    // 3. Bericht unabhaengig verifizieren
    verify_report_bytes(signed_canonical.as_bytes(), &parsed_vk)?;
    println!("✓ Loeschbericht erfolgreich und strikt verifiziert (Ed25519 verify_strict).");

    // 4. Unabhaengigen Feldscan durchfuehren (MK1, MK2, MK3)
    let dir = tempdir()?;
    let canary_needle = b"CANARY_POSITIVE_CONTROL";
    let target_tag_hex = "0102030405060708090a0b0c0d0e0f10";

    // Canary-Datei erstellen (Positivkontrolle)
    let canary_file_path = dir.path().join("canary.dat");
    let mut cf = File::create(&canary_file_path)?;
    cf.write_all(canary_needle)?;

    let scan_opts = FieldScanOptions {
        data_dir: dir.path(),
        target_tag_hex,
        control_needle: Some(canary_needle),
        passphrase_fd: None,
    };

    match execute_field_scan(scan_opts) {
        Ok(()) => println!("✓ Unabhaengiger Feldscan erfolgreich: Positivkontrolle bestanden, keine Zielreste gefunden."),
        Err(code) => return Err(format!("Feldscan unerwartet fehlgeschlagen: {:?}", code).into()),
    }

    // 5. Sabotage-Test: Wenn keine Positivkontrolle existiert, MUSS der Scan abbrechen
    let empty_dir = tempdir()?;
    let fail_opts = FieldScanOptions {
        data_dir: empty_dir.path(),
        target_tag_hex,
        control_needle: Some(canary_needle),
        passphrase_fd: None,
    };

    match execute_field_scan(fail_opts) {
        Err(InvalidCode::MissingPositiveControl) => {
            println!(
                "✓ Sabotage-Schutz: Negativbefund ohne Positivkontrolle wurde korrekt abgewiesen."
            );
        }
        other => {
            return Err(format!(
                "Unerwartetes Ergebnis bei fehlender Positivkontrolle: {:?}",
                other
            )
            .into())
        }
    }

    println!("=== Alle Verifikationsdemonstrationen erfolgreich ===");
    Ok(())
}
