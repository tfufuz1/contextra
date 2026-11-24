use contextra_storage::codec::header::FileHeader;
use contextra_storage::env::memfs::MemFs;
use contextra_storage::store::run::builder::RunFileBuilder;
use contextra_storage::store::run::directory::ClusterDirectory;
use contextra_storage::store::run::format::{RunFooter, RUN_FOOTER_LEN};
use contextra_storage::store::run::extension::ExtensionBlock;
use contextra_storage::store::run::reader::RunReader;
use contextra_storage::store::run::table_cache::TableCache;
use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::ClusterId;
use std::path::Path;

const IO: IoCtx = IoCtx { role: FileRole::Run, purpose: "run_file_builder_test" };

fn header() -> FileHeader {
    FileHeader { magic: *b"CTXRUN01", file_kind: 3, format_ver: 1, feature_bits: 0,
        file_id: 22, instance_id: [4; 16] }
}

#[test]
fn physical_builder_writes_sections_directory_pack_extensions_and_footer() {
    let fs = MemFs::new();
    let mut builder = RunFileBuilder::create(&fs, Path::new("/runs/22.building"), &header()).unwrap();
    builder.append_section(ClusterId::from_wire([1; 16]), b"sealed-section-one", 2, 10, 1, 9).unwrap();
    builder.append_section(ClusterId::from_wire([2; 16]), b"sealed-section-two", 1, 4, 3, 7).unwrap();
    assert!(builder.append_section(ClusterId::from_wire([2; 16]), b"duplicate", 1, 1, 1, 1).is_err());
    let ext = [ExtensionBlock::new(7, 1, vec![3, 2, 1]).unwrap()];
    let built = builder.finish(b"opaque-pack-meta", &ext).unwrap();
    fs.sync_dir(Path::new("/runs"), IO).unwrap();
    assert_eq!(built.directory.entries().len(), 2);
    assert_eq!(built.file_bytes, fs.open(Path::new("/runs/22.building"), IO).unwrap().size().unwrap());

    let cache = TableCache::new(2).unwrap();
    let reader = RunReader::open(&fs, &cache, Path::new("/runs/22.building"), *b"CTXRUN01", 3, 0, 8).unwrap();
    assert_eq!(reader.header(), &header());
    assert_eq!(reader.pack_meta(), b"opaque-pack-meta");
    assert_eq!(reader.extensions(), ext);
    assert_eq!(reader.read_section(ClusterId::from_wire([2; 16]), 1024).unwrap().unwrap().as_slice(), b"sealed-section-two");
    assert!(reader.read_section(ClusterId::from_wire([9; 16]), 1024).unwrap().is_none());

    let crashed = fs.crash_image().unwrap();
    let file = crashed.open(Path::new("/runs/22.building"), IO).unwrap();
    let mut bytes = vec![0; usize::try_from(file.size().unwrap()).unwrap()];
    let mut offset = 0usize;
    while offset < bytes.len() {
        let target = bytes.get_mut(offset..).unwrap();
        let read = file.read_at(offset as u64, target, IO).unwrap();
        assert_ne!(read, 0);
        offset += read;
    }
    let footer_start = bytes.len() - RUN_FOOTER_LEN;
    let footer = RunFooter::decode(&bytes[footer_start..], bytes.len() as u64).unwrap();
    let directory_start = usize::try_from(footer.directory_offset).unwrap();
    let directory_end = directory_start + usize::try_from(footer.directory_len).unwrap();
    let directory = ClusterDirectory::decode(&bytes[directory_start..directory_end], 8).unwrap();
    assert_eq!(directory, built.directory);
}

#[test]
fn empty_run_is_a_valid_durable_file() {
    let fs = MemFs::new();
    let builder = RunFileBuilder::create(&fs, Path::new("/runs/empty.building"), &header()).unwrap();
    let built = builder.finish(b"empty-pack", &[]).unwrap();
    fs.sync_dir(Path::new("/runs"), IO).unwrap();
    assert!(built.directory.entries().is_empty());
    let crashed = fs.crash_image().unwrap();
    assert_eq!(crashed.open(Path::new("/runs/empty.building"), IO).unwrap().size().unwrap(), built.file_bytes);
}
