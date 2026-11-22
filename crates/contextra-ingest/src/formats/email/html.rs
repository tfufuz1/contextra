// FILE-CONTEXT
// ZWECK: Minimale, abhängigkeitsfreie Entfernung von HTML-Tags und Skripten für E-Mails.
// INVARIANTEN: Zero-Panic, kein html5ever, Verwerfen von <script>/<style> samt Inhalt.
// STAND: TS:2026-10-31T00:00:00Z

/// Minimaler HTML-Entferner für E-Mail-Bodies, wenn nur HTML verfügbar ist.
///
/// DECISION & LIMITATION:
/// Skript- und Style-Blöcke (<script...>, <style...>) werden inklusive Inhalt vollständig verworfen,
/// um Prompt-Injection und unerwünschte CSS-Artefakte zu verhindern.
/// `html5ever` wird hier bewusst nicht verwendet, um das E-Mail-Modul unabhängig vom `fmt-web`-Feature zu halten.
pub fn strip_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let bytes = html.as_bytes();
    let len = bytes.len();
    let mut pos = 0;

    while pos < len {
        if bytes.get(pos) == Some(&b'<') {
            // Check for <script> or <style>
            if match_tag_start(bytes, pos, b"script") {
                pos = skip_until_end_tag(bytes, pos, b"script");
                continue;
            }
            if match_tag_start(bytes, pos, b"style") {
                pos = skip_until_end_tag(bytes, pos, b"style");
                continue;
            }
            // Block-level element line break handling
            if match_block_tag(bytes, pos) && !out.ends_with('\n') && !out.is_empty() {
                out.push('\n');
            }
            // Skip tag
            pos = skip_tag(bytes, pos);
        } else if bytes.get(pos) == Some(&b'&') {
            // HTML entities
            let (decoded_char, next_pos) = decode_entity(html, pos);
            out.push_str(decoded_char);
            pos = next_pos;
        } else if let Some(&b) = bytes.get(pos) {
            // UTF-8 sequence boundary safety
            let ch_len = utf8_char_len(b);
            if let Some(slice) = html.get(pos..pos.saturating_add(ch_len)) {
                out.push_str(slice);
                pos = pos.saturating_add(ch_len);
            } else {
                pos = pos.saturating_add(1);
            }
        } else {
            break;
        }
    }

    out
}

fn match_tag_start(bytes: &[u8], pos: usize, name: &[u8]) -> bool {
    // Expect `<` at pos, then optional whitespace/slash, then `name`
    let mut idx = pos.saturating_add(1);
    while bytes.get(idx) == Some(&b' ') || bytes.get(idx) == Some(&b'\t') {
        idx = idx.saturating_add(1);
    }
    for &b in name {
        if let Some(&c) = bytes.get(idx) {
            if c.to_ascii_lowercase() != b {
                return false;
            }
        } else {
            return false;
        }
        idx = idx.saturating_add(1);
    }
    matches!(
        bytes.get(idx),
        Some(b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/') | None
    )
}

fn match_block_tag(bytes: &[u8], pos: usize) -> bool {
    const BLOCK_TAGS: &[&[u8]] = &[
        b"p",
        b"div",
        b"br",
        b"h1",
        b"h2",
        b"h3",
        b"h4",
        b"h5",
        b"h6",
        b"li",
        b"tr",
        b"blockquote",
    ];
    for &tag in BLOCK_TAGS {
        if match_tag_start(bytes, pos, tag) {
            return true;
        }
    }
    false
}

fn skip_tag(bytes: &[u8], start: usize) -> usize {
    let mut pos = start.saturating_add(1);
    while pos < bytes.len() {
        if bytes.get(pos) == Some(&b'>') {
            return pos.saturating_add(1);
        }
        pos = pos.saturating_add(1);
    }
    bytes.len()
}

fn skip_until_end_tag(bytes: &[u8], start: usize, tag: &[u8]) -> usize {
    let mut pos = start.saturating_add(1);
    while pos < bytes.len() {
        if bytes.get(pos) == Some(&b'<') {
            let mut idx = pos.saturating_add(1);
            if bytes.get(idx) == Some(&b'/') {
                idx = idx.saturating_add(1);
                while bytes.get(idx) == Some(&b' ') || bytes.get(idx) == Some(&b'\t') {
                    idx = idx.saturating_add(1);
                }
                let mut matches = true;
                for &b in tag {
                    if let Some(&c) = bytes.get(idx) {
                        if c.to_ascii_lowercase() != b {
                            matches = false;
                            break;
                        }
                    } else {
                        matches = false;
                        break;
                    }
                    idx = idx.saturating_add(1);
                }
                if matches {
                    return skip_tag(bytes, pos);
                }
            }
        }
        pos = pos.saturating_add(1);
    }
    bytes.len()
}

fn decode_entity(html: &str, pos: usize) -> (&'static str, usize) {
    let bytes = html.as_bytes();
    let mut end = pos.saturating_add(1);
    let max_len = pos.saturating_add(10).min(bytes.len());

    while end < max_len {
        match bytes.get(end) {
            Some(&b';') => {
                let entity_slice = html.get(pos..end.saturating_add(1)).unwrap_or("");
                let decoded = match entity_slice {
                    "&lt;" => "<",
                    "&gt;" => ">",
                    "&amp;" => "&",
                    "&quot;" => "\"",
                    "&#39;" | "&apos;" => "'",
                    "&nbsp;" => " ",
                    _ => "",
                };
                if !decoded.is_empty() {
                    return (decoded, end.saturating_add(1));
                }
                break;
            }
            Some(&b) if b.is_ascii_alphanumeric() || b == b'#' => {
                end = end.saturating_add(1);
            }
            _ => break,
        }
    }

    ("&", pos.saturating_add(1))
}

fn utf8_char_len(b: u8) -> usize {
    if b & 0b1000_0000 == 0 {
        1
    } else if b & 0b1110_0000 == 0b1100_0000 {
        2
    } else if b & 0b1111_0000 == 0b1110_0000 {
        3
    } else if b & 0b1111_1000 == 0b1111_0000 {
        4
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_basic() {
        let input = "<html><body><h1>Title</h1><p>Hello &amp; World!</p></body></html>";
        let text = strip_html(input);
        assert!(text.contains("Title"));
        assert!(text.contains("Hello & World!"));
    }

    #[test]
    fn test_strip_html_script_and_style_discarded() {
        let input = "<p>Before</p><script>alert('prompt injection')</script><style>body { color: red; }</style><p>After</p>";
        let text = strip_html(input);
        assert!(text.contains("Before"));
        assert!(text.contains("After"));
        assert!(!text.contains("alert"));
        assert!(!text.contains("prompt injection"));
        assert!(!text.contains("color: red"));
    }
}
