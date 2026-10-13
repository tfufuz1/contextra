//! Wiring entry point for `contextra-avv-generator` (W3-01 / Rule 7).

use crate::{render_avv_markdown, AvvContext, TechnicalMeasure};
use contextra_ports::clock::Clock;
use contextra_types::{ContextraError, TenantId};
use std::fmt;
use std::sync::Arc;

/// Entry point wiring structure for AVV document generation bound to a [`Clock`] port.
#[derive(Clone)]
pub struct AvvWiring {
    ctx: AvvContext,
    clock: Arc<dyn Clock>,
}

impl fmt::Debug for AvvWiring {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AvvWiring")
            .field("ctx", &self.ctx)
            .field("clock", &"<dyn Clock>")
            .finish()
    }
}

impl AvvWiring {
    /// Builds a new [`AvvWiring`] instance.
    pub fn build(
        controller_name: impl Into<String>,
        processor_name: impl Into<String>,
        tenant_id: TenantId,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, ContextraError> {
        let controller = controller_name.into();
        let processor = processor_name.into();

        if controller.trim().is_empty() {
            return Err(ContextraError::InvalidInput(
                "Verantwortlicher (controller_name) darf nicht leer sein.".to_string(),
            ));
        }
        if processor.trim().is_empty() {
            return Err(ContextraError::InvalidInput(
                "Auftragsverarbeiter (processor_name) darf nicht leer sein.".to_string(),
            ));
        }

        let ctx = AvvContext::new(controller, processor, tenant_id);

        Ok(Self { ctx, clock })
    }

    /// Returns a reference to the underlying [`AvvContext`].
    pub fn context(&self) -> &AvvContext {
        &self.ctx
    }

    /// Returns a mutable reference to the underlying [`AvvContext`].
    pub fn context_mut(&mut self) -> &mut AvvContext {
        &mut self.ctx
    }

    /// Returns a reference to the injected clock port.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Customizes technical measures on the AVV context.
    pub fn with_technical_measures(mut self, measures: Vec<TechnicalMeasure>) -> Self {
        self.ctx.technical_measures = measures;
        self
    }

    /// Appends a subprocessor to the AVV context.
    pub fn add_subprocessor(mut self, subprocessor: impl Into<String>) -> Self {
        self.ctx.subprocessors.push(subprocessor.into());
        self
    }

    /// Renders the AVV context into a formatted Markdown document.
    pub fn render(&self) -> Result<String, ContextraError> {
        render_avv_markdown(&self.ctx).map_err(|e| ContextraError::Internal(e.to_string()))
    }
}
