// FILE-CONTEXT
// ZWECK: Compile-Time Objektsicherheits- und Bound-Tests für contract.rs (WP-VEC-02).
// INVARIANTEN: Zero-Panic, dyn-Sicherheit für DistanceFn, BlockTransform, Clock, Rng, Send + Sync.
// HOTSPOTS: assert_object_safety
// STAND: 2026-11-11

use contextra_vector::contract::{
    BlockTransform, Clock, DistanceFn, Error as ContractError, ExternalId, Metric, Result, Rng,
    ScoredPoint,
};

#[allow(dead_code)]
fn assert_distance_fn_object_safety(_: &dyn DistanceFn) {}

#[allow(dead_code)]
fn assert_block_transform_object_safety(_: &dyn BlockTransform) {}

#[allow(dead_code)]
fn assert_clock_object_safety(_: &dyn Clock) {}

#[allow(dead_code)]
fn assert_rng_object_safety(_: &mut dyn Rng) {}

fn assert_send_sync<T: Send + Sync>() {}
fn assert_send<T: Send>() {}

#[test]
fn test_contract_traits_object_safety_and_bounds() {
    assert_send_sync::<Box<dyn DistanceFn>>();
    assert_send_sync::<Box<dyn BlockTransform>>();
    assert_send_sync::<Box<dyn Clock>>();
    assert_send::<Box<dyn Rng>>();

    assert_send_sync::<ExternalId>();
    assert_send_sync::<Metric>();
    assert_send_sync::<ScoredPoint>();
    assert_send_sync::<ContractError>();

    // Test Instanziierung / Konvertierung ExternalId
    let ext_id_u64: ExternalId = 42_u64.into();
    assert_eq!(ext_id_u64.0, 42_u128);
    let ext_id = ExternalId(100);
    assert_eq!(ext_id, ExternalId(100));

    // Test Metric & ScoredPoint
    let sp = ScoredPoint {
        id: ext_id,
        score: 0.95,
    };
    assert_eq!(sp.id, ExternalId(100));
    assert_eq!(sp.score, 0.95);
}

struct DummyDist;
impl DistanceFn for DummyDist {
    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::NAN;
        }
        0.0
    }
}

struct DummyTransform;
impl BlockTransform for DummyTransform {
    fn transform(&self, block: &mut [u8]) -> Result<()> {
        for b in block.iter_mut() {
            *b ^= 0xAA;
        }
        Ok(())
    }
    fn inverse_transform(&self, block: &mut [u8]) -> Result<()> {
        for b in block.iter_mut() {
            *b ^= 0xAA;
        }
        Ok(())
    }
}

struct DummyClock;
impl Clock for DummyClock {
    fn now_unix_ms(&self) -> u64 {
        1700000000000
    }
}

struct DummyRng(u64);
impl Rng for DummyRng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(1);
        self.0
    }
}

#[test]
fn test_dummy_implementations_via_dyn_dispatch() {
    let dist: &dyn DistanceFn = &DummyDist;
    assert_eq!(dist.distance(&[1.0, 2.0], &[1.0, 2.0]), 0.0);

    let transform: &dyn BlockTransform = &DummyTransform;
    let mut data = [1, 2, 3, 4];
    transform.transform(&mut data).expect("transform");
    assert_eq!(data, [1 ^ 0xAA, 2 ^ 0xAA, 3 ^ 0xAA, 4 ^ 0xAA]);
    transform.inverse_transform(&mut data).expect("inverse");
    assert_eq!(data, [1, 2, 3, 4]);

    let clock: &dyn Clock = &DummyClock;
    assert_eq!(clock.now_unix_ms(), 1700000000000);

    let mut rng = DummyRng(10);
    let rng_dyn: &mut dyn Rng = &mut rng;
    assert_eq!(rng_dyn.next_u64(), 11);
    assert_eq!(rng_dyn.next_u64(), 12);
}
