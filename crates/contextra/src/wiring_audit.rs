// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Audit-Export & AVV-Generator Composition Root Wiring for contextra facade (Ring 4).
// INVARIANTEN: #![forbid(unsafe_code)]; Exposes AuditExport & AVV capabilities cleanly via facade extension.

#![forbid(unsafe_code)]

#[cfg(feature = "audit-export")]
use std::sync::Arc;

#[cfg(feature = "audit-export")]
use contextra_audit_export::wiring::AuditExportWiring;
#[cfg(feature = "audit-export")]
use contextra_core::error::ContextraError;
#[cfg(feature = "audit-export")]
use contextra_ports::clock::Clock;
#[cfg(feature = "audit-export")]
use contextra_types::TenantId;

#[cfg(feature = "avv-generator")]
use contextra_avv_generator::wiring::AvvWiring;

/// Bundles compliance export capabilities (AuditExport, AVV).
#[cfg(feature = "audit-export")]
#[derive(Clone, Debug)]
pub struct AuditWiringBundle {
    pub clock: Arc<dyn Clock>,
    #[cfg(feature = "avv-generator")]
    pub avv: Option<AvvWiring>,
}

#[cfg(feature = "audit-export")]
impl AuditWiringBundle {
    /// Constructs a new `AuditWiringBundle`.
    pub fn build(tenant_id: TenantId, clock: Arc<dyn Clock>) -> Result<Self, ContextraError> {
        #[cfg(feature = "avv-generator")]
        let avv = AvvWiring::build(
            "Contextra Controller",
            "Contextra Engine",
            tenant_id,
            clock.clone(),
        )
        .ok();

        Ok(Self {
            clock,
            #[cfg(feature = "avv-generator")]
            avv,
        })
    }

    /// Renders AVV markdown if AVV generator feature is active and configured.
    #[cfg(feature = "avv-generator")]
    pub fn render_avv_markdown(&self) -> Result<String, ContextraError> {
        if let Some(ref avv) = self.avv {
            Ok(avv.render_markdown())
        } else {
            Err(ContextraError::CapabilityUnsupported {
                capability: "avv_generator".to_string(),
                reason: "AVV generator feature is not active".to_string(),
            })
        }
    }
}

/// Extension trait providing compliance export capabilities on Contextra facade instances.
#[cfg(feature = "audit-export")]
pub trait ContextraComplianceExt {
    /// Renders Art. 28 DSGVO AVV contract markdown template.
    fn render_avv_markdown(&self) -> Result<String, ContextraError>;
}

#[cfg(feature = "audit-export")]
impl ContextraComplianceExt for contextra_db::Contextra {
    fn render_avv_markdown(&self) -> Result<String, ContextraError> {
        let bundle = AuditWiringBundle::build(TenantId::SYSTEM, self.license_gate().clock())?;
        #[cfg(feature = "avv-generator")]
        {
            bundle.render_avv_markdown()
        }
        #[cfg(not(feature = "avv-generator"))]
        {
            Err(ContextraError::CapabilityUnsupported {
                capability: "avv_generator".to_string(),
                reason: "AVV generator feature is not enabled".to_string(),
            })
        }
    }
}
