//! Deterministic scope construction for engine-owned namespaces.

use contextra_ports::erasure::{DocScope, KeyPattern, KeyScope, ResolvedScope, Verdict};
use contextra_types::{DocId, EraseTarget, ErasureError, ErasureId, TargetKind, TenantId};
use std::collections::BTreeSet;

/// Pure scope resolver helpers. All generated patterns are physical tenant keys, matching
/// the raw-key view used by the store's erasure kernel.
pub struct ScopeResolverImpl;

impl ScopeResolverImpl {
    pub fn collection(
        id: ErasureId,
        tenant: TenantId,
        collection: &str,
    ) -> Result<ResolvedScope, ErasureError> {
        let target = EraseTarget::Collection {
            tenant,
            collection: collection.to_owned(),
        };
        target.validate()?;
        let mut rules = Vec::new();
        let ns = tenant_prefix(tenant);
        let col = format!("__col:{collection}:\0");
        rules.push((KeyPattern::Prefix(prefixed(&ns, col.as_bytes())), Verdict::In));
        let txt = format!("__txt:{collection}:");
        rules.push((KeyPattern::Prefix(prefixed(&ns, txt.as_bytes())), Verdict::In));
        let graph = format!("__graph:prov:{collection}:");
        rules.push((KeyPattern::Prefix(prefixed(&ns, graph.as_bytes())), Verdict::In));
        let idx = prefixed(&ns, format!("__col_idx:\0{collection}").as_bytes());
        rules.push((KeyPattern::Exact(idx), Verdict::In));
        let keys = KeyScope::new(rules)?;
        Ok(ResolvedScope::new(
            id,
            TargetKind::Collection,
            &target,
            keys,
            vec![DocScope::Collection { tenant, collection: collection.to_owned() }],
            0,
        ))
    }

    pub fn tenant(id: ErasureId, tenant: TenantId) -> Result<ResolvedScope, ErasureError> {
        let target = EraseTarget::Tenant { tenant };
        target.validate()?;
        let prefix = tenant_prefix(tenant);
        let keys = KeyScope::new(vec![(KeyPattern::Prefix(prefix), Verdict::In)])?;
        Ok(ResolvedScope::new(
            id,
            TargetKind::Tenant,
            &target,
            keys,
            vec![DocScope::Tenant { tenant }],
            0,
        ))
    }

    pub fn document(
        id: ErasureId,
        tenant: TenantId,
        collection: &str,
        key: &str,
        user_key: Vec<u8>,
        doc_key: Vec<u8>,
        reverse_value: &[u8],
    ) -> Result<ResolvedScope, ErasureError> {
        Self::document_with_extras(
            id,
            tenant,
            collection,
            key,
            user_key,
            doc_key,
            reverse_value,
            Vec::new(),
            0,
        )
    }

    /// Document scope including text rows, relation rows and the graph provenance row.
    ///
    /// `extra_keys` are collection-logical exact keys (no tenant prefix yet). The graph
    /// provenance key `__graph:prov:{collection}:{doc}` is added here (08 §4);
    /// `unprovenanced` is the count of graph objects without provenance.
    #[allow(clippy::too_many_arguments)]
    pub fn document_with_extras(
        id: ErasureId,
        tenant: TenantId,
        collection: &str,
        key: &str,
        user_key: Vec<u8>,
        doc_key: Vec<u8>,
        reverse_value: &[u8],
        extra_keys: Vec<Vec<u8>>,
        unprovenanced: u64,
    ) -> Result<ResolvedScope, ErasureError> {
        let target = EraseTarget::Document {
            tenant,
            collection: collection.to_owned(),
            key: key.to_owned(),
        };
        target.validate()?;
        let reverse = serde_json::from_slice::<crate::collection::StoredDocumentMeta>(reverse_value)
            .map_err(|_| ErasureError::InvalidTarget("document not found"))?;
        if reverse.id.as_bytes() != key.as_bytes() {
            return Err(ErasureError::ScopeAmbiguous);
        }
        let doc = DocId::from_key(key).map_err(|_| ErasureError::InvalidTarget("invalid document key"))?;
        let ns = tenant_prefix(tenant);
        let mut rules = vec![
            (KeyPattern::Exact(prefixed(&ns, &user_key)), Verdict::In),
            (KeyPattern::Exact(prefixed(&ns, &doc_key)), Verdict::In),
        ];
        for extra in &extra_keys {
            rules.push((KeyPattern::Exact(prefixed(&ns, extra)), Verdict::In));
        }
        let prov = format!("__graph:prov:{collection}:{}", doc.inner());
        rules.push((KeyPattern::Exact(prefixed(&ns, prov.as_bytes())), Verdict::In));
        let keys = KeyScope::new(rules)?;
        // Tenant isolation check (I-8): every In-rule carries the tenant prefix.
        if !keys.rules().iter().all(|(p, _)| p.bytes().starts_with(&ns)) {
            return Err(ErasureError::TenantViolation);
        }
        let mut docs = BTreeSet::new();
        docs.insert(doc);
        Ok(ResolvedScope::new(
            id,
            TargetKind::Document,
            &target,
            keys,
            vec![DocScope::Set { tenant, collection: collection.to_owned(), docs }],
            unprovenanced,
        ))
    }
}

fn tenant_prefix(tenant: TenantId) -> Vec<u8> {
    format!("t:{}:", tenant.inner()).into_bytes()
}

fn prefixed(prefix: &[u8], key: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(prefix.len() + key.len());
    out.extend_from_slice(prefix);
    out.extend_from_slice(key);
    out
}
