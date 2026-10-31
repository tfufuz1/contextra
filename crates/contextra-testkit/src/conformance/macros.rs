//! # Hilfsmakros für Konformitätsprüfungen (`conformance/macros`)
//!
//! Dieses Modul definiert Hilfsmakros zur vereinfachten Auswertung von
//! [`crate::conformance::ConformanceResult`] in Testläufen.

// ANCHOR[contextra-testkit:CONF-MACRO-ASSERT-001]
/// Überprüft, ob ein Konformitätsprüfungsergebnis erfolgreich war.
///
/// **Verhalten:**
/// Falls `res` ein `Err(ConformanceFailure)` ist, löst das Makro im Testläufer
/// eine Panic mit dem formatierten Fehlerbericht aus.
///
/// **Hinweis zu Regel 2 (Kein Panic in Produktion):**
/// Dieses Makro ist ausschließlich für Testcode bestimmt.
#[macro_export]
macro_rules! assert_conformance {
    ($res:expr) => {
        if let Err(failure) = $res {
            panic!("{}", failure);
        }
    };
}
