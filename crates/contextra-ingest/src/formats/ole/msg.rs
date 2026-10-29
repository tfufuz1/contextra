// FILE-CONTEXT
// ZWECK: Extractor für MS-OXMSG (Outlook Message .msg Format) (WP M4).
// INVARIANTEN: Zero-Panic, substg-Properties (ANSI/UTF-16LE), E-Mail Header/Body,
//              FILETIME-Datumsdekodierung (ISO-8601 UTC, ohne Systemzeit),
//              Grobes HTML-Stripping bei reine HTML-Bodys, Anhangs-Extraktion über ctx.extract_nested,
//              Anhang-Fehler fangen als AttachmentSkipped, rekursive eingebettete Nachrichten.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::IngestError;
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::formats::ole::cfb_reader::SafeCfbReader;
use crate::ir::{BlockKind, DocumentIr, ExtractWarning, Locator, MessageHeader, WarningCode};

/// Property Tags nach MS-OXMSG.
const TAG_SUBJECT: &str = "0037";
const TAG_SENDER_NAME: &str = "0C1A";
const TAG_SENDER_EMAIL: &str = "0C1F";
const TAG_SENDER_SMTP: &str = "5D01";
const TAG_BODY: &str = "1000";
const TAG_BODY_HTML: &str = "1013";
const TAG_CLIENT_SUBMIT_TIME: &str = "0039";
const TAG_MESSAGE_DELIVERY_TIME: &str = "0E06";

const TAG_RECIP_EMAIL: &str = "0030";
const TAG_RECIP_NAME: &str = "3001";
const TAG_RECIP_SMTP: &str = "39FE";
const TAG_RECIP_TYPE: &str = "0C15";

const TAG_ATTACH_METHOD: &str = "3705";
const TAG_ATTACH_LONG_FILENAME: &str = "3707";
const TAG_ATTACH_FILENAME: &str = "3704";
const TAG_ATTACH_DATA: &str = "3701";

/// Extrahiert MS-OXMSG aus einer OLE/CFB Struktur.
pub fn extract_msg(
    reader: &mut SafeCfbReader<'_>,
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    extract_msg_in_storage(reader, "/", ctx)
}

/// Rekursive Extraktion eines MS-OXMSG aus einem Storage-Pfad.
fn extract_msg_in_storage(
    reader: &mut SafeCfbReader<'_>,
    storage_path: &str,
    ctx: &ExtractContext<'_>,
) -> Result<DocumentIr, IngestError> {
    let mut ir = DocumentIr::new(FormatId::Msg);

    // 1. Root/Sub-Storage Properties auslesen
    let subject = read_property_string(reader, storage_path, TAG_SUBJECT, ctx);
    let sender_name = read_property_string(reader, storage_path, TAG_SENDER_NAME, ctx);
    let sender_addr = read_property_string(reader, storage_path, TAG_SENDER_SMTP, ctx)
        .or_else(|| read_property_string(reader, storage_path, TAG_SENDER_EMAIL, ctx));

    let from_str = match (sender_name, sender_addr) {
        (Some(name), Some(addr)) => vec![format!("{} <{}>", name, addr)],
        (Some(name), None) => vec![name],
        (None, Some(addr)) => vec![addr],
        (None, None) => Vec::new(),
    };

    // 2. Datum aus FILETIME Property (0039 oder 0E06)
    let date_iso = read_property_filetime(reader, storage_path, TAG_CLIENT_SUBMIT_TIME, ctx)
        .or_else(|| read_property_filetime(reader, storage_path, TAG_MESSAGE_DELIVERY_TIME, ctx));

    // 3. Empfänger
    let mut to_list = Vec::new();
    let mut cc_list = Vec::new();
    parse_recipients(reader, storage_path, &mut to_list, &mut cc_list, ctx);

    // Header-Block erstellen
    let header = MessageHeader {
        subject: subject.clone(),
        from: from_str,
        to: to_list,
        cc: cc_list,
        date: date_iso.clone(),
        message_id: None,
        in_reply_to: None,
    };

    ir.meta.title = subject;
    ir.meta.created = date_iso;
    ir.push(BlockKind::Message(header), Locator::default());

    // 4. E-Mail Body
    let text_body = read_property_string(reader, storage_path, TAG_BODY, ctx);
    if let Some(body_text) = text_body {
        for line in body_text.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                ir.push(
                    BlockKind::Paragraph {
                        text: trimmed.to_string(),
                    },
                    Locator::default(),
                );
            }
        }
    } else if let Some(html_body) = read_property_string(reader, storage_path, TAG_BODY_HTML, ctx) {
        ir.warnings.push(ExtractWarning {
            code: WarningCode::FeatureLimited,
            detail: "Nur HTML-Body vorhanden; grobes HTML-Tag-Stripping angewendet".into(),
        });
        let stripped = strip_html_coarse(&html_body);
        for line in stripped.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                ir.push(
                    BlockKind::Paragraph {
                        text: trimmed.to_string(),
                    },
                    Locator::default(),
                );
            }
        }
    }

    // 5. Anhänge
    parse_attachments(reader, storage_path, ctx, &mut ir);

    Ok(ir)
}

/// Liest eine Eigenschaft als String (prüft UTF-16LE .001F und ANSI .001E).
fn read_property_string(
    reader: &mut SafeCfbReader<'_>,
    storage_path: &str,
    prop_tag: &str,
    ctx: &ExtractContext<'_>,
) -> Option<String> {
    let prefix = if storage_path == "/" {
        "/__substg1.0_".to_string()
    } else {
        format!("{}/__substg1.0_", storage_path.trim_end_matches('/'))
    };

    let utf16_path = format!("{}{}{}", prefix, prop_tag, "001F");
    if reader.exists(&utf16_path) {
        if let Ok(bytes) = reader.read_stream(&utf16_path, ctx) {
            let u16s: Vec<u16> = bytes
                .chunks_exact(2)
                .filter_map(|chunk| match (chunk.first(), chunk.get(1)) {
                    (Some(&b0), Some(&b1)) => Some(u16::from_le_bytes([b0, b1])),
                    _ => None,
                })
                .collect();
            let s = String::from_utf16_lossy(&u16s);
            let trimmed = s.trim_matches('\0').trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    let ansi_path = format!("{}{}{}", prefix, prop_tag, "001E");
    if reader.exists(&ansi_path) {
        if let Ok(bytes) = reader.read_stream(&ansi_path, ctx) {
            let s: String = bytes.iter().map(|&b| b as char).collect();
            let trimmed = s.trim_matches('\0').trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

/// Liest ein U32 Property (Type 0003).
fn read_property_u32(
    reader: &mut SafeCfbReader<'_>,
    storage_path: &str,
    prop_tag: &str,
    ctx: &ExtractContext<'_>,
) -> Option<u32> {
    let prefix = if storage_path == "/" {
        "/__substg1.0_".to_string()
    } else {
        format!("{}/__substg1.0_", storage_path.trim_end_matches('/'))
    };

    let path = format!("{}{}{}", prefix, prop_tag, "0003");
    if reader.exists(&path) {
        if let Ok(bytes) = reader.read_stream(&path, ctx) {
            if let (Some(&b0), Some(&b1), Some(&b2), Some(&b3)) =
                (bytes.first(), bytes.get(1), bytes.get(2), bytes.get(3))
            {
                return Some(u32::from_le_bytes([b0, b1, b2, b3]));
            }
        }
    }

    None
}

/// Liest ein FILETIME Property (Type 0040) und wandelt es in ISO-8601 UTC um.
fn read_property_filetime(
    reader: &mut SafeCfbReader<'_>,
    storage_path: &str,
    prop_tag: &str,
    ctx: &ExtractContext<'_>,
) -> Option<String> {
    let prefix = if storage_path == "/" {
        "/__substg1.0_".to_string()
    } else {
        format!("{}/__substg1.0_", storage_path.trim_end_matches('/'))
    };

    let path = format!("{}{}{}", prefix, prop_tag, "0040");
    if reader.exists(&path) {
        if let Ok(bytes) = reader.read_stream(&path, ctx) {
            if let (Some(&b0), Some(&b1), Some(&b2), Some(&b3), Some(&b4), Some(&b5), Some(&b6), Some(&b7)) = (
                bytes.first(),
                bytes.get(1),
                bytes.get(2),
                bytes.get(3),
                bytes.get(4),
                bytes.get(5),
                bytes.get(6),
                bytes.get(7),
            ) {
                let ft = u64::from_le_bytes([b0, b1, b2, b3, b4, b5, b6, b7]);
                return filetime_to_iso8601(ft);
            }
        }
    }

    None
}

/// Wandelt ein Windows FILETIME (100ns Intervalle seit 1601-01-01 UTC) ohne Systemzeit in ISO-8601 um.
fn filetime_to_iso8601(filetime: u64) -> Option<String> {
    if filetime == 0 {
        return None;
    }

    let secs_1601 = filetime / 10_000_000;
    if secs_1601 < 11_644_473_600 {
        return None;
    }
    let unix_secs = secs_1601 - 11_644_473_600;

    let sec = (unix_secs % 60) as u32;
    let min = ((unix_secs / 60) % 60) as u32;
    let hour = ((unix_secs / 3600) % 24) as u32;
    let mut days = unix_secs / 86400;

    let mut year = 1970u32;
    loop {
        let leap = is_leap_year(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year = year.saturating_add(1);
    }

    let leap = is_leap_year(year);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut month = 1u32;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month = month.saturating_add(1);
    }
    let day = u32::try_from(days.saturating_add(1)).unwrap_or(31);

    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hour, min, sec
    ))
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Liest Empfänger aus `__recip_version1.0_#...`
fn parse_recipients(
    reader: &mut SafeCfbReader<'_>,
    parent_storage_path: &str,
    to_list: &mut Vec<String>,
    cc_list: &mut Vec<String>,
    ctx: &ExtractContext<'_>,
) {
    if let Ok(entries) = reader.walk_all() {
        let parent_prefix = if parent_storage_path == "/" {
            "".to_string()
        } else {
            parent_storage_path.trim_end_matches('/').to_string()
        };

        for entry in entries {
            let raw_path = entry.path().to_string_lossy().replace('\\', "/");
            let path = if raw_path.starts_with('/') {
                raw_path
            } else {
                format!("/{}", raw_path)
            };

            if path.starts_with(&parent_prefix) && path.contains("__recip_version1.0_#") && reader.is_storage(&path) {
                let recip_type = read_property_u32(reader, &path, TAG_RECIP_TYPE, ctx).unwrap_or(1);
                let name = read_property_string(reader, &path, TAG_RECIP_NAME, ctx);
                let addr = read_property_string(reader, &path, TAG_RECIP_SMTP, ctx)
                    .or_else(|| read_property_string(reader, &path, TAG_RECIP_EMAIL, ctx));

                let recip_str = match (name, addr) {
                    (Some(n), Some(a)) => format!("{} <{}>", n, a),
                    (Some(n), None) => n,
                    (None, Some(a)) => a,
                    (None, None) => continue,
                };

                if recip_type == 2 {
                    cc_list.push(recip_str);
                } else {
                    to_list.push(recip_str);
                }
            }
        }
    }
}

/// Liest Anhänge aus `__attach_version1.0_#...`
fn parse_attachments(
    reader: &mut SafeCfbReader<'_>,
    parent_storage_path: &str,
    ctx: &ExtractContext<'_>,
    ir: &mut DocumentIr,
) {
    let parent_prefix = if parent_storage_path == "/" {
        "".to_string()
    } else {
        parent_storage_path.trim_end_matches('/').to_string()
    };

    let mut attach_paths = Vec::new();
    if let Ok(entries) = reader.walk_all() {
        for entry in entries {
            let raw_path = entry.path().to_string_lossy().replace('\\', "/");
            let path = if raw_path.starts_with('/') {
                raw_path
            } else {
                format!("/{}", raw_path)
            };

            if path.starts_with(&parent_prefix)
                && path.contains("__attach_version1.0_#")
                && reader.is_storage(&path)
                && !attach_paths.contains(&path)
            {
                attach_paths.push(path);
            }
        }
    }

    for path in attach_paths {
        let attach_method = read_property_u32(reader, &path, TAG_ATTACH_METHOD, ctx).unwrap_or(1);
        let filename = read_property_string(reader, &path, TAG_ATTACH_LONG_FILENAME, ctx)
            .or_else(|| read_property_string(reader, &path, TAG_ATTACH_FILENAME, ctx))
            .unwrap_or_else(|| "attachment.bin".to_string());

        if attach_method == 1 {
            let data_path = format!(
                "{}/__substg1.0_{}0102",
                path.trim_end_matches('/'),
                TAG_ATTACH_DATA
            );
            if reader.exists(&data_path) {
                match reader.read_stream(&data_path, ctx) {
                    Ok(data_bytes) => match ctx.extract_nested(&filename, None, &data_bytes) {
                        Ok(nested_ir) => {
                            ir.append_nested(&filename, None, nested_ir);
                        }
                        Err(err) => {
                            ir.warnings.push(ExtractWarning {
                                code: WarningCode::AttachmentSkipped,
                                detail: format!(
                                    "Anhang '{}' konnte nicht verarbeitet werden: {}",
                                    filename, err
                                ),
                            });
                        }
                    },
                    Err(err) => {
                        ir.warnings.push(ExtractWarning {
                            code: WarningCode::AttachmentSkipped,
                            detail: format!(
                                "Anhang-Daten stream '{}' nicht lesbar: {}",
                                filename, err
                            ),
                        });
                    }
                }
            }
        } else if attach_method == 5 {
            // AF_EMBEDDED_MSG
            let embed_msg_name = if filename.ends_with(".msg") {
                filename.clone()
            } else {
                format!("{}.msg", filename)
            };

            let data_path = format!(
                "{}/__substg1.0_{}0102",
                path.trim_end_matches('/'),
                TAG_ATTACH_DATA
            );

            if reader.exists(&data_path) {
                // Embedded MSG binary stream
                if let Ok(data_bytes) = reader.read_stream(&data_path, ctx) {
                    match ctx.extract_nested(&embed_msg_name, Some("application/vnd.ms-outlook"), &data_bytes) {
                        Ok(nested_ir) => {
                            ir.append_nested(&embed_msg_name, Some("application/vnd.ms-outlook"), nested_ir);
                        }
                        Err(err) => {
                            ir.warnings.push(ExtractWarning {
                                code: WarningCode::AttachmentSkipped,
                                detail: format!(
                                    "Eingebettete Nachricht '{}' konnte nicht extrahiert werden: {}",
                                    embed_msg_name, err
                                ),
                            });
                        }
                    }
                }
            } else if let Ok(nested_ir) = extract_msg_in_storage(reader, &path, ctx) {
                // Embedded MSG sub-storage
                ir.append_nested(&embed_msg_name, Some("application/vnd.ms-outlook"), nested_ir);
            } else {
                ir.warnings.push(ExtractWarning {
                    code: WarningCode::AttachmentSkipped,
                    detail: format!("Eingebettete Nachricht '{}' nicht lesbar", embed_msg_name),
                });
            }
        }
    }
}

/// Grobes HTML-Tag-Stripping ohne externe Bibliotheken oder Panic.
fn strip_html_coarse(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut in_style = false;
    let mut in_script = false;

    let lower = html.to_lowercase();
    let chars: Vec<char> = html.chars().collect();
    let lower_chars: Vec<char> = lower.chars().collect();
    let len = chars.len();
    let mut i = 0usize;

    while i < len {
        let remaining = lower_chars.get(i..);

        if !in_tag && starts_with_chars(remaining, &['<', 's', 't', 'y', 'l', 'e']) {
            in_style = true;
            in_tag = true;
        } else if !in_tag && starts_with_chars(remaining, &['<', 's', 'c', 'r', 'i', 'p', 't']) {
            in_script = true;
            in_tag = true;
        } else if in_style && starts_with_chars(remaining, &['<', '/', 's', 't', 'y', 'l', 'e', '>']) {
            in_style = false;
            in_tag = false;
            i = i.saturating_add(8);
            continue;
        } else if in_script && starts_with_chars(remaining, &['<', '/', 's', 'c', 'r', 'i', 'p', 't', '>']) {
            in_script = false;
            in_tag = false;
            i = i.saturating_add(9);
            continue;
        } else if !in_style && !in_script {
            if let Some(&ch) = chars.get(i) {
                if ch == '<' {
                    in_tag = true;
                } else if ch == '>' {
                    in_tag = false;
                    out.push(' ');
                } else if !in_tag {
                    out.push(ch);
                }
            }
        }
        i = i.saturating_add(1);
    }

    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

fn starts_with_chars(slice: Option<&[char]>, prefix: &[char]) -> bool {
    match slice {
        Some(s) if s.len() >= prefix.len() => s.starts_with(prefix),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_coarse_safety() {
        let html_with_umlauts =
            "<html><style>body { color: red; }</style><body><h1>Überschrift &amp; Test</h1><p>Äöü βγδ ☃ text</p></body></html>";
        let stripped = strip_html_coarse(html_with_umlauts);
        assert!(stripped.contains("Überschrift & Test"));
        assert!(stripped.contains("Äöü βγδ ☃ text"));
        assert!(!stripped.contains("color: red"));
    }
}
