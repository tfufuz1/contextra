# AGENTS.md — `contextra`

## 1. Rolle & Zweck (Baukasten-Prinzip)
`contextra` bildet als Hauptfassade die einzige offizielle Composition Root des Gesamtsystems (Ring 4).
Es assembliert und konfiguriert die autarken Bausteine (`contextra-types`, `contextra-ports`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-engine`, `contextra-erasure`) über typensichere Builder (`ContextraBuilder`, `EngineBuilder`).
Als reine Fassade enthält es keinerlei interne Geschäfts- oder Speicherlogik und delegiert alle Operationen strikt an die spezialisierten Bausteine.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 4 (Haupt-Fassade & Composition Root)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types`, `contextra-ports`, `contextra-license`, `contextra-db`, `contextra-rank`, `contextra-crypto` (Spec §6.9, `capabilities.toml`).

## 3. Garantien & Architektur-Invarianten
- **Strikte Speichersicherheit:** `#![forbid(unsafe_code)]` wird über das gesamte Crate hinweg erzwungen.
- **Typestate-Disziplin (D-123):** `EngineBuilder` erzwingt verbindliche Typestates (Passphrase, Runtime Handle, Backup-Retention, Vektordimension) zur Compile-Zeit.
- **Fail-Closed bei unvollständigen Features:** Unimplementierte Fassaden-Stubs schlagen deterministisch mit `Err(Error::NotImplemented)` fehl anstatt stillen Erfolg vorzutäuschen.
- **Zero-Panic-Doktrin:** Vollständiger Ausschluss von `unwrap()` und `expect()`; saubere Fehlerfortpflanzung über `ContextraError`.
- **UTF-8 Strenge:** Passphrasen und Dokument-Schlüssel werden strikt auf gültiges UTF-8 validiert; verlustbehaftetes Decoding (`from_utf8_lossy`) ist untersagt.

## 4. Verbotene Muster
- Direkte Implementierung von Speicher-, Index- oder Krypto-Algorithmen im Crate (Delegation an Subsysteme).
- Verwendung von `contextra-core` (statdessen direkte Nutzung von `contextra-types` und `contextra-ports`).
- Ignorieren von Typestate-Garantien bei der System-Initialisierung.
- Ungeprüfte Re-Exports unfertiger APIs.

## 5. Schnittstellen & Datenfluss (Ports & Abstraktionen)
- **Exportiert:** Öffentliche Hauptschnittstelle für Entwickler und externe Integrationen (`ContextraBuilder`, `Contextra`, `Collection`).
- **Orchestrierung:** Bündelt Ring 0 bis 3 Bausteine zur lauffähigen Gesamtinstanz.

## 6. Tests, Sabotage & Qualitätssicherung
- **Composition Root Tests:** Verifikation der korrekten Assemblierung aller Teilsysteme unter verschiedenen Profilen.
- **Typestate Compile-Fail Tests:** Prüfung, dass fehlerhafte Builder-Aufrufe bereits zur Compile-Zeit abgewiesen werden.
- **Sabotage-Testing:** Prüfung der sauberen Fehlerbehandlung bei fehlgeschlagener Subsystem-Initialisierung.

## 7. Offene Punkte & Technische Schulden
- DEBT-04: Vollständige Bereinigung verbliebener Übergangs-Delegationen.
