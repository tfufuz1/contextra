// FILE-CONTEXT
// ZWECK: Testet den kompilierte Default-Wert fuer AutoExtractionConfig (enabled: true vs. opt-out).

use contextra_engine::collection::crud::AutoExtractionConfig;

#[test]
fn test_auto_extraction_config_default_value() {
    let cfg = AutoExtractionConfig::default();

    #[cfg(not(feature = "auto-extraction-opt-out"))]
    {
        assert!(
            cfg.enabled,
            "AutoExtractionConfig::default().enabled must be true by default"
        );
    }

    #[cfg(feature = "auto-extraction-opt-out")]
    {
        assert!(
            !cfg.enabled,
            "AutoExtractionConfig::default().enabled must be false when auto-extraction-opt-out feature is active"
        );
    }
}
