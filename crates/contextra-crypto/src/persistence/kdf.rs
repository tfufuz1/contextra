//! `Kdf` als HKDF-SHA256 (SPEC §4, §16.2; algo_set 1: „HKDF-artig über Kdf-Port“).
//!
//! Die Spec nennt die Hashfunktion nicht. SHA-256 ist eine Annahme (siehe PR-Beschreibung).
//! `key` ist ein 256-Bit-Schlüssel und dient als Eingangsschlüsselmaterial (IKM); die
//! Extraktion nutzt einen Null-Salt. Kontextteile werden ohne Längenangabe konkateniert
//! (SPEC §4: der Kern kodiert Längen selbst).

use contextra_ports::persistence::Kdf;
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

/// HKDF-SHA256, auf 32 Byte Ausgabe festgelegt. Zustandslos.
#[derive(Debug, Clone, Copy, Default)]
pub struct HkdfSha256Kdf;

impl Kdf for HkdfSha256Kdf {
    fn derive(&self, key: &[u8; 32], context: &[&[u8]]) -> [u8; 32] {
        let hk = Hkdf::<Sha256>::new(None, key);
        let mut out = [0u8; 32];
        // 32 Byte liegen unter 255 · HashLen; Expand scheitert hier nicht.
        if hk.expand_multi_info(context, &mut out).is_err() {
            out.zeroize();
        }
        out
    }
}
