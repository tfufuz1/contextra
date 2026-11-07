# Implementierungsspezifikation & Arbeitsplan: `contextra-sys`

> **Ring:** 0 | **Klasse:** `K` | **Code-Budget:** 450 LOC | **Unsafe:** Genehmigte Unsafe-Insel (// SAFETY: Pflicht)
> **Architekturrolle:** Autarke OS-Primitive, Memory-Locking, Speichersperren und Datei-Locks als gekapselte Unsafe-Insel
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-sys` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

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
| `memlock` | mlock/munlock Kapselung für sensible Keys und Buffers |
| `hardening` | RLIMIT_CORE Deaktivierung, Speicherabdrücke verhindern |
| `perm` | POSIX-Berechtigungen (0o700 / 0o600) für Vault- und Datendateien |
| `durability` | Plattformspezifisches fsync/fdatasync/F_FULLFSYNC |
| `fs_probe` | Dateisystem-Typ-Erkennung (ext4, xfs, btrfs, zfs, apfs) |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `Keine (autarkes Low-Level-Systemfundament)`
- **Verbotene Abhängigkeiten:** Höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Spezifiziert in `AGENTS.md` und `docs/SPEC.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-SYS-1: POSIX Memory-Locking & Hardening
- **Fokus / Betroffene Dateien:** `src/memlock.rs, src/hardening.rs`
- **Problemstellung & Baukasten-Ziel:** Sichere mlock- und madvise-Wrapper gegen Swap-Out und Core-Dumps; strikte // SAFETY: Dokumentation für jeden unsafe Block.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-SYS-2: Durability & Sync-Primitive
- **Fokus / Betroffene Dateien:** `src/durability.rs`
- **Problemstellung & Baukasten-Ziel:** Kanalisierung aller atomaren Sync-Operationen; F_FULLFSYNC unter macOS, fdatasync unter Linux; saubere Fehlerpropagation.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-SYS-3: Dateisystem-Probe & Validierung
- **Fokus / Betroffene Dateien:** `src/fs_probe.rs`
- **Problemstellung & Baukasten-Ziel:** Erkennung nicht-unterstützter Filesysteme (NFS, SMB, FAT); Fail-Closed-Verhalten bei unzureichenden Durability-Semantiken.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-SYS-4: Line-Budget & Unsafe-Audit
- **Fokus / Betroffene Dateien:** `src/lib.rs`
- **Problemstellung & Baukasten-Ziel:** Einhaltung des Budgets <= 450 LOC; statische Verifikation aller 19 genehmigten unsafe-Blöcke gegen docs/unsafe-register.md.
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
2. `cargo check -p contextra-sys`, `cargo test -p contextra-sys` und `cargo clippy -p contextra-sys` laufen fehlerfrei durch.
3. Code-Budget von maximal 450 LOC wird strikt eingehalten.
4. Keine unzulässigen Querabhängigkeiten oder Lecks von Implementierungsdetails.
5. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel.
