//! Key hierarchy derivation according to SPEC 10.1 and D-80.

use crate::kdf::hkdf_expand;
use crate::labels;
use crate::secret::SecretKey;
use rand::RngCore;

/// Master key wrapper (ZeroizeOnDrop).
pub struct MasterKey(pub SecretKey<32>);

impl MasterKey {
    /// Generates a new random master key using OsRng.
    pub fn generate_random() -> Self {
        let mut b = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut b);
        Self(SecretKey::new(b))
    }

    /// Wraps existing 32 bytes into MasterKey.
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(SecretKey::new(bytes))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

// Strongly typed keys to prevent confusion at compile time
pub struct EncKey(pub SecretKey<32>);
pub struct IdxKey(pub SecretKey<32>);
pub struct UnitKey(pub SecretKey<32>);
pub struct SignSeed(pub SecretKey<32>);
pub struct WalKey(pub SecretKey<32>);
pub struct LedgerKey(pub SecretKey<32>);
pub struct HdrKey(pub SecretKey<32>);
pub struct CommitKey(pub SecretKey<32>);
pub struct VaultMacKey(pub SecretKey<32>);
pub struct FileKey(pub SecretKey<32>);

impl FileKey {
    pub fn new(b: [u8; 32]) -> Self {
        Self(SecretKey::new(b))
    }
    pub fn as_bytes(&self) -> &[u8; 32] {
        self.0.as_bytes()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileKind {
    Wal = 1,
    SSTable = 2,
    Manifest = 3,
    TextSegment = 4,
    VectorSnapshot = 5,
    Journal = 6,
    GateLog = 7,
    Ledger = 8,
    Scope = 9,
    Vault = 10,
}

pub struct KeyHierarchy {
    enc: EncKey,
    idx: IdxKey,
    unit: UnitKey,
    sign_seed: SignSeed,
    wal: WalKey,
    ledger: LedgerKey,
    hdr: HdrKey,
    master: MasterKey,
}

impl KeyHierarchy {
    pub fn from_master(m: MasterKey) -> Self {
        let master_bytes = m.as_bytes();

        let mut enc_bytes = [0u8; 32];
        let mut idx_bytes = [0u8; 32];
        let mut unit_bytes = [0u8; 32];
        let mut sign_bytes = [0u8; 32];
        let mut wal_bytes = [0u8; 32];
        let mut ledger_bytes = [0u8; 32];
        let mut hdr_bytes = [0u8; 32];

        let _ = hkdf_expand(master_bytes, labels::ENC, &mut enc_bytes);
        let _ = hkdf_expand(master_bytes, labels::IDX, &mut idx_bytes);
        let _ = hkdf_expand(master_bytes, labels::UNIT, &mut unit_bytes);
        let _ = hkdf_expand(master_bytes, labels::SIGN, &mut sign_bytes);
        let _ = hkdf_expand(master_bytes, labels::WAL, &mut wal_bytes);
        let _ = hkdf_expand(master_bytes, labels::LEDGER, &mut ledger_bytes);
        let _ = hkdf_expand(master_bytes, labels::HDR, &mut hdr_bytes);

        Self {
            enc: EncKey(SecretKey::new(enc_bytes)),
            idx: IdxKey(SecretKey::new(idx_bytes)),
            unit: UnitKey(SecretKey::new(unit_bytes)),
            sign_seed: SignSeed(SecretKey::new(sign_bytes)),
            wal: WalKey(SecretKey::new(wal_bytes)),
            ledger: LedgerKey(SecretKey::new(ledger_bytes)),
            hdr: HdrKey(SecretKey::new(hdr_bytes)),
            master: m,
        }
    }

    pub fn enc(&self) -> &EncKey {
        &self.enc
    }

    pub fn idx(&self) -> &IdxKey {
        &self.idx
    }

    pub fn unit(&self) -> &UnitKey {
        &self.unit
    }

    pub fn sign_seed(&self) -> &SignSeed {
        &self.sign_seed
    }

    pub fn wal(&self) -> &WalKey {
        &self.wal
    }

    pub fn ledger(&self) -> &LedgerKey {
        &self.ledger
    }

    pub fn hdr(&self) -> &HdrKey {
        &self.hdr
    }

    pub fn commit(&self, epoch: u32) -> CommitKey {
        let mut info = Vec::with_capacity(labels::COMMIT.len() + 4);
        info.extend_from_slice(labels::COMMIT);
        info.extend_from_slice(&epoch.to_be_bytes());
        let mut out = [0u8; 32];
        let _ = hkdf_expand(self.master.as_bytes(), &info, &mut out);
        CommitKey(SecretKey::new(out))
    }

    pub fn file_key(&self, kind: FileKind, id: [u8; 16]) -> FileKey {
        let mut info = Vec::with_capacity(1 + 16);
        info.push(kind as u8);
        info.extend_from_slice(&id);
        let mut out = [0u8; 32];
        let _ = hkdf_expand(self.enc.0.as_bytes(), &info, &mut out);
        FileKey(SecretKey::new(out))
    }

    pub fn fkmask(&self, id: [u8; 16]) -> [u8; 32] {
        let mut info = Vec::with_capacity(labels::FKMASK.len() + 16);
        info.extend_from_slice(labels::FKMASK);
        info.extend_from_slice(&id);
        let mut out = [0u8; 32];
        let _ = hkdf_expand(self.enc.0.as_bytes(), &info, &mut out);
        out
    }

    pub fn vault_mac(&self) -> VaultMacKey {
        let mut out = [0u8; 32];
        let _ = hkdf_expand(self.enc.0.as_bytes(), labels::VAULT_MAC, &mut out);
        VaultMacKey(SecretKey::new(out))
    }
}
