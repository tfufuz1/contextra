# Fehlerabbildung contextra-text: TextError <-> ContextraError (TX6.2 / SPEC §5.3)

Gemäß SPEC-v2 §5.3, B12 gilt: Display und Variante bleiben bitgleich. Table-driven Test sichert die Konvertierungen ab.

| `TextError`-Variante | `ContextraError`-Variante | Display-Format | Bedeutung |
|---|---|---|---|
| `InvalidInput(String)` | `InvalidInput(String)` | `ungültige Eingabedaten: {0}` | Ungültige Query, k1/b Parameter, leere Eingabe |
| `Text(String)` | `Text(String)` | `Textverarbeitungsfehler: {0}` | Regex-Fehler, Tokenisierungsfehler |
| `CapacityExceeded { requested, maximum }` | `InvalidInput(String)` | `Kapazitätsgrenze überschritten: gefordert {requested}, maximal {maximum}` | Überschreitung von Maximalgrenzen |
| `Storage(String)` | `Storage(String)` | `Speicher- oder E/A-Fehler: {0}` | Storage-/LSM-Lese- oder Schreibfehler |
| `Serialization(String)` | `Storage(String)` / `Serialization` | `Serialisierungsfehler: {0}` | Bincode / Serde Fehler |
| `Io(std::io::Error)` | `Io(std::io::Error)` | `E/A-Fehler aufgetreten: {0}` | Dateisystem-/Netz-I/O der Standardbibliothek |
| `Internal(String)` | `Internal(String)` | `interner Zustandskonflikt: {0}` | Unerwarteter interner Zustand |
