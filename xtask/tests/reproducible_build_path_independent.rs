use std::fs;
use std::path::PathBuf;
use xtask::reproducible_build::{collect_executable_hashes, execute_build};

#[test]
fn test_reproducible_build_path_independent() {
    let root_dir = xtask::find_root_dir();
    let dir_a = PathBuf::from("/tmp/repro_a");
    let dir_b = PathBuf::from("/tmp/repro_b_longer_name");

    let _ = fs::remove_dir_all(&dir_a);
    let _ = fs::remove_dir_all(&dir_b);

    println!("Building Run A in {}", dir_a.display());
    let success_a = execute_build(&root_dir, &dir_a, Some("contextra-mcp"));
    assert!(success_a, "Build Run A failed");

    println!("Building Run B in {}", dir_b.display());
    let success_b = execute_build(&root_dir, &dir_b, Some("contextra-mcp"));
    assert!(success_b, "Build Run B failed");

    let hashes_a = collect_executable_hashes(&dir_a.join("release"));
    let hashes_b = collect_executable_hashes(&dir_b.join("release"));

    assert!(
        !hashes_a.is_empty(),
        "No executables collected in Run A ({})",
        dir_a.join("release").display()
    );
    assert!(
        !hashes_b.is_empty(),
        "No executables collected in Run B ({})",
        dir_b.join("release").display()
    );

    assert_eq!(
        hashes_a.len(),
        hashes_b.len(),
        "Executable counts differ between Run A and Run B"
    );

    for (name, hash_a) in &hashes_a {
        let hash_b = hashes_b
            .get(name)
            .unwrap_or_else(|| panic!("Executable {} missing in Run B", name));
        assert_eq!(
            hash_a, hash_b,
            "Binary SHA-256 hash mismatch for '{}' between Run A ({}) and Run B ({})",
            name, hash_a, hash_b
        );
    }
}
