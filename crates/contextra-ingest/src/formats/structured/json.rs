// FILE-CONTEXT
// ZWECK: Extractor-Implementierung für JSON und JSONL Formate (WP M4).
// INVARIANTEN: FILE-CONTEXT Kopf, Zero-Panic, Erhalt doppelter Schlüssel in Dokumentreihenfolge.
// STAND: TS:2026-10-31T00:00:00Z

use super::path::{append_index, append_key};
use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, ExtractWarning, Locator, WarningCode};
use serde::de::{Deserializer as _, MapAccess, SeqAccess, Visitor};
use std::collections::BTreeSet;
use std::fmt;

/// Eigenes JSON-AST-Element zur Erhaltung doppelter Schlüssel und exakter Dokumentreihenfolge.
#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

#[derive(Clone, Copy)]
struct JsonVisitor {
    current_depth: usize,
    max_depth: usize,
}

impl<'de> serde::de::DeserializeSeed<'de> for JsonVisitor {
    type Value = JsonValue;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for JsonVisitor {
    type Value = JsonValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ein gültiges JSON-Element")
    }

    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
        Ok(JsonValue::Bool(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
        Ok(JsonValue::Number(v.into()))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
        Ok(JsonValue::Number(v.into()))
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
        if let Some(n) = serde_json::Number::from_f64(v) {
            Ok(JsonValue::Number(n))
        } else {
            Ok(JsonValue::Null)
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(JsonValue::String(v.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
        Ok(JsonValue::String(v))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(JsonValue::Null)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(JsonValue::Null)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if self.current_depth >= self.max_depth {
            return Err(serde::de::Error::custom("LIMIT_EXCEEDED_DEPTH"));
        }

        let mut items = Vec::new();
        let next_visitor = JsonVisitor {
            current_depth: self.current_depth + 1,
            max_depth: self.max_depth,
        };

        while let Some(item) = seq.next_element_seed(next_visitor)? {
            items.push(item);
        }

        Ok(JsonValue::Array(items))
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        if self.current_depth >= self.max_depth {
            return Err(serde::de::Error::custom("LIMIT_EXCEEDED_DEPTH"));
        }

        let mut entries = Vec::new();
        let next_visitor = JsonVisitor {
            current_depth: self.current_depth + 1,
            max_depth: self.max_depth,
        };

        while let Some(key) = map.next_key::<String>()? {
            let value = map.next_value_seed(next_visitor)?;
            entries.push((key, value));
        }

        Ok(JsonValue::Object(entries))
    }
}

impl JsonValue {
    pub fn parse(bytes: &[u8], max_depth: usize) -> Result<JsonValue, IngestError> {
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        let visitor = JsonVisitor {
            current_depth: 0,
            max_depth,
        };

        deserializer.deserialize_any(visitor).map_err(|err| {
            let msg = err.to_string();
            if msg.contains("LIMIT_EXCEEDED_DEPTH") || msg.contains("recursion limit exceeded") {
                IngestError::LimitExceeded {
                    kind: LimitKind::NestingDepth,
                    limit: max_depth as u64,
                    observed: (max_depth + 1) as u64,
                }
            } else {
                IngestError::Corrupt {
                    format: Some(FormatId::Json),
                    detail: msg,
                }
            }
        })
    }
}

/// Extrahiert ein JSON-Dokument in die `DocumentIr`.
pub fn extract_json(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError> {
    let max_depth = usize::try_from(ctx.limits.max_xml_depth).unwrap_or(256);
    let root = JsonValue::parse(input, max_depth)?;

    let mut ir = DocumentIr::new(FormatId::Json);

    // meta.title aus oberstem Objekt extrahieren
    if let JsonValue::Object(ref entries) = root {
        for (k, v) in entries {
            if k == "title" || k == "name" {
                match v {
                    JsonValue::String(s) => ir.meta.title = Some(s.clone()),
                    JsonValue::Number(n) => ir.meta.title = Some(n.to_string()),
                    _ => {}
                }
                break;
            }
        }
    }

    emit_records(&root, "", &mut ir, ctx)?;
    Ok(ir)
}

/// Extrahiert ein JSONL-Dokument zeilenweise in die `DocumentIr`.
pub fn extract_jsonl(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError> {
    let mut ir = DocumentIr::new(FormatId::Jsonl);
    let max_depth = usize::try_from(ctx.limits.max_xml_depth).unwrap_or(256);
    let max_lines = u64::from(ctx.limits.max_archive_entries);

    let text = std::str::from_utf8(input).map_err(|e| IngestError::Corrupt {
        format: Some(FormatId::Jsonl),
        detail: format!("UTF-8 Dekodierfehler: {}", e),
    })?;

    let mut line_index = 0u64;
    let mut valid_lines = 0u64;
    let mut invalid_lines = 0u64;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        line_index += 1;
        if line_index > max_lines {
            return Err(IngestError::LimitExceeded {
                kind: LimitKind::ArchiveEntries,
                limit: max_lines,
                observed: line_index,
            });
        }

        let line_offset = usize::try_from(line_index.saturating_sub(1)).unwrap_or(usize::MAX);
        let prefix = append_index("", line_offset);
        match JsonValue::parse(trimmed.as_bytes(), max_depth) {
            Ok(value) => {
                valid_lines += 1;
                emit_records(&value, &prefix, &mut ir, ctx)?;
            }
            Err(_) => {
                invalid_lines += 1;
            }
        }
    }

    if valid_lines == 0 && line_index > 0 {
        return Err(IngestError::Corrupt {
            format: Some(FormatId::Jsonl),
            detail: "Alle JSONL-Zeilen sind ungültig".into(),
        });
    }

    if invalid_lines > 0 {
        ir.warnings.push(ExtractWarning {
            code: WarningCode::FeatureLimited,
            detail: format!("{} ungültige Zeile(n) in JSONL übersprungen", invalid_lines),
        });
    }

    Ok(ir)
}

fn emit_records(
    node: &JsonValue,
    current_path: &str,
    ir: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
) -> Result<(), IngestError> {
    match node {
        JsonValue::Null => {
            push_record(current_path, "null", ir, ctx)?;
        }
        JsonValue::Bool(b) => {
            push_record(current_path, if *b { "true" } else { "false" }, ir, ctx)?;
        }
        JsonValue::Number(n) => {
            push_record(current_path, &n.to_string(), ir, ctx)?;
        }
        JsonValue::String(s) => {
            push_record(current_path, s, ir, ctx)?;
        }
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                push_record(current_path, "[]", ir, ctx)?;
            } else {
                for (idx, item) in arr.iter().enumerate() {
                    let path = append_index(current_path, idx);
                    emit_records(item, &path, ir, ctx)?;
                }
            }
        }
        JsonValue::Object(entries) => {
            if entries.is_empty() {
                push_record(current_path, "{}", ir, ctx)?;
            } else {
                let mut seen_keys = BTreeSet::new();
                for (key, val) in entries {
                    if !seen_keys.insert(key.clone()) {
                        ir.warnings.push(ExtractWarning {
                            code: WarningCode::FeatureLimited,
                            detail: format!("Doppelter Schlüssel '{}' in JSON erkannt", key),
                        });
                    }
                    let path = append_key(current_path, key);
                    emit_records(val, &path, ir, ctx)?;
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
    fn test_extract_json_duplicate_keys_and_scalars() {
        let (reg, limits, budget, policy) = mock_context(IngestLimits::default());
        let ctx = ExtractContext {
            limits: &limits,
            policy: &policy,
            budget: &budget,
            registry: &reg,
            filename: None,
        };

        let json = br#"{"title": "Demo", "a": 1, "a": 2, "empty_obj": {}, "empty_arr": []}"#;
        let ir = extract_json(json, &ctx).expect("JSON extraction should succeed");

        assert_eq!(ir.meta.title.as_deref(), Some("Demo"));
        assert_eq!(ir.blocks.len(), 5);

        assert_eq!(ir.warnings.len(), 1);
        assert_eq!(ir.warnings[0].code, WarningCode::FeatureLimited);

        let records: Vec<(&str, &str)> = ir
            .blocks
            .iter()
            .map(|b| match &b.kind {
                BlockKind::Record { path, value } => (path.as_str(), value.as_str()),
                _ => panic!("Expected Record block"),
            })
            .collect();

        assert_eq!(records[0], ("title", "Demo"));
        assert_eq!(records[1], ("a", "1"));
        assert_eq!(records[2], ("a", "2"));
        assert_eq!(records[3], ("empty_obj", "{}"));
        assert_eq!(records[4], ("empty_arr", "[]"));
    }

    #[test]
    fn test_extract_jsonl_with_invalid_line() {
        let (reg, limits, budget, policy) = mock_context(IngestLimits::default());
        let ctx = ExtractContext {
            limits: &limits,
            policy: &policy,
            budget: &budget,
            registry: &reg,
            filename: None,
        };

        let jsonl = b"{\"x\": 10}\n{invalid_json}\n{\"y\": 20}\n";
        let ir = extract_jsonl(jsonl, &ctx).expect("JSONL extraction should succeed");

        assert_eq!(ir.blocks.len(), 2);
        assert_eq!(ir.warnings.len(), 1);
        assert_eq!(ir.warnings[0].code, WarningCode::FeatureLimited);

        let r0 = match &ir.blocks[0].kind {
            BlockKind::Record { path, value } => (path.as_str(), value.as_str()),
            _ => panic!("Expected Record"),
        };
        let r1 = match &ir.blocks[1].kind {
            BlockKind::Record { path, value } => (path.as_str(), value.as_str()),
            _ => panic!("Expected Record"),
        };

        assert_eq!(r0, ("[0].x", "10"));
        assert_eq!(r1, ("[2].y", "20"));
    }
}
