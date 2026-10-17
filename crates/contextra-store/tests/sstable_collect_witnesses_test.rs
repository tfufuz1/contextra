use contextra_store::sstable::{collect_witnesses, SstableBuilder};
use tempfile::TempDir;

#[tokio::test]
async fn test_collect_witnesses_facade_integration() -> Result<(), Box<dyn std::error::Error>> {
    let tmp = TempDir::new()?;
    let sst_path = tmp.path().join("sst-001.sst");

    let mut builder = SstableBuilder::create(&sst_path).await?;
    builder.add(b"user:100:name", b"Alice", 1, 1).await?;
    builder.add(b"user:101:name", b"Bob", 2, 2).await?;
    builder.finish().await?;

    let live_files = vec![sst_path.clone()];

    let evidence = collect_witnesses(
        tmp.path(),
        live_files,
        1,
        10,
        vec![b"user:100:name".to_vec()],
        vec![b"user:".to_vec()],
        None,
    )?;

    assert_eq!(evidence.files_in_manifest, 1);
    assert_eq!(evidence.files_scanned, 1);
    assert!(evidence.versions_found >= 1);

    Ok(())
}
