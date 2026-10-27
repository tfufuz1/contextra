// FILE-CONTEXT
// ZWECK: Trait LayerScanner und Modul-Export für gesperrte Scanner-Messungen (SPEC §12.7.4, INV-WITNESS-MINT-1).
// INVARIANTEN: LayerScanner::scan besitzt eine Default-Implementierung, die begin_scan, scan_units und finish ausführt.
// HOTSPOTS: [1-120]

#![forbid(unsafe_code)]

/// Layer and technique descriptor types.
pub mod layer;
/// ScanSession state and finish gate.
pub mod session;
/// LayerWitness verified measurement type.
pub mod witness;

pub use layer::*;
pub use session::*;
pub use witness::*;

use crate::clock::Clock;
use contextra_types::Result;

/// Trait für Storage-Scanner zur Durchführung von Restmengen-Messungen.
///
/// # Doctests (Compile Fail Checks)
///
/// (a) `LayerWitness` lässt sich außerhalb des Moduls nicht direkt konstruieren:
/// ```compile_fail
/// use contextra_ports::scanner::{LayerWitness, WitnessLayer, ScanTechnique};
/// let _w = LayerWitness {
///     layer: WitnessLayer::LsmMemtable,
///     scanned: 0,
///     remaining: 0,
///     technique: ScanTechnique::Structural,
///     digest: [0u8; 32],
///     measured_at: 0,
/// };
/// ```
///
/// (b) `ScanSession::finish` ist von außen nicht aufrufbar:
/// ```compile_fail
/// use contextra_ports::scanner::ScanSession;
/// fn try_finish(session: ScanSession, clock: &dyn contextra_ports::Clock) {
///     let _ = session.finish(clock);
/// }
/// ```
///
/// (c) `LayerWitness` hat keinen `Default`-Trait:
/// ```compile_fail
/// use contextra_ports::scanner::LayerWitness;
/// let _w = LayerWitness::default();
/// ```
pub trait LayerScanner: Send + Sync {
    /// Gibt die von diesem Scanner bediente Schicht zurück.
    fn layer(&self) -> WitnessLayer;

    /// Führt das Scannen von Einheiten auf der übergebenen `ScanSession` aus.
    fn scan_units(
        &self,
        session: &mut ScanSession,
        ctx: &contextra_types::RequestContext,
    ) -> Result<()>;

    /// Default-Implementierung von `scan`, die `begin`, `scan_units` und `finish` ausführt.
    fn scan(
        &self,
        ctx: &contextra_types::RequestContext,
        target: ScanTarget,
        technique: ScanTechnique,
        clock: &dyn Clock,
    ) -> Result<LayerWitness> {
        let mut session = ScanSession::begin(self.layer(), target, technique);
        self.scan_units(&mut session, ctx)?;
        session.finish(clock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::SystemClock;
    use contextra_types::{RequestContext, TenantId, TxId};

    struct TestScanner;

    impl LayerScanner for TestScanner {
        fn layer(&self) -> WitnessLayer {
            WitnessLayer::LsmMemtable
        }

        fn scan_units(&self, session: &mut ScanSession, _ctx: &RequestContext) -> Result<()> {
            session.record_control_match();
            session.record_unit([1u8; 32], false);
            session.record_unit([2u8; 32], false);
            Ok(())
        }
    }

    #[test]
    fn test_scan_session_success() {
        let clock = SystemClock::new();
        let scanner = TestScanner;
        let ctx = RequestContext::new(TenantId::try_new(1).unwrap(), TxId(100));
        let target = ScanTarget::new("tenant_1", [0xAA; 32]);

        let witness = scanner
            .scan(&ctx, target, ScanTechnique::Structural, &clock)
            .expect("scan should succeed");

        assert_eq!(witness.witness_layer(), &WitnessLayer::LsmMemtable);
        assert_eq!(witness.layer(), "LsmMemtable");
        assert_eq!(witness.scanned(), 2);
        assert_eq!(witness.remaining(), 0);
        assert_ne!(witness.digest(), &[0u8; 32]);
    }

    #[test]
    fn test_scan_session_scanned_zero_fails() {
        let clock = SystemClock::new();
        let target = ScanTarget::new("tenant_1", [0xAA; 32]);

        let mut session =
            ScanSession::begin(WitnessLayer::LsmMemtable, target, ScanTechnique::Structural);
        session.record_control_match();

        let res = session.finish(&clock);
        assert!(res.is_err(), "scanned == 0 must fail finish");
    }

    #[test]
    fn test_scan_session_control_matches_zero_fails() {
        let clock = SystemClock::new();
        let target = ScanTarget::new("tenant_1", [0xAA; 32]);

        let mut session =
            ScanSession::begin(WitnessLayer::LsmMemtable, target, ScanTechnique::Structural);
        session.record_unit([1u8; 32], false);

        let res = session.finish(&clock);
        assert!(res.is_err(), "control_matches == 0 must fail finish");
    }

    #[test]
    fn test_scan_session_not_measured_empty_reason_fails() {
        let clock = SystemClock::new();
        let target = ScanTarget::new("tenant_1", [0xAA; 32]);

        let session = ScanSession::begin(
            WitnessLayer::LsmMemtable,
            target,
            ScanTechnique::NotMeasured("".to_string()),
        );

        let res = session.finish(&clock);
        assert!(
            res.is_err(),
            "NotMeasured with empty reason must fail finish"
        );
    }

    #[test]
    fn test_digest_is_deterministic_and_changes_with_unit() {
        let clock = SystemClock::new();
        let target = ScanTarget::new("tenant_1", [0xAA; 32]);

        let mut s1 = ScanSession::begin(
            WitnessLayer::LsmMemtable,
            target.clone(),
            ScanTechnique::Structural,
        );
        s1.record_control_match();
        s1.record_unit([1u8; 32], false);
        s1.record_unit([2u8; 32], false);

        let mut s2 = ScanSession::begin(
            WitnessLayer::LsmMemtable,
            target.clone(),
            ScanTechnique::Structural,
        );
        s2.record_control_match();
        s2.record_unit([2u8; 32], false);
        s2.record_unit([1u8; 32], false); // order swapped, should produce same digest due to sorting

        let mut s3 =
            ScanSession::begin(WitnessLayer::LsmMemtable, target, ScanTechnique::Structural);
        s3.record_control_match();
        s3.record_unit([1u8; 32], false);
        s3.record_unit([3u8; 32], false); // unit changed

        let w1 = s1.finish(&clock).unwrap();
        let w2 = s2.finish(&clock).unwrap();
        let w3 = s3.finish(&clock).unwrap();

        assert_eq!(
            w1.digest(),
            w2.digest(),
            "Digests must match regardless of unit insertion order"
        );
        assert_ne!(
            w1.digest(),
            w3.digest(),
            "Digest must change when units change"
        );
    }
}
