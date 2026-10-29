/// Asserts that a conformance test suite execution succeeded.
/// If `res` is an `Err(failure)`, panics with a detailed failure report for test runners.
#[macro_export]
macro_rules! assert_conformance {
    ($res:expr) => {
        if let Err(failure) = $res {
            panic!("{}", failure);
        }
    };
}
