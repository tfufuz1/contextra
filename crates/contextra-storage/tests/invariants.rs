#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::collapsible_if, clippy::unnecessary_map_or)]

//! # Invarianten-Vollständigkeitstest
//!
//! Liest SPEC.md, extrahiert alle Invarianten-IDs (INV-*) aus Abschnitt 18
//! und prüft, dass jede Invariante in mindestens einer Quell- oder Testdatei zitiert wird.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_rs_files(&p, out);
            } else if p.extension().map_or(false, |e| e == "rs") {
                out.push(p);
            }
        }
    }
}

#[test]
fn test_all_invariants_cited() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let spec_path = root.join("SPEC.md");
    let spec_content = fs::read_to_string(&spec_path).expect("SPEC.md must exist");

    // Extrahiere alle INV-*-Kennungen aus der Invariantentabelle (§18)
    let mut expected_invs = HashSet::new();
    let mut in_table = false;

    for line in spec_content.lines() {
        if line.contains("## 18. Invariantenkatalog") {
            in_table = true;
            continue;
        }
        if in_table && line.starts_with("## ") {
            break;
        }
        if in_table && line.starts_with("|`INV-") {
            if let Some(end) = line[2..].find('`') {
                let id = &line[2..2 + end];
                expected_invs.insert(id.to_string());
            }
        }
    }

    assert!(
        expected_invs.len() >= 35,
        "Erwartet mindestens 35 Invarianten in SPEC.md §18, gefunden: {}",
        expected_invs.len()
    );

    // Rekursiv alle Quell- und Testdateien durchsuchen
    let mut rs_files = Vec::new();
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    collect_rs_files(&manifest_dir.join("src"), &mut rs_files);
    collect_rs_files(&manifest_dir.join("tests"), &mut rs_files);
    collect_rs_files(&manifest_dir.join("../contextra-storage-ports/src"), &mut rs_files);
    collect_rs_files(&manifest_dir.join("../contextra-crypto-ref/src"), &mut rs_files);
    collect_rs_files(&manifest_dir.join("../contextra-crypto-ref/tests"), &mut rs_files);

    let mut cited_invs = HashSet::new();
    for path in rs_files {
        if let Ok(code) = fs::read_to_string(&path) {
            for id in &expected_invs {
                if code.contains(id) {
                    cited_invs.insert(id.clone());
                }
            }
        }
    }

    println!(
        "Invarianten in SPEC.md: {}, zitiert im neuen Code: {}",
        expected_invs.len(),
        cited_invs.len()
    );

    let mut uncited: Vec<_> = expected_invs.difference(&cited_invs).cloned().collect();
    uncited.sort();
    assert!(
        uncited.is_empty(),
        "Nicht alle Invarianten aus SPEC §18 sind zitiert! Fehlende Invarianten ({}/{}): {:?}",
        uncited.len(),
        expected_invs.len(),
        uncited
    );
}
