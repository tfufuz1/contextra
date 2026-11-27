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
            #[allow(clippy::panic)]
            {
                ::std::panic::panic_any(failure.to_string());
            }
        }
    };
}

/// Erzeugt für eine [`crate::conformance::erasure_layer::LayerFixture`] die Tests C1–C7,
/// K1 und K2 als einzelne `#[tokio::test]`s. Der Aufrufer braucht `tokio` als Dev-Dependency.
#[cfg(feature = "erasure-conformance")]
#[macro_export]
macro_rules! assert_layer_conformance {
    ($fixture:ty) => {
        $crate::assert_layer_conformance!(@cases $fixture;
            c1_idempotence, c2_resumption, c3_scope_isolation, c4_witness_honesty,
            c5_fault_injection, c6_cancel_safety, c7_gate_respect, k1_gate_idempotent,
            k2_determinism);
    };
    (@cases $fixture:ty; $($case:ident),+) => {
        $(
            #[tokio::test]
            async fn $case() {
                $crate::assert_conformance!(
                    $crate::conformance::erasure_layer::$case(
                        <$fixture as $crate::conformance::erasure_layer::LayerFixture>::new,
                    )
                    .await
                );
            }
        )+
    };
}

#[cfg(test)]
mod tests {
    use crate::conformance::failure::ConformanceFailure;

    #[test]
    fn test_assert_conformance_macro_success() {
        let ok_res: Result<(), ConformanceFailure> = Ok(());
        assert_conformance!(ok_res);
    }

    #[test]
    #[should_panic(expected = "synthetic test failure")]
    fn test_assert_conformance_macro_failure() {
        let err_res: Result<(), ConformanceFailure> = Err(ConformanceFailure::new(
            "TestSuite",
            "test_case",
            "synthetic test failure",
        ));
        assert_conformance!(err_res);
    }
}
