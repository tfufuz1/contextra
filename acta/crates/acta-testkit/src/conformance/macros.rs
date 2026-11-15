//! Hilfsmakro für Konformitätsprüfungen (Spec 16.2).
//!
//! Das Makro ist nur für Testcode gedacht. Produktionscode verwendet es nicht.

/// Macht ein `ConformanceResult` im Testlauf zum Fehlschlag.
///
/// Ist das Ergebnis ein `Err(ConformanceFailure)`, bricht das Makro mit einer Panic
/// ab, die den formatierten Fehlerbericht als Nutzlast trägt.
#[macro_export]
macro_rules! assert_conformance {
    ($res:expr) => {
        if let ::core::result::Result::Err(failure) = $res {
            ::std::panic::panic_any(::std::string::ToString::to_string(&failure));
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::conformance::failure::ConformanceFailure;

    #[test]
    fn success_passes_through() {
        let ok: Result<(), ConformanceFailure> = Ok(());
        assert_conformance!(ok);
    }

    #[test]
    #[should_panic(expected = "synthetic test failure")]
    fn failure_panics_with_report() {
        let err: Result<(), ConformanceFailure> = Err(ConformanceFailure::new(
            "TestSuite",
            "test_case",
            "synthetic test failure",
        ));
        assert_conformance!(err);
    }
}
