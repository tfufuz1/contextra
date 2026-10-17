//! Example demonstrating GDPR AVV draft generation using Contextra.

use contextra::compliance::AvvParams;
use contextra::ContextraBuilder;
use contextra_types::TenantId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = std::env::temp_dir().join("contextra_example_avv");
    let _ = std::fs::remove_dir_all(&tmp_path);

    let db = ContextraBuilder::new(128)
        .with_storage_path(&tmp_path)
        .build()
        .await?;

    let params = AvvParams {
        controller_name: "Kanzlei Dr. Mustermann".to_string(),
        processor_name: "Contextra Cloud Systems".to_string(),
        tenant_id: TenantId::try_new(1001)?,
        technical_measures: None,
        subprocessors: vec!["Hosting Provider EU".to_string()],
        deletion_sla_days: 14,
    };

    let draft = db.generate_avv_draft(&params)?;

    println!("Review Header:\n{}", draft.review_header);
    println!("\nGenerated Markdown Preview:\n{}", &draft.markdown[..300]);

    let _ = std::fs::remove_dir_all(&tmp_path);
    Ok(())
}
