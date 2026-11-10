# Fehlerabbildung `contextra-graph` (GR6.2)

Gemäß SPEC-v2 §5.3 bildet `contextra-graph` interne Fehler (`GraphError`, `GraphMutationError`) auf kanonische `ContextraError`-Varianten ab. Die Umwandlungen befinden sich in `src/compat/error.rs` hinter den Kompatibilitäts-Features (`types-compat` / `ports-compat`).

## Abbildungstabelle

| Lokale Variante (`GraphError`) | `ContextraError`-Zielvariante | Display-Format |
|---|---|---|
| `InvalidInput(msg)` | `ContextraError::InvalidInput(msg)` | `"Invalid input: {msg}"` |
| `NotFound(msg)` | `ContextraError::NotFound(msg)` | `"Not found: {msg}"` |
| `PolicyViolation(msg)` | `ContextraError::PolicyViolation(msg)` | `"Policy violation: {msg}"` |
| `Serialization(msg)` | `ContextraError::Serialization(msg)` | `"Serialization error: {msg}"` |
| `Storage(msg)` | `ContextraError::Storage(msg)` | `"Storage error: {msg}"` |
| `BudgetExhausted(msg)` | `ContextraError::Internal(format!("Budget exhausted: {msg}"))` | `"Budget exhausted: {msg}"` |
| `Mutation(e)` | `ContextraError::Internal(e.to_string())` | `"Mutation error: {e}"` |
| `Internal(msg)` | `ContextraError::Internal(msg)` | `"Internal graph error: {msg}"` |

### `GraphMutationError` Abbildung

| Lokale Variante (`GraphMutationError`) | `ContextraError`-Zielvariante |
|---|---|
| `LockAcquisitionTimeout(msg)` | `ContextraError::Transaction(format!("Lock acquisition timed out: {msg}"))` |
| `RoleBindingInvalid(msg)` | `ContextraError::InvalidInput(format!("Invalid role binding: {msg}"))` |
| `EpochReclamationPending(epoch)` | `ContextraError::Conflict(format!("Epoch reclamation pending for epoch: {epoch}"))` |
| `InsufficientParticipants { expected, found }` | `ContextraError::InvalidInput(...)` |
| `HyperedgeNotFound(id)` | `ContextraError::NotFound(format!("Hyperedge not found: {id}"))` |
| `InvalidWeight { weight, reason }` | `ContextraError::InvalidInput(format!("Invalid hyperedge weight {weight}: {reason}"))` |
| `DuplicateHyperEdgeId(id)` | `ContextraError::Conflict(format!("Duplicate hyperedge ID: {id}"))` |
| `Internal(msg)` | `ContextraError::Internal(msg)` |
