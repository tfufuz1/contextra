#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Invarianten- und Vertragstests für contextra-erasure-spi.

use contextra_erasure_spi::{
    KeyScope, LayerScanner, PurgeStats, Purger, ScanError, ScanSession, ScanTarget, ScanTechnique,
    SpiError, WitnessLayer,
};
use contextra_types::{RequestContext, TenantId, TxId};

struct MockScanner {
    layer: WitnessLayer,
    fail_positive_control: bool,
    scanned_units: u64,
    remaining_units: u64,
}

impl LayerScanner for MockScanner {
    fn layer(&self) -> WitnessLayer {
        self.layer.clone()
    }

    fn scan_units(
        &self,
        session: &mut ScanSession,
        _ctx: &RequestContext,
    ) -> Result<(), ScanError> {
        if !self.fail_positive_control {
            session.record_control_match();
        }
        for i in 0..self.scanned_units {
            let mut hash = [0u8; 32];
            hash[0] = (i & 0xFF) as u8;
            session.record_unit(hash, i < self.remaining_units);
        }
        Ok(())
    }
}

struct MockPurger {
    layer: WitnessLayer,
}

impl Purger for MockPurger {
    fn layer(&self) -> WitnessLayer {
        self.layer.clone()
    }

    fn purge(
        &self,
        _ctx: &RequestContext,
        _scope: &KeyScope,
    ) -> Result<PurgeStats, contextra_erasure_spi::PurgeError> {
        Ok(PurgeStats::new(42, 1024))
    }
}

#[test]
fn test_inv_witness_mint_valid_flow() {
    let scanner = MockScanner {
        layer: WitnessLayer::HnswIndex,
        fail_positive_control: false,
        scanned_units: 10,
        remaining_units: 0,
    };
    let tenant = match TenantId::try_new(1) {
        Ok(t) => t,
        Err(_) => return,
    };
    let ctx = RequestContext::new(tenant, TxId(10));
    let target = ScanTarget::new("test_target", [0x11; 32]);

    let res = scanner.scan(&ctx, target, ScanTechnique::Structural, 123456789);
    assert!(res.is_ok());
    if let Ok(witness) = res {
        assert_eq!(witness.witness_layer(), &WitnessLayer::HnswIndex);
        assert_eq!(witness.scanned(), 10);
        assert_eq!(witness.remaining(), 0);
        assert_eq!(witness.measured_at(), 123456789);
        assert_ne!(witness.digest(), &[0u8; 32]);
    }
}

#[test]
fn test_inv_witness_mint_missing_positive_control_fails() {
    let scanner = MockScanner {
        layer: WitnessLayer::HnswIndex,
        fail_positive_control: true, // Canary control fails
        scanned_units: 10,
        remaining_units: 0,
    };
    let tenant = match TenantId::try_new(1) {
        Ok(t) => t,
        Err(_) => return,
    };
    let ctx = RequestContext::new(tenant, TxId(10));
    let target = ScanTarget::new("test_target", [0x11; 32]);

    let res = scanner.scan(&ctx, target, ScanTechnique::Structural, 123456789);
    assert!(
        matches!(res, Err(SpiError::ControlMatchFailed(_))),
        "Expected ControlMatchFailed error, got: {:?}",
        res
    );
}

#[test]
fn test_inv_witness_mint_scanned_zero_fails_for_active_scan() {
    let scanner = MockScanner {
        layer: WitnessLayer::HnswIndex,
        fail_positive_control: false,
        scanned_units: 0, // Zero units
        remaining_units: 0,
    };
    let tenant = match TenantId::try_new(1) {
        Ok(t) => t,
        Err(_) => return,
    };
    let ctx = RequestContext::new(tenant, TxId(10));
    let target = ScanTarget::new("test_target", [0x11; 32]);

    let res = scanner.scan(&ctx, target, ScanTechnique::Structural, 123456789);
    assert!(
        matches!(res, Err(SpiError::WitnessMintViolation(_))),
        "Expected WitnessMintViolation error, got: {:?}",
        res
    );
}

#[test]
fn test_not_measured_empty_reason_fails() {
    let scanner = MockScanner {
        layer: WitnessLayer::CsrGraph,
        fail_positive_control: false,
        scanned_units: 0,
        remaining_units: 0,
    };
    let tenant = match TenantId::try_new(1) {
        Ok(t) => t,
        Err(_) => return,
    };
    let ctx = RequestContext::new(tenant, TxId(10));
    let target = ScanTarget::new("test_target", [0x11; 32]);

    let res = scanner.scan(
        &ctx,
        target,
        ScanTechnique::NotMeasured("   ".to_string()),
        123456789,
    );
    assert!(
        matches!(res, Err(SpiError::WitnessMintViolation(_))),
        "Expected WitnessMintViolation error, got: {:?}",
        res
    );
}

#[test]
fn test_digest_deterministic_independent_of_insertion_order() {
    let target = ScanTarget::new("test_target", [0xAA; 32]);

    let mut session1 = ScanSession::begin(
        WitnessLayer::SsTableAllLevels,
        target.clone(),
        ScanTechnique::Structural,
    );
    session1.record_control_match();
    session1.record_unit([1u8; 32], false);
    session1.record_unit([2u8; 32], false);
    session1.record_unit([3u8; 32], false);

    let mut session2 = ScanSession::begin(
        WitnessLayer::SsTableAllLevels,
        target,
        ScanTechnique::Structural,
    );
    session2.record_control_match();
    session2.record_unit([3u8; 32], false);
    session2.record_unit([1u8; 32], false);
    session2.record_unit([2u8; 32], false);

    let w1 = match session1.finish(100) {
        Ok(w) => w,
        Err(e) => panic!("session1 failed: {:?}", e),
    };
    let w2 = match session2.finish(100) {
        Ok(w) => w,
        Err(e) => panic!("session2 failed: {:?}", e),
    };

    assert_eq!(
        w1.digest(),
        w2.digest(),
        "Digests must match regardless of unit order"
    );
}

#[test]
fn test_purger_execution() {
    let purger = MockPurger {
        layer: WitnessLayer::LsmMemtable,
    };
    let tenant = match TenantId::try_new(1) {
        Ok(t) => t,
        Err(_) => return,
    };
    let ctx = RequestContext::new(tenant, TxId(10));
    let scope = KeyScope::Tenant { tenant };

    let stats = purger.purge(&ctx, &scope);
    assert!(stats.is_ok());
    if let Ok(s) = stats {
        assert_eq!(s.purged_units, 42);
        assert_eq!(s.bytes_freed, 1024);
    }
}
