// Contextra — Deletion & Witness Gates Implementation
// Gate 5: witness-mint (INV-WITNESS-MINT-1)
// Gate 6: deletion-types-once (REQ-DEL-027)
// Gate 7: deletion-honesty (INV-CLAIMS-1, AC-10, AC-19)
// Gate 8: no-dummy-proof (INV-NO-DUMMY-PROOF)
// Gate 9: scanner-coverage (REQ-SCANNER-COV)

use crate::core::ctx::Context;
use crate::core::gate::{Gate, GateSpec, Outcome};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// ── Gate 5: WitnessMintGate ──────────────────────────────────────────────────

pub struct WitnessMintGate;

impl Gate for WitnessMintGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "witness-mint".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH1".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["INV-WITNESS-MINT-1".to_string()],
            why: "Prüft Erzeugung und Signatur von Löschzeugen; LayerWitness darf nur in ScanSession::finish entstehen (SPEC §12.7.4).".to_string(),
            owner: "MP-08".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_witness_mint(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "witness-mint" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }

        let temp = match tempfile::TempDir::new() {
            Ok(t) => t,
            Err(e) => {
                return Outcome::Fail {
                    reason: format!("Failed to create tempdir for selftest: {e}"),
                };
            }
        };
        let root = temp.path();

        // 1. Positive control: clean allowed module in contextra-ports/src/scanner
        if let Err(e) = fs::create_dir_all(root.join("crates/contextra-ports/src/scanner")) {
            return Outcome::Fail {
                reason: format!("Failed to create scanner dir in selftest: {e}"),
            };
        }
        if let Err(e) = fs::write(
            root.join("crates/contextra-ports/src/scanner/mod.rs"),
            "let w = LayerWitness { remaining_count: 0 };",
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write clean fixture in selftest: {e}"),
            };
        }
        if scan_witness_mint(root) != Outcome::Pass {
            return Outcome::Fail {
                reason: "Selftest failed: scan_witness_mint rejected legal LayerWitness in scanner"
                    .to_string(),
            };
        }

        // 2. Negative control: illegal module in contextra-engine
        if let Err(e) = fs::create_dir_all(root.join("crates/contextra-engine/src")) {
            return Outcome::Fail {
                reason: format!("Failed to create dummy crate dir in selftest: {e}"),
            };
        }
        if let Err(e) = fs::write(
            root.join("crates/contextra-engine/src/bad.rs"),
            "let w = LayerWitness { remaining_count: 0 };",
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write bad fixture in selftest: {e}"),
            };
        }

        match scan_witness_mint(root) {
            Outcome::Fail { reason } if reason.contains("LayerWitness") => Outcome::Pass,
            _ => Outcome::Fail {
                reason: "Selftest failed: scan_witness_mint failed to detect illegal LayerWitness instantiation in contextra-engine".to_string(),
            },
        }
    }
}

pub fn scan_witness_mint(root: &Path) -> Outcome {
    let mut violations = Vec::new();

    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return Outcome::Pass;
    }

    for entry in WalkDir::new(&crates_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        };

        // Allowed modules: contextra-ports/src/scanner, contextra-testkit
        if rel_path.contains("contextra-ports/src/scanner")
            || rel_path.contains("contextra-testkit")
        {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.contains("LayerWitness {")
                || trimmed.contains("LayerWitness::new")
                || trimmed.contains("issue_from_scanner")
                || trimmed.contains("mint_witness_for_testkit")
            {
                violations.push(format!(
                    "{}:{} - Direct instantiation of LayerWitness outside allowed modules: '{}'",
                    rel_path, line_num, trimmed
                ));
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "witness-mint check failed with {} violation(s):\n{}",
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

// ── Gate 6: DeletionTypesOnceGate ────────────────────────────────────────────

pub struct DeletionTypesOnceGate;

impl Gate for DeletionTypesOnceGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "deletion-types-once".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH1".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["REQ-DEL-027".to_string()],
            why: "Stellt sicher, dass Löschtypen (DeletionLayer, DeletionProof, LayerCleanupProof) nicht doppelt definiert werden.".to_string(),
            owner: "MP-08".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_deletion_types_once(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "deletion-types-once" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub fn scan_deletion_types_once(root: &Path) -> Outcome {
    let mut violations = Vec::new();

    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return Outcome::Pass;
    }

    for entry in WalkDir::new(&crates_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        };

        // Canonical locations: contextra-types, contextra-ports, contextra-crypto
        if rel_path.contains("contextra-types/src")
            || rel_path.contains("contextra-ports/src")
            || rel_path.contains("contextra-crypto/src")
            || rel_path.contains("/tests/")
            || rel_path.contains("/benches/")
            || rel_path.contains("/examples/")
        {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if (trimmed.starts_with("pub enum DeletionLayer")
                || trimmed.starts_with("enum DeletionLayer"))
                || (trimmed.starts_with("pub struct DeletionProof")
                    || trimmed.starts_with("struct DeletionProof"))
                || (trimmed.starts_with("pub struct LayerCleanupProof")
                    || trimmed.starts_with("struct LayerCleanupProof"))
            {
                violations.push(format!(
                    "{}:{} - Duplicate definition of canonical deletion type: '{}'",
                    rel_path, line_num, trimmed
                ));
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "deletion-types-once check failed with {} violation(s):\n{}",
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

// ── Gate 7: DeletionHonestyGate ──────────────────────────────────────────────

pub struct DeletionHonestyGate;

impl Gate for DeletionHonestyGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "deletion-honesty".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH0".to_string(),
            kind: "builtin".to_string(),
            enforces: vec![
                "INV-CLAIMS-1".to_string(),
                "AC-10".to_string(),
                "AC-19".to_string(),
            ],
            why: "Verifiziert die Korrektheit von Löschansprüchen, fängt ungemessene Außentexte, hardcodierte Zähler 0 und dummy Ok(true) verifications.".to_string(),
            owner: "MP-08".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_deletion_honesty(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "deletion-honesty" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }

        let temp = match tempfile::TempDir::new() {
            Ok(t) => t,
            Err(e) => {
                return Outcome::Fail {
                    reason: format!("Failed to create tempdir for selftest: {e}"),
                };
            }
        };
        let root = temp.path();

        // Positive control: clean doc
        if let Err(e) = fs::write(root.join("README.md"), "# Contextra") {
            return Outcome::Fail {
                reason: format!("Failed to write README.md in selftest: {e}"),
            };
        }
        if scan_deletion_honesty(root) != Outcome::Pass {
            return Outcome::Fail {
                reason: "Selftest failed: scan_deletion_honesty rejected clean doc".to_string(),
            };
        }

        // Positive control: exempted file containing new phrases must NOT trigger a violation
        if let Err(e) = fs::write(
            root.join("docs/SPEC.md"),
            "DSGVO-konform, Art.-17-konform, unwiederbringlich, Löschzertifikat, Löschung nach Art. 17, rechtskonform, rechtssicher",
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write docs/SPEC.md in selftest: {e}"),
            };
        }
        if scan_deletion_honesty(root) != Outcome::Pass {
            return Outcome::Fail {
                reason: "Selftest failed: scan_deletion_honesty triggered violation on new phrases in exempted docs/SPEC.md".to_string(),
            };
        }
        let _ = fs::remove_file(root.join("docs/SPEC.md"));

        // Positive control: "kryptografisch gelöscht" WITH custody / assumption MUST pass
        if let Err(e) = fs::write(
            root.join("README.md"),
            "Das Dokument ist kryptografisch gelöscht unter Annahme A-FLASH mit Custody K1.",
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write README.md in selftest: {e}"),
            };
        }
        if scan_deletion_honesty(root) != Outcome::Pass {
            return Outcome::Fail {
                reason: "Selftest failed: scan_deletion_honesty rejected 'kryptografisch gelöscht' with valid custody/assumption".to_string(),
            };
        }

        // Positive control: line with <!-- claim-ignore --> MUST pass
        if let Err(e) = fs::write(
            root.join("README.md"),
            "100 % gelöscht <!-- claim-ignore -->",
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write README.md in selftest: {e}"),
            };
        }
        if scan_deletion_honesty(root) != Outcome::Pass {
            return Outcome::Fail {
                reason:
                    "Selftest failed: scan_deletion_honesty failed to respect <!-- claim-ignore -->"
                        .to_string(),
            };
        }

        // Negative control test cases for each new prohibited phrase:
        let test_cases = [
            ("100 % gelöscht", "100 % gelöscht"),
            ("unwiederbringlich", "unwiederbringlich"),
            ("DSGVO-konform", "DSGVO-konform"),
            ("Art.-17-konform", "Art.-17-konform"),
            ("rechtskonform gelöscht", "rechtskonform gelöscht"),
            ("rechtskonform", "rechtskonform"),
            ("rechtssicher", "rechtssicher"),
            ("Löschzertifikat", "Löschzertifikat"),
            ("Löschung nach Art. 17", "Löschung nach Art. 17"),
            ("kryptografisch gelöscht", "kryptografisch gelöscht"),
        ];

        for (phrase_name, payload) in test_cases {
            if let Err(e) = fs::write(root.join("README.md"), payload) {
                return Outcome::Fail {
                    reason: format!("Failed to write test payload for {phrase_name}: {e}"),
                };
            }
            match scan_deletion_honesty(root) {
                Outcome::Fail { reason }
                    if reason.contains(phrase_name)
                        || reason.contains(&payload.to_lowercase())
                        || reason.contains("kryptografisch") => {}
                res => {
                    return Outcome::Fail {
                        reason: format!("Selftest failed: scan_deletion_honesty failed to detect forbidden phrase '{phrase_name}' (got {:?}", res),
                    };
                }
            }
        }

        // Negative control: string literal in MCP non-test source file
        if let Err(e) = fs::create_dir_all(root.join("crates/contextra-mcp/src")) {
            return Outcome::Fail {
                reason: format!("Failed to create dummy mcp dir in selftest: {e}"),
            };
        }
        if let Err(e) = fs::write(
            root.join("crates/contextra-mcp/src/bad_mcp_phrase.rs"),
            "let s = \"DSGVO-konform\";",
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write bad MCP phrase fixture in selftest: {e}"),
            };
        }
        match scan_deletion_honesty(root) {
            Outcome::Fail { reason }
                if reason.contains("DSGVO-konform") || reason.contains("dsgvo konform") => {}
            _ => {
                return Outcome::Fail {
                    reason: "Selftest failed: scan_deletion_honesty failed to detect prohibited phrase in MCP source string literal".to_string(),
                };
            }
        }
        let _ = fs::remove_file(root.join("crates/contextra-mcp/src/bad_mcp_phrase.rs"));

        // Negative control: fake proof in tool response
        if let Err(e) = fs::write(
            root.join("crates/contextra-mcp/src/bad_tool.rs"),
            "json!({ \"proof\": dummy_proof })",
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write bad tool fixture in selftest: {e}"),
            };
        }

        match scan_deletion_honesty(root) {
            Outcome::Fail { reason } if reason.contains("proof") => Outcome::Pass,
            _ => Outcome::Fail {
                reason: "Selftest failed: scan_deletion_honesty failed to detect illegal proof construction in tool response".to_string(),
            },
        }
    }
}

/// Normalizes claim text for deletion honesty checks:
/// - converts to lowercase
/// - collapses spaces before `%` ("100 %" -> "100%")
/// - standardizes article 17 citations ("art.-17", "art. 17", "art.17" -> "art 17")
/// - converts hyphens to spaces ("dsgvo-konform" -> "dsgvo konform")
/// - collapses multiple whitespace sequences
fn normalize_claim_text(text: &str) -> String {
    let lower = text.to_lowercase();
    let norm = lower.replace(" %", "%");
    let norm = norm
        .replace("art.-17", "art 17")
        .replace("art. 17", "art 17")
        .replace("art.17", "art 17");
    let norm = norm.replace('-', " ");
    norm.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_excluded_from_new_phrases(rel_path: &str) -> bool {
    rel_path == "docs/SPEC.md"
        || rel_path == "docs/PLAN.md"
        || rel_path.starts_with("docs/decisions/")
        || rel_path.starts_with("docs/analysis/")
        || rel_path.starts_with("docs/spec/")
}

fn is_target_for_new_phrases(rel_path: &str) -> bool {
    if is_excluded_from_new_phrases(rel_path) {
        return false;
    }
    if rel_path == "README.md"
        || rel_path == "AGENTS.md"
        || rel_path == "CLAUDE.md"
        || rel_path == "docs/ARCHITECTURE.md"
        || rel_path == "docs/SECURITY.md"
        || rel_path == "docs/CONTRIBUTING.md"
    {
        return true;
    }
    if rel_path.starts_with("crates/") && rel_path.ends_with("README.md") {
        return true;
    }
    if rel_path.starts_with("crates/contextra-mcp/src/")
        && rel_path.ends_with(".rs")
        && !rel_path.contains("/tests/")
        && !rel_path.contains("/benches/")
        && !rel_path.contains("/examples/")
    {
        return true;
    }
    false
}

pub fn scan_deletion_honesty(root: &Path) -> Outcome {
    let mut violations = Vec::new();

    // 1. Legacy phrase scan (README.md, AGENTS.md, docs/SPEC.md)
    let legacy_docs = ["README.md", "AGENTS.md", "docs/SPEC.md"];
    for doc in legacy_docs {
        let path = root.join(doc);
        if !path.exists() {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.contains("<!-- claim-ignore -->") {
                continue;
            }

            if trimmed.contains("100% gelöscht")
                || trimmed.contains("garantiert gelöscht")
                || trimmed.contains("unwiederbringlich gelöscht")
            {
                violations.push(format!(
                    "{}:{} - Unmeasured absolute deletion claim found in doc: '{}'",
                    doc, line_num, trimmed
                ));
            }
        }
    }

    // 2. New expanded phrases scan (README.md, AGENTS.md, CLAUDE.md, docs/ARCHITECTURE.md, docs/SECURITY.md, docs/CONTRIBUTING.md, crates/*/README.md, crates/contextra-mcp/src/*.rs)
    // EXCLUDING docs/SPEC.md, docs/PLAN.md, docs/decisions/**, docs/analysis/**, docs/spec/**
    let mut files_to_scan = Vec::new();
    let fixed_docs = [
        "README.md",
        "AGENTS.md",
        "CLAUDE.md",
        "docs/ARCHITECTURE.md",
        "docs/SECURITY.md",
        "docs/CONTRIBUTING.md",
    ];
    for doc in fixed_docs {
        let path = root.join(doc);
        if path.exists() {
            files_to_scan.push((doc.to_string(), path));
        }
    }

    let crates_dir = root.join("crates");
    if crates_dir.exists() {
        for entry in WalkDir::new(&crates_dir).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let rel_path = match path.strip_prefix(root) {
                Ok(p) => p.to_string_lossy().to_string().replace('\\', "/"),
                Err(_) => path.to_string_lossy().to_string().replace('\\', "/"),
            };

            if is_target_for_new_phrases(&rel_path) && !fixed_docs.contains(&rel_path.as_str()) {
                files_to_scan.push((rel_path, path.to_path_buf()));
            }
        }
    }

    for (rel_path, path) in files_to_scan {
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.contains("<!-- claim-ignore -->") {
                continue;
            }

            let norm = normalize_claim_text(line);

            if norm.contains("unwiederbringlich") {
                violations.push(format!(
                    "{}:{} - Prohibited claim 'unwiederbringlich' found in doc/code: '{}'",
                    rel_path, line_num, trimmed
                ));
            } else if norm.contains("dsgvo konform") {
                violations.push(format!(
                    "{}:{} - Prohibited claim 'DSGVO-konform' found in doc/code: '{}'",
                    rel_path, line_num, trimmed
                ));
            } else if norm.contains("art 17 konform") {
                violations.push(format!(
                    "{}:{} - Prohibited claim 'Art.-17-konform' found in doc/code: '{}'",
                    rel_path, line_num, trimmed
                ));
            } else if norm.contains("löschung nach art 17") {
                violations.push(format!(
                    "{}:{} - Prohibited claim 'Löschung nach Art. 17' found in doc/code: '{}'",
                    rel_path, line_num, trimmed
                ));
            } else if norm.contains("löschzertifikat") {
                violations.push(format!(
                    "{}:{} - Prohibited claim 'Löschzertifikat' found in doc/code: '{}'",
                    rel_path, line_num, trimmed
                ));
            } else if norm.contains("rechtskonform") || norm.contains("rechtssicher") {
                violations.push(format!(
                    "{}:{} - Prohibited legal claim ('rechtskonform' / 'rechtssicher') found in doc/code: '{}'",
                    rel_path, line_num, trimmed
                ));
            } else if norm.contains("100% gelöscht") {
                violations.push(format!(
                    "{}:{} - Prohibited claim '100% gelöscht' found in doc/code: '{}'",
                    rel_path, line_num, trimmed
                ));
            } else if norm.contains("kryptografisch gelöscht") {
                // Heuristic: "kryptografisch gelöscht" is only a violation if the same line does NOT mention any custody or assumption
                // Search terms: "Annahme", "A-FLASH", "Custody", "K1", "K2", "K3", "C1"
                let line_lower = line.to_lowercase();
                let has_custody_or_assumption = line_lower.contains("annahme")
                    || line_lower.contains("a-flash")
                    || line_lower.contains("custody")
                    || line_lower.contains("k1")
                    || line_lower.contains("k2")
                    || line_lower.contains("k3")
                    || line_lower.contains("c1");
                if !has_custody_or_assumption {
                    violations.push(format!(
                        "{}:{} - Prohibited claim 'kryptografisch gelöscht' without custody or assumption found in doc/code: '{}'",
                        rel_path, line_num, trimmed
                    ));
                }
            }
        }
    }

    // 3. Scan code for hardcoded zero counts, dummy proof verifications, and ungrounded tool proofs
    if crates_dir.exists() {
        for entry in WalkDir::new(&crates_dir).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }

            let rel_path = match path.strip_prefix(root) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => path.to_string_lossy().to_string(),
            };

            if rel_path.contains("/tests/")
                || rel_path.contains("/benches/")
                || rel_path.contains("/examples/")
            {
                continue;
            }

            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Check if tool/MCP file returns "proof" without calling erase_* or drop_collection
            if rel_path.contains("contextra-mcp/src/") {
                for (idx, line) in content.lines().enumerate() {
                    let line_num = idx + 1;
                    let trimmed = line.trim();

                    if (trimmed.contains("\"proof\":") || trimmed.contains("'proof':"))
                        && !trimmed.contains("proof_val")
                        && !trimmed.contains("null")
                        && !trimmed.contains("None")
                        && !content.contains("erase_")
                        && !content.contains("drop_collection")
                    {
                        violations.push(format!(
                            "{}:{} - Tool response includes 'proof' without calling engine deletion (erase_*/drop_collection): '{}'",
                            rel_path, line_num, trimmed
                        ));
                    }
                }
            }

            for (idx, line) in content.lines().enumerate() {
                let line_num = idx + 1;
                let trimmed = line.trim();

                if trimmed.contains("new_after_verified_empty(layer, 0)")
                    || trimmed.contains("new_after_verified_empty(layer, 0usize)")
                {
                    violations.push(format!(
                        "{}:{} - Hardcoded zero scanner count in proof creation: '{}'",
                        rel_path, line_num, trimmed
                    ));
                }

                if trimmed.contains("pub fn verify")
                    && (trimmed.contains("Ok(true)") || trimmed.contains("return Ok(true)"))
                {
                    violations.push(format!(
                        "{}:{} - Dummy verification returning Ok(true) found: '{}'",
                        rel_path, line_num, trimmed
                    ));
                }
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "deletion-honesty check failed with {} violation(s):\n{}",
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

// ── Gate 8: NoDummyProofGate ─────────────────────────────────────────────────

pub struct NoDummyProofGate;

impl Gate for NoDummyProofGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "no-dummy-proof".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH0".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["INV-NO-DUMMY-PROOF".to_string()],
            why: "Stellt sicher, dass keine Testschlüssel ('default_test_deletion_proof_key') oder deprecated Konstruktoren ('new_after_verified_empty', 'verify_and_create') im Produktionscode verwendet werden.".to_string(),
            owner: "MP-08".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_no_dummy_proof(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "no-dummy-proof" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }

        let temp = match tempfile::TempDir::new() {
            Ok(t) => t,
            Err(e) => {
                return Outcome::Fail {
                    reason: format!("Failed to create tempdir for selftest: {e}"),
                };
            }
        };
        let root = temp.path();

        // Positive control
        if let Err(e) = fs::create_dir_all(root.join("crates/contextra-engine/src")) {
            return Outcome::Fail {
                reason: format!("Failed to create engine dir in selftest: {e}"),
            };
        }
        if let Err(e) = fs::write(
            root.join("crates/contextra-engine/src/good.rs"),
            "let key = resolve_key();",
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write clean fixture in selftest: {e}"),
            };
        }
        if scan_no_dummy_proof(root) != Outcome::Pass {
            return Outcome::Fail {
                reason: "Selftest failed: scan_no_dummy_proof rejected clean code".to_string(),
            };
        }

        // Negative control: forbidden string in production src
        if let Err(e) = fs::write(
            root.join("crates/contextra-engine/src/bad.rs"),
            "let k = \"default_test_deletion_proof_key\";",
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write bad fixture in selftest: {e}"),
            };
        }

        match scan_no_dummy_proof(root) {
            Outcome::Fail { reason } if reason.contains("default_test_deletion_proof_key") => {
                Outcome::Pass
            }
            _ => Outcome::Fail {
                reason: "Selftest failed: scan_no_dummy_proof failed to detect default_test_deletion_proof_key in production src".to_string(),
            },
        }
    }
}

pub fn scan_no_dummy_proof(root: &Path) -> Outcome {
    let mut violations = Vec::new();
    let crates_dir = root.join("crates");

    if !crates_dir.exists() {
        return Outcome::Pass;
    }

    for entry in WalkDir::new(&crates_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        };

        // Only scan production code (crates/*/src/**)
        if !rel_path.contains("/src/")
            || rel_path.contains("/tests/")
            || rel_path.contains("/benches/")
            || rel_path.contains("/examples/")
        {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.contains("default_test_deletion_proof_key")
                || trimmed.contains("new_after_verified_empty")
                || trimmed.contains("verify_and_create")
            {
                violations.push(format!(
                    "{}:{} - Forbidden dummy proof construct/key found in production code: '{}'",
                    rel_path, line_num, trimmed
                ));
            }
        }
    }

    if violations.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "no-dummy-proof check failed with {} violation(s):\n{}",
                violations.len(),
                violations.join("\n")
            ),
        }
    }
}

// ── Gate 9: ScannerCoverageGate ──────────────────────────────────────────────

pub struct ScannerCoverageGate;

impl Gate for ScannerCoverageGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "scanner-coverage".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH1".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["REQ-SCANNER-COV".to_string()],
            why: "Stellt sicher, dass für jede in DeletionProof::issue als Pflichtschicht geltende WitnessLayer-Variante (LsmMemtable, SsTableAllLevels, WalAllSegments) mindestens eine LayerScanner-Implementierung im Workspace existiert.".to_string(),
            owner: "MP-08".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        scan_scanner_coverage(&ctx.root_dir)
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "scanner-coverage" {
            return Outcome::Fail {
                reason: "Spec name mismatch".to_string(),
            };
        }

        let temp = match tempfile::TempDir::new() {
            Ok(t) => t,
            Err(e) => {
                return Outcome::Fail {
                    reason: format!("Failed to create tempdir for selftest: {e}"),
                };
            }
        };
        let root = temp.path();

        // 1. Positive control
        if let Err(e) = fs::create_dir_all(root.join("crates/contextra-store/src/scanner")) {
            return Outcome::Fail {
                reason: format!("Failed to create scanner dir in selftest: {e}"),
            };
        }
        let code = r#"
            impl LayerScanner for MemtableScanner { fn witness_layer(&self) -> WitnessLayer { WitnessLayer::LsmMemtable } }
            impl LayerScanner for SstableScanner { fn witness_layer(&self) -> WitnessLayer { WitnessLayer::SsTableAllLevels } }
            impl LayerScanner for WalScanner { fn witness_layer(&self) -> WitnessLayer { WitnessLayer::WalAllSegments } }
        "#;
        if let Err(e) = fs::write(root.join("crates/contextra-store/src/scanner/mod.rs"), code) {
            return Outcome::Fail {
                reason: format!("Failed to write scanner fixture in selftest: {e}"),
            };
        }

        if scan_scanner_coverage(root) != Outcome::Pass {
            return Outcome::Fail {
                reason: "Selftest failed: scan_scanner_coverage rejected complete scanner coverage"
                    .to_string(),
            };
        }

        // 2. Negative control: missing WalAllSegments scanner
        let incomplete_code = r#"
            impl LayerScanner for MemtableScanner { fn witness_layer(&self) -> WitnessLayer { WitnessLayer::LsmMemtable } }
            impl LayerScanner for SstableScanner { fn witness_layer(&self) -> WitnessLayer { WitnessLayer::SsTableAllLevels } }
        "#;
        if let Err(e) = fs::write(
            root.join("crates/contextra-store/src/scanner/mod.rs"),
            incomplete_code,
        ) {
            return Outcome::Fail {
                reason: format!("Failed to write incomplete fixture in selftest: {e}"),
            };
        }

        match scan_scanner_coverage(root) {
            Outcome::Fail { reason } if reason.contains("WalAllSegments") => Outcome::Pass,
            _ => Outcome::Fail {
                reason: "Selftest failed: scan_scanner_coverage failed to detect missing WalAllSegments scanner".to_string(),
            },
        }
    }
}

pub fn scan_scanner_coverage(root: &Path) -> Outcome {
    let mandatory_layers = ["LsmMemtable", "SsTableAllLevels", "WalAllSegments"];

    let crates_dir = root.join("crates");
    if !crates_dir.exists() {
        return Outcome::Pass;
    }

    let mut found_layers = std::collections::HashSet::new();

    for entry in WalkDir::new(&crates_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }

        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if content.contains("LayerScanner") {
            for mandatory in &mandatory_layers {
                if content.contains(mandatory) {
                    found_layers.insert(*mandatory);
                }
            }
        }
    }

    let mut missing = Vec::new();
    for mandatory in &mandatory_layers {
        if !found_layers.contains(mandatory) {
            missing.push(*mandatory);
        }
    }

    if missing.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail {
            reason: format!(
                "scanner-coverage check failed: missing LayerScanner implementation for mandatory WitnessLayer(s): {}",
                missing.join(", ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_witness_mint_clean_and_dirty() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/contextra-ports/src/scanner")).unwrap();
        fs::write(
            root.join("crates/contextra-ports/src/scanner/mod.rs"),
            "let w = LayerWitness { remaining_count: 0 };",
        )
        .unwrap();

        assert_eq!(scan_witness_mint(root), Outcome::Pass);

        fs::create_dir_all(root.join("crates/contextra-engine/src")).unwrap();
        fs::write(
            root.join("crates/contextra-engine/src/bad.rs"),
            "let w = LayerWitness { remaining_count: 0 };",
        )
        .unwrap();

        match scan_witness_mint(root) {
            Outcome::Fail { reason } => assert!(reason.contains("LayerWitness")),
            _ => panic!("Expected Fail for illegal LayerWitness minting"),
        }
    }

    #[test]
    fn test_deletion_types_once_clean_and_dirty() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/contextra-types/src")).unwrap();
        fs::write(
            root.join("crates/contextra-types/src/lib.rs"),
            "pub enum DeletionLayer {}",
        )
        .unwrap();

        assert_eq!(scan_deletion_types_once(root), Outcome::Pass);

        fs::create_dir_all(root.join("crates/contextra-engine/src")).unwrap();
        fs::write(
            root.join("crates/contextra-engine/src/dup.rs"),
            "pub enum DeletionLayer {}",
        )
        .unwrap();

        match scan_deletion_types_once(root) {
            Outcome::Fail { reason } => assert!(reason.contains("Duplicate definition")),
            _ => panic!("Expected Fail for duplicate DeletionLayer definition"),
        }
    }

    #[test]
    fn test_deletion_honesty_clean_and_dirty() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::write(root.join("README.md"), "# Contextra").unwrap();
        assert_eq!(scan_deletion_honesty(root), Outcome::Pass);

        fs::write(root.join("README.md"), "Das System ist 100% gelöscht.").unwrap();

        match scan_deletion_honesty(root) {
            Outcome::Fail { reason } => assert!(reason.contains("100% gelöscht")),
            _ => panic!("Expected Fail for unmeasured claim in doc"),
        }
    }

    #[test]
    fn test_no_dummy_proof_clean_and_dirty() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/contextra-engine/src")).unwrap();
        fs::write(
            root.join("crates/contextra-engine/src/lib.rs"),
            "let x = 42;",
        )
        .unwrap();

        assert_eq!(scan_no_dummy_proof(root), Outcome::Pass);

        fs::write(
            root.join("crates/contextra-engine/src/lib.rs"),
            "let k = \"default_test_deletion_proof_key\";",
        )
        .unwrap();

        match scan_no_dummy_proof(root) {
            Outcome::Fail { reason } => assert!(reason.contains("default_test_deletion_proof_key")),
            _ => panic!("Expected Fail for dummy proof key in production code"),
        }
    }

    #[test]
    fn test_scanner_coverage_clean_and_dirty() {
        let temp = TempDir::new().unwrap();
        let root = temp.path();

        fs::create_dir_all(root.join("crates/contextra-store/src/scanner")).unwrap();
        let code = r#"
            impl LayerScanner for MemtableScanner { fn witness_layer(&self) -> WitnessLayer { WitnessLayer::LsmMemtable } }
            impl LayerScanner for SstableScanner { fn witness_layer(&self) -> WitnessLayer { WitnessLayer::SsTableAllLevels } }
            impl LayerScanner for WalScanner { fn witness_layer(&self) -> WitnessLayer { WitnessLayer::WalAllSegments } }
        "#;
        fs::write(root.join("crates/contextra-store/src/scanner/mod.rs"), code).unwrap();

        assert_eq!(scan_scanner_coverage(root), Outcome::Pass);

        let incomplete = r#"
            impl LayerScanner for MemtableScanner { fn witness_layer(&self) -> WitnessLayer { WitnessLayer::LsmMemtable } }
        "#;
        fs::write(
            root.join("crates/contextra-store/src/scanner/mod.rs"),
            incomplete,
        )
        .unwrap();

        match scan_scanner_coverage(root) {
            Outcome::Fail { reason } => assert!(reason.contains("SsTableAllLevels")),
            _ => panic!("Expected Fail for missing scanner implementations"),
        }
    }
}
