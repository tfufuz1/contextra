#![cfg(any())]

use contextra_store::wal::WalEntry;

#[test]
fn test_wal_v4_fuzz_decoder_zero_panic_arbitrary_bytes() {
    let inputs: Vec<Vec<u8>> = vec![
        vec![],
        vec![0],
        vec![0, 1, 2, 3],
        vec![0; 100],
        vec![0xFF; 256],
        b"MFW4_RANDOM_GARBAGE_PAYLOAD".to_vec(),
    ];

    for input in inputs {
        let _ = WalEntry::from_bytes_classified(&input);
    }

    use rand::RngCore;
    let mut rng = rand::thread_rng();
    for _ in 0..10_000 {
        let len = (rng.next_u32() % 512) as usize;
        let mut buf = vec![0u8; len];
        rng.fill_bytes(&mut buf);
        let _ = WalEntry::from_bytes_classified(&buf);
    }
}
