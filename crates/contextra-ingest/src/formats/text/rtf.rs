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
// ZWECK: RTF (Rich Text Format) Extractor ohne externe Abhängigkeiten (WP M4).
// INVARIANTEN: Zero-Panic, Stack-Tiefe aus max_xml_depth, \u/\'hh/\v/Tabellen/\info Handling.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::sanitize_text;
use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::ir::{BlockKind, DocumentIr, Locator, Table};

#[derive(Debug, Clone)]
struct RtfGroupState {
    skip_group: bool,
    is_info: bool,
    is_title: bool,
    is_hidden: bool,
    uc_count: usize,
}

impl Default for RtfGroupState {
    fn default() -> Self {
        Self {
            skip_group: false,
            is_info: false,
            is_title: false,
            is_hidden: false,
            uc_count: 1,
        }
    }
}

/// Extrahiert RTF-Dokumente unter Einhaltung des Verschachtelungslimits und der ExtractPolicy.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::indexing_slicing)]
pub fn parse_rtf(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError> {
    let mut ir = DocumentIr::new(FormatId::Rtf);
    let max_depth = ctx.limits.max_xml_depth as usize;

    let mut stack: Vec<RtfGroupState> = vec![RtfGroupState::default()];
    let mut current_text = String::new();
    let mut title_text = String::new();

    let mut current_row: Vec<String> = Vec::new();
    let mut current_cell = String::new();
    let mut table_rows: Vec<Vec<String>> = Vec::new();

    let mut codepage = 1252u16;
    let mut idx = 0usize;

    while idx < input.len() {
        let b = match input.get(idx) {
            Some(&byte) => byte,
            None => break,
        };

        if b == b'{' {
            if stack.len() >= max_depth {
                return Err(IngestError::LimitExceeded {
                    kind: LimitKind::NestingDepth,
                    limit: ctx.limits.max_xml_depth as u64,
                    observed: stack.len() as u64,
                });
            }

            let parent_state = stack.last().cloned().unwrap_or_default();
            stack.push(parent_state);
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'}' {
            if stack.len() > 1 {
                let closing = stack.pop().unwrap_or_default();
                if closing.is_title && !title_text.is_empty() {
                    let (clean_title, _) = sanitize_text(&title_text);
                    ir.meta.title = Some(clean_title);
                }
            }
            idx = idx.saturating_add(1);
            continue;
        }

        let state = stack.last().cloned().unwrap_or_default();

        if state.skip_group {
            idx = idx.saturating_add(1);
            continue;
        }

        if b == b'\\' {
            // Control Word oder Control Symbol
            idx = idx.saturating_add(1);
            if idx >= input.len() {
                break;
            }

            let next_b = match input.get(idx) {
                Some(&byte) => byte,
                None => break,
            };

            // Escaped Sonderzeichen
            if next_b == b'\\' || next_b == b'{' || next_b == b'}' {
                push_char(
                    &mut current_text,
                    &mut current_cell,
                    &mut title_text,
                    &state,
                    ctx,
                    next_b as char,
                );
                idx = idx.saturating_add(1);
                continue;
            }

            // Hex Escape \'hh
            if next_b == b'\'' {
                idx = idx.saturating_add(1);
                if idx.saturating_add(2) <= input.len() {
                    let hex_str = std::str::from_utf8(&input[idx..idx.saturating_add(2)]).unwrap_or_default();
                    if let Ok(byte_val) = u8::from_str_radix(hex_str, 16) {
                        let ch = decode_cp1252(byte_val, codepage);
                        push_char(
                            &mut current_text,
                            &mut current_cell,
                            &mut title_text,
                            &state,
                            ctx,
                            ch,
                        );
                    }
                    idx = idx.saturating_add(2);
                }
                continue;
            }

            // Control Word name lesen
            let mut word = String::new();
            while idx < input.len() {
                let cb = match input.get(idx) {
                    Some(&byte) => byte,
                    None => break,
                };
                if cb.is_ascii_alphabetic() {
                    word.push(cb as char);
                    idx = idx.saturating_add(1);
                } else {
                    break;
                }
            }

            // Optional numeric parameter
            let mut arg_str = String::new();
            let mut negative = false;
            if idx < input.len() && input.get(idx) == Some(&b'-') {
                negative = true;
                idx = idx.saturating_add(1);
            }
            while idx < input.len() {
                let cb = match input.get(idx) {
                    Some(&byte) => byte,
                    None => break,
                };
                if cb.is_ascii_digit() {
                    arg_str.push(cb as char);
                    idx = idx.saturating_add(1);
                } else {
                    break;
                }
            }

            let param: Option<i32> = if !arg_str.is_empty() {
                arg_str
                    .parse::<i32>()
                    .ok()
                    .map(|n| if negative { -n } else { n })
            } else {
                None
            };

            // Spatium nach Control Word verzehren
            if idx < input.len() && input.get(idx) == Some(&b' ') {
                idx = idx.saturating_add(1);
            }

            // Process Control Word
            match word.as_str() {
                "ansicpg" => {
                    if let Some(cpg) = param {
                        codepage = cpg as u16;
                    }
                }
                "*" => {
                    // Skip ignorable destination group
                    if let Some(top) = stack.last_mut() {
                        top.skip_group = true;
                    }
                }
                "fonttbl" | "colortbl" | "stylesheet" | "pict" | "object" => {
                    if let Some(top) = stack.last_mut() {
                        top.skip_group = true;
                    }
                }
                "info" => {
                    if let Some(top) = stack.last_mut() {
                        top.is_info = true;
                    }
                }
                "title" => {
                    if let Some(top) = stack.last_mut() {
                        if top.is_info {
                            top.is_title = true;
                        }
                    }
                }
                "v" => {
                    let val = param.unwrap_or(1);
                    if let Some(top) = stack.last_mut() {
                        top.is_hidden = val != 0;
                    }
                }
                "uc" => {
                    let count = param.unwrap_or(1).max(0) as usize;
                    if let Some(top) = stack.last_mut() {
                        top.uc_count = count;
                    }
                }
                "u" => {
                    if let Some(code_val) = param {
                        let u16_val = (code_val as i16) as u16;
                        if let Some(ch) = char::from_u32(u16_val as u32) {
                            push_char(
                                &mut current_text,
                                &mut current_cell,
                                &mut title_text,
                                &state,
                                ctx,
                                ch,
                            );
                        }
                    }
                    // Skip uc_count representation bytes
                    idx = idx.saturating_add(state.uc_count);
                }
                "par" => {
                    if !current_text.trim().is_empty() {
                        let (clean, _) = sanitize_text(&current_text);
                        ir.push(BlockKind::Paragraph { text: clean }, Locator::default());
                        current_text.clear();
                    }
                }
                "line" => {
                    push_char(
                        &mut current_text,
                        &mut current_cell,
                        &mut title_text,
                        &state,
                        ctx,
                        '\n',
                    );
                }
                "tab" => {
                    push_char(
                        &mut current_text,
                        &mut current_cell,
                        &mut title_text,
                        &state,
                        ctx,
                        '\t',
                    );
                }
                "trowd" => {
                    current_row.clear();
                    current_cell.clear();
                }
                "cell" => {
                    let (clean_cell, _) = sanitize_text(&current_cell);
                    current_row.push(clean_cell);
                    current_cell.clear();
                }
                "row" => {
                    if !current_row.is_empty() {
                        table_rows.push(current_row.clone());
                        current_row.clear();
                    }
                }
                _ => {}
            }
            continue;
        }

        // Standard Text Zeichen
        if b != b'\r' && b != b'\n' {
            let ch = if b < 0x80 {
                b as char
            } else {
                decode_cp1252(b, codepage)
            };
            push_char(
                &mut current_text,
                &mut current_cell,
                &mut title_text,
                &state,
                ctx,
                ch,
            );
        }
        idx = idx.saturating_add(1);
    }

    // Flush table if present
    if !table_rows.is_empty() {
        ir.push(
            BlockKind::Table(Table {
                caption: None,
                header_rows: 1,
                rows: table_rows,
                origin: (0, 0),
            }),
            Locator::default(),
        );
    }

    // Flush remaining text
    if !current_text.trim().is_empty() {
        let (clean, _) = sanitize_text(&current_text);
        ir.push(BlockKind::Paragraph { text: clean }, Locator::default());
    }

    Ok(ir)
}

fn push_char(
    current_text: &mut String,
    current_cell: &mut String,
    title_text: &mut String,
    state: &RtfGroupState,
    ctx: &ExtractContext<'_>,
    ch: char,
) {
    if state.is_hidden && !ctx.policy.include_hidden {
        return;
    }
    if state.is_title {
        title_text.push(ch);
    } else {
        current_text.push(ch);
        current_cell.push(ch);
    }
}

fn decode_cp1252(byte: u8, _cpg: u16) -> char {
    let buf = [byte];
    let (cow, _) = encoding_rs::WINDOWS_1252.decode_without_bom_handling(&buf);
    cow.chars().next().unwrap_or('?')
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_rtf_basic_parsing() {
        let rtf_bytes = br#"{\rtf1\ansi{\fonttbl{\f0 Arial;}}{\info{\title Mein RTF Dokument}}\f0\fs24 Hallo \b Welt\b0!\par Neue Zeile.}"#;
        let limits = crate::limits::IngestLimits::default();
        let policy = crate::limits::ExtractPolicy::default();
        let budget = crate::extractor::ResourceBudget::new(limits.clone());
        let registry = crate::registry::ExtractorRegistry::new_empty();
        let ctx = ExtractContext {
            limits: &limits,
            policy: &policy,
            budget: &budget,
            registry: &registry,
            filename: Some("test.rtf"),
        };

        let ir = parse_rtf(rtf_bytes, &ctx).expect("RTF parsing failed");
        assert_eq!(ir.meta.title.as_deref(), Some("Mein RTF Dokument"));
        assert!(!ir.blocks.is_empty());
    }
}
