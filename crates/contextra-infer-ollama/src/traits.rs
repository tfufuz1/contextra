//! Lokale Trait-Abstraktionen für Inferenz, Zeit und Zufall (B3, B11, SPEC §13.3).

use crate::error::Result;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

/// Gepinntes dynamisches Future für asynchrone Trait-Methoden.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Lokale Abstraktion für Embedding-Modelle (SPEC §13.3).
pub trait Embedder: Send + Sync {
    /// Erzeugt ein Embedding für den übergebenen Text.
    fn embed<'a>(&'a self, text: &'a str) -> BoxFuture<'a, Result<Vec<f32>>>;

    /// Gibt die Vektordimension der erzeugten Einbettungen zurück.
    fn dimension(&self) -> usize;

    /// Erzeugt Embeddings für einen Batch von Texten.
    fn embed_batch<'a>(&'a self, texts: &'a [&'a str]) -> BoxFuture<'a, Result<Vec<Vec<f32>>>> {
        Box::pin(async move {
            let mut results = Vec::with_capacity(texts.len());
            for text in texts {
                results.push(self.embed(text).await?);
            }
            Ok(results)
        })
    }
}

/// Lokale Abstraktion für Textgenerierung (LLM).
pub trait Generator: Send + Sync {
    /// Generiert Text basierend auf dem Prompt.
    fn generate<'a>(&'a self, prompt: &'a str) -> BoxFuture<'a, Result<String>>;
}

/// Lokale Abstraktion für Monotone Zeitmessung (B11).
pub trait Clock: Send + Sync {
    /// Gibt den aktuellen monotonen Zeitstempel in Nanosekunden zurück.
    fn monotonic_nanos(&self) -> u64;

    /// Gibt den aktuellen monotonen Zeitstempel in Millisekunden zurück.
    fn now_millis(&self) -> u64 {
        self.monotonic_nanos() / 1_000_000
    }

    /// Schläft für die angegebene Dauer.
    fn sleep_millis(&self, millis: u64);
}

/// Standard-Implementierung von `Clock` mit Monotext-Systemzeit.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl SystemClock {
    pub fn new() -> Self {
        Self
    }
}

#[allow(clippy::disallowed_methods)]
impl Clock for SystemClock {
    fn monotonic_nanos(&self) -> u64 {
        use std::sync::OnceLock;
        use std::time::Instant;
        static START: OnceLock<Instant> = OnceLock::new();
        let start = START.get_or_init(Instant::now);
        u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn sleep_millis(&self, millis: u64) {
        std::thread::sleep(Duration::from_millis(millis));
    }
}

/// Lokale Abstraktion für deterministischen Pseudozufall (B11).
use parking_lot::Mutex;

pub trait Rng: Send + Sync {
    /// Gibt einen u64-Zufallswert zurück.
    fn next_u64(&self) -> u64;

    /// Gibt einen f64-Zufallswert im Intervall [0.0, 1.0) zurück.
    fn next_f64(&self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}

/// Einfacher deterministischer Pseudozufallsgenerator (SplitMix64) für Jitter (B11).
#[derive(Debug)]
pub struct SeededRng {
    state: Mutex<u64>,
}

impl SeededRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: Mutex::new(seed),
        }
    }

    #[allow(clippy::disallowed_methods, clippy::cast_possible_truncation)]
    pub fn from_entropy() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x8541_5583_498b_8535);
        Self::new(seed)
    }
}

impl Rng for SeededRng {
    fn next_u64(&self) -> u64 {
        let mut state = self.state.lock();
        *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}
