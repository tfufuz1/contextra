//! Schlüsselkodierung und Matcher (SPEC 9.1, D-10, D-70, D-74, TY-03, TY-04, TY-05, L-8).
//!
//! Feste Schlüssellängen:
//! - KeyBound = 65 Byte: region(1) + 4 × 16 (tenant, coll, cluster, doc)
//! - KeyBuf = 78 Byte: KeyBound(65) + family(1) + sub(u32, 4) + !commit_seq(u64, 8)
//!
//! `!commit_seq` (bitweises Komplement, big-endian) sorgt für absteigende Sortierung (jüngste zuerst).
//!
//! Serde-Hinweis: `serde` leitet `Serialize`/`Deserialize` für `[u8; N]` mit N > 32 nicht ab.
//! `KeyBound` ([u8; 65]) und `KeyBuf` ([u8; 78]) verwenden daher manuelle Impls über `serde_bytes`.
//! Das kanonische Format ist `bytes` (hex-kompakt in JSON, effizient in binären Formaten).

use core::fmt;
use serde::{Deserialize, Serialize};

use crate::tags::{ClusterTag, CollTag, DocTag, NameTag, TenantTag};

/// Länge des Präfix-Bereichsbegrenzers (65 Bytes).
pub const KEY_BOUND_LEN: usize = 65;

/// Feste Gesamtlänge des Speicherschlüssels (78 Bytes, TY-03).
pub const KEY_LEN: usize = 78;

/// Region im Schlüsselraum (SPEC 9.1).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Region {
    /// Dokumentfamilien unter Cluster (0x01).
    Data = 0x01,
    /// 0x02 ist reserviert (frühere Region LOC entfällt laut D-70).
    ReservedLoc = 0x02,
    /// Collection-Katalog und Zähler (0x03).
    Meta = 0x03,
}

impl Region {
    /// Liefert den numerischen U8-Wert der Region.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Speicherfamilie innerhalb einer Region (SPEC 9.2).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Family {
    /// Chunks und Metadaten (0x01).
    Doc = 0x01,
    /// Chunk-Zuordnung und Vektor-Slot-Nummern (0x02).
    DocMap = 0x02,
    /// Manifest, Versionen, Identität (0x03).
    Manifest = 0x03,
    /// 0x04 reserviert (Locator entfällt laut D-70).
    ReservedLocator = 0x04,
    /// Noch nicht segmentierte Textdeltas (0x05).
    TextBuf = 0x05,
}

impl Family {
    /// Liefert den numerischen U8-Wert der Familie.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// 65-Byte-Bereichsgrenze für prefixfreie Bereichssuche (SPEC 9.1, D-10).
///
/// Serde nutzt manuelle Impls, da `[u8; 65]` das Derive-Limit von 32 überschreitet.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBound([u8; KEY_BOUND_LEN]);

impl Serialize for KeyBound {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for KeyBound {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = KeyBound;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "exactly {} bytes for KeyBound", KEY_BOUND_LEN)
            }
            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<KeyBound, E> {
                let arr: [u8; KEY_BOUND_LEN] = v.try_into().map_err(|_| {
                    E::invalid_length(v.len(), &self)
                })?;
                Ok(KeyBound(arr))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<KeyBound, A::Error> {
                let mut arr = [0u8; KEY_BOUND_LEN];
                for (i, b) in arr.iter_mut().enumerate() {
                    *b = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::invalid_length(i, &self)
                    })?;
                }
                Ok(KeyBound(arr))
            }
        }
        d.deserialize_bytes(Visitor)
    }
}

impl KeyBound {
    /// Erzeugt ein `KeyBound` aus rohen 65 Bytes.
    #[inline]
    pub const fn new(raw: [u8; KEY_BOUND_LEN]) -> Self {
        Self(raw)
    }

    /// Liefert die rohen Bytes als Slice.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; KEY_BOUND_LEN] {
        &self.0
    }
}

impl fmt::Debug for KeyBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KeyBound({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// 78-Byte-Schlüssel fester Länge (SPEC 9.1, D-10, TY-03).
///
/// Serde nutzt manuelle Impls, da `[u8; 78]` das Derive-Limit von 32 überschreitet.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBuf([u8; KEY_LEN]);

impl Serialize for KeyBuf {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for KeyBuf {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = KeyBuf;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "exactly {} bytes for KeyBuf", KEY_LEN)
            }
            fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> Result<KeyBuf, E> {
                let arr: [u8; KEY_LEN] = v.try_into().map_err(|_| {
                    E::invalid_length(v.len(), &self)
                })?;
                Ok(KeyBuf(arr))
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<KeyBuf, A::Error> {
                let mut arr = [0u8; KEY_LEN];
                for (i, b) in arr.iter_mut().enumerate() {
                    *b = seq.next_element()?.ok_or_else(|| {
                        serde::de::Error::invalid_length(i, &self)
                    })?;
                }
                Ok(KeyBuf(arr))
            }
        }
        d.deserialize_bytes(Visitor)
    }
}

impl KeyBuf {
    /// Erzeugt einen neuen 78-Byte-Schlüssel für die DATA-Region (TY-03).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        region: Region,
        tenant: TenantTag,
        coll: CollTag,
        cluster: ClusterTag,
        doc: DocTag,
        family: Family,
        sub: u32,
        commit_seq: u64,
    ) -> Self {
        let mut b = [0u8; KEY_LEN];
        b[0] = region.as_u8();
        b[1..17].copy_from_slice(tenant.as_bytes());
        b[17..33].copy_from_slice(coll.as_bytes());
        b[33..49].copy_from_slice(cluster.as_bytes());
        b[49..65].copy_from_slice(doc.as_bytes());
        b[65] = family.as_u8();
        b[66..70].copy_from_slice(&sub.to_be_bytes());
        // Bitweises Komplement für absteigende Sortierung (TY-04)
        b[70..78].copy_from_slice(&(!commit_seq).to_be_bytes());
        Self(b)
    }

    /// Erzeugt einen Schlüssel für die META-Region (L-8 Golden-Format).
    /// Format: cluster_tag = 0¹⁶, doc_tag = name_tag, family = Manifest (0x03).
    pub fn meta(
        tenant: TenantTag,
        coll: CollTag,
        name: NameTag,
        sub: u32,
        commit_seq: u64,
    ) -> Self {
        let mut b = [0u8; KEY_LEN];
        b[0] = Region::Meta.as_u8();
        b[1..17].copy_from_slice(tenant.as_bytes());
        b[17..33].copy_from_slice(coll.as_bytes());
        // cluster_tag = 0x00¹⁶
        b[33..49].fill(0x00);
        b[49..65].copy_from_slice(name.as_bytes());
        b[65] = Family::Manifest.as_u8();
        b[66..70].copy_from_slice(&sub.to_be_bytes());
        b[70..78].copy_from_slice(&(!commit_seq).to_be_bytes());
        Self(b)
    }

    /// Erzeugt ein `KeyBuf` aus rohen 78 Bytes.
    #[inline]
    pub const fn from_raw(raw: [u8; KEY_LEN]) -> Self {
        Self(raw)
    }

    /// Liefert die rohen Bytes des Schlüssels.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    /// Rekonstruiert die Commit-Sequenznummer ohne DEK (D-74, TY-04).
    #[inline]
    pub fn commit_seq(&self) -> u64 {
        let mut raw = [0u8; 8];
        raw.copy_from_slice(&self.0[70..78]);
        !u64::from_be_bytes(raw)
    }

    /// Liefert die 65-Byte-Bereichsgrenze dieses Schlüssels.
    #[inline]
    pub fn bound(&self) -> KeyBound {
        let mut b = [0u8; KEY_BOUND_LEN];
        b.copy_from_slice(&self.0[0..KEY_BOUND_LEN]);
        KeyBound(b)
    }
}

impl fmt::Debug for KeyBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "KeyBuf({:02x}{:02x}{:02x}{:02x}..)",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

/// Inklusiver Schlüsselbereich mit fester 65-Byte-Länge (SPEC 9.1, D-10).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub struct KeyRange {
    /// Untere Grenze (inklusiv).
    pub lo: KeyBound,
    /// Obere Grenze (inklusiv).
    pub hi: KeyBound,
}

impl KeyRange {
    /// Bereich für einen gesamten Mandanten.
    pub fn tenant(r: Region, t: TenantTag) -> Self {
        let mut lo = [0u8; KEY_BOUND_LEN];
        lo[0] = r.as_u8();
        lo[1..17].copy_from_slice(t.as_bytes());

        let mut hi = [0xFFu8; KEY_BOUND_LEN];
        hi[0] = r.as_u8();
        hi[1..17].copy_from_slice(t.as_bytes());

        Self {
            lo: KeyBound(lo),
            hi: KeyBound(hi),
        }
    }

    /// Bereich für eine Collection.
    pub fn collection(r: Region, t: TenantTag, c: CollTag) -> Self {
        let mut lo = [0u8; KEY_BOUND_LEN];
        lo[0] = r.as_u8();
        lo[1..17].copy_from_slice(t.as_bytes());
        lo[17..33].copy_from_slice(c.as_bytes());

        let mut hi = [0xFFu8; KEY_BOUND_LEN];
        hi[0] = r.as_u8();
        hi[1..17].copy_from_slice(t.as_bytes());
        hi[17..33].copy_from_slice(c.as_bytes());

        Self {
            lo: KeyBound(lo),
            hi: KeyBound(hi),
        }
    }

    /// Bereich für ein Cluster (z. B. ein Betroffener).
    pub fn cluster(r: Region, t: TenantTag, c: CollTag, cl: ClusterTag) -> Self {
        let mut lo = [0u8; KEY_BOUND_LEN];
        lo[0] = r.as_u8();
        lo[1..17].copy_from_slice(t.as_bytes());
        lo[17..33].copy_from_slice(c.as_bytes());
        lo[33..49].copy_from_slice(cl.as_bytes());

        let mut hi = [0xFFu8; KEY_BOUND_LEN];
        hi[0] = r.as_u8();
        hi[1..17].copy_from_slice(t.as_bytes());
        hi[17..33].copy_from_slice(c.as_bytes());
        hi[33..49].copy_from_slice(cl.as_bytes());

        Self {
            lo: KeyBound(lo),
            hi: KeyBound(hi),
        }
    }

    /// Bereich für ein einzelnes Dokument.
    pub fn doc(r: Region, t: TenantTag, c: CollTag, cl: ClusterTag, d: DocTag) -> Self {
        let mut lo = [0u8; KEY_BOUND_LEN];
        lo[0] = r.as_u8();
        lo[1..17].copy_from_slice(t.as_bytes());
        lo[17..33].copy_from_slice(c.as_bytes());
        lo[33..49].copy_from_slice(cl.as_bytes());
        lo[49..65].copy_from_slice(d.as_bytes());

        let mut hi = lo;
        Self {
            lo: KeyBound(lo),
            hi: KeyBound(hi),
        }
    }

    /// Prüft, ob ein 65-Byte-Bound innerhalb des Bereichs liegt.
    #[inline]
    pub fn contains_bound(&self, b: &KeyBound) -> bool {
        &self.lo <= b && b <= &self.hi
    }

    /// Prüft, ob ein 78-Byte-KeyBuf innerhalb des Bereichs liegt.
    #[inline]
    pub fn contains_key(&self, k: &KeyBuf) -> bool {
        let b = k.bound();
        self.contains_bound(&b)
    }
}

/// Element eines Matchers: Entweder ein KeyRange oder ein exakter KeyBuf.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
pub enum MatcherItem {
    /// Bereich fester Länge (65 Bytes).
    Range(KeyRange),
    /// Exakter Schlüssel (78 Bytes).
    Exact(KeyBuf),
}

/// Normalisierter Matcher für Löschbereiche und Exakt-Filter (SPEC 9.1, D-10, TY-05).
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct Matcher(Vec<MatcherItem>);

impl Matcher {
    /// Erzeugt einen neuen Matcher.
    pub fn new(items: Vec<MatcherItem>) -> Self {
        Self(items).normalize()
    }

    /// Liefert die Elemente des Matchers.
    pub fn items(&self) -> &[MatcherItem] {
        &self.0
    }

    /// Prüft, ob ein Schlüssel von einem der Kriterien getroffen wird.
    pub fn matches(&self, k: &KeyBuf) -> bool {
        for item in &self.0 {
            match item {
                MatcherItem::Exact(exact) => {
                    if exact == k {
                        return true;
                    }
                }
                MatcherItem::Range(range) => {
                    if range.contains_key(k) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Normalisiert den Matcher: sortiert und verschmilzt überlappende/angrenzende Bereiche (TY-05).
    pub fn normalize(mut self) -> Self {
        if self.0.is_empty() {
            return self;
        }

        self.0.sort();

        let mut normalized_ranges: Vec<KeyRange> = Vec::new();
        let mut exact_keys: Vec<KeyBuf> = Vec::new();

        for item in self.0 {
            match item {
                MatcherItem::Range(r) => {
                    if let Some(last) = normalized_ranges.last_mut() {
                        // Prüfen auf Überlappung
                        if r.lo <= last.hi {
                            if r.hi > last.hi {
                                last.hi = r.hi;
                            }
                        } else {
                            normalized_ranges.push(r);
                        }
                    } else {
                        normalized_ranges.push(r);
                    }
                }
                MatcherItem::Exact(k) => {
                    exact_keys.push(k);
                }
            }
        }

        // Filtern von exakten Keys, die bereits durch einen Bereich abgedeckt sind
        let mut final_items: Vec<MatcherItem> = Vec::new();
        for r in normalized_ranges {
            final_items.push(MatcherItem::Range(r));
        }

        for k in exact_keys {
            let already_covered = final_items.iter().any(|item| match item {
                MatcherItem::Range(r) => r.contains_key(&k),
                MatcherItem::Exact(_) => false,
            });
            if !already_covered && !final_items.contains(&MatcherItem::Exact(k)) {
                final_items.push(MatcherItem::Exact(k));
            }
        }

        final_items.sort();
        Self(final_items)
    }
}
