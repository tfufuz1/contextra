//! Dokument- und Kontext-Referenzen (SPEC 7.2, 7.3, D-70, I-11).
//!
//! Re-Export der Typen aus `ids` für Abwärtskompatibilität.

pub use crate::ids::{
    CollectionRef, DocHandle, DocKey, DocRef, OperatorRef, SubjectRef, TenantRef, MAX_REF_LEN,
};
pub use crate::tenant_scope::TenantContext;
