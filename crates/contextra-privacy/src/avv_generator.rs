//! `avv_generator` — Re-Export-Schicht auf `contextra-avv-generator`.
//! # Abkündigung & Konsolidierung (AV-1)

#![deprecated(note = "use contextra-avv-generator crate directly")]

#[deprecated(note = "use contextra_avv_generator::AvvContext")]
pub use contextra_avv_generator::AvvContext;

#[deprecated(note = "use contextra_avv_generator::render_avv_markdown")]
pub fn generate_avv_draft(ctx: &contextra_avv_generator::AvvContext) -> String {
    contextra_avv_generator::render_avv_markdown(ctx).unwrap_or_else(|e| format!("# Fehler: {e}"))
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_avv_draft_equivalence() {
        let mut ctx = AvvContext::default();
        ctx.controller_name = "Acme Corp".to_string();
        ctx.processor_name = "Contextra AG".to_string();
        let md = generate_avv_draft(&ctx);
        assert!(md.contains("Acme Corp") && md.contains("Art. 28 DSGVO"));
    }
}
