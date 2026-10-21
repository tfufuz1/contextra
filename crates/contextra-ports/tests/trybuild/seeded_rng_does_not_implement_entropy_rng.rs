use contextra_ports::{SecretBytes, SeededRng};

struct MockSeededRng;
impl SeededRng for MockSeededRng {
    fn next_u64(&self) -> u64 {
        42
    }
    fn fill_bytes(&self, dest: &mut [u8]) {
        dest.fill(0x55);
    }
    fn next_unit_f64(&self) -> f64 {
        0.5
    }
}

fn main() {
    let rng = MockSeededRng;
    // MockSeededRng does not implement EntropyRng, so this should fail compilation.
    let _ = SecretBytes::<32>::generate(&rng);
}
