//! Markdown sanitization and escaping utilities.
//!
//! Provides escaping for arbitrary untrusted string inputs prior to embedding
//! in generated Markdown documents and tables.

/// Escapes arbitrary string inputs so that they can be safely embedded in Markdown
/// documents without altering document structure or inserting unintended Markdown syntax
/// such as headings, links, list items, or table rows/cells.
pub fn escape_markdown(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + input.len() / 2);
    let mut chars = input.chars().peekable();
    let mut line_start = true;

    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push_str("<br/>");
                line_start = true;
            }
            '\n' => {
                out.push_str("<br/>");
                line_start = true;
            }
            '|' => {
                out.push_str("\\|");
                line_start = false;
            }
            '\\' => {
                out.push_str("\\\\");
                line_start = false;
            }
            '`' => {
                out.push_str("\\`");
                line_start = false;
            }
            '*' => {
                out.push_str("\\*");
                line_start = false;
            }
            '_' => {
                out.push_str("\\_");
                line_start = false;
            }
            '~' => {
                out.push_str("\\~");
                line_start = false;
            }
            '#' => {
                if line_start {
                    out.push_str("\\#");
                } else {
                    out.push('#');
                }
                line_start = false;
            }
            '[' => {
                out.push_str("\\[");
                line_start = false;
            }
            ']' => {
                out.push_str("\\]");
                line_start = false;
            }
            '(' => {
                out.push_str("\\(");
                line_start = false;
            }
            ')' => {
                out.push_str("\\)");
                line_start = false;
            }
            '<' => {
                out.push_str("&lt;");
                line_start = false;
            }
            '>' => {
                out.push_str("&gt;");
                line_start = false;
            }
            '&' => {
                out.push_str("&amp;");
                line_start = false;
            }
            '-' => {
                if line_start && matches!(chars.peek(), Some(' ') | None) {
                    out.push_str("\\-");
                } else {
                    out.push('-');
                }
                line_start = false;
            }
            '+' => {
                if line_start && matches!(chars.peek(), Some(' ') | None) {
                    out.push_str("\\+");
                } else {
                    out.push('+');
                }
                line_start = false;
            }
            '0'..='9' if line_start => {
                out.push(c);
                line_start = false;

                let mut digits = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_digit() {
                        if let Some(digit) = chars.next() {
                            digits.push(digit);
                        }
                    } else {
                        break;
                    }
                }
                if let Some(&delim) = chars.peek() {
                    if delim == '.' || delim == ')' {
                        out.push_str(&digits);
                        out.push('\\');
                        if let Some(d) = chars.next() {
                            out.push(d); // consume '.' or ')'
                        }
                    } else {
                        out.push_str(&digits);
                    }
                } else {
                    out.push_str(&digits);
                }
            }
            ' ' if line_start => {
                out.push(' ');
            }
            _ => {
                line_start = false;
                out.push(c);
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_escape_markdown_basic() {
        assert_eq!(escape_markdown("Hello World"), "Hello World");
        assert_eq!(escape_markdown("# Heading"), "\\# Heading");
        assert_eq!(escape_markdown("[link](http://x)"), "\\[link\\]\\(http://x\\)");
        assert_eq!(escape_markdown("col1 | col2"), "col1 \\| col2");
        assert_eq!(escape_markdown("line1\nline2"), "line1<br/>line2");
        assert_eq!(escape_markdown("- item"), "\\- item");
        assert_eq!(escape_markdown("1. item"), "1\\. item");
        assert_eq!(escape_markdown("10) item"), "10\\) item");
        assert_eq!(escape_markdown("HMAC-SHA256 (Ed25519)"), "HMAC-SHA256 \\(Ed25519\\)");
        assert_eq!(escape_markdown("`<script>` & 'bold' *"), "\\`&lt;script&gt;\\` &amp; 'bold' \\*");
    }

    #[test]
    fn test_escape_markdown_headers_and_leading_spaces() {
        assert_eq!(escape_markdown("   # Heading"), "   \\# Heading");
        assert_eq!(escape_markdown("#Heading"), "\\#Heading");
        assert_eq!(escape_markdown("## Subheading"), "\\## Subheading");
    }

    #[test]
    fn test_escape_markdown_links() {
        assert_eq!(
            escape_markdown("[evil_link](http://hostile.test)"),
            "\\[evil\\_link\\]\\(http://hostile.test\\)"
        );
    }

    #[test]
    fn test_escape_markdown_lists() {
        assert_eq!(escape_markdown("- item"), "\\- item");
        assert_eq!(escape_markdown("* item"), "\\* item");
        assert_eq!(escape_markdown("+ item"), "\\+ item");
        assert_eq!(escape_markdown("1. item"), "1\\. item");
        assert_eq!(escape_markdown("10) item"), "10\\) item");
        assert_eq!(escape_markdown("a - b"), "a - b");
        assert_eq!(escape_markdown("a + b"), "a + b");
    }

    #[test]
    fn test_escape_markdown_table_pipes_and_backticks() {
        assert_eq!(escape_markdown("col1 | col2"), "col1 \\| col2");
        assert_eq!(escape_markdown("`code`"), "\\`code\\`");
    }

    #[test]
    fn test_escape_markdown_line_breaks() {
        assert_eq!(escape_markdown("line1\nline2"), "line1<br/>line2");
        assert_eq!(escape_markdown("line1\r\nline2"), "line1<br/>line2");
        assert_eq!(escape_markdown("line1\rline2"), "line1<br/>line2");
    }

    proptest! {
        #[test]
        fn prop_escape_markdown_no_extra_lines_or_headings_or_tables(input in ".*") {
            let escaped = escape_markdown(&input);

            // 1. Escaped output must never contain raw newlines or CRs
            prop_assert!(!escaped.contains('\n'));
            prop_assert!(!escaped.contains('\r'));

            // 2. Embedded in a single template line: "PREFIX " + escaped + " SUFFIX"
            let template_line = format!("PREFIX {escaped} SUFFIX");

            // Line count when split by real newlines must be exactly 1
            prop_assert_eq!(template_line.lines().count(), 1);

            // 3. Must not create a Markdown heading (line starting with `#` or ` #`)
            let trimmed = template_line.trim_start();
            prop_assert!(!trimmed.starts_with('#'));

            // 4. Embedded in a table cell: "| " + escaped + " |"
            let table_cell_line = format!("| {escaped} |");

            // Table cell line must not contain unescaped pipes other than the boundary pipes
            let inner = &table_cell_line[1..table_cell_line.len() - 1];
            let raw_pipes_in_inner = inner.chars().zip(inner.chars().skip(1))
                .filter(|&(c1, c2)| c2 == '|' && c1 != '\\')
                .count();
            let first_is_pipe = inner.starts_with('|');
            prop_assert_eq!(raw_pipes_in_inner + if first_is_pipe { 1 } else { 0 }, 0);

            // 5. Must not start as a Markdown list item if placed at line start
            let list_line = format!("{escaped} ITEM");
            let list_trimmed = list_line.trim_start();
            prop_assert!(!list_trimmed.starts_with("- "));
            prop_assert!(!list_trimmed.starts_with("* "));
            prop_assert!(!list_trimmed.starts_with("+ "));
            if let Some(dot_pos) = list_trimmed.find(". ") {
                let prefix = &list_trimmed[..dot_pos];
                prop_assert!(!prefix.chars().all(|c| c.is_ascii_digit()) || prefix.is_empty());
            }
        }
    }
}
