// FILE-CONTEXT
// STAND: 2026-11-10
// ZWECK: Integrationstest für contextra-router (golden_corpus_test.rs).
// INVARIANTEN: Deterministische Ausführung, Zero-Panic, korrekte Fehlertaxonomie.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::cast_possible_truncation
)]
// FILE-CONTEXT
// ZWECK: Golden-/Orakel-Korpus-Verifikation für contextra-router (RT2.1, INV-5).
// INVARIANTEN: Bitgleiche Determinismus-Prüfung gegen MANIFEST.sha256, Panik-freies Rückwärtslesen.

use contextra_router::{ArmRegistry, DecisionId, DecisionIdGenerator, RoutingOutcome, SlmProfile};
use contextra_types::{ContextChunk, ContextWindow, DocId, TokenBudget};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ArmMappingEntry {
    arm: u32,
    strategy_debug: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DecisionEntry {
    step: u64,
    decision_id: u64,
    context_chunks_count: usize,
    budget_remaining: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ProfilesAndOutcomesData {
    profiles: Vec<SlmProfile>,
    outcomes: Vec<RoutingOutcome>,
    decision_ids: Vec<DecisionId>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DispatchMessageEntry {
    profile_name: String,
    jsonrpc: String,
    method: String,
    payload_hash: String,
}

fn compute_sha256_hex(bytes: &[u8]) -> String {
    let mut child = Command::new("sha256sum")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn sha256sum");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(bytes)
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait sha256sum");
    let out_str = String::from_utf8(output.stdout).expect("utf8");
    out_str.split_whitespace().next().unwrap_or("").to_string()
}

fn golden_dir() -> &'static Path {
    Path::new("tests/golden")
}

#[test]
#[ignore]
fn test_generate_golden_corpus() {
    let dir = golden_dir();
    fs::create_dir_all(dir).expect("create golden dir");

    // 1. Arm-Registry
    let reg = ArmRegistry::default();
    let mut arm_entries = Vec::new();
    for arm in 0..5 {
        let strat = reg.strategy_for(arm).expect("valid strategy");
        arm_entries.push(ArmMappingEntry {
            arm,
            strategy_debug: format!("{strat:?}"),
        });
    }
    let arm_json = serde_json::to_string_pretty(&arm_entries).expect("serialize arm entries");
    fs::write(dir.join("arm_registry.json"), arm_json + "\n").expect("write arm_registry.json");

    // 2. Deterministische Entscheidungsfolgen (feste Seeds / Monotonie)
    let gen = DecisionIdGenerator::new(42);
    let mut decisions = Vec::new();
    for step in 0..10 {
        let dec_id = gen.next();
        decisions.push(DecisionEntry {
            step,
            decision_id: dec_id.inner(),
            context_chunks_count: (step as usize) % 4,
            budget_remaining: 1000 - step * 50,
        });
    }
    let dec_json = serde_json::to_string_pretty(&decisions).expect("serialize decisions");
    fs::write(dir.join("routing_decisions.json"), dec_json + "\n")
        .expect("write routing_decisions.json");

    // 3. Profiles und Outcomes
    let p1 = SlmProfile::new(
        "mock-fast",
        "bin/mock-fast",
        vec![1, 2],
        TokenBudget::new(2048, 128),
        0.6,
    );
    let p2 = SlmProfile::new(
        "mock-accurate",
        "bin/mock-accurate",
        vec![3],
        TokenBudget::new(8192, 256),
        0.85,
    );
    let profiles = vec![p1, p2];

    let outcomes = vec![
        RoutingOutcome::Success,
        RoutingOutcome::Rejected {
            reason: Some("Token budget exceeded".to_string()),
        },
        RoutingOutcome::Escalated {
            escalated_to: "mock-accurate".to_string(),
        },
    ];

    let decision_ids = vec![
        DecisionId::from_raw(1),
        DecisionId::from_raw(2),
        DecisionId::from_raw(100),
    ];
    let po_data = ProfilesAndOutcomesData {
        profiles,
        outcomes,
        decision_ids,
    };
    let po_json = serde_json::to_string_pretty(&po_data).expect("serialize profiles and outcomes");
    fs::write(dir.join("profiles_and_outcomes.json"), po_json + "\n")
        .expect("write profiles_and_outcomes.json");

    // 4. Dispatch-Nachrichten
    let mut dispatch_entries = Vec::new();
    for p in &po_data.profiles {
        let cw = ContextWindow {
            chunks: vec![ContextChunk {
                doc_id: DocId::new(42),
                content: "test-chunk-content".to_string(),
                relevance: 0.9,
                token_count: 64,
                metadata: None,
                contextual_prefix: None,
                links: vec![],
            }],
            total_tokens: 64,
            truncated: false,
        };
        let bytes = serde_json::to_vec(&cw).expect("cw serialize");
        let hash = compute_sha256_hex(&bytes);
        dispatch_entries.push(DispatchMessageEntry {
            profile_name: p.name.clone(),
            jsonrpc: "2.0".to_string(),
            method: "slm_process_context".to_string(),
            payload_hash: hash,
        });
    }
    let disp_json =
        serde_json::to_string_pretty(&dispatch_entries).expect("serialize dispatch entries");
    fs::write(dir.join("dispatch_messages.json"), disp_json + "\n")
        .expect("write dispatch_messages.json");

    println!("Golden files generated in {:?}", dir);
}

#[test]
fn test_verify_golden_corpus_checksums() {
    let dir = golden_dir();
    let manifest_path = dir.join("MANIFEST.sha256");
    assert!(
        manifest_path.exists(),
        "MANIFEST.sha256 must exist in tests/golden"
    );

    let content = fs::read_to_string(&manifest_path).expect("read MANIFEST.sha256");
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        assert_eq!(parts.len(), 2, "Invalid line in MANIFEST.sha256: {line}");
        let expected_hash = parts.first().copied().unwrap_or("");
        let file_name = parts.get(1).copied().unwrap_or("");
        let file_path = dir.join(file_name);
        assert!(
            file_path.exists(),
            "File {file_name} in manifest does not exist"
        );

        let bytes = fs::read(&file_path).expect("read golden file");
        let actual_hash = compute_sha256_hex(&bytes);
        assert_eq!(
            actual_hash, expected_hash,
            "Bitwise mismatch in golden file {file_name}!"
        );
    }
}

#[test]
fn test_golden_corpus_backward_reader() {
    let dir = golden_dir();

    // 1. arm_registry.json
    let arm_bytes = fs::read(dir.join("arm_registry.json")).expect("read arm_registry.json");
    let arm_entries: Vec<ArmMappingEntry> =
        serde_json::from_slice(&arm_bytes).expect("parse arm_registry.json");
    assert_eq!(arm_entries.len(), 5);

    // 2. routing_decisions.json
    let dec_bytes =
        fs::read(dir.join("routing_decisions.json")).expect("read routing_decisions.json");
    let dec_entries: Vec<DecisionEntry> =
        serde_json::from_slice(&dec_bytes).expect("parse routing_decisions.json");
    assert_eq!(dec_entries.len(), 10);

    // 3. profiles_and_outcomes.json
    let po_bytes =
        fs::read(dir.join("profiles_and_outcomes.json")).expect("read profiles_and_outcomes.json");
    let po_data: ProfilesAndOutcomesData =
        serde_json::from_slice(&po_bytes).expect("parse profiles_and_outcomes.json");
    assert_eq!(po_data.profiles.len(), 2);
    assert_eq!(po_data.outcomes.len(), 3);
    assert_eq!(po_data.decision_ids.len(), 3);

    // 4. dispatch_messages.json
    let disp_bytes =
        fs::read(dir.join("dispatch_messages.json")).expect("read dispatch_messages.json");
    let disp_entries: Vec<DispatchMessageEntry> =
        serde_json::from_slice(&disp_bytes).expect("parse dispatch_messages.json");
    assert_eq!(disp_entries.len(), 2);
}

#[test]
fn test_golden_corpus_corruption_resilience_err_never_panic() {
    let dir = golden_dir();
    let files = [
        "arm_registry.json",
        "routing_decisions.json",
        "profiles_and_outcomes.json",
        "dispatch_messages.json",
    ];

    for file_name in files {
        let file_path = dir.join(file_name);
        let orig_bytes = fs::read(&file_path).expect("read original golden file");

        // 1. Truncation Tests
        for cut in [0, 1, 5, orig_bytes.len() / 2] {
            if cut >= orig_bytes.len() {
                continue;
            }
            let truncated = &orig_bytes[..cut];
            let res: Result<serde_json::Value, _> = serde_json::from_slice(truncated);
            assert!(
                res.is_err() || cut == 0,
                "Truncated file {file_name} at {cut} should error"
            );
        }

        // 2. Bitflip Mutation Tests
        for flip_pos in [0, 10, orig_bytes.len().saturating_sub(1)] {
            if flip_pos < orig_bytes.len() {
                let mut mutated = orig_bytes.clone();
                if let Some(byte) = mutated.get_mut(flip_pos) {
                    *byte ^= 0xFF;
                }
                let res: Result<serde_json::Value, _> = serde_json::from_slice(&mutated);
                let _ = res; // May error or parse as invalid structure, but MUST never panic
            }
        }
    }
}
