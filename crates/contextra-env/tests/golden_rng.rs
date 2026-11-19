//! Bit-exakte Golden-Vektoren für `SeededRng` (FND.1, INV-1).
//!
//! Die Referenz `tests/golden/seeded_rng_sequences.json` ist eingefroren; eine Abweichung
//! bedeutet eine Änderung persistierter/reproduzierbarer Zufallsfolgen und ist ein Fehler.
//! Der Parser ist bewusst dependency-frei (`[dev-dependencies]` bleibt leer, Spec §3).

use contextra_env::{Rng, SeededRng};
use std::collections::BTreeMap;

const GOLDEN_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/golden/seeded_rng_sequences.json"
);

/// Liest das von `serde_json::to_string_pretty` erzeugte Format: `"<seed>": [ <u64>, ... ]`.
fn parse_golden(text: &str) -> BTreeMap<u64, Vec<u64>> {
    let mut out = BTreeMap::new();
    let mut current: Option<(u64, Vec<u64>)> = None;

    for raw in text.lines() {
        let line = raw.trim().trim_end_matches(',');
        if let Some(stripped) = line.strip_prefix('"') {
            if let Some((key, rest)) = stripped.split_once('"') {
                if rest.trim_start().starts_with(':') && rest.contains('[') {
                    if let Ok(seed) = key.parse::<u64>() {
                        current = Some((seed, Vec::new()));
                    }
                }
            }
        } else if line == "]" {
            if let Some((seed, values)) = current.take() {
                out.insert(seed, values);
            }
        } else if let Some((_, values)) = current.as_mut() {
            if let Ok(value) = line.parse::<u64>() {
                values.push(value);
            }
        }
    }
    out
}

fn load_golden() -> BTreeMap<u64, Vec<u64>> {
    let text = std::fs::read_to_string(GOLDEN_PATH).unwrap_or_default();
    parse_golden(&text)
}

#[test]
fn golden_file_covers_canonical_seeds() {
    // NOTE: Sanity-Check der Referenzdatei selbst (Seeds 0, 1, 42, u64::MAX à 1024 Werte).
    let golden = load_golden();
    let seeds: Vec<u64> = golden.keys().copied().collect();
    assert_eq!(seeds, vec![0, 1, 42, u64::MAX]);
    for (seed, values) in &golden {
        assert_eq!(values.len(), 1024, "seed {seed} must carry 1024 vectors");
    }
}

#[test]
fn seeded_rng_matches_golden_vectors_bit_exactly() {
    let golden = load_golden();
    assert!(
        !golden.is_empty(),
        "golden file missing or unparsable: {GOLDEN_PATH}"
    );

    for (seed, expected) in golden {
        let rng = SeededRng::new(seed);
        let draws: Vec<u64> = (0..expected.len()).map(|_| rng.next_u64()).collect();
        assert_eq!(draws, expected, "golden mismatch for seed {seed}");
    }
}

#[test]
fn seeded_rng_next_u32_matches_golden_high_halves() {
    // NOTE: next_u32 ist per Definition die oberen 32 Bit der entsprechenden Golden-Ziehung.
    let golden = load_golden();
    let expected = golden.get(&42).cloned().unwrap_or_default();
    assert!(!expected.is_empty(), "seed 42 missing in golden file");
    let rng = SeededRng::new(42);
    let got: Vec<u32> = (0..expected.len()).map(|_| rng.next_u32()).collect();
    let want: Vec<u32> = expected
        .iter()
        .map(|v| u32::try_from(v >> 32).unwrap_or_default())
        .collect();
    assert_eq!(got, want);
}
