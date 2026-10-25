pub mod arch;
pub mod code;
pub mod docs;
pub mod gov;
pub mod spec;

use crate::core::gate::Gate;

pub fn registered_gates() -> Vec<Box<dyn Gate>> {
    vec![
        Box::new(gov::meta_gates::GateSelftestsGate),
        Box::new(gov::meta_gates::CatalogSyncGate),
    ]
}
