// FILE-CONTEXT
// ZWECK: EmailChunker für E-Mail-Nachrichten und Anhänge (EML, MBOX, MSG) (WP M4 / WP-ING-15).
// INVARIANTEN: Zero-Panic, deterministisch, keine nachrichtenübergreifende Bündelung.
// STAND: TS:2026-10-31T00:00:00Z

use crate::canonical::CanonicalText;
use crate::chunking::common::{token_count, window_split};
use crate::chunking::config::ChunkingConfig;
use crate::chunking::draft_block_range;
use crate::chunking::{ChunkDraft, IrChunker};
use crate::error::IngestError;
use crate::format::{ChunkStrategy, FormatId};
use crate::ir::{BlockKind, Boundary, DocumentIr, Locator, MessageHeader};
use serde_json::{json, Map, Value};
use std::ops::Range;

/// Chunker für E-Mail-Formate (EML, MBOX, MSG).
#[derive(Debug, Default)]
pub struct EmailChunker;

impl IrChunker for EmailChunker {
    fn strategy(&self) -> ChunkStrategy {
        ChunkStrategy::Email
    }

    fn draft(
        &self,
        ir: &DocumentIr,
        canon: &CanonicalText,
        cfg: &ChunkingConfig,
    ) -> Result<Vec<ChunkDraft>, IngestError> {
        let max_tokens = cfg.max_tokens;
        let hard_limit = (max_tokens.saturating_mul(cfg.hard_limit_pct as usize)) / 100;
        let hard_limit = hard_limit.max(10);

        let include_quoted = cfg.email.include_quoted;
        let header_context = cfg.email.header_context;

        // 1. Blöcke nach Nachrichten und Anhängen strukturieren
        let messages = parse_email_messages(ir, canon);

        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_drafts = Vec::new();

        for msg in messages {
            let short_header = build_short_header(&msg.header);
            let subject = msg
                .header
                .subject
                .clone()
                .unwrap_or_else(|| "Kein Betreff".to_string());

            // 2. Body-Blöcke verarbeiten
            let mut skipped_quotes_cnt = 0usize;
            let mut body_items = Vec::new();

            for item in msg.body_blocks {
                if matches!(item.kind, BlockKind::Quote { .. }) && !include_quoted {
                    skipped_quotes_cnt = skipped_quotes_cnt.saturating_add(1);
                    continue;
                }
                body_items.push(item);
            }

            if body_items.is_empty() && msg.attachment_ranges.is_empty() {
                // Leere Nachricht (nur Header)
                let text = match canon.text.get(msg.header_span.clone()) {
                    Some(s) => s.to_string(),
                    None => String::new(),
                };

                let final_text = if header_context {
                    format!("{}\n\n{}", short_header, text)
                } else {
                    text
                };

                let mut extra = Map::new();
                if header_context {
                    extra.insert("synthesized".into(), Value::Bool(true));
                }

                let mut locator = msg.header_locator.clone();
                locator.message = Some(msg.message_index);

                all_drafts.push(ChunkDraft {
                    text: final_text,
                    span: msg.header_span.clone(),
                    locator,
                    breadcrumb: Some(subject.clone()),
                    heading_level: None,
                    extra,
                });
                continue;
            }

            // Body Chunks erzeugen
            if !body_items.is_empty() {
                let emit_ctx = BodyEmitContext {
                    msg_idx: msg.message_index,
                    subject: &subject,
                    short_header: &short_header,
                    header_context,
                    skipped_quotes_cnt,
                    hard_limit,
                };
                emit_body_chunks(&body_items, &emit_ctx, canon, &mut all_drafts)?;
            }

            // 3. Anhang-Blöcke verarbeiten über draft_block_range
            for att in msg.attachment_ranges {
                let strategy = determine_attachment_strategy(&att.name, att.mime.as_deref());
                let sub_drafts = draft_block_range(ir, canon, att.block_range, strategy, cfg)?;

                for mut draft in sub_drafts {
                    draft.locator.message = Some(msg.message_index);
                    draft.locator.attachment = Some(att.name.clone());
                    draft.breadcrumb = Some(format!("{} > Anhang: {}", subject, att.name));

                    if header_context {
                        draft.text = format!("{}\n\n{}", short_header, draft.text);
                        draft.extra.insert("synthesized".into(), Value::Bool(true));
                    }

                    all_drafts.push(draft);
                }
            }
        }

        Ok(all_drafts)
    }
}

#[derive(Debug)]
struct EmailBlockItem<'a> {
    kind: &'a BlockKind,
    span: Range<usize>,
    locator: Locator,
    tokens: usize,
}

#[derive(Debug)]
struct AttachmentSection {
    name: String,
    mime: Option<String>,
    block_range: Range<usize>,
}

#[derive(Debug)]
struct MessageData<'a> {
    message_index: u32,
    header: MessageHeader,
    header_span: Range<usize>,
    header_locator: Locator,
    body_blocks: Vec<EmailBlockItem<'a>>,
    attachment_ranges: Vec<AttachmentSection>,
}

#[derive(Debug)]
struct BodyEmitContext<'a> {
    msg_idx: u32,
    subject: &'a str,
    short_header: &'a str,
    header_context: bool,
    skipped_quotes_cnt: usize,
    hard_limit: usize,
}

fn parse_email_messages<'a>(ir: &'a DocumentIr, canon: &CanonicalText) -> Vec<MessageData<'a>> {
    let mut messages = Vec::new();
    let mut msg_counter = 0u32;

    let mut current_msg: Option<MessageData<'a>> = None;
    let mut active_attachment: Option<(String, Option<String>, usize)> = None; // name, mime, start_block_idx

    for (block_idx, (block, span)) in ir.blocks.iter().zip(canon.block_spans.iter()).enumerate() {
        match &block.kind {
            BlockKind::Message(header) => {
                // Aktiven Anhang der vorherigen Nachricht abschließen
                if let (Some(ref mut msg), Some((att_name, att_mime, start_idx))) =
                    (&mut current_msg, active_attachment.take())
                {
                    msg.attachment_ranges.push(AttachmentSection {
                        name: att_name,
                        mime: att_mime,
                        block_range: start_idx..block_idx,
                    });
                }

                if let Some(msg) = current_msg.take() {
                    messages.push(msg);
                }

                msg_counter = msg_counter.saturating_add(1);
                let msg_idx = block.locator.message.unwrap_or(msg_counter);

                let header_item = EmailBlockItem {
                    kind: &block.kind,
                    span: span.clone(),
                    locator: block.locator.clone(),
                    tokens: token_count(&canon.text[span.clone()]),
                };

                let mut new_msg = MessageData {
                    message_index: msg_idx,
                    header: header.clone(),
                    header_span: span.clone(),
                    header_locator: block.locator.clone(),
                    body_blocks: Vec::new(),
                    attachment_ranges: Vec::new(),
                };
                new_msg.body_blocks.push(header_item);

                current_msg = Some(new_msg);
            }
            BlockKind::Boundary(Boundary::Attachment { name, mime }) => {
                if let (Some(ref mut msg), Some((att_name, att_mime, start_idx))) =
                    (&mut current_msg, active_attachment.take())
                {
                    msg.attachment_ranges.push(AttachmentSection {
                        name: att_name,
                        mime: att_mime,
                        block_range: start_idx..block_idx,
                    });
                }
                active_attachment = Some((name.clone(), mime.clone(), block_idx));
            }
            _ => {
                if let Some(ref mut msg) = current_msg {
                    let att_name_opt = block
                        .locator
                        .attachment
                        .clone()
                        .or_else(|| active_attachment.as_ref().map(|(n, _, _)| n.clone()));

                    if let Some(att_name) = att_name_opt {
                        if active_attachment.is_none() {
                            active_attachment = Some((att_name, None, block_idx));
                        }
                    } else {
                        // Body-Block
                        if let Some((att_name, att_mime, start_idx)) = active_attachment.take() {
                            msg.attachment_ranges.push(AttachmentSection {
                                name: att_name,
                                mime: att_mime,
                                block_range: start_idx..block_idx,
                            });
                        }

                        let text_slice = &canon.text[span.clone()];
                        let tok_cnt = token_count(text_slice);

                        msg.body_blocks.push(EmailBlockItem {
                            kind: &block.kind,
                            span: span.clone(),
                            locator: block.locator.clone(),
                            tokens: tok_cnt,
                        });
                    }
                }
            }
        }
    }

    // Letzten Anhang und Nachricht abschließen
    if let (Some(ref mut msg), Some((att_name, att_mime, start_idx))) =
        (&mut current_msg, active_attachment.take())
    {
        msg.attachment_ranges.push(AttachmentSection {
            name: att_name,
            mime: att_mime,
            block_range: start_idx..ir.blocks.len(),
        });
    }

    if let Some(msg) = current_msg.take() {
        messages.push(msg);
    }

    messages
}

fn emit_body_chunks(
    items: &[EmailBlockItem<'_>],
    ctx: &BodyEmitContext<'_>,
    canon: &CanonicalText,
    out: &mut Vec<ChunkDraft>,
) -> Result<(), IngestError> {
    let mut current_batch = Vec::new();
    let mut current_toks = 0usize;

    for item in items {
        if item.tokens > ctx.hard_limit && items.len() == 1 {
            // Einzelner übergroßer Body-Block -> Window Split
            emit_oversize_body_windows(item, ctx, canon, out)?;
            return Ok(());
        }

        if !current_batch.is_empty() && current_toks.saturating_add(item.tokens) > ctx.hard_limit {
            finalize_body_batch(&current_batch, ctx, canon, out)?;
            current_batch.clear();
            current_toks = 0;
        }

        current_batch.push(item);
        current_toks = current_toks.saturating_add(item.tokens);
    }

    if !current_batch.is_empty() {
        finalize_body_batch(&current_batch, ctx, canon, out)?;
    }

    Ok(())
}

fn finalize_body_batch(
    batch: &[&EmailBlockItem<'_>],
    ctx: &BodyEmitContext<'_>,
    canon: &CanonicalText,
    out: &mut Vec<ChunkDraft>,
) -> Result<(), IngestError> {
    let first = match batch.first() {
        Some(item) => item,
        None => return Ok(()),
    };
    let last = match batch.last() {
        Some(item) => item,
        None => return Ok(()),
    };

    let span = first.span.start..last.span.end;
    let base_text = match canon.text.get(span.clone()) {
        Some(s) => s.to_string(),
        None => return Ok(()),
    };

    let final_text = if ctx.header_context {
        format!("{}\n\n{}", ctx.short_header, base_text)
    } else {
        base_text
    };

    let mut extra = Map::new();
    if ctx.header_context {
        extra.insert("synthesized".into(), Value::Bool(true));
    }
    if ctx.skipped_quotes_cnt > 0 {
        extra.insert("quoted_skipped".into(), json!(ctx.skipped_quotes_cnt));
    }

    let mut locator = first.locator.clone();
    locator.message = Some(ctx.msg_idx);

    out.push(ChunkDraft {
        text: final_text,
        span,
        locator,
        breadcrumb: Some(ctx.subject.to_string()),
        heading_level: None,
        extra,
    });

    Ok(())
}

fn emit_oversize_body_windows(
    item: &EmailBlockItem<'_>,
    ctx: &BodyEmitContext<'_>,
    canon: &CanonicalText,
    out: &mut Vec<ChunkDraft>,
) -> Result<(), IngestError> {
    let window_chars = 1500usize;
    let overlap_chars = 150usize;

    let text_str = match canon.text.get(item.span.clone()) {
        Some(s) => s,
        None => return Ok(()),
    };

    let windows = window_split(text_str, window_chars, overlap_chars);

    for win_span in windows {
        let win_text = text_str.get(win_span).unwrap_or_default();

        let final_text = if ctx.header_context {
            format!("{}\n\n{}", ctx.short_header, win_text)
        } else {
            win_text.to_string()
        };

        let mut extra = Map::new();
        if ctx.header_context {
            extra.insert("synthesized".into(), Value::Bool(true));
        }
        if ctx.skipped_quotes_cnt > 0 {
            extra.insert("quoted_skipped".into(), json!(ctx.skipped_quotes_cnt));
        }

        let mut locator = item.locator.clone();
        locator.message = Some(ctx.msg_idx);

        out.push(ChunkDraft {
            text: final_text,
            span: item.span.clone(),
            locator,
            breadcrumb: Some(ctx.subject.to_string()),
            heading_level: None,
            extra,
        });
    }

    Ok(())
}

fn build_short_header(header: &MessageHeader) -> String {
    let subject = header.subject.as_deref().unwrap_or("Kein Betreff").trim();
    let from = if header.from.is_empty() {
        "Unbekannt".to_string()
    } else {
        header.from.join(", ")
    };
    let date = header.date.as_deref().unwrap_or("").trim();

    let res = if date.is_empty() {
        format!("{} | {}", subject, from)
    } else {
        format!("{} | {} | {}", subject, from, date)
    };

    if res.chars().count() > 200 {
        res.chars().take(200).collect()
    } else {
        res
    }
}

fn determine_attachment_strategy(filename: &str, mime: Option<&str>) -> ChunkStrategy {
    let ext_opt = filename.split('.').next_back();
    if let Some(ext) = ext_opt {
        if let Some(fmt) = FormatId::from_extension(ext) {
            return fmt.default_strategy();
        }
    }

    if let Some(mime_str) = mime {
        if mime_str.contains("csv")
            || mime_str.contains("spreadsheet")
            || mime_str.contains("excel")
        {
            return ChunkStrategy::Tabular;
        }
        if mime_str.contains("pdf") {
            return ChunkStrategy::Paged;
        }
        if mime_str.contains("json") || mime_str.contains("yaml") || mime_str.contains("xml") {
            return ChunkStrategy::Structured;
        }
    }

    ChunkStrategy::Prose
}
