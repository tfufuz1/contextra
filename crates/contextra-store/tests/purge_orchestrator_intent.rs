// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Integrationstests für OrxPurgeIntent Persistenz, Checksumme und Korruptionserkennung.

use contextra_store::compaction::purge_filter::PurgeScope;
use contextra_store::lsm::purge::{OrxPurgeIntent, PurgePhase};
use tempfile::tempdir;

#[test]
fn test_intent_roundtrip_and_crc() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    let scope = PurgeScope::Keys(vec![b"key1".to_vec(), b"key2".to_vec()]);
    let intent = OrxPurgeIntent::new([1u8; 16], &scope, 42, Some(100), Some([2u8; 32]), PurgePhase::Admitted);

    // Save intent
    intent.write_to_file(&intent_path)?;

    // Read back and verify equality
    let loaded = OrxPurgeIntent::read_from_file(&intent_path)?;
    assert!(loaded.is_some());
    let loaded_intent = loaded.unwrap();

    assert_eq!(loaded_intent.purge_id, intent.purge_id);
    assert_eq!(loaded_intent.ledger_seq, intent.ledger_seq);
    assert_eq!(loaded_intent.proof_id, intent.proof_id);
    assert_eq!(loaded_intent.scope_hash, intent.scope_hash);
    assert_eq!(loaded_intent.as_of_tx, intent.as_of_tx);
    assert_eq!(loaded_intent.phase, PurgePhase::Admitted);

    Ok(())
}

#[test]
fn test_intent_nonexistent_returns_none() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("missing.intent");

    let loaded = OrxPurgeIntent::read_from_file(&intent_path)?;
    assert!(loaded.is_none());

    Ok(())
}

#[test]
fn test_intent_corruption_returns_error() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("corrupted.intent");

    let scope = PurgeScope::Keys(vec![b"key1".to_vec()]);
    let intent = OrxPurgeIntent::new([1u8; 16], &scope, 10, None, None, PurgePhase::Admitted);
    intent.write_to_file(&intent_path)?;

    // Mutate a byte in the payload to invalidate CRC
    let mut bytes = std::fs::read(&intent_path)?;
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    std::fs::write(&intent_path, bytes)?;

    // Must return Err on corruption, NEVER Ok(None)
    let res = OrxPurgeIntent::read_from_file(&intent_path);
    assert!(res.is_err(), "Corrupted intent must return Err");

    Ok(())
}

#[test]
fn test_intent_truncated_returns_error() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("truncated.intent");

    std::fs::write(&intent_path, b"ORXI_TOO_SHORT")?;

    let res = OrxPurgeIntent::read_from_file(&intent_path);
    assert!(res.is_err(), "Truncated file must return Err");

    Ok(())
}
