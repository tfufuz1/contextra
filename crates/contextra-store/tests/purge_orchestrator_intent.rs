use contextra_store::lsm::purge::{OrxPurgeIntent, PurgePhase};
use tempfile::tempdir;

#[test]
fn test_purge_intent_save_and_load_roundtrip() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    let original = OrxPurgeIntent::new(
        [1; 16],
        [2; 32],
        42,
        Some(100),
        Some("proof-123".to_string()),
        500,
        PurgePhase::Admitted,
    );

    original.save(&intent_path)?;
    assert!(intent_path.exists());

    let loaded = OrxPurgeIntent::load(&intent_path)?;
    assert_eq!(original, loaded);

    Ok(())
}

#[test]
fn test_purge_intent_corrupted_crc_fails() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    let intent = OrxPurgeIntent::new(
        [1; 16],
        [2; 32],
        1,
        None,
        None,
        100,
        PurgePhase::Flush,
    );

    intent.save(&intent_path)?;

    let mut bytes = std::fs::read(&intent_path)?;
    if bytes.len() > 10 {
        bytes[8] ^= 0xFF;
    }
    std::fs::write(&intent_path, &bytes)?;

    let load_res = OrxPurgeIntent::load(&intent_path);
    assert!(load_res.is_err(), "Loading corrupted intent must return Err");

    Ok(())
}

#[test]
fn test_purge_intent_truncated_fails() -> std::io::Result<()> {
    let dir = tempdir()?;
    let intent_path = dir.path().join("purge.intent");

    std::fs::write(&intent_path, b"12")?;

    let load_res = OrxPurgeIntent::load(&intent_path);
    assert!(load_res.is_err(), "Loading truncated intent must return Err");

    Ok(())
}
