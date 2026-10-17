#![no_main]

use libfuzzer_sys::fuzz_target;
use contextra_store::sstable::WitnessReader;
use std::io::Write;

fuzz_target!(|data: &[u8]| {
    if data.len() > 2 * 1024 * 1024 {
        return;
    }

    let mut temp_file = match tempfile::NamedTempFile::new() {
        Ok(f) => f,
        Err(_) => return,
    };

    if temp_file.write_all(data).is_err() {
        return;
    }

    if temp_file.flush().is_err() {
        return;
    }

    if let Ok(mut reader) = WitnessReader::open(temp_file.path(), None) {
        let _ = reader.read_first_key();
        let _ = reader.read_all_entries();
    }
});
