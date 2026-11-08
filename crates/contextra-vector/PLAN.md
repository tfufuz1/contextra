# Implementierungsspezifikation & Arbeitsplan: `contextra-vector`

> **Ring:** 1 | **Klasse:** `K` | **Code-Budget:** 6800 LOC | **Unsafe:** Streng verboten (#![forbid(unsafe_code)])
> **Architekturrolle:** Autarker HNSW-Vektorindex, SQ8-Quantisierung, Mandantenisolation und modulare Wartungsprimitive
> **Spezifikationsbezug:** `docs/SPEC.md` §4, `PLAN.md`

---

## 1. Architektonische Leitprinzipien & Baukasten-Vertrag

Für `contextra-vector` gelten im Rahmen der Transformation zum modularen Baukastensystem folgende verbindliche Richtlinien:

1. **Modulare Autarkie & Isolation:** Der Crate funktioniert als eigenständiger, isolierter Baustein. Direkte Kopplungen an Fremd-Crates werden über wohldefinierte Schnittstellen in `contextra-ports` oder generische Abstraktionen gelöst.
2. **Entkopplung der Löschkette:** Sämtliche juristischen Begrifflichkeiten (DSGVO Art. 17), Nachweisketten, Deletion-Proofs und Saga-Orchestrierungen verbleiben exklusiv in `contextra-erasure`. Datenhaltende Module implementieren ausschließlich neutrale, funktionale Bereinigungs- und Wartungsprimitive.
3. **Design by Contract (DbC) & Invarianten:** Alle Vorbedingungen, Nachbedingungen und Systeminvarianten werden strikt geprüft (Fail-Closed).
4. **Zero Shared Mutable State:** Nebenläufigkeit wird über deterministisches Message-Passing, unteilbare Transaktionszustände oder atomare Sequenznummern strukturell abgesichert.
5. **Code- & Unsafe-Disziplin:** `#![forbid(unsafe_code)]` ist im gesamten Crate verbindlich gesetzt. Keine unwrap(), expect() oder Panics im Produktivcode.

---

## 2. Modul- & Komponentenarchitektur

### 2.1 Interne Struktur & Verantwortung

| Komponente / Modul | Verantwortung & Schnittstellen |
|---|---|
| `hnsw` | HNSW Hierarchie, Graph-Aufbau, euklidische/Kosinus-Suche |
| `quant` | SQ8 Skalare Quantisierung zur Speicherreduktion um 75% |
| `deletion` | Geisterzeigerfreie Vektorknoten-Löschung und Rebuild-Notify |
| `persistence` | BufferIndex - persistente Serialisierung |
| `purge` | Wartungs- und Bereinigungsprimitive für Vektorräume (deprecated) |

### 2.2 Abhängigkeitsmatrix & Schnittstellen

- **Erlaubte interne Abhängigkeiten:** `contextra-types, contextra-ports, contextra-mvcc, contextra-simd, contextra-sys`
- **Verbotene Abhängigkeiten:** `contextra-crypto`, höhere Ringe sowie Querverbindungen außerhalb der deklarierten Ports.
- **Konsumierte / Implementierte Ports:** Spezifiziert in `AGENTS.md` und `docs/SPEC.md`.

---

## 3. Zerlegung in Arbeitspakete (Work Packages)

### WP-VEC-1: Entkopplung von Krypto & Löschbeweisen
- **Fokus / Betroffene Dateien:** `src/diskann/integrity.rs, src/lib.rs`
- **Status:** Abgeschlossen. `contextra-crypto` vollständig entfernt.

### WP-VEC-2: BufferIndex & mmap-Optimierung
- **Fokus / Betroffene Dateien:** `src/persistence/`
- **Status:** In Bearbeitung / Migrationspfad.

### WP-VEC-3: Geisterzeigerfreie Löschung & Reparatur
- **Fokus / Betroffene Dateien:** `src/hnsw/deletion.rs`
- **Status:** Etabliert; Absicherung von INV-HNSW-SLOT-1 und VETO-F02.

---

## 4. Definition of Done (DoD) für diesen Baustein

1. Alle deklarierten Arbeitspakete sind vollständig implementiert.
2. `cargo check -p contextra-vector`, `cargo test -p contextra-vector` und `cargo clippy -p contextra-vector` laufen fehlerfrei durch.
3. Keine unzulässigen Querabhängigkeiten zu `contextra-crypto`.
4. Die Crate-README dokumentiert den autarken Einsatz in Drittprojekten mit einem Minimalbeispiel (`examples/standalone_vector.rs`).
