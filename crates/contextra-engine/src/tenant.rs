//! Mandantengebundenes Handle (SPEC §7.1, D-07).

use std::sync::Arc;
use contextra_types::{Result, TenantContext, TenantId};
use crate::collection::{Collection, CollectionOptions};
use crate::instance::Contextra;

/// Mandantengebundenes Handle auf die Engine.
#[derive(Clone)]
pub struct Tenant {
    pub(crate) engine: Arc<Contextra>,
    pub(crate) ctx: TenantContext,
    pub(crate) tenant_id: TenantId,
}

impl Tenant {
    /// Liefert den zugehörigen Mandantenkontext.
    pub fn context(&self) -> &TenantContext {
        &self.ctx
    }

    /// Liefert die Mandanten-ID.
    pub fn tenant_id(&self) -> TenantId {
        self.tenant_id
    }

    /// Öffnet oder erstellt eine Collection für diesen Mandanten.
    pub async fn collection(&self, name: &str, opts: CollectionOptions) -> Result<Arc<Collection>> {
        self.engine.collection_for_tenant(name, self.tenant_id, opts).await
    }
}
