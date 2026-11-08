//! Tests für Scope-Codec und persistente Scope-Dateien (SPEC §11.1, D-75, D-85, Anhang F.1, F.9).
//!
//! T-ID: T-CRASH-1, T-ERID-1

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use contextra_erasure::scope::codec::{decode_scope, encode_scope};
use contextra_erasure::scope::set::{LayerScopeSet, ResolvedLayerScope};
use contextra_erasure::scope::{load_scope_file, store_scope_file};
use contextra_erasure::types_local::Vfs;
use contextra_erasure::types_local::{FileId, FileKey, FileKeyring, FileKind};
use futures_util::future::BoxFuture;
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct MockKeyring {
    keys: RwLock<BTreeMap<u64, Vec<u8>>>,
}

impl MockKeyring {
    fn new() -> Self {
        Self {
            keys: RwLock::new(BTreeMap::new()),
        }
    }
}

impl FileKeyring for MockKeyring {
    fn create(&self, _kind: FileKind, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move {
            let key_bytes = vec![0x42u8; 32];
            self.keys.write().insert(id.as_u64(), key_bytes.clone());
            Ok(FileKey::new(key_bytes, id))
        })
    }

    fn open(&self, id: FileId) -> BoxFuture<'_, Result<FileKey, String>> {
        Box::pin(async move {
            let guard = self.keys.read();
            if let Some(k) = guard.get(&id.as_u64()) {
                Ok(FileKey::new(k.clone(), id))
            } else {
                Err("Key not found in keyring".to_string())
            }
        })
    }

    fn destroy(&self, id: FileId) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            self.keys.write().remove(&id.as_u64());
            Ok(())
        })
    }
}

struct MemVfs {
    files: RwLock<BTreeMap<PathBuf, Vec<u8>>>,
}

impl MemVfs {
    fn new() -> Self {
        Self {
            files: RwLock::new(BTreeMap::new()),
        }
    }
}

impl Vfs for MemVfs {
    fn read(
        &self,
        path: &Path,
    ) -> std::result::Result<Vec<u8>, contextra_erasure::error::ErasureError> {
        let guard = self.files.read();
        guard.get(path).cloned().ok_or_else(|| {
            contextra_erasure::error::ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                path.display().to_string(),
            ))
        })
    }

    fn write_atomic(
        &self,
        path: &Path,
        data: &[u8],
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        self.files.write().insert(path.to_path_buf(), data.to_vec());
        Ok(())
    }

    fn create_dir_all(
        &self,
        _path: &Path,
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        Ok(())
    }

    fn remove_file(
        &self,
        path: &Path,
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        self.files.write().remove(path);
        Ok(())
    }

    fn exists(
        &self,
        path: &Path,
    ) -> std::result::Result<bool, contextra_erasure::error::ErasureError> {
        Ok(self.files.read().contains_key(path))
    }

    fn len(&self, path: &Path) -> std::result::Result<u64, contextra_erasure::error::ErasureError> {
        let guard = self.files.read();
        guard.get(path).map(|v| v.len() as u64).ok_or_else(|| {
            contextra_erasure::error::ErasureError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                path.display().to_string(),
            ))
        })
    }

    fn list_dir(
        &self,
        _path: &Path,
    ) -> std::result::Result<Vec<String>, contextra_erasure::error::ErasureError> {
        Ok(Vec::new())
    }

    fn sync_dir(
        &self,
        _path: &Path,
    ) -> std::result::Result<(), contextra_erasure::error::ErasureError> {
        Ok(())
    }
}

/// T-ID: T-CRASH-1
#[test]
fn t_crash_1_scope_binary_codec_roundtrip() {
    let mut set = LayerScopeSet::new();
    set.push(ResolvedLayerScope::with_units([0x01; 16], vec![10, 20, 30]));
    set.push(ResolvedLayerScope::with_units([0x02; 16], vec![100, 200]));

    let encoded = encode_scope(&set).expect("encode scope binary");
    // Binary format shouldn't be JSON! Must not start with '{'
    assert_ne!(encoded.first(), Some(&b'{'));

    let decoded = decode_scope(&encoded).expect("decode scope binary");
    assert_eq!(decoded.scopes.len(), 2);
    assert_eq!(decoded.scopes[0].layer_id, [0x01; 16]);
    assert_eq!(decoded.scopes[0].units, vec![10, 20, 30]);
    assert_eq!(decoded.scopes[1].layer_id, [0x02; 16]);
    assert_eq!(decoded.scopes[1].units, vec![100, 200]);
}

/// T-ID: T-ERID-1
#[tokio::test]
async fn t_erid_1_scope_file_persistence_threshold() {
    let vfs = Arc::new(MemVfs::new());
    let keyring = Arc::new(MockKeyring::new());

    // Kleiner Scope: <= 16 Einheiten -> is_embeddable == true
    let small_scope = ResolvedLayerScope::with_units([0xaa; 16], vec![1; 16]);
    assert!(small_scope.is_embeddable());

    // Großer Scope: > 16 Einheiten -> is_embeddable == false -> speichere in scope/<erasure_id>.scp
    let large_units: Vec<u64> = (1..=20).collect();
    let mut large_set = LayerScopeSet::new();
    large_set.push(ResolvedLayerScope::with_units([0xbb; 16], large_units));
    assert!(!large_set.is_embeddable());

    let erasure_id = [0x55u8; 16];
    let file_id = FileId::new(999);

    store_scope_file(
        vfs.as_ref(),
        keyring.as_ref(),
        &erasure_id,
        file_id,
        &large_set,
    )
    .await
    .expect("store scope file");

    // Prüfe Pfad: scope/5555...55.scp (D-75: nur Tags/ErasureId)
    let hex_id = erasure_id
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    let expected_path = PathBuf::from(format!("scope/{}.scp", hex_id));
    assert!(vfs.exists(&expected_path).expect("exists"));

    // Lese Datei wieder ein
    let loaded = load_scope_file(vfs.as_ref(), keyring.as_ref(), &erasure_id, file_id)
        .await
        .expect("load scope file");
    assert_eq!(loaded.total_units(), 20);
    assert_eq!(loaded.scopes[0].units.len(), 20);
}
