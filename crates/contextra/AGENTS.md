# AGENTS.md — contextra
> Ring 4 · stable · Quelle: capabilities.toml · Spec: III.26 / K.33

## 1. Zweck
Haupt-Facade und Composition Root für Endanwender zur Konfiguration und Instanziierung von Contextra.
Bündelt Speicherschicht, Vektor-Index, Krypto-Integrität und Inferenz hinter einer einfachen High-Level-API (`ContextraBuilder`, `AgentMemory`).
Ermöglicht den schnellen Einstieg mit sicheren Standardeinstellungen (Ring Fast, Pure Rust Candle Backend).

## 2. Modul-Karte

| Datei | Verantwortung |
|---|---|
| `lib.rs` | Öffentliche Einstiegspunkte `open()`, `open_with_config()` und `builder()` |
| `builder.rs` | `ContextraBuilder` zur Konfiguration von Speicherpfad, Passphrase, Embedder und Lizenz-Gates |
| `agent_memory.rs` | High-Level `AgentMemory`-Facade für speicher- und beziehungsorientierte Agenten-Interaktionen |
| `collection_profile.rs` | `CollectionProfile`, `DeploymentTier` und LSM-Tuning für unterschiedliche Betriebsumgebungen |
| `performance_profile.rs` | `PerformanceProfile` (Compliance, Balanced, BareMetal) und `ResolvedProfileConfig` |
| `query_rewriter.rs` | `LlmQueryRewriter` zur LLM-basierten Abfrageerweiterung und Multi-Signal-Expansion |

## 3. Invarianten

- **INV-PERF-PROFILE-1**: PerformanceProfile erzwingt Lizenzprüfung bei Nicht-Fast-Ringen (`ResolvedProfileConfig::enforce_license`).
  *Prüfung*: `cargo test -p contextra --lib`
- **INV-COLLECTION-PROFILE-1**: DeploymentTier (EdgeMinimal, PowerUserLocal, EnterpriseShared, EnterpriseRegulated) löst deterministische LSM-Tuning-Parameter auf.
  *Prüfung*: `cargo test -p contextra --lib`
- **INV-FACADE-BOUNDS**: `#![forbid(unsafe_code)]` und keine direkten Abhängigkeiten zu ungeschützten Ring-0/1-Interna.
  *Prüfung*: `cargo check -p contextra`

## 4. Verboten / Anti-Patterns

- **Keine direkten Low-Level-Bypasses**: Bypasse niemals die Composition Root, um direkte Interaktionen mit Ring 0/1 Modulen zu erzwingen.
- **Keine unbeabsichtigte Ring-Erhöhung**: Konfigurationen höherer Feature-Ringe (`Sovereign`, `Compliance`) dürfen nicht ohne gültiges Lizenz-Gate geladen werden.
- **Keine ungeschützten Locks**: In Facade-Methoden keine geschachtelten Sperren über Thread-Grenzen hinweg halten.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Invarianten aus Ring 4 (P26) erzwingen Thread-Sicherheit über Arc-geschützte Engine-Instanzen (`AgentMemory::new_arc`).
- Sperrenreihenfolge innerhalb des Builders und der Profile ist strikt flach (keine geschachtelten `Mutex` / `RwLock` Aufrufe).
- Asynchrone Operationen nutzen Tokio-Executor-Kontext der zugrundeliegenden `contextra-db` Engine.

## 6. Verifikation

- `cargo test -p contextra`
- `cargo xtask check-ring-layering`

## 7. Bekannte Lücken / SOLL

- EnterpriseRegulated Tier verwendet noch `AutoExtractionMode::Enabled` (Spec D.6 fordert ggf. Opt-In Bestätigung).
