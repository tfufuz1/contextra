#![no_main]

use contextra_lab::format::sstable::SsTableFile;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = SsTableFile::parse(data);
});
