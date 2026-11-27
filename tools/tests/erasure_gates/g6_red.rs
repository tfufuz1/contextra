pub trait ReadGate: Send + Sync {
    fn is_active(&self) -> bool {} //~
    fn epoch(&self) -> u64;
}

impl ReadGate for Partial { //~
    fn is_active(&self) -> bool {
        false
    }
}
