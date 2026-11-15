//! Geheime Schlüssel (Spec 4.2, 4.6, Z10).
//!
//! Geheimnisse haben kein `Default`, `Clone`, `Debug` und keinen `Serialize`. Das Material
//! liegt in `Zeroizing` und wird beim Verwerfen überschrieben. Jeder Schlüsseltyp ist ein
//! eigener Typ, damit etwa ein `SubjectKey` nie als `DocumentKey` übergeben werden kann.
//! Rohes Schlüsselmaterial verlässt das Crate nie. Außen gibt es nur den AEAD-Cipher
//! ([`crate::aead::SealKey`]) und die Ableitungen in [`crate::kdf`].

use crate::error::CryptoError;
use acta_base::CryptoRng;
use zeroize::Zeroizing;

/// Länge eines Schlüssels in Byte (256 Bit).
pub const KEY_LEN: usize = 32;

/// Versiegelter Supertrait: nur Schlüsseltypen dieses Crates implementieren [`crate::aead::SealKey`].
#[doc(hidden)]
pub mod private {
    /// Marker für die Verschlossenheit von `SealKey`.
    pub trait Sealed {}
}

macro_rules! secret_key {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        pub struct $name {
            bytes: Zeroizing<[u8; KEY_LEN]>,
        }

        impl $name {
            /// Erzeugt einen frischen Schlüssel aus dem Zufall des Ports.
            ///
            /// # Errors
            /// [`CryptoError::Entropy`], wenn die Quelle versagt. Dann entsteht kein Schlüssel.
            pub fn generate(rng: &dyn CryptoRng) -> Result<Self, CryptoError> {
                let mut bytes = Zeroizing::new([0_u8; KEY_LEN]);
                rng.try_fill(bytes.as_mut_slice())?;
                Ok(Self { bytes })
            }

            /// Übernimmt 32 Byte, etwa beim Laden aus dem Tresor. Der Aufrufer ist für die Herkunft verantwortlich.
            #[must_use]
            pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
                Self {
                    bytes: Zeroizing::new(bytes),
                }
            }

            /// Schlüsselmaterial für Ableitungen und Cipher in diesem Crate.
            pub(crate) fn expose(&self) -> &[u8] {
                self.bytes.as_slice()
            }
        }

        impl private::Sealed for $name {}

        impl crate::aead::SealKey for $name {
            fn cipher(&self) -> Result<chacha20poly1305::XChaCha20Poly1305, CryptoError> {
                use chacha20poly1305::KeyInit;
                chacha20poly1305::XChaCha20Poly1305::new_from_slice(self.expose())
                    .map_err(|_| CryptoError::Internal)
            }
        }
    };
}

secret_key!(
    /// Datenschlüssel (DEK) einer Dokument-Version, Umschlag-Schlüssel der Ebene P.
    DataKey
);
secret_key!(
    /// Schlüssel einer Betroffenen-Person (`SubjectKey`).
    SubjectKey
);
secret_key!(
    /// Schlüssel eines Dokuments über alle Versionen (`DocumentKey`).
    DocumentKey
);
secret_key!(
    /// Schlüssel eines Segments (`SegmentKey`).
    SegmentKey
);
secret_key!(
    /// Generationsgeheimnis des Schlüsselankers (`MasterKey`, Spec 4.2).
    MasterKey
);
secret_key!(
    /// Geheimnis für Tags und Dedup-Salze (`k_idx`, Spec 2.4). Ohne Standardwert.
    IndexSecret
);

impl DataKey {
    /// Abgeleiteter Datenschlüssel aus einer Ableitung in diesem Crate.
    pub(crate) fn from_derived(bytes: Zeroizing<[u8; KEY_LEN]>) -> Self {
        Self { bytes }
    }
}
