// FILE-CONTEXT
// ZWECK: Golden- und Vertragstests für E-Mail- (EML) und MBOX-Extraktor.
// INVARIANTEN: Zero-Panic, deterministisch, Fail-Closed bei Limits, Einhaltung der Privacy-Regeln.
// STAND: TS:2026-10-31T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use contextra_ingest::error::{IngestError, LimitKind};
use contextra_ingest::extractor::ExtractContext;
use contextra_ingest::format::FormatId;
use contextra_ingest::ir::{BlockKind, Boundary, WarningCode};
use contextra_ingest::limits::{ExtractPolicy, IngestLimits};
use contextra_ingest::registry::ExtractorRegistry;

fn make_ctx<'a>(
    limits: &'a IngestLimits,
    policy: &'a ExtractPolicy,
    budget: &'a contextra_ingest::extractor::ResourceBudget,
    registry: &'a ExtractorRegistry,
) -> ExtractContext<'a> {
    ExtractContext {
        limits,
        policy,
        budget,
        registry,
        filename: None,
    }
}

#[test]
fn test_email_plain_umlauts_utf8_iso8859_qp() {
    let raw_eml = b"From: sender@example.com\n\
To: receiver@example.com\n\
Subject: =?ISO-8859-1?Q?H=F6henluft_und_Gr=FC=DF23?=\n\
Content-Type: text/plain; charset=utf-8\n\
Content-Transfer-Encoding: quoted-printable\n\
\n\
Hallo Welt, =C3=9Cberfall und Gr=C3=BC=C3=9Fe!\n";

    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let doc = reg.extract(FormatId::Eml, raw_eml, &ctx).unwrap();

    assert_eq!(doc.format, FormatId::Eml);
    assert_eq!(doc.meta.title.as_deref(), Some("Höhenluft und Grüß23"));

    // Check blocks
    let mut found_msg_header = false;
    let mut found_paragraph = false;

    for block in &doc.blocks {
        match &block.kind {
            BlockKind::Message(hdr) => {
                found_msg_header = true;
                assert_eq!(hdr.from, vec!["sender@example.com"]);
                assert_eq!(hdr.to, vec!["receiver@example.com"]);
            }
            BlockKind::Paragraph { text } => {
                found_paragraph = true;
                assert!(text.contains("Überfall und Grüße"));
            }
            _ => {}
        }
    }

    assert!(found_msg_header);
    assert!(found_paragraph);
}

#[test]
fn test_email_multipart_alternative_preference() {
    let raw_eml = b"From: sender@example.com\n\
Subject: Alternative Test\n\
Content-Type: multipart/alternative; boundary=\"ALT_BOUND\"\n\
\n\
--ALT_BOUND\n\
Content-Type: text/plain; charset=utf-8\n\
\n\
Plain text body preference\n\
--ALT_BOUND\n\
Content-Type: text/html; charset=utf-8\n\
\n\
<b>HTML text body</b>\n\
--ALT_BOUND--\n";

    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let doc = reg.extract(FormatId::Eml, raw_eml, &ctx).unwrap();

    let mut plain_found = false;
    for block in &doc.blocks {
        if let BlockKind::Paragraph { text } = &block.kind {
            if text.contains("Plain text body preference") {
                plain_found = true;
            }
            assert!(!text.contains("<b>HTML"));
        }
    }
    assert!(plain_found);
}

#[test]
fn test_email_html_only_script_prompt_injection_discarded() {
    let raw_eml = b"From: html@example.com\n\
Subject: HTML Only\n\
Content-Type: text/html; charset=utf-8\n\
\n\
<html><body>\n\
<script>alert('system prompt injection attack');</script>\n\
<style>body { color: red; }</style>\n\
<p>Safe paragraph text</p>\n\
</body></html>\n";

    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let doc = reg.extract(FormatId::Eml, raw_eml, &ctx).unwrap();

    let canon = doc.canonical();
    assert!(canon.text.contains("Safe paragraph text"));
    assert!(!canon.text.contains("prompt injection"));
    assert!(!canon.text.contains("alert"));
    assert!(!canon.text.contains("color: red"));
}

#[test]
fn test_email_quoted_reply_and_signature_discarded() {
    let raw_eml = b"From: alice@example.com\n\
Subject: Re: Meeting\n\
Content-Type: text/plain; charset=utf-8\n\
\n\
Thanks for the update.\n\
\n\
> First line of quote\n\
> Second line of quote\n\
\n\
-- \n\
Alice Smith\n\
CEO Example Corp\n";

    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let doc = reg.extract(FormatId::Eml, raw_eml, &ctx).unwrap();

    let mut para_found = false;
    let mut quote_found = false;

    for block in &doc.blocks {
        match &block.kind {
            BlockKind::Paragraph { text } => {
                if text.contains("Thanks for the update") {
                    para_found = true;
                }
                assert!(!text.contains("Alice Smith"));
            }
            BlockKind::Quote { text } => {
                if text.contains("First line of quote") && text.contains("Second line of quote") {
                    quote_found = true;
                }
            }
            _ => {}
        }
    }

    assert!(para_found);
    assert!(quote_found);

    // Signatur muss komplett verworfen worden sein
    let canon = doc.canonical();
    assert!(!canon.text.contains("Alice Smith"));
    assert!(!canon.text.contains("CEO Example Corp"));
}

#[test]
fn test_email_attachments_txt_and_recursive_eml() {
    let raw_eml = b"From: parent@example.com\n\
Subject: Parent Mail\n\
Content-Type: multipart/mixed; boundary=\"MIX_BOUND\"\n\
\n\
--MIX_BOUND\n\
Content-Type: text/plain\n\
\n\
Main email body\n\
--MIX_BOUND\n\
Content-Type: text/plain; name=\"notes.txt\"\n\
Content-Disposition: attachment; filename=\"notes.txt\"\n\
\n\
Attached notes text\n\
--MIX_BOUND\n\
Content-Type: message/rfc822; name=\"sub.eml\"\n\
Content-Disposition: attachment; filename=\"sub.eml\"\n\
\n\
From: child@example.com\n\
Subject: Child Mail\n\
\n\
Child email body\n\
--MIX_BOUND--\n";

    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let doc = reg.extract(FormatId::Eml, raw_eml, &ctx).unwrap();

    let mut attachment_boundaries = 0;
    let mut child_body_found = false;

    for block in &doc.blocks {
        match &block.kind {
            BlockKind::Boundary(Boundary::Attachment { name, .. }) => {
                attachment_boundaries += 1;
                assert!(name == "notes.txt" || name == "sub.eml");
            }
            BlockKind::Paragraph { text } => {
                if text.contains("Child email body") {
                    child_body_found = true;
                }
            }
            _ => {}
        }
    }

    assert_eq!(attachment_boundaries, 2);
    assert!(child_body_found);
}

#[test]
fn test_email_broken_attachment_handled_gracefully() {
    let raw_eml = b"From: sender@example.com\n\
Subject: Broken Attachment\n\
Content-Type: multipart/mixed; boundary=\"BOUND\"\n\
\n\
--BOUND\n\
Content-Type: text/plain\n\
\n\
Body content\n\
--BOUND\n\
Content-Type: application/unknown-binary-format; name=\"bad.xyz\"\n\
Content-Disposition: attachment; filename=\"bad.xyz\"\n\
\n\
BINARY_GARBAGE_DATA_12345\n\
--BOUND--\n";

    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let doc = reg.extract(FormatId::Eml, raw_eml, &ctx).unwrap();

    // Die E-Mail-Nachricht muss intakt bleiben
    let canon = doc.canonical();
    assert!(canon.text.contains("Body content"));

    // Warnung AttachmentSkipped muss vorhanden sein
    let warning_found = doc
        .warnings
        .iter()
        .any(|w| w.code == WarningCode::AttachmentSkipped);
    assert!(warning_found);
}

#[test]
fn test_email_mbox_three_messages_one_defective() {
    let raw_mbox = b"From sender1@example.com Mon Jan  1 10:00:00 2026\n\
From: sender1@example.com\n\
Subject: First Mail\n\
\n\
Body of first mail\n\
\n\
From sender2@example.com Mon Jan  1 11:00:00 2026\n\
Bad-Corrupt-Header-Line-Without-Colon-Or-Value\n\
\n\
From sender3@example.com Mon Jan  1 12:00:00 2026\n\
From: sender3@example.com\n\
Subject: Third Mail\n\
\n\
Body of third mail\n";

    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let doc = reg.extract(FormatId::Mbox, raw_mbox, &ctx).unwrap();

    assert_eq!(doc.format, FormatId::Mbox);
    assert_eq!(doc.meta.title.as_deref(), Some("First Mail"));

    let canon = doc.canonical();
    assert!(canon.text.contains("Body of first mail"));
    assert!(canon.text.contains("Body of third mail"));
}

#[test]
fn test_email_mbox_from_unescaping() {
    let raw_mbox = b"From sender@example.com Mon Jan  1 10:00:00 2026\n\
From: sender@example.com\n\
Subject: Mbox Escaping\n\
\n\
Line 1\n\
>From line that was escaped in mbox\n\
Line 3\n";

    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let doc = reg.extract(FormatId::Mbox, raw_mbox, &ctx).unwrap();

    let canon = doc.canonical();
    assert!(canon.text.contains("From line that was escaped in mbox"));
    assert!(!canon.text.contains(">From line"));
}

#[test]
fn test_email_keep_personal_metadata_policy() {
    let raw_eml = b"From: Alice <alice@example.com>\n\
To: Bob <bob@example.com>\n\
Subject: Privacy Policy Test\n\
\n\
Mail body content\n";

    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();

    // Policy 1: keep_personal_metadata = false
    let mut policy_no_pii = ExtractPolicy::default();
    policy_no_pii.keep_personal_metadata = false;

    let budget1 = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx1 = make_ctx(&limits, &policy_no_pii, &budget1, &reg);

    let doc1 = reg.extract(FormatId::Eml, raw_eml, &ctx1).unwrap();

    // DocMeta.authors muss leer sein
    assert!(doc1.meta.authors.is_empty());

    // MessageHeader muss From/To/Cc trotzdem enthalten
    if let BlockKind::Message(hdr) = &doc1.blocks[0].kind {
        assert_eq!(hdr.from, vec!["Alice <alice@example.com>"]);
        assert_eq!(hdr.to, vec!["Bob <bob@example.com>"]);
    } else {
        panic!("First block must be MessageHeader");
    }

    // Policy 2: keep_personal_metadata = true
    let mut policy_pii = ExtractPolicy::default();
    policy_pii.keep_personal_metadata = true;

    let budget2 = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx2 = make_ctx(&limits, &policy_pii, &budget2, &reg);

    let doc2 = reg.extract(FormatId::Eml, raw_eml, &ctx2).unwrap();

    // DocMeta.authors ist befüllt
    assert_eq!(doc2.meta.authors, vec!["Alice <alice@example.com>"]);
}

#[test]
fn test_email_nesting_depth_bomb() {
    let mut bomb = String::from("From: root@example.com\nSubject: Depth Bomb\n");
    for i in 0..15 {
        bomb.push_str(&format!(
            "Content-Type: message/rfc822; name=\"nest{}.eml\"\n\nFrom: level{}@example.com\nSubject: L{}\n",
            i, i, i
        ));
    }

    let reg = ExtractorRegistry::with_defaults();
    let mut limits = IngestLimits::default();
    limits.max_nesting_depth = 3;

    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let err = reg.extract(FormatId::Eml, bomb.as_bytes(), &ctx).unwrap_err();

    assert!(matches!(
        err,
        IngestError::LimitExceeded {
            kind: LimitKind::NestingDepth,
            ..
        }
    ));
}

#[test]
fn test_email_mbox_max_archive_entries_limit() {
    let mut mbox_data = Vec::new();
    for i in 0..10 {
        mbox_data.extend_from_slice(
            format!(
                "From sender{}@example.com\nFrom: sender{}@example.com\nSubject: Mail {}\n\nBody {}\n\n",
                i, i, i, i
            )
            .as_bytes(),
        );
    }

    let reg = ExtractorRegistry::with_defaults();
    let mut limits = IngestLimits::default();
    limits.max_archive_entries = 5;

    let policy = ExtractPolicy::default();
    let budget = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
    let ctx = make_ctx(&limits, &policy, &budget, &reg);

    let err = reg
        .extract(FormatId::Mbox, &mbox_data, &ctx)
        .unwrap_err();

    assert!(matches!(
        err,
        IngestError::LimitExceeded {
            kind: LimitKind::ArchiveEntries,
            ..
        }
    ));
}

#[test]
fn test_email_fuzz_random_bytes_no_panic() {
    let reg = ExtractorRegistry::with_defaults();
    let limits = IngestLimits::default();
    let policy = ExtractPolicy::default();

    let seeds: [&[u8]; 6] = [
        b"",
        b"\x00\xff\xfe\x12\x34",
        b"From: \x00\r\n\r\n\xfe",
        b"Content-Type: multipart/mixed; boundary=\"---\"\n\n-----",
        b"From \x00\n>From \x00\n",
        b"Subject: =?UTF-8?Q?=FF?= \r\n",
    ];

    for seed in seeds {
        let budget1 = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
        let ctx1 = make_ctx(&limits, &policy, &budget1, &reg);
        let _ = reg.extract(FormatId::Eml, seed, &ctx1);

        let budget2 = contextra_ingest::extractor::ResourceBudget::new(limits.clone());
        let ctx2 = make_ctx(&limits, &policy, &budget2, &reg);
        let _ = reg.extract(FormatId::Mbox, seed, &ctx2);
    }
}
