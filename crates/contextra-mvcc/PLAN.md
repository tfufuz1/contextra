# Implementierungsspezifikation & Arbeitsplan: `contextra-mvcc`

> **Ring:** 0 | **Klasse:** `K` | **Code-Budget:** 1650 LOC | **Unsafe:** Streng verboten (#![forbid(unsafe_code)])
> **Architekturrolle:** Multi-Version Concurrency Control und Serializable Snapshot Isolation (SSI)
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-mvcc` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

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
| `registry` | Lock-freie Snapshot-Registrierung und Pin-Management |
| `ssi` | Erkennung von Read-Write-Konflikten (Anti-Dependency Cycles) |
| `watermark` | Globale Low-Watermark Berechnung für Garbage Collection |
| `tx_guard` | RAII-Guards für MVCC-Lese- und Schreibtransaktionen |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `contextra-types, contextra-ports`
- **Verbotene Abhängigkeiten:** Höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Spezifiziert in `AGENTS.md` und `docs/SPEC.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-MVC-1: Lock-freie Snapshot-Registrierung
- **Fokus / Betroffene Dateien:** `src/registry.rs`
- **Problemstellung & Baukasten-Ziel:** Verwaltung aktiver Reader-Snapshots mittels atomarer Bitmaps und Epochen; allokationsfreier Snapshot-Erwerb.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-MVC-2: SSI-Konflikt-Detektion
- **Fokus / Betroffene Dateien:** `src/ssi.rs`
- **Problemstellung & Baukasten-Ziel:** Erkennung von Gefahrenzyklen (si-edges) zur Garantie echter Serialisierbarkeit ohne globale Lesesperren.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-MVC-3: Low-Watermark & PinGuard
- **Fokus / Betroffene Dateien:** `src/watermark.rs, src/tx_guard.rs`
- **Problemstellung & Baukasten-Ziel:** Sichere Freigabe veralteter Versionen; RAII-PinGuard hält MVCC-Pins ohne Deadlocks.
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
2. `cargo check -p contextra-mvcc`, `cargo test -p contextra-mvcc` und `cargo clippy -p contextra-mvcc` laufen fehlerfrei durch.
3. Code-Budget von maximal 1650 LOC wird strikt eingehalten.
4. Keine unzulässigen Querabhängigkeiten oder Lecks von Implementierungsdetails.
5. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel.
