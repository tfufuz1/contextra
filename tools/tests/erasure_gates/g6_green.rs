pub trait ReadGate: Send + Sync {
    fn is_active(&self, k: [u8; 4]) -> u64;
    fn epoch(&self, k: [u8; 4]) -> u64;
    fn check(&self, k: [u8; 4]) -> u64;
    fn check_version(&self, k: [u8; 4]) -> u64;
    fn check_prefix(&self, k: [u8; 4]) -> u64;
    fn discard_rule(&self, k: [u8; 4]) -> u64;
    fn check_doc(&self, k: [u8; 4]) -> u64;
    fn gated_seq(&self, k: [u8; 4]) -> u64;
}

impl<T: ReadGate + ?Sized> ReadGate for Arc<T> {
    fn is_active(&self, k: [u8; 4]) -> u64 {
        (**self).is_active(k)
    }
    fn epoch(&self, k: [u8; 4]) -> u64 {
        (**self).epoch(k)
    }
    fn check(&self, k: [u8; 4]) -> u64 {
        (**self).check(k)
    }
    fn check_version(&self, k: [u8; 4]) -> u64 {
        (**self).check_version(k)
    }
    fn check_prefix(&self, k: [u8; 4]) -> u64 {
        (**self).check_prefix(k)
    }
    fn discard_rule(&self, k: [u8; 4]) -> u64 {
        (**self).discard_rule(k)
    }
    fn check_doc(&self, k: [u8; 4]) -> u64 {
        (**self).check_doc(k)
    }
    fn gated_seq(&self, k: [u8; 4]) -> u64 {
        (**self).gated_seq(k)
    }
}

fn takes(g: &dyn ReadGate) {}
