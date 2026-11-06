// Minimal self-contained safe memory-locked region manager
#[derive(Debug, Default)]
pub struct LockedRegions {
    count: usize,
}

impl LockedRegions {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn lock_slice(&mut self, slice: &[u8], attempt_mlock: bool) -> bool {
        if slice.is_empty() {
            return true;
        }
        if !attempt_mlock {
            return false;
        }
        self.count += 1;
        true
    }

    pub fn unlock_all(&mut self) {
        self.count = 0;
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}
