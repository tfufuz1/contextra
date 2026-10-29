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

        // Positive control
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

        // Negative control: fake proof in tool response
        if let Err(e) = fs::create_dir_all(root.join("crates/contextra-mcp/src")) {
            return Outcome::Fail {
                reason: format!("Failed to create dummy mcp dir in selftest: {e}"),
            };
        }
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

pub fn scan_deletion_honesty(root: &Path) -> Outcome {
    let mut violations = Vec::new();

    // 1. Scan external docs (README, AGENTS.md, etc.) for unmeasured absolute claims
    let docs = ["README.md", "AGENTS.md", "docs/SPEC.md"];
    for doc in docs {
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

    // 2. Scan code for hardcoded zero counts, dummy proof verifications, and ungrounded tool proofs
    let crates_dir = root.join("crates");
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
