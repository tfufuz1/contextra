# ADR-N16: Bereinigung verwaister Ports und No-Op Cargo-Features

* **Status:** Proposed
* **Datum:** 2026-10-04
* **Kontext:** Befunde FIND-07 und FIND-08 (Verwaiste Ring-0-Traits und tote Cargo-Features)

---

## 1. Belegter Ist-Zustand

1. **Verwaiste Ports in `contextra-ports`**:
   - `pub trait Snapshot`: 0 Implementierungen im gesamten Workspace.
   - `pub trait TextGenerator`: 0 Implementierungen im gesamten Workspace (während `LlmTextGenerator` 27 Implementierungen besitzt).

2. **No-Op Cargo-Features**:
   - `sieve-cache` in `contextra-store`
   - `ppr-forward-push` in `contextra-graph`
   - `kvcache-attention-eviction` in `contextra-kvcache`
   - `egress-sherman-morrison` in `contextra-router`
   - `bm25f` in `contextra-text`
   - `graph` in `contextra-vector`
   - `edge-reinforcement-learning` und `coherence-bonus-fusion` in `contextra-engine`

---

## 2. Optionen zur Bereinigung

### Option 1: Vollständiges Entfernen ungenutzter Artefakte
* **Beschreibung:**
  - Deprecate/Löschen der Trait-Definitionen `Snapshot` und `TextGenerator` aus `contextra-ports`.
  - Entfernen aller No-Op Cargo-Features aus den jeweiligen `Cargo.toml`-Dateien.
* **Vorteile:**
  - Bereinigt die API-Schnittstelle.
  - Verhindert, dass Anwender wirkungslose Features in `Cargo.toml` aktivieren.
* **Nachteile:**
  - Breaking Change, falls externe Nutzer (nicht-workspace) diese Features referenzieren.

### Option 2: Deprecation-Marker belassen
* **Beschreibung:** Markieren mit `#[deprecated]` und Beibehaltung bis v1.0.

---

## 3. Empfehlung der Architektur
**Option 1**: Da `contextra` vor v1.0 steht, sollten wir ungenutzte Sackgassen umgehend entfernen.
