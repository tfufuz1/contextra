//! Pre-Submit Gate Executor & Hard-Gate Ledger System
//!
//! Führt alle Schritte der Phase 6 (Rebase-Check, Full Gate Chain, Worktree-Bound Ledger,
//! Phantom-File-Check, Claim-Release, Stale-Claim-Cleanup) aus.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct GateStep {
    pub name: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GateLedgerEntry {
    pub timestamp_utc: String,
    pub worktree_hash: String,
    pub mode: String,
    pub steps: Vec<GateStep>,
    pub overall_status: String,
}

pub fn get_ledger_path() -> PathBuf {
    let root = crate::find_root_dir();
    root.join(".jules").join("gate_ledger.json")
}

pub fn git_stash_create_hash() -> String {
    let output = Command::new("git")
        .arg("stash")
        .arg("create")
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if hash.is_empty() {
                // If working tree has no uncommitted changes relative to HEAD, get HEAD commit hash
                let head_out = Command::new("git")
                    .args(["rev-parse", "HEAD"])
                    .output();
                if let Ok(h_out) = head_out {
                    String::from_utf8_lossy(&h_out.stdout).trim().to_string()
                } else {
                    "UNKNOWN_WORKTREE_HASH".to_string()
                }
            } else {
                hash
            }
        }
        _ => "UNKNOWN_WORKTREE_HASH".to_string(),
    }
}

pub fn iso8601_now() -> String {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    format!("{}", since_epoch.as_secs())
}

pub fn write_ledger(entry: &GateLedgerEntry) -> Result<(), String> {
    let path = get_ledger_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(entry)
        .map_err(|e| format!("Serialization error for ledger: {e}"))?;
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write ledger at {}: {e}", path.display()))
}

pub fn read_ledger() -> Option<GateLedgerEntry> {
    let path = get_ledger_path();
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn verify_ledger_matches_worktree() -> bool {
    let current_hash = git_stash_create_hash();
    match read_ledger() {
        Some(entry)
            if entry.overall_status == "PASS"
                && entry.mode == "full"
                && entry.worktree_hash == current_hash =>
        {
            true
        }
        Some(entry) => {
            eprintln!("❌ [SUBMIT-GATE]: Ledger ungültig oder veraltet.");
            eprintln!("   Ledger status: {}, mode: {}, worktree_hash: {}", entry.overall_status, entry.mode, entry.worktree_hash);
            eprintln!("   Aktueller Worktree-Hash: {}", current_hash);
            false
        }
        None => {
            eprintln!("❌ [SUBMIT-GATE]: Kein Ledger (.jules/gate_ledger.json) gefunden.");
            false
        }
    }
}

pub fn run_full_gate_chain() -> GateLedgerEntry {
    println!("=== Running Full Hard-Gate Chain (`cargo xtask pre-push`) ===");
    let mut steps = Vec::new();
    let mut overall_ok = true;

    // 1. fmt check
    println!("→ Step 1/8: cargo fmt --check");
    let fmt_status = Command::new("cargo")
        .args(["fmt", "--all", "--", "--check"])
        .status();
    let fmt_ok = matches!(fmt_status, Ok(s) if s.success());
    steps.push(GateStep {
        name: "fmt".to_string(),
        status: if fmt_ok { "PASS".into() } else { "FAIL".into() },
        detail: None,
    });
    if !fmt_ok { overall_ok = false; }

    // 2. clippy
    println!("→ Step 2/8: cargo clippy --workspace --all-targets --locked");
    let clippy_status = Command::new("cargo")
        .args(["clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"])
        .status();
    let clippy_ok = matches!(clippy_status, Ok(s) if s.success());
    steps.push(GateStep {
        name: "clippy".to_string(),
        status: if clippy_ok { "PASS".into() } else { "FAIL".into() },
        detail: None,
    });
    if !clippy_ok { overall_ok = false; }

    // 3. test
    println!("→ Step 3/8: cargo test --workspace --exclude contextra-py --locked");
    let test_status = Command::new("cargo")
        .args(["test", "--workspace", "--exclude", "contextra-py", "--locked"])
        .status();
    let test_ok = matches!(test_status, Ok(s) if s.success());
    steps.push(GateStep {
        name: "test".to_string(),
        status: if test_ok { "PASS".into() } else { "FAIL".into() },
        detail: None,
    });
    if !test_ok { overall_ok = false; }

    // 4. check-ring-layering
    println!("→ Step 4/8: xtask check-ring-layering");
    let ring_ok = match crate::check_ring_layering::run_check_ring_layering_full(false) {
        Ok(passed) => passed,
        Err(_) => false,
    };
    steps.push(GateStep {
        name: "check-ring-layering".to_string(),
        status: if ring_ok { "PASS".into() } else { "FAIL".into() },
        detail: None,
    });
    if !ring_ok { overall_ok = false; }

    // 5. check-vetoes
    println!("→ Step 5/8: xtask check-vetoes");
    let vetoes_ok = crate::check_vetoes::check_vetoes().is_ok();
    steps.push(GateStep {
        name: "check-vetoes".to_string(),
        status: if vetoes_ok { "PASS".into() } else { "FAIL".into() },
        detail: None,
    });
    if !vetoes_ok { overall_ok = false; }

    // 6. check-agents-integrity
    println!("→ Step 6/8: xtask check-agents-integrity");
    let agents_ok = crate::check_agents_integrity::run_check_agents_integrity();
    steps.push(GateStep {
        name: "check-agents-integrity".to_string(),
        status: if agents_ok { "PASS".into() } else { "FAIL".into() },
        detail: None,
    });
    if !agents_ok { overall_ok = false; }

    // 7. check-phantom-files
    println!("→ Step 7/8: xtask check-phantom-files");
    let phantom_ok = crate::check_phantom_files::run_check_phantom_files();
    steps.push(GateStep {
        name: "check-phantom-files".to_string(),
        status: if phantom_ok { "PASS".into() } else { "FAIL".into() },
        detail: None,
    });
    if !phantom_ok { overall_ok = false; }

    // 8. debt-audit
    println!("→ Step 8/8: xtask debt-audit");
    let debt_ok = crate::gates::debt_audit::run_debt_audit().is_ok();
    steps.push(GateStep {
        name: "debt-audit".to_string(),
        status: if debt_ok { "PASS".into() } else { "FAIL".into() },
        detail: None,
    });
    if !debt_ok { overall_ok = false; }

    let worktree_hash = git_stash_create_hash();
    let entry = GateLedgerEntry {
        timestamp_utc: iso8601_now(),
        worktree_hash,
        mode: "full".to_string(),
        steps,
        overall_status: if overall_ok { "PASS".to_string() } else { "FAIL".to_string() },
    };

    if let Err(e) = write_ledger(&entry) {
        eprintln!("⚠️ Konnte Gate-Ledger nicht schreiben: {e}");
    } else {
        println!("✅ Gate Ledger erfolgreich geschrieben unter {}", get_ledger_path().display());
    }

    entry
}

pub fn run_pre_push() -> bool {
    let entry = run_full_gate_chain();
    if entry.overall_status == "PASS" {
        println!("✅ [PRE-PUSH]: Alle Hard-Gates bestanden. Push freigegeben.");
        true
    } else {
        eprintln!("❌ [PRE-PUSH]: Mindestens ein Hard-Gate ist rot. Push verweigert.");
        false
    }
}

pub fn run_jules_submit_gate(crate_name: Option<&str>) -> bool {
    println!("=== Phase 6: Jules Pre-Submit Gate ===");
    let mut all_passed = true;

    // ── 6.1 REBASE-CHECK ──────────────────────────────────────────────────
    println!("→ [6.1 Rebase-Check]: git fetch origin main...");
    let fetch_status = Command::new("git")
        .args(["fetch", "origin", "main"])
        .status();

    let rebase_ok = match fetch_status {
        Ok(st) if st.success() => {
            let ancestor_status = Command::new("git")
                .args(["merge-base", "--is-ancestor", "origin/main", "HEAD"])
                .status();
            match ancestor_status {
                Ok(ast) if ast.success() => true,
                _ => false,
            }
        }
        _ => false,
    };

    if rebase_ok {
        println!("✅ [6.1 Rebase-Check]: Branch ist aktuell gegenüber origin/main.");
    } else {
        eprintln!("❌ [6.1 Rebase-Check]: Branch ist nicht aktuell gegenüber origin/main.");
        eprintln!("   Bitte 'git rebase origin/main' ausführen und erneut versuchen.");
        all_passed = false;
    }

    // ── 6.2 HARD-GATE LEDGER VERIFIKATION ──────────────────────────────
    println!("→ [6.2 Hard-Gate-Ledger-Check]: Prüfe worktree-gebundenen PASS-Ledger...");
    if verify_ledger_matches_worktree() {
        println!("✅ [6.2 Hard-Gate-Ledger-Check]: Gültiger PASS-Ledger für aktuellen Worktree verifiziert.");
    } else {
        eprintln!("❌ [6.2 Hard-Gate-Ledger-Check]: Kein gültiger PASS-Ledger für den aktuellen Worktree-Stand.");
        eprintln!("   Führe zuerst `cargo xtask pre-push` aus.");
        all_passed = false;
    }

    // ── 6.3 PHANTOM-FILE-CHECK ────────────────────────────────────────────
    println!("→ [6.3 Phantom-File-Check]: Prüfe behauptete Dateien im Diff...");
    if crate::check_phantom_files::run_check_phantom_files() {
        println!("✅ [6.3 Phantom-File-Check]: Keine Phantom-Dateien gefunden.");
    } else {
        eprintln!("❌ [6.3 Phantom-File-Check]: Phantom-Dateien erkannt.");
        all_passed = false;
    }

    // Abbrechen, falls ein Pflichtschritt (6.1–6.3) fehlgeschlagen ist
    if !all_passed {
        eprintln!("❌ SUBMIT GATE FEHLEGESCHLAGEN — Vor 6.4/6.5 abgebaut.");
        return false;
    }

    // ── 6.4 CLAIM-RELEASE ────────────────────────────────────────────────
    if let Some(c) = crate_name {
        println!(
            "→ [6.4 Claim-Release]: Gebe Claim für Crate '{}' frei...",
            c
        );
        let release_ok = crate::claim::run_release_local(&["--crate".to_string(), c.to_string()]);
        if release_ok {
            println!(
                "✅ [6.4 Claim-Release]: Claim für '{}' erfolgreich freigegeben.",
                c
            );
        } else {
            eprintln!(
                "❌ [6.4 Claim-Release]: Freigabe für '{}' fehlgeschlagen.",
                c
            );
            all_passed = false;
        }
    } else {
        println!("ℹ️ [6.4 Claim-Release]: Übersprungen (kein --crate angegeben).");
    }

    // ── 6.5 STALE-CLAIM CLEANUP (P0-1) ────────────────────────────────────
    println!("→ [6.5 Stale-Claim-Cleanup]: Führe Bereinigung abgelaufener Claims aus...");
    let cleanup_ok = crate::claim::run_claim(&["--prune".to_string()]);
    if cleanup_ok {
        println!("✅ [6.5 Stale-Claim-Cleanup]: Abgelaufene Claims bereinigt.");
    } else {
        eprintln!("⚠️ [6.5 Stale-Claim-Cleanup]: Warnung bei Claim-Bereinigung.");
    }

    if all_passed {
        println!("════════════════════════════════════════════════");
        println!("✅ SUBMIT GATE BESTANDEN — Submit erlaubt.");
        println!("════════════════════════════════════════════════");
        true
    } else {
        eprintln!("❌ SUBMIT GATE FEHLEGESCHLAGEN.");
        false
    }
}
