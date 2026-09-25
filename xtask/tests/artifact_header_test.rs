use xtask::artifact_header::ArtifactHeader;

#[test]
fn test_artifact_header_capture_commit_format() {
    let header = ArtifactHeader::capture("cargo xtask test")
        .expect("ArtifactHeader::capture failed");

    assert_eq!(
        header.commit.len(),
        40,
        "Commit SHA must be 40 characters long, got '{}'",
        header.commit
    );
    assert!(
        header.commit.chars().all(|c| c.is_ascii_hexdigit()),
        "Commit SHA must contain only valid hex characters, got '{}'",
        header.commit
    );
    assert_eq!(header.generated_by, "cargo xtask test");
    assert!(!header.generated_at.is_empty(), "generated_at must not be empty");
    assert!(
        header.toolchain.starts_with("rustc "),
        "toolchain must start with 'rustc ', got '{}'",
        header.toolchain
    );
}

#[test]
fn test_render_markdown_frontmatter() {
    let header = ArtifactHeader {
        commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
        generated_by: "cargo xtask generate-diagnostics".to_string(),
        generated_at: "2026-09-17T12:00:00Z".to_string(),
        toolchain: "rustc 1.80.0 (123456789 2024-07-25)".to_string(),
    };

    let frontmatter = header.render_markdown_frontmatter();

    assert!(frontmatter.starts_with("---\n"));
    assert!(frontmatter.ends_with("---\n"));
    assert!(frontmatter.contains("commit: 0123456789abcdef0123456789abcdef01234567"));
    assert!(frontmatter.contains("generated_by: cargo xtask generate-diagnostics"));
    assert!(frontmatter.contains("generated_at: 2026-09-17T12:00:00Z"));
    assert!(frontmatter.contains("toolchain: rustc 1.80.0 (123456789 2024-07-25)"));
}

#[test]
fn test_render_json_deserializable() {
    let header = ArtifactHeader {
        commit: "0123456789abcdef0123456789abcdef01234567".to_string(),
        generated_by: "cargo xtask test".to_string(),
        generated_at: "2026-09-17T12:00:00Z".to_string(),
        toolchain: "rustc 1.80.0 (123456789 2024-07-25)".to_string(),
    };

    let json_str = header
        .render_json()
        .expect("render_json failed");

    let deserialized: ArtifactHeader = serde_json::from_str(&json_str)
        .expect("Failed to deserialize JSON frontmatter");

    assert_eq!(header, deserialized);
    assert_eq!(deserialized.commit, "0123456789abcdef0123456789abcdef01234567");
    assert_eq!(deserialized.generated_by, "cargo xtask test");
    assert_eq!(deserialized.generated_at, "2026-09-17T12:00:00Z");
    assert_eq!(deserialized.toolchain, "rustc 1.80.0 (123456789 2024-07-25)");
}
