# Analyse der öffentlichen Schnittstelle (`contextra`)

Gemäß SPEC D-89, SPEC §6.4 und SPEC §7 (Öffentliche API und Verdrahtung. Enthält keine Logik.)

## 1. Öffentliche Symbole und Stabilitätszusage

| Symbol | Typ | Stabilitätszusage | Konsumententest | Beschreibung |
|---|---|---|---|---|
| `contextra::Contextra` | `struct` | SemVer (`#[non_exhaustive]`) | `trybuild`, `test_erase_vs_soft_delete_facade` | Zentrales Datenbank-Handle (Arc-basiert, Clone, async) |
| `contextra::EngineBuilder` | `struct` | SemVer (Typestate-Builder) | `trybuild/01_public_api_surface.rs` | Builder mit Compile-Time-Prüfung von Passphrase und Runtime |
| `contextra::Tenant` | `struct` | SemVer (`#[non_exhaustive]`) | `trybuild/04_missing_tenant_context_compile_fail.rs` | Mandanten-Handle, erzwingt `TenantContext` |
| `contextra::Collection` | `struct` | SemVer (`#[non_exhaustive]`) | `deletion_e2e.rs` | Collection-Handle mit `insert`, `update`, `get`, `soft_delete`, `search_text` |
| `contextra::CollectionOptions` | `struct` | SemVer (`#[non_exhaustive]`) | `deletion_e2e.rs` | Optionen zur Collection-Erstellung (Analyzer, Positions, Dimension) |
| `contextra::Document` | `struct` | SemVer (`#[non_exhaustive]`) | `deletion_e2e.rs` | Dokumentdatenstruktur mit Content, Metadata und Vektor |
| `contextra::DocHandle` | `struct` | SemVer (`#[non_exhaustive]`) | `trybuild/01_public_api_surface.rs` | Opaker Dokument-Handle für Suchergebnisse |
| `contextra::Hit` | `struct` | SemVer (`#[non_exhaustive]`) | `trybuild/01_public_api_surface.rs` | Suchtreffer mit DocHandle und Score |
| `contextra::SearchPage` | `struct` | SemVer (`#[non_exhaustive]`) | `trybuild/01_public_api_surface.rs` | Paginierte Suchergebnisseite mit Exhaustion-Flag |
| `contextra::HealthReport` | `struct` | SemVer (`#[non_exhaustive]`) | `trybuild/01_public_api_surface.rs` | Status- und Zustandsbericht ohne I/O |
| `contextra::system_defaults` | `fn` | SemVer | `trybuild/01_public_api_surface.rs` | Verdrahtung von `RealVfs`, `SystemClock`, `OsRng` |
| `contextra::SystemDefaults` | `struct` | SemVer (`#[non_exhaustive]`) | `trybuild/01_public_api_surface.rs` | Bündelung der System-Ports |
| `contextra::OsRng` | `struct` | SemVer | `trybuild/01_public_api_surface.rs` | OS-CSPRNG-Adapter |
| `contextra::Error` | `enum` | SemVer (`#[non_exhaustive]`) | `test_error_secrets_property.rs` | Zentrale Fehlertypen mit Handlungsempfehlung, leakt keine Secrets |
| `contextra::Result<T>` | `type` | SemVer | `trybuild/01_public_api_surface.rs` | Result-Alias für Fassaden-Rückgaben |
| `contextra::Secret` | `struct` | SemVer (ZeroizeOnDrop) | `trybuild/01_public_api_surface.rs` | Zeroisierter Speicherpuffer für Passphrasen (kein Debug/Display) |
| `contextra::TenantContext` | `struct` | SemVer | `trybuild/04_missing_tenant_context_compile_fail.rs` | Pflichtparameter für Mandantenisolation (D-07) |
| `contextra::EraseRequest` | `struct` | SemVer | `deletion_e2e.rs` | Strukturierter Löschauftrag (SPEC 7.3) |
| `contextra::EraseReceipt` | `struct` | SemVer | `deletion_e2e.rs` | Quittung mit ErasureId und Timestamp |
| `contextra::EraseTarget` | `enum` | SemVer | `deletion_e2e.rs` | Löschziel (Document, Subject, Collection, Tenant) |
| `contextra::EraseMode` | `enum` | SemVer | `deletion_e2e.rs` | Erase vs Restrict (Art. 18 DSGVO) |
| `contextra::RetentionAttestation` | `struct` | SemVer | `deletion_e2e.rs` | Pflicht-Attestierung für Löschungen |
| `contextra::RetentionCode` | `enum` | SemVer | `deletion_e2e.rs` | Abschließender Enum für Aufbewahrungspflichten |

## 2. API-Diff zur Vorversion (D-89)

### Entfernt (da unzulässige Logik / Profiling / Re-Exporte):
- `contextra::builder(dimension)` -> Abgelöst durch typisierten `Contextra::builder()` ohne Schein-Dimension.
- `contextra::EncryptionMode` (samt `Unencrypted`) -> Entfallen per D-05 und D-89 (Verschlüsselung ist Pflicht).
- `contextra::LlmQueryRewriter` -> Entfallen nach D-89.
- `contextra::collection_profile::*` -> Entfallen nach D-89.
- `contextra::performance_profile::*` -> Entfallen nach D-89.
- `contextra::DocId`, `contextra::chunker`, `contextra::TextEmbeddingEngine`, `contextra::DriftStatusProvider`, `contextra::rank` -> Entfallen nach D-89.
- `Collection::delete` -> Vollständig entfernt per VETO-D51. Stattdessen `Collection::soft_delete` für logische Tombstones und `Contextra::erase` für Löschpflichten nach Art. 17.

### Hinzugefügt / Gestärkt:
- Typestate-Pattern in `EngineBuilder`: Fehlende Passphrase oder fehlender Runtime-Handle führen zu statischen Compilerfehlern (`compile_fail`), nicht zu Laufzeitfehlern.
- `system_defaults()`: Stellt `RealVfs`, Systemuhr und OS-CSPRNG bereit.
- `Error`: Strikt redigierte Fehlermeldungen mit Handlungsempfehlungen nach SPEC §6.4.
