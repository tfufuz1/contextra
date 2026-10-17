use std::fs;
use std::path::Path;

fn scan_file(path: &Path, unwrapped: &mut Vec<String>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();

        // Check if line contains `pub async fn`
        if line.contains("pub async fn ") || line.starts_with("pub async fn") {
            // Check preceding lines for #[doc(hidden)] or #[allow(panic_guard_check)]
            let mut is_hidden = false;
            let mut j = i.saturating_sub(1);
            while j < i {
                let prev = lines[j].trim();
                if prev.contains("#[doc(hidden)]") || prev.contains("panic_guard_check") {
                    is_hidden = true;
                    break;
                }
                if !prev.starts_with('#') && !prev.starts_with("//") && !prev.is_empty() {
                    break;
                }
                if j == 0 {
                    break;
                }
                j -= 1;
            }

            if !is_hidden {
                // Extract fn name
                let fn_part = if let Some(idx) = line.find("pub async fn ") {
                    &line[idx + "pub async fn ".len()..]
                } else {
                    line
                };
                let fn_name = fn_part
                    .split(&['(', '<', ' '][..])
                    .next()
                    .unwrap_or("unknown")
                    .trim();

                // Find fn body (search next ~40 lines for `guard_panic` or `public_api!`)
                let mut has_guard = false;
                let mut search_idx = i;
                let end_search = (i + 40).min(lines.len());
                while search_idx < end_search {
                    if lines[search_idx].contains("guard_panic") || lines[search_idx].contains("public_api!") {
                        has_guard = true;
                        break;
                    }
                    search_idx += 1;
                }

                if !has_guard {
                    unwrapped.push(format!(
                        "{}:{}: pub async fn {}() missing guard_panic",
                        path.display(),
                        i + 1,
                        fn_name
                    ));
                }
            }
        }
        i += 1;
    }
}

fn scan_dir(dir: &Path, unwrapped: &mut Vec<String>) {
    if !dir.exists() {
        return;
    }
    for entry in fs::read_dir(dir).expect("read dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.is_dir() {
            scan_dir(&path, unwrapped);
        } else if path.extension().map_or(false, |ext| ext == "rs") {
            scan_file(&path, unwrapped);
        }
    }
}

#[test]
fn test_all_public_async_fns_are_panic_guarded() {
    let mut unwrapped = Vec::new();
    let facade_src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    scan_dir(&facade_src, &mut unwrapped);

    let db_src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("contextra-db")
        .join("src");
    scan_dir(&db_src, &mut unwrapped);

    assert!(
        unwrapped.is_empty(),
        "Found public async fn in facade/db without guard_panic:\n{:#?}",
        unwrapped
    );
}
