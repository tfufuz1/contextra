use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruncationMarker {
    pub truncated: bool,
    pub omitted: usize,
}

pub struct RecallTruncator {
    max_response_tokens: usize,
}

impl RecallTruncator {
    pub fn new(max_response_tokens: usize) -> Self {
        Self {
            max_response_tokens,
        }
    }

    pub fn truncate(&self, text: &str) -> (String, TruncationMarker) {
        let approx_chars_limit = self.max_response_tokens * 4;

        if text.len() <= approx_chars_limit {
            (
                text.to_string(),
                TruncationMarker {
                    truncated: false,
                    omitted: 0,
                },
            )
        } else {
            let truncated_sub = &text[..approx_chars_limit];
            let omitted_chars = text.len() - approx_chars_limit;
            let omitted_tokens = omitted_chars / 4;

            let formatted = format!(
                "{}... [TRUNCATED: max_response_tokens reached]",
                truncated_sub
            );

            (
                formatted,
                TruncationMarker {
                    truncated: true,
                    omitted: omitted_tokens,
                },
            )
        }
    }
}
