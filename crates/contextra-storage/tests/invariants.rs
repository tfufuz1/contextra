#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing, clippy::collapsible_if, clippy::unnecessary_map_or)]

//! # Invarianten-Vollständigkeitstest
//!
//! Liest docs/SPEC.md (normativ), extrahiert alle Invarianten-IDs (INV-*) aus Abschnitt 18
//! und prüft, dass jede Invariante in mindestens einer Quell- oder Testdatei zitiert wird.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// Invarianten, die der Bestand (Entwurf R2) noch nicht zitiert, weil der zugehörige Code
/// erst in späteren Gates entsteht (Spec §22, IMPLEMENTATION_SPEC WP-1.9). Die Liste darf nur
/// schrumpfen: Wird eine Invariante zitiert, muss sie hier entfernt werden (der Test erzwingt das).
/// Neue Invarianten in Spec §18 gehören **nicht** hierher, sondern müssen zitiert werden.
const NOCH_OFFEN: &[&str] = &[
    "INV-CLUSTER-1",
    "INV-CLUSTER-2",
    "INV-COMMIT-1",
    "INV-COMMIT-2",
    "INV-COMMIT-3",
    "INV-CONS-1",
    "INV-ERA-8",
    "INV-ERA-9",
    "INV-FMT-3",
    "INV-FMT-4",
    "INV-IDEM-1",
    "INV-KEY-3",
    "INV-KEY-4",
    "INV-KEYP-2",
    "INV-LIM-1",
    "INV-LOCK-2",
    "INV-LOG-9",
    "INV-PL-1",
    "INV-PL-2",
    "INV-PL-3",
    "INV-PL-4",
    "INV-RECOV-1",
    "INV-RECOV-2",
    "INV-STAMP-3",
    "INV-SYNC-1",
    "INV-VIEW-1",
];

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
    // Normativ ist docs/SPEC.md (CLAUDE.md, Spec §1.3); die Root-SPEC.md ist ein anderes Dokument.
    let spec_path = root.join("docs").join("SPEC.md");
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
        // Spec R8 §18: Tabellenzeilen der Form `| INV-XXX-n | Aussage | Prüfung |`.
        if in_table {
            if let Some(rest) = line.strip_prefix("| INV-") {
                if let Some(end) = rest.find(' ') {
                    expected_invs.insert(format!("INV-{}", &rest[..end]));
                }
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

    let mut cited_invs = HashSet::new();
    for path in rs_files {
        // Diese Datei nennt die Ausnahmeliste und zählt nicht als Zitat.
        if path.file_name().is_some_and(|n| n == "invariants.rs") {
            continue;
        }
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

    let offen: HashSet<String> = NOCH_OFFEN.iter().map(|s| s.to_string()).collect();

    // Die Ausnahmeliste muss zur Spec passen und darf zitierte Invarianten nicht enthalten.
    let mut unbekannt: Vec<_> = offen.difference(&expected_invs).cloned().collect();
    unbekannt.sort();
    assert!(unbekannt.is_empty(), "NOCH_OFFEN nennt Invarianten, die nicht in SPEC §18 stehen: {unbekannt:?}");
    let mut veraltet: Vec<_> = offen.intersection(&cited_invs).cloned().collect();
    veraltet.sort();
    assert!(veraltet.is_empty(), "Diese Invarianten sind jetzt zitiert und aus NOCH_OFFEN zu entfernen: {veraltet:?}");

    let mut uncited: Vec<_> = expected_invs
        .difference(&cited_invs)
        .filter(|id| !offen.contains(*id))
        .cloned()
        .collect();
    uncited.sort();
    assert!(
        uncited.is_empty(),
        "Nicht alle Invarianten aus SPEC §18 sind zitiert! Fehlende Invarianten ({}/{}): {:?}",
        uncited.len(),
        expected_invs.len(),
        uncited
    );
}
