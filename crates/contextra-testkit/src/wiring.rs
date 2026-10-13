// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Composition Root wiring entrypoint for Testkit.

use crate::fault_vfs::FaultVfs;
use crate::manual_clock::ManualClock;
use crate::reference_model::ReferenceModel;
use contextra_types::ContextraError;

/// Dependencies required to build `TestkitWiring`.
#[derive(Debug, Clone, Default)]
pub struct TestkitWiringDeps {
    /// Initial nanoseconds timestamp for ManualClock.
    pub initial_nanos: u64,
}

/// Composition Root wiring handle for contextra-testkit.
#[derive(Debug)]
pub struct TestkitWiring {
    /// Initialized deterministic clock.
    pub clock: ManualClock,
    /// Initialized fault VFS.
    pub vfs: FaultVfs,
    /// Initialized MVCC reference model.
    pub reference_model: ReferenceModel,
}

impl TestkitWiring {
    /// Builds the `TestkitWiring` instance.
    pub fn build(deps: TestkitWiringDeps) -> Result<Self, ContextraError> {
        let clock = ManualClock::new(deps.initial_nanos);
        let vfs = FaultVfs::new();
        let reference_model = ReferenceModel::new();

        Ok(Self {
            clock,
            vfs,
            reference_model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_testkit_wiring_build() {
        let deps = TestkitWiringDeps { initial_nanos: 5000 };
        let wiring = TestkitWiring::build(deps).expect("build TestkitWiring");
        assert_eq!(wiring.clock.now_nanos(), 5000);
    }
}
