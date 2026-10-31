# Contextra

> **Lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust mit gemessenem Löschen**
> 
> *Normative Grundlage: [`docs/SPEC.md`](docs/SPEC.md) (Fassung 2.4, Zielzustand v1.0) & [`PLAN.md`](PLAN.md)*

[![Rust Version](https://img.shields.io/badge/rust-1.89%2B-blue.svg)](rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green.svg)](LICENSE-MIT)
[![Architecture](https://img.shields.io/badge/architecture-Ring%200--4%20DAG-orange.svg)](docs/ARCHITECTURE.md)
[![Safety](https://img.shields.io/badge/safety-zero--panic%20%7C%20fail--closed-red.svg)](docs/CONSTITUTION.md)

---

## 1. Was Contextra ist

Contextra ist ein **lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust** (Single-Node, ein Primärprozess hält ein Datenverzeichnis). Er speichert Dokumente in einem LSM-Speicher mit Write-Ahead-Log (WAL) als einziger Quelle der Wahrheit, findet sie über eine hybride Suche (Vektorsuche HNSW + Volltext BM25, reziproke Rangfusion RRF, strikte Mandantentrennung) und **löscht gemessen** über physischen Purge der Arbeitsspeicher-Strukturen und kryptografische Schlüsselvernichtung (*Crypto-Shredding*) mit extern prüfbarem Löschbericht.

### Hauptzugänge
1. **Rust-Bibliothek (`contextra`, `contextra-db`):** Direkte Einbindung in Desktop- und Serveranwendungen via `ContextraBuilder`.
2. **Model Context Protocol (`contextra-mcp`):** Lokaler stdio MCP-Server (JSON-RPC 2.0) für KI-Agenten und Entwicklungsumgebungen.

---

## 2. Architektur & Ringmodell

Die Architektur folgt einem strikt gerichteten, azyklischen Graphen (DAG). Abhängigkeiten fließen **ausschließlich abwärts** (Ring 4 → Ring 0).

```
Ring 4: Fassade & Schnittstellen   contextra (Composition Root) · contextra-mcp · [R] contextra-py · [E] license · [E] audit-export · [E] avv-generator
Ring 3: Orchestrierung            contextra-engine · contextra-db · [R] contextra-privacy
Ring 2: Isolation & Inferenz      contextra-infer-ollama · [R] infer-candle* · [R] sandbox*
Ring 1: Persistenz                contextra-store · contextra-checkpoint
Ring 0: Fundament (Sovereign)     types · ports · mvcc · crypto · vector · text · rank · durable-fs · core
Unsafe-Inseln (Ring 0)            sys · simd · wire
Tooling & Benchmarks              testkit · bench

* = Im Rahmen der Kernfokussierung (PLAN.md §12.5) temporär eingefroren
```

### Die 15 Kern-Crates (Klasse `[K]`)
1. **`contextra-types`** (Ring 0): `RequestContext`, Identitäten, Fehlerdomänen (`ContextraError`).
2. **`contextra-ports`** (Ring 0): Synchrone Verträge (`StorageEngine`, `Clock`, `Rng`, `IdGen`, `KmsProvider`, `Vfs`).
3. **`contextra-sys`** (Ring 0, *Unsafe*): OS-FFI, `mmap_readonly`, Durability-Primitive (`durable_flush_file`).
4. **`contextra-simd`** (Ring 0, *Unsafe*): Vektordistanz-Kernel mit skalarem Fallback-Orakel.
5. **`contextra-wire`** (Ring 0, *Unsafe*): FlatBuffers IPC und Zero-Copy-Dekodierung.
6. **`contextra-mvcc`** (Ring 0): Snapshot-Isolation, SequenceLog, TxBuffer, SSI-Verifikation.
7. **`contextra-crypto`** (Ring 0): AEAD, Argon2id/HKDF KDF, KeyRegistry, RevocationLog, Ed25519.
8. **`contextra-text`** (Ring 0): BM25/BM25F, Komposita-Zerlegung, Block-Max WAND.
9. **`contextra-vector`** (Ring 0): HNSW-Index, SQ8-Quantisierung, gefilterte Suche, bitset-gefilterte Distanz.
10. **`contextra-rank`** (Ring 0): Reziproke Rangfusion (RRF), Normalisierung, Explainability.
11. **`contextra-store`** (Ring 1): LSM-Tree, WAL, SSTables, Kompaktion, Block-Cache, Segment-Verschlüsselung.
12. **`contextra-checkpoint`** (Ring 1): Snapshot-Erstellung, Restore-Guard, Backup-Export.
13. **`contextra-engine`** (Ring 3): Hybrid-Planer, Background-Reaper, ErasureSaga, Lesesperre.
14. **`contextra-db`** (Ring 3): Collections, Multi-Tenant-Scoping, Transaktionen.
15. **`contextra`** (Ring 4): Offizielle Rust-Hauptfassade & einzige Composition Root.

---

## 3. Ziel-Produktfamilie (PLAN §11)

| Produkt | Zweck & Wertversprechen | Crates |
|---|---|---|
| **EP1 Memory** | Eingebettete, verschlüsselte Hybrid-Suche mit Gedächtnis-Lebenszyklus, mandantengetrennt, lokal | `store`, `vector`, `text`, `engine`, `db`, `durable-fs` |
| **EP2 Proof** | Extern prüfbare Löschbeweise und DSGVO-Nachweise (HMAC + Ed25519) | `audit-export`, `avv-generator`, `verify` |
| **EP3 Graph** *(eingefroren)* | GraphRAG (Wissensgraph, PPR, Community-Detection) | `graph` |
| **EP4 Inference** | Lokale Inferenz-Provider (Ollama, ONNX, Candle) | `infer-ollama`, `infer-onnx`, `infer-candle` |
| **EP5 Agent** *(eingefroren)* | Wiederanlauffähige, budgetierte Agenten-Laufzeit mit Routing | `agent`, `router` |
| **EP6 Guard** | Fail-Closed-Firewall für LLM-Ein-/Ausgänge, PII-Detektion, Sandbox | `privacy`, `sandbox` |
| **EP7 Ingest** | Offline-Dokumentenextraktion (PDF, DOCX, Mail, Markdown) zu kanonischen Chunks | `ingest` |

---

## 4. Für Implementierer: Bauen & Kompilieren

Detaillierte Schritt-für-Schritt-Anweisungen und Fehleranalysen befinden sich in [`docs/COMPILATION.md`](docs/COMPILATION.md).

### Schnellstart

```bash
# 1. Metadaten-Check
cargo metadata --format-version 1 --no-deps

# 2. Fundament prüfen (Ring 0)
cargo check -p contextra-types
cargo check -p contextra-sys
cargo check -p contextra-crypto
cargo check -p contextra-vector
cargo check -p contextra-text

# 3. Speicher & Engine prüfen (Ring 1 & Ring 3)
cargo check -p contextra-store
cargo check -p contextra-engine
cargo check -p contextra-db

# 4. Fassade prüfen (Ring 4)
cargo check -p contextra
cargo check -p contextra-mcp

# 5. Alle aktiven Standard-Mitglieder kompilieren
cargo check --workspace
```

### Entwicklungs-Workflow mit `just`

```bash
just check       # Formatierung, Clippy & Cargo-Check
just test        # Führt Tests aus (cargo test --locked)
just gates       # Führt Layering- und Architektur-Gates aus
```

---

## 5. Kern-Regeln für Codeänderungen

1. **Ring & Klasse:** Keine Kante zeigt nach oben; Kern `[K]` hängt niemals von Rand `[R]` oder experimentellem Code `[E]` ab.
2. **Zero Panic:** Niemals `unwrap()`, `expect()`, `panic!` oder ungeprüftes Indexieren in Produktionscode. Alle Fehler sind typisiert (`ContextraError`).
3. **P28 Determinismus:** Zeit, Zufall und IDs ausschließlich über die Ports `Clock`, `Rng`, `IdGen` (Ausnahme: Krypto-RNG).
4. **WAL First:** Keine In-Memory-Modifikation vor dauerhaftem WAL-Commit (`fsync`).
5. **Fail-Closed:** Unbekannte Zustände oder Krypto-Fehler führen zu sofortigem Fehlerabbruch, niemals zu stillen Fallbacks.
6. **Mandanten-Isolation:** `TenantId` stammt zwingend aus dem `RequestContext`, niemals aus Nutzereingaben oder Parametern (VETO-F10).

---

## 6. Wichtige Dokumente

- 📖 [Gesamtspezifikation 2.4 (normativ)](docs/SPEC.md)
- 📋 [Implementierungsplan & Umsetzungsstrategie](PLAN.md)
- 🏗️ [Architektur-Dokumentation](docs/ARCHITECTURE.md)
- 🛠️ [Kompilieranleitung für Implementierer](docs/COMPILATION.md)
- 🔒 [Sicherheitsmodell & Bedrohungsanalyse](docs/SECURITY.md)
- ⚖️ [DSGVO-Löschkonzept & Crypto-Shredding](docs/DELETION-DSGVO.md)
- 🏛️ [Projekt-Verfassung & Governance](docs/CONSTITUTION.md)
- 🛑 [Veto-Register](docs/VETOES.md)
- 📜 [Quellen der Wahrheit](docs/SOURCE_OF_TRUTH.md)
- 🏷️ [Typ-Registry](docs/TYPE_REGISTRY.md)
