# Implementierungsspezifikation & Arbeitsplan: `contextra-types`

> **Ring:** 0 | **Klasse:** `K` | **Code-Budget:** 2700 LOC | **Unsafe:** Streng verboten (#![forbid(unsafe_code)])
> **Architekturrolle:** Kanonische Grundtypen, IDs, Fehlerklassen, RequestContext und Basis-Datenmodelle des Gesamtsystems
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-types` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

1. **Modulare Autarkie & Isolation:** Der Crate funktioniert als eigenständiger, isolierter Baustein. Direkte Kopplungen an Fremd-Crates werden über wohldefinierte Schnittstellen in `contextra-ports` oder generische Abstraktionen gelöst.
2. **Entkopplung der Löschkette:** Sämtliche juristischen Begrifflichkeiten (DSGVO Art. 17), Nachweisketten, Deletion-Proofs und Saga-Orchestrierungen verbleiben exklusiv in `contextra-erasure`. Datenhaltende Module implementieren ausschließlich neutrale, funktionale Bereinigungs-Ports (`Purger`, `LayerScanner`).
3. **Design by Contract (DbC) & Invarianten:** Alle Vorbedingungen, Nachbedingungen und Systeminvarianten werden strikt geprüft (Fail-Closed).
4. **Zero Shared Mutable State:** Nebenläufigkeit wird über deterministisches Message-Passing, unteilbare Transaktionszustände oder atomare Sequenznummern strukturell abgesichert.
5. **Code- & Unsafe-Disziplin:** `#![forbid(unsafe_code)]` ist im gesamten Crate verbindlich gesetzt. Keine unwrap(), expect() oder Panics im Produktivcode.

---

## 2. Modul- & Komponentenarchitektur

### 2.1 Interne Struktur & Verantwortung

| Komponente / Modul | Verantwortung & Schnittstellen |
|---|---|
| `ids` | DocRef, DocKey, DocHandle, ErasureId, FileId, StoreId, TenantId, TxId |
| `tags` | ClusterTag, CollTag, DocTag, NameTag, SubjectTag, TenantTag, TermTag |
| `key` | KeyBuf (78 Byte fixiert, Commit-Seq Maskierung) |
| `erasure` | EraseTarget, LayerWitness, ResidualRisk, KeyCoverage, ScanMethod |
| `error` | Geschlossenes ContextraError-Enum (SPEC §7.5, frozen messages) |
| `limits` | Ressourcen- und Kapazitätsgrenzen (SPEC §14.4, D-87) |
| `secret` | ZeroizeOnDrop Geheimnisträger, geschwärzte Debug-Ausgaben |
| `tenant_scope` | TenantContext und Mandanten-Isolation |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `Keine (autarkes Typen-Fundament)`
- **Verbotene Abhängigkeiten:** Höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Spezifiziert in `AGENTS.md` und `docs/SPEC.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-TYP-1: Kanonische Identifikatoren & Breiten-Agnostik
- **Fokus / Betroffene Dateien:** `src/ids.rs`
- **Problemstellung & Baukasten-Ziel:** Bereitstellung von DocId::new mit nativer Unterstützung für 64-bit und 128-bit (docid-128); zero-copy Konvertierungsmethoden from_le_bytes und to_le_bytes.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-TYP-2: Geschlossenes Fehlersystem (ContextraError)
- **Fokus / Betroffene Dateien:** `src/error.rs`
- **Problemstellung & Baukasten-Ziel:** Vollständige Standardisierung aller Systemfehler; frozen error messages ohne Leckage sensibler Benutzer- oder Klartextdaten; Lossless DTO-Konvertierung.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-TYP-3: Fixed-Size KeyBuf & Tagging-Primitives
- **Fokus / Betroffene Dateien:** `src/key.rs, src/tags.rs`
- **Problemstellung & Baukasten-Ziel:** 78-Byte fixierte KeyBuf-Allokation für zero-copy LSM- und Index-Transfers; strikte Maskierung von !commit_seq; typisierte Tags für semantische Netze.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-TYP-4: Bereinigungs- und Witness-Typen
- **Fokus / Betroffene Dateien:** `src/erasure.rs`
- **Problemstellung & Baukasten-Ziel:** Strukturierte Definitionen für EraseTarget, LayerWitness und ResidualRisk zur neutralen Kommunikation mit Purger-Ports und contextra-erasure.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-TYP-5: Ressourcen-Budgets & Secret-Wrapper
- **Fokus / Betroffene Dateien:** `src/limits.rs, src/secret.rs`
- **Problemstellung & Baukasten-Ziel:** ResourceBudget, ResourceTracker; ZeroizeOnDrop für sensible Speicherbereiche mit Constant-Time Debug-Redaction.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

---

## 4. Test-, Benchmark- & Qualitätssicherungsplan

### 4.1 Unit-, Integration- & Conformance-Tests
- **Unit-Tests:** 100% Testabdeckung für Kernfunktionen und Randfall-Behandlungen.
- **Port-Konformität:** Validierung gegen die Prüfsuiten in `contextra-testkit`.
- **Traceability:** Jeder Test annotiert die Spezifikationsanforderung (z. B. `/// SPEC: REQ-...`).

### 4.2 Property-Based Testing & Fuzzing
- Testen aller Invarianten unter zufälligen Eingaben (proptest).
- Fuzzing aller Parser-, Deserialisierungs- und Eingabepfade.

### 4.3 Mutations-Testing (Sabotage-Checks)
- Mutanten-Injektion zur Verifikation der Testgüte (Ziel: Mutation-Score >= 85%).

---

## 5. Definition of Done (DoD) für diesen Baustein

1. Alle deklarierten Arbeitspakete sind vollständig implementiert.
2. `cargo check -p contextra-types`, `cargo test -p contextra-types` und `cargo clippy -p contextra-types` laufen fehlerfrei durch.
3. Code-Budget von maximal 2700 LOC wird strikt eingehalten.
4. Keine unzulässigen Querabhängigkeiten oder Lecks von Implementierungsdetails.
5. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel.
