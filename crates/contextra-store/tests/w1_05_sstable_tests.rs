use bytes::Bytes;
use contextra_store::sstable::{create_block_cache, SstableBuilder, SstableReader};
use tempfile::TempDir;

#[test]
fn sstable_reader_does_not_block_current_thread_runtime() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio current_thread runtime");

    rt.block_on(async {
        let temp_dir = TempDir::new().expect("tempdir");
        let sst_path = temp_dir.path().join("test_sstable.sst");

        let mut builder = SstableBuilder::create(&sst_path)
            .await
            .expect("SstableBuilder create");

        for i in 0..10u64 {
            let key = format!("key_{:04}", i);
            let val = format!("val_{:04}", i);
            builder
                .add(key.as_bytes(), val.as_bytes(), i + 1, i + 1)
                .await
                .expect("builder add");
        }

        builder.finish().await.expect("builder finish");

        let block_cache = create_block_cache(1);
        let reader = SstableReader::open(&sst_path, block_cache)
            .await
            .expect("SstableReader open");

        let res = reader
            .get(b"key_0005")
            .await
            .expect("get")
            .expect("key_0005 found");

        assert_eq!(res.0, Bytes::from("val_0005"));
        assert_eq!(res.1, 6);
        assert_eq!(res.2, 6);
    });
}
