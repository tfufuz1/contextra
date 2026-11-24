#![no_main]

use contextra_store::tenant_codec::TenantKeyCodec;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Decoding arbitrary bytes with decode_tenant_id must never panic
    let _ = TenantKeyCodec::decode_tenant_id(data);
});
