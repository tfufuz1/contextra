#![cfg(feature = "testing")]

use contextra_crypto_ref::mem_key_provider::MemKeyProvider;
use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    BatchId, ClusterId, DestroyBehavior, Error, KekId, KeyAssurance, KeyProvider, KeyState,
    KeysInBackups,
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
    let entropy: Arc<dyn Entropy> = Arc::new(TestEntropy(AtomicU64::new(1)));
    let provider = MemKeyProvider::new(Arc::clone(&entropy));
    assert_eq!(provider.info().destroy, DestroyBehavior::Immediate);
    assert_eq!(provider.info().keys_in_backups, KeysInBackups::No);
    assert_eq!(provider.info().assurance, KeyAssurance::Kms);

    let kek_id = KekId::from_bytes([3; 16]);
    let first_kek = provider.kek(kek_id)?;
    let second_kek = provider.kek(kek_id)?;
    assert_eq!(first_kek.expose(), second_kek.expose());
    assert!(!format!("{first_kek:?}").contains("["));

    let cluster = ClusterId::from_wire([4; 16]);
    assert_eq!(provider.probe(cluster, 1)?, KeyState::Absent);

    let batch_id = BatchId::from_bytes([9; 16]);
    let res1 = provider.create_cluster_keys(batch_id, &[cluster], entropy.as_ref())?;
    let epoch = res1[0].result.as_ref().copied().map_err(|_| Error::Corrupt { place: "batch" })?;
    // Idempotent per BatchId
    let res2 = provider.create_cluster_keys(batch_id, &[cluster], entropy.as_ref())?;
    assert_eq!(res2[0].result.as_ref().copied().ok(), Some(epoch));
    assert_eq!(provider.probe(cluster, epoch)?, KeyState::Present);

    let listed = provider.list()?;
    assert!(listed.iter().any(|e| e.cluster == cluster && e.created_utc > 0));
    assert!(provider.cluster_key(cluster, epoch)?.is_some());

    provider.set_disabled(cluster, true)?;
    assert_eq!(provider.probe(cluster, epoch)?, KeyState::Disabled);
    assert!(matches!(provider.cluster_key(cluster, epoch), Err(Error::KeyUnavailable { .. })));
    provider.set_disabled(cluster, false)?;

    // INV-KEY-4: destroy_many removes every epoch, including older retained keys.
    provider.insert_epoch_for_test(cluster, epoch.saturating_add(1), [5; 32])?;
    assert_eq!(provider.probe(cluster, epoch.saturating_add(1))?, KeyState::Present);
    let receipt = provider.destroy_many(&[cluster])?;
    assert_eq!(receipt.requested_clusters, 1);
    assert_eq!(receipt.absent_clusters, 1);
    assert_eq!(provider.probe(cluster, epoch)?, KeyState::Absent);
    assert_eq!(provider.probe(cluster, epoch.saturating_add(1))?, KeyState::Absent);
    assert!(provider.cluster_key(cluster, epoch)?.is_none());
    assert!(provider.cluster_key(cluster, epoch.saturating_add(1))?.is_none());
    assert!(!provider.list()?.iter().any(|e| e.cluster == cluster));
    assert_eq!(provider.destroy_many(&[cluster])?, receipt);

    // Tomb epoch lifecycle (K-31, INV-KEY-5): missing != retired
    assert!(matches!(provider.tomb_key(0), Err(Error::KeyLost)));
    provider.create_tomb_epoch(0)?;
    provider.create_tomb_epoch(0)?; // idempotent
    let tk = provider.tomb_key(0)?.expect("open tomb key must exist");
    assert_eq!(provider.tomb_key(0)?.expect("stable").expose(), tk.expose());
    assert_eq!(provider.retire_tomb_epoch(0)?, KeyState::Absent);
    assert_eq!(provider.retire_tomb_epoch(0)?, KeyState::Absent); // idempotent
    assert!(provider.tomb_key(0)?.is_none());
    Ok(())
}
