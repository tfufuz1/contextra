use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub struct Violation {
    pub file_path: String,
    pub start_line: usize,
    pub trait_name: String,
}

pub fn run_check_toctou_trait_defaults(root: &Path) -> Result<Vec<Violation>, String> {
    let mut violations = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != "target"
                && name != "tests"
                && name != ".git"
                && name != ".cargo"
                && name != "node_modules"
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file()
            && path.extension().and_then(|s| s.to_str()) == Some("rs")
            && !path.to_string_lossy().contains("/tests/")
            && !path.to_string_lossy().ends_with("_test.rs")
        {
            let rel_path = path
                .strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");

            if rel_path.starts_with("xtask/") {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                let lines: Vec<&str> = content.lines().collect();
                let mut i = 0;

                while i < lines.len() {
                    let line = lines[i];
                    let trimmed = line.trim();

                    if trimmed.starts_with("//") {
                        i += 1;
                        continue;
                    }

                    if let Some(trait_pos) = trimmed.find("trait ") {
                        let after_trait = &trimmed[trait_pos + 6..];
                        let trait_name = after_trait
                            .split(|c: char| c.is_whitespace() || c == '{' || c == '<' || c == ':')
                            .next()
                            .unwrap_or("Unknown")
                            .to_string();

                        let start_line = i + 1;
                        let mut block_lines = Vec::new();
                        let mut brace_depth = 0;
                        let mut found_open = false;

                        for idx in i..lines.len() {
                            let curr_line = lines[idx];
                            block_lines.push(curr_line);

                            for ch in curr_line.chars() {
                                if ch == '{' {
                                    brace_depth += 1;
                                    found_open = true;
                                } else if ch == '}' {
                                    brace_depth -= 1;
                                }
                            }

                            if found_open && brace_depth == 0 {
                                i = idx;
                                break;
                            }
                        }

                        let block_str = block_lines.join("\n");

                        if block_str.contains("// TOCTOU-ACCEPTED") {
                            i += 1;
                            continue;
                        }

                        let has_get = block_str.contains("self.get(");
                        let has_put_like = block_str.contains("self.put(")
                            || block_str.contains("self.insert(")
                            || block_str.contains("self.write(");

                        if has_get && has_put_like {
                            violations.push(Violation {
                                file_path: rel_path.clone(),
                                start_line,
                                trait_name,
                            });
                        }
                    }

                    i += 1;
                }
            }
        }
    }

    Ok(violations)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_detects_toctou_trait_default() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("storage.rs");
        fs::write(
            &file,
            r#"
pub trait KvStorage {
    fn get(&self, key: &str) -> Option<Vec<u8>>;
    fn put(&self, key: &str, val: Vec<u8>);

    fn update(&self, key: &str) {
        if let Some(v) = self.get(key) {
            self.put(key, v);
        }
    }
}
"#,
        )
        .unwrap();

        let violations = run_check_toctou_trait_defaults(dir.path()).unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].trait_name, "KvStorage");
    }

    #[test]
    fn test_ignores_accepted_toctou() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("storage.rs");
        fs::write(
            &file,
            r#"
pub trait KvStorage {
    // TOCTOU-ACCEPTED
    fn update(&self, key: &str) {
        if let Some(v) = self.get(key) {
            self.put(key, v);
        }
    }
}
"#,
        )
        .unwrap();

        let violations = run_check_toctou_trait_defaults(dir.path()).unwrap();
        assert_eq!(violations.len(), 0);
    }
}
