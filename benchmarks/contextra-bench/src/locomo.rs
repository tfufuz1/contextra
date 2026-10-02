// FILE-CONTEXT
// STAND: 2026-10-02
// ZWECK: Loader und Evaluation Harness für das LoCoMo Benchmark (SNAP Research / Long Conversational Memory) mit echten Embeddings und Turn-Level Evidence Resolution
// INVARIANTEN: Lose Kopplung über Search-Closure, keine Panic im Produktionscode, aussagekräftige Fehler.

use crate::long_mem_eval::{BoxFuture, ScoredChunk};
use contextra_core::{ContextraError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

/// Canonical LoCoMo Question Categories (1 to 5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocomoQuestionCategory {
    MultiHop = 1,
    Temporal = 2,
    OpenDomain = 3,
    SingleHop = 4,
    Adversarial = 5,
}

impl LocomoQuestionCategory {
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            1 => Some(Self::MultiHop),
            2 => Some(Self::Temporal),
            3 => Some(Self::OpenDomain),
            4 => Some(Self::SingleHop),
            5 => Some(Self::Adversarial),
            _ => None,
        }
    }
}

impl std::fmt::Display for LocomoQuestionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MultiHop => write!(f, "Multi-hop"),
            Self::Temporal => write!(f, "Temporal"),
            Self::OpenDomain => write!(f, "Open-domain"),
            Self::SingleHop => write!(f, "Single-hop"),
            Self::Adversarial => write!(f, "Adversarial"),
        }
    }
}

/// Evaluation test case for LoCoMo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocomoCase {
    pub sample_id: String,
    pub question_id: String,
    pub category: LocomoQuestionCategory,
    pub question: String,
    pub expected_answer: String,
    pub evidence: Vec<String>,
}

/// Report containing evaluation metrics per question category for LoCoMo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocomoReport {
    pub per_category_recall_at_5: HashMap<LocomoQuestionCategory, f64>,
    pub per_category_mrr: HashMap<LocomoQuestionCategory, f64>,
    pub overall_recall_at_5: f64,
    pub overall_mrr: f64,
    pub total_eval_cases: usize,
}

const LOCOMO_URL: &str = "https://github.com/snap-research/locomo";

/// Index mapping `sample_id` -> (`dia_id` -> turn_text).
pub type LocomoConversationIndex = HashMap<String, HashMap<String, String>>;

/// Container holding extracted LoCoMo evaluation cases and the conversation index.
#[derive(Debug, Clone)]
pub struct LocomoDataset {
    pub cases: Vec<LocomoCase>,
    pub conversation_index: LocomoConversationIndex,
}

impl std::ops::Deref for LocomoDataset {
    type Target = [LocomoCase];

    fn deref(&self) -> &Self::Target {
        &self.cases
    }
}

/// Schema documentation for official `locomo10.json` (ACL 2024 / SNAP Research):
/// - `sample_id`: Identifier string for conversation sample (e.g. "conv-1").
/// - `conversation`: Object/Map with keys like `session_1`, `session_2`, ..., `speaker_a`, `speaker_b`.
///   Each session value is an array of dialog turn objects.
/// - A turn object within a session array contains:
///   - `dia_id`: Dialog identifier string (e.g., "D1:1", "D1:3").
///   - `speaker`: Name of speaker (e.g., "Caroline").
///   - `text`: Raw turn conversation utterance text.
/// - `qa`: List of question-answering annotations, each containing:
///   - `question`: Question text string.
///   - `answer`: String, Number, or Array of strings representing expected answer.
///   - `evidence`: List of dialog IDs (e.g., ["D1:3"]) or raw text evidence strings.
///   - `category`: Category integer (1=MultiHop, 2=Temporal, 3=OpenDomain, 4=SingleHop, 5=Adversarial).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawLocomoTurn {
    pub dia_id: Option<String>,
    pub speaker: Option<String>,
    pub text: Option<String>,
}

/// Raw representation of QA item in official `locomo10.json`.
#[derive(Debug, Deserialize)]
struct RawLocomoQaItem {
    question: Option<String>,
    answer: Option<serde_json::Value>,
    adversarial_answer: Option<String>,
    evidence: Option<Vec<String>>,
    category: Option<u8>,
}

/// Raw representation of conversation sample in official `locomo10.json`.
#[derive(Debug, Deserialize)]
struct RawLocomoSample {
    sample_id: Option<String>,
    qa: Option<Vec<RawLocomoQaItem>>,
    conversation: Option<serde_json::Value>,
}

/// Helper function to extract turn entries into `LocomoConversationIndex` from raw turns slice.
fn extract_turns(sample_id: &str, turns: &[RawLocomoTurn], index: &mut LocomoConversationIndex) {
    for turn in turns {
        if let (Some(dia_id), Some(text)) = (&turn.dia_id, &turn.text) {
            if !dia_id.trim().is_empty() && !text.trim().is_empty() {
                index
                    .entry(sample_id.to_string())
                    .or_default()
                    .insert(dia_id.trim().to_string(), text.trim().to_string());
            }
        }
    }
}

/// Loads LoCoMo cases and conversation index from `locomo10.json` or custom dataset file.
/// Returns a helpful `Result::Err` if the file does not exist.
pub fn load_locomo_dataset(path: &Path) -> Result<LocomoDataset> {
    if !path.exists() {
        return Err(ContextraError::NotFound(format!(
            "LoCoMo dataset file not found at {}. Download locomo10.json from official repository: {}",
            path.display(),
            LOCOMO_URL
        )));
    }

    let file = File::open(path)?;

    let reader = BufReader::new(file);

    let samples: Vec<RawLocomoSample> = serde_json::from_reader(reader).map_err(|e| {
        ContextraError::Serialization(format!(
            "Failed to parse LoCoMo dataset JSON from {}: {}",
            path.display(),
            e
        ))
    })?;

    let mut cases = Vec::new();
    let mut conversation_index: LocomoConversationIndex = HashMap::new();

    for sample in samples {
        let sample_id = sample
            .sample_id
            .unwrap_or_else(|| "conv_unknown".to_string());

        if let Some(conv_val) = sample.conversation {
            if let Some(obj) = conv_val.as_object() {
                for (_key, val) in obj {
                    if let Ok(turns) = serde_json::from_value::<Vec<RawLocomoTurn>>(val.clone()) {
                        extract_turns(&sample_id, &turns, &mut conversation_index);
                    }
                }
            } else if let Ok(turns) = serde_json::from_value::<Vec<RawLocomoTurn>>(conv_val) {
                extract_turns(&sample_id, &turns, &mut conversation_index);
            }
        }

        if let Some(qa_list) = sample.qa {
            for (qa_idx, qa) in qa_list.into_iter().enumerate() {
                let category = qa
                    .category
                    .and_then(LocomoQuestionCategory::from_u8)
                    .unwrap_or(LocomoQuestionCategory::SingleHop);

                let question = match qa.question {
                    Some(q) if !q.trim().is_empty() => q,
                    _ => continue,
                };

                let expected_answer = match qa.answer {
                    Some(serde_json::Value::String(s)) => s,
                    Some(serde_json::Value::Number(n)) => n.to_string(),
                    Some(serde_json::Value::Array(arr)) => arr
                        .into_iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                        .join(" "),
                    _ => qa.adversarial_answer.unwrap_or_default(),
                };

                let evidence = qa.evidence.unwrap_or_default();
                let qid = format!("{}_qa_{}", sample_id, qa_idx + 1);

                cases.push(LocomoCase {
                    sample_id: sample_id.clone(),
                    question_id: qid,
                    category,
                    question,
                    expected_answer,
                    evidence,
                });
            }
        }
    }

    if cases.is_empty() {
        return Err(ContextraError::InvalidInput(format!(
            "No valid LoCoMo cases extracted from {}. Verify format against official release at {}",
            path.display(),
            LOCOMO_URL
        )));
    }

    Ok(LocomoDataset {
        cases,
        conversation_index,
    })
}

/// Runs LoCoMo benchmark evaluation over dataset cases using an injected search function.
/// Evidence entries (e.g. dialog IDs such as `"D1:3"`) are resolved against `conversation_index`
/// before performing substring evaluation against retrieved candidate chunks.
pub async fn run_locomo_eval<'a, F>(
    cases: &[LocomoCase],
    conversation_index: &LocomoConversationIndex,
    search_fn: F,
) -> Result<LocomoReport>
where
    F: Fn(&str) -> BoxFuture<'a, Result<Vec<ScoredChunk>>>,
{
    // Filter out category 5 (Adversarial) for standard recall/MRR metric aggregation per official benchmark specification
    let eval_cases: Vec<&LocomoCase> = cases
        .iter()
        .filter(|c| c.category != LocomoQuestionCategory::Adversarial)
        .collect();

    let mut category_rec5_hits: HashMap<LocomoQuestionCategory, usize> = HashMap::new();
    let mut category_mrr_sum: HashMap<LocomoQuestionCategory, f64> = HashMap::new();
    let mut category_total: HashMap<LocomoQuestionCategory, usize> = HashMap::new();

    let mut total_rec5_hits = 0;
    let mut total_mrr_sum = 0.0;

    for case in &eval_cases {
        let entry_tot = category_total.entry(case.category).or_insert(0);
        *entry_tot += 1;

        let results = search_fn(&case.question).await?;

        let expected_lower = case.expected_answer.to_lowercase();
        let sample_map = conversation_index.get(&case.sample_id);

        let resolved_evidence_lowers: Vec<String> = case
            .evidence
            .iter()
            .map(|ev| {
                if let Some(map) = sample_map {
                    if let Some(turn_text) = map.get(ev) {
                        return turn_text.to_lowercase();
                    }
                }
                ev.to_lowercase()
            })
            .collect();

        let mut hit_rank: Option<usize> = None;

        for (rank_idx, chunk) in results.iter().enumerate() {
            let chunk_lower = chunk.text.to_lowercase();
            let is_match = (!expected_lower.is_empty() && chunk_lower.contains(&expected_lower))
                || resolved_evidence_lowers
                    .iter()
                    .any(|ev| !ev.is_empty() && chunk_lower.contains(ev));

            if is_match {
                hit_rank = Some(rank_idx + 1);
                break;
            }
        }

        if let Some(rank) = hit_rank {
            let mrr = 1.0 / (rank as f64);
            let entry_mrr = category_mrr_sum.entry(case.category).or_insert(0.0);
            *entry_mrr += mrr;
            total_mrr_sum += mrr;

            if rank <= 5 {
                let entry_rec = category_rec5_hits.entry(case.category).or_insert(0);
                *entry_rec += 1;
                total_rec5_hits += 1;
            }
        }
    }

    let mut per_category_recall_at_5 = HashMap::new();
    let mut per_category_mrr = HashMap::new();

    for (&cat, &total) in &category_total {
        let hits = category_rec5_hits.get(&cat).copied().unwrap_or(0);
        let mrr_sum = category_mrr_sum.get(&cat).copied().unwrap_or(0.0);

        let rec5 = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };
        let mrr = if total > 0 {
            mrr_sum / total as f64
        } else {
            0.0
        };

        per_category_recall_at_5.insert(cat, rec5);
        per_category_mrr.insert(cat, mrr);
    }

    let n = eval_cases.len();
    let overall_recall_at_5 = if n > 0 {
        total_rec5_hits as f64 / n as f64
    } else {
        0.0
    };
    let overall_mrr = if n > 0 { total_mrr_sum / n as f64 } else { 0.0 };

    Ok(LocomoReport {
        per_category_recall_at_5,
        per_category_mrr,
        overall_recall_at_5,
        overall_mrr,
        total_eval_cases: n,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_locomo_question_category_enum_and_display() {
        assert_eq!(
            LocomoQuestionCategory::from_u8(1),
            Some(LocomoQuestionCategory::MultiHop)
        );
        assert_eq!(
            LocomoQuestionCategory::from_u8(2),
            Some(LocomoQuestionCategory::Temporal)
        );
        assert_eq!(
            LocomoQuestionCategory::from_u8(3),
            Some(LocomoQuestionCategory::OpenDomain)
        );
        assert_eq!(
            LocomoQuestionCategory::from_u8(4),
            Some(LocomoQuestionCategory::SingleHop)
        );
        assert_eq!(
            LocomoQuestionCategory::from_u8(5),
            Some(LocomoQuestionCategory::Adversarial)
        );
        assert_eq!(LocomoQuestionCategory::from_u8(0), None);
        assert_eq!(LocomoQuestionCategory::from_u8(6), None);

        assert_eq!(format!("{}", LocomoQuestionCategory::MultiHop), "Multi-hop");
        assert_eq!(format!("{}", LocomoQuestionCategory::Temporal), "Temporal");
        assert_eq!(
            format!("{}", LocomoQuestionCategory::OpenDomain),
            "Open-domain"
        );
        assert_eq!(
            format!("{}", LocomoQuestionCategory::SingleHop),
            "Single-hop"
        );
        assert_eq!(
            format!("{}", LocomoQuestionCategory::Adversarial),
            "Adversarial"
        );
    }

    #[test]
    fn test_load_locomo_dataset_nonexistent_and_empty() {
        let missing_path = Path::new("nonexistent_locomo_12345.json");
        let err = load_locomo_dataset(missing_path);
        assert!(err.is_err());
        match err {
            Err(ContextraError::NotFound(msg)) => {
                assert!(msg.contains("LoCoMo dataset file not found"));
            }
            other => panic!("Expected NotFound error, got {:?}", other),
        }

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "[]").unwrap();
        let err_empty = load_locomo_dataset(temp.path());
        assert!(err_empty.is_err());
        match err_empty {
            Err(ContextraError::InvalidInput(msg)) => {
                assert!(msg.contains("No valid LoCoMo cases extracted"));
            }
            other => panic!("Expected InvalidInput error, got {:?}", other),
        }
    }

    #[test]
    fn test_load_locomo_dataset_various_answer_types() {
        let json_data = r#"[
            {
                "sample_id": "conv_1",
                "conversation": {
                    "session_1": [
                        {
                            "dia_id": "D1:1",
                            "speaker": "Alice",
                            "text": "I visited Berlin in May 2023."
                        }
                    ]
                },
                "qa": [
                    {
                        "question": "What is the capital of France?",
                        "answer": "Paris",
                        "category": 1,
                        "evidence": ["D1:1"]
                    },
                    {
                        "question": "What is 2 + 2?",
                        "answer": 4,
                        "category": 4,
                        "evidence": ["Math fact"]
                    },
                    {
                        "question": "List fruits",
                        "answer": ["apple", "banana"],
                        "category": 3,
                        "evidence": ["Fruit list"]
                    },
                    {
                        "question": "Trick question?",
                        "adversarial_answer": "None",
                        "category": 5,
                        "evidence": []
                    }
                ]
            }
        ]"#;

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", json_data).unwrap();

        let dataset = load_locomo_dataset(temp.path()).unwrap();
        assert_eq!(dataset.cases.len(), 4);

        let conv_map = dataset.conversation_index.get("conv_1").unwrap();
        assert_eq!(conv_map.get("D1:1").unwrap(), "I visited Berlin in May 2023.");

        assert_eq!(dataset[0].expected_answer, "Paris");
        assert_eq!(dataset[0].category, LocomoQuestionCategory::MultiHop);

        assert_eq!(dataset[1].expected_answer, "4");
        assert_eq!(dataset[1].category, LocomoQuestionCategory::SingleHop);

        assert_eq!(dataset[2].expected_answer, "apple banana");
        assert_eq!(dataset[2].category, LocomoQuestionCategory::OpenDomain);

        assert_eq!(dataset[3].expected_answer, "None");
        assert_eq!(dataset[3].category, LocomoQuestionCategory::Adversarial);
    }

    #[tokio::test]
    async fn test_run_locomo_eval_with_turn_dia_id_evidence_resolution() {
        let json_data = r#"[
            {
                "sample_id": "conv_test",
                "conversation": {
                    "session_1": [
                        {
                            "dia_id": "D1:10",
                            "speaker": "Bob",
                            "text": "Bob moved to Munich in 2024."
                        }
                    ]
                },
                "qa": [
                    {
                        "question": "Where did Bob move in 2024?",
                        "answer": "Munich",
                        "category": 4,
                        "evidence": ["D1:10"]
                    }
                ]
            }
        ]"#;

        let mut temp = NamedTempFile::new().unwrap();
        write!(temp, "{}", json_data).unwrap();

        let dataset = load_locomo_dataset(temp.path()).unwrap();

        let mock_search = |_q: &str| {
            Box::pin(async move {
                Ok(vec![ScoredChunk {
                    id: "chunk_1".into(),
                    text: "Bob moved to Munich in 2024.".into(),
                    score: 0.99,
                }])
            })
                as crate::long_mem_eval::BoxFuture<
                    'static,
                    contextra_core::Result<Vec<ScoredChunk>>,
                >
        };

        let report = run_locomo_eval(&dataset.cases, &dataset.conversation_index, mock_search)
            .await
            .unwrap();

        assert_eq!(report.overall_recall_at_5, 1.0);
        assert_eq!(report.overall_mrr, 1.0);
    }
}
