//! SealFrame and canonical AAD. Logical identities are stable across file
//! copies and relocation; physical file IDs and offsets are deliberately absent.

use contextra_storage_ports::traits::{Checksum, Cipher, Kdf, Mac};
use contextra_storage_ports::{ClusterId, Error, Key32, SpaceId};
use zeroize::Zeroizing;

const LABEL: &[u8] = b"ctxs seal v1";
const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;
const HEADER_LEN: usize = 4 + NONCE_LEN;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealMode { Plain, Authenticated, Encrypted }

#[derive(Debug, Clone, Copy)]
pub enum Aad<'a> {
    Block { instance: &'a [u8; 16], space: SpaceId, cluster: ClusterId, key_epoch: u32, block_uid: &'a [u8; 16] },
    RedoOp { instance: &'a [u8; 16], space: SpaceId, cluster: ClusterId, key_epoch: u32, op_uid: &'a [u8; 16], op_index: u16 },
    BlobRecord { instance: &'a [u8; 16], space: SpaceId, cluster: ClusterId, key_epoch: u32, record_uid: &'a [u8; 16] },
}

impl Aad<'_> {
    fn key_epoch(self) -> u32 {
        match self { Self::Block { key_epoch, .. } | Self::RedoOp { key_epoch, .. } | Self::BlobRecord { key_epoch, .. } => key_epoch }
    }

    pub fn encode(self) -> Vec<u8> {
        let (domain, instance, space, cluster, epoch) = match self {
            Self::Block { instance, space, cluster, key_epoch, .. } => (1, instance, space, cluster, key_epoch),
            Self::RedoOp { instance, space, cluster, key_epoch, .. } => (2, instance, space, cluster, key_epoch),
            Self::BlobRecord { instance, space, cluster, key_epoch, .. } => (3, instance, space, cluster, key_epoch),
        };
        let mut out = Vec::with_capacity(4 + 16 + 2 + 16 + 4 + 18);
        out.extend_from_slice(b"R8AD");
        out.push(domain);
        out.extend_from_slice(instance);
        out.extend_from_slice(&space.0.to_le_bytes());
        out.extend_from_slice(cluster.as_bytes());
        out.extend_from_slice(&epoch.to_le_bytes());
        match self {
            Self::Block { block_uid, .. } | Self::BlobRecord { record_uid: block_uid, .. } => out.extend_from_slice(block_uid),
            Self::RedoOp { op_uid, op_index, .. } => { out.extend_from_slice(op_uid); out.extend_from_slice(&op_index.to_le_bytes()); }
        }
        out
    }
}

pub fn seal_frame(
    mode: SealMode,
    key_epoch: u32,
    source: Option<&Key32>,
    nonce: &[u8; NONCE_LEN],
    aad: Aad<'_>,
    plaintext: &[u8],
    cipher: &dyn Cipher,
    kdf: &dyn Kdf,
    mac: &dyn Mac,
    checksum: &dyn Checksum,
) -> Result<Vec<u8>, Error> {
    if aad.key_epoch() != key_epoch { return Err(Error::Limit { limit: "SealFrame/AAD key epoch mismatch" }); }
    let aad = aad.encode();
    let mut out = Vec::with_capacity(HEADER_LEN + plaintext.len() + 32);
    out.extend_from_slice(&key_epoch.to_le_bytes());
    out.extend_from_slice(nonce);
    match mode {
        SealMode::Plain => {
            out.extend_from_slice(plaintext);
            let mut authenticated = Vec::with_capacity(aad.len() + out.len());
            authenticated.extend_from_slice(&aad);
            authenticated.extend_from_slice(&out);
            out.extend_from_slice(&checksum.crc32c(&authenticated).to_le_bytes());
        }
        SealMode::Authenticated => {
            let source = source.ok_or(Error::KeyUnavailable { retry_after_secs: None })?;
            let subkey = kdf.derive_subkey_fast(source, LABEL, nonce)?;
            out.extend_from_slice(plaintext);
            let mut authenticated = Vec::with_capacity(aad.len() + out.len());
            authenticated.extend_from_slice(&aad);
            authenticated.extend_from_slice(&out);
            let tag = mac.tag(&subkey, &authenticated);
            out.extend_from_slice(tag.get(..TAG_LEN).ok_or(Error::Corrupt { place: "MAC tag length" })?);
        }
        SealMode::Encrypted => {
            let source = source.ok_or(Error::KeyUnavailable { retry_after_secs: None })?;
            let subkey = kdf.derive_subkey_fast(source, LABEL, nonce)?;
            let mut ciphertext = Vec::new();
            cipher.seal(&subkey, nonce, &aad, plaintext, &mut ciphertext)?;
            out.extend_from_slice(&ciphertext);
        }
    }
    Ok(out)
}

pub fn unseal_frame(
    mode: SealMode,
    source: Option<&Key32>,
    aad: Aad<'_>,
    frame: &[u8],
    cipher: &dyn Cipher,
    kdf: &dyn Kdf,
    mac: &dyn Mac,
    checksum: &dyn Checksum,
) -> Result<(u32, Zeroizing<Vec<u8>>), Error> {
    if frame.len() < HEADER_LEN { return Err(Error::Corrupt { place: "seal frame header" }); }
    let epoch_bytes: [u8; 4] = frame.get(..4).ok_or(Error::Corrupt { place: "seal frame epoch" })?
        .try_into().map_err(|_| Error::Corrupt { place: "seal frame epoch" })?;
    let key_epoch = u32::from_le_bytes(epoch_bytes);
    if aad.key_epoch() != key_epoch { return Err(Error::Corrupt { place: "SealFrame/AAD key epoch mismatch" }); }
    let aad = aad.encode();
    let nonce: &[u8; NONCE_LEN] = frame.get(4..HEADER_LEN).ok_or(Error::Corrupt { place: "seal frame nonce" })?
        .try_into().map_err(|_| Error::Corrupt { place: "seal frame nonce" })?;
    let body = frame.get(HEADER_LEN..).ok_or(Error::Corrupt { place: "seal frame body" })?;
    let mut plaintext = Zeroizing::new(Vec::new());
    match mode {
        SealMode::Plain => {
            if body.len() < 4 { return Err(Error::Corrupt { place: "plain seal checksum" }); }
            let split = body.len() - 4;
            let data = body.get(..split).ok_or(Error::Corrupt { place: "plain seal body" })?;
            let received = u32::from_le_bytes(body.get(split..).ok_or(Error::Corrupt { place: "plain seal checksum" })?
                .try_into().map_err(|_| Error::Corrupt { place: "plain seal checksum" })?);
            let mut authenticated = Vec::with_capacity(aad.len() + HEADER_LEN + data.len());
            authenticated.extend_from_slice(&aad);
            authenticated.extend_from_slice(frame.get(..HEADER_LEN).ok_or(Error::Corrupt { place: "seal frame header" })?);
            authenticated.extend_from_slice(data);
            if checksum.crc32c(&authenticated) != received { return Err(Error::Corrupt { place: "plain seal checksum" }); }
            plaintext.extend_from_slice(data);
        }
        SealMode::Authenticated => {
            if body.len() < TAG_LEN { return Err(Error::Corrupt { place: "authenticated seal tag" }); }
            let source = source.ok_or(Error::KeyUnavailable { retry_after_secs: None })?;
            let split = body.len() - TAG_LEN;
            let data = body.get(..split).ok_or(Error::Corrupt { place: "authenticated seal body" })?;
            let tag = body.get(split..).ok_or(Error::Corrupt { place: "authenticated seal tag" })?;
            let subkey = kdf.derive_subkey_fast(source, LABEL, nonce)?;
            let mut authenticated = Vec::with_capacity(aad.len() + HEADER_LEN + data.len());
            authenticated.extend_from_slice(&aad);
            authenticated.extend_from_slice(frame.get(..HEADER_LEN).ok_or(Error::Corrupt { place: "seal frame header" })?);
            authenticated.extend_from_slice(data);
            let actual = mac.tag(&subkey, &authenticated);
            if !constant_time_eq(actual.get(..TAG_LEN).ok_or(Error::Corrupt { place: "MAC tag length" })?, tag) {
                return Err(Error::Corrupt { place: "authenticated seal tag" });
            }
            plaintext.extend_from_slice(data);
        }
        SealMode::Encrypted => {
            let source = source.ok_or(Error::KeyUnavailable { retry_after_secs: None })?;
            let subkey = kdf.derive_subkey_fast(source, LABEL, nonce)?;
            cipher.open(&subkey, nonce, &aad, body, &mut plaintext)?;
        }
    }
    Ok((u32::from_le_bytes(epoch_bytes), plaintext))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() { return false; }
    left.iter().zip(right).fold(0u8, |difference, (a, b)| difference | (a ^ b)) == 0
}
