use std::path::Path;
use std::process::Command;

fn main() {
    let schema_path = "../../schemas/contextra.fbs";
    let out_dir = "src";

    if Path::new(schema_path).exists() {
        println!("cargo:rerun-if-changed={schema_path}");

        let output_file = Path::new(out_dir).join("contextra_generated.rs");

        let flatc_exists = Command::new("flatc")
            .arg("--version")
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if flatc_exists {
            if let Ok(status) = Command::new("flatc")
                .args(["--rust", "-o", out_dir, schema_path])
                .status()
            {
                assert!(status.success(), "flatc failed to generate code");
                post_process_generated_code(&output_file);
            }
        } else if !output_file.exists() {
            eprintln!("flatc not found and generated code does not exist. Please install flatbuffers compiler.");
            std::process::exit(1);
        } else {
            println!("cargo:warning=flatc not found, using existing generated code.");
        }
    }
}

fn wrap_tab_get_calls_in_unsafe(line: &str) -> String {
    let pattern = "self._tab.get";
    if !line.contains(pattern) {
        return line.to_string();
    }

    let mut result = String::with_capacity(line.len() + 32);
    let mut search_from = 0;

    while let Some(rel_idx) = line[search_from..].find(pattern) {
        let idx = search_from + rel_idx;

        // Check if this occurrence is already preceded by `unsafe` on the same line
        let prefix = &line[..idx];
        if prefix.trim_end().ends_with("unsafe {") || prefix.contains("unsafe") {
            // Already wrapped
            result.push_str(&line[search_from..idx + pattern.len()]);
            search_from = idx + pattern.len();
            continue;
        }

        // Find the opening parenthesis of `get::<...>(...`
        if let Some(rel_paren_open) = line[idx + pattern.len()..].find('(') {
            let paren_open = idx + pattern.len() + rel_paren_open;

            // Find matching closing parenthesis
            let mut depth = 0;
            let mut close_paren = None;
            for (i, ch) in line[paren_open..].char_indices() {
                if ch == '(' {
                    depth += 1;
                } else if ch == ')' {
                    depth -= 1;
                    if depth == 0 {
                        close_paren = Some(paren_open + i);
                        break;
                    }
                }
            }

            if let Some(end_idx) = close_paren {
                result.push_str(&line[search_from..idx]);
                result.push_str("unsafe { ");
                result.push_str(&line[idx..=end_idx]);
                result.push_str(" }");
                search_from = end_idx + 1;
                continue;
            }
        }

        result.push_str(&line[search_from..idx + pattern.len()]);
        search_from = idx + pattern.len();
    }

    result.push_str(&line[search_from..]);
    result
}

fn post_process_generated_code(output_file: &Path) {
    let content = match std::fs::read_to_string(output_file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut modified = false;
    let new_lines: Vec<String> = content
        .lines()
        .map(|line| {
            let processed = wrap_tab_get_calls_in_unsafe(line);
            if processed != line {
                modified = true;
            }
            processed
        })
        .collect();

    if modified {
        let mut new_content = new_lines.join("\n");
        if content.ends_with('\n') {
            new_content.push('\n');
        }
        let _ = std::fs::write(output_file, new_content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_tab_get_calls() {
        let input1 = "    self._tab.get::<u32>(RoleId::VT_ID, Some(0)).unwrap()";
        let expected1 = "    unsafe { self._tab.get::<u32>(RoleId::VT_ID, Some(0)) }.unwrap()";
        assert_eq!(wrap_tab_get_calls_in_unsafe(input1), expected1);

        let input2 = "    self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(ScoredDocument::VT_ID, None)";
        let expected2 = "    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(ScoredDocument::VT_ID, None) }";
        assert_eq!(wrap_tab_get_calls_in_unsafe(input2), expected2);

        let input3 = "    unsafe { self._tab.get::<i8>(Embedding::VT_METRIC, Some(0)).unwrap()}";
        assert_eq!(wrap_tab_get_calls_in_unsafe(input3), input3);
    }
}
