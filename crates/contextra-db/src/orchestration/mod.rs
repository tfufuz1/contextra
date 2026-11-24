//! Autonome Typestate-basierte State-Machine und Service-Container für Workflows.
//!
//! Dieses Modul entkoppelt Workflow-Schritte, RAG-Pipelines und Tool-Ausführungen
//! von konkreten Storage-, Vektor- und Text-Backends durch Dependency Injection (Ports/Traits).

// Ports dependency injection
use contextra_types::{ContextraError, Result};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

/// Service-Container für Dependency Injection in State-Machines und Orchestratoren.
pub struct EngineServiceContainer<S, V, T>
where
    S: ?Sized,
    V: ?Sized,
    T: ?Sized,
{
    storage: Arc<S>,
    vector_index: Arc<V>,
    text_index: Arc<T>,
}

impl<S, V, T> EngineServiceContainer<S, V, T>
where
    S: ?Sized,
    V: ?Sized,
    T: ?Sized,
{
    /// Erzeugt einen neuen typisierten Service-Container.
    pub fn new(storage: Arc<S>, vector_index: Arc<V>, text_index: Arc<T>) -> Self {
        Self {
            storage,
            vector_index,
            text_index,
        }
    }

    /// Liefert Referenz auf die StorageEngine.
    pub fn storage(&self) -> &Arc<S> {
        &self.storage
    }

    /// Liefert Referenz auf den VectorIndex.
    pub fn vector_index(&self) -> &Arc<V> {
        &self.vector_index
    }

    /// Liefert Referenz auf den TextIndex.
    pub fn text_index(&self) -> &Arc<T> {
        &self.text_index
    }
}

/// Dynamischer Workflow-Kontext zur Übergabe von Zwischenergebnissen zwischen Schritten.
#[derive(Debug, Default, Clone)]
pub struct WorkflowContext {
    data: std::collections::HashMap<String, String>,
}

impl WorkflowContext {
    /// Erstellt einen leeren Kontext.
    pub fn new() -> Self {
        Self {
            data: std::collections::HashMap::new(),
        }
    }

    /// Setzt einen Schlüssel-Wert-Eintrag.
    pub fn set(&mut self, key: impl Into<String>, val: impl Into<String>) {
        self.data.insert(key.into(), val.into());
    }

    /// Liest einen Wert aus.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
}

/// Trait für zustandslose oder isolierte Workflow-Schritte (Dyn-kompatibel via Pin<Box<dyn Future>>).
pub trait WorkflowStep: Send + Sync {
    /// Name oder Bezeichner des Schritts.
    fn name(&self) -> &str;

    /// Führt den Schritt asynchron auf einem Kontext-Objekt aus.
    fn run<'a>(
        &'a self,
        context: &'a mut WorkflowContext,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
}

// Typestates
pub struct Draft;
pub struct Prepared;
pub struct Executing;
pub struct Completed;
pub struct Failed;

/// Typestate-basierte State Machine für Workflows.
pub struct WorkflowState<State> {
    workflow_id: String,
    steps: Vec<Box<dyn WorkflowStep>>,
    context: WorkflowContext,
    completed_steps: usize,
    error: Option<ContextraError>,
    _state: PhantomData<State>,
}

impl WorkflowState<Draft> {
    /// Startet die Erstellung eines neuen Workflows im Zustand `Draft`.
    pub fn new(workflow_id: impl Into<String>) -> Self {
        Self {
            workflow_id: workflow_id.into(),
            steps: Vec::new(),
            context: WorkflowContext::new(),
            completed_steps: 0,
            error: None,
            _state: PhantomData,
        }
    }

    /// Fügt dem Entwurf einen Schritt hinzu.
    pub fn add_step(mut self, step: Box<dyn WorkflowStep>) -> Self {
        self.steps.push(step);
        self
    }

    /// Validiert und überführt den Workflow in den Zustand `Prepared`.
    pub fn prepare(self) -> Result<WorkflowState<Prepared>> {
        if self.steps.is_empty() {
            return Err(ContextraError::Internal(
                "Workflow enthält keine Ausführungsschritte".to_string(),
            ));
        }
        Ok(WorkflowState {
            workflow_id: self.workflow_id,
            steps: self.steps,
            context: self.context,
            completed_steps: 0,
            error: None,
            _state: PhantomData,
        })
    }
}

impl WorkflowState<Prepared> {
    /// Beginnt die Ausführung und wechselt in den Typestate `Executing`.
    pub fn start_execution(self) -> WorkflowState<Executing> {
        WorkflowState {
            workflow_id: self.workflow_id,
            steps: self.steps,
            context: self.context,
            completed_steps: 0,
            error: None,
            _state: PhantomData,
        }
    }
}

impl WorkflowState<Executing> {
    /// Führt alle Schritte sequentiell aus und wechselt je nach Ergebnis in `Completed` oder `Failed`.
    pub async fn execute_all(
        mut self,
    ) -> std::result::Result<WorkflowState<Completed>, WorkflowState<Failed>> {
        for step in &self.steps {
            if let Err(e) = step.run(&mut self.context).await {
                return Err(WorkflowState {
                    workflow_id: self.workflow_id,
                    steps: self.steps,
                    context: self.context,
                    completed_steps: self.completed_steps,
                    error: Some(e),
                    _state: PhantomData,
                });
            }
            self.completed_steps = self.completed_steps.saturating_add(1);
        }

        Ok(WorkflowState {
            workflow_id: self.workflow_id,
            steps: self.steps,
            context: self.context,
            completed_steps: self.completed_steps,
            error: None,
            _state: PhantomData,
        })
    }
}

impl WorkflowState<Completed> {
    /// Liefert die ID des abgeschlossenen Workflows.
    pub fn workflow_id(&self) -> &str {
        &self.workflow_id
    }

    /// Liefert die Anzahl erfolgreich ausgeführter Schritte.
    pub fn completed_steps(&self) -> usize {
        self.completed_steps
    }

    /// Gibt eine Referenz auf den resultierenden Kontext zurück.
    pub fn context(&self) -> &WorkflowContext {
        &self.context
    }
}

impl WorkflowState<Failed> {
    /// Liefert die ID des fehlgeschlagenen Workflows.
    pub fn workflow_id(&self) -> &str {
        &self.workflow_id
    }

    /// Liefert die Anzahl bis zum Abbruch ausgeführter Schritte.
    pub fn completed_steps(&self) -> usize {
        self.completed_steps
    }

    /// Liefert den aufgetretenen Fehler.
    pub fn error(&self) -> Option<&ContextraError> {
        self.error.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockStep {
        name: &'static str,
        append_key: String,
        append_value: String,
    }

    impl WorkflowStep for MockStep {
        fn name(&self) -> &str {
            self.name
        }

        fn run<'a>(
            &'a self,
            context: &'a mut WorkflowContext,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
            Box::pin(async move {
                context.set(&self.append_key, &self.append_value);
                Ok(())
            })
        }
    }

    struct FailingStep;

    impl WorkflowStep for FailingStep {
        fn name(&self) -> &str {
            "failing_step"
        }

        fn run<'a>(
            &'a self,
            _context: &'a mut WorkflowContext,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
            Box::pin(async move { Err(ContextraError::Internal("Simulierter Fehler".to_string())) })
        }
    }

    #[tokio::test]
    async fn test_typestate_workflow_success() {
        let draft = WorkflowState::new("wf-1")
            .add_step(Box::new(MockStep {
                name: "step-1",
                append_key: "k1".to_string(),
                append_value: "v1".to_string(),
            }))
            .add_step(Box::new(MockStep {
                name: "step-2",
                append_key: "k2".to_string(),
                append_value: "done".to_string(),
            }));

        let prepared = draft
            .prepare()
            .unwrap_or_else(|e| panic!("prepare failed: {e}"));
        let executing = prepared.start_execution();
        let completed = executing
            .execute_all()
            .await
            .unwrap_or_else(|_| panic!("expected success"));

        assert_eq!(completed.workflow_id(), "wf-1");
        assert_eq!(completed.completed_steps(), 2);
        assert_eq!(completed.context().get("k1"), Some("v1"));
        assert_eq!(completed.context().get("k2"), Some("done"));
    }

    #[tokio::test]
    async fn test_typestate_workflow_failure() {
        let draft = WorkflowState::new("wf-fail").add_step(Box::new(FailingStep));

        let prepared = draft
            .prepare()
            .unwrap_or_else(|e| panic!("prepare failed: {e}"));
        let executing = prepared.start_execution();
        let failed = executing
            .execute_all()
            .await
            .err()
            .unwrap_or_else(|| panic!("expected failure"));

        assert_eq!(failed.workflow_id(), "wf-fail");
        assert_eq!(failed.completed_steps(), 0);
        assert!(failed.error().is_some());
    }

    #[tokio::test]
    async fn test_engine_service_container_wiring() {
        struct MockStorage;
        struct MockVector;
        struct MockText;

        let container = EngineServiceContainer::new(
            Arc::new(MockStorage),
            Arc::new(MockVector),
            Arc::new(MockText),
        );

        assert_eq!(Arc::strong_count(container.storage()), 1);
        assert_eq!(Arc::strong_count(container.vector_index()), 1);
        assert_eq!(Arc::strong_count(container.text_index()), 1);
    }
}
