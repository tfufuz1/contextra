#![cfg(feature = "testing")]

use contextra_crypto_ref::mem_key_provider::MemKeyProvider;
use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    ClusterId, DestroyBehavior, Error, KekId, KeyProvider, KeyState,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct TestEntropy(AtomicU64);

impl Entropy for TestEntropy {
    fn fill(&self, output: &mut [u8]) -> Result<(), Error> {
        let mut next = self.0.fetch_add(1, Ordering::AcqRel);
        for byte in output {
            *byte = next as u8;
            next = next.wrapping_add(1);
        }
        Ok(())
    }
}

/// KeyProviderConformance: exercises the normative §9.4 provider contract.
#[test]
fn memory_provider_conforms_to_spec_scope_and_destroy_semantics() -> Result<(), Error> {
    let entropy = Arc::new(TestEntropy(AtomicU64::new(1)));
    let provider = MemKeyProvider::new(Arc::clone(&entropy));
    assert_eq!(provider.info().destroy, DestroyBehavior::Immediate);
    assert!(!provider.info().keys_in_backups);

    let kek_id = KekId::from_bytes([3; 16]);
    let first_kek = provider.kek(kek_id)?;
    let second_kek = provider.kek(kek_id)?;
    assert_eq!(first_kek.expose(), second_kek.expose());
    assert!(!format!("{first_kek:?}").contains("["));

    let cluster = ClusterId::from_wire([4; 16]);
    assert_eq!(provider.probe(cluster)?, KeyState::Absent);
    let epoch = provider.create_cluster_key(cluster, entropy.as_ref())?;
    assert_eq!(provider.create_cluster_key(cluster, entropy.as_ref())?, epoch);
    assert_eq!(provider.probe(cluster)?, KeyState::Present);
    assert!(provider.list()?.contains(&cluster));
    assert!(provider.cluster_key(cluster, epoch)?.is_some());

    provider.set_disabled(cluster, true)?;
    assert_eq!(provider.probe(cluster)?, KeyState::Disabled);
    assert!(matches!(provider.cluster_key(cluster, epoch), Err(Error::KeyUnavailable { .. })));
    provider.set_disabled(cluster, false)?;

    // INV-KEY-4: destroy_many removes every epoch, including older retained keys.
    provider.insert_epoch_for_test(cluster, epoch.saturating_add(1), [5; 32])?;
    let receipt = provider.destroy_many(&[cluster])?;
    assert_eq!(receipt.requested_clusters, 1);
    assert_eq!(receipt.absent_clusters, 1);
    assert_eq!(provider.probe(cluster)?, KeyState::Absent);
    assert!(provider.cluster_key(cluster, epoch)?.is_none());
    assert!(provider.cluster_key(cluster, epoch.saturating_add(1))?.is_none());
    assert!(!provider.list()?.contains(&cluster));
    assert_eq!(provider.destroy_many(&[cluster])?, receipt);
    Ok(())
}
