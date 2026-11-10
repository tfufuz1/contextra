# Fehlerabbildungsmatrix: infer (INFER-3)

## 1. contextra-infer-ollama: `OllamaError` ↔ `ContextraError`

| OllamaError | ContextraError | Mapping-Art |
|---|---|---|
| `InvalidInput(msg)` | `ContextraError::InvalidInput(msg)` | Bitgleich / Direkt |
| `ModelNotFound(msg)` | `ContextraError::NotFound(msg)` | Bitgleich / Direkt |
| `Network(msg)` | `ContextraError::Io(ConnectionRefused, msg)` | Bitgleich / IO |
| `Timeout(msg)` | `ContextraError::Io(TimedOut, msg)` | Bitgleich / IO |
| `Server(msg)` | `ContextraError::Storage(msg)` | Bitgleich |
| `Deserialization(msg)` | `ContextraError::Internal(msg)` | Bitgleich |
| `EgressViolation(msg)` | `ContextraError::PolicyViolation(msg)` | Bitgleich |
| `CircuitBreakerOpen` | `ContextraError::Internal("circuit breaker open")` | Bitgleich |
| `Internal(msg)` | `ContextraError::Internal(msg)` | Bitgleich |

## 2. contextra-infer-candle: `Error` ↔ `ContextraError`

| Candle Error | ContextraError | Mapping-Art |
|---|---|---|
| `Io(e)` | `ContextraError::Io(e)` | Bitgleich / IO |
| `ModelLoad { reason }` | `ContextraError::Internal(reason)` | Bitgleich |
| `Tokenizer(msg)` | `ContextraError::Internal(msg)` | Bitgleich |
| `Inference(msg)` | `ContextraError::Internal(msg)` | Bitgleich |
| `InvalidInput(msg)` | `ContextraError::InvalidInput(msg)` | Bitgleich |
| `ResourceExceeded(msg)` | `ContextraError::limit_exceeded(0, msg)` | Bitgleich |
| `PolicyViolation(msg)` | `ContextraError::PolicyViolation(msg)` | Bitgleich |
| `BackendUnavailable(msg)` | `ContextraError::NotFound(msg)` | Bitgleich |
