// FILE-CONTEXT
// STAND: 2026-10-13
// ZWECK: Cognition & MultiStep Composition Root Wiring for contextra facade (Ring 4).
// INVARIANTEN: #![forbid(unsafe_code)]; Sets consolidation_launcher on ContextraConfig when consolidation_enabled.

#![forbid(unsafe_code)]

#[cfg(feature = "cognition")]
use std::sync::Arc;

#[cfg(feature = "cognition")]
use contextra_cognition::wiring::{CognitionWiring, CognitionWiringDeps};
#[cfg(feature = "cognition")]
use contextra_core::error::ContextraError;
#[cfg(feature = "cognition")]
use contextra_db::{ConsolidationLauncher, ContextraConfig};
#[cfg(feature = "cognition")]
use contextra_ports::clock::Clock;

/// Bundles Cognition & MultiStep pipeline components.
#[cfg(feature = "cognition")]
pub struct CognitionWiringBundle {
    pub cognition: CognitionWiring,
}

#[cfg(feature = "cognition")]
impl CognitionWiringBundle {
    /// Builds `CognitionWiringBundle` and configures `config.consolidation_launcher`.
    pub fn build(
        config: &mut ContextraConfig,
        clock: Arc<dyn Clock>,
    ) -> Result<Self, ContextraError> {
        let cognition = CognitionWiring::build(CognitionWiringDeps {
            clock: Some(clock.clone()),
            ..Default::default()
        })
        .map_err(|e| ContextraError::PolicyViolation(e.to_string()))?;

        if config.consolidation_enabled {
            let launcher: ConsolidationLauncher = Arc::new(
                |collection, interval, max_calls, cancel_token| {
                    tokio::spawn(async move {
                        let _ = contextra_db::execute_background_consolidation(
                            collection,
                            interval,
                            max_calls,
                            cancel_token,
                            None,
                            None,
                            None,
                        )
                        .await;
                    })
                },
            );
            config.consolidation_launcher = Some(launcher);
        }

        Ok(Self { cognition })
    }
}
