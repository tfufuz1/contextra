// TODO(acta-altlast): Bruecken-Modul der Altquelle (HR-6 verbietet Bruecken). Referenz fuer die Trait-Implementierungen (GraphIndex, Purger); beim Umbau in direkte Implementierungen aufloesen.
//! Algorithm error bridge for workspace compatibility.

use crate::tl_hfd::TlHfdError;
#[cfg(feature = "apprh-diffusion")]
use crate::apprh::ApprhError;
use acta_base::ActaError; // TODO(acta-hr3): Hub-Import (ex types/ports) nach HR-2/HR-3 aufloesen

impl From<TlHfdError> for ActaError {
    fn from(err: TlHfdError) -> Self {
        ActaError::InvalidInput(err.to_string())
    }
}

#[cfg(feature = "apprh-diffusion")]
impl From<ApprhError> for ActaError {
    fn from(err: ApprhError) -> Self {
        ActaError::InvalidInput(err.to_string())
    }
}
