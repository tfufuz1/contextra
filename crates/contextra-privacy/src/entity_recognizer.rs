// FILE-CONTEXT
// ZWECK: Handgeschriebener, regex-freier RuleBasedEntityRecognizer für PII (E-Mail, IBAN, Phone, CC, Steuer-ID, IP, PLZ, Kennzeichen, Gazetteer).
// INVARIANTEN: Zero Panic, no unsafe code, strict fail-closed, byte-boundary safe character mapping on original text.

use crate::egress_vault::EntityRecognizer;
use std::collections::BTreeSet;
use std::ops::Range;

/// Extension point for custom term, name, or company recognition lists.
#[derive(Debug, Clone, Default)]
pub struct Gazetteer {
    terms: BTreeSet<String>,
}

impl Gazetteer {
    /// Creates a new empty `Gazetteer`.
    pub fn new() -> Self {
        Self {
            terms: BTreeSet::new(),
        }
    }

    /// Adds a term to the gazetteer (stored in lowercase for case-insensitive matching).
    pub fn add_term(&mut self, term: impl Into<String>) {
        let t = term.into().trim().to_lowercase();
        if !t.is_empty() {
            self.terms.insert(t);
        }
    }

    /// Returns `true` if the gazetteer contains no terms.
    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

/// A rule-based entity recognizer without regex dependencies.
///
/// Detects PII categories: EMAIL, IBAN, PHONE, CREDIT_CARD, STEUER_ID, IPV4, IPV6, PLZ, LICENSE_PLATE, GAZETTEER.
#[derive(Debug, Clone, Default)]
pub struct RuleBasedEntityRecognizer {
    gazetteer: Gazetteer,
}

impl RuleBasedEntityRecognizer {
    /// Creates a new `RuleBasedEntityRecognizer` with no gazetteer terms.
    pub fn new() -> Self {
        Self {
            gazetteer: Gazetteer::new(),
        }
    }

    /// Creates a new `RuleBasedEntityRecognizer` with the specified gazetteer.
    pub fn with_gazetteer(gazetteer: Gazetteer) -> Self {
        Self { gazetteer }
    }

    /// Builder method to add a gazetteer term.
    pub fn add_gazetteer_term(mut self, term: impl Into<String>) -> Self {
        self.gazetteer.add_term(term);
        self
    }
}

impl EntityRecognizer for RuleBasedEntityRecognizer {
    fn recognize(&self, text: &str) -> Vec<(Range<usize>, String)> {
        if text.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        // Step 1: Build normalized view with back-mapping to raw text byte offsets
        let norm = Normalizer::normalize(text);

        // Step 2: Run detectors on normalized view and map spans back to raw byte ranges
        detect_emails(&norm, &mut results);
        detect_ibans(&norm, &mut results);
        detect_credit_cards(&norm, &mut results);
        detect_steuer_ids(&norm, &mut results);
        detect_phone_numbers(&norm, &mut results);
        detect_ip_addresses(&norm, &mut results);
        detect_plz_and_license_plates(&norm, &mut results);
        detect_gazetteer(&norm, &self.gazetteer, &mut results);

        // Step 3: Validate and deduplicate byte ranges
        let mut valid_results = Vec::with_capacity(results.len());
        for (range, label) in results {
            if range.start < range.end
                && range.end <= text.len()
                && text.is_char_boundary(range.start)
                && text.is_char_boundary(range.end)
            {
                valid_results.push((range, label));
            }
        }

        // Deduplicate overlapping ranges preferring longer / earlier spans
        deduplicate_spans(valid_results)
    }
}

/// Internal helper for normalizing text and tracking raw byte offsets.
struct Normalizer {
    /// Cleaned normalized characters
    chars: Vec<char>,
    /// Maps character index in `chars` to start byte offset in raw text
    char_start_bytes: Vec<usize>,
    /// Maps character index in `chars` to end byte offset in raw text
    char_end_bytes: Vec<usize>,
}

impl Normalizer {
    fn normalize(text: &str) -> Self {
        let mut chars = Vec::with_capacity(text.len());
        let mut char_start_bytes = Vec::with_capacity(text.len());
        let mut char_end_bytes = Vec::with_capacity(text.len());

        for (byte_offset, c) in text.char_indices() {
            let char_len = c.len_utf8();
            let end_byte = byte_offset + char_len;

            // Strip zero-width & non-printing control characters
            if is_zero_width_or_control(c) {
                continue;
            }

            // Homoglyph & unicode folding to ASCII where appropriate
            let folded = fold_homoglyph(c);

            chars.push(folded);
            char_start_bytes.push(byte_offset);
            char_end_bytes.push(end_byte);
        }

        Self {
            chars,
            char_start_bytes,
            char_end_bytes,
        }
    }

    fn len(&self) -> usize {
        self.chars.len()
    }

    fn get_char(&self, idx: usize) -> Option<char> {
        self.chars.get(idx).copied()
    }

    /// Converts a character span `[start_char..end_char]` in normalized view into a raw byte range `[start_byte..end_byte]`.
    fn char_span_to_byte_range(&self, start_char: usize, end_char: usize) -> Option<Range<usize>> {
        if start_char >= end_char || end_char > self.len() {
            return None;
        }
        let start_byte = *self.char_start_bytes.get(start_char)?;
        let end_byte = *self.char_end_bytes.get(end_char.saturating_sub(1))?;
        Some(start_byte..end_byte)
    }

    fn as_string(&self) -> String {
        self.chars.iter().collect()
    }
}

fn is_zero_width_or_control(c: char) -> bool {
    matches!(
        c,
        '\u{200B}' // Zero Width Space
        | '\u{200C}' // Zero Width Non-Joiner
        | '\u{200D}' // Zero Width Joiner
        | '\u{FEFF}' // Zero Width No-Break Space / BOM
        | '\u{00AD}' // Soft Hyphen
        | '\u{200E}' // Left-to-Right Mark
        | '\u{200F}' // Right-to-Left Mark
    )
}

fn fold_homoglyph(c: char) -> char {
    match c {
        // Cyrillic lookalikes
        'а' | 'А' => 'a',
        'в' | 'В' => 'B',
        'е' | 'Е' => 'e',
        'к' | 'К' => 'K',
        'м' | 'М' => 'M',
        'о' | 'О' => 'o',
        'р' | 'Р' => 'p',
        'с' | 'С' => 'c',
        'т' | 'Т' => 'T',
        'х' | 'Х' => 'x',
        // Fullwidth digits
        '０'..='９' => {
            let offset = c as u32 - '０' as u32;
            let ascii_code = '0' as u32 + offset;
            char::from_u32(ascii_code).unwrap_or(c)
        }
        _ => c,
    }
}

// ============================================================================
// Scanners
// ============================================================================

/// Detects email addresses.
fn detect_emails(norm: &Normalizer, results: &mut Vec<(Range<usize>, String)>) {
    let len = norm.len();
    let mut idx = 0;

    while idx < len {
        if norm.get_char(idx) == Some('@') {
            let at_idx = idx;

            // Find local part start
            let mut local_start = at_idx;
            while local_start > 0 {
                if let Some(prev_c) = norm.get_char(local_start.saturating_sub(1)) {
                    if is_email_local_char(prev_c) {
                        local_start -= 1;
                        continue;
                    }
                }
                break;
            }

            // Find domain end
            let mut domain_end = at_idx + 1;
            let mut has_dot = false;

            while domain_end < len {
                if let Some(next_c) = norm.get_char(domain_end) {
                    if next_c == '.' {
                        has_dot = true;
                        domain_end += 1;
                    } else if is_email_domain_char(next_c) {
                        domain_end += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            // Trim trailing dot if any
            if domain_end > at_idx + 1 && norm.get_char(domain_end.saturating_sub(1)) == Some('.') {
                domain_end -= 1;
            }

            let local_len = at_idx - local_start;
            let domain_len = domain_end - (at_idx + 1);

            if local_len >= 1 && domain_len >= 3 && has_dot {
                // Ensure TLD length >= 2
                let domain_chars: Vec<char> = (at_idx + 1..domain_end)
                    .filter_map(|i| norm.get_char(i))
                    .collect();
                if let Some(dot_pos) = domain_chars.iter().rposition(|&c| c == '.') {
                    let tld_len = domain_len - dot_pos - 1;
                    if tld_len >= 2 {
                        if let Some(byte_range) =
                            norm.char_span_to_byte_range(local_start, domain_end)
                        {
                            results.push((byte_range, "EMAIL".to_string()));
                        }
                    }
                }
            }
        }
        idx += 1;
    }
}

fn is_email_local_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '%' | '+' | '-')
}

fn is_email_domain_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-'
}

/// Detects IBANs (country length check + Mod-97 checksum, optional spaces/hyphens).
fn detect_ibans(norm: &Normalizer, results: &mut Vec<(Range<usize>, String)>) {
    let len = norm.len();
    let mut i = 0;

    while i < len {
        let prev_ok = i == 0
            || match norm.get_char(i.saturating_sub(1)) {
                Some(c) => !c.is_ascii_alphanumeric(),
                None => true,
            };
        let c1 = norm.get_char(i);
        let c2 = norm.get_char(i + 1);

        let is_uppercase_prefix = match (c1, c2) {
            (Some(a), Some(b)) => a.is_ascii_uppercase() && b.is_ascii_uppercase(),
            _ => false,
        };

        if prev_ok && is_uppercase_prefix {
            let country_code: String = [c1, c2].iter().filter_map(|&c| c).collect();
            if let Some(expected_len) = get_iban_country_length(&country_code) {
                let mut char_count = 0;
                let mut scan_idx = i;
                let mut last_alnum_idx = i;
                let mut raw_iban_chars = Vec::with_capacity(expected_len);

                while scan_idx < len && char_count < expected_len {
                    if let Some(c) = norm.get_char(scan_idx) {
                        if c.is_ascii_alphanumeric() {
                            raw_iban_chars.push(c);
                            char_count += 1;
                            last_alnum_idx = scan_idx;
                            scan_idx += 1;
                        } else if matches!(c, ' ' | '-' | '_') {
                            scan_idx += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if char_count == expected_len {
                    let iban_str: String = raw_iban_chars.iter().collect();
                    if validate_iban_checksum(&iban_str) {
                        let end_char_idx = last_alnum_idx + 1;
                        if let Some(byte_range) = norm.char_span_to_byte_range(i, end_char_idx) {
                            results.push((byte_range, "IBAN".to_string()));
                            i = end_char_idx;
                            continue;
                        }
                    }
                }
            }
        }
        i += 1;
    }
}

fn get_iban_country_length(country: &str) -> Option<usize> {
    match country {
        "DE" => Some(22),
        "AT" => Some(20),
        "CH" => Some(21),
        "FR" => Some(27),
        "NL" => Some(18),
        "GB" => Some(22),
        "IT" => Some(27),
        "ES" => Some(24),
        "BE" => Some(16),
        "PL" => Some(28),
        "CZ" => Some(24),
        "DK" => Some(18),
        "FI" => Some(18),
        "SE" => Some(24),
        "NO" => Some(15),
        "LU" => Some(20),
        _ => None,
    }
}

fn validate_iban_checksum(iban: &str) -> bool {
    if iban.len() < 15 || iban.len() > 34 {
        return false;
    }

    let head = iban.get(..4);
    let tail = iban.get(4..);
    let (head_str, tail_str) = match (head, tail) {
        (Some(h), Some(t)) => (h, t),
        _ => return false,
    };

    let rearranged = format!("{tail_str}{head_str}");

    let mut remainder = 0u64;
    for c in rearranged.chars() {
        if c.is_ascii_digit() {
            let digit = (c as u8 - b'0') as u64;
            remainder = (remainder * 10 + digit) % 97;
        } else if c.is_ascii_uppercase() {
            let val = (c as u8 - b'A' + 10) as u64;
            let tens = val / 10;
            let ones = val % 10;
            remainder = (remainder * 10 + tens) % 97;
            remainder = (remainder * 10 + ones) % 97;
        } else {
            return false;
        }
    }

    remainder == 1
}

/// Detects credit card numbers (13-19 digits, optional spaces/hyphens, Luhn algorithm).
fn detect_credit_cards(norm: &Normalizer, results: &mut Vec<(Range<usize>, String)>) {
    let len = norm.len();
    let mut i = 0;

    while i < len {
        let prev_ok = i == 0
            || match norm.get_char(i.saturating_sub(1)) {
                Some(c) => !c.is_ascii_alphanumeric(),
                None => true,
            };
        let is_digit = norm.get_char(i).is_some_and(|c| c.is_ascii_digit());

        if prev_ok && is_digit {
            let mut digits = Vec::with_capacity(19);
            let mut scan_idx = i;
            let mut last_digit_idx = i;

            while scan_idx < len {
                if let Some(c) = norm.get_char(scan_idx) {
                    if c.is_ascii_digit() {
                        digits.push(c as u8 - b'0');
                        last_digit_idx = scan_idx;
                        scan_idx += 1;
                    } else if matches!(c, ' ' | '-') {
                        scan_idx += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let num_digits = digits.len();
            if (13..=19).contains(&num_digits) && validate_luhn_checksum(&digits) {
                let end_char_idx = last_digit_idx + 1;
                if let Some(byte_range) = norm.char_span_to_byte_range(i, end_char_idx) {
                    results.push((byte_range, "CREDIT_CARD".to_string()));
                    i = end_char_idx;
                    continue;
                }
            }
        }
        i += 1;
    }
}

fn validate_luhn_checksum(digits: &[u8]) -> bool {
    let mut sum = 0u32;
    let mut alternate = false;

    for &digit in digits.iter().rev() {
        let mut d = digit as u32;
        if alternate {
            d *= 2;
            if d > 9 {
                d -= 9;
            }
        }
        sum += d;
        alternate = !alternate;
    }

    sum % 10 == 0
}

/// Detects German Steuer-ID (11 digits, TIN Mod 11 checksum).
fn detect_steuer_ids(norm: &Normalizer, results: &mut Vec<(Range<usize>, String)>) {
    let len = norm.len();
    let mut i = 0;

    while i < len {
        let prev_ok = i == 0
            || match norm.get_char(i.saturating_sub(1)) {
                Some(c) => !c.is_ascii_alphanumeric(),
                None => true,
            };
        let is_valid_start_digit = norm
            .get_char(i)
            .is_some_and(|c| c.is_ascii_digit() && c != '0');

        if prev_ok && is_valid_start_digit {
            let mut digits = Vec::with_capacity(11);
            let mut scan_idx = i;
            let mut last_digit_idx = i;

            while scan_idx < len && digits.len() < 11 {
                if let Some(c) = norm.get_char(scan_idx) {
                    if c.is_ascii_digit() {
                        digits.push((c as u8 - b'0') as u64);
                        last_digit_idx = scan_idx;
                        scan_idx += 1;
                    } else if matches!(c, ' ' | '/') {
                        scan_idx += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if digits.len() == 11 {
                let end_char_idx = last_digit_idx + 1;
                let next_ok = match norm.get_char(end_char_idx) {
                    Some(c) => !c.is_ascii_alphanumeric(),
                    None => true,
                };

                if next_ok && validate_german_steuer_id(&digits) {
                    if let Some(byte_range) = norm.char_span_to_byte_range(i, end_char_idx) {
                        results.push((byte_range, "STEUER_ID".to_string()));
                        i = end_char_idx;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
}

fn validate_german_steuer_id(digits: &[u64]) -> bool {
    if digits.len() != 11 {
        return false;
    }
    if digits.first() == Some(&0) {
        return false;
    }

    // First 10 digits must contain either one digit 2 or 3 times, others 1 time
    let mut counts = [0u8; 10];
    if let Some(first10) = digits.get(..10) {
        for &d in first10 {
            if d > 9 {
                return false;
            }
            if let Ok(idx) = usize::try_from(d) {
                if let Some(slot) = counts.get_mut(idx) {
                    *slot += 1;
                }
            }
        }
    } else {
        return false;
    }

    let count_twos = counts.iter().filter(|&&c| c == 2).count();
    let count_threes = counts.iter().filter(|&&c| c == 3).count();

    let valid_distribution =
        (count_twos == 1 && count_threes == 0) || (count_twos == 0 && count_threes == 1);
    if !valid_distribution {
        return false;
    }

    // ISO 7064 Mod 11, 10 checksum over first 10 digits
    let mut remainder = 10u64;
    if let Some(first10) = digits.get(..10) {
        for &d in first10 {
            let sum = (d + remainder) % 10;
            let s = if sum == 0 { 10 } else { sum };
            remainder = (s * 2) % 11;
        }
    }

    let mut check_digit = (11 - remainder) % 11;
    if check_digit == 10 {
        check_digit = 0;
    }

    digits.get(10) == Some(&check_digit)
}

/// Detects DE/EU phone numbers (+49..., 0049..., 0170...).
fn detect_phone_numbers(norm: &Normalizer, results: &mut Vec<(Range<usize>, String)>) {
    let len = norm.len();
    let mut i = 0;

    while i < len {
        let prev_ok = i == 0
            || match norm.get_char(i.saturating_sub(1)) {
                Some(c) => !c.is_ascii_alphanumeric(),
                None => true,
            };
        let c = norm.get_char(i);
        let next_c = norm.get_char(i + 1);

        let is_phone_start = match (c, next_c) {
            (Some('+'), _) => true,
            (Some('('), Some('0')) => true,
            (Some('0'), Some('0')) => true,
            (Some('0'), Some(nc)) => nc.is_ascii_digit(),
            _ => false,
        };

        if prev_ok && is_phone_start {
            let mut digit_count = 0;
            let mut scan_idx = i;
            let mut last_digit_idx = i;

            while scan_idx < len {
                if let Some(curr) = norm.get_char(scan_idx) {
                    if curr.is_ascii_digit() {
                        digit_count += 1;
                        last_digit_idx = scan_idx;
                        scan_idx += 1;
                    } else if matches!(curr, '+' | ' ' | '-' | '/' | '(' | ')') {
                        scan_idx += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let end_char_idx = last_digit_idx + 1;
            let next_ok = match norm.get_char(end_char_idx) {
                Some(c) => !c.is_ascii_alphanumeric(),
                None => true,
            };

            if next_ok && (7..=15).contains(&digit_count) {
                if let Some(byte_range) = norm.char_span_to_byte_range(i, end_char_idx) {
                    results.push((byte_range, "PHONE".to_string()));
                    i = end_char_idx;
                    continue;
                }
            }
        }
        i += 1;
    }
}

/// Detects IPv4 and IPv6 addresses.
fn detect_ip_addresses(norm: &Normalizer, results: &mut Vec<(Range<usize>, String)>) {
    let len = norm.len();
    let mut i = 0;

    while i < len {
        let prev_ok = i == 0
            || match norm.get_char(i.saturating_sub(1)) {
                Some(c) => !c.is_ascii_alphanumeric(),
                None => true,
            };
        let curr_char = norm.get_char(i);

        let is_hex_or_digit =
            curr_char.is_some_and(|c| c.is_ascii_digit() || c.is_ascii_hexdigit());

        if prev_ok && is_hex_or_digit {
            // Check for IPv4
            if curr_char.is_some_and(|c| c.is_ascii_digit()) {
                let mut scan_idx = i;
                let mut octet_count = 0;
                let mut valid_ipv4 = true;

                while scan_idx < len && octet_count < 4 {
                    let mut octet_val = 0u32;
                    let mut digit_len = 0;

                    while scan_idx < len && digit_len < 3 {
                        if let Some(d_char) = norm.get_char(scan_idx) {
                            if d_char.is_ascii_digit() {
                                let d = (d_char as u8 - b'0') as u32;
                                octet_val = octet_val * 10 + d;
                                digit_len += 1;
                                scan_idx += 1;
                                continue;
                            }
                        }
                        break;
                    }

                    if digit_len == 0 || octet_val > 255 {
                        valid_ipv4 = false;
                        break;
                    }

                    octet_count += 1;
                    if octet_count < 4 {
                        if norm.get_char(scan_idx) == Some('.') {
                            scan_idx += 1;
                        } else {
                            valid_ipv4 = false;
                            break;
                        }
                    }
                }

                if valid_ipv4 && octet_count == 4 {
                    let next_ok = match norm.get_char(scan_idx) {
                        Some(c) => !c.is_ascii_alphanumeric(),
                        None => true,
                    };

                    if next_ok {
                        if let Some(byte_range) = norm.char_span_to_byte_range(i, scan_idx) {
                            results.push((byte_range, "IPV4".to_string()));
                            i = scan_idx;
                            continue;
                        }
                    }
                }
            }

            // Check for IPv6 (e.g., 2001:db8::1 or fe80::1)
            let mut scan_idx = i;
            let mut colons = 0;
            let mut hex_group_len = 0;

            while scan_idx < len {
                if let Some(curr) = norm.get_char(scan_idx) {
                    if curr.is_ascii_hexdigit() {
                        hex_group_len += 1;
                        if hex_group_len > 4 {
                            break;
                        }
                        scan_idx += 1;
                    } else if curr == ':' {
                        colons += 1;
                        hex_group_len = 0;
                        scan_idx += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if (2..=7).contains(&colons) {
                let next_ok = match norm.get_char(scan_idx) {
                    Some(c) => !c.is_ascii_alphanumeric(),
                    None => true,
                };

                if next_ok {
                    if let Some(byte_range) = norm.char_span_to_byte_range(i, scan_idx) {
                        results.push((byte_range, "IPV6".to_string()));
                        i = scan_idx;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
}

/// Detects German PLZ (5-digit) and German license plates (e.g. B-MW 1234).
fn detect_plz_and_license_plates(norm: &Normalizer, results: &mut Vec<(Range<usize>, String)>) {
    let len = norm.len();
    let mut i = 0;

    while i < len {
        let prev_ok = i == 0
            || match norm.get_char(i.saturating_sub(1)) {
                Some(c) => !c.is_ascii_alphanumeric(),
                None => true,
            };
        if prev_ok {
            // PLZ: 5 consecutive digits
            let is_digit = norm.get_char(i).is_some_and(|c| c.is_ascii_digit());
            if is_digit {
                let mut scan_idx = i;
                while scan_idx < len && norm.get_char(scan_idx).is_some_and(|c| c.is_ascii_digit())
                {
                    scan_idx += 1;
                }

                if scan_idx - i == 5 {
                    let next_ok = match norm.get_char(scan_idx) {
                        Some(c) => !c.is_ascii_alphanumeric(),
                        None => true,
                    };

                    if next_ok {
                        if let Some(byte_range) = norm.char_span_to_byte_range(i, scan_idx) {
                            results.push((byte_range, "PLZ".to_string()));
                            i = scan_idx;
                            continue;
                        }
                    }
                }
            }

            // License Plate: 1-3 uppercase letters, space/hyphen, 1-2 letters, space/hyphen, 1-4 digits
            let is_uppercase = norm.get_char(i).is_some_and(|c| c.is_ascii_uppercase());
            if is_uppercase {
                let mut scan_idx = i;
                let mut city_len = 0;

                while scan_idx < len
                    && norm
                        .get_char(scan_idx)
                        .is_some_and(|c| c.is_ascii_uppercase())
                    && city_len < 3
                {
                    city_len += 1;
                    scan_idx += 1;
                }

                let is_sep1 = norm
                    .get_char(scan_idx)
                    .is_some_and(|c| matches!(c, ' ' | '-'));
                if (1..=3).contains(&city_len) && scan_idx < len && is_sep1 {
                    scan_idx += 1;
                    let mut mid_len = 0;
                    while scan_idx < len
                        && norm
                            .get_char(scan_idx)
                            .is_some_and(|c| c.is_ascii_uppercase())
                        && mid_len < 2
                    {
                        mid_len += 1;
                        scan_idx += 1;
                    }

                    let is_sep2 = norm
                        .get_char(scan_idx)
                        .is_some_and(|c| matches!(c, ' ' | '-'));
                    if (1..=2).contains(&mid_len) && scan_idx < len && is_sep2 {
                        scan_idx += 1;
                        let mut num_len = 0;
                        while scan_idx < len
                            && norm.get_char(scan_idx).is_some_and(|c| c.is_ascii_digit())
                            && num_len < 4
                        {
                            num_len += 1;
                            scan_idx += 1;
                        }

                        if (1..=4).contains(&num_len) {
                            let next_ok = match norm.get_char(scan_idx) {
                                Some(c) => !c.is_ascii_alphanumeric(),
                                None => true,
                            };

                            if next_ok {
                                if let Some(byte_range) = norm.char_span_to_byte_range(i, scan_idx)
                                {
                                    results.push((byte_range, "LICENSE_PLATE".to_string()));
                                    i = scan_idx;
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }

        i += 1;
    }
}

/// Detects custom gazetteer terms in the text using character index alignment.
fn detect_gazetteer(
    norm: &Normalizer,
    gazetteer: &Gazetteer,
    results: &mut Vec<(Range<usize>, String)>,
) {
    if gazetteer.is_empty() {
        return;
    }

    let norm_str = norm.as_string().to_lowercase();

    for term in &gazetteer.terms {
        let mut start = 0;
        while let Some(match_pos) = norm_str.get(start..).and_then(|s| s.find(term)) {
            let abs_start = start + match_pos;
            let abs_end = abs_start + term.len();

            let prev_ok = abs_start == 0
                || match norm.get_char(abs_start.saturating_sub(1)) {
                    Some(c) => !c.is_alphanumeric(),
                    None => true,
                };
            let next_ok = abs_end >= norm.len()
                || match norm.get_char(abs_end) {
                    Some(c) => !c.is_alphanumeric(),
                    None => true,
                };

            if prev_ok && next_ok {
                if let Some(byte_range) = norm.char_span_to_byte_range(abs_start, abs_end) {
                    results.push((byte_range, "GAZETTEER".to_string()));
                }
            }

            start = abs_start + 1;
        }
    }
}

/// Deduplicates byte ranges preferring longer spans and earlier positions.
fn deduplicate_spans(mut spans: Vec<(Range<usize>, String)>) -> Vec<(Range<usize>, String)> {
    if spans.len() <= 1 {
        return spans;
    }

    // Sort by start position ascending, then by length descending
    spans.sort_by(|a, b| {
        if a.0.start != b.0.start {
            a.0.start.cmp(&b.0.start)
        } else {
            (b.0.end - b.0.start).cmp(&(a.0.end - a.0.start))
        }
    });

    let mut result: Vec<(Range<usize>, String)> = Vec::with_capacity(spans.len());

    for span in spans {
        if let Some(last) = result.last() {
            // Check if current span overlaps with last span
            if span.0.start < last.0.end {
                continue;
            }
        }
        result.push(span);
    }

    result
}
