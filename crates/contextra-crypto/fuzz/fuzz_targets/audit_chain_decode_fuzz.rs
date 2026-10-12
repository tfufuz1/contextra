#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = contextra_crypto::audit_chain::AuditChainEntry::from_bytes(data);
});
