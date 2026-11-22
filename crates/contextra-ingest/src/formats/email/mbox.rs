// FILE-CONTEXT
// ZWECK: MBOX (RFC 4155) Extractor-Implementierung.
// INVARIANTEN: Zero-Panic, isolierte Nachrichtenverarbeitung, max_archive_entries Limitierung.
// STAND: TS:2026-10-31T00:00:00Z

use crate::error::{IngestError, LimitKind};
use crate::extractor::ExtractContext;
use crate::format::FormatId;
use crate::formats::email::eml::extract_eml;
use crate::ir::{DocumentIr, ExtractWarning, WarningCode};

/// Extrahiert eine MBOX-Datei mit mehreren Nachrichten.
pub fn extract_mbox(input: &[u8], ctx: &ExtractContext<'_>) -> Result<DocumentIr, IngestError> {
    // Message-Grenzen in der MBOX ermitteln
    let raw_messages = split_mbox_messages(input);

    let count = raw_messages.len() as u64;
    let max_entries = u64::from(ctx.limits.max_archive_entries);

    if count > max_entries {
        return Err(IngestError::LimitExceeded {
            kind: LimitKind::ArchiveEntries,
            limit: max_entries,
            observed: count,
        });
    }

    if raw_messages.is_empty() {
        return Err(IngestError::Corrupt {
            format: Some(FormatId::Mbox),
            detail: "MBOX enthält keine Nachrichten".into(),
        });
    }

    let mut doc = DocumentIr::new(FormatId::Mbox);
    let mut success_count = 0u32;
    let mut corrupt_count = 0u32;

    for (idx, raw_msg) in raw_messages.into_iter().enumerate() {
        let msg_number = u32::try_from(idx).unwrap_or(u32::MAX).saturating_add(1);

        // ">From "-Escaping zurückübersetzen
        let unescaped_msg = unescape_mbox_from(&raw_msg);

        match extract_eml(&unescaped_msg, ctx, Some(msg_number)) {
            Ok(mut msg_ir) => {
                success_count = success_count.saturating_add(1);

                // meta.title = Betreff der ersten erfolgreich geparsten Nachricht
                if doc.meta.title.is_none() && msg_ir.meta.title.is_some() {
                    doc.meta.title = msg_ir.meta.title.take();
                }

                doc.blocks.append(&mut msg_ir.blocks);
                doc.warnings.append(&mut msg_ir.warnings);
            }
            Err(IngestError::LimitExceeded {
                kind,
                limit,
                observed,
            }) => {
                // Hard fail bei Limit-Überschreitungen
                return Err(IngestError::LimitExceeded {
                    kind,
                    limit,
                    observed,
                });
            }
            Err(err) => {
                corrupt_count = corrupt_count.saturating_add(1);
                doc.warnings.push(ExtractWarning {
                    code: WarningCode::FeatureLimited,
                    detail: format!("Nachricht {} beschädigt: {}", msg_number, err),
                });
            }
        }
    }

    // Wenn alle Nachrichten defekt sind => Corrupt
    if corrupt_count > 0 && success_count == 0 {
        return Err(IngestError::Corrupt {
            format: Some(FormatId::Mbox),
            detail: "Alle Nachrichten in der MBOX sind beschädigt".into(),
        });
    }

    Ok(doc)
}

fn split_mbox_messages(input: &[u8]) -> Vec<Vec<u8>> {
    let mut messages = Vec::new();
    let mut current = Vec::new();

    let mut is_start_of_line = true;
    let mut blank_line_preceding = true;
    let mut current_line_has_non_cr = false;

    let len = input.len();
    let mut i = 0;

    while i < len {
        if is_start_of_line && blank_line_preceding && starts_with_from(input, i) {
            if !current.is_empty() {
                messages.push(current);
                current = Vec::new();
            }
        }

        if let Some(&b) = input.get(i) {
            current.push(b);
            if b == b'\n' {
                blank_line_preceding = !current_line_has_non_cr;
                current_line_has_non_cr = false;
                is_start_of_line = true;
            } else {
                if b != b'\r' {
                    current_line_has_non_cr = true;
                }
                is_start_of_line = false;
            }
        }
        i = i.saturating_add(1);
    }

    if !current.is_empty() {
        messages.push(current);
    }

    messages
}

fn starts_with_from(input: &[u8], pos: usize) -> bool {
    input.get(pos..pos.saturating_add(5)) == Some(b"From ")
}

fn unescape_mbox_from(msg: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(msg.len());
    let mut is_start_of_line = true;
    let len = msg.len();
    let mut i = 0;

    while i < len {
        if is_start_of_line && msg.get(i..i.saturating_add(6)) == Some(b">From ") {
            out.extend_from_slice(b"From ");
            i = i.saturating_add(6);
            is_start_of_line = false;
            continue;
        }

        if let Some(&b) = msg.get(i) {
            out.push(b);
            if b == b'\n' {
                is_start_of_line = true;
            } else if b != b'\r' {
                is_start_of_line = false;
            }
        }
        i = i.saturating_add(1);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mbox_unescape() {
        let input = b"From sender@test.com\n>From escaped line\nNormal line";
        let unescaped = unescape_mbox_from(input);
        let s = String::from_utf8_lossy(&unescaped);
        assert!(s.contains("From escaped line"));
        assert!(!s.contains(">From escaped line"));
    }

    #[test]
    fn test_mbox_split_only_after_empty_line() {
        let input = b"From sender1@test.com Mon Jan 1 10:00:00 2026\n\
From: sender1@test.com\n\
Header: From not a split line\n\
\n\
Body line 1\n\
\n\
From sender2@test.com Mon Jan 1 11:00:00 2026\n\
From: sender2@test.com\n";

        let messages = split_mbox_messages(input);
        assert_eq!(messages.len(), 2);
    }
}
