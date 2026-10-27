use std::fs;
use std::path::Path;

#[test]
fn test_no_test_hooks_in_release_api() {
    let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    fn check_dir(dir: &Path, violations: &mut Vec<String>) {
        for entry in fs::read_dir(dir).expect("read_dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                check_dir(&path, violations);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = fs::read_to_string(&path).expect("read_to_string");
                let lines: Vec<&str> = content.lines().collect();
                for (i, line) in lines.iter().enumerate() {
                    if (line.contains("pub fn ") || line.contains("pub(crate) fn "))
                        && (line.contains("_for_test") || line.contains("integrity_key_for_test"))
                    {
                        // Check preceding lines for cfg
                        let mut has_cfg = false;
                        for j in (0..i).rev() {
                            let prev = lines[j].trim();
                            if prev.starts_with("#[cfg(")
                                && (prev.contains("test") || prev.contains("test-hooks"))
                            {
                                has_cfg = true;
                                break;
                            }
                            if !prev.starts_with("#[") && !prev.is_empty() && !prev.starts_with("//") {
                                break;
                            }
                        }
                        if !has_cfg {
                            violations.push(format!(
                                "{}:{}: {}",
                                path.display(),
                                i + 1,
                                line.trim()
                            ));
                        }
                    }
                }
            }
        }
    }

    check_dir(&src_dir, &mut violations);

    assert!(
        violations.is_empty(),
        "Found test hooks exposed without cfg protection in production src:\n{}",
        violations.join("\n")
    );
}
