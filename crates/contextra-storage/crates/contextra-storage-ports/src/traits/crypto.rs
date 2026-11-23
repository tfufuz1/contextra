//! Kryptoports (SP-PORTS §2, SP-KEYS §3.2). Referenzimplementierung in contextra-crypto-ref.
use crate::error::Error;
use crate::keys::Key32;

pub trait Cipher: Send + Sync {
    fn seal(
        &self,
        key: &Key32,
        nonce: &[u8; 24],
        aad: &[u8],
        pt: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<(), Error>;
    /// Fehlschlag der Authentizität ⇒ Error::Corrupt, nie Teilausgabe.
    fn open(
        &self,
        key: &Key32,
        nonce: &[u8; 24],
        aad: &[u8],
        ct: &[u8],
        out: &mut Vec<u8>,
    ) -> Result<(), Error>;
}

pub trait Mac: Send + Sync {
    /// Produces a fixed-width tag.  A fallible port avoids hiding a provider
    /// or algorithm setup failure behind a fabricated authentication value.
    fn tag(&self, key: &Key32, data: &[u8]) -> Result<[u8; 32], Error>;
}

pub trait Kdf: Send + Sync {
    fn derive(&self, master: &Key32, label: &[u8]) -> Result<Key32, Error>;
}

pub trait Checksum: Send + Sync {
    fn crc32c(&self, data: &[u8]) -> u32;
}
