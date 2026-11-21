# contextra-text

**Zweck:** Textsuche: BM25 (mit Feld-Gewichtung BM25F), Block-Max WAND, invertierter Index, Tokenizer, deutsche Morphologie (Stemmer, Stoppwörter, Komposita-Zerlegung). SPEC §6.4.

**Ebene:** E1

## Öffentliche API (`src/lib.rs`, Stand dieser README)

Ist-Stand. Module oder Namen, die laut PLAN.md entfallen, können hier noch stehen.

```rust
pub mod compat
pub mod contract
pub mod error
pub mod memory_index
pub mod storage
pub mod traits
pub mod types
pub use contract::{Clock, DocKey, Epoch, Error as ContractError, Result as ContractResult, Rng}
pub use error::{Result as LocalResult, TextError}
pub use memory_index::{CompactStats, MemoryTextIndex}
pub use storage::PostingStore
pub use traits::{Index, Maintain}
pub use types::{DocId as LocalDocId, TextHit, MAX_SEARCH_K as LOCAL_MAX_SEARCH_K}
pub mod bm25
pub mod domain
pub mod inverted
pub mod morphology
pub mod posting_list
pub mod stream
pub mod tokenizer
pub mod wand
pub use bm25::{score_term_bm25f, FieldId, FieldWeight, BM25, BM25F}
pub use domain::{DomainVocabulary, LegalDomainVocabulary, MedicalDomainVocabulary}
pub use inverted::{InvertedIndex, Language}
pub use inverted::
pub use morphology::{normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer}
pub use posting_list::{Posting, PostingList, ResidentPostingIndex}
pub use stream::{Bm25CandidateStream, DEFAULT_STREAM_BATCH_SIZE}
pub use tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer}
pub use crate::compat::scorer::Bm25Scorer
```

## Features (`Cargo.toml`)

`default`, `std`, `serde`, `tracing`, `inspect`, `experimental`, `workspace`, `types-compat`, `ports-compat`, `bm25f`, `docid-128`

## Dokumentation

Implementierungsspezifikation: [`PLAN.md`](PLAN.md). Maßgeblich ist `docs/SPEC.md` v4.1.

