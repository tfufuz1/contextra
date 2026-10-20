use contextra_ports::{EntropyRng, SecretBytes};

struct MockEntropy;
impl EntropyRng for MockEntropy {
    fn fill_bytes(&self, dest: &mut [u8]) {
        dest.fill(1);
    }
    fn next_u64(&self) -> u64 {
        1
    }
    fn fill_secret(&self, dest: &mut [u8]) -> Result<(), contextra_ports::RngError> {
        dest.fill(1);
        Ok(())
    }
}

fn main() {
    let s = SecretBytes::<32>::generate(&MockEntropy).unwrap();
    let _cloned = s.clone();
}
