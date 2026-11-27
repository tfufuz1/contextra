//! Selbsttest der Konformanzsuite: korrekte Schicht grün, jeder Defekt rot.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_testkit::assert_layer_conformance;
use contextra_testkit::conformance::erasure_layer as suite;
use contextra_testkit::conformance::ConformanceResult;
use contextra_testkit::erasure_mocks::{Defect, MockLayerFixture};

assert_layer_conformance!(MockLayerFixture);

macro_rules! red {
    ($name:ident, $case:ident, $defect:ident, $expected:literal) => {
        #[tokio::test]
        async fn $name() {
            let result: ConformanceResult =
                suite::$case(|vfs| MockLayerFixture::with_defect(vfs, Defect::$defect)).await;
            let failure = result.expect_err("suite must turn red").to_string();
            assert!(failure.contains($expected), "wrong reason: {failure}");
        }
    };
}

red!(
    red_c1_recount,
    c1_idempotence,
    RecountsOnRerun,
    "second run committed units"
);
red!(
    red_c2_skip,
    c2_resumption,
    SkipsOnResume,
    "residue after purge"
);
red!(
    red_c3_neighbor,
    c3_scope_isolation,
    TouchesNeighbor,
    "neighbor state changed"
);
red!(
    red_c4_blind,
    c4_witness_honesty,
    BlindScanner,
    "scanner blind"
);
red!(
    red_c5_partial_ok,
    c5_fault_injection,
    PartialOkOnFault,
    "returned Ok"
);
red!(
    red_c6_progress,
    c6_cancel_safety,
    ProgressBeforeEffect,
    "residue after purge"
);
red!(
    red_c7_gate,
    c7_gate_respect,
    IgnoresGate,
    "visible after on_gate"
);
red!(
    red_c7_direct,
    c7_gate_respect,
    DirectRead,
    "around cx.reader"
);
red!(
    red_k1_gate,
    k1_gate_idempotent,
    NonIdempotentGate,
    "changed the state"
);
red!(
    red_k2_order,
    k2_determinism,
    NondeterministicOutput,
    "identical runs differ"
);

mod kernel {
    use contextra_ports::erasure::{EraseRequest, ErasureKernel, ResolvedScope};
    use contextra_ports::scanner::{ScanTechnique, WitnessLayer};
    use contextra_ports::{KeyPattern, KeyScope, Verdict};
    use contextra_testkit::erasure_mocks::*;
    use contextra_testkit::{mint_witness_for_tests, ManualClock};
    use contextra_types::*;

    #[tokio::test]
    async fn mock_kernel_contract_order_gate_lifetime_and_witnesses() {
        let layer = WitnessLayer::Custom("l".into());
        let kernel = MockErasureKernel::new(vec![layer.clone()]);
        kernel.insert_data(b"k/1".to_vec(), b"v".to_vec());
        kernel.insert_data(b"n/1".to_vec(), b"v".to_vec());
        let id = ErasureId::from_u128(9);
        let target = EraseTarget::Tenant {
            tenant: TenantId::try_new(1).unwrap(),
        };
        let keys = KeyScope::new(vec![(KeyPattern::Prefix(b"k/".to_vec()), Verdict::In)]).unwrap();
        let plan = ResolvedScope::new(id, target.kind(), &target, keys, Vec::new(), 0);
        let req = EraseRequest {
            id,
            target,
            reason: ErasureReason::UserRequest,
            policy: ErasurePolicy::default(),
        };
        assert!(!kernel.accept(&req, &plan).await.unwrap().existing);
        assert!(kernel.accept(&req, &plan).await.unwrap().existing);
        assert!(kernel.complete(id).await.is_err());
        assert!(kernel.seal(id, &[]).await.is_err());

        let gate = kernel.gate();
        assert!(!gate.check(b"k/1").is_blocked());
        let first = kernel.publish_gate(id).await.unwrap();
        assert_eq!(
            kernel.publish_gate(id).await.unwrap().gated_seq,
            first.gated_seq
        );
        assert!(gate.check(b"k/1").is_blocked() && !gate.check(b"n/1").is_blocked());

        let clock = ManualClock::new(1);
        let budget = ManualIoBudget::unlimited();
        let reader = kernel.privileged_reader(id).unwrap();
        assert!(reader.get(b"n/1").await.is_err());
        let progress = MockProgressSink::new();
        struct Never;
        impl contextra_ports::erasure::Cancel for Never {
            fn is_cancelled(&self) -> bool {
                false
            }
            fn cancelled(&self) -> contextra_ports::BoxFuture<'_, ()> {
                Box::pin(std::future::pending())
            }
        }
        let cx = contextra_ports::erasure::PurgeCx {
            erasure_id: id,
            clock: &clock,
            cancel: &Never,
            budget: &budget,
            cursor: None,
            reader: reader.as_ref(),
            progress: &progress,
            max_units: 8,
        };
        assert_eq!(
            kernel.purge_store(id, &cx).await.unwrap().units_committed,
            1
        );

        let dirty = mint_witness_for_tests(layer.clone(), 1, 1, ScanTechnique::Structural);
        assert!(kernel.seal(id, &[dirty]).await.is_err());
        assert!(gate.check(b"k/1").is_blocked());
        let state = kernel
            .fail(id, Some(&layer), LayerErrorKind::Retryable)
            .await
            .unwrap();
        assert!(matches!(state, ErasureState::Failed { attempt: 1, .. }));
        assert!(
            gate.check(b"k/1").is_blocked(),
            "I-9: gate survives failure"
        );
        assert!(kernel.status(id).unwrap().reached.gated);
        kernel.purge_store(id, &cx).await.unwrap();

        let receipt = kernel.seal(id, &[]).await.unwrap();
        assert!(!receipt.guarantee.purged, "I-6: unmeasured is not purged");
        assert!(receipt
            .exceptions
            .contains(&ErasureException::NotMeasured { layer: "l".into() }));
        assert!(gate.check(b"k/1").is_blocked());
        kernel.complete(id).await.unwrap();
        assert!(!gate.check(b"k/1").is_blocked());
        assert!(kernel.open_jobs().is_empty());
    }
}
