//! Gesundheitsstatus und Metriken der Engine (SPEC §14.5, D-64).

use std::time::Duration;

/// Grundlegender Systemzustand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineState {
    /// Normalbetrieb.
    Ok,
    /// Beeinträchtigter Betrieb.
    Degraded,
    /// Fataler E/A- oder Integritätsfehler.
    Poisoned,
    /// Wiederanlauf und Derivate-Wiederaufbau.
    Recovering,
}

/// Grund für den Zustand `Degraded`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DegradedReason {
    /// Ledger fehlt (neuer Rechner).
    LedgerMissing,
    /// Purge-Fortschritt hinter SLA.
    PurgeSlow,
    /// Freier Speicherplatz unter Reserve.
    FreeSpaceLow,
    /// Prozesshärtung nur teilweise wirksam.
    HardeningPartial,
    /// Auffällige fsync-Latenz.
    SuspiciousFsyncLatency,
    /// Observer-Warteschlange voll.
    ObserverBacklog,
    /// Vektor-Rebuild dauert länger als SLA.
    RebuildSlow,
    /// Zeitrückwärtsbewegung erkannt.
    ClockRegression,
}

/// Gesundheitsbericht der Engine ohne blockierende Datei-I/O (D-64).
#[derive(Debug, Clone)]
pub struct HealthReport {
    /// Aktueller Zustand.
    pub state: EngineState,
    /// p50 fsync Latenz der Startprobe.
    pub fsync_p50: Duration,
    /// Anzahl offener Löschvorgänge.
    pub open_erasures: usize,
    /// Alter der ältesten offenen Löschung in Sekunden.
    pub oldest_open_age_secs: u64,
    /// Anzahl registrierter SLA-Verletzungen.
    pub sla_violations: u64,
    /// Verhältnis Tombstones zu Gesamteinträgen.
    pub tombstone_ratio: f32,
    /// Freier Speicherplatz in Bytes auf der Datenpartition.
    pub free_bytes: u64,
    /// Maximale Wartezeit auf die Purge-Barriere.
    pub max_purge_barrier_wait: Duration,
    /// Status der Prozesshärtung.
    pub process_hardening: bool,
    /// Liste von Gründen bei Beeinträchtigung.
    pub reasons: Vec<DegradedReason>,
}

impl Default for HealthReport {
    fn default() -> Self {
        Self {
            state: EngineState::Ok,
            fsync_p50: Duration::from_micros(1000),
            open_erasures: 0,
            oldest_open_age_secs: 0,
            sla_violations: 0,
            tombstone_ratio: 0.0,
            free_bytes: 10 * 1024 * 1024 * 1024,
            max_purge_barrier_wait: Duration::ZERO,
            process_hardening: true,
            reasons: Vec::new(),
        }
    }
}
