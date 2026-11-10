#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

//! Generator for golden files and BSI mapping in `contextra-audit-export`.

use std::fs;
use std::path::Path;

use contextra_audit_export::bsi_mapping::{bsi_mapping_table, render_bsi_mapping_markdown};
use contextra_audit_export::testkit::InMemoryProcessingRegisterSource;
use contextra_audit_export::{
    render_register_json, render_register_markdown, ProcessingRegisterSource, TenantId,
};

#[test]
#[ignore]
fn generate_audit_export_golden_corpus() {
    let out_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden");
    fs::create_dir_all(&out_dir).expect("Failed to create golden dir");

    let tenant_id = TenantId::try_new(42).expect("tenant id");
    let source = InMemoryProcessingRegisterSource::with_sample_data_for(tenant_id);
    let entries = source.collect_entries(tenant_id).expect("collect entries");

    let json_output = render_register_json(&entries).expect("render json");
    let md_output = render_register_markdown(&entries, "2026-10-28").expect("render markdown");

    let bsi_table = bsi_mapping_table();
    let bsi_md = render_bsi_mapping_markdown(&bsi_table);
    let bsi_json = serde_json::to_string_pretty(&bsi_table).expect("bsi json");

    fs::write(out_dir.join("golden_register.json"), json_output.as_bytes()).expect("write json");
    fs::write(out_dir.join("golden_register.md"), md_output.as_bytes()).expect("write md");
    fs::write(out_dir.join("bsi_mapping.md"), bsi_md.as_bytes()).expect("write bsi md");
    fs::write(out_dir.join("bsi_mapping.json"), bsi_json.as_bytes()).expect("write bsi json");
}
