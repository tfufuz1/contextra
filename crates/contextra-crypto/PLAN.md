# Implementierungsspezifikation & Arbeitsplan: `contextra-crypto`

> **Ring:** 0 | **Klasse:** `K` | **Code-Budget:** 3200 LOC | **Unsafe:** Streng verboten (#![forbid(unsafe_code)])
> **Architekturrolle:** Reiner Kryptografiekern: AEAD, KDF, WAL-HMAC und Anti-Tamper (ohne Lösch-/DSGVO-Logik)
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-crypto` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

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
| `aead` | XChaCha20-Poly1305 und AES-256-GCM für verschlüsselte Blockdaten |
| `kdf` | HKDF-SHA256 und Argon2id Schlüsselableitung (MIN_M_KIB >= 64 MiB) |
| `chain` | WAL HMAC-Chaining und Anti-Tamper-Verifikation |
| `sign` | Ed25519 Signatur- und Verifikationsprimitiven |
| `tag` | Konstantzeit-MAC-Validierung mit subtle::ConstantTimeEq |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `contextra-types, contextra-ports`
- **Verbotene Abhängigkeiten:** Höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Spezifiziert in `AGENTS.md` und `docs/SPEC.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-CRY-1: Entflechtung der Löschlogik
- **Fokus / Betroffene Dateien:** `src/lib.rs`
- **Problemstellung & Baukasten-Ziel:** Vollständige Auslagerung von deletion_proof/, kv_shredding und DSGVO-Art-17-Nachweisen in contextra-erasure; Reduzierung auf reine Krypto-Bausteine.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-CRY-2: WAL HMAC Chain Completeness
- **Fokus / Betroffene Dateien:** `src/chain.rs`
- **Problemstellung & Baukasten-Ziel:** Inkrementelle HMAC-SHA256 Verkettung für WAL-Einträge; Konstantzeit-Validierung verhindert Chain-Manipulationen.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-CRY-3: Gehärtete KDF & Schlüsselableitung
- **Fokus / Betroffene Dateien:** `src/kdf.rs`
- **Problemstellung & Baukasten-Ziel:** Arretierung von Argon2id Parametern (Memory >= 64 MiB) gemäß SPEC D-06; ZeroizeOnDrop für alle intermediären Schlüssel.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-CRY-4: Loom Concurrency Härtung
- **Fokus / Betroffene Dateien:** `tests/loom_kv.rs`
- **Problemstellung & Baukasten-Ziel:** Verifikation von nebenläufigen Zeroize- und Zugriffspfaden unter loom.
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
2. `cargo check -p contextra-crypto`, `cargo test -p contextra-crypto` und `cargo clippy -p contextra-crypto` laufen fehlerfrei durch.
3. Code-Budget von maximal 3200 LOC wird strikt eingehalten.
4. Keine unzulässigen Querabhängigkeiten oder Lecks von Implementierungsdetails.
5. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel.
