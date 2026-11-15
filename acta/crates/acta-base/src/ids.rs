//! Identifikatoren (Spec 2.4). Ordinale sind dicht, nie wiederverwendet und nur intern sichtbar.

macro_rules! handle16 {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub [u8; 16]);
        impl $name {
            /// Rohbytes.
            #[must_use]
            pub fn as_bytes(&self) -> &[u8; 16] {
                &self.0
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
    fn debug_hides_content() {
        let s = format!("{:?}", TenantId([7; 16]));
        assert!(!s.contains('7'));
    }
}
