/// Escapes XML special characters in string inputs to prevent tag injection.
pub fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_escape_basic() {
        assert_eq!(xml_escape("<foo & bar>"), "&lt;foo &amp; bar&gt;");
    }
}
