// FILE-CONTEXT
// STAND: 2026-10-21 (W1-13)
// ZWECK: Trybuild test demonstrating unrepresentability of "both None" fail-open state for LedgerSource.

use contextra_store::restore_guard::LedgerSource;

fn main() {
    // Attempting to construct LedgerSource without Direct(...) or Handle(...) fails compilation.
    let _source: LedgerSource = None;
}
