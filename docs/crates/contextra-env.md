> **Löschpfad v3.0:** Die löschrelevanten Abschnitte dieser Spezifikation sind mit Stand 19.11.2026 durch `docs/SPEC.md` v3.0 ersetzt bzw. präzisiert (§5, §6, §7, §9, §12, §13, §14, §16 der SPEC). Widerspricht ein Abschnitt unten der SPEC, gilt die SPEC. Abschnitte, die so markiert sind, sind überholt. Weitere Verweise: §23 SPEC (Ersetzungstabelle).

# contextra-env — Implementierungsspezifikation (Soll-Zustand)

**Ebene:** E0 (Autarkes Fundament / Ring 0)  
**Kern-Closure:** Leer (0 interne Workspace-Abhängigkeiten)  
**Status:** Normativ (Soll-Zustand für Entwickler)  
**Version:** 2.0  
**Geltungsbereich:** `crates/contextra-env`  
**Referenzdokumente:** `docs/SPEC.md` §2, §5; `ANALYSIS-E0-CONTRADICTIONS.md` (D-04); `ALTLASTEN-AUDIT.md`  

---

## 1. Zweck & Architekturgarantien

`contextra-env` stellt deterministische, injizierbare Umgebungsabstraktionen für das Gesamtsystem bereit:
- **Zeitabstraktion (`Clock`):** Entkopplung von unkontrollierter Systemzeit für deterministische Zeitreise-, Replay- und Saga-Tests.
- **Zufallsquellen (`Rng`):** Deterministische Pseudozufalls-Generatoren (`SeededRng`) für reproduzierbare Tests neben kryptographischem CSPRNG.
- **Sequenz- & ID-Generierung (`IdGen`):** Lock-freie atomare Generierung lokaler Sequenznummern.
- **Metriken- & Telemetriesenke (`MetricsSink`):** Autarke Erfassung von Latenzen, I/O-Zählern und Zuständen ohne Netzwerk-Sockets.

### Architekturentscheidung VFS (D-04)
Das in früheren Entwürfen in `contextra-env` skizzierte VFS-Trait entfällt hier vollständig. Die Dateisystemabstraktion mit Crash-Konsistenz, Fsync-Barrieren und Handle-Poisoning gehört konzeptionell und physisch exklusiv nach `contextra-durable-fs`. `contextra-env` bleibt ein 100 % I/O- und runtime-freier Rechenkern.

### Architekturgarantien (R1–R6, INV-3, INV-4)
1. **0 interne Workspace-Abhängigkeiten:** Reines Fundament.
2. **`#![forbid(unsafe_code)]`:** Vollständig sicheres Rust.
3. **Zero-Panic-Policy:** Keine Panics im Produktivcode.
4. **Leere Default-Features (R3):** `default = []`.
5. **Kein globaler Zustand:** Instanzen werden explizit per `Arc<dyn Clock>` oder generischem Parameter injiziert.

---

## 2. Kanonische Abstraktionen & Traits

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                                contextra-env                                │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │          Clock           │             │            Rng             │   │
│   │ now_unix_nanos(),        │             │ next_u64(), next_u32(),    │   │
│   │ monotonic_nanos()        │             │ fill_bytes(&mut [u8])      │   │
│   └─────────────┬────────────┘             └─────────────┬──────────────┘   │
│                 │                                        │                  │
│         ┌───────┴───────┐                                │                  │
│         ▼               ▼                                ▼                  │
│   SystemClock      ManualClock                       SeededRng              │
│   (Prod/OS)       (Test-Steuerung)                 (SplitMix64)             │
│                                                                             │
│   ┌──────────────────────────┐             ┌────────────────────────────┐   │
│   │          IdGen           │             │        MetricsSink         │   │
│   │        next_id()         │             │ record_counter(),          │   │
│   │    (SequentialIdGen)     │             │ record_gauge(), histogram  │   │
│   └──────────────────────────┘             └────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### 2.1 Zeitabstraktion (`Clock` & Implementierungen)

```rust
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Port-Trait für Zeiterfassung ohne unkontrollierte direkte OS-Aufrufe.
pub trait Clock: Send + Sync + 'static {
    /// Liefert die aktuelle Unix-Epochenzeit in Nanosekunden.
    /// Liefert 0 bei Zeitständen vor der Epoche (Fail-Closed, kein Panic).
    fn now_unix_nanos(&self) -> u64;

    /// Liefert einen monoton steigenden Nanosekunden-Zähler.
    fn monotonic_nanos(&self) -> u64;
}

/// Standard-Produktions-Uhr basierend auf OS-Instanten.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock {
    start_instant: Instant,
}

impl SystemClock {
    #[allow(clippy::disallowed_methods)]
    pub fn new() -> Self {
        Self {
            start_instant: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    #[allow(clippy::disallowed_methods)]
    fn now_unix_nanos(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| u64::try_from(d.as_nanos()).unwrap_or(u64::MAX))
            .unwrap_or(0)
    }

    #[allow(clippy::disallowed_methods)]
    fn monotonic_nanos(&self) -> u64 {
        let nanos = Instant::now()
            .saturating_duration_since(self.start_instant)
            .as_nanos();
        u64::try_from(nanos).unwrap_or(u64::MAX)
    }
}

impl<T: Clock + ?Sized> Clock for Arc<T> {
    #[inline]
    fn now_unix_nanos(&self) -> u64 {
        (**self).now_unix_nanos()
    }

    #[inline]
    fn monotonic_nanos(&self) -> u64 {
        (**self).monotonic_nanos()
    }
}

/// Kanonische Umrechnung in Unix-Millisekunden (SPEC §4.2, B02).
#[inline]
pub fn unix_ms(c: &dyn Clock) -> u64 {
    c.now_unix_nanos() / 1_000_000
}
```

#### Test-Doppel: `ManualClock` (Feature `test-utils`)
```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// Manuell steuerbare Uhr für deterministische Test-Szenarien.
#[derive(Debug)]
pub struct ManualClock {
    unix_nanos: AtomicU64,
    monotonic_nanos: AtomicU64,
}

impl ManualClock {
    pub fn new(initial_unix_nanos: u64) -> Self {
        Self {
            unix_nanos: AtomicU64::new(initial_unix_nanos),
            monotonic_nanos: AtomicU64::new(0),
        }
    }

    pub fn advance_millis(&self, ms: u64) {
        let nanos = ms.saturating_mul(1_000_000);
        self.unix_nanos.fetch_add(nanos, Ordering::SeqCst);
        self.monotonic_nanos.fetch_add(nanos, Ordering::SeqCst);
    }

    pub fn set_unix_nanos(&self, nanos: u64) {
        self.unix_nanos.store(nanos, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_unix_nanos(&self) -> u64 {
        self.unix_nanos.load(Ordering::SeqCst)
    }

    fn monotonic_nanos(&self) -> u64 {
        self.monotonic_nanos.load(Ordering::SeqCst)
    }
}
```

---

### 2.2 Pseudozufall (`Rng` & `SeededRng`)

```rust
/// Trait für deterministische Pseudozufalls-Generierung in Simulationen und Indizes.
pub trait Rng: Send + Sync + 'static {
    fn next_u64(&self) -> u64;
    fn next_u32(&self) -> u32 {
        (self.next_u64() >> 32) as u32
    }
    fn fill_bytes(&self, dest: &mut [u8]);
}

/// Fadenfester SplitMix64 PRNG.
pub struct SeededRng {
    state: std::sync::atomic::AtomicU64,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: std::sync::atomic::AtomicU64::new(seed),
        }
    }
}

impl Rng for SeededRng {
    fn next_u64(&self) -> u64 {
        use std::sync::atomic::Ordering;
        let mut z = self.state.fetch_add(0x9E3779B97F4A7C15, Ordering::Relaxed);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    fn fill_bytes(&self, dest: &mut [u8]) {
        let mut chunks = dest.chunks_exact_mut(8);
        for chunk in chunks.by_ref() {
            let val = self.next_u64();
            chunk.copy_from_slice(&val.to_le_bytes());
        }
        let rem = chunks.into_remainder();
        if !rem.is_empty() {
            let val = self.next_u64();
            let bytes = val.to_le_bytes();
            rem.copy_from_slice(&bytes[..rem.len()]);
        }
    }
}
```

---

### 2.3 ID-Generierung (`IdGen` & `SequentialIdGen`)

```rust
use std::sync::atomic::{AtomicU64, Ordering};

/// Port-Trait für monoton wachsende, lock-freie In-Memory-IDs.
pub trait IdGen: Send + Sync + 'static {
    fn next_id(&self) -> u64;
}

#[derive(Debug)]
pub struct SequentialIdGen {
    next: AtomicU64,
}

impl SequentialIdGen {
    pub const fn new(start: u64) -> Self {
        Self {
            next: AtomicU64::new(start),
        }
    }
}

impl Default for SequentialIdGen {
    fn default() -> Self {
        Self::new(1)
    }
}

impl IdGen for SequentialIdGen {
    fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
}
```

---

### 2.4 Metriken (`MetricsSink`)

```rust
/// Port-Trait für lokale Observability ohne externe Abhängigkeiten.
pub trait MetricsSink: Send + Sync + 'static {
    fn record_counter(&self, key: &'static str, value: u64);
    fn record_gauge(&self, key: &'static str, value: i64);
    fn record_histogram_duration_ns(&self, key: &'static str, nanos: u64);
}

/// No-Op Implementierung für Produktionsläufe ohne Overhead.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopMetricsSink;

impl MetricsSink for NoopMetricsSink {
    #[inline]
    fn record_counter(&self, _key: &'static str, _value: u64) {}
    #[inline]
    fn record_gauge(&self, _key: &'static str, _value: i64) {}
    #[inline]
    fn record_histogram_duration_ns(&self, _key: &'static str, _nanos: u64) {}
}
```

---

## 3. `Cargo.toml` Spezifikation

```toml
[package]
name = "contextra-env"
version.workspace = true
edition.workspace = true
license.workspace = true
rust-version.workspace = true
description = "Injectable environment (clock, rng, id generation, metrics) for Contextra"

[lints]
workspace = true

[dependencies]

[dev-dependencies]

[features]
default = []
std = []
test-utils = []
```

---

## 4. Implementierungs-Arbeitspakete für Entwickler

1. **`Cargo.toml` Bereinigung:**
   - `[lints] workspace = true` ergänzen.
   - `default = []` setzen (R3).
2. **VFS-Reste eliminieren:**
   - Sicherstellen, dass kein Dummy- oder Pseudo-VFS in `contextra-env` verbleibt (D-04).
3. **Dokumentation & Altlasten:**
   - Entfernte Crate-Referenzen in Doc-Kommentaren bereinigen (`AP-21`).
4. **Verifikation:**
   - `cargo test -p contextra-env --features test-utils`
