#![no_main]

use contextra_lab::format::record_envelope::RecordEnvelopeV2;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = RecordEnvelopeV2::parse(data);
});
