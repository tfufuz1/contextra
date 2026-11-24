//! Golden- und Orakel-Tests für contextra-text (TX6.1 / SPEC §5.1).
// ANCHOR[contextra-text:TEST-195]

use contextra_text::bm25::score_term;
use contextra_text::contract::{Clock, DocKey};
use contextra_text::posting_list::{Posting, PostingList};
use contextra_text::tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};
use contextra_ids::DocId;
use std::fs;
use std::path::Path;

struct FrozenManualClock {
    now_ms: u64,
}

impl Clock for FrozenManualClock {
    fn now_unix_ms(&self) -> u64 {
        self.now_ms
    }
}

/// Erzeugt die Golden-Dateien deterministisch neu.
#[test]
#[ignore]
fn generate_golden_corpus() {
    let out_dir = Path::new("tests/golden");
    fs::create_dir_all(out_dir).unwrap();

    // 1. Posting list V2 encoding
    let p1 = Posting::new(DocId::new(10), 2, 100);
    let p2 = Posting::new(DocId::new(100), 5, 250);
    let p3 = Posting::new(DocId::new(1000), 1, 50);
    let plist = PostingList::new(vec![p1, p2, p3]);
    let plist_bytes = plist.encode_compact().unwrap();
    fs::write(out_dir.join("posting_list_v2.bin"), plist_bytes).unwrap();

    // 2. Tokenizer outputs (DefaultTokenizer + GermanMorphTokenizer)
    let sample_text =
        "Der Donaudampfschifffahrtsgesellschaftskapitän fuhr über die Donau bei 30°C! #Rust 2026.";
    let def_tok = DefaultTokenizer;
    let def_tokens = def_tok.tokenize(sample_text);
    fs::write(
        out_dir.join("tokenizer_default.json"),
        serde_json::to_string_pretty(&def_tokens).unwrap(),
    )
    .unwrap();

    let ger_tok = GermanMorphTokenizer::try_new().unwrap();
    let ger_tokens = ger_tok.tokenize(sample_text);
    fs::write(
        out_dir.join("tokenizer_german.json"),
        serde_json::to_string_pretty(&ger_tokens).unwrap(),
    )
    .unwrap();

    // 3. BM25 scores with tie-breaking
    let mut scores = Vec::new();
    let n = 100;
    let avg_dl = 150.0;
    for (doc_id, tf, doc_len, df) in [
        (1u64, 3u32, 120u32, 5u32),
        (2u64, 3u32, 120u32, 5u32), // Exact tie with doc_id 1
        (3u64, 1u32, 80u32, 10u32),
        (4u64, 5u32, 200u32, 2u32),
    ] {
        let score = score_term(tf, doc_len, avg_dl, df, n);
        scores.push((doc_id, score));
    }
    // Deterministic sort: score desc, doc_id asc
    scores.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    fs::write(
        out_dir.join("bm25_ranking.json"),
        serde_json::to_string_pretty(&scores).unwrap(),
    )
    .unwrap();

    // 4. DocId serde representation
    let doc_id = DocId::new(42);
    let doc_id_json = serde_json::to_string(&doc_id).unwrap();
    let doc_id_bin = bincode::serialize(&doc_id).unwrap();
    fs::write(out_dir.join("doc_id.json"), doc_id_json).unwrap();
    fs::write(out_dir.join("doc_id.bin"), doc_id_bin).unwrap();

    // DocKey serialization (u128 representation)
    let doc_key = DocKey::from(42u64);
    let doc_key_bytes = doc_key.0.to_le_bytes();
    fs::write(out_dir.join("doc_key.bin"), doc_key_bytes).unwrap();

    // 5. Clock now_unix_ms reference
    let clock = FrozenManualClock {
        now_ms: 1_770_000_000_123,
    };
    let clock_val = clock.now_unix_ms();
    fs::write(
        out_dir.join("clock_reference.json"),
        serde_json::to_string(&clock_val).unwrap(),
    )
    .unwrap();
}

#[test]
fn test_golden_oracle_backward_read_and_exact_match() {
    let golden_dir = Path::new("tests/golden");
    if !golden_dir.exists() {
        return;
    }

    // 1. Posting list V2
    let plist_bytes = fs::read(golden_dir.join("posting_list_v2.bin")).unwrap();
    let decoded = PostingList::decode_compact(&plist_bytes).unwrap();
    assert_eq!(decoded.len(), 3);
    assert_eq!(decoded.as_slice()[0].doc_id(), DocId::new(10));
    assert_eq!(decoded.as_slice()[1].doc_id(), DocId::new(100));
    assert_eq!(decoded.as_slice()[2].doc_id(), DocId::new(1000));

    // Re-encoding check
    let re_encoded = decoded.encode_compact().unwrap();
    assert_eq!(plist_bytes, re_encoded);

    // 2. Tokenizer
    let sample_text =
        "Der Donaudampfschifffahrtsgesellschaftskapitän fuhr über die Donau bei 30°C! #Rust 2026.";
    let def_tokens = DefaultTokenizer.tokenize(sample_text);
    let exp_def_json = fs::read_to_string(golden_dir.join("tokenizer_default.json")).unwrap();
    assert_eq!(
        serde_json::to_string_pretty(&def_tokens).unwrap(),
        exp_def_json.trim()
    );

    let ger_tokens = GermanMorphTokenizer::try_new()
        .unwrap()
        .tokenize(sample_text);
    let exp_ger_json = fs::read_to_string(golden_dir.join("tokenizer_german.json")).unwrap();
    assert_eq!(
        serde_json::to_string_pretty(&ger_tokens).unwrap(),
        exp_ger_json.trim()
    );

    // 3. BM25 ranking
    let mut scores = Vec::new();
    let n = 100;
    let avg_dl = 150.0;
    for (doc_id, tf, doc_len, df) in [
        (1u64, 3u32, 120u32, 5u32),
        (2u64, 3u32, 120u32, 5u32),
        (3u64, 1u32, 80u32, 10u32),
        (4u64, 5u32, 200u32, 2u32),
    ] {
        let score = score_term(tf, doc_len, avg_dl, df, n);
        scores.push((doc_id, score));
    }
    scores.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    let exp_ranking_json = fs::read_to_string(golden_dir.join("bm25_ranking.json")).unwrap();
    assert_eq!(
        serde_json::to_string_pretty(&scores).unwrap(),
        exp_ranking_json.trim()
    );

    // 4. DocId serde
    let doc_id = DocId::new(42);
    let doc_id_json = serde_json::to_string(&doc_id).unwrap();
    let exp_doc_id_json = fs::read_to_string(golden_dir.join("doc_id.json")).unwrap();
    assert_eq!(doc_id_json, exp_doc_id_json.trim());

    let doc_id_bin = bincode::serialize(&doc_id).unwrap();
    let exp_doc_id_bin = fs::read(golden_dir.join("doc_id.bin")).unwrap();
    assert_eq!(doc_id_bin, exp_doc_id_bin);

    let doc_key = DocKey::from(42u64);
    let exp_doc_key_bin = fs::read(golden_dir.join("doc_key.bin")).unwrap();
    assert_eq!(doc_key.0.to_le_bytes(), exp_doc_key_bin.as_slice());

    // 5. Clock reference
    let clock = FrozenManualClock {
        now_ms: 1_770_000_000_123,
    };
    let exp_clock: u64 =
        serde_json::from_str(&fs::read_to_string(golden_dir.join("clock_reference.json")).unwrap())
            .unwrap();
    assert_eq!(clock.now_unix_ms(), exp_clock);
}

#[test]
fn test_golden_manifest_sha256() {
    let manifest_path = Path::new("tests/golden/MANIFEST.sha256");
    if !manifest_path.exists() {
        return;
    }
    let content = fs::read_to_string(manifest_path).unwrap();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Invalid line in MANIFEST.sha256");
        let expected_sha = parts[0];
        let file_name = parts[1];
        let output = std::process::Command::new("sha256sum")
            .arg(Path::new("tests/golden").join(file_name))
            .output()
            .unwrap();
        let calc = String::from_utf8_lossy(&output.stdout);
        let calc_sha = calc.split_whitespace().next().unwrap();
        assert_eq!(expected_sha, calc_sha, "Mismatch for file {file_name}");
    }
}

#[test]
fn test_golden_bitflip_and_truncation_no_panic() {
    let golden_dir = Path::new("tests/golden");
    if !golden_dir.exists() {
        return;
    }
    let orig = fs::read(golden_dir.join("posting_list_v2.bin")).unwrap();

    // 1. Truncation at every byte prefix
    for len in 0..orig.len() {
        let truncated = &orig[..len];
        let res = PostingList::decode_compact(truncated);
        if len < 4 || len < orig.len() {
            assert!(res.is_err(), "Truncated at {len} should return Err");
        }
    }

    // 2. Deterministic bitflip at every byte and bit
    for byte_idx in 0..orig.len() {
        for bit in 0..8 {
            let mut corrupted = orig.clone();
            corrupted[byte_idx] ^= 1 << bit;
            let res = PostingList::decode_compact(&corrupted);
            if let Ok(pl) = res {
                assert!(pl.len() <= 10_000);
            }
        }
    }
}

#[test]
fn test_search_determinism_permutations() {
    use contextra_text::bm25::BM25;
    let bm25 = BM25::default();

    let docs = vec![
        (10u64, 3u32, 100u32),
        (20u64, 3u32, 100u32), // Tie in score with doc 10
        (30u64, 1u32, 50u32),
        (40u64, 4u32, 120u32),
    ];

    let score_fn = |list: &[(u64, u32, u32)]| -> Vec<(u64, f32)> {
        let mut scored: Vec<(u64, f32)> = list
            .iter()
            .map(|&(id, tf, dl)| (id, bm25.score_term(tf, dl, 100.0, 5, 50)))
            .collect();
        // Deterministic sort: score desc, id asc
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        scored
    };

    let canonical = score_fn(&docs);

    // Permutation 1: reverse
    let mut perm1 = docs.clone();
    perm1.reverse();
    assert_eq!(
        score_fn(&perm1),
        canonical,
        "Permutation 1 must match canonical order"
    );

    // Permutation 2: rotated
    let mut perm2 = docs.clone();
    perm2.rotate_left(2);
    assert_eq!(
        score_fn(&perm2),
        canonical,
        "Permutation 2 must match canonical order"
    );
}
