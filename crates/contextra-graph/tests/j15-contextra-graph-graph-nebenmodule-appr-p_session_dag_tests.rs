//! Integration tests for `SessionBranchTree` DAG methods: `append_step`, `children_of`, and `set_active_head`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_graph::SessionBranchTree;

#[test]
fn test_session_branch_tree_dag_operations_integration() {
    let tree = SessionBranchTree::new("Root prompt".into(), "Root response".into());
    assert_eq!(tree.active_head(), 0);

    let step1 = tree
        .append_step("Prompt 1".into(), "Resp 1".into(), None, vec![], "main")
        .unwrap();
    assert_eq!(step1, 1);
    assert_eq!(tree.active_head(), 1);

    let step2 = tree
        .branch_from(step1, "Prompt 2 alt".into(), "Resp 2 alt".into(), None, vec![], "explore")
        .unwrap();
    assert_eq!(step2, 2);
    assert_eq!(tree.active_head(), 1);

    let children = tree.children_of(step1);
    assert_eq!(children, vec![step2]);

    tree.set_active_head(step2).unwrap();
    assert_eq!(tree.active_head(), 2);
}
