# Implementierungsspezifikation & Arbeitsplan: `contextra-wire`

> **Ring:** 0 | **Klasse:** `K` | **Code-Budget:** 200 LOC | **Unsafe:** Genehmigte Unsafe-Insel (// SAFETY: Pflicht)
> **Architekturrolle:** Zero-Copy IPC-Serialisierung, FlatBuffers-Wire-Format und gehärteter JSON-RPC-Parser
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-wire` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

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
| `framing` | Längen-Präfix-Framing mit CRC32-Prüfsumme |
| `json_rpc` | Gehärteter JSON-RPC 2.0 Parser ohne dynamische Allokation im Fehlerfall |
| `generated` | Isolierte FlatBuffers-Generierung mit SAFETY-Invarianz |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `Keine (autarkes Framing- und Wire-Fundament)`
- **Verbotene Abhängigkeiten:** Höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Spezifiziert in `AGENTS.md` und `docs/SPEC.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-WIR-1: Zero-Copy Framing & Längenbegrenzung
- **Fokus / Betroffene Dateien:** `src/framing.rs`
- **Problemstellung & Baukasten-Ziel:** Verarbeitung von FlatBuffers-Nachrichten mit striktem Frame-Limit (16 MiB); CRC32-Validierung vor Speicherallokation.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-WIR-2: Gehärteter JSON-RPC 2.0 Parser
- **Fokus / Betroffene Dateien:** `src/json_rpc.rs`
- **Problemstellung & Baukasten-Ziel:** Zero-Allocation Parsing für Fehlermeldungen und standardisierte Wire-Status-Codes; strikter Schutz vor Rekursionsexplosion.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-WIR-3: FlatBuffers Isolation & SAFETY-Audit
- **Fokus / Betroffene Dateien:** `src/generated.rs`
- **Problemstellung & Baukasten-Ziel:** Kapselung der generierten FlatBuffers-Tabellen; lückenlose // SAFETY: Dokumentation für verbleibende Tabellen-Offsets.
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
2. `cargo check -p contextra-wire`, `cargo test -p contextra-wire` und `cargo clippy -p contextra-wire` laufen fehlerfrei durch.
3. Code-Budget von maximal 200 LOC wird strikt eingehalten.
4. Keine unzulässigen Querabhängigkeiten oder Lecks von Implementierungsdetails.
5. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel.
