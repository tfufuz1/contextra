// FILE-CONTEXT
// ZWECK: Hilfsfunktionen für die Pfadformatierung in strukturierten Dokumenten (Json, Xml, Yaml, Toml).
// INVARIANTEN: FILE-CONTEXT Kopf, Zero-Panic, deterministische Pfadsyntax.
// STAND: TS:2026-10-31T00:00:00Z

/// Prüft, ob ein Objektschlüssel in `["..."]` gefasst werden muss.
/// Ein Schlüssel benötigt `["..."]`, wenn er leer ist, mit einer Ziffer beginnt,
/// oder Sonderzeichen (außer ASCII-Alphanumerik, `_`, `-`) enthält.
pub fn needs_quoting(key: &str) -> bool {
    if key.is_empty() {
        return true;
    }
    let key_trimmed = key.strip_prefix('@').unwrap_or(key);
    if key_trimmed.is_empty() {
        return true;
    }
    if key_trimmed.starts_with(|c: char| c.is_ascii_digit()) {
        return true;
    }
    !key_trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Maskiert Sonderzeichen in JSON-Strings für Pfade in `["..."]`.
pub fn escape_json_string(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{0008}' => escaped.push_str("\\b"),
            '\u{000C}' => escaped.push_str("\\f"),
            other => {
                if other.is_ascii_control() {
                    escaped.push_str(&format!("\\u{:04x}", other as u32));
                } else {
                    escaped.push(other);
                }
            }
        }
    }
    escaped
}

/// Hängt einen Objektschlüssel an einen bestehenden Pfad an.
pub fn append_key(parent_path: &str, key: &str) -> String {
    if needs_quoting(key) {
        let escaped = escape_json_string(key);
        if parent_path.is_empty() {
            format!("[\"{}\"]", escaped)
        } else {
            format!("{}[\"{}\"]", parent_path, escaped)
        }
    } else if parent_path.is_empty() {
        key.to_string()
    } else {
        format!("{}.{}", parent_path, key)
    }
}

/// Hängt einen Array-Index `[index]` an einen bestehenden Pfad an.
pub fn append_index(parent_path: &str, index: usize) -> String {
    format!("{}[{}]", parent_path, index)
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

    #[test]
    fn test_path_formatting_simple() {
        assert_eq!(append_key("", "kunden"), "kunden");
        assert_eq!(append_index("kunden", 3), "kunden[3]");
        assert_eq!(append_key("kunden[3]", "adresse"), "kunden[3].adresse");
        assert_eq!(
            append_key("kunden[3].adresse", "plz"),
            "kunden[3].adresse.plz"
        );
    }

    #[test]
    fn test_path_formatting_special_keys() {
        assert_eq!(append_key("", "user.name"), "[\"user.name\"]");
        assert_eq!(
            append_key("kunden[0]", "zip code"),
            "kunden[0][\"zip code\"]"
        );
        assert_eq!(append_key("", "3d_model"), "[\"3d_model\"]");
        assert_eq!(append_key("", "überschrift"), "[\"überschrift\"]");
        assert_eq!(append_key("", "quote\"in\"key"), "[\"quote\\\"in\\\"key\"]");
        assert_eq!(append_key("", ""), "[\"\"]");
    }
}
