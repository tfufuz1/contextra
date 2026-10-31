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
// ZWECK: EML (RFC 5322 / MIME) Extractor-Implementierung mit mail-parser.
// INVARIANTEN: Zero-Panic, Fail-Closed, kein unsafe, Einhaltung der Privacy- und Budget-Regeln.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::sanitize_text;
use crate::error::IngestError;
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::formats::email::html::strip_html;
use crate::ir::{
    BlockKind, Boundary, DocumentIr, ExtractWarning, Locator, MessageHeader, WarningCode,
};
use mail_parser::{
    Addr, Address, HeaderValue, Message, MessagePart, MessageParser, MimeHeaders, PartType,
};

/// Extrahiert eine einzelne EML-Nachricht.
pub fn extract_eml(
    input: &[u8],
    ctx: &ExtractContext<'_>,
    message_number: Option<u32>,
) -> Result<DocumentIr, IngestError> {
    // Budget für Dekompression / Text verbuchen
    ctx.budget.charge_decompressed(input.len() as u64)?;

    let msg = MessageParser::default()
        .parse(input)
        .ok_or_else(|| IngestError::Corrupt {
            format: Some(FormatId::Eml),
            detail: "E-Mail konnte nicht geparst werden".into(),
        })?;

    let mut doc = DocumentIr::new(FormatId::Eml);
    let locator = Locator {
        message: message_number,
        ..Locator::default()
    };

    // Header extrahieren
    let headers = extract_message_headers(&msg);

    // DECISION & INVARIANT: "keep_personal_metadata" betrifft nur Dokument-Metadaten (DocMeta.authors).
    // In MessageHeader (Message-Block) bleiben From/To/Cc unverändert, da Absender und Empfänger
    // wesentlicher Inhalt der E-Mail-Nachricht sind.
    if ctx.policy.keep_personal_metadata {
        doc.meta.authors = headers.from.clone();
    }

    if let Some(ref subj) = headers.subject {
        doc.meta.title = Some(subj.clone());
    }
    if let Some(ref date_str) = headers.date {
        doc.meta.created = Some(date_str.clone());
    }

    // Block 1: Message Header Block
    doc.push(BlockKind::Message(headers), locator.clone());

    // Body extrahieren und verarbeiten
    extract_body(&msg, &mut doc, &locator);

    // Anhänge verarbeiten
    extract_attachments(&msg, &mut doc, ctx, &locator)?;

    Ok(doc)
}

fn extract_message_headers(msg: &Message<'_>) -> MessageHeader {
    let subject = msg.subject().map(sanitize_header_str);

    let from = extract_addrs(msg.from());
    let to = extract_addrs(msg.to());
    let cc = extract_addrs(msg.cc());

    let date = msg.date().map(|d| {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            d.year, d.month, d.day, d.hour, d.minute, d.second
        )
    });

    let message_id = msg.message_id().map(sanitize_header_str);

    let in_reply_to = match msg.in_reply_to() {
        HeaderValue::Text(t) => Some(sanitize_header_str(t)),
        HeaderValue::TextList(tl) => tl.first().map(|t| sanitize_header_str(t)),
        _ => None,
    };

    MessageHeader {
        subject,
        from,
        to,
        cc,
        date,
        message_id,
        in_reply_to,
    }
}

fn extract_addrs(addr: Option<&Address<'_>>) -> Vec<String> {
    let mut addrs = Vec::new();
    if let Some(a) = addr {
        match a {
            Address::List(list) => {
                for item in list {
                    push_addr(&mut addrs, item);
                }
            }
            Address::Group(groups) => {
                for group in groups {
                    for item in &group.addresses {
                        push_addr(&mut addrs, item);
                    }
                }
            }
        }
    }
    addrs
}

fn push_addr(out: &mut Vec<String>, addr: &Addr<'_>) {
    let address = addr.address.as_deref().unwrap_or("");
    if address.is_empty() {
        return;
    }
    let clean_addr = sanitize_header_str(address);
    if let Some(name) = addr.name.as_deref() {
        let clean_name = sanitize_header_str(name);
        if !clean_name.is_empty() {
            out.push(format!("{} <{}>", clean_name, clean_addr));
            return;
        }
    }
    out.push(clean_addr);
}

fn sanitize_header_str(input: &str) -> String {
    let (clean, _) = sanitize_text(input);
    let single_line = clean.replace(['\n', '\t'], " ");
    single_line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_body(msg: &Message<'_>, doc: &mut DocumentIr, locator: &Locator) {
    let mut raw_text = String::new();
    let mut is_html = false;

    if let Some(text_part) = msg.text_part(0) {
        raw_text = extract_part_text(text_part, doc, &mut is_html);
    }

    if raw_text.is_empty() {
        if let Some(html_part) = msg.html_part(0) {
            raw_text = extract_part_text(html_part, doc, &mut is_html);
            is_html = true;
        }
    }

    if raw_text.is_empty() {
        return;
    }

    let text_content = if is_html {
        strip_html(&raw_text)
    } else {
        raw_text
    };

    // Body in Zeilen aufteilen & Signatur abspalten
    let lines = process_body_lines(&text_content);

    // Blöcke bilden: Quote-Gruppen vs. Absätze
    build_body_blocks(lines, doc, locator);
}

fn extract_part_text(part: &MessagePart<'_>, doc: &mut DocumentIr, is_html: &mut bool) -> String {
    if part.is_content_type("text", "html") {
        *is_html = true;
    }

    let charset_opt = part.content_type().and_then(|ct| ct.attribute("charset"));

    if let Some(charset) = charset_opt {
        if encoding_rs::Encoding::for_label(charset.as_bytes()).is_none() {
            // Unbekanntes Charset => Fallback auf Windows-1252 + WarningCode::EncodingGuessed
            doc.warnings.push(ExtractWarning {
                code: WarningCode::EncodingGuessed,
                detail: format!(
                    "Unbekannter Charset '{}', Fallback auf Windows-1252",
                    charset
                ),
            });
            let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(part.contents());
            return decoded.into_owned();
        }
    }

    if *is_html {
        return String::from_utf8_lossy(part.contents()).into_owned();
    }

    match &part.body {
        PartType::Text(t) => t.to_string(),
        PartType::Html(h) => {
            *is_html = true;
            h.to_string()
        }
        _ => {
            let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(part.contents());
            decoded.into_owned()
        }
    }
}

fn process_body_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_end();
        // RFC 3676 Signatur-Trennzeichen "-- " (oder "--\n" / "--\r\n")
        // DECISION: Signaturabschnitt nach "-- " wird verworfen.
        // Es wird weder WarningCode::TextSanitized noch eine andere Warnung erzeugt.
        if trimmed == "-- " || trimmed == "--" {
            break;
        }
        lines.push(line.to_string());
    }
    lines
}

fn build_body_blocks(lines: Vec<String>, doc: &mut DocumentIr, locator: &Locator) {
    let mut current_quote = Vec::new();
    let mut current_para = Vec::new();

    for line in lines {
        let trimmed_start = line.trim_start();
        if trimmed_start.starts_with('>') {
            if !current_para.is_empty() {
                flush_paragraph(&mut current_para, doc, locator);
            }
            // Leading '>' entfernen
            let quote_line = strip_quote_prefix(trimmed_start);
            current_quote.push(quote_line);
        } else {
            if !current_quote.is_empty() {
                flush_quote(&mut current_quote, doc, locator);
            }
            if line.trim().is_empty() {
                if !current_para.is_empty() {
                    flush_paragraph(&mut current_para, doc, locator);
                }
            } else {
                current_para.push(line);
            }
        }
    }

    if !current_quote.is_empty() {
        flush_quote(&mut current_quote, doc, locator);
    }
    if !current_para.is_empty() {
        flush_paragraph(&mut current_para, doc, locator);
    }
}

fn strip_quote_prefix(line: &str) -> String {
    let s = line.strip_prefix('>').unwrap_or(line);
    if let Some(rest) = s.strip_prefix(' ') {
        rest.to_string()
    } else {
        s.to_string()
    }
}

fn flush_paragraph(para: &mut Vec<String>, doc: &mut DocumentIr, locator: &Locator) {
    let text = para.join("\n");
    para.clear();
    let (clean_text, _) = sanitize_text(&text);
    let trimmed = clean_text.trim();
    if !trimmed.is_empty() {
        doc.push(
            BlockKind::Paragraph {
                text: trimmed.to_string(),
            },
            locator.clone(),
        );
    }
}

fn flush_quote(quote: &mut Vec<String>, doc: &mut DocumentIr, locator: &Locator) {
    let text = quote.join("\n");
    quote.clear();
    let (clean_text, _) = sanitize_text(&text);
    let trimmed = clean_text.trim();
    if !trimmed.is_empty() {
        doc.push(
            BlockKind::Quote {
                text: trimmed.to_string(),
            },
            locator.clone(),
        );
    }
}

fn extract_attachments(
    msg: &Message<'_>,
    doc: &mut DocumentIr,
    ctx: &ExtractContext<'_>,
    locator: &Locator,
) -> Result<(), IngestError> {
    for part in msg.attachments() {
        // Inline-Bilder nie verarbeiten
        if is_inline_image(part) {
            continue;
        }

        let filename = part
            .attachment_name()
            .map(sanitize_header_str)
            .unwrap_or_else(|| "attachment.bin".to_string());

        let mime = part
            .content_type()
            .map(|ct| ct.c_type.as_ref())
            .map(sanitize_header_str);

        let bytes = part.contents();
        if bytes.is_empty() {
            continue;
        }

        // Budget prüfen / verbuchen
        ctx.budget.charge_decompressed(bytes.len() as u64)?;

        // Verschachtelte Extraktion versuchen
        match ctx.extract_nested(&filename, mime.as_deref(), bytes) {
            Ok(nested_ir) => {
                doc.append_nested(&filename, mime.as_deref(), nested_ir);
            }
            Err(IngestError::LimitExceeded {
                kind,
                limit,
                observed,
            }) => {
                // Limit-Überschreitung (z.B. NestingDepth) MUSS hart scheitern
                return Err(IngestError::LimitExceeded {
                    kind,
                    limit,
                    observed,
                });
            }
            Err(err) => {
                // Sonstige Fehler => AttachmentSkipped Warnung, Nachricht bleibt intakt
                doc.warnings.push(ExtractWarning {
                    code: WarningCode::AttachmentSkipped,
                    detail: format!("{}: {}", filename, err),
                });
                doc.push(
                    BlockKind::Boundary(Boundary::Attachment {
                        name: filename,
                        mime,
                    }),
                    locator.clone(),
                );
            }
        }
    }

    Ok(())
}

fn is_inline_image(part: &MessagePart<'_>) -> bool {
    let is_inline = part.content_disposition().map(|d| d.c_type.as_ref()) == Some("inline");
    let is_image = part
        .content_type()
        .map(|ct| ct.c_type.as_ref().starts_with("image/"))
        .unwrap_or(false);

    is_inline && is_image
}
