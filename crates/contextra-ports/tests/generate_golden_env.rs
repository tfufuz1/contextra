#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Generator test for Golden Data contextra-env (FND.1)

use contextra_ports::{Rng, SeededRng};
use std::fs;

#[test]
#[ignore]
fn generate_golden_env_data() {
    let out_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../contextra-env/tests/golden");
    fs::create_dir_all(out_dir).unwrap();

    let seeds = vec![0u64, 1u64, 42u64, u64::MAX];
    let mut results = std::collections::BTreeMap::new();

    for seed in seeds {
        let rng = SeededRng::new(seed);
        let mut vals = Vec::with_capacity(1024);
        for _ in 0..1024 {
            vals.push(rng.next_u64());
        }
        results.insert(seed.to_string(), vals);
    }

    fs::write(
        format!("{}/seeded_rng_sequences.json", out_dir),
        serde_json::to_string_pretty(&results).unwrap(),
    )
    .unwrap();

    println!("Golden files for contextra-env successfully generated.");
}
