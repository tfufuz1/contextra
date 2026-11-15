//! Umschlag v1 (Spec 4.5): Festwert gegen eine unabhängige Referenz, Negativtests und Positivkontrollen.

use acta_base::{CryptoRng, EntropyError};
use acta_crypto::aead::{open, seal, Knowledge, Personal, SealCtx, Sealed};
use acta_crypto::error::CryptoError;
use acta_crypto::key::DataKey;
use std::io;

/// Festwert: XChaCha20-Poly1305 aus PyNaCl (libsodium) mit denselben Eingaben wie unten.
/// Schlüssel 0x00..0x1f, Nonce 0x64..0x7b, AAD = "acta/rec/v1" ‖ Mandant ‖ P ‖ Keyspace ‖ Datensatz ‖ Version,
/// Klartext "Akte 42: Antrag" gerahmt und auf 256 Byte gepolstert.
const KAT_ENVELOPE_HEX: &str = "0101016465666768696a6b6c6d6e6f707172737475767778797a7b7c73f9bfbe45fde7a704af05862ed3bd29da3571e42ccab08c5564a72b5e7810ab06437e2cb6f7c7ea06651c545608516ee5d01f9fb234a6cf0f0b48fb88be6c56d3c4e5d06ea42dee0dbc94cdfa8e4bf3ed81258ee3c3fb915da991e690fcb54c7d9d77f121d813008c023f91355992b8602b7291a0aa5cae3f13e4550120ca8971bdc369c8d86c4ce0606be42bcf183b4889c75b38944b161be8eaddbf39fa4f1ee95d8c91ffdebf4bc8b220ea2822812c7689c750a7bf774db9a5e127c4adaa4e8a50c34f080b8023bd2b29ca6708da4e949ea1dc06048068d276de2fd40923978bb6da2324f05606412ac622f869fd6bca00d530090e494e81d8aecc5f92aa8a8fd0932df59bc8b4dd22a5b14384";

/// Liefert eine feste Byte-Folge. Ersatz für die Nonce-Quelle in Tests.
struct FixedRng(Vec<u8>);
impl CryptoRng for FixedRng {
    fn try_fill(&self, dst: &mut [u8]) -> Result<(), EntropyError> {
        for (d, s) in dst.iter_mut().zip(self.0.iter()) {
            *d = *s;
        }
        Ok(())
    }
}

/// Zählende Quelle: jede Ziehung liefert neue Bytes. Für Roundtrip-Tests.
struct CountingRng(std::sync::atomic::AtomicU8);
impl CryptoRng for CountingRng {
    fn try_fill(&self, dst: &mut [u8]) -> Result<(), EntropyError> {
        for d in dst.iter_mut() {
            *d = self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    }
}

/// Immer fehlerhafte Quelle. Positivkontrolle für `Entropy` (Z10).
struct FailingRng;
impl CryptoRng for FailingRng {
    fn try_fill(&self, _dst: &mut [u8]) -> Result<(), EntropyError> {
        Err(EntropyError)
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn ctx() -> SealCtx {
    SealCtx {
        tenant: [0x11; 16],
        keyspace: 0x0001,
        record_ref: [0x22; 16],
        version: 7,
    }
}

fn key() -> DataKey {
    DataKey::from_bytes(std::array::from_fn(|i| i as u8))
}

fn kat_nonce() -> FixedRng {
    FixedRng(std::array::from_fn::<u8, 24, _>(|i| 100 + i as u8).to_vec())
}

#[test]
fn envelope_matches_independent_reference() -> Result<(), CryptoError> {
    let sealed: Sealed<Personal> = seal(&key(), &ctx(), b"Akte 42: Antrag", &kat_nonce())?;
    assert_eq!(hex(sealed.as_bytes()), KAT_ENVELOPE_HEX);
    Ok(())
}

#[test]
fn roundtrip_over_size_classes() -> Result<(), CryptoError> {
    let rng = CountingRng(std::sync::atomic::AtomicU8::new(1));
    for len in [0_usize, 1, 251, 252, 4092, 4093, 4096, 5000, 20_000] {
        let data: Vec<u8> = (0..len).map(|i| (i % 253) as u8).collect();
        let sealed: Sealed<Personal> = seal(&key(), &ctx(), &data, &rng)?;
        let plain = open(&key(), &ctx(), &sealed)?;
        assert_eq!(plain.as_slice(), data.as_slice(), "length {len}");
    }
    Ok(())
}

#[test]
fn manipulation_and_wrong_context_are_rejected() -> Result<(), CryptoError> {
    let sealed: Sealed<Personal> = seal(&key(), &ctx(), b"geheim", &CountingRng(0.into()))?;
    let good = open(&key(), &ctx(), &sealed)?;
    assert_eq!(good.as_slice(), b"geheim");

    // Ein einzelnes veraendertes Byte im Chiffrat (nach dem Header).
    let mut bytes = sealed.as_bytes().to_vec();
    let idx = 27 + 3;
    if let Some(b) = bytes.get_mut(idx) {
        *b ^= 0x80;
    }
    let tampered = Sealed::<Personal>::from_bytes(bytes)?;
    assert!(matches!(
        open(&key(), &ctx(), &tampered),
        Err(CryptoError::DecryptionFailed)
    ));

    // Jede Kontextangabe gehoert zur AAD und muss passen.
    let mut other_tenant = ctx();
    other_tenant.tenant[0] ^= 1;
    let mut other_ref = ctx();
    other_ref.record_ref[15] ^= 1;
    let mut other_version = ctx();
    other_version.version += 1;
    let mut other_keyspace = ctx();
    other_keyspace.keyspace += 1;
    for wrong in [other_tenant, other_ref, other_version, other_keyspace] {
        assert!(matches!(
            open(&key(), &wrong, &sealed),
            Err(CryptoError::DecryptionFailed)
        ));
    }

    // Falscher Schlüssel.
    let wrong_key = DataKey::from_bytes([0x99; 32]);
    assert!(matches!(
        open(&wrong_key, &ctx(), &sealed),
        Err(CryptoError::DecryptionFailed)
    ));
    Ok(())
}

#[test]
fn header_fields_are_checked_on_load() -> Result<(), CryptoError> {
    let sealed: Sealed<Personal> = seal(&key(), &ctx(), b"x", &CountingRng(0.into()))?;
    let base = sealed.as_bytes().to_vec();

    let mut bad_version = base.clone();
    if let Some(v) = bad_version.get_mut(0) {
        *v = 0x02;
    }
    assert!(matches!(
        Sealed::<Personal>::from_bytes(bad_version),
        Err(CryptoError::Unsupported)
    ));

    // Ebenenfremde Bytes: ein P-Umschlag ist kein K-Umschlag.
    assert!(matches!(
        Sealed::<Knowledge>::from_bytes(base.clone()),
        Err(CryptoError::Unsupported)
    ));

    assert!(matches!(
        Sealed::<Personal>::from_bytes(base.get(..40).unwrap_or(&[]).to_vec()),
        Err(CryptoError::Malformed)
    ));
    Ok(())
}

#[test]
fn entropy_failure_produces_no_envelope() -> io::Result<()> {
    let res: Result<Sealed<Personal>, CryptoError> = seal(&key(), &ctx(), b"x", &FailingRng);
    assert!(matches!(res, Err(CryptoError::Entropy)));
    // Positivkontrolle: mit funktionierender Quelle entsteht ein Umschlag.
    let ok: Result<Sealed<Personal>, CryptoError> =
        seal(&key(), &ctx(), b"x", &CountingRng(0.into()));
    assert!(ok.is_ok());
    Ok(())
}
