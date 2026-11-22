// FILE-CONTEXT
// ZWECK: Property-basierter Fuzz-Test für detect (WP M3 / P01): "detect paniert nie".
// INVARIANTEN: Zero-Panic für beliebige Eingabe-Bytes und Dateinamen/MIME Hints.
// STAND: TS:2026-10-31T00:00:00Z

use contextra_ingest::detect::{detect, DetectHints};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]

    #[test]
    fn test_detect_never_panics_arbitrary_bytes(
        data in proptest::collection::vec(any::<u8>(), 0..1000),
        filename in proptest::option::of(".*"),
        mime in proptest::option::of(".*")
    ) {
        let hints = DetectHints {
            filename: filename.as_deref(),
            mime: mime.as_deref(),
        };

        // detect darf niemals paniken
        let _ = detect(&data, &hints);
    }
}
