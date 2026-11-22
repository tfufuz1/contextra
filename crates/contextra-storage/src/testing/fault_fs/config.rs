//! # FaultFs Konfiguration und PRNG
//!
//! SPEC §5.1, §10.5, §19.2 und IMPL §6 WP-1.3.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_storage_ports::{HookPoint, Hooks};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

/// Absturz-Verhalten bei nicht synchronisierten Daten.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrashMode {
    /// Alle unsynchronisierten Schreibvorgänge gehen verloren.
    Lost,
    /// Schreibvorgänge werden auf Sektorgröße zerrissen (z.B. 512 B oder 4096 B).
    Torn(usize),
    /// Schreibvorgänge werden in zufälliger Reihenfolge übernommen.
    Reordered,
}

impl Default for CrashMode {
    fn default() -> Self {
        Self::Lost
    }
}

/// Deterministischer Xorshift64 PRNG nach SPEC §19.1 / §19.2.
#[derive(Debug, Clone)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    /// Erstellt eine neue Instanz von `XorShift64` mit dem angegebenen Seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0xdeadbeef_cafebabe } else { seed },
        }
    }

    /// Liefert die nächste pseudozufällige 64-Bit-Ganzzahl.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Liefert eine Pseudozufallszahl im Bereich `[min, max]`.
    pub fn next_range(&mut self, min: u64, max: u64) -> u64 {
        if min >= max {
            return min;
        }
        min + (self.next_u64() % (max - min + 1))
    }

    /// Liefert einen pseudozufälligen booleschen Wert.
    pub fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) != 0
    }
}

/// Konfiguration für Fehlereinspritzungen nach SPEC §19.2.
pub struct FaultConfig {
    /// PRNG-Seed für deterministische Läufe
    pub seed: u64,
    /// Thread-sicherer Zufallsgenerator
    pub rng: Mutex<XorShift64>,

    /// Vorgabe, nach wie vielen Operationen ein Absturz ausgelöst wird (0 = deaktiviert).
    pub fail_after_ops: AtomicU64,
    /// Zähler der bisher ausgeführten E/A-Operationen.
    pub op_counter: AtomicU64,
    /// Absturzmodus (Lost, Torn, Reordered).
    pub crash_mode: Mutex<CrashMode>,
    /// Wurde bereits ein Absturz ausgelöst?
    pub crashed: AtomicBool,
    /// Handler zur Ausführung des Absturzes auf dem Dateisystem.
    pub crash_handler: Mutex<Option<Arc<dyn Fn() + Send + Sync>>>,

    /// Löst Fehler bei Sync-Operationen (fsync, sync_dir) aus.
    pub fail_sync: AtomicBool,
    /// Simuliert vollen Speicherplatz (ENOSPC) bei Schreib- und Allokationsoperationen.
    pub fail_enospc: AtomicBool,
    /// Simuliert generelle Dateisystemfehler (EIO).
    pub fail_io: AtomicBool,

    /// Simuliert Lesefehler (EIO) gezielt beim Lesen.
    pub fail_read_eio: AtomicBool,
    /// Offset für gezielten Lesefehler (u64::MAX = an jedem Offset).
    pub fail_read_offset: AtomicU64,

    /// Zerreißen von Schreibvorgängen (z.B. max Bytes vor Fehler, 0 = deaktiviert).
    pub torn_write_limit: AtomicU64,
    /// Kurzschreibvorgang bei Schreiboperationen (z.B. max Bytes vor ENOSPC).
    pub short_write_limit: AtomicU64,

    /// Verlorener Schreibvorgang (quittiert, aber nicht geschrieben).
    pub lost_write: AtomicBool,
    /// Fehlgeleiteter Schreibvorgang (falscher Offset derselben Datei).
    pub misdirected_write: AtomicBool,
    /// Offset-Delta für fehlgeleitete Schreibvorgänge.
    pub misdirected_offset_delta: AtomicU64,

    /// Bitfäule beim Lesen aktivieren.
    pub enable_bit_rot: AtomicBool,
    /// Offset für Bitfäule beim Lesen.
    pub bit_rot_offset: AtomicU64,

    /// Optionaler Test-Hooks-Empfänger.
    pub hooks: Mutex<Option<Arc<dyn Hooks>>>,
    /// Ereignis-Sequenz für Determinismus-Prüfungen.
    pub events: Mutex<Vec<String>>,
    /// Rollender 64-Bit-Hash der Ereignissequenz.
    pub event_hash: AtomicU64,
}

impl Default for FaultConfig {
    fn default() -> Self {
        Self::new(0x1337_c0de)
    }
}

impl FaultConfig {
    /// Erstellt eine neue `FaultConfig` mit dem angegebenen Seed.
    pub fn new(seed: u64) -> Self {
        let initial_seed = if seed == 0 { 0xdeadbeef_cafebabe } else { seed };
        Self {
            seed: initial_seed,
            rng: Mutex::new(XorShift64::new(initial_seed)),
            fail_after_ops: AtomicU64::new(0),
            op_counter: AtomicU64::new(0),
            crash_mode: Mutex::new(CrashMode::Lost),
            crashed: AtomicBool::new(false),
            crash_handler: Mutex::new(None),
            fail_sync: AtomicBool::new(false),
            fail_enospc: AtomicBool::new(false),
            fail_io: AtomicBool::new(false),
            fail_read_eio: AtomicBool::new(false),
            fail_read_offset: AtomicU64::new(u64::MAX),
            torn_write_limit: AtomicU64::new(0),
            short_write_limit: AtomicU64::new(0),
            lost_write: AtomicBool::new(false),
            misdirected_write: AtomicBool::new(false),
            misdirected_offset_delta: AtomicU64::new(0),
            enable_bit_rot: AtomicBool::new(false),
            bit_rot_offset: AtomicU64::new(0),
            hooks: Mutex::new(None),
            events: Mutex::new(Vec::new()),
            event_hash: AtomicU64::new(0xcbf29ce484222325),
        }
    }

    /// Setzt den Handler für Abstürze.
    pub fn set_crash_handler(&self, handler: Arc<dyn Fn() + Send + Sync>) {
        *self.crash_handler.lock().unwrap() = Some(handler);
    }

    /// Führt den Absturz aus, falls ein Handler registriert ist.
    pub fn trigger_crash(&self) {
        self.crashed.store(true, Ordering::SeqCst);
        if let Some(h) = self.crash_handler.lock().unwrap().as_ref() {
            h();
        }
    }

    /// Setzt einen Test-Hooks-Empfänger.
    pub fn set_hooks(&self, hooks: Arc<dyn Hooks>) {
        *self.hooks.lock().unwrap() = Some(hooks);
    }

    /// Löst einen Test-Hook-Punkt aus.
    pub fn trigger_hook(&self, p: HookPoint) {
        if let Some(h) = self.hooks.lock().unwrap().as_ref() {
            h.point(p);
        }
        self.record_event(&format!("Hook:{p:?}"));
    }

    /// Zeichnet ein Ereignis auf und aktualisiert den Hash der Sequenz.
    pub fn record_event(&self, event: &str) {
        let mut events = self.events.lock().unwrap();
        events.push(event.to_string());
        let mut h = self.event_hash.load(Ordering::Relaxed);
        for byte in event.as_bytes() {
            h ^= *byte as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        self.event_hash.store(h, Ordering::Relaxed);
    }

    /// Liefert den Hash der bisherigen Ereignissequenz.
    pub fn event_sequence_hash(&self) -> u64 {
        self.event_hash.load(Ordering::Relaxed)
    }

    /// Liefert den Seed der Konfiguration.
    pub fn seed(&self) -> u64 {
        self.seed
    }
}
