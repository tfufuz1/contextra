# ADR-N15: Konsolidierung von EgressGuard und AdaptiveDecayController (Entfernung von Ring-Duplikaten)

* **Status:** Proposed
* **Datum:** 2026-10-04
* **Kontext:** Befunde FIND-04 und FIND-05 (Strukturelle Duplikation von Ring-0/Ring-2 Komponenten)

---

## 1. Belegter Ist-Zustand

1. **`EgressGuard`**:
   - Existiert sowohl in Ring 2 (`crates/contextra-privacy/src/egress_guard.rs`) als auch in Ring 4 (`crates/contextra-mcp/src/egress_guard.rs`).
   - `contextra-mcp` re-implementiert die Egress-Filterregeln unabhängig, was zu Regeldivergenz führt.

2. **`AdaptiveDecayController`**:
   - Existiert sowohl in Ring 0 (`crates/contextra-adapt/src/decay_controller.rs`) als auch in Ring 3 (`crates/contextra-engine/src/decay_controller.rs`).
   - `contextra-engine` nutzt die eigene Kopie statt das spezialisierte Ring-0-Crate `contextra-adapt`.

---

## 2. Optionen zur Behebung

### Option 1: Reines Re-Exportieren / Nutzung der Ring-0/Ring-2 Typen
* **Beschreibung:**
  - `contextra-mcp` entfernt die eigene `EgressGuard`-Implementierung und importiert `contextra_privacy::EgressGuard`.
  - `contextra-engine` entfernt die eigene `AdaptiveDecayController`-Implementierung und importiert `contextra_adapt::AdaptiveDecayController`.
* **Vorteile:**
  - Eliminierung von ~600 Zeilen doppeltem Code.
  - Garantiert konsistentes Verhalten über den gesamten Workspace.
* **Nachteile:**
  - `Cargo.toml` von `contextra-mcp` benötigt explizite Abhängigkeit auf `contextra-privacy`.
* **Aufwand/Risiko:** Gering (ca. 0.5 Personentage).

### Option 2: Verschiebungen nach `contextra-ports`
* **Beschreibung:** Trait-Abstraktionen für Egress Guard und Decay Controller in `contextra-ports` definieren.
* **Vorteile:** Entkopplung über Trait-Objekte.
* **Nachteile:** Unnötige Indirektion für konkrete Hilfsklassen.

---

## 3. Empfehlung der Architektur
**Option 1**: Direkte Konsolidierung durch Nutzung der kanonischen Implementierungen in `contextra-privacy` und `contextra-adapt`.
