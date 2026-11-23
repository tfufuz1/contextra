use contextra_storage::codec::checksums::crc32c;
use contextra_storage::codec::header::FileHeader;
use contextra_storage::env::memfs::MemFs;
use contextra_storage::store::run::builder::RunFileBuilder;
use contextra_storage::store::run::copy::{copy_sections, RawSectionChecksumVerifier};
use contextra_storage::store::run::reader::RunReader;
use contextra_storage::store::run::table_cache::TableCache;
use contextra_storage_ports::traits::{FileRole, Fs, IoCtx};
use contextra_storage_ports::{ClusterId, Error};
use std::path::Path;

const IO: IoCtx = IoCtx { role: FileRole::Run, purpose: "run_copy_sections_test" };

struct CrcAtTail;
impl RawSectionChecksumVerifier for CrcAtTail {
    fn verify_raw_checksums(&self, section: &[u8]) -> Result<(), Error> {
        let split = section.len().checked_sub(4).ok_or(Error::Corrupt { place: "test section CRC" })?;
        let crc = u32::from_le_bytes(section.get(split..).ok_or(Error::Corrupt { place: "test section CRC" })?
            .try_into().map_err(|_| Error::Corrupt { place: "test section CRC" })?);
        if crc32c(section.get(..split).ok_or(Error::Corrupt { place: "test section CRC input" })?) != crc {
            return Err(Error::Corrupt { place: "test section CRC" });
        }
        Ok(())
    }
}

fn header(file_id: u64) -> FileHeader {
    FileHeader { magic: *b"CTXRUN01", file_kind: 3, format_ver: 1, feature_bits: 0, file_id, instance_id: [1; 16] }
}

fn crc_section(payload: &[u8]) -> Vec<u8> {
    let mut bytes = payload.to_vec();
    bytes.extend_from_slice(&crc32c(payload).to_le_bytes());
    bytes
}

#[test]
fn selected_sections_are_verified_then_copied_byte_for_byte() {
    let fs = MemFs::new();
    let first = crc_section(b"first-sealed-cluster-section");
    let survivor = crc_section(b"survivor-sealed-cluster-section");
    let mut source_builder = RunFileBuilder::create(&fs, Path::new("/runs/source.building"), &header(1)).unwrap();
    source_builder.append_section(ClusterId::from_wire([1; 16]), &first, 1, 2, 1, 2).unwrap();
    source_builder.append_section(ClusterId::from_wire([2; 16]), &survivor, 3, 9, 4, 12).unwrap();
    source_builder.finish(b"pack", &[]).unwrap();
    fs.sync_dir(Path::new("/runs"), IO).unwrap();

    let cache = TableCache::new(4).unwrap();
    let source = RunReader::open(&fs, &cache, Path::new("/runs/source.building"), *b"CTXRUN01", 3, 0, 8).unwrap();
    let mut destination = RunFileBuilder::create(&fs, Path::new("/runs/copy.building"), &header(2)).unwrap();
    copy_sections(&source, &[ClusterId::from_wire([2; 16])], &mut destination, 1024, &CrcAtTail).unwrap();
    destination.finish(b"pack", &[]).unwrap();
    fs.sync_dir(Path::new("/runs"), IO).unwrap();

    let copied = RunReader::open(&fs, &cache, Path::new("/runs/copy.building"), *b"CTXRUN01", 3, 0, 8).unwrap();
    assert_eq!(copied.directory().entries().len(), 1);
    assert_eq!(copied.read_section(ClusterId::from_wire([2; 16]), 1024).unwrap().unwrap().as_slice(), survivor);
    assert!(copied.read_section(ClusterId::from_wire([1; 16]), 1024).unwrap().is_none());
}

#[test]
fn copy_aborts_on_checksum_failure_or_unsorted_cluster_selection() {
    let fs = MemFs::new();
    let invalid = b"bad-crc";
    let mut source_builder = RunFileBuilder::create(&fs, Path::new("/runs/source.building"), &header(1)).unwrap();
    source_builder.append_section(ClusterId::from_wire([1; 16]), invalid, 1, 1, 1, 1).unwrap();
    source_builder.append_section(ClusterId::from_wire([2; 16]), invalid, 1, 1, 1, 1).unwrap();
    source_builder.finish(b"pack", &[]).unwrap();
    fs.sync_dir(Path::new("/runs"), IO).unwrap();
    let cache = TableCache::new(4).unwrap();
    let source = RunReader::open(&fs, &cache, Path::new("/runs/source.building"), *b"CTXRUN01", 3, 0, 8).unwrap();
    let mut destination = RunFileBuilder::create(&fs, Path::new("/runs/failed.building"), &header(3)).unwrap();
    assert!(copy_sections(&source, &[ClusterId::from_wire([1; 16])], &mut destination, 1024, &CrcAtTail).is_err());
    assert!(copy_sections(&source, &[ClusterId::from_wire([2; 16]), ClusterId::from_wire([1; 16])],
        &mut destination, 1024, &CrcAtTail).is_err());
}
