//! # Key-Codec v2 (Binär, ordnungserhaltend, clusterbewusst)
//!
//! Format v2 Aufbau:
//! `[0x02][region: u16 BE][tenant: u64 BE][collection: u64 BE][cluster: u32 BE][tag: u8][entity_id: u64 BE]`
//!
//! Invarianten:
//! - Feste Breiten, Big-Endian -> Lexikografische Byte-Ordnung entspricht Integer-Ordnung.
//! - Präfix-Byte `0x02` unterscheidet v2 von v1 (`t:`).
//! - Keine Trennzeichen-Mehrdeutigkeit.

use contextra_ids::{CollectionId, DocId, TenantId};

/// Formatversionsbyte für Key-Codec v2.
pub const KEY_VERSION_V2: u8 = 0x02;

/// Standardregion falls keine explizite Region angegeben wird.
pub const DEFAULT_REGION: u16 = 0;

/// Standardcluster falls kein expliziter Cluster angegeben wird.
pub const DEFAULT_CLUSTER: u32 = 0;

/// Typ-Tag für Keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum KeyTag {
    Chunk = 1,
    Graph = 2,
    Custom(u8) = 255,
}

impl From<u8> for KeyTag {
    fn from(b: u8) -> Self {
        match b {
            1 => KeyTag::Chunk,
            2 => KeyTag::Graph,
            other => KeyTag::Custom(other),
        }
    }
}

impl KeyTag {
    #[inline]
    pub const fn to_u8(self) -> u8 {
        match self {
            KeyTag::Chunk => 1,
            KeyTag::Graph => 2,
            KeyTag::Custom(b) => b,
        }
    }
}

/// Strukturierte Repräsentation eines decodierten v2-Schlüssels.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecodedKeyV2 {
    pub region: u16,
    pub tenant_id: TenantId,
    pub collection_id: CollectionId,
    pub cluster: u32,
    pub tag: KeyTag,
    pub entity_id: u64,
}

/// Encoder & Präfix-Generator für v2 Key-Format.
#[derive(Debug, Clone)]
pub struct KeyCodecV2 {
    region: u16,
    tenant_id: TenantId,
}

impl KeyCodecV2 {
    /// Erstellt einen neuen `KeyCodecV2` mit Standard-Region.
    pub fn new(tenant_id: TenantId) -> Self {
        Self::with_region(DEFAULT_REGION, tenant_id)
    }

    /// Erstellt einen neuen `KeyCodecV2` mit expliziter Region.
    pub fn with_region(region: u16, tenant_id: TenantId) -> Self {
        Self { region, tenant_id }
    }

    /// Region dieses Codecs.
    #[inline]
    pub const fn region(&self) -> u16 {
        self.region
    }

    /// TenantId dieses Codecs.
    #[inline]
    pub const fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Scan-Präfix für gesamten Tenant: `[0x02][region: 2B][tenant: 8B]` (11 Bytes).
    #[inline]
    pub fn tenant_prefix(&self) -> [u8; 11] {
        let mut buf = [0u8; 11];
        buf[0] = KEY_VERSION_V2;
        buf[1..3].copy_from_slice(&self.region.to_be_bytes());
        buf[3..11].copy_from_slice(&self.tenant_id.inner().to_be_bytes());
        buf
    }

    /// Scan-Präfix für Collection: `[0x02][region: 2B][tenant: 8B][collection: 8B]` (19 Bytes).
    #[inline]
    pub fn collection_prefix(&self, collection: &CollectionId) -> [u8; 19] {
        let mut buf = [0u8; 19];
        buf[0] = KEY_VERSION_V2;
        buf[1..3].copy_from_slice(&self.region.to_be_bytes());
        buf[3..11].copy_from_slice(&self.tenant_id.inner().to_be_bytes());
        buf[11..19].copy_from_slice(&collection.0.to_be_bytes());
        buf
    }

    /// Scan-Präfix für Cluster: `[0x02][region: 2B][tenant: 8B][collection: 8B][cluster: 4B]` (23 Bytes).
    #[inline]
    pub fn cluster_prefix(&self, collection: &CollectionId, cluster: u32) -> [u8; 23] {
        let mut buf = [0u8; 23];
        buf[0] = KEY_VERSION_V2;
        buf[1..3].copy_from_slice(&self.region.to_be_bytes());
        buf[3..11].copy_from_slice(&self.tenant_id.inner().to_be_bytes());
        buf[11..19].copy_from_slice(&collection.0.to_be_bytes());
        buf[19..23].copy_from_slice(&cluster.to_be_bytes());
        buf
    }

    /// Kodiert einen Chunk-Key mit Standard-Cluster: 32 Bytes.
    #[inline]
    pub fn encode_chunk_key(&self, collection: &CollectionId, doc_id: DocId) -> [u8; 32] {
        self.encode_chunk_key_with_cluster(collection, DEFAULT_CLUSTER, doc_id)
    }

    /// Kodiert einen Chunk-Key mit explizitem Cluster: 32 Bytes.
    #[inline]
    pub fn encode_chunk_key_with_cluster(
        &self,
        collection: &CollectionId,
        cluster: u32,
        doc_id: DocId,
    ) -> [u8; 32] {
        self.encode_key(collection, cluster, KeyTag::Chunk, doc_id.0)
    }

    /// Kodiert einen Graph-Key mit explizitem Cluster: 32 Bytes.
    #[inline]
    pub fn encode_graph_key_with_cluster(
        &self,
        collection: &CollectionId,
        cluster: u32,
        entity_id: u64,
    ) -> [u8; 32] {
        self.encode_key(collection, cluster, KeyTag::Graph, entity_id)
    }

    /// Generischer v2 Key-Encoder (32 Bytes).
    #[inline]
    pub fn encode_key(
        &self,
        collection: &CollectionId,
        cluster: u32,
        tag: KeyTag,
        entity_id: u64,
    ) -> [u8; 32] {
        let mut buf = [0u8; 32];
        buf[0] = KEY_VERSION_V2;
        buf[1..3].copy_from_slice(&self.region.to_be_bytes());
        buf[3..11].copy_from_slice(&self.tenant_id.inner().to_be_bytes());
        buf[11..19].copy_from_slice(&collection.0.to_be_bytes());
        buf[19..23].copy_from_slice(&cluster.to_be_bytes());
        buf[23] = tag.to_u8();
        buf[24..32].copy_from_slice(&entity_id.to_be_bytes());
        buf
    }

    /// Prüft ob ein gegebener Byte-Slice ein v2 Key ist.
    #[inline]
    pub fn is_v2_key(key: &[u8]) -> bool {
        key.first() == Some(&KEY_VERSION_V2) && key.len() == 32
    }

    /// Extrahiert die TenantId aus einem v2 Key.
    #[inline]
    pub fn decode_tenant_id(key: &[u8]) -> Option<TenantId> {
        if !Self::is_v2_key(key) {
            return None;
        }
        let tenant_bytes: [u8; 8] = key.get(3..11)?.try_into().ok()?;
        let raw = u64::from_be_bytes(tenant_bytes);
        TenantId::try_new(raw).ok()
    }

    /// Extrahiert den Cluster aus einem v2 Key.
    #[inline]
    pub fn extract_cluster(key: &[u8]) -> Option<u32> {
        if !Self::is_v2_key(key) {
            return None;
        }
        let cluster_bytes: [u8; 4] = key.get(19..23)?.try_into().ok()?;
        Some(u32::from_be_bytes(cluster_bytes))
    }

    /// Dekodiert einen v2 Key vollständig.
    pub fn decode_key(key: &[u8]) -> Option<DecodedKeyV2> {
        if !Self::is_v2_key(key) {
            return None;
        }
        let region_bytes: [u8; 2] = key.get(1..3)?.try_into().ok()?;
        let region = u16::from_be_bytes(region_bytes);

        let tenant_bytes: [u8; 8] = key.get(3..11)?.try_into().ok()?;
        let tenant_id = TenantId::try_new(u64::from_be_bytes(tenant_bytes)).ok()?;

        let col_bytes: [u8; 8] = key.get(11..19)?.try_into().ok()?;
        let collection_id = CollectionId(u64::from_be_bytes(col_bytes));

        let cluster_bytes: [u8; 4] = key.get(19..23)?.try_into().ok()?;
        let cluster = u32::from_be_bytes(cluster_bytes);

        let tag = KeyTag::from(*key.get(23)?);

        let entity_bytes: [u8; 8] = key.get(24..32)?.try_into().ok()?;
        let entity_id = u64::from_be_bytes(entity_bytes);

        Some(DecodedKeyV2 {
            region,
            tenant_id,
            collection_id,
            cluster,
            tag,
            entity_id,
        })
    }
}
