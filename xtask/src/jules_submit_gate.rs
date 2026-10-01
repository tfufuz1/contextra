//! Pre-Submit Gate Executor & Hard-Gate Ledger System
//!
//! Führt alle Schritte der Phase 6 (Rebase-Check, Full Gate Chain, Worktree-Bound Ledger,
//! Phantom-File-Check, Claim-Release, Stale-Claim-Cleanup) aus.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

pub fn submit_gate_tree_hash() -> String {
    submit_gate_tree_hash_in_dir(None)
}

pub fn submit_gate_tree_hash_in_dir(workdir: Option<&std::path::Path>) -> String {
    let temp_index = match tempfile::NamedTempFile::new() {
        Ok(t) => t,
        Err(_) => return "UNKNOWN_WORKTREE_HASH".to_string(),
    };
    let temp_path = temp_index.path().to_path_buf();

    let mut cmd_read = Command::new("git");
    cmd_read.env("GIT_INDEX_FILE", &temp_path);
    if let Some(dir) = workdir {
        cmd_read.current_dir(dir);
    }
    let _ = cmd_read.args(["read-tree", "HEAD"]).output();

    let mut cmd_add = Command::new("git");
    cmd_add.env("GIT_INDEX_FILE", &temp_path);
    if let Some(dir) = workdir {
        cmd_add.current_dir(dir);
    }
    let add_out = cmd_add.args(["add", "-A"]).output();

    if let Ok(out) = add_out {
        if !out.status.success() {
            return "UNKNOWN_WORKTREE_HASH".to_string();
        }
    } else {
        return "UNKNOWN_WORKTREE_HASH".to_string();
    }

    let mut cmd_tree = Command::new("git");
    cmd_tree.env("GIT_INDEX_FILE", &temp_path);
    if let Some(dir) = workdir {
        cmd_tree.current_dir(dir);
    }
    let tree_out = cmd_tree.args(["write-tree"]).output();

    match tree_out {
        Ok(out) if out.status.success() => {
            let hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if hash.is_empty() {
                "UNKNOWN_WORKTREE_HASH".to_string()
            } else {
                hash
            }
        }
        _ => "UNKNOWN_WORKTREE_HASH".to_string(),
    }
}

pub fn iso8601_now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

pub fn write_ledger(entry: &GateLedgerEntry) -> Result<(), String> {
    let path = get_ledger_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(entry)
        .map_err(|e| format!("Serialization error for ledger: {e}"))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write ledger at {}: {e}", path.display()))
}

pub fn read_ledger() -> Option<GateLedgerEntry> {
    let path = get_ledger_path();
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn verify_ledger_matches_worktree() -> bool {
    let current_hash = submit_gate_tree_hash();
    if current_hash == "UNKNOWN_WORKTREE_HASH" {
        eprintln!("❌ [SUBMIT-GATE]: Worktree-Hash konnte nicht ermittelt werden.");
        return false;
    }
    match read_ledger() {
        Some(entry)
            if entry.overall_status == "PASS"
                && entry.mode == "full"
                && entry.worktree_hash == current_hash =>
        {
            println!("ℹ️ [SUBMIT-GATE]: Lokaler (beratender) PASS-Ledger verifiziert.");
            true
        }
        Some(entry) => {
            eprintln!("❌ [SUBMIT-GATE]: Lokaler (beratender) Ledger ungültig oder veraltet.");
            eprintln!(
                "   Ledger status: {}, mode: {}, worktree_hash: {}",
                entry.overall_status, entry.mode, entry.worktree_hash
            );
            eprintln!("   Aktueller Worktree-Hash: {}", current_hash);
            false
        }
        None => {
            eprintln!("❌ [SUBMIT-GATE]: Kein lokaler (beratender) Ledger (.jules/gate_ledger.json) gefunden.");
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
    if !fmt_ok {
        overall_ok = false;
    }

    // 2. clippy
    println!("→ Step 2/8: cargo clippy --workspace --all-targets --locked");
    let clippy_status = Command::new("cargo")
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ])
        .status();
    let clippy_ok = matches!(clippy_status, Ok(s) if s.success());
    steps.push(GateStep {
        name: "clippy".to_string(),
        status: if clippy_ok {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        detail: None,
    });
    if !clippy_ok {
        overall_ok = false;
    }

    // 3. test
    println!("→ Step 3/8: cargo test --workspace --exclude contextra-py --locked");
    let test_status = Command::new("cargo")
        .args([
            "test",
            "--workspace",
            "--exclude",
            "contextra-py",
            "--locked",
        ])
        .status();
    let test_ok = matches!(test_status, Ok(s) if s.success());
    steps.push(GateStep {
        name: "test".to_string(),
        status: if test_ok {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        detail: None,
    });
    if !test_ok {
        overall_ok = false;
    }

    // 4. check-ring-layering
    println!("→ Step 4/8: xtask check-ring-layering");
    let ring_ok =
        crate::check_ring_layering::run_check_ring_layering_full(false).unwrap_or_default();
    steps.push(GateStep {
        name: "check-ring-layering".to_string(),
        status: if ring_ok {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        detail: None,
    });
    if !ring_ok {
        overall_ok = false;
    }

    // 5. check-vetoes
    println!("→ Step 5/8: xtask check-vetoes");
    let vetoes_ok = crate::check_vetoes::check_vetoes().is_ok();
    steps.push(GateStep {
        name: "check-vetoes".to_string(),
        status: if vetoes_ok {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        detail: None,
    });
    if !vetoes_ok {
        overall_ok = false;
    }

    // 6. check-agents-integrity
    println!("→ Step 6/8: xtask check-agents-integrity");
    let agents_ok = crate::check_agents_integrity::run_check_agents_integrity();
    steps.push(GateStep {
        name: "check-agents-integrity".to_string(),
        status: if agents_ok {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        detail: None,
    });
    if !agents_ok {
        overall_ok = false;
    }

    // 7. check-phantom-files
    println!("→ Step 7/8: xtask check-phantom-files");
    let phantom_ok = crate::check_phantom_files::run_check_phantom_files();
    steps.push(GateStep {
        name: "check-phantom-files".to_string(),
        status: if phantom_ok {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        detail: None,
    });
    if !phantom_ok {
        overall_ok = false;
    }

    // 8. debt-audit
    println!("→ Step 8/8: xtask debt-audit");
    let debt_ok = crate::gates::debt_audit::run_debt_audit().is_ok();
    steps.push(GateStep {
        name: "debt-audit".to_string(),
        status: if debt_ok {
            "PASS".into()
        } else {
            "FAIL".into()
        },
        detail: None,
    });
    if !debt_ok {
        overall_ok = false;
    }

    let worktree_hash = submit_gate_tree_hash();
    let entry = GateLedgerEntry {
        timestamp_utc: iso8601_now(),
        worktree_hash,
        mode: "full".to_string(),
        steps,
        overall_status: if overall_ok {
            "PASS".to_string()
        } else {
            "FAIL".to_string()
        },
    };

    if let Err(e) = write_ledger(&entry) {
        eprintln!("⚠️ Konnte Gate-Ledger nicht schreiben: {e}");
    } else {
        println!(
            "✅ Gate Ledger erfolgreich geschrieben unter {}",
            get_ledger_path().display()
        );
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
            matches!(ancestor_status, Ok(ast) if ast.success())
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
    println!("→ [6.2 Hard-Gate-Ledger-Check]: Prüfe worktree-gebundenen PASS-Ledger (lokal, beratend)...");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso8601_now_rfc3339() {
        let ts = iso8601_now();
        let parsed = chrono::DateTime::parse_from_rfc3339(&ts);
        assert!(
            parsed.is_ok(),
            "Timestamp {} should be valid RFC-3339",
            ts
        );
    }

    #[test]
    fn test_submit_gate_tree_hash_changes() {
        let temp_dir = tempfile::tempdir().expect("Failed to create tempdir");
        let dir = temp_dir.path();

        let run_git = |args: &[&str]| {
            let status = Command::new("git")
                .current_dir(dir)
                .args(args)
                .status()
                .expect("Failed to run git");
            assert!(status.success());
        };

        run_git(&["init"]);
        run_git(&["config", "user.name", "Test"]);
        run_git(&["config", "user.email", "test@example.com"]);

        fs::write(dir.join("tracked.txt"), "hello").unwrap();
        run_git(&["add", "tracked.txt"]);
        run_git(&["commit", "-m", "initial commit"]);

        let hash_base = submit_gate_tree_hash_in_dir(Some(dir));
        assert_ne!(hash_base, "UNKNOWN_WORKTREE_HASH");

        // Adding an ignored file shouldn't change the tree hash
        fs::write(dir.join(".gitignore"), "*.log\n").unwrap();
        run_git(&["add", ".gitignore"]);
        run_git(&["commit", "-m", "add gitignore"]);
        let hash_with_ignore = submit_gate_tree_hash_in_dir(Some(dir));

        fs::write(dir.join("ignored.log"), "log content").unwrap();
        let hash_with_ignored_file = submit_gate_tree_hash_in_dir(Some(dir));
        assert_eq!(
            hash_with_ignore, hash_with_ignored_file,
            "Ignored file must not alter tree hash"
        );

        // Untracked file changes tree hash
        fs::write(dir.join("untracked.txt"), "new file").unwrap();
        let hash_untracked = submit_gate_tree_hash_in_dir(Some(dir));
        assert_ne!(
            hash_with_ignore, hash_untracked,
            "Untracked file must alter tree hash"
        );

        // Tracked file modification changes tree hash
        fs::write(dir.join("tracked.txt"), "hello modified").unwrap();
        let hash_modified = submit_gate_tree_hash_in_dir(Some(dir));
        assert_ne!(
            hash_untracked, hash_modified,
            "Modifying tracked file must alter tree hash"
        );
    }
}
