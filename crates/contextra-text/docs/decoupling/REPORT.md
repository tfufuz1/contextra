# Decoupling Report: contextra-text (AUFTRAG P5)

## 1. Baseline V0
- `cargo check -p contextra-text --no-default-features`: PASSED
- `cargo check -p contextra-text --all-targets`: PASSED
- `cargo test -p contextra-text --lib`: 110 passed, 1 flaky timing test passed in isolation.

## 2. Work Packages & Commits
- WP-1 & WP-2 & WP-3..6: Commit `ebbac0996`:
  - `Cargo.toml`: Introduced `workspace` feature gating `contextra-types` and `contextra-ports`. Added `contextra-testkit` as dev-dependency. Aliases `types-compat` and `ports-compat` retained.
  - `src/error.rs`: Moved conversions to `src/compat/error.rs`.
  - `src/types.rs`: Moved DocId / TextHit conversions to `src/compat/types.rs`.
  - `src/wand.rs`: Uses local types `crate::types::{DocId, MAX_SEARCH_K}` and `crate::error::{Result, TextError}`.
  - `src/stream.rs`: Uses local types.
  - `src/inverted/index_struct.rs`: Uses local types and `PostingStore`. `TextIndex` trait implementation moved to `src/compat/text_index.rs`.
  - `src/inverted/tests/`: Moved to `src/compat/inverted_tests/`.

## 3. Verification Output
- V1: `cargo check -p contextra-text --no-default-features`: PASSED (exit code 0)
- V2: `cargo check -p contextra-text --all-targets`: PASSED (exit code 0)
- V3: `cargo test -p contextra-text --lib`: PASSED
- V5: `grep -rEn 'contextra_(types|ports|testkit)' crates/contextra-text/src --include=*.rs | grep -v '/compat/'`: PASSED (0 occurrences)
- V6: `cargo tree -p contextra-text -e normal --no-default-features --prefix none | grep '^contextra-' | sort -u`:
  ```
  contextra-text v0.1.0 (/app/crates/contextra-text)
  ```
  (Closure is strictly empty of other internal workspace crates).
- V7: Workspace callers (`contextra-db`, `contextra-engine`, `contextra-agent`) compile without breaking changes.

## 4. Foreign Caller Inventory (H3(a))
- `crates/contextra-db`: Language, InvertedIndex, score_term
- `crates/contextra-engine`: Language, InvertedIndex, TextIndex adapters
- `crates/contextra-agent`: Language::English
- `crates/contextra-rank`: comment reference

## 5. Facade List (T2 Cases)
- All public signatures remain backwards compatible under default features.
