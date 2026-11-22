//! # Governor: Ressourcenverwaltung und faire Quoten
//!
//! SPEC §17.1-17.5:
//! - Wird als `Arc<Governor>` über `Env` gehalten, nicht global (P-5).
//! - Quoten je Instanz als Gewichte mit Mindestanteil für `BlockCache` und `TableCache`.
//! - Token-Bucket für Lese- und Schreibbytes je Sekunde (Clock-basiert).
//! - `PressureSource`-Anbindung als optionaler Eingang.

use contextra_storage_ports::{Clock, Pressure, PressureSource};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Quoten-Schnittstelle für offene Dateideskriptoren (kompatibel mit TableCache).
pub trait FileQuota: Send + Sync {
    /// Versucht, einen Dateideskriptor zu belegen.
    fn acquire_file_fd(&self) -> bool;
    /// Gibt einen belegten Dateideskriptor frei.
    fn release_file_fd(&self);
}

/// Token-Bucket zur Ratenbegrenzung von Hintergrundoperationen.
#[derive(Debug)]
pub struct TokenBucket {
    rate_bytes_per_sec: u64,
    burst_bytes: u64,
    available_tokens: f64,
    last_update_ms: u64,
}

impl TokenBucket {
    /// Erstellt einen neuen Token-Bucket.
    pub fn new(rate_bytes_per_sec: u64, burst_bytes: u64, now_ms: u64) -> Self {
        Self {
            rate_bytes_per_sec,
            burst_bytes,
            available_tokens: burst_bytes as f64,
            last_update_ms: now_ms,
        }
    }

    /// Versucht, `bytes` aus dem Kontingent zu entnehmen.
    pub fn try_consume(&mut self, bytes: u64, now_ms: u64) -> bool {
        let elapsed_ms = now_ms.saturating_sub(self.last_update_ms);
        self.last_update_ms = now_ms;

        let added = (elapsed_ms as f64 / 1000.0) * (self.rate_bytes_per_sec as f64);
        let max_burst = self.burst_bytes as f64;
        self.available_tokens = (self.available_tokens + added).min(max_burst);

        let cost = bytes as f64;
        if self.available_tokens >= cost {
            self.available_tokens -= cost;
            true
        } else {
            false
        }
    }

    /// Liefert die aktuell verfügbaren Tokens.
    pub fn available(&self) -> f64 {
        self.available_tokens
    }
}

/// Quotenkonfiguration und Status einer Instanz.
#[derive(Debug, Clone)]
pub struct InstanceBudget {
    /// Gewicht für die faire Verteilung (SPEC §17.4).
    pub weight: u32,
    /// Mindestanteil an offenen Dateideskriptoren (Verhinderung von Verhungern).
    pub min_fds: usize,
    /// Mindestanteil an Cache-Blöcken.
    pub min_blocks: usize,
    /// Aktuell belegte Dateideskriptoren.
    pub current_fds: usize,
    /// Aktuell belegte Cache-Blöcke.
    pub current_blocks: usize,
}

struct GovernorState {
    instances: HashMap<u64, InstanceBudget>,
    total_max_open_files: usize,
    total_block_cache_limit: usize,
    read_bucket: TokenBucket,
    write_bucket: TokenBucket,
}

fn lock_state<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match m.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Zentraler Ressourcen-Governor für Instanzen eines Prozesses.
pub struct Governor {
    state: Mutex<GovernorState>,
    clock: Arc<dyn Clock>,
    pressure: Option<Arc<dyn PressureSource>>,
}

impl Governor {
    /// Erstellt einen neuen Governor.
    pub fn new(
        clock: Arc<dyn Clock>,
        pressure: Option<Arc<dyn PressureSource>>,
        total_max_open_files: usize,
        total_block_cache_limit: usize,
        read_rate_bytes_per_sec: u64,
        write_rate_bytes_per_sec: u64,
    ) -> Self {
        let now = clock.now_ms();
        Self {
            state: Mutex::new(GovernorState {
                instances: HashMap::new(),
                total_max_open_files,
                total_block_cache_limit,
                read_bucket: TokenBucket::new(read_rate_bytes_per_sec, read_rate_bytes_per_sec, now),
                write_bucket: TokenBucket::new(write_rate_bytes_per_sec, write_rate_bytes_per_sec, now),
            }),
            clock,
            pressure,
        }
    }

    /// Ermittelt die aktuelle Druckstufe.
    pub fn pressure_level(&self) -> Pressure {
        match &self.pressure {
            Some(source) => source.level(),
            None => Pressure::Normal,
        }
    }

    /// Registriert eine Speicherinstanz mit Gewicht und Mindestanteil.
    pub fn register_instance(&self, instance_id: u64, weight: u32, min_fds: usize, min_blocks: usize) {
        let mut state = lock_state(&self.state);
        let budget = InstanceBudget {
            weight: weight.max(1),
            min_fds,
            min_blocks,
            current_fds: 0,
            current_blocks: 0,
        };
        state.instances.insert(instance_id, budget);
    }

    /// Entfernt eine Instanz aus dem Governor.
    pub fn unregister_instance(&self, instance_id: u64) {
        let mut state = lock_state(&self.state);
        state.instances.remove(&instance_id);
    }

    /// Liefert die Dateiquoten-Schnittstelle für eine bestimmte Instanz.
    pub fn file_quota(self: &Arc<Self>, instance_id: u64) -> InstanceFileQuota {
        InstanceFileQuota {
            governor: Arc::clone(self),
            instance_id,
        }
    }

    /// Belegt einen Dateideskriptor für eine Instanz.
    pub fn acquire_file_fd(&self, instance_id: u64) -> bool {
        let mut state = lock_state(&self.state);
        let total_fds: usize = state.instances.values().map(|b| b.current_fds).sum();
        let total_max = state.total_max_open_files;

        if let Some(budget) = state.instances.get_mut(&instance_id) {
            // Innerhalb des garantierten Mindestanteils immer erlauben
            if budget.current_fds < budget.min_fds && total_fds < total_max {
                budget.current_fds = budget.current_fds.saturating_add(1);
                return true;
            }
            // Außerhalb des Mindestanteils: Gesamtkapazität prüfen
            if total_fds < total_max {
                budget.current_fds = budget.current_fds.saturating_add(1);
                return true;
            }
        }
        false
    }

    /// Gibt einen belegten Dateideskriptor frei.
    pub fn release_file_fd(&self, instance_id: u64) {
        let mut state = lock_state(&self.state);
        if let Some(budget) = state.instances.get_mut(&instance_id) {
            budget.current_fds = budget.current_fds.saturating_sub(1);
        }
    }

    /// Prüft, ob ein Block für den `BlockCache` aufgenommen werden darf.
    pub fn can_admit_block(&self, instance_id: u64, current_usage: usize, item_size: usize) -> bool {
        let state = lock_state(&self.state);
        let total_blocks: usize = state.instances.values().map(|b| b.current_blocks).sum();
        let max_limit = match self.pressure_level() {
            Pressure::Normal => state.total_block_cache_limit,
            Pressure::Soft => state.total_block_cache_limit / 2,
            Pressure::Hard => state.total_block_cache_limit / 4,
        };

        if let Some(budget) = state.instances.get(&instance_id) {
            if current_usage < budget.min_blocks {
                return true;
            }
            total_blocks.saturating_add(item_size) <= max_limit
        } else {
            total_blocks.saturating_add(item_size) <= max_limit
        }
    }

    /// Prüft, ob ein Block verdrängt werden soll (z. B. wenn Quoten überschritten oder Druck herrscht).
    pub fn should_evict_block(&self, instance_id: u64, current_usage: usize) -> bool {
        let state = lock_state(&self.state);
        if let Some(budget) = state.instances.get(&instance_id) {
            if current_usage <= budget.min_blocks {
                return false;
            }
        }
        match self.pressure_level() {
            Pressure::Normal => false,
            Pressure::Soft | Pressure::Hard => true,
        }
    }

    /// Aktualisiert den Blockverbrauch einer Instanz.
    pub fn update_block_usage(&self, instance_id: u64, usage: usize) {
        let mut state = lock_state(&self.state);
        if let Some(budget) = state.instances.get_mut(&instance_id) {
            budget.current_blocks = usage;
        }
    }

    /// Drosselt Lesezugriffe im Hintergrund über den Lese-Token-Bucket.
    pub fn throttle_background_read(&self, bytes: u64) -> bool {
        let now = self.clock.now_ms();
        let mut state = lock_state(&self.state);
        state.read_bucket.try_consume(bytes, now)
    }

    /// Drosselt Schreibzugriffe im Hintergrund über den Schreib-Token-Bucket.
    pub fn throttle_background_write(&self, bytes: u64) -> bool {
        let now = self.clock.now_ms();
        let mut state = lock_state(&self.state);
        state.write_bucket.try_consume(bytes, now)
    }
}

/// Anbindungsadapter für TableCache einer konkreten Instanz.
#[derive(Clone)]
pub struct InstanceFileQuota {
    governor: Arc<Governor>,
    instance_id: u64,
}

impl FileQuota for InstanceFileQuota {
    fn acquire_file_fd(&self) -> bool {
        self.governor.acquire_file_fd(self.instance_id)
    }

    fn release_file_fd(&self) {
        self.governor.release_file_fd(self.instance_id);
    }
}
