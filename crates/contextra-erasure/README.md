# contextra-erasure

Revisionssicherer Löschbaustein: Saga, Gate-Log, Tresor, Witness & signierter Audit-Bericht.

Normativitätsklasse: **Kern `[K]`** (Ring 3, `beta`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra-erasure` ist der isolierte Löschbaustein des Contextra-Ökosystems (Ring 3). Er kapselt die vormals im Speicherkern verstreute mehrstufige Löschkette und stellt die kryptografisch verifizierbare Bereinigung als autarke Komponente bereit.

Im Baukastensystem übernimmt `contextra-erasure` die vollständige Verantwortung für Löschprotokolle und Nachweise, ohne dass Speicherkerne (Store, Vector, Text) rechtliche oder kryptografische Löschkettenlogik enthalten müssen.

## Subsysteme & Kernkomponenten

- **Saga (`saga`):** Koordiniert die mehrstufige Löschungs-Transaktion über registrierte Schichten hinweg (`ErasureCore`).
- **Gate-Log (`gate`):** Protokolliert und überwacht Read-Gates und Zutrittssperren für zu bereinigende Identifikatoren.
- **Tresor (`vault`):** Verwaltet Schlüsselableitung, Key-Slots und sichere Zerstörung ephemerer kryptografischer Schlüssel (Crypto-Shredding).
- **Journal (`journal`):** Härtet den Zustandsübergang der Lösch-Saga gegen unvollständige Durchläufe und Systemabstürze ab.
- **Witness & Messung (`measure`):** Führt physische Restspeicher-Scans und Kontrollprüfungen durch, um die Bereinigung der Speicherbereiche messtechnisch zu belegen.
- **Audit-Bericht (`report`):** Generiert kryptografisch signierte (Ed25519) Bereinigungsnachweise mit Hash-Verkettung.
- **Backend & Target Abstraktionen (`backend`):** Definiert generische Schnittstellen (`StorageBackend`, `ErasureTarget`) zur vollständigen Entkopplung von konkreten Datenhaltern.

## Schnittstellen & Entkopplung

- **Eingehend:** Wird von Orchestratoren (`contextra-db`, `contextra`) oder API-Gateways (`contextra-mcp`) aufgerufen.
- **Ausgehend:** Kommuniziert mit Datenhaltungs-Kernen ausschließlich über standardisierte Ports (`Purger`, `LayerScanner` aus `contextra-ports`) oder generische Abstraktionen (`StorageBackend`).
- **Abhängigkeiten (M1):** Ausschließliche Bindung an Ring 0 (`contextra-types`, `contextra-ports`, `contextra-crypto`).
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`, Zero-Panic in Produktionspfaden.

## Baukasten-Nutzung (M10 Minimalbeispiel)

```rust
use contextra_erasure::backend::{ErasureTarget, StorageBackend};
use contextra_ports::erasure::{LayerId, PurgeCx, PurgeOutcome};

// Der Löschbaustein kann isoliert instanziiert und an beliebige
// Speicherkerne angebunden werden, die die Bereinigungs-Ports implementieren.
// Siehe vollständiges, lauffähiges Beispiel unter examples/erasure_lifecycle.rs:
// cargo run --example erasure_lifecycle -p contextra-erasure
```
