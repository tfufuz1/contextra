//! File-backed KeyProvider for single-host deployments (SP-KEYS §3.8, SPEC §9.4).
#![forbid(unsafe_code)]

use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    ClusterId, DestroyBehavior, DestroyReceipt, Error, KekId, Key32, KeyProvider,
    KeyProviderInfo, KeyState,
};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const KEY_FILE_MAGIC: &[u8; 8] = b"CTXSKEY1";

#[derive(Default)]
struct FileClusterEntry {
    epochs: BTreeMap<u32, [u8; 32]>,
    destroyed: bool,
    disabled: bool,
}

pub struct FileKeyProvider {
    base_dir: PathBuf,
    entries: Mutex<BTreeMap<ClusterId, FileClusterEntry>>,
    keks: Mutex<BTreeMap<KekId, [u8; 32]>>,
}

fn cluster_hex(cluster: ClusterId) -> String {
    let mut out = String::with_capacity(32);
    for b in cluster.as_bytes() {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn parse_cluster_hex(name: &str) -> Option<ClusterId> {
    let stem = name.strip_suffix(".key")?;
    if stem.len() != 32 {
        return None;
    }
    let mut raw = [0u8; 16];
    for (i, byte) in raw.iter_mut().enumerate() {
        let part = stem.get(i * 2..i * 2 + 2)?;
        *byte = u8::from_str_radix(part, 16).ok()?;
    }
    Some(ClusterId::from_wire(raw))
}

fn key_file_path(base_dir: &Path, cluster: ClusterId) -> PathBuf {
    base_dir.join(format!("{}.key", cluster_hex(cluster)))
}

fn write_cluster_file(
    base_dir: &Path,
    cluster: ClusterId,
    epochs: &BTreeMap<u32, [u8; 32]>,
) -> Result<(), Error> {
    fs::create_dir_all(base_dir).map_err(|e| Error::Io {
        role: "keys",
        kind: e.kind(),
    })?;
    let tmp_path = base_dir.join(format!(".tmp.{}.key", cluster_hex(cluster)));
    let final_path = key_file_path(base_dir, cluster);

    let mut payload = Vec::with_capacity(12 + epochs.len() * 36);
    payload.extend_from_slice(KEY_FILE_MAGIC);
    payload.extend_from_slice(&(epochs.len() as u32).to_be_bytes());
    for (epoch, key) in epochs {
        payload.extend_from_slice(&epoch.to_be_bytes());
        payload.extend_from_slice(key);
    }

    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    opts.mode(0o600);
    let mut f = opts.open(&tmp_path).map_err(|e| Error::Io {
        role: "keys",
        kind: e.kind(),
    })?;
    f.write_all(&payload).map_err(|e| Error::Io {
        role: "keys",
        kind: e.kind(),
    })?;
    f.sync_all().map_err(|e| Error::Io {
        role: "keys",
        kind: e.kind(),
    })?;
    drop(f);

    fs::rename(&tmp_path, &final_path).map_err(|e| Error::Io {
        role: "keys",
        kind: e.kind(),
    })?;
    if let Ok(dir) = OpenOptions::new().read(true).open(base_dir) {
        let _ = dir.sync_all();
    }
    Ok(())
}

fn load_existing_entries(base_dir: &Path) -> BTreeMap<ClusterId, FileClusterEntry> {
    let mut map = BTreeMap::new();
    let Ok(rd) = fs::read_dir(base_dir) else {
        return map;
    };
    for entry in rd.flatten() {
        let fname = entry.file_name();
        let Some(name_str) = fname.to_str() else {
            continue;
        };
        let Some(cluster) = parse_cluster_hex(name_str) else {
            continue;
        };
        let Ok(mut f) = OpenOptions::new().read(true).open(entry.path()) else {
            continue;
        };
        let mut buf = Vec::new();
        if f.read_to_end(&mut buf).is_err() || buf.len() < 12 || &buf[..8] != KEY_FILE_MAGIC {
            continue;
        }
        let count = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]) as usize;
        if buf.len() != 12 + count * 36 {
            continue;
        }
        let mut epochs = BTreeMap::new();
        for i in 0..count {
            let off = 12 + i * 36;
            let epoch = u32::from_be_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]]);
            let mut key = [0u8; 32];
            key.copy_from_slice(&buf[off + 4..off + 36]);
            epochs.insert(epoch, key);
        }
        map.insert(
            cluster,
            FileClusterEntry {
                epochs,
                destroyed: false,
                disabled: false,
            },
        );
    }
    map
}

impl FileKeyProvider {
    pub fn new(base_dir: PathBuf) -> Self {
        let entries = load_existing_entries(&base_dir);
        Self {
            base_dir,
            entries: Mutex::new(entries),
            keks: Mutex::new(BTreeMap::new()),
        }
    }
}

impl KeyProvider for FileKeyProvider {
    fn info(&self) -> KeyProviderInfo {
        KeyProviderInfo {
            destroy: DestroyBehavior::Immediate,
            keys_in_backups: false,
        }
    }

    fn kek(&self, id: KekId) -> Result<Key32, Error> {
        let mut keks = self.keks.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider kek lock",
        })?;
        let entry = keks.entry(id).or_insert([0x42u8; 32]);
        Ok(Key32::from_array(*entry))
    }

    fn create_cluster_key(&self, cluster: ClusterId, entropy: &dyn Entropy) -> Result<u32, Error> {
        let mut candidate = [0u8; 32];
        entropy.fill(&mut candidate)?;

        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        let cluster_entry = entries.entry(cluster).or_default();
        if cluster_entry.destroyed {
            return Err(Error::Shredded);
        }
        if cluster_entry.disabled {
            return Err(Error::KeyUnavailable {
                retry_after_secs: None,
            });
        }
        if let Some(epoch) = cluster_entry.epochs.keys().next_back() {
            return Ok(*epoch);
        }
        cluster_entry.epochs.insert(1, candidate);
        write_cluster_file(&self.base_dir, cluster, &cluster_entry.epochs)?;
        Ok(1)
    }

    fn cluster_key(&self, cluster: ClusterId, epoch: u32) -> Result<Option<Key32>, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        if let Some(cluster_entry) = entries.get(&cluster) {
            if cluster_entry.disabled {
                return Err(Error::KeyUnavailable {
                    retry_after_secs: None,
                });
            }
            if cluster_entry.destroyed {
                return Ok(None);
            }
            return Ok(cluster_entry.epochs.get(&epoch).map(|k| Key32::from_array(*k)));
        }
        Ok(None)
    }

    fn probe(&self, cluster: ClusterId) -> Result<KeyState, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        Ok(match entries.get(&cluster) {
            Some(e) if e.destroyed => KeyState::Absent,
            Some(e) if e.disabled => KeyState::Disabled,
            Some(e) if !e.epochs.is_empty() => KeyState::Present,
            _ => KeyState::Absent,
        })
    }

    fn list(&self) -> Result<Vec<ClusterId>, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        Ok(entries
            .iter()
            .filter(|(_, e)| !e.destroyed && !e.epochs.is_empty())
            .map(|(c, _)| *c)
            .collect())
    }

    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<DestroyReceipt, Error> {
        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        let mut absent = 0;
        for c in clusters {
            let entry = entries.entry(*c).or_default();
            entry.epochs.clear();
            entry.destroyed = true;
            let path = key_file_path(&self.base_dir, *c);
            match fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    return Err(Error::Io {
                        role: "keys",
                        kind: e.kind(),
                    })
                }
            }
            absent += 1;
        }
        if self.base_dir.exists() {
            if let Ok(dir) = OpenOptions::new().read(true).open(&self.base_dir) {
                dir.sync_all().map_err(|e| Error::Io {
                    role: "keys",
                    kind: e.kind(),
                })?;
            }
        }
        Ok(DestroyReceipt {
            requested_clusters: clusters.len(),
            absent_clusters: absent,
        })
    }
}
