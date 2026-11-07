# Implementierungsspezifikation & Arbeitsplan: `contextra-simd`

> **Ring:** 0 | **Klasse:** `K` | **Code-Budget:** 1200 LOC | **Unsafe:** Genehmigte Unsafe-Insel (// SAFETY: Pflicht)
> **Architekturrolle:** Autarke SIMD-Vektorbeschleunigung (AVX2, AVX-512, NEON) mit bitidentischem Skalar-Fallback
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-simd` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

1. **Modulare Autarkie & Isolation:** Der Crate funktioniert als eigenständiger, isolierter Baustein. Direkte Kopplungen an Fremd-Crates werden über wohldefinierte Schnittstellen in `contextra-ports` oder generische Abstraktionen gelöst.
2. **Entkopplung der Löschkette:** Sämtliche juristischen Begrifflichkeiten (DSGVO Art. 17), Nachweisketten, Deletion-Proofs und Saga-Orchestrierungen verbleiben exklusiv in `contextra-erasure`. Datenhaltende Module implementieren ausschließlich neutrale, funktionale Bereinigungs-Ports (`Purger`, `LayerScanner`).
3. **Design by Contract (DbC) & Invarianten:** Alle Vorbedingungen, Nachbedingungen und Systeminvarianten werden strikt geprüft (Fail-Closed).
4. **Zero Shared Mutable State:** Nebenläufigkeit wird über deterministisches Message-Passing, unteilbare Transaktionszustände oder atomare Sequenznummern strukturell abgesichert.
5. **Code- & Unsafe-Disziplin:** Unsafe-Blöcke sind strikt auf das Notwendigste beschränkt, auditierbar und in `docs/unsafe-register.md` registriert.

---

## 2. Modul- & Komponentenarchitektur

### 2.1 Interne Struktur & Verantwortung

| Komponente / Modul | Verantwortung & Schnittstellen |
|---|---|
| `cosine` | Kanonische Kosinus-Ähnlichkeit mit 16 Teilsummen |
| `dot_product` | Skalarprodukt-Kernel für normierte Vektoren |
| `euclidean` | Euklidische Distanzberechnung |
| `dispatch` | Dynamischer CPU-Feature-Dispatch zur Laufzeit |
| `scalar` | Bitidentische Skalar-Referenzimplementierung |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `contextra-types`
- **Verbotene Abhängigkeiten:** Höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Spezifiziert in `AGENTS.md` und `docs/SPEC.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-SMD-1: Kanonische Summation & Register-Akkumulation
- **Fokus / Betroffene Dateien:** `src/cosine.rs, src/dot_product.rs`
- **Problemstellung & Baukasten-Ziel:** Implementierung der 16 Teilsummen (lane = index mod 16) ohne FMA; paarweise Reduktion 16->8->4->2->1 gemäß SPEC D-122.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-SMD-2: Plattform-Dispatch (AVX2, AVX-512, NEON)
- **Fokus / Betroffene Dateien:** `src/dispatch.rs`
- **Problemstellung & Baukasten-Ziel:** Robuste Erkennung von CPU-Fähigkeiten ohne Panics; nahtloser Fallback auf Skalarpfade.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-SMD-3: Differenzielle Property-Tests (T-SIMD-1)
- **Fokus / Betroffene Dateien:** `tests/prop_simd.rs`
- **Problemstellung & Baukasten-Ziel:** Verifikation gegen unaligned Slices, Tails (1..15 Reste), Denormals, NaN/Inf und Dimensionen 0..4096.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-SMD-4: Bit-Identitäts-Gate (T-SIMD-2)
- **Fokus / Betroffene Dateien:** `tests/bit_exact.rs`
- **Problemstellung & Baukasten-Ziel:** Garantie identischer f32::to_bits Ergebnisse zwischen AVX2, AVX-512, NEON und Skalar-Referenz.
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
2. `cargo check -p contextra-simd`, `cargo test -p contextra-simd` und `cargo clippy -p contextra-simd` laufen fehlerfrei durch.
3. Code-Budget von maximal 1200 LOC wird strikt eingehalten.
4. Keine unzulässigen Querabhängigkeiten oder Lecks von Implementierungsdetails.
5. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel.
