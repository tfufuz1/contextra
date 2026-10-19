//! `avv_generator` — Auftragsverarbeitungsvertrag (AVV) Generierung gemäß Art. 28 DSGVO.
//!
//! Re-exports `AvvContext` and AVV document generation from `contextra-avv-generator`.

pub use contextra_avv_generator::{
    default_technical_measures, render_avv_markdown, AvvContext, AvvContextBuilder,
    AvvGeneratorError, TechnicalMeasure,
};

/// Generiert einen vollständigen AVV-Entwurf im Markdown-Format unter Verwendung von [`render_avv_markdown`].
#[deprecated(
    note = "Nutze contextra-avv-generator anstelle von contextra-privacy::avv_generator"
)]
pub fn generate_avv_draft(ctx: &AvvContext) -> String {
    render_avv_markdown(ctx).unwrap_or_else(|err| format!("Error rendering AVV draft: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::TenantId;

    #[test]
    fn test_generate_avv_draft_delegates_to_avv_generator() {
        let tenant_id = TenantId::try_new(100).expect("valid tenant_id");
        let ctx = AvvContext::new("Acme Corp GmbH", "Contextra Cloud Systems AG", tenant_id);
        let draft = generate_avv_draft(&ctx);

        let required_disclaimer =
            "Entwurf – juristische Prüfung erforderlich";
        assert!(
            draft.contains(required_disclaimer),
            "AVV draft must contain legal disclaimer"
        );
        assert!(draft.contains("Acme Corp GmbH"));
        assert!(draft.contains("Contextra Cloud Systems AG"));
    }
}
