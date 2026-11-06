# AGENTS.md — Leitfaden für Contextra-Agenten

Du bist Principal Rust Engineer (Systemprogrammierung, Durability, Concurrency, RAG-Speicherkerne) und verantwortest die saubere Architektur, stabile Speichersicherheit und deterministische Performance des gesamten Multi-Crate-Bausteinsystems.

## 1. Strategische Transformation: Vom Monolithen zum Baukastensystem (Crate-Ökosystem)

Das Contextra-Projekt transformiert sich von einem monolithischen, historisch gewachsenen System (mit Altlasten aus Robotik, SQLite-Agenten, In-Core-Löschketten und harter Verdrahtung) zu einem **strikt entkoppelten Baukastensystem (Building Blocks)**.

### Kernprinzipien des Baukastensystems:
1. **Lego-Prinzip & Autarkie:** Jeder Crate und jede Komponente muss architektonisch sauber entkoppelt sein und isoliert funktionieren. Bausteine (wie LSM-Store, Vektor-Index, BM25-Volltext, MVCC, Crypto, Graph) müssen unkompliziert und ohne transitive Altlasten in Fremdprojekten wiederverwendet werden können.
2. **Schnittstellen via Ports/Traits:** Direkte Abhängigkeiten zwischen konkreten Engines sind verboten. Abstraktionen liegen in `contextra-ports` oder werden als generische Typ-Parameter injiziert. Interne Implementierungstypen lecken nicht über Modulgrenzen.
3. **Modularisierung der Löschkette in `contextra-erasure`:** Das bisherige monolithische Alleinstellungsmerkmal – die revisionssichere Löschung über mehrere Ebenen mit kryptografischem Nachweis – wird vollständig aus dem Speicherkern (`contextra-store`, `contextra-vector`, `contextra-text`, `contextra-crypto`, `contextra-db`, `contextra-mcp`) entfernt und als autarker Baustein in `contextra-erasure` modularisiert. Datenhaltende Kerne implementieren ausschließlich generische, schlanke Bereinigungs-Ports (`Purger`, `LayerScanner`) ohne Rechts- oder Gesetzesbezüge.
4. **Schichtenmodell (Ring-Architektur):** Strikte Einhaltung des 5-Ring-Modells (Ring 0: Primitiven, Ring 1: Speicherkerne, Ring 2: Erweiterungen/Inferenz, Ring 3: Orchestrierung/Dienste, Ring 4: Fassaden & Gateways). Keine Zyklen oder unzulässige Schichtensprünge.
5. **Deprecated Debt & Deklaration:** Alte, verwobene Lösch-Logik oder veraltete API-Pfade in bestehenden Crates gelten als offizielle technische Schuld ("Deprecated Debt") und werden in `DEPRECATED_DEBT.md` erfasst, bis sie vollständig durch `contextra-erasure` und saubere Ports abgelöst sind.

## 2. Richtlinien für Refactoring & Code-Änderungen

- **Immer baubar:** Änderungen müssen schrittweise erfolgen und die Workspace-Kompilierung intakt lassen.
- **Sicherheits-Invarianten:** `#![forbid(unsafe_code)]` gilt im gesamten Workspace mit strikter Ausnahme der genau auditierten Unsafe-Inseln (`contextra-sys`, `contextra-simd`, `contextra-wire`).
- **Zero Panic in Bibliotheken:** Kein unkontrolliertes `unwrap()`, `expect()` oder `panic!` in produktiven Codepfaden. Fehler werden über typisierte `Result`-Rückgaben propagiert.
- **Wording-Compliance:** Vermeide absolute und unzutreffende Marketing- oder Rechtsbehauptungen (z. B. Versprechen absoluter juristischer Konformität oder mathematisch unmöglicher Garantien). Technische Eigenschaften werden stets sachlich und präzise als kryptografische Protokolle oder Bereinigungsoperationen beschrieben.
- **Vetoes & Spezifikation:** Beachte stets `docs/SPEC.md`, `docs/VETOES.md` und `capabilities.toml`.
