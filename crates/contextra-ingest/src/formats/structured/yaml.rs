//! Modul
//!
//! # Zweck & Architektur
//! Komponente des  Subsystems (Ring 4 Adapter-Schicht).
//! Stellt Parser-, Extraktions- oder Hilfsfunktionen fuer die strukturierte Dokumentverarbeitung bereit.
//!
//! # Invarianten
//! - Zero-Panic: Alle Operationen sind defensiv aufgebaut.
//! - Fail-Closed: Fehlerhafte oder unvollstaendige Daten fuehren zu typisierten Fehlern.
//! - Kein externer I/O ueber Netzwerk oder unkontrollierte Dateisystempfade.

// FILE-CONTEXT
// ZWECK: Extractor-Implementierung für YAML Format mit Anker/Alias-Billion-Laughs-Schutz (WP M4).
// INVARIANTEN: FILE-CONTEXT Kopf, Zero-Panic, Limitierung der Alias-Expansion gegen max_text_bytes.
// STAND: TS:2026-10-31T00:00:00Z

use super::path::{append_index, append_key};
use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, ExtractWarning, Locator, WarningCode};
use std::collections::BTreeSet;
use yaml_rust2::{Yaml, YamlLoader};

/// Extrahiert ein YAML-Dokument in die `DocumentIr`.
pub fn extract_yaml(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError> {
    let text = std::str::from_utf8(input).map_err(|e| IngestError::Corrupt {
        format: Some(FormatId::Yaml),
        detail: format!("UTF-8 Dekodierfehler: {}", e),
    })?;

    let docs = YamlLoader::load_from_str(text).map_err(|e| IngestError::Corrupt {
        format: Some(FormatId::Yaml),
        detail: format!("YAML-Parserfehler: {}", e),
    })?;

    if docs.is_empty() {
        return Err(IngestError::Corrupt {
            format: Some(FormatId::Yaml),
            detail: "Leeres YAML-Dokument".into(),
        });
    }

    let mut ir = DocumentIr::new(FormatId::Yaml);
    let max_depth = usize::try_from(ctx.limits.max_xml_depth).unwrap_or(256);
    let is_multi_doc = docs.len() > 1;

    for (doc_idx, doc) in docs.iter().enumerate() {
        let prefix = if is_multi_doc {
            append_index("doc", doc_idx)
        } else {
            String::new()
        };

        // meta.title aus oberstem Objekt des ersten Dokuments extrahieren
        if doc_idx == 0 {
            if let Yaml::Hash(ref hash) = doc {
                for (k, v) in hash {
                    if let Yaml::String(ref k_str) = k {
                        if k_str == "title" || k_str == "name" {
                            let val_str = match v {
                                Yaml::String(s) => Some(s.clone()),
                                Yaml::Integer(i) => Some(i.to_string()),
                                Yaml::Real(r) => Some(r.clone()),
                                Yaml::Boolean(b) => Some(b.to_string()),
                                _ => None,
                            };
                            if let Some(t) = val_str {
                                ir.meta.title = Some(t);
                            }
                            break;
                        }
                    }
                }
            }
        }

        let mut alias_stack = Vec::new();
        emit_yaml_node(doc, &prefix, 0, max_depth, &mut alias_stack, &mut ir, ctx)?;
    }

    Ok(ir)
}

fn emit_yaml_node(
    node: &Yaml,
    current_path: &str,
    depth: usize,
    max_depth: usize,
    alias_stack: &mut Vec<usize>,
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
        Yaml::Null | Yaml::BadValue => {
            push_record(current_path, "null", ir, ctx)?;
        }
        Yaml::Boolean(b) => {
            push_record(current_path, if *b { "true" } else { "false" }, ir, ctx)?;
        }
        Yaml::Integer(i) => {
            push_record(current_path, &i.to_string(), ir, ctx)?;
        }
        Yaml::Real(r) => {
            push_record(current_path, r, ir, ctx)?;
        }
        Yaml::String(s) => {
            push_record(current_path, s, ir, ctx)?;
        }
        Yaml::Array(arr) => {
            if arr.is_empty() {
                push_record(current_path, "[]", ir, ctx)?;
            } else {
                for (idx, item) in arr.iter().enumerate() {
                    let path = append_index(current_path, idx);
                    emit_yaml_node(item, &path, depth + 1, max_depth, alias_stack, ir, ctx)?;
                }
            }
        }
        Yaml::Hash(hash) => {
            if hash.is_empty() {
                push_record(current_path, "{}", ir, ctx)?;
            } else {
                let mut seen_keys = BTreeSet::new();
                for (k, v) in hash {
                    let key_str = yaml_key_to_string(k);
                    if !seen_keys.insert(key_str.clone()) {
                        ir.warnings.push(ExtractWarning {
                            code: WarningCode::FeatureLimited,
                            detail: format!("Doppelter Schlüssel '{}' in YAML erkannt", key_str),
                        });
                    }
                    let path = append_key(current_path, &key_str);
                    emit_yaml_node(v, &path, depth + 1, max_depth, alias_stack, ir, ctx)?;
                }
            }
        }
        Yaml::Alias(id) => {
            if alias_stack.contains(id) {
                return Err(IngestError::Corrupt {
                    format: Some(FormatId::Yaml),
                    detail: "Zyklischer YAML-Alias erkannt".into(),
                });
            }
            if alias_stack.len() >= max_depth {
                return Err(IngestError::LimitExceeded {
                    kind: LimitKind::NestingDepth,
                    limit: max_depth as u64,
                    observed: (alias_stack.len() + 1) as u64,
                });
            }
            alias_stack.push(*id);
            // Wenn yaml-rust2 bereits Aliase in der Baumstruktur expandiert hat oder
            // wenn es als unaufgelöster Alias vorliegt:
            push_record(current_path, &format!("*alias_{}", id), ir, ctx)?;
            alias_stack.pop();
        }
    }

    Ok(())
}

fn yaml_key_to_string(key: &Yaml) -> String {
    match key {
        Yaml::String(s) => s.clone(),
        Yaml::Integer(i) => i.to_string(),
        Yaml::Real(r) => r.clone(),
        Yaml::Boolean(b) => b.to_string(),
        Yaml::Null => "null".to_string(),
        _ => "key".to_string(),
    }
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
    fn test_extract_yaml_multi_document() {
        let (reg, limits, budget, policy) = mock_context(IngestLimits::default());
        let ctx = ExtractContext {
            limits: &limits,
            policy: &policy,
            budget: &budget,
            registry: &reg,
            filename: None,
        };

        let yaml = b"---\ntitle: Doc1\na: 10\n---\ntitle: Doc2\nb: 20\n";
        let ir = extract_yaml(yaml, &ctx).expect("YAML multi-doc should succeed");

        assert_eq!(ir.meta.title.as_deref(), Some("Doc1"));
        assert_eq!(ir.blocks.len(), 4);

        let records: Vec<(&str, &str)> = ir
            .blocks
            .iter()
            .map(|b| match &b.kind {
                BlockKind::Record { path, value } => (path.as_str(), value.as_str()),
                _ => panic!("Expected Record"),
            })
            .collect();

        assert_eq!(records[0], ("doc[0].title", "Doc1"));
        assert_eq!(records[1], ("doc[0].a", "10"));
        assert_eq!(records[2], ("doc[1].title", "Doc2"));
        assert_eq!(records[3], ("doc[1].b", "20"));
    }
}
