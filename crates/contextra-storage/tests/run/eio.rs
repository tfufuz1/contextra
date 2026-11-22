//! # EIO-Fehlertests mit FaultFs (SPEC §4, F-6, INV-FMT-2)
//!
//! EIO beim Lesen wird immer als Fehler gemeldet, nie als leer (F-6).

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::panic)]

use contextra_storage::codec::tag::{Tag, TagKind};
use contextra_storage::run::block::BlockSecurity;
use contextra_storage::run::builder::{RunBuilder, RunBuilderOptions};
use contextra_storage::run::error::RunError;
use contextra_storage::run::reader::RunReader;
use contextra_storage::testing::fault_fs::{FaultConfig, FaultFs};
use contextra_storage::testing::mem_fs::MemFs;
use contextra_storage_ports::{File, Fs, OpenMode, Perm};
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[test]
fn test_eio_reported_as_error_never_empty() {
    let mem_fs = MemFs::new();
    let fault_config = Arc::new(FaultConfig::default());
    let fault_fs = FaultFs::new(mem_fs, Arc::clone(&fault_config));

    let run_path = Path::new("/run_001.sst");

    // 1. Run mit Testdaten bauen
    let opts = RunBuilderOptions {
        space_id: 1,
        run_id: 888,
        header_len: 0,
        weights_k: 1,
        prefix_filter_len: Some(3),
        created_ms: 5000,
    };
    let mut builder = RunBuilder::new(opts, BlockSecurity::Plain);
    builder.add(b"key_a", Tag::new(1, TagKind::Put), b"val_a").unwrap();
    builder.add(b"key_b", Tag::new(2, TagKind::Put), b"val_b").unwrap();
    let run_bytes = builder.finish().unwrap();

    // In Datei schreiben
    let file = fault_fs.open(run_path, OpenMode::CreateNew(Perm::Private)).unwrap();
    file.write_all_at(&run_bytes, 0).unwrap();
    file.sync_all().unwrap();
    drop(file);

    let file_len = run_bytes.len() as u64;

    // 2. Fehlerfall 1: Totaler EIO beim Öffnen
    fault_config.fail_io.store(true, Ordering::Relaxed);
    let open_res = fault_fs.open(run_path, OpenMode::Read);
    assert!(open_res.is_err(), "Open muss mit EIO fehlschlagen");
    fault_config.fail_io.store(false, Ordering::Relaxed);

    // 3. Fehlerfall 2: Öffnen gelingt, aber Datenblock-Lesen schlägt fehl mit EIO
    let opened_file: Arc<dyn File> = Arc::from(fault_fs.open(run_path, OpenMode::Read).unwrap());
    let reader = RunReader::open(opened_file, file_len, BlockSecurity::Plain, None).unwrap();

    // Jetzt EIO einspritzen
    fault_config.fail_io.store(true, Ordering::Relaxed);

    // F-6: Muss zwingend Err liefern, NIEMALS Ok(None) ("nie als leer")!
    let get_res = reader.get(b"key_a");
    match get_res {
        Err(RunError::Io(err)) => {
            assert_eq!(err.to_string(), "Injected EIO");
        }
        other => {
            panic!("F-6 verletzt: Erwartet Err(Io), erhalten: {:?}", other);
        }
    }
}
