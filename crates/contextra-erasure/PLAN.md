# Implementierungsspezifikation & Arbeitsplan: `contextra-erasure`

> **Ring:** 3 | **Klasse:** `K` | **Code-Budget:** 7500 LOC | **Unsafe:** Streng verboten (#![forbid(unsafe_code)])
> **Architekturrolle:** Zentraler modularer Löschkern: Saga-Orchestrierung, FileVault, ReadGate, Audit-Ledger und Deletion-Reports
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-erasure` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

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
| `saga` | ErasureCore und DeletionSaga Zustandsmaschine |
| `vault` | FileVault: 16 Shards, 80-Byte Slots, physisches Zeroing von DEKs |
| `gate` | ReadGate und GateLog: RCU-basierte Lesesperren (ErasedSet) |
| `ledger` | ErasureLedger: Revisionssicheres Append-Only Logbuch |
| `journal` | Crash-resistentes WAL-Journal für unvollendete Sagas |
| `purge` | PurgeWindow: Sequenzielle Orchestrierung der Bereinigung über Purger-Ports (a-g) |
| `measure` | Restdaten-Messung (LayerScanner), Canary-Prüfung, LayerWitness |
| `report` | DeletionReport: Ed25519-signierte Zertifikate und DSGVO-Nachweisführung |
| `restore` | RestoreGuard: Schutz gegen unbemerktes Wiederherstellen aus Alt-Backups |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `contextra-types, contextra-ports, contextra-crypto`
- **Verbotene Abhängigkeiten:** Höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Spezifiziert in `AGENTS.md` und `docs/SPEC.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-ERA-1: Autarke Port-Anbindung
- **Fokus / Betroffene Dateien:** `src/backend.rs, src/purge.rs`
- **Problemstellung & Baukasten-Ziel:** Steuerung von Datenhaltern ausschließlich über Purger und LayerScanner aus contextra-ports ohne Crate-Querabhängigkeiten.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-ERA-2: FileVault Härtung & Poison-Semantik
- **Fokus / Betroffene Dateien:** `src/vault.rs, src/vault/`
- **Problemstellung & Baukasten-Ziel:** 80-Byte Slots, 2-Phasen-Vernichtung (Zeroing + Sync + Readback-Check); dauerhafte Poison-Vergiftung bei I/O-Fehlern (D-55).
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-ERA-3: Saga-Zustandsmaschine & Cancel-Safety
- **Fokus / Betroffene Dateien:** `src/saga.rs, src/saga/`
- **Problemstellung & Baukasten-Ziel:** Strikte Abfolge (Requested -> Gated -> KeyDestroyed -> Purging -> Measured -> Completed); Entkopplung mutatinger Operationen via tokio::spawn (ER-24).
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-ERA-4: Ed25519-Signierte Deletion-Reports V5
- **Fokus / Betroffene Dateien:** `src/report.rs, src/report/`
- **Problemstellung & Baukasten-Ziel:** Kanonische JSON-Serialisierung (MFDR), kryptografische Signatur und Einbettung aller LayerWitnesses.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-ERA-5: RestoreGuard & ErasedSet-Mirror
- **Fokus / Betroffene Dateien:** `src/restore.rs, src/restore/`
- **Problemstellung & Baukasten-Ziel:** Lückenloser Schutz vor Re-Import gelöschter Schlüssel aus alten Snapshots.
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
2. `cargo check -p contextra-erasure`, `cargo test -p contextra-erasure` und `cargo clippy -p contextra-erasure` laufen fehlerfrei durch.
3. Code-Budget von maximal 7500 LOC wird strikt eingehalten.
4. Keine unzulässigen Querabhängigkeiten oder Lecks von Implementierungsdetails.
5. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel.
