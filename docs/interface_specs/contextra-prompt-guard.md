# Schnittstellenspezifikation: `contextra-prompt-guard`

> Erzeugt durch `tools/extract_interfaces.py` aus dem Quellcode. Keine Handpflege. Jede Angabe trägt Datei:Zeile. Maßgeblich bleibt der Quellcode.

## 1. Kopf

| Feld | Wert |
|---|---|
| Name | `contextra-prompt-guard` |
| Version | 0.1.0 |
| Pfad | `crates/contextra-prompt-guard` |
| Rolle | Prompt-Injection-Schutz |
| Bibliothekspfad | src/lib.rs (Standard) |

**Crate-Dokumentation (//!):**

> # contextra-prompt-guard
>
> Signatur- und phrasenbasierte Prompt-Injection-Erkennung und Quarantäne-Policy.
> Vollständig autarker E0-Baustein ohne `contextra-*`-Abhängigkeiten.

## 2. Abhängigkeiten

| Name | Art | Quelle | optional | verwendet von (normal/dev) |
|---|---|---|---|---|
| `serde` | normal | registry | nein |  |
| `serde_json` | normal | registry | nein |  |
| `tempfile` | dev | registry | nein |  |
| `thiserror` | normal | registry | nein |  |
| `tracing` | normal | registry | nein |  |
| `unicode-normalization` | normal | registry | nein |  |

**Verwendet von (Crate-Ebene):** `contextra-mcp`, `contextra-prompt-guard-fuzz`

## 3. Features

| Feature | Aktiviert |
|---|---|
| `default` | `std` |
| `file-audit` | – |
| `std` | – |

## 4. Metriken (Quellcode)

| Kennzahl | Wert |
|---|---|
| Quelldateien | 10 |
| Quellzeilen | 1389 |
| Öffentliche Items (Modul-Ebene) | 19 |
| davon `fn` | 1 |
| davon `struct` | 6 |
| davon `enum` | 2 |
| davon `trait` | 1 |
| davon `const` | 1 |
| davon `mod` | 8 |
| Öffentliche Makros (`#[macro_export]`) | 0 |
| Öffentliche Re-Exports (`pub use`) | 7 |
| `unsafe`-Vorkommen (Produktionscode) | 0 |
| `.unwrap()` (Produktionscode) | 10 |
| `.expect(` (Produktionscode) | 0 |
| `panic!` (Produktionscode) | 0 |
| `todo!/unimplemented!/unreachable!` | 0 |
| Indexierungen `x[…]` (grob, Produktionscode) | 7 |
| `#[deprecated]` | 0 |
| `#[cfg(feature…)]`-Gates | 10 |
| Parser-Abdeckung (geparste `pub`-Deklarationen / Grep-Treffer) | 44 / 45 = 98 % |

## 5. Modulbaum und Dateien

| Modul | Datei | Zeilen |
|---|---|---|
| `audit` | `crates/contextra-prompt-guard/src/audit.rs` | 82 |
| `clock` | `crates/contextra-prompt-guard/src/clock.rs` | 31 |
| `compat::error` | `crates/contextra-prompt-guard/src/compat/error.rs` | 12 |
| `compat` | `crates/contextra-prompt-guard/src/compat/mod.rs` | 8 |
| `contract` | `crates/contextra-prompt-guard/src/contract.rs` | 13 |
| `error` | `crates/contextra-prompt-guard/src/error.rs` | 82 |
| `guard` | `crates/contextra-prompt-guard/src/guard.rs` | 662 |
| `crate` | `crates/contextra-prompt-guard/src/lib.rs` | 25 |
| `policy` | `crates/contextra-prompt-guard/src/policy.rs` | 71 |
| `tests` | `crates/contextra-prompt-guard/src/tests.rs` | 403 |

## 6. Re-Exports (`pub use`)

- `pub use crate::contract::Clock` — `crates/contextra-prompt-guard/src/clock.rs:6` (Modul `clock`)
- `pub use audit::` — `crates/contextra-prompt-guard/src/lib.rs:20` (Modul `crate`)
- `pub use clock::SystemClock` — `crates/contextra-prompt-guard/src/lib.rs:21` (Modul `crate`)
- `pub use contract::Clock` — `crates/contextra-prompt-guard/src/lib.rs:22` (Modul `crate`)
- `pub use error::GuardError` — `crates/contextra-prompt-guard/src/lib.rs:23` (Modul `crate`)
- `pub use guard::` — `crates/contextra-prompt-guard/src/lib.rs:24` (Modul `crate`)
- `pub use policy::` — `crates/contextra-prompt-guard/src/lib.rs:25` (Modul `crate`)

## 8. Öffentliche Schnittstelle je Item

### Modul `audit`

#### `struct SecurityAuditLogger`
- Ort: `crates/contextra-prompt-guard/src/audit.rs:30`
```rust
pub struct SecurityAuditLogger
```
Dokumentation: Separater Audit-Logger für Sicherheitsvorfälle (vollständig isoliert vom Vektor-Index).
- Impl-Block `crates/contextra-prompt-guard/src/audit.rs:36`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new(file_path: Option<PathBuf>) -> Self` | `crates/contextra-prompt-guard/src/audit.rs:38` | Erstellt eine neue Instanz des `SecurityAuditLogger`, optional mit Dateipfad für Datei-Audits. |
| `pub fn log_event(&self, record: SecurityAuditRecord)` | `crates/contextra-prompt-guard/src/audit.rs:46` | Protokolliert ein Sicherheitsereignis im Speicher und optional in einer Datei. |
| `pub fn get_recorded_events(&self) -> Vec<SecurityAuditRecord>` | `crates/contextra-prompt-guard/src/audit.rs:76` | Gibt alle im Speicher erfassten Audit-Ereignisse zurück. |

#### `struct SecurityAuditRecord`
- Ort: `crates/contextra-prompt-guard/src/audit.rs:13`
```rust
pub struct SecurityAuditRecord
```
Dokumentation: Struktur für Sicherheits-Audit-Logeinträge bei Auslösung des `Escalate`-Modus.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `timestamp` | `String` | `crates/contextra-prompt-guard/src/audit.rs:15` |
| `event_type` | `String` | `crates/contextra-prompt-guard/src/audit.rs:17` |
| `doc_id` | `String` | `crates/contextra-prompt-guard/src/audit.rs:19` |
| `collection` | `String` | `crates/contextra-prompt-guard/src/audit.rs:21` |
| `pattern_matched` | `String` | `crates/contextra-prompt-guard/src/audit.rs:23` |
| `action_taken` | `String` | `crates/contextra-prompt-guard/src/audit.rs:25` |

### Modul `clock`

#### `struct SystemClock`
- Ort: `crates/contextra-prompt-guard/src/clock.rs:10`
```rust
pub struct SystemClock
```
Dokumentation: Standard-Implementierung basierend auf `std::time::SystemTime`.
- Impl-Block `crates/contextra-prompt-guard/src/clock.rs:12`
- Impl-Block (Trait `Clock`) `crates/contextra-prompt-guard/src/clock.rs:19`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new() -> Self` | `crates/contextra-prompt-guard/src/clock.rs:14` | Erstellt eine neue `SystemClock`. |

### Modul `compat`

#### `mod error`
- Ort: `crates/contextra-prompt-guard/src/compat/mod.rs:8`
```rust
pub mod error
```

### Modul `contract`

#### `trait Clock`
- Ort: `crates/contextra-prompt-guard/src/contract.rs:7`
```rust
pub trait Clock: Send + Sync
```
Dokumentation: Trait für Zeitgeber (ohne Abhängigkeit von contextra-ports oder externen Runtimes).

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `fn now_unix_nanos(&self) -> u64` | `crates/contextra-prompt-guard/src/contract.rs:9` | Liefert die aktuellen Unix-Nanosekunden. |
| `fn monotonic_nanos(&self) -> u64` | `crates/contextra-prompt-guard/src/contract.rs:12` | Liefert monotone Nanosekunden (z. B. für Zeitdifferenzen oder Benchmarks). |

### Modul `crate`

#### `mod audit`
- Ort: `crates/contextra-prompt-guard/src/lib.rs:9`
```rust
pub mod audit
```

#### `mod clock`
- Ort: `crates/contextra-prompt-guard/src/lib.rs:10`
```rust
pub mod clock
```

#### `mod compat`
- Ort: `crates/contextra-prompt-guard/src/lib.rs:11`
```rust
pub mod compat
```

#### `mod contract`
- Ort: `crates/contextra-prompt-guard/src/lib.rs:12`
```rust
pub mod contract
```

#### `mod error`
- Ort: `crates/contextra-prompt-guard/src/lib.rs:13`
```rust
pub mod error
```

#### `mod guard`
- Ort: `crates/contextra-prompt-guard/src/lib.rs:14`
```rust
pub mod guard
```

#### `mod policy`
- Ort: `crates/contextra-prompt-guard/src/lib.rs:15`
```rust
pub mod policy
```

### Modul `error`

#### `enum GuardError`
- Ort: `crates/contextra-prompt-guard/src/error.rs:8`
```rust
pub enum GuardError
```
Dokumentation: Fehler beim Laden, Parsen oder Verarbeiten von Prompt-Injection-Regeln und -Audits.
- Impl-Block `crates/contextra-prompt-guard/src/compat/error.rs:7`

Varianten:

- `Io(String)` — `crates/contextra-prompt-guard/src/error.rs:11`
- `Serialization(String)` — `crates/contextra-prompt-guard/src/error.rs:15`
- `InvalidConfiguration(String)` — `crates/contextra-prompt-guard/src/error.rs:19`
- `DecodeError(String)` — `crates/contextra-prompt-guard/src/error.rs:23`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn error_message(&self) -> String` | `crates/contextra-prompt-guard/src/compat/error.rs:9` | Liefert die Fehlermeldung als String-Repräsentation. |

### Modul `guard`

#### `struct NormalizedPattern`
- Ort: `crates/contextra-prompt-guard/src/guard.rs:18`
```rust
pub struct NormalizedPattern
```
Dokumentation: Vornormalisiertes Injection-Pattern zur Performance-Optimierung.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `original` | `String` | `crates/contextra-prompt-guard/src/guard.rs:20` |
| `collapsed` | `String` | `crates/contextra-prompt-guard/src/guard.rs:22` |
| `no_ws` | `String` | `crates/contextra-prompt-guard/src/guard.rs:24` |

#### `struct PromptInjectionGuard`
- Ort: `crates/contextra-prompt-guard/src/guard.rs:44`
```rust
pub struct PromptInjectionGuard
```
Dokumentation: Signatur- und phrasenbasierter Prompt-Injection-Erkennungsfilter. Erkennt bekannte Angriffsmuster (direkte Phrasen, whitespace-verschleierte Varianten, Base64-kodierte Varianten bis Tiefe 2) in englischer und deutscher Sprache sowie sprachunabhängige strukturelle Marker. # Grenzen Dieser Filter ist **kein vollständiger Schutz** gegen Prompt-Injection-Angriffe: - Unbekannte oder neuartige Formulierungen werden nicht erkannt. - Semantisch äquivalente, aber lexikalisch abweichende Phrasierungen können den Filter umgehen. - Clients müssen abgerufene Dokumenteninhalte weiterhin in isolierten Prompt
- Impl-Block (Trait `std::fmt::Debug`) `crates/contextra-prompt-guard/src/guard.rs:54`
- Impl-Block (Trait `Default`) `crates/contextra-prompt-guard/src/guard.rs:66`
- Impl-Block `crates/contextra-prompt-guard/src/guard.rs:77`

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `redaction_placeholder` | `String` | `crates/contextra-prompt-guard/src/guard.rs:47` |

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn new( policy: QuarantinePolicy, redaction_placeholder: String, patterns: Vec<String>, audit_logger: SecurityAuditLogger, ) -> Self` | `crates/contextra-prompt-guard/src/guard.rs:79` | Erstellt eine neue Instanz des `PromptInjectionGuard`. |
| `pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self` | `crates/contextra-prompt-guard/src/guard.rs:110` | Überschreibt die verwendete `Clock`-Instanz für zeitabhängige Operationen. |
| `pub fn clock(&self) -> &Arc<dyn Clock>` | `crates/contextra-prompt-guard/src/guard.rs:116` | Liefert eine Referenz auf den aktuellen Zeitgeber. |
| `pub fn default_patterns() -> Vec<String>` | `crates/contextra-prompt-guard/src/guard.rs:136` | Gibt die Standard-Erkennungsmuster zurück. Enthält: - 16 englische Kern-Patterns (Token-Marker, direkte Befehle) - 9 deutsche Patterns (Projekt-Sprache für Komp |
| `pub fn policy(&self) -> QuarantinePolicy` | `crates/contextra-prompt-guard/src/guard.rs:190` | Liefert die aktive Quarantäne-Policy. |
| `pub fn patterns(&self) -> &[String]` | `crates/contextra-prompt-guard/src/guard.rs:195` | Liefert die konfigurierten Muster. |
| `pub fn audit_logger(&self) -> &SecurityAuditLogger` | `crates/contextra-prompt-guard/src/guard.rs:200` | Liefert eine Referenz auf den Audit-Logger. |
| `pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, GuardError>` | `crates/contextra-prompt-guard/src/guard.rs:206` | Lädt die Konfiguration aus einer externen JSON-Datei. |
| `pub fn from_env() -> Self` | `crates/contextra-prompt-guard/src/guard.rs:245` | Erstellt eine Instanz basierend auf Umgebungsvariablen. |
| `pub fn is_zero_width(c: char) -> bool` | `crates/contextra-prompt-guard/src/guard.rs:280` | Prüft ob ein Zeichen ein Zero-Width- oder unsichtbares Steuer-Zeichen ist. |
| `pub fn skeletonize_char(c: char) -> char` | `crates/contextra-prompt-guard/src/guard.rs:296` | Mappt gängige kyrillische und griechische Homoglyphen (Confusables) auf ihre lateinischen Äquivalente. |
| `pub fn normalize_text(text: &str) -> String` | `crates/contextra-prompt-guard/src/guard.rs:435` | Normalisiert den Eingabetext (Zero-Width-Stripping, NFKC Normalisierung, Skeletonisierung, Lowercasing). |
| `pub fn collapse_whitespace(s: &str) -> String` | `crates/contextra-prompt-guard/src/guard.rs:443` | Collapsiert aufeinanderfolgende Whitespaces zu einem einzelnen Leerzeichen. |
| `pub fn strip_whitespace(s: &str) -> String` | `crates/contextra-prompt-guard/src/guard.rs:461` | Entfernt sämtliche Whitespaces vollständig (zur Erkennung von Zeichen-Einstreuung). |
| `pub fn decode_base64(input: &str) -> Option<Vec<u8>>` | `crates/contextra-prompt-guard/src/guard.rs:466` | Versucht, einen Base64-String (Standard oder URL-Safe) ohne Panics zu dekodieren. |
| `pub fn extract_base64_candidates(text: &str) -> Vec<&str>` | `crates/contextra-prompt-guard/src/guard.rs:504` | Extrahiert kandidate Base64-Substrings (Länge >= 16) aus einem Eingabetext. |
| `pub fn detect_recursive(&self, text: &str, depth: usize) -> Option<String>` | `crates/contextra-prompt-guard/src/guard.rs:535` | Rekursive Injektions-Erkennung mit harter Tiefenbegrenzung (max. Tiefe 2 für Base64-Dekodierung). |
| `pub fn detect(&self, text: &str) -> Option<String>` | `crates/contextra-prompt-guard/src/guard.rs:573` | Prüft den Eingabetext auf bekannte Prompt-Injection-Muster unter Verwendung von Normalisierung, Whitespace-Analysen und rekursiver Base64-Dekodierung. Gibt den  |
| `pub fn process_result( &self, doc_id: &str, collection: &str, obj: &mut serde_json::Map<String, serde_json::Value>, ) -> bool` | `crates/contextra-prompt-guard/src/guard.rs:581` | Prüft und verarbeitet das JSON-Ergebnisobjekt eines Such- oder Get-Aufrufs. Wendet die konfigurierte `QuarantinePolicy` an und liefert `true` zurück, falls ein  |

### Modul `policy`

#### `struct PromptInjectionConfig`
- Ort: `crates/contextra-prompt-guard/src/policy.rs:53`
```rust
pub struct PromptInjectionConfig
```
Dokumentation: Konfiguration zur Initialisierung aus einer externen Konfigurationsdatei.

Felder:

| Feld | Typ | Ort |
|---|---|---|
| `policy` | `QuarantinePolicy` | `crates/contextra-prompt-guard/src/policy.rs:56` |
| `redaction_placeholder` | `String` | `crates/contextra-prompt-guard/src/policy.rs:59` |
| `audit_log_path` | `Option<String>` | `crates/contextra-prompt-guard/src/policy.rs:62` |
| `custom_patterns` | `Vec<String>` | `crates/contextra-prompt-guard/src/policy.rs:65` |

#### `enum QuarantinePolicy`
- Ort: `crates/contextra-prompt-guard/src/policy.rs:18`
```rust
pub enum QuarantinePolicy
```
Dokumentation: Konfigurierbare Quarantäne-Policy für die Prompt-Injection-Behandlung. **SICHERHEITSHINWEIS & HEURISTIK-LIMITATION**: Pattern-Matching und Normalisierung stellen ein Defense-in-Depth Heuristik-System dar. Es bietet **keine absolute Garantie** gegen neuartige oder komplexe Prompt-Injection-Angriffe. Clients müssen abgerufene Dokumenteninhalte weiterhin in isolierten Prompt-Kontexten (z.B. `<untrusted_context>`) verarbeiten.
- Impl-Block `crates/contextra-prompt-guard/src/policy.rs:34`

Varianten:

- `Strict` — `crates/contextra-prompt-guard/src/policy.rs:21`
- `FlagOnly` — `crates/contextra-prompt-guard/src/policy.rs:28`
- `Escalate` — `crates/contextra-prompt-guard/src/policy.rs:31`

Methoden / Assoziierte Funktionen:

| Signatur | Ort | Doku |
|---|---|---|
| `pub fn from_env() -> Self` | `crates/contextra-prompt-guard/src/policy.rs:38` | Liest die Policy aus der Umgebungsvariablen `CONTEXTRA_MCP_QUARANTINE_POLICY`. |

#### `fn default_redaction_placeholder`
- Ort: `crates/contextra-prompt-guard/src/policy.rs:69`
```rust
pub fn default_redaction_placeholder() -> String
```
Dokumentation: Gibt den Standard-Redaktionsplatzhalter zurück.

#### `const DEFAULT_REDACTION_PLACEHOLDER`
- Ort: `crates/contextra-prompt-guard/src/policy.rs:6`
```rust
pub const DEFAULT_REDACTION_PLACEHOLDER: &str = ""
```
Dokumentation: Neutraler Standard-Platzhalter für als verdächtig erkannten Text im Strict/Escalate-Modus.
