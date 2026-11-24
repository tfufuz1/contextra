//! Deterministic simulation environment (SimClock, SimEntropy) (SPEC §3.1, §3.8, SP-ENV).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::{Clock, ClockReading, ClockSkew, Entropy};
use contextra_storage_ports::Error;
use std::sync::Mutex;

pub struct SimClock {
    mono_nanos: Mutex<u64>,
    utc_millis: Mutex<u64>,
    durable_utc_floor: Mutex<u64>,
    skew: Mutex<ClockSkew>,
}

impl SimClock {
    pub fn new(initial_utc_millis: u64) -> Self {
        Self {
            mono_nanos: Mutex::new(0),
            utc_millis: Mutex::new(initial_utc_millis),
            durable_utc_floor: Mutex::new(initial_utc_millis),
            skew: Mutex::new(ClockSkew::Certain),
        }
    }

    pub fn advance_mono_nanos(&self, delta: u64) {
        if let Ok(mut m) = self.mono_nanos.lock() {
            *m = m.saturating_add(delta);
        }
    }

    pub fn advance_utc_millis(&self, delta: u64) {
        if let Ok(mut u) = self.utc_millis.lock() {
            *u = u.saturating_add(delta);
        }
    }

    pub fn set_skew(&self, skew: ClockSkew) {
        if let Ok(mut s) = self.skew.lock() {
            *s = skew;
        }
    }
}

impl Clock for SimClock {
    fn read(&self) -> Result<ClockReading, Error> {
        let mono = *self.mono_nanos.lock().map_err(|_| Error::Poisoned {
            cause: "sim clock lock",
        })?;
        let utc = *self.utc_millis.lock().map_err(|_| Error::Poisoned {
            cause: "sim clock lock",
        })?;
        let floor = *self.durable_utc_floor.lock().map_err(|_| Error::Poisoned {
            cause: "sim clock lock",
        })?;
        let skew = *self.skew.lock().map_err(|_| Error::Poisoned {
            cause: "sim clock lock",
        })?;
        Ok(ClockReading {
            monotonic_nanos: mono,
            utc_millis: utc,
            durable_utc_millis: floor,
            skew,
        })
    }

    fn advance_utc_floor(&self, expect_millis: u64, next_millis: u64) -> Result<(), Error> {
        let mut floor = self.durable_utc_floor.lock().map_err(|_| Error::Poisoned {
            cause: "sim clock lock",
        })?;
        if *floor != expect_millis {
            return Err(Error::CasLost);
        }
        if next_millis < expect_millis {
            return Err(Error::Limit {
                limit: "clock floor must advance monotonically",
            });
        }
        *floor = next_millis;
        Ok(())
    }
}

pub struct SimEntropy {
    counter: Mutex<u64>,
}

impl SimEntropy {
    pub fn new(seed: u64) -> Self {
        Self {
            counter: Mutex::new(seed),
        }
    }
}

impl Default for SimEntropy {
    fn default() -> Self {
        Self::new(1)
    }
}

impl Entropy for SimEntropy {
    fn fill(&self, buf: &mut [u8]) -> Result<(), Error> {
        let mut count = self.counter.lock().map_err(|_| Error::Poisoned {
            cause: "sim entropy lock",
        })?;
        for chunk in buf.chunks_mut(8) {
            *count = count.wrapping_mul(6364136223846793005).wrapping_add(1);
            let bytes = count.to_be_bytes();
            let len = chunk.len();
            chunk.copy_from_slice(&bytes[..len]);
        }
        Ok(())
    }
}
