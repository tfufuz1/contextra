//! Kooperatives I/O-Budget und Token-Bucket-Ratenbegrenzung (WP-4.0, §7.6).
//!
//! Vereinheitlicht die I/O-Drosselung für reguläre Compaction (`IoClass::Compaction`)
//! und physische Löschung (`IoClass::Erase`), während `IoClass::Interactive` ungedrosselt bleibt.

use crate::system_pressure::{PressureLevel, SystemPressure};
use contextra_ports::clock::Clock;
use contextra_ports::erasure::{IoBudget, IoClass, PressureHint};
use contextra_ports::BoxFuture;
use contextra_types::ErasureError;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

/// Deterministischer Token-Bucket für I/O-Ratenbegrenzung.
///
/// Kapselt die exakte Arithmetik aus der Compaction-Schleife (`compaction/engine.rs`),
/// sodass bestehende Ratenbegrenzungs-Tests (`compaction_io_rate_limit.rs`) verhaltensgleich bleiben.
pub struct TokenBucket {
    bytes_written: u64,
    last_reset_nanos: u64,
    max_bps: u64,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for TokenBucket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenBucket")
            .field("bytes_written", &self.bytes_written)
            .field("last_reset_nanos", &self.last_reset_nanos)
            .field("max_bps", &self.max_bps)
            .finish_non_exhaustive()
    }
}

impl TokenBucket {
    /// Erzeugt einen neuen Token-Bucket mit `max_bps` (0 = unbegrenzt).
    pub fn new(max_bps: u64, clock: Arc<dyn Clock>) -> Self {
        let now = clock.monotonic_nanos();
        Self {
            bytes_written: 0,
            last_reset_nanos: now,
            max_bps,
            clock,
        }
    }

    /// Verbucht `bytes` und liefert eine optionale Wartezeit zurück, falls das Budget überschritten wurde.
    ///
    /// Hält keine Locks über `.await`; der Aufrufer schläft bei `Some(delay)` und ruft danach
    /// [`Self::on_sleep_complete`] auf.
    pub fn charge_step(&mut self, bytes: u64) -> Option<Duration> {
        self.charge_step_with_limit(bytes, self.max_bps)
    }

    /// Wie [`Self::charge_step`], jedoch mit einem dynamischen effektiven Limit `effective_bps`.
    pub fn charge_step_with_limit(&mut self, bytes: u64, effective_bps: u64) -> Option<Duration> {
        if effective_bps == 0 {
            return None;
        }
        self.bytes_written = self.bytes_written.saturating_add(bytes);
        let elapsed_nanos = self
            .clock
            .monotonic_nanos()
            .saturating_sub(self.last_reset_nanos);
        let elapsed = Duration::from_nanos(elapsed_nanos);
        let target = Duration::from_secs_f64(self.bytes_written as f64 / effective_bps as f64);
        if target > elapsed {
            Some((target - elapsed).min(Duration::from_millis(100)))
        } else {
            None
        }
    }

    /// Aktualisiert den Token-Bucket nach Ablauf einer Wartezeit.
    pub fn on_sleep_complete(&mut self) {
        self.on_sleep_complete_with_limit(self.max_bps);
    }

    /// Aktualisiert den Token-Bucket nach Ablauf einer Wartezeit mit effektivem Limit.
    pub fn on_sleep_complete_with_limit(&mut self, effective_bps: u64) {
        if effective_bps == 0 {
            self.last_reset_nanos = self.clock.monotonic_nanos();
            return;
        }
        let now = self.clock.monotonic_nanos();
        let total_elapsed_nanos = now.saturating_sub(self.last_reset_nanos);
        let total_elapsed = Duration::from_nanos(total_elapsed_nanos);
        let allowed_bytes = (total_elapsed.as_secs_f64() * effective_bps as f64) as u64;
        self.bytes_written = self.bytes_written.saturating_sub(allowed_bytes);
        self.last_reset_nanos = now;
    }

    /// Verbucht `bytes` asynchron und wartet bei Bedarf kooperativ.
    pub async fn charge(&mut self, bytes: u64) {
        if let Some(delay) = self.charge_step(bytes) {
            tokio::time::sleep(delay).await;
            self.on_sleep_complete();
        }
    }
}

struct SharedBucketState {
    total_bucket: TokenBucket,
    erase_bucket: TokenBucket,
    last_compaction_waiter_nanos: u64,
    last_erase_waiter_nanos: u64,
    jitter_state: u64,
}

/// Zentrales kooperatives I/O-Budget je Store (`ports::IoBudget`).
///
/// # Fehler
/// `acquire` liefert [`ErasureError::Cancelled`], wenn die Abbruchbedingung eintritt.
/// # Crash
/// Reiner RAM-Zustand ohne persistente Seiteneffekte.
pub struct StoreIoBudget {
    state: Arc<Mutex<SharedBucketState>>,
    clock: Arc<dyn Clock>,
    pressure_rx: watch::Receiver<SystemPressure>,
    max_io_bytes_per_second: u64,
    erase_io_share: f32,
}

impl StoreIoBudget {
    /// Erzeugt ein neues `StoreIoBudget`.
    ///
    /// `max_io_bytes_per_second == 0` bedeutet unbegrenzten Durchsatz (nur Druckkopplung greift).
    pub fn new(
        max_io_bytes_per_second: u64,
        erase_io_share: f32,
        clock: Arc<dyn Clock>,
        pressure_rx: watch::Receiver<SystemPressure>,
    ) -> Self {
        let clamped_share = if erase_io_share.is_finite() {
            erase_io_share.clamp(0.05, 0.95)
        } else {
            0.5
        };
        let now = clock.monotonic_nanos();
        Self {
            state: Arc::new(Mutex::new(SharedBucketState {
                total_bucket: TokenBucket::new(max_io_bytes_per_second, Arc::clone(&clock)),
                erase_bucket: TokenBucket::new(max_io_bytes_per_second, Arc::clone(&clock)),
                last_compaction_waiter_nanos: 0,
                last_erase_waiter_nanos: 0,
                jitter_state: now ^ 0x9E37_79B9_7F4A_7C15,
            })),
            clock,
            pressure_rx,
            max_io_bytes_per_second,
            erase_io_share: clamped_share,
        }
    }

    fn next_jitter_ms(&self, base_ms: u64) -> u64 {
        let mut s = self.state.lock();
        // Einfacher deterministischer xorshift64 für kooperativen Backoff-Jitter.
        let mut x = s.jitter_state.max(1);
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.jitter_state = x;
        base_ms.saturating_add(x % 15)
    }

    fn effective_erase_bps(
        &self,
        now_nanos: u64,
        last_compaction_nanos: u64,
        hint: PressureHint,
    ) -> u64 {
        if self.max_io_bytes_per_second == 0 {
            return 0;
        }
        let compaction_idle = last_compaction_nanos == 0
            || now_nanos.saturating_sub(last_compaction_nanos) > 1_000_000_000;
        let base = if compaction_idle {
            // Ungenutzten Compaction-Anteil leihen.
            self.max_io_bytes_per_second
        } else {
            ((self.max_io_bytes_per_second as f64) * (self.erase_io_share as f64)).max(1.0) as u64
        };
        match hint {
            PressureHint::Normal => base,
            PressureHint::Elevated => (base / 2).max(1),
            PressureHint::Critical => 1,
        }
    }

    fn effective_compaction_bps(&self, now_nanos: u64, last_erase_nanos: u64) -> u64 {
        if self.max_io_bytes_per_second == 0 {
            return 0;
        }
        let erase_idle =
            last_erase_nanos == 0 || now_nanos.saturating_sub(last_erase_nanos) > 1_000_000_000;
        if erase_idle {
            self.max_io_bytes_per_second
        } else {
            let share = 1.0 - (self.erase_io_share as f64);
            ((self.max_io_bytes_per_second as f64) * share).max(1.0) as u64
        }
    }
}

impl IoBudget for StoreIoBudget {
    fn acquire<'a>(
        &'a self,
        class: IoClass,
        bytes: u64,
    ) -> BoxFuture<'a, Result<(), ErasureError>> {
        Box::pin(async move {
            if class == IoClass::Interactive {
                return Ok(());
            }

            loop {
                let hint = self.pressure();
                if hint == PressureHint::Critical {
                    let wait_ms = self.next_jitter_ms(25);
                    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                    continue;
                }

                let now = self.clock.monotonic_nanos();
                let (delay_opt, class_limit) = {
                    let mut st = self.state.lock();
                    match class {
                        IoClass::Interactive => (None, 0),
                        IoClass::Erase => {
                            st.last_erase_waiter_nanos = now;
                            let erase_limit = self.effective_erase_bps(
                                now,
                                st.last_compaction_waiter_nanos,
                                hint,
                            );
                            let total_limit = self.max_io_bytes_per_second;
                            let d_erase =
                                st.erase_bucket.charge_step_with_limit(bytes, erase_limit);
                            let d_total =
                                st.total_bucket.charge_step_with_limit(bytes, total_limit);
                            let delay = match (d_erase, d_total) {
                                (Some(a), Some(b)) => Some(a.max(b)),
                                (Some(a), None) => Some(a),
                                (None, Some(b)) => Some(b),
                                (None, None) => None,
                            };
                            (delay, erase_limit)
                        }
                        IoClass::Compaction => {
                            st.last_compaction_waiter_nanos = now;
                            let comp_limit =
                                self.effective_compaction_bps(now, st.last_erase_waiter_nanos);
                            let delay = st.total_bucket.charge_step_with_limit(bytes, comp_limit);
                            (delay, comp_limit)
                        }
                    }
                };

                if let Some(delay) = delay_opt {
                    tokio::time::sleep(delay).await;
                    let mut st = self.state.lock();
                    match class {
                        IoClass::Interactive => {}
                        IoClass::Erase => {
                            st.erase_bucket.on_sleep_complete_with_limit(class_limit);
                            st.total_bucket
                                .on_sleep_complete_with_limit(self.max_io_bytes_per_second);
                        }
                        IoClass::Compaction => {
                            st.total_bucket.on_sleep_complete_with_limit(class_limit);
                        }
                    }
                    // Nur beim ersten Durchlauf Bytes belasten, danach ist Budget gewährt.
                    return Ok(());
                }
                return Ok(());
            }
        })
    }

    fn pressure(&self) -> PressureHint {
        match self.pressure_rx.borrow().pressure_level {
            PressureLevel::Normal => PressureHint::Normal,
            PressureLevel::Elevated => PressureHint::Elevated,
            PressureLevel::Critical => PressureHint::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::clock::SystemClock;

    #[tokio::test]
    async fn test_io_budget_interactive_never_throttled() {
        let clock = Arc::new(SystemClock::new());
        let (_tx, rx) = watch::channel(SystemPressure {
            wal_queue_depth: 1000,
            scheduler_queue_depth: 1.0,
            blocking_util: 1.0,
            embedding_queue_depth: 0,
            pressure_level: PressureLevel::Critical,
        });
        let budget = StoreIoBudget::new(100, 0.5, clock, rx);
        budget
            .acquire(IoClass::Interactive, 10_000_000)
            .await
            .unwrap();
        assert_eq!(budget.pressure(), PressureHint::Critical);
    }

    #[tokio::test]
    async fn test_io_budget_unlimited_when_zero() {
        let clock = Arc::new(SystemClock::new());
        let (_tx, rx) = watch::channel(SystemPressure {
            wal_queue_depth: 0,
            scheduler_queue_depth: 0.0,
            blocking_util: 0.0,
            embedding_queue_depth: 0,
            pressure_level: PressureLevel::Normal,
        });
        let budget = StoreIoBudget::new(0, 0.5, clock, rx);
        budget.acquire(IoClass::Erase, 1_000_000).await.unwrap();
        budget
            .acquire(IoClass::Compaction, 1_000_000)
            .await
            .unwrap();
    }
}
