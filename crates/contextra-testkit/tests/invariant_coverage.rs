//! Zuordnung Test ↔ Invariante (docs/erasure/13 §11) als Test.
//!
//! Jede in der Tabelle genannte Testkennung muss als `fn` im Workspace existieren.
//! Noch nicht implementierte Pakete stehen in [`PENDING`]. Die Liste ist eine Ratsche:
//! fehlt eine Kennung außerhalb der Liste **oder** existiert eine gelistete inzwischen,
//! schlägt der Test fehl.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const SPEC: &str = "docs/erasure/13-contextra-testkit.md";

/// Kennungen, deren Tests aus noch offenen Arbeitspaketen stammen.
const PENDING: &[&str] = &[
    // Store-Crash-Matrix (docs/erasure/03 §9)
    "CP-5",
    "CP-6",
    "CP-7",
    "CP-11",
    "CP-14",
    // Checkpoint (09), Engine (10), MCP (12)
    "TST-CKPT-ERA-6",
    "TST-ENG-ERA-2",
    "TST-ENG-ERA-3",
    "TST-ENG-ERA-5",
    "TST-ENG-ERA-7",
    "TST-MCP-ERA-5",
];

/// `TST-ENG-ERA-3` → `tst_eng_era_3`, führende Nullen entfernt.
fn normalize(id: &str) -> String {
    id.to_ascii_lowercase()
        .split(['-', '_'])
        .map(|part| match part.parse::<u32>() {
            Ok(n) => n.to_string(),
            Err(_) => part.to_string(),
        })
        .collect::<Vec<_>>()
        .join("_")
}

/// Kennungen einer Tabellenzelle: `TST-…-N`, `LOOM-Rn[/Rm]`, `CP-n[…CP-m]`, `Cn`.
fn ids_in(cell: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let tokens: Vec<&str> = cell
        .split(|c: char| !(c.is_ascii_alphanumeric() || "-/…".contains(c)))
        .filter(|t| !t.is_empty())
        .collect();
    for token in tokens {
        if let Some((a, b)) = token.split_once('…') {
            if let (Some(a), Some(b)) = (a.strip_prefix("CP-"), b.strip_prefix("CP-")) {
                let range = a.parse::<u32>().unwrap()..=b.parse::<u32>().unwrap();
                out.extend(range.map(|n| format!("CP-{n}")));
            }
        } else if let Some(rest) = token.strip_prefix("LOOM-") {
            out.extend(rest.split('/').map(|r| format!("LOOM-{r}")));
        } else {
            let numbered = (token.starts_with("TST-") || token.starts_with("CP-"))
                && token.ends_with(|c: char| c.is_ascii_digit());
            let case = token
                .strip_prefix('C')
                .is_some_and(|n| n.len() == 1 && n.chars().all(|c| c.is_ascii_digit()));
            if numbered || case {
                out.insert(token.to_string());
            }
        }
    }
    out
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "target") {
                rust_files(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Namen aller Funktionen, die direkt unter einem Testattribut stehen.
fn test_fns(source: &str) -> Vec<String> {
    let mut armed = false;
    let mut out = Vec::new();
    for line in source.lines().map(str::trim) {
        if line.starts_with("#[test]") || line.starts_with("#[tokio::test") {
            armed = true;
        } else if armed && !line.starts_with("#[") && !line.starts_with("//") {
            if let Some(rest) = line.split("fn ").nth(1) {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                out.push(normalize(&name));
            }
            armed = false;
        }
    }
    out
}

fn covered(id: &str, fns: &BTreeSet<String>) -> bool {
    let want = normalize(id);
    fns.iter()
        .any(|f| *f == want || f.starts_with(&format!("{want}_")))
}

#[test]
fn every_invariant_test_id_exists_or_is_pending() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let spec = fs::read_to_string(root.join(SPEC)).unwrap();
    let section = spec
        .split("\n## ")
        .find(|s| s.starts_with("11."))
        .expect("section 11");

    let mut invariants = BTreeSet::new();
    let mut required = BTreeSet::new();
    for row in section.lines().filter(|l| l.starts_with("| I-")) {
        let mut cells = row.split('|').map(str::trim).skip(1);
        let name = cells.next().unwrap();
        invariants.insert(name.split(' ').next().unwrap().to_string());
        required.extend(ids_in(cells.next().unwrap()));
    }
    let all: BTreeSet<String> = (1..=12).map(|n| format!("I-{n}")).collect();
    assert_eq!(invariants, all, "every invariant needs a table row");

    let mut files = Vec::new();
    rust_files(&root.join("crates"), &mut files);
    let me = Path::new(file!()).file_name().unwrap();
    let mut fns = BTreeSet::new();
    for file in files.iter().filter(|f| f.file_name() != Some(me)) {
        fns.extend(test_fns(&fs::read_to_string(file).unwrap()));
    }
    // C1–C7 entstehen durch `assert_layer_conformance!`: die Fälle sind `async fn c<n>_…`
    // in der Suite und werden vom Selbsttest als Tests instanziiert.
    let suite =
        fs::read_to_string(root.join("crates/contextra-testkit/src/conformance/erasure_layer.rs"))
            .unwrap();
    for n in 1..=7 {
        if suite.contains(&format!("pub async fn c{n}_")) {
            fns.insert(format!("c{n}"));
        }
    }

    let missing: BTreeSet<&str> = required
        .iter()
        .filter(|id| !covered(id, &fns))
        .map(String::as_str)
        .collect();
    let pending: BTreeSet<&str> = PENDING.iter().copied().collect();
    let unexpected: Vec<_> = missing.difference(&pending).collect();
    let stale: Vec<_> = pending.difference(&missing).collect();
    assert!(
        unexpected.is_empty() && stale.is_empty(),
        "missing without PENDING entry: {unexpected:?}\nPENDING entries to remove: {stale:?}"
    );
}

#[test]
fn id_parsing_and_matching() {
    let ids = ids_in("C4, TST-ENG-ERA-5; LOOM-R1/R4, CP-5…CP-7, `fail_sync` G-3 (CP-14)");
    let want = [
        "C4",
        "CP-14",
        "CP-5",
        "CP-6",
        "CP-7",
        "LOOM-R1",
        "LOOM-R4",
        "TST-ENG-ERA-5",
    ];
    assert_eq!(ids, want.iter().map(|s| (*s).to_string()).collect());
    let fns: BTreeSet<String> = test_fns("#[test]\nfn tst_ports_09_x() {}\nfn cp_1_helper() {}")
        .into_iter()
        .collect();
    assert!(covered("TST-PORTS-09", &fns));
    assert!(!covered("TST-PORTS-9X", &fns) && !covered("CP-1", &fns));
}
