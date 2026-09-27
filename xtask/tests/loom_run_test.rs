#[path = "../src/loom_run.rs"]
mod loom_run;

use std::fs;
use tempfile::TempDir;

#[test]
fn test_discover_loom_tests_synthetic() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    let crate_a_tests = root.join("crates").join("crate-a").join("tests");
    let crate_b_tests = root.join("crates").join("crate-b").join("tests");
    fs::create_dir_all(&crate_a_tests).unwrap();
    fs::create_dir_all(&crate_b_tests).unwrap();

    fs::write(crate_a_tests.join("loom_first.rs"), "// test").unwrap();
    fs::write(crate_a_tests.join("normal_test.rs"), "// test").unwrap();
    fs::write(crate_b_tests.join("loom_second.rs"), "// test").unwrap();

    let discovered = loom_run::discover_loom_tests(root);

    assert_eq!(discovered.len(), 2);
    assert_eq!(discovered[0].crate_name, "crate-a");
    assert_eq!(discovered[0].path, "crates/crate-a/tests/loom_first.rs");
    assert_eq!(discovered[1].crate_name, "crate-b");
    assert_eq!(discovered[1].path, "crates/crate-b/tests/loom_second.rs");
}

#[test]
fn test_discover_loom_tests_real_workspace() {
    let root = xtask::find_root_dir();
    let discovered = loom_run::discover_loom_tests(&root);

    assert_eq!(
        discovered.len(),
        7,
        "Expected exactly 7 loom test files in workspace, found {}",
        discovered.len()
    );

    let paths: Vec<&str> = discovered.iter().map(|f| f.path.as_str()).collect();
    assert!(paths.contains(&"crates/contextra-db/tests/loom_relate_n_ary.rs"));
    assert!(paths.contains(&"crates/contextra-crypto/tests/loom_kv_deferred_zeroize.rs"));
    assert!(paths.contains(&"crates/contextra-crypto/tests/loom_eviction_worker_shutdown_race.rs"));
    assert!(paths.contains(&"crates/contextra-vector/tests/loom_quantizer_race_test.rs"));
    assert!(paths.contains(&"crates/contextra-store/tests/loom_multi_key_lock.rs"));
    assert!(paths.contains(&"crates/contextra-store/tests/loom_group_commit.rs"));
    assert!(paths.contains(&"crates/contextra-store/tests/loom_group_commit_handoff.rs"));
}
