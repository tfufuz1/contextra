# Contextra

> **Eingebettete Rust-Bibliothek für lokale Speicherung von Wissen und Gedächtnis von Agenten- und RAG-Systemen mit gemessenem Löschen.**
>
> *Normative Quelle der Wahrheit: [`docs/SPEC.md`](docs/SPEC.md) (Fassung 4.0.0 für Release 1.0.0)*

[![Rust Version](https://img.shields.io/badge/rust-1.89%2B-blue.svg)](rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)](LICENSE-MIT)
[![Architecture](https://img.shields.io/badge/architecture-Ring%200--4%20DAG-orange.svg)](docs/ARCHITECTURE.md)
[![Safety](https://img.shields.io/badge/safety-zero--panic%20%7C%20fail--closed-red.svg)](docs/CONSTITUTION.md)

---

## 1. Zulässige Außenaussage (SPEC §6.4)

> **DE:** Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis, aus den Indexen und aus den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Für jedes Ziel wird zusätzlich der zugehörige Schlüssel aus dem Schlüsseltresor entfernt; ob und wie das rechtlich zu werten ist, entscheidet der Betreiber. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter. Der Bericht listet diese Restrisiken.
>
> **EN:** Contextra removes the data of an erasure target from the data directory, from its indexes and from stored time-travel versions, and provides per-layer, measured evidence of the resulting state. For each target the associated key is additionally removed from the key vault; whether and how this is to be assessed legally is for the operator to decide. Not covered are external backups, flash remanence, file-system journals, process memory and external processors. The report lists these residual risks.

*Hinweis (HR-3):* Contextra garantiert einen messbaren technischen Zustand. Die rechtliche Bewertung obliegt stets dem Betreiber.

---

## 2. Mission & Alleinstellungsmerkmal (SPEC §1.1)

Contextra ist eine **eingebettete Rust-Bibliothek** (Single-Node, kein Server) für lokale Speicherung von Dokumenten (Textchunks, Metadaten, Zuordnung zu einer betroffenen Person), Volltextsuche (BM25), Vektorsuche (HNSW), Hybridsuche (RRF) und Zeitreisen über MVCC-Versionen.

**Löschen ist ein eigener, entkoppelter, gemessener Prozess auf drei Säulen (D-01, ADR-002):**
1. **Sofortwirkung unabhängig von der Zielgröße (P16):** Die Zeit bis `Gated` (Lesesperre persistiert, Schlüssel des Ziels im Tresor vernichtet) hängt nicht von der Dokumentenzahl ab (D-79, BM-ERASE-O1).
2. **Physischer Aufwand skaliert mit dem Ziel, nicht mit der Datenbank (P11):** Durch den Löschcluster-Schlüsselraum (D-31, D-70) schreibt der Purge nur Dateien um, die das Ziel tatsächlich enthalten (BM-ERASE-WA).
3. **Unabhängige Messung & Prüfbericht:** Typisierter Nachweis (`LayerWitness`) mit Positivkontrolle (Messklassen MK1 Struktur, MK2 Rohscan, MK3 Schlüsselprobe) und Ed25519-signierter Bericht, prüfbar durch `ctx-verify`.

---

## 3. Architektur & Ringmodell (SPEC §5.1, §5.2)

Die Architektur folgt einem strikten gerichteten azyklischen Graphen (DAG). Abhängigkeiten fließen ausschließlich abwärts:

```
Ring 4  contextra                         (Fassade, Composition Root)
Ring 3  contextra-erasure   contextra-engine
Ring 2  contextra-mvcc  contextra-store  contextra-text  contextra-vector  contextra-rank
Ring 1  contextra-sys  contextra-simd  contextra-durable-fs  contextra-crypto
Ring 0  contextra-types  contextra-ports
dev     testkit  lab  ctx-verify  tests/arch
```

### Die 14 Produktions-Crates

- **Ring 0:** `contextra-types` (reine Typen, blinde Tags, I/O-frei), `contextra-ports` (Port-Traits, `Vfs`, `StorageEngine`, `ErasureLayer`, Determinismus).
- **Ring 1:** `contextra-sys` (POSIX-Unsafe-Insel), `contextra-simd` (f32-Distanzkernel-Unsafe-Insel), `contextra-durable-fs` (`RealVfs`), `contextra-crypto` (AES-256-GCM-SIV, Argon2id, HKDF, Ed25519).
- **Ring 2:** `contextra-mvcc` (Snapshots, SSI, Sequenzen), `contextra-store` (LSM, WAL-HMAC-Kette, SSTables, Purge-Primitive), `contextra-text` (segmentierter BM25-Index), `contextra-vector` (HNSW mit Surrogat-Zeroing & Rebuild), `contextra-rank` (RRF-Fusion).
- **Ring 3:** `contextra-erasure` (Löschkern, Saga, Journal, ReadGate, Tresor, Messung, Bericht), `contextra-engine` (Orchestrierung, Collections, Transaktionen, Suche).
- **Ring 4:** `contextra` (Fassade, einzige öffentliche Oberfläche).

---

## 4. Kernregeln

- **Safe Rust:** `#![forbid(unsafe_code)]` in 12 von 14 Produktions-Crates. `unsafe` ist ausschließlich in `contextra-sys` und `contextra-simd` mit `// SAFETY:`-Beweiskommentar erlaubt.
- **Zero-Panic:** Keine unhandled Panics (`unwrap`, `expect`, `panic!`, ungeprüftes Index-Slicing verboten).
- **Determinismus:** Zeit, Zufall und IDs stammen ausschließlich aus Ports (`Clock`, `Rng`, `IdGen`).
- **Pflichtverschlüsselung:** AES-256-GCM-SIV ohne Opt-Out.
- **Single-Node:** Keine Netzwerk-Crates, kein Sharding, kein Server-Socket im Produktionsbaum.

---

## 5. Dokumentation

- [`docs/SPEC.md`](docs/SPEC.md): Einzige normative Quelle der Wahrheit für Contextra v1.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md): Detaillierte Crate- und Ring-Architektur.
- [`docs/CONSTITUTION.md`](docs/CONSTITUTION.md): Verfassungsprinzipien.
- [`docs/VETOES.md`](docs/VETOES.md): Feature-Vetos.
- [`docs/SECURITY.md`](docs/SECURITY.md): Bedrohungsmodell und CRA-Meldewege.
- [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md): Entwicklungsrichtlinien und Wortlaut-Lint.
