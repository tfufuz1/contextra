// FILE-CONTEXT
// ZWECK: Extractor-Implementierung für TOML Format (WP M4).
// INVARIANTEN: FILE-CONTEXT Kopf, Zero-Panic, ISO-8601 Formatierung von Datumswerten.
// STAND: TS:2026-10-31T00:00:00Z

use super::path::{append_index, append_key};
use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, Locator};
use toml::Value;

/// Extrahiert ein TOML-Dokument in die `DocumentIr`.
pub fn extract_toml(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError> {
    let text = std::str::from_utf8(input).map_err(|e| IngestError::Corrupt {
        format: Some(FormatId::Toml),
        detail: format!("UTF-8 Dekodierfehler: {}", e),
    })?;

    let root: Value = toml::from_str(text).map_err(|e| IngestError::Corrupt {
        format: Some(FormatId::Toml),
        detail: format!("TOML-Parserfehler: {}", e),
    })?;

    let mut ir = DocumentIr::new(FormatId::Toml);
    let max_depth = usize::try_from(ctx.limits.max_xml_depth).unwrap_or(256);

    // meta.title aus oberster Tabelle extrahieren
    if let Value::Table(ref table) = root {
        if let Some(t) = table.get("title").or_else(|| table.get("name")) {
            match t {
                Value::String(s) => ir.meta.title = Some(s.clone()),
                Value::Integer(i) => ir.meta.title = Some(i.to_string()),
                _ => {}
            }
        }
    }

    emit_toml_value(&root, "", 0, max_depth, &mut ir, ctx)?;
    Ok(ir)
}

fn emit_toml_value(
    node: &Value,
    current_path: &str,
    depth: usize,
    max_depth: usize,
    ir: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    if depth > max_depth {
        return Err(IngestError::LimitExceeded {
            kind: LimitKind::NestingDepth,
            limit: max_depth as u64,
            observed: depth as u64,
        });
    }

    match node {
        Value::String(s) => {
            push_record(current_path, s, ir, ctx)?;
        }
        Value::Integer(i) => {
            push_record(current_path, &i.to_string(), ir, ctx)?;
        }
        Value::Float(f) => {
            push_record(current_path, &f.to_string(), ir, ctx)?;
        }
        Value::Boolean(b) => {
            push_record(current_path, if *b { "true" } else { "false" }, ir, ctx)?;
        }
        Value::Datetime(dt) => {
            push_record(current_path, &dt.to_string(), ir, ctx)?;
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                push_record(current_path, "[]", ir, ctx)?;
            } else {
                for (idx, item) in arr.iter().enumerate() {
                    let path = append_index(current_path, idx);
                    emit_toml_value(item, &path, depth + 1, max_depth, ir, ctx)?;
                }
            }
        }
        Value::Table(table) => {
            if table.is_empty() {
                push_record(current_path, "{}", ir, ctx)?;
            } else {
                for (key, val) in table {
                    let path = append_key(current_path, key);
                    emit_toml_value(val, &path, depth + 1, max_depth, ir, ctx)?;
                }
            }
        }
    }

    Ok(())
}

fn push_record(
    path: &str,
    value: &str,
    ir: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    ctx.budget.charge_text(path.len() as u64)?;
    ctx.budget.charge_text(value.len() as u64)?;

    ir.push(
        BlockKind::Record {
            path: path.to_string(),
            value: value.to_string(),
        },
        Locator::default(),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;
    use crate::extractor::ResourceBudget;
    use crate::limits::IngestLimits;
    use crate::registry::ExtractorRegistry;

    fn mock_context(
        limits: IngestLimits,
    ) -> (
        ExtractorRegistry,
        IngestLimits,
        ResourceBudget,
        crate::limits::ExtractPolicy,
    ) {
        let reg = ExtractorRegistry::new_empty();
        let budget = ResourceBudget::new(limits.clone());
        let policy = crate::limits::ExtractPolicy::default();
        (reg, limits, budget, policy)
    }

    #[test]
    fn test_extract_toml_tables_and_datetime() {
        let (reg, limits, budget, policy) = mock_context(IngestLimits::default());
        let ctx = ExtractContext {
            limits: &limits,
            policy: &policy,
            budget: &budget,
            registry: &reg,
            filename: None,
        };

        let toml_src = b"title = \"TOML Demo\"\ndate = 1979-05-27T07:32:00Z\n\n[[products]]\nname = \"Hammer\"\nprice = 10\n";
        let ir = extract_toml(toml_src, &ctx).expect("TOML extraction should succeed");

        assert_eq!(ir.meta.title.as_deref(), Some("TOML Demo"));
        assert_eq!(ir.blocks.len(), 4);

        let records: Vec<(&str, &str)> = ir
            .blocks
            .iter()
            .map(|b| match &b.kind {
                BlockKind::Record { path, value } => (path.as_str(), value.as_str()),
                _ => panic!("Expected Record"),
            })
            .collect();

        assert_eq!(records[0], ("date", "1979-05-27T07:32:00Z"));
        assert_eq!(records[1], ("products[0].name", "Hammer"));
        assert_eq!(records[2], ("products[0].price", "10"));
        assert_eq!(records[3], ("title", "TOML Demo"));
    }
}
