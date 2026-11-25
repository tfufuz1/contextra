# API: contextra-text

> Erzeugt aus dem Quellcode am 2026-11-25, Repository HEAD ce4579a56. Eine Zeile je Item: Datei:Zeile und die **erste Zeile** der Signatur. Mehrzeilige Signaturen sind nur angeschnitten. Nicht kompiliert, daher ohne Typprüfung. Testcode (`tests`, `testing`, `test_doubles`) ist ausgeschlossen.

Umfang: 222 öffentliche Items, 20 crate-interne Sichtbarkeitsgrenzen.

## Innen: Abhängigkeiten zu anderen Workspace-Crates

- contextra-ports
- contextra-types

## Außen: öffentliche Schnittstelle (`pub`)

```rust
src/bm25.rs:24  pub const BM25_K1: f32 = 1.5
src/bm25.rs:28  pub const BM25_B: f32 = 0.75
src/bm25.rs:36  pub struct BM25
src/bm25.rs:48  pub fn new(k1: f32, b: f32) -> Result<Self>
src/bm25.rs:62  pub fn score_term(&self, tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32
src/bm25.rs:86  pub fn score_term(tf: u32, doc_len: u32, avg_doc_len: f32, df: u32, n: u32) -> f32
src/bm25.rs:98  pub fn score_term_with_params(
src/bm25.rs:135  pub type FieldId = u32
src/bm25.rs:140  pub struct FieldWeight
src/bm25.rs:149  pub fn new(field_id: FieldId, weight: f32, b: f32) -> Result<Self>
src/bm25.rs:171  pub struct BM25F
src/bm25.rs:180  pub fn new(k1: f32, field_weights: Vec<FieldWeight>) -> Result<Self>
src/bm25.rs:197  pub fn score_term(
src/bm25.rs:219  pub fn score_term_bm25f(
src/contract.rs:3  pub enum Error
src/contract.rs:15  pub type Result<T> = core::result::Result<T, Error>
src/contract.rs:16  pub trait Clock: Send + Sync
src/contract.rs:19  pub trait Rng: Send
src/contract.rs:23  pub struct DocKey(pub u128); // From<u64> vorhanden, erhält 128-Bit-Fähigkeit
src/contract.rs:39  pub struct Epoch(pub u64); // opake, monotone Sichtbarkeitsmarke; ersetzt TxId (Konvertierung nur im Adapter)
src/domain/legal_de.rs:24  pub struct LegalDomainVocabulary
src/domain/legal_de.rs:42  pub fn get_legal_compound_stems() -> &'static [&'static str]
src/domain/legal_de.rs:48  pub fn get_legal_protected_terms() -> &'static [&'static str]
src/domain/legal_de.rs:54  pub static LEGAL_COMPOUND_STEMS: std::sync::LazyLock<&'static [&'static str]> =
src/domain/legal_de.rs:59  pub static LEGAL_PROTECTED_TERMS: std::sync::LazyLock<&'static [&'static str]> =
src/domain/medical_de.rs:24  pub struct MedicalDomainVocabulary
src/domain/medical_de.rs:42  pub fn get_medical_compound_stems() -> &'static [&'static str]
src/domain/medical_de.rs:48  pub fn get_medical_protected_terms() -> &'static [&'static str]
src/domain/medical_de.rs:54  pub static MEDICAL_COMPOUND_STEMS: std::sync::LazyLock<&'static [&'static str]> =
src/domain/medical_de.rs:59  pub static MEDICAL_PROTECTED_TERMS: std::sync::LazyLock<&'static [&'static str]> =
src/domain/mod.rs:11  pub mod legal_de
src/domain/mod.rs:12  pub mod medical_de
src/domain/mod.rs:14  pub use legal_de::LegalDomainVocabulary
src/domain/mod.rs:15  pub use medical_de::MedicalDomainVocabulary
src/domain/mod.rs:22  pub trait DomainVocabulary: Send + Sync
src/error.rs:8  pub enum TextError
src/error.rs:44  pub type Result<T> = core::result::Result<T, TextError>
src/inverted.rs:29  pub mod purge
src/inverted.rs:30  pub mod scanner
src/inverted.rs:36  pub use index_struct::InvertedIndex
src/inverted.rs:37  pub use morph_index::BM25MorphIndex
src/inverted.rs:38  pub use purge::{text_purger, PurgeReport, TextPurger}
src/inverted.rs:39  pub use scanner::{text_scanner, ParsedTextTarget, TextScanner}
src/inverted.rs:40  pub use types::{Language, TextIndexMetadata}
src/inverted/index_struct.rs:18  pub struct InvertedIndex<S: StorageEngine>
src/inverted/index_struct.rs:51  pub const MAX_TEXT_BYTES: usize = 10 * 1024 * 1024
src/inverted/index_struct.rs:55  pub const MAX_STAGED_TRANSACTIONS: usize = 10_000
src/inverted/index_struct.rs:65  pub fn new_with_language(storage: Arc<S>, namespace: &str, language: Language) -> Self
src/inverted/index_struct.rs:93  pub fn new(storage: Arc<S>, namespace: &str) -> Self
src/inverted/index_struct.rs:99  pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self
src/inverted/index_struct.rs:105  pub fn storage(&self) -> &Arc<S>
src/inverted/index_struct.rs:110  pub fn prefix(&self) -> &[u8]
src/inverted/index_struct.rs:115  pub fn commit_lock(&self) -> &parking_lot::Mutex<()>
src/inverted/morph_index.rs:11  pub struct BM25MorphIndex<S: StorageEngine>
src/inverted/morph_index.rs:19  pub fn new(
src/inverted/morph_index.rs:36  pub fn tokenizer(&self) -> &dyn MorphologicalTokenizer
src/inverted/purge.rs:44  pub struct PurgeReport
src/inverted/purge.rs:58  pub fn new() -> Self
src/inverted/purge.rs:69  pub fn is_success(&self) -> bool
src/inverted/purge.rs:75  pub fn to_purge_stats(&self) -> Result<PurgeStats>
src/inverted/purge.rs:102  pub struct TextPurger<S: StorageEngine + 'static>
src/inverted/purge.rs:108  pub fn new(index: Arc<InvertedIndex<S>>) -> Self
src/inverted/purge.rs:114  pub fn purge_document_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport
src/inverted/purge.rs:120  pub fn purge_document(&self, tx: TxId, doc_id: DocId) -> Result<PurgeStats>
src/inverted/purge.rs:420  pub fn purge_collection_report(&self, tx: TxId, namespace: &str) -> PurgeReport
src/inverted/purge.rs:482  pub fn purge_collection(&self, tx: TxId, namespace: &str) -> Result<PurgeStats>
src/inverted/purge.rs:505  pub fn text_purger<S: StorageEngine + 'static>(index: Arc<InvertedIndex<S>>) -> Arc<dyn Purger>
src/inverted/scanner.rs:33  pub enum ParsedTextTarget
src/inverted/scanner.rs:40  pub fn parse(target_id: &str) -> Self
src/inverted/scanner.rs:94  pub struct TextScanner<S: StorageEngine + 'static>
src/inverted/scanner.rs:102  pub fn new(index: Arc<InvertedIndex<S>>) -> Self
src/inverted/scanner.rs:111  pub fn new_with_target(index: Arc<InvertedIndex<S>>, target: ScanTarget) -> Self
src/inverted/scanner.rs:120  pub fn with_target(self, target: ScanTarget) -> Self
src/inverted/scanner.rs:127  pub fn set_target(&self, target: ScanTarget)
src/inverted/scanner.rs:303  pub fn text_scanner<S: StorageEngine + 'static>(
src/inverted/types.rs:10  pub enum Language
src/inverted/types.rs:26  pub fn from_iso(code: &str) -> Self
src/inverted/types.rs:39  pub struct TextIndexMetadata
src/lib.rs:40  pub mod contract
src/lib.rs:41  pub mod error
src/lib.rs:42  pub mod memory_index
src/lib.rs:43  pub mod traits
src/lib.rs:44  pub mod types
src/lib.rs:46  pub use contract::{Clock, DocKey, Epoch, Error as ContractError, Result as ContractResult, Rng}
src/lib.rs:48  pub use error::{Result as LocalResult, TextError}
src/lib.rs:49  pub use memory_index::{CompactStats, MemoryTextIndex}
src/lib.rs:50  pub use traits::{Index, Maintain}
src/lib.rs:51  pub use types::{DocId as LocalDocId, TextHit, MAX_SEARCH_K as LOCAL_MAX_SEARCH_K}
src/lib.rs:53  pub mod bm25
src/lib.rs:54  pub mod domain
src/lib.rs:55  pub mod morphology
src/lib.rs:56  pub mod posting_list
src/lib.rs:57  pub mod tokenizer
src/lib.rs:59  pub use bm25::{score_term_bm25f, FieldId, FieldWeight, BM25, BM25F}
src/lib.rs:60  pub use domain::{DomainVocabulary, LegalDomainVocabulary, MedicalDomainVocabulary}
src/lib.rs:61  pub use morphology::{normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer}
src/lib.rs:62  pub use posting_list::{Posting, PostingList, ResidentPostingIndex}
src/lib.rs:63  pub use tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer}
src/memory_index.rs:25  pub const MAX_DOC_BYTES: usize = 10 * 1024 * 1024
src/memory_index.rs:28  pub const MAX_DOC_TERMS: usize = 1_000_000
src/memory_index.rs:31  pub const MAX_POSTINGS_PER_TERM: usize = 10_000_000
src/memory_index.rs:35  pub struct CompactStats
src/memory_index.rs:45  pub struct KeyInspection
src/memory_index.rs:56  pub struct IndexSnapshot
src/memory_index.rs:85  pub fn search(&self, query_tokens: &[String], scorer: &BM25, limit: usize) -> Vec<TextHit>
src/memory_index.rs:151  pub struct MemoryTextIndex
src/memory_index.rs:165  pub fn new() -> Self
src/memory_index.rs:174  pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self
src/memory_index.rs:180  pub fn snapshot(&self) -> Arc<IndexSnapshot>
src/memory_index.rs:185  pub fn current_epoch(&self) -> Epoch
src/memory_index.rs:190  pub fn len(&self) -> usize
src/memory_index.rs:195  pub fn is_empty(&self) -> bool
src/memory_index.rs:201  pub fn inspect_key(&self, key: &DocKey) -> KeyInspection
src/memory_index.rs:221  pub fn inspect_postings(&self) -> BTreeMap<String, Vec<Posting>>
src/memory_index.rs:227  pub fn inspect_doc_lengths(&self) -> BTreeMap<DocKey, u32>
src/morphology.rs:29  pub use compound_splitter::{GermanCompoundSplitter, MorphologicalTokenizer}
src/morphology.rs:30  pub use metrics::TokenReductionMetrics
src/morphology.rs:31  pub use passthrough::PassthroughTokenizer
src/morphology.rs:32  pub use stopwords::{get_german_stopwords, is_german_stopword, normalize_umlauts}
src/morphology.rs:33  pub use trie::{Trie, TrieNode}
src/morphology/compound_splitter.rs:24  pub trait MorphologicalTokenizer: Send + Sync
src/morphology/compound_splitter.rs:42  pub struct GermanCompoundSplitter
src/morphology/compound_splitter.rs:54  pub fn new() -> Self
src/morphology/compound_splitter.rs:60  pub fn with_min_length(min_len: usize) -> Self
src/morphology/compound_splitter.rs:81  pub fn with_dictionary(min_len: usize, custom_words: HashSet<String>) -> Self
src/morphology/compound_splitter.rs:98  pub fn with_domains(min_len: usize, domains: &[&dyn DomainVocabulary]) -> Self
src/morphology/compound_splitter.rs:119  pub fn new_with_all_domains() -> Self
src/morphology/compound_splitter.rs:127  pub fn extend_vocabulary(&mut self, words: impl IntoIterator<Item = impl AsRef<str>>)
src/morphology/compound_splitter.rs:138  pub fn min_component_len(&self) -> usize
src/morphology/compound_splitter.rs:143  pub fn with_max_depth(mut self, max_depth: usize) -> Self
src/morphology/compound_splitter.rs:149  pub fn max_depth(&self) -> usize
src/morphology/metrics.rs:6  pub struct TokenReductionMetrics
src/morphology/metrics.rs:26  pub fn expansion_ratio(&self) -> f32
src/morphology/passthrough.rs:9  pub struct PassthroughTokenizer
src/morphology/passthrough.rs:16  pub fn new(lang: impl Into<String>) -> Self
src/morphology/stopwords.rs:13  pub fn get_german_stopwords() -> &'static HashSet<String>
src/morphology/stopwords.rs:32  pub fn is_german_stopword(word: &str) -> bool
src/morphology/stopwords.rs:56  pub fn normalize_umlauts(input: &str) -> String
src/morphology/trie.rs:6  pub struct TrieNode
src/morphology/trie.rs:16  pub struct Trie
src/morphology/trie.rs:23  pub fn new() -> Self
src/morphology/trie.rs:29  pub fn insert(&mut self, word: &str)
src/morphology/trie.rs:39  pub fn contains(&self, word: &str) -> bool
src/morphology/trie.rs:53  pub fn starts_with(&self, prefix: &str) -> bool
src/posting_list.rs:30  pub const POSTING_LIST_V2_MAGIC: &[u8; 4] = b"PL\x02\x00"
src/posting_list.rs:34  pub const BLOCK_SIZE: usize = 64
src/posting_list.rs:37  pub type DocIdRaw = u64
src/posting_list.rs:40  pub type DocIdRaw = u128
src/posting_list.rs:46  pub struct PostingBlockInfo
src/posting_list.rs:61  pub struct Posting
src/posting_list.rs:72  pub fn new(doc_id: DocId, tf: u32, doc_len: u32) -> Self
src/posting_list.rs:81  pub fn doc_id(&self) -> DocId
src/posting_list.rs:119  pub struct PostingList
src/posting_list.rs:127  pub fn new(mut postings: Vec<Posting>) -> Self
src/posting_list.rs:136  pub fn empty() -> Self
src/posting_list.rs:146  pub fn as_slice(&self) -> &[Posting]
src/posting_list.rs:153  pub fn blocks(&self) -> &[PostingBlockInfo]
src/posting_list.rs:160  pub fn len(&self) -> usize
src/posting_list.rs:167  pub fn is_empty(&self) -> bool
src/posting_list.rs:173  pub fn upsert(&self, posting: Posting) -> Self
src/posting_list.rs:194  pub fn remove(&self, doc_id: DocId) -> Self
src/posting_list.rs:208  pub fn encode_compact(&self) -> Result<Vec<u8>, ContextraError>
src/posting_list.rs:234  pub fn decode_compact(bytes: &[u8]) -> Result<Self, ContextraError>
src/posting_list.rs:354  pub struct ResidentPostingIndex
src/posting_list.rs:361  pub fn new() -> Self
src/posting_list.rs:369  pub fn get(&self, term: &str) -> Option<Arc<PostingList>>
src/posting_list.rs:375  pub fn insert_list(&self, term: String, list: PostingList)
src/posting_list.rs:381  pub fn upsert_posting(&self, term: &str, posting: Posting)
src/posting_list.rs:393  pub fn remove_posting_from_terms(&self, terms: &[String], doc_id: DocId)
src/posting_list.rs:409  pub fn remove_terms(&self, terms: &[String])
src/posting_list.rs:418  pub fn clear(&self)
src/stream.rs:25  pub const DEFAULT_STREAM_BATCH_SIZE: usize = 16
src/stream.rs:42  pub struct Bm25CandidateStream<'a, S: StorageEngine>
src/stream.rs:62  pub fn new(
src/stream.rs:82  pub fn with_batch_size(mut self, n: usize) -> Self
src/stream.rs:129  pub fn yielded(&self) -> usize
src/stream.rs:135  pub fn is_exhausted(&self) -> bool
src/stream.rs:141  pub const fn max_depth() -> usize
src/tokenizer.rs:34  pub fn compile_protected_regex(pattern: &str) -> Result<Regex>
src/tokenizer.rs:42  pub fn get_protected_regex() -> Result<&'static Regex>
src/tokenizer.rs:134  pub trait Tokenizer: Send + Sync
src/tokenizer.rs:141  pub struct DefaultTokenizer
src/tokenizer.rs:146  pub fn try_new() -> Result<Self>
src/tokenizer.rs:158  pub fn new() -> Self
src/tokenizer.rs:202  pub struct GermanMorphTokenizer
src/tokenizer.rs:209  pub fn try_new() -> Result<Self>
src/tokenizer.rs:222  pub fn new() -> Self
src/tokenizer.rs:300  pub fn tokenize(text: &str) -> Vec<String>
src/tokenizer.rs:311  pub struct Token
src/tokenizer.rs:324  pub fn new(
src/tokenizer.rs:342  pub enum OverflowPolicy
src/tokenizer.rs:353  pub struct PipelineLimits
src/tokenizer.rs:378  pub enum NormalizationPolicy
src/tokenizer.rs:392  pub fn normalize(&self, text: &str) -> String
src/tokenizer.rs:431  pub trait TokenFilter: Send + Sync
src/tokenizer.rs:442  pub trait Analyzer: Send + Sync
src/tokenizer.rs:453  pub struct AnalyzerPipeline
src/tokenizer.rs:465  pub fn new(name: impl Into<String>, version: u32) -> Self
src/tokenizer.rs:478  pub fn with_limits(mut self, limits: PipelineLimits) -> Self
src/tokenizer.rs:484  pub fn with_normalization(mut self, norm: NormalizationPolicy) -> Self
src/tokenizer.rs:490  pub fn with_stopwords(mut self, stopwords: HashSet<String>) -> Self
src/tokenizer.rs:496  pub fn with_filter(mut self, filter: Arc<dyn TokenFilter>) -> Self
src/tokenizer.rs:502  pub fn with_german_morphology(
src/tokenizer.rs:511  pub fn default_english() -> Self
src/tokenizer.rs:519  pub fn default_german() -> Self
src/traits.rs:6  pub trait Index<Id, Query: ?Sized>
src/traits.rs:23  pub trait Maintain
src/types.rs:9  pub struct DocId(pub u64)
src/types.rs:15  pub struct DocId(pub u128)
src/types.rs:21  pub const fn new(id: u64) -> Self
src/types.rs:28  pub const fn new(id: u128) -> Self
src/types.rs:35  pub const fn inner(self) -> u64
src/types.rs:42  pub const fn inner(self) -> u128
src/types.rs:49  pub const fn as_u64(self) -> u64
src/types.rs:72  pub struct TextHit
src/types.rs:80  pub const MAX_SEARCH_K: usize = 10_000
src/wand.rs:57  pub struct BoundedTopK
src/wand.rs:65  pub fn new(capacity: usize) -> Self
src/wand.rs:75  pub fn push(&mut self, doc_id: DocId, score: f32)
src/wand.rs:92  pub fn min_threshold(&self) -> f32
src/wand.rs:102  pub fn into_sorted_vec(self) -> Vec<(DocId, f32)>
src/wand.rs:242  pub struct WandSearchResult
```

## Innen: crate-interne Grenzen (`pub(crate)`, `pub(super)`, `pub(in …)`)

```rust
src/domain/mod.rs:32  pub(crate) struct ParsedVocabulary
src/domain/mod.rs:37  pub(crate) fn parse_domain_resource(raw: &'static str) -> ParsedVocabulary
src/inverted/index_struct.rs:22  pub(crate) total_docs: Arc<AtomicU64>,
src/inverted/index_struct.rs:23  pub(crate) total_tokens: Arc<AtomicU64>,
src/inverted/index_struct.rs:24  pub(crate) avg_doc_len_x1000: Arc<AtomicU64>, // Cached fixed-point (FIND-TXT-004)
src/inverted/index_struct.rs:25  pub(crate) staged_stats: Arc<parking_lot::Mutex<HashMap<TxId, StagedStatsChange>>>,
src/inverted/index_struct.rs:26  pub(crate) staged_terms: Arc<parking_lot::Mutex<HashMap<TxId, Vec<String>>>>,
src/inverted/index_struct.rs:28  pub(crate) resident_index: Arc<ResidentPostingIndex>,
src/inverted/index_struct.rs:134  pub(crate) fn key(&self, suffix: &str) -> Vec<u8>
src/inverted/index_struct.rs:141  pub(crate) fn key_with_id(&self, type_prefix: &str, id: impl Into<u128>) -> Vec<u8>
src/inverted/index_struct.rs:151  pub(crate) fn key_with_term_doc(&self, term: &str, doc_id: DocId) -> Vec<u8>
src/inverted/index_struct.rs:163  pub(crate) fn key_term_prefix(&self, term: &str) -> Vec<u8>
src/inverted/index_struct.rs:172  pub(crate) fn key_batch_posting_list(&self, term: &str) -> Vec<u8>
src/inverted/index_struct.rs:181  pub(crate) async fn ensure_term_loaded(
src/inverted/index_struct.rs:249  pub(crate) fn key_tombstone(&self, doc_id: DocId, term: &str) -> Vec<u8>
src/inverted/index_struct.rs:539  pub(crate) async fn commit_stats(&self, tx: TxId) -> Result<()>
src/inverted/index_struct.rs:598  pub(crate) async fn rollback_stats(&self, tx: TxId) -> Result<()>
src/inverted/types.rs:51  pub(crate) struct StagedStatsChange
src/inverted/types.rs:52  pub(crate) docs_delta: i64,
src/inverted/types.rs:53  pub(crate) tokens_delta: i64,
```
