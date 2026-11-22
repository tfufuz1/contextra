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
        Box::new(gov::security_gates::NoSecurityStubGate),
        Box::new(gov::security_gates::NoLiteralSecretsGate),
        Box::new(gov::security_gates::SecurityPortsGate),
        Box::new(gov::security_gates::KeyPurposesGate),
        Box::new(spec::deletion_gates::WitnessMintGate),
        Box::new(spec::deletion_gates::DeletionTypesOnceGate),
        Box::new(spec::deletion_gates::DeletionHonestyGate),
        Box::new(spec::deletion_gates::NoDummyProofGate),
        Box::new(spec::deletion_gates::ScannerCoverageGate),
        Box::new(code::code_gates::DeterminismGate),
        Box::new(code::code_gates::PanicSurfaceGate),
        Box::new(code::code_gates::CodeLintsGate),
        Box::new(arch::async_gates::AsyncIoGate),
    ]
}
