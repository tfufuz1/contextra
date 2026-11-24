//! File-backed KeyProvider for single-host deployments (SP-KEYS §3.8, SPEC §3.7.4, §9.4).
//!
//! Uses Argon2id (m, t, p persisted in header) to derive a master wrapping key
//! from a caller-supplied passphrase, and wraps KEKs, cluster keys, and tomb
//! keys with XChaCha20-Poly1305. Cluster keys are never stored in plaintext.
#![forbid(unsafe_code)]

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{AeadInPlace, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use contextra_storage_ports::traits::Entropy;
use contextra_storage_ports::{
    BatchId, ClusterId, DestroyBehavior, DestroyReceipt, Error, KekId, Key32, KeyAssurance,
    KeyCreateResult, KeyListEntry, KeyLookup, KeyProvider, KeyProviderInfo, KeyState,
    KeysInBackups, KEY_PROVIDER_BATCH_MAX,
};
use std::collections::{BTreeMap, HashMap};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroizing;

const HEADER_MAGIC: &[u8; 8] = b"CTXKFH01";
const KEY_FILE_MAGIC: &[u8; 8] = b"CTXSKEY2";
const KEK_FILE_MAGIC: &[u8; 8] = b"CTXSKEK1";
const TOMB_FILE_MAGIC: &[u8; 8] = b"CTXSTMB1";

const DEFAULT_M_COST_KIB: u32 = 19_456;
const DEFAULT_T_COST: u32 = 2;
const DEFAULT_P_COST: u32 = 1;

#[derive(Debug, Clone, Copy)]
pub struct Argon2Params {
    pub m_cost_kib: u32,
    pub t_cost: u32,
    pub p_cost: u32,
}

impl Default for Argon2Params {
    fn default() -> Self {
        Self {
            m_cost_kib: DEFAULT_M_COST_KIB,
            t_cost: DEFAULT_T_COST,
            p_cost: DEFAULT_P_COST,
        }
    }
}

#[derive(Default)]
struct FileClusterEntry {
    epochs: BTreeMap<u32, Zeroizing<[u8; 32]>>,
    created_utc: u64,
    destroyed: bool,
    disabled: bool,
}

enum FileTombEntry {
    Open(Zeroizing<[u8; 32]>),
    Retired,
}

pub struct FileKeyProvider {
    base_dir: PathBuf,
    master_key: Zeroizing<[u8; 32]>,
    entries: Mutex<BTreeMap<ClusterId, FileClusterEntry>>,
    keks: Mutex<BTreeMap<KekId, Zeroizing<[u8; 32]>>>,
    tombs: Mutex<BTreeMap<u32, FileTombEntry>>,
    batches: Mutex<HashMap<BatchId, Vec<(ClusterId, u32)>>>,
    utc_override: AtomicU64,
}

fn fill_random(buf: &mut [u8]) -> Result<(), Error> {
    getrandom::fill(buf).map_err(|_| Error::Io {
        role: "keys",
        kind: std::io::ErrorKind::Other,
    })
}

fn current_utc_secs(override_secs: &AtomicU64) -> u64 {
    let ov = override_secs.load(Ordering::Acquire);
    if ov > 0 {
        return ov;
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1)
        .max(1)
}

fn derive_master_key(passphrase: &[u8], salt: &[u8; 16], p: Argon2Params) -> Result<Zeroizing<[u8; 32]>, Error> {
    if passphrase.is_empty() {
        return Err(Error::Limit {
            limit: "passphrase must not be empty",
        });
    }
    let params = Params::new(p.m_cost_kib, p.t_cost, p.p_cost, Some(32)).map_err(|_| Error::Limit {
        limit: "invalid Argon2id parameters",
    })?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = Zeroizing::new([0u8; 32]);
    argon2
        .hash_password_into(passphrase, salt, &mut *out)
        .map_err(|_| Error::Corrupt {
            place: "argon2id key derivation",
        })?;
    Ok(out)
}

fn seal_blob(master: &[u8; 32], aad: &[u8], pt: &[u8]) -> Result<Vec<u8>, Error> {
    let cipher = XChaCha20Poly1305::new_from_slice(master)
        .map_err(|_| Error::Corrupt { place: "file key cipher init" })?;
    let mut nonce = [0u8; 24];
    fill_random(&mut nonce)?;
    let mut buf = Zeroizing::new(pt.to_vec());
    let tag = cipher
        .encrypt_in_place_detached(XNonce::from_slice(&nonce), aad, &mut buf)
        .map_err(|_| Error::Corrupt { place: "file key seal" })?;
    let mut out = Vec::with_capacity(24 + buf.len() + 16);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&buf);
    out.extend_from_slice(&tag);
    Ok(out)
}

fn open_blob(master: &[u8; 32], aad: &[u8], ct: &[u8]) -> Result<Zeroizing<Vec<u8>>, Error> {
    if ct.len() < 24 + 16 {
        return Err(Error::Corrupt { place: "wrapped file key length" });
    }
    let cipher = XChaCha20Poly1305::new_from_slice(master)
        .map_err(|_| Error::Corrupt { place: "file key cipher init" })?;
    let nonce = XNonce::from_slice(&ct[..24]);
    let tag_start = ct.len() - 16;
    let mut pt = Zeroizing::new(ct[24..tag_start].to_vec());
    let tag = chacha20poly1305::Tag::from_slice(&ct[tag_start..]);
    cipher
        .decrypt_in_place_detached(nonce, aad, &mut pt, tag)
        .map_err(|_| Error::Corrupt { place: "wrapped file key auth" })?;
    Ok(pt)
}

fn atomic_write_private(dir: &Path, tmp_name: &str, final_name: &str, bytes: &[u8]) -> Result<(), Error> {
    fs::create_dir_all(dir).map_err(|e| Error::Io {
        role: "keys",
        kind: e.kind(),
    })?;
    let tmp_path = dir.join(tmp_name);
    let final_path = dir.join(final_name);
    let mut opts = OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    opts.mode(0o600);
    let mut f = opts.open(&tmp_path).map_err(|e| Error::Io {
        role: "keys",
        kind: e.kind(),
    })?;
    f.write_all(bytes).map_err(|e| Error::Io {
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
    if let Ok(d) = OpenOptions::new().read(true).open(dir) {
        let _ = d.sync_all();
    }
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn parse_hex<const N: usize>(stem: &str) -> Option<[u8; N]> {
    if stem.len() != N * 2 {
        return None;
    }
    let mut raw = [0u8; N];
    for (i, byte) in raw.iter_mut().enumerate() {
        let part = stem.get(i * 2..i * 2 + 2)?;
        *byte = u8::from_str_radix(part, 16).ok()?;
    }
    Some(raw)
}

fn load_or_init_master(
    base_dir: &Path,
    passphrase: &[u8],
    params: Argon2Params,
) -> Result<Zeroizing<[u8; 32]>, Error> {
    fs::create_dir_all(base_dir).map_err(|e| Error::Io {
        role: "keys",
        kind: e.kind(),
    })?;
    let header_path = base_dir.join("PROVIDER-HEADER");
    match OpenOptions::new().read(true).open(&header_path) {
        Ok(mut f) => {
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).map_err(|e| Error::Io {
                role: "keys",
                kind: e.kind(),
            })?;
            // 8 magic + 4*3 params + 16 salt + wrapped 32-byte canary (24+32+16 = 72) = 108
            if buf.len() != 108 || &buf[..8] != HEADER_MAGIC {
                return Err(Error::Corrupt { place: "keys provider header" });
            }
            let p = Argon2Params {
                m_cost_kib: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
                t_cost: u32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]),
                p_cost: u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]),
            };
            let salt: [u8; 16] = buf[20..36].try_into().map_err(|_| Error::Corrupt { place: "keys header salt" })?;
            let master = derive_master_key(passphrase, &salt, p)?;
            let check = open_blob(&master, b"ctxs-file-provider-v1", &buf[36..])?;
            if check.as_slice() != b"CTXS-MASTER-VERIFIED-V1-00000000" {
                return Err(Error::Corrupt { place: "keys passphrase check" });
            }
            Ok(master)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            let mut salt = [0u8; 16];
            fill_random(&mut salt)?;
            let master = derive_master_key(passphrase, &salt, params)?;
            let check = seal_blob(&master, b"ctxs-file-provider-v1", b"CTXS-MASTER-VERIFIED-V1-00000000")?;
            let mut out = Vec::with_capacity(108);
            out.extend_from_slice(HEADER_MAGIC);
            out.extend_from_slice(&params.m_cost_kib.to_be_bytes());
            out.extend_from_slice(&params.t_cost.to_be_bytes());
            out.extend_from_slice(&params.p_cost.to_be_bytes());
            out.extend_from_slice(&salt);
            out.extend_from_slice(&check);
            atomic_write_private(base_dir, ".tmp.PROVIDER-HEADER", "PROVIDER-HEADER", &out)?;
            Ok(master)
        }
        Err(e) => Err(Error::Io {
            role: "keys",
            kind: e.kind(),
        }),
    }
}

fn write_cluster_file(
    base_dir: &Path,
    master: &[u8; 32],
    cluster: ClusterId,
    created_utc: u64,
    epochs: &BTreeMap<u32, Zeroizing<[u8; 32]>>,
) -> Result<(), Error> {
    let mut pt = Zeroizing::new(Vec::with_capacity(12 + epochs.len() * 36));
    pt.extend_from_slice(&created_utc.to_be_bytes());
    pt.extend_from_slice(&(epochs.len() as u32).to_be_bytes());
    for (epoch, key) in epochs {
        pt.extend_from_slice(&epoch.to_be_bytes());
        pt.extend_from_slice(&**key);
    }
    let mut aad = Vec::with_capacity(20);
    aad.extend_from_slice(b"cluster:");
    aad.extend_from_slice(cluster.as_bytes());
    let wrapped = seal_blob(master, &aad, &pt)?;
    let mut out = Vec::with_capacity(8 + wrapped.len());
    out.extend_from_slice(KEY_FILE_MAGIC);
    out.extend_from_slice(&wrapped);
    let hex = hex_encode(cluster.as_bytes());
    atomic_write_private(base_dir, &format!(".tmp.{hex}.key"), &format!("{hex}.key"), &out)
}

fn write_kek_file(base_dir: &Path, master: &[u8; 32], id: KekId, key: &[u8; 32]) -> Result<(), Error> {
    let mut aad = Vec::with_capacity(20);
    aad.extend_from_slice(b"kek:");
    aad.extend_from_slice(id.as_bytes());
    let wrapped = seal_blob(master, &aad, key)?;
    let mut out = Vec::with_capacity(8 + wrapped.len());
    out.extend_from_slice(KEK_FILE_MAGIC);
    out.extend_from_slice(&wrapped);
    let hex = hex_encode(id.as_bytes());
    atomic_write_private(base_dir, &format!(".tmp.{hex}.kek"), &format!("{hex}.kek"), &out)
}

fn write_tomb_file(base_dir: &Path, master: &[u8; 32], epoch: u32, state: &FileTombEntry) -> Result<(), Error> {
    let mut pt = Zeroizing::new(Vec::with_capacity(33));
    match state {
        FileTombEntry::Open(k) => {
            pt.push(1);
            pt.extend_from_slice(&**k);
        }
        FileTombEntry::Retired => {
            pt.push(2);
        }
    }
    let mut aad = Vec::with_capacity(9);
    aad.extend_from_slice(b"tomb:");
    aad.extend_from_slice(&epoch.to_be_bytes());
    let wrapped = seal_blob(master, &aad, &pt)?;
    let mut out = Vec::with_capacity(8 + wrapped.len());
    out.extend_from_slice(TOMB_FILE_MAGIC);
    out.extend_from_slice(&wrapped);
    atomic_write_private(base_dir, &format!(".tmp.{epoch:08x}.tomb"), &format!("{epoch:08x}.tomb"), &out)
}

fn load_existing(
    base_dir: &Path,
    master: &[u8; 32],
) -> Result<(BTreeMap<ClusterId, FileClusterEntry>, BTreeMap<KekId, Zeroizing<[u8; 32]>>, BTreeMap<u32, FileTombEntry>), Error> {
    let mut clusters = BTreeMap::new();
    let mut keks = BTreeMap::new();
    let mut tombs = BTreeMap::new();
    let Ok(rd) = fs::read_dir(base_dir) else {
        return Ok((clusters, keks, tombs));
    };
    for entry in rd.flatten() {
        let fname = entry.file_name();
        let Some(name) = fname.to_str() else { continue };
        if name.starts_with(".tmp.") { continue; }
        let mut buf = Vec::new();
        let Ok(mut f) = OpenOptions::new().read(true).open(entry.path()) else { continue };
        if f.read_to_end(&mut buf).is_err() { continue; }

        if let Some(stem) = name.strip_suffix(".key") {
            let Some(raw) = parse_hex::<16>(stem) else { continue };
            if buf.len() < 8 || &buf[..8] != KEY_FILE_MAGIC {
                return Err(Error::Corrupt { place: "cluster key file header" });
            }
            let cluster = ClusterId::from_wire(raw);
            let mut aad = Vec::with_capacity(24);
            aad.extend_from_slice(b"cluster:");
            aad.extend_from_slice(cluster.as_bytes());
            let pt = open_blob(master, &aad, &buf[8..])?;
            if pt.len() < 12 {
                return Err(Error::Corrupt { place: "cluster key payload" });
            }
            let created_utc = u64::from_be_bytes([pt[0], pt[1], pt[2], pt[3], pt[4], pt[5], pt[6], pt[7]]);
            let count = u32::from_be_bytes([pt[8], pt[9], pt[10], pt[11]]) as usize;
            if pt.len() != 12 + count * 36 {
                return Err(Error::Corrupt { place: "cluster key payload count" });
            }
            let mut epochs = BTreeMap::new();
            for i in 0..count {
                let off = 12 + i * 36;
                let epoch = u32::from_be_bytes([pt[off], pt[off + 1], pt[off + 2], pt[off + 3]]);
                let mut key = Zeroizing::new([0u8; 32]);
                key.copy_from_slice(&pt[off + 4..off + 36]);
                epochs.insert(epoch, key);
            }
            clusters.insert(
                cluster,
                FileClusterEntry {
                    epochs,
                    created_utc,
                    destroyed: false,
                    disabled: false,
                },
            );
        } else if let Some(stem) = name.strip_suffix(".kek") {
            let Some(raw) = parse_hex::<16>(stem) else { continue };
            if buf.len() < 8 || &buf[..8] != KEK_FILE_MAGIC {
                return Err(Error::Corrupt { place: "kek file header" });
            }
            let id = KekId::from_bytes(raw);
            let mut aad = Vec::with_capacity(20);
            aad.extend_from_slice(b"kek:");
            aad.extend_from_slice(id.as_bytes());
            let pt = open_blob(master, &aad, &buf[8..])?;
            let raw_key: [u8; 32] = pt.as_slice().try_into().map_err(|_| Error::Corrupt { place: "kek payload length" })?;
            keks.insert(id, Zeroizing::new(raw_key));
        } else if let Some(stem) = name.strip_suffix(".tomb") {
            let Ok(epoch) = u32::from_str_radix(stem, 16) else { continue };
            if buf.len() < 8 || &buf[..8] != TOMB_FILE_MAGIC {
                return Err(Error::Corrupt { place: "tomb file header" });
            }
            let mut aad = Vec::with_capacity(9);
            aad.extend_from_slice(b"tomb:");
            aad.extend_from_slice(&epoch.to_be_bytes());
            let pt = open_blob(master, &aad, &buf[8..])?;
            match pt.first().copied() {
                Some(1) if pt.len() == 33 => {
                    let mut raw = Zeroizing::new([0u8; 32]);
                    raw.copy_from_slice(&pt[1..33]);
                    tombs.insert(epoch, FileTombEntry::Open(raw));
                }
                Some(2) if pt.len() == 1 => {
                    tombs.insert(epoch, FileTombEntry::Retired);
                }
                _ => return Err(Error::Corrupt { place: "tomb file payload" }),
            }
        }
    }
    Ok((clusters, keks, tombs))
}

impl FileKeyProvider {
    /// Opens or initializes a passphrase-protected FileKeyProvider using Argon2id (SPEC §3.7.4).
    pub fn open(base_dir: PathBuf, passphrase: &[u8]) -> Result<Self, Error> {
        Self::open_with_params(base_dir, passphrase, Argon2Params::default())
    }

    pub fn open_with_params(
        base_dir: PathBuf,
        passphrase: &[u8],
        params: Argon2Params,
    ) -> Result<Self, Error> {
        let master_key = load_or_init_master(&base_dir, passphrase, params)?;
        let (clusters, keks, tombs) = load_existing(&base_dir, &master_key)?;
        Ok(Self {
            base_dir,
            master_key,
            entries: Mutex::new(clusters),
            keks: Mutex::new(keks),
            tombs: Mutex::new(tombs),
            batches: Mutex::new(HashMap::new()),
            utc_override: AtomicU64::new(0),
        })
    }

    pub fn set_utc_secs_for_test(&self, utc: u64) {
        self.utc_override.store(utc, Ordering::Release);
    }
}

impl KeyProvider for FileKeyProvider {
    fn info(&self) -> KeyProviderInfo {
        KeyProviderInfo {
            destroy: DestroyBehavior::Immediate,
            keys_in_backups: KeysInBackups::No,
            assurance: KeyAssurance::FilePassphrase,
        }
    }

    fn kek(&self, id: KekId) -> Result<Key32, Error> {
        let mut keks = self.keks.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider kek lock",
        })?;
        if let Some(existing) = keks.get(&id) {
            return Ok(Key32::from_array(**existing));
        }
        let mut raw = Zeroizing::new([0u8; 32]);
        fill_random(&mut *raw)?;
        write_kek_file(&self.base_dir, &self.master_key, id, &raw)?;
        keks.insert(id, Zeroizing::new(*raw));
        Ok(Key32::from_array(*raw))
    }

    fn create_cluster_keys(
        &self,
        request_id: BatchId,
        clusters: &[ClusterId],
        entropy: &dyn Entropy,
    ) -> Result<Vec<KeyCreateResult>, Error> {
        if clusters.len() > KEY_PROVIDER_BATCH_MAX {
            return Err(Error::Limit { limit: "key provider batch exceeds 256" });
        }
        if let Some(prev) = self
            .batches
            .lock()
            .map_err(|_| Error::Poisoned { cause: "FileKeyProvider batch lock" })?
            .get(&request_id)
            .cloned()
        {
            return Ok(prev
                .into_iter()
                .map(|(cluster, epoch)| KeyCreateResult { cluster, result: Ok(epoch) })
                .collect());
        }

        let now_utc = current_utc_secs(&self.utc_override);
        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        let mut results = Vec::with_capacity(clusters.len());
        let mut recorded = Vec::new();
        let mut all_ok = true;

        for &cluster in clusters {
            let entry = entries.entry(cluster).or_default();
            if entry.destroyed {
                all_ok = false;
                results.push(KeyCreateResult { cluster, result: Err(Error::Shredded) });
                continue;
            }
            if entry.disabled {
                all_ok = false;
                results.push(KeyCreateResult {
                    cluster,
                    result: Err(Error::KeyUnavailable { retry_after_secs: None }),
                });
                continue;
            }
            let epoch = if let Some(&existing) = entry.epochs.keys().next_back() {
                existing
            } else {
                let mut candidate = Zeroizing::new([0u8; 32]);
                entropy.fill(&mut *candidate)?;
                entry.epochs.insert(1, candidate);
                if entry.created_utc == 0 {
                    entry.created_utc = now_utc;
                }
                write_cluster_file(&self.base_dir, &self.master_key, cluster, entry.created_utc, &entry.epochs)?;
                1
            };
            recorded.push((cluster, epoch));
            results.push(KeyCreateResult { cluster, result: Ok(epoch) });
        }

        if all_ok {
            self.batches
                .lock()
                .map_err(|_| Error::Poisoned { cause: "FileKeyProvider batch lock" })?
                .insert(request_id, recorded);
        }
        Ok(results)
    }

    fn cluster_keys(&self, req: &[(ClusterId, u32)]) -> Result<Vec<KeyLookup>, Error> {
        if req.len() > KEY_PROVIDER_BATCH_MAX {
            return Err(Error::Limit { limit: "key provider lookup batch exceeds 256" });
        }
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        let mut out = Vec::with_capacity(req.len());
        for &(cluster, epoch) in req {
            let res = match entries.get(&cluster) {
                Some(e) if e.disabled => Err(Error::KeyUnavailable { retry_after_secs: None }),
                Some(e) if e.destroyed => Ok(None),
                Some(e) => Ok(e.epochs.get(&epoch).map(|k| Key32::from_array(**k))),
                None => Ok(None),
            };
            out.push(KeyLookup { cluster, epoch, result: res });
        }
        Ok(out)
    }

    fn destroy_many(&self, clusters: &[ClusterId]) -> Result<DestroyReceipt, Error> {
        if clusters.len() > KEY_PROVIDER_BATCH_MAX {
            return Err(Error::Limit { limit: "key provider destroy batch exceeds 256" });
        }
        let mut entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        let mut absent = 0;
        for c in clusters {
            let entry = entries.entry(*c).or_default();
            entry.epochs.clear();
            entry.destroyed = true;
            let path = self.base_dir.join(format!("{}.key", hex_encode(c.as_bytes())));
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

    fn probe(&self, cluster: ClusterId, epoch: u32) -> Result<KeyState, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        Ok(match entries.get(&cluster) {
            Some(e) if e.destroyed => KeyState::Absent,
            Some(e) if e.disabled => KeyState::Disabled,
            Some(e) if e.epochs.contains_key(&epoch) => KeyState::Present,
            _ => KeyState::Absent,
        })
    }

    fn list(&self) -> Result<Vec<KeyListEntry>, Error> {
        let entries = self.entries.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider lock",
        })?;
        Ok(entries
            .iter()
            .filter(|(_, e)| !e.destroyed && !e.epochs.is_empty())
            .map(|(c, e)| KeyListEntry {
                cluster: *c,
                created_utc: e.created_utc,
            })
            .collect())
    }

    fn create_tomb_epoch(&self, epoch: u32) -> Result<(), Error> {
        let mut tombs = self.tombs.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider tomb lock",
        })?;
        match tombs.get(&epoch) {
            Some(FileTombEntry::Open(_)) => Ok(()),
            Some(FileTombEntry::Retired) => Err(Error::Shredded),
            None => {
                let mut raw = Zeroizing::new([0u8; 32]);
                fill_random(&mut *raw)?;
                let entry = FileTombEntry::Open(raw);
                write_tomb_file(&self.base_dir, &self.master_key, epoch, &entry)?;
                tombs.insert(epoch, entry);
                Ok(())
            }
        }
    }

    fn tomb_key(&self, epoch: u32) -> Result<Option<Key32>, Error> {
        let tombs = self.tombs.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider tomb lock",
        })?;
        match tombs.get(&epoch) {
            Some(FileTombEntry::Open(k)) => Ok(Some(Key32::from_array(**k))),
            Some(FileTombEntry::Retired) => Ok(None),
            None => Err(Error::KeyLost),
        }
    }

    fn retire_tomb_epoch(&self, epoch: u32) -> Result<KeyState, Error> {
        let mut tombs = self.tombs.lock().map_err(|_| Error::Poisoned {
            cause: "FileKeyProvider tomb lock",
        })?;
        let retired = FileTombEntry::Retired;
        write_tomb_file(&self.base_dir, &self.master_key, epoch, &retired)?;
        tombs.insert(epoch, retired);
        Ok(KeyState::Absent)
    }
}
