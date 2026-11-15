//! Identifikatoren (Spec 2.4). Ordinale sind dicht, nie wiederverwendet und nur intern sichtbar.

macro_rules! handle16 {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub [u8; 16]);
        impl $name {
            /// Aus Rohbytes.
            #[must_use]
            pub const fn from_bytes(b: [u8; 16]) -> Self {
                Self(b)
            }

            /// Rohbytes.
            #[must_use]
            pub fn as_bytes(&self) -> &[u8; 16] {
                &self.0
            }

            /// Zufaelliges Handle ohne Inhaltsbezug (Z10: ein Entropiefehler liefert nie einen
            /// Ersatzwert).
            ///
            /// # Errors
            /// `EntropyError`, wenn die Quelle nicht liefern kann.
            pub fn random(rng: &dyn crate::ports::CryptoRng) -> Result<Self, crate::ports::EntropyError> {
                let mut b = [0_u8; 16];
                rng.try_fill(&mut b)?;
                Ok(Self(b))
            }
        }
        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                // Kein Inhalt im Log (Z15).
                write!(f, concat!(stringify!($name), "(..)"))
            }
        }
    };
}

macro_rules! ordinal {
    ($(#[$m:meta])* $name:ident, $t:ty) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        pub struct $name(pub $t);
        impl $name {
            /// Naechstes Ordinal; `None` bei Ueberlauf (Vergiftung `Node`, Spec 2.4).
            #[must_use]
            pub fn next(self) -> Option<Self> {
                self.0.checked_add(1).map(Self)
            }
        }
    };
}

handle16!(/// Mandantenkennung (zufaellig, nicht geheim).
    TenantId);
handle16!(/// Dokumentidentitaet in P (stabil ueber Versionen).
    DocRef);
handle16!(/// Version eines P-Dokuments (Tresor-ID des `DocumentKey`).
    VerRef);
handle16!(/// Zufaelliges Handle aus K auf P.
    PersonalRef);
handle16!(/// Vorgang.
    CaseId);
handle16!(/// Zufaelliges Handle auf eine Identitaet in P.
    ActorRef);
handle16!(/// Loeschvorgang (UUIDv7).
    ErasureId);

ordinal!(/// Dichte Mandantennummer; 0 ist fuer Systeminterna reserviert.
    TenantOrd, u32);

impl TenantOrd {
    /// Reserviertes Ordinal fuer Systeminterna.
    pub const SYSTEM: Self = Self(0);

    /// `true` fuer das reservierte Systemordinal.
    #[must_use]
    pub const fn is_system(self) -> bool {
        self.0 == 0
    }
}

impl ErasureId {
    /// UUIDv7 (RFC 9562) aus Millisekundenzeit und 74 Zufallsbits (`rand` liefert 10 Bytes,
    /// genutzt werden 74 Bit). `None`, wenn `ms` nicht in 48 Bit passt.
    #[must_use]
    pub fn from_v7(ms: u64, rand: [u8; 10]) -> Option<Self> {
        let [hi0, hi1, t0, t1, t2, t3, t4, t5] = ms.to_be_bytes();
        if hi0 != 0 || hi1 != 0 {
            return None;
        }
        let [r0, r1, r2, r3, r4, r5, r6, r7, r8, r9] = rand;
        Some(Self([
            t0,
            t1,
            t2,
            t3,
            t4,
            t5,
            0x70 | (r0 & 0x0f),
            r1,
            0x80 | (r2 & 0x3f),
            r3,
            r4,
            r5,
            r6,
            r7,
            r8,
            r9,
        ]))
    }

    /// Zeitanteil in Millisekunden (aus einer UUIDv7).
    #[must_use]
    pub fn v7_ms(&self) -> u64 {
        let [t0, t1, t2, t3, t4, t5, ..] = self.0;
        u64::from_be_bytes([0, 0, t0, t1, t2, t3, t4, t5])
    }
}
ordinal!(/// Dichte Betroffenennummer je Mandant.
    SubjOrd, u32);
ordinal!(/// Dichte Versionsnummer je Mandant.
    VerOrd, u32);
ordinal!(/// Dichte K-Objektnummer je Mandant.
    KOrd, u64);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinal_overflow_is_none() {
        assert_eq!(VerOrd(u32::MAX).next(), None);
        assert_eq!(VerOrd(1).next(), Some(VerOrd(2)));
    }

    #[test]
    fn tenant_ord_zero_is_system() {
        assert!(TenantOrd::SYSTEM.is_system());
        assert!(!TenantOrd(1).is_system());
    }

    #[test]
    fn uuid_v7_layout() {
        let id = ErasureId::from_v7(0x0123_4567_89ab, [0xff; 10]);
        let id = id.map(|i| i.0);
        assert_eq!(
            id,
            Some([
                0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0x7f, 0xff, 0xbf, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff
            ])
        );
        assert_eq!(ErasureId::from_v7(1 << 48, [0; 10]), None);
        let e = ErasureId::from_v7(1_700_000_000_000, [1; 10]);
        assert_eq!(e.map(|i| i.v7_ms()), Some(1_700_000_000_000));
    }

    #[test]
    fn uuid_v7_is_time_ordered() {
        let a = ErasureId::from_v7(10, [0xff; 10]);
        let b = ErasureId::from_v7(11, [0; 10]);
        assert!(a < b);
    }

    #[test]
    fn random_handle_uses_port_and_fails_closed() {
        struct Fixed(u8);
        impl crate::ports::CryptoRng for Fixed {
            fn try_fill(&self, d: &mut [u8]) -> Result<(), crate::ports::EntropyError> {
                d.fill(self.0);
                Ok(())
            }
        }
        struct Broken;
        impl crate::ports::CryptoRng for Broken {
            fn try_fill(&self, _: &mut [u8]) -> Result<(), crate::ports::EntropyError> {
                Err(crate::ports::EntropyError)
            }
        }
        assert_eq!(DocRef::random(&Fixed(9)), Ok(DocRef([9; 16])));
        assert_eq!(DocRef::random(&Broken), Err(crate::ports::EntropyError));
        assert_eq!(DocRef::from_bytes([3; 16]).as_bytes(), &[3; 16]);
    }

    #[test]
    fn debug_hides_content() {
        let s = format!("{:?}", TenantId([7; 16]));
        assert!(!s.contains('7'));
    }
}
