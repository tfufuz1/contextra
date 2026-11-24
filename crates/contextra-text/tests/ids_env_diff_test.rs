//! Differenztest für Ids und Env (SPEC §5.4, WP5).

use contextra_env::Clock as EnvClock;
use contextra_ids::DocId as IdsDocId;
use contextra_text::contract::Clock as TextClock;
use contextra_text::types::DocId as TextDocId;

struct LocalManualClock(u64);

impl EnvClock for LocalManualClock {
    fn now_unix_nanos(&self) -> u64 {
        self.0
    }

    fn monotonic_nanos(&self) -> u64 {
        self.0
    }
}

#[test]
fn test_docid_difference_and_equivalence() {
    let t_id = TextDocId::new(12345u64);
    let i_id = IdsDocId::new(12345u64);

    assert_eq!(t_id.inner(), i_id.inner());
    assert_eq!(t_id.inner() as u64, i_id.as_u64());
}

#[test]
fn test_clock_difference_and_truncation_bitgleich() {
    let nanos = 1_770_000_000_123_456_789u64;
    let clock = LocalManualClock(nanos);

    // Alte ms-Berechnung über contract::Clock
    let text_ms = clock.now_unix_ms();

    // Kanonische env::unix_ms Berechnung
    let env_ms = contextra_env::unix_ms(&clock);

    assert_eq!(text_ms, env_ms);
    assert_eq!(text_ms, 1_770_000_000_123u64);
}
