//! Golden file and determinism test suite.

use contextra_audit_export::{
    render_register_json, render_register_markdown, testkit::InMemoryProcessingRegisterSource,
    ProcessingRegisterSource,
};
use contextra_types::TenantId;

#[test]
fn test_golden_file_json_export() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(42)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let entries = source.collect_entries(tenant_id)?;

    let json_output = render_register_json(&entries)?;
    let golden_json = include_str!("fixtures/golden_register.json");

    assert_eq!(
        json_output.trim(),
        golden_json.trim(),
        "JSON output must be byte-identical to golden fixture"
    );

    Ok(())
}

#[test]
fn test_golden_file_markdown_export() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(42)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let entries = source.collect_entries(tenant_id)?;

    let md_output = render_register_markdown(&entries, "2026-10-28")?;
    let golden_md = include_str!("fixtures/golden_register.md");

    assert_eq!(
        md_output.trim(),
        golden_md.trim(),
        "Markdown output must be byte-identical to golden fixture"
    );

    Ok(())
}

#[test]
fn test_byte_identical_markdown_output() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(100)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let entries = source.collect_entries(tenant_id)?;

    let md1 = render_register_markdown(&entries, "2026-10-28")?;
    let md2 = render_register_markdown(&entries, "2026-10-28")?;

    assert_eq!(
        md1, md2,
        "Markdown rendering must produce byte-identical output across runs"
    );
    Ok(())
}

#[test]
fn test_byte_identical_json_output() -> Result<(), Box<dyn std::error::Error>> {
    let tenant_id = TenantId::try_new(100)?;
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let entries = source.collect_entries(tenant_id)?;

    let json1 = render_register_json(&entries)?;
    let json2 = render_register_json(&entries)?;

    assert_eq!(
        json1, json2,
        "JSON rendering must produce byte-identical output across runs"
    );
    Ok(())
}

#[test]
fn test_deterministic_sorting_by_tenant_and_purpose() -> Result<(), Box<dyn std::error::Error>> {
    let tenant1 = TenantId::try_new(10)?;
    let tenant2 = TenantId::try_new(20)?;

    let source1 = InMemoryProcessingRegisterSource::with_sample_data_for(tenant1);
    let entries2 =
        InMemoryProcessingRegisterSource::with_sample_data_for(tenant2).collect_entries(tenant2)?;

    let mut all_entries = source1.collect_entries(tenant1)?;
    all_entries.extend(entries2);

    // Reverse order before rendering to test sorting
    all_entries.reverse();

    let json_output = render_register_json(&all_entries)?;

    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_output)?;
    let t_id_0 = parsed
        .first()
        .and_then(|v| v.get("tenant_id"))
        .and_then(|v| v.as_u64())
        .ok_or("Missing element")?;
    let t_id_1 = parsed
        .get(1)
        .and_then(|v| v.get("tenant_id"))
        .and_then(|v| v.as_u64())
        .ok_or("Missing element")?;
    let t_id_2 = parsed
        .get(2)
        .and_then(|v| v.get("tenant_id"))
        .and_then(|v| v.as_u64())
        .ok_or("Missing element")?;
    let t_id_3 = parsed
        .get(3)
        .and_then(|v| v.get("tenant_id"))
        .and_then(|v| v.as_u64())
        .ok_or("Missing element")?;

    assert_eq!(t_id_0, 10);
    assert_eq!(t_id_1, 10);
    assert_eq!(t_id_2, 20);
    assert_eq!(t_id_3, 20);

    Ok(())
}
