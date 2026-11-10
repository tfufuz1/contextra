use contextra_prompt_guard::PromptInjectionGuard;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_golden_file_truncation_mutation() {
    let original = include_str!("golden/corpus_inputs.json");

    // Deterministische Truncation an verschiedenen Positionen
    let truncation_points = [10, 50, 100, 200, 500];

    for &len in &truncation_points {
        if len >= original.len() {
            continue;
        }
        let truncated = &original[..len];
        let mut tmp_file = NamedTempFile::new().expect("Failed to create tempfile");
        tmp_file
            .write_all(truncated.as_bytes())
            .expect("Failed to write truncated content");

        // Öffnen/Laden muss mit Err(GuardError::...) terminieren, NIE mit Panic
        let res = PromptInjectionGuard::load_from_file(tmp_file.path());
        assert!(
            res.is_err(),
            "Truncated golden JSON must fail gracefully with Err"
        );
    }
}

#[test]
fn test_golden_file_bitflip_mutation() {
    let original = include_str!("../../contextra-mcp/tests/golden/corpus_inputs.json");
    let mut bytes = original.as_bytes().to_vec();

    // Deterministische Bitflips an strukturellen JSON-Positionen
    let flip_indices = [0, 1, 15, 30, 45, 60];

    for &idx in &flip_indices {
        if idx < bytes.len() {
            bytes[idx] ^= 0xFF; // Flip all bits of this byte

            let mut tmp_file = NamedTempFile::new().expect("Failed to create tempfile");
            tmp_file
                .write_all(&bytes)
                .expect("Failed to write bitflipped content");

            let res = PromptInjectionGuard::load_from_file(tmp_file.path());
            assert!(
                res.is_err(),
                "Corrupted golden JSON must fail gracefully with Err without panicking"
            );

            // Zurücksetzen für nächsten deterministischen Test
            bytes[idx] ^= 0xFF;
        }
    }
}
