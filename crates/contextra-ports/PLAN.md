# Implementierungsspezifikation & Arbeitsplan: `contextra-ports`

> **Ring:** 0 | **Klasse:** `K` | **Code-Budget:** 1600 LOC | **Unsafe:** Streng verboten (#![forbid(unsafe_code)])
> **Architekturrolle:** Zentraler Trait-Vertrag für alle Bausteine: Vfs, StorageEngine, VectorIndex, TextIndex, Purger, LayerScanner, Clock, Rng
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-ports` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

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
| `vfs` | Konsolidierter asynchroner Vfs- und VfsFile-Port (Streaming, Atomic Writes, Fsync) |
| `storage` | StorageEngine Trait für Key-Value- und LSM-Backends |
| `vector_index` | VectorIndex Trait für HNSW- und DiskANN-Schnittstellen |
| `text_index` | TextIndex Trait für BM25/BM25F-Volltext-Backends |
| `graph_index` | GraphIndex Trait für semantische Wissensnetze und CSR-Segmente |
| `purge` | Purger & LayerScanner Schnittstellen zur neutralen Bereinigung und Restdatenmessung |
| `clock` | Clock Trait für deterministische Zeitinjektion ohne direkte OS-Zeit-Kopplung |
| `rng` | EntropyRng Trait für kryptografische Zufallszahlen ohne globale Singletons |
| `egress` | EgressPolicy Port zur Entkopplung von Router und PII-Filtern |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `contextra-types`
- **Verbotene Abhängigkeiten:** Höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Spezifiziert in `AGENTS.md` und `docs/SPEC.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-PRT-1: Vfs-Port Konsolidierung
- **Fokus / Betroffene Dateien:** `src/vfs.rs`
- **Problemstellung & Baukasten-Ziel:** Vollständige Abstraktion aller Dateisystem- und I/O-Zugriffe; Streaming-Lese/Schreiboperationen mit read_at und write_at; sync_data und sync_all.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-PRT-2: Neutrale Bereinigungs-Ports (Purger & LayerScanner)
- **Fokus / Betroffene Dateien:** `src/purge.rs`
- **Problemstellung & Baukasten-Ziel:** Definition von purge_scope(&KeyScope) -> PurgeOutcome und scan_for_residual(&KeyScope) -> ScanOutcome; vollständige Abwesenheit von DSGVO- oder Rechtslogik.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-PRT-3: Core Storage- & Index-Schnittstellen
- **Fokus / Betroffene Dateien:** `src/storage.rs, src/vector_index.rs, src/text_index.rs`
- **Problemstellung & Baukasten-Ziel:** Asynchrone Trait-Definitionen für get, put, commit, scan_prefix, search_knn und bm25_query.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-PRT-4: Umwelt-Invertierung (Clock, Rng, IdGen)
- **Fokus / Betroffene Dateien:** `src/clock.rs, src/rng.rs, src/id_gen.rs`
- **Problemstellung & Baukasten-Ziel:** Vollständige Invertierung von OS-Abhängigkeiten zur Ermöglichung deterministischer Zeitreise- und Sabotage-Tests.
- **Design by Contract (DbC) & Invarianten:**
  - Vorbedingungen: Gültige, typisierte Parameter; keine ungültigen Zustände oder unvollständige Kontexte.
  - Nachbedingungen: Deterministisches Verhalten; saubere Fehlerkapselung in `ContextraError` ohne Panics.
- **Akzeptanzkriterien:**
  - [ ] Implementierung ist modular gekapselt und autark testbar.
  - [ ] Alle Unit- und Integrationstests sind grün.
  - [ ] Keine Verletzung der Ring-Hierarchie oder Crate-Grenzen.

### WP-PRT-5: Egress- und Barriere-Ports
- **Fokus / Betroffene Dateien:** `src/egress.rs, src/barrier.rs`
- **Problemstellung & Baukasten-Ziel:** Definition von EgressPolicy zur Entkopplung von Query-Routing und PII-Redaktion; PurgeBarrier für koordinierte Flush-Zyklen.
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
2. `cargo check -p contextra-ports`, `cargo test -p contextra-ports` und `cargo clippy -p contextra-ports` laufen fehlerfrei durch.
3. Code-Budget von maximal 1600 LOC wird strikt eingehalten.
4. Keine unzulässigen Querabhängigkeiten oder Lecks von Implementierungsdetails.
5. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel.
