# Fehlerabbildung: contextra-prompt-guard

Entkopplung der Fehlerbehandlung aus `contextra-mcp` in `contextra-prompt-guard::GuardError`.

## 1. Fehler-Enum `GuardError`

```rust
#[derive(Debug, Error, PartialEq, Eq)]
#[non_exhaustive]
pub enum GuardError {
    #[error("I/O-Fehler beim Guard-Betrieb: {0}")]
    Io(String),

    #[error("Format- oder Parsefehler im Guard-Schema: {0}")]
    Serialization(String),

    #[error("Ungültige Guard-Konfiguration: {0}")]
    InvalidConfiguration(String),

    #[error("Fehler beim Dekodieren von Inhalten: {0}")]
    DecodeError(String),
}
```

## 2. Zuordnung zu `ContextraError`

| `GuardError`-Variante | Herkunft / Auslöser | Verhalten in `mcp` |
|---|---|---|
| `Io(String)` | `std::fs`-Fehler beim Laden von Konfiguration oder Mustern | Fehlermeldung wird propagiert oder Fallback genutzt |
| `Serialization(String)` | `serde_json`-Fehler beim Parsen | Fehlermeldung wird als Fehler zurückgegeben |
| `InvalidConfiguration(String)` | Inkonsistente Muster-Konfiguration | Konfigurationsablehnung |
| `DecodeError(String)` | Ungültige Encodings in rekursiver Base64-Auflösung | Ignoriert oder als Non-Match bewertet |
