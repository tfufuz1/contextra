# Contextra – Architektur (Ist-Stand)

| Feld | Wert |
|---|---|
| Stand | 31.10.2026, Commit `3a0b11699` |
| Status | Nicht normativ. Normativ: `docs/SPEC.md` 4.1 |
| Messmethode | Statische Auswertung von `Cargo.toml` (`[dependencies]`), `capabilities.toml`, `git grep` auf `src/` |
| Nicht gemessen | Laufzeitverhalten, Speicherbedarf, Latenz, Build- und Testergebnisse |

---

## 1. Überblick

Contextra ist ein Cargo-Workspace mit 35 Crates unter `crates/`. Der Workspace enthält zusätzlich `benchmarks/contextra-bench` und das Mitglied `xtask`, dessen Verzeichnis fehlt (siehe §5).

Die Crates sind in Ringe eingeteilt. Ein Ring ist eine Schicht: Crates in Ring N dürfen nur von Ringen ≤ N abhängen. Die Zuordnung steht in `capabilities.toml`. Das Ringmodell ist über die deklarierten Abhängigkeiten am Commit vollständig eingehalten (0 Verletzungen).

## 2. Ringmodell

| Ring | Rolle | Crates (Anzahl) |
|---|---|---|
| 0 | Fundament: Typen, Ports, Speicher-Primitive, Index, Volltext, Rang, Krypto | 15 |
| 1 | Speicher und Checkpoint | 3 |
| 2 | Adapter und Ausführung (Inferenz, Sandbox) | 4 |
| 3 | Engine, Routing, Privacy, Agent, Datenbank-Fassade | 6 |
| 4 | Oberfläche: Fassade, MCP, Python, Ingest, Lizenz | 5 |
| Tooling | Testkit | 1 |

`capabilities.toml` enthält 34 Einträge, das Repo 35 Crate-Verzeichnisse. Es fehlt `contextra-durable-fs` (Befund D6). Die Ring-Tabelle unten zählt deshalb 34 Crates.

Die Ring-Zuordnung in `capabilities.toml` weicht an einer Stelle vom Inventar ab: `contextra-license` ist dort Ring 4. Die Spezifikation 3.1 nennte Ring 0. Die Datei `capabilities.toml` gilt (SPEC §1).

Hinweis zur Ringverletzung: Ein Ring-0-Crate darf kein Dateisystem-I/O ausführen (Invariante A3). `contextra-vector` verletzt das mit 28 Treffern in `src/` (ohne Testcode). Die Regel ist in der Prüfung `check_ring0_async_purity` nicht vollständig abgedeckt.

## 3. Crate-Inventar

Generiert aus `Cargo.toml` und `capabilities.toml` am Commit `3a0b11699`. „Abh.“ zählt die internen `contextra-*`-Abhängigkeiten aus `[dependencies]`.

| Crate | Ring | Reife | Abh. | Interne Abhängigkeiten |
|---|---|---|---|---|
| `contextra` | Ring 4 | stable | 12 | audit-export, avv-generator, core, crypto, db, infer-candle, license, ports, privacy, rank, store, types |
| `contextra-adapt` | Ring 0 | beta | 2 | ports, types |
| `contextra-agent` | Ring 3 | beta | 7 | checkpoint, db, graph, ports, router, store, types |
| `contextra-audit-export` | Ring 0 | experimental | 2 | ports, types |
| `contextra-avv-generator` | Ring 0 | experimental | 1 | types |
| `contextra-checkpoint` | Ring 1 | stable | 3 | mvcc, ports, types |
| `contextra-cognition` | Ring 3 | experimental | 6 | engine, graph, ports, store, types, vector |
| `contextra-core` | Ring 0 | deprecated | 4 | mvcc, ports, types, wire |
| `contextra-crypto` | Ring 0 | beta | 2 | ports, types |
| `contextra-db` | Ring 3 | beta | 13 | adapt, checkpoint, cognition, crypto, engine, graph, ports, rank, store, sys, text, types, vector |
| `contextra-durable-fs` | ? | ? | 1 | sys |
| `contextra-engine` | Ring 3 | beta | 12 | checkpoint, crypto, graph, mvcc, ports, rank, sandbox, store, sys, text, types, vector |
| `contextra-graph` | Ring 0 | stable | 3 | adapt, ports, types |
| `contextra-infer-candle` | Ring 2 | stable | 4 | crypto, ports, rank, types |
| `contextra-infer-ollama` | Ring 2 | beta | 3 | ports, rank, types |
| `contextra-infer-onnx` | Ring 2 | experimental | 3 | ports, rank, types |
| `contextra-ingest` | Ring 4 | experimental | 2 | engine, types |
| `contextra-kvcache` | Ring 1 | beta | 3 | crypto, ports, types |
| `contextra-license` | Ring 4 | experimental | 2 | ports, types |
| `contextra-mcp` | Ring 4 | beta | 12 | contextra, crypto, infer-candle, infer-ollama, infer-onnx, license, ports, privacy, rank, router, types, wire |
| `contextra-mvcc` | Ring 0 | stable | 2 | ports, types |
| `contextra-ports` | Ring 0 | stable | 1 | types |
| `contextra-privacy` | Ring 3 | stable | 3 | avv-generator, ports, types |
| `contextra-py` | Ring 4 | stable | 6 | adapt, db, rank, store, types, wire |
| `contextra-rank` | Ring 0 | stable | 2 | ports, types |
| `contextra-router` | Ring 3 | stable | 5 | adapt, ports, privacy, types, wire |
| `contextra-sandbox` | Ring 2 | stable | 0 | – |
| `contextra-simd` | Ring 0 | beta | 1 | types |
| `contextra-store` | Ring 1 | beta | 5 | crypto, mvcc, ports, sys, types |
| `contextra-sys` | Ring 0 | stable | 0 | – |
| `contextra-testkit` | Tooling | stable | 2 | ports, types |
| `contextra-text` | Ring 0 | beta | 2 | ports, types |
| `contextra-types` | Ring 0 | stable | 0 | – |
| `contextra-vector` | Ring 0 | beta | 6 | crypto, mvcc, ports, simd, sys, types |
| `contextra-wire` | Ring 0 | stable | 0 | – |

In dieser Tabelle steht `?`, wo `capabilities.toml` keinen Eintrag hat (Befund D6). Die Tabelle wird von Hand gepflegt, bis das Generator-Skript aus PLAN K2.3 existiert.

## 4. Abhängigkeitsstruktur

Zentrale Bausteine (Fan-in, Anzahl direkter Abhängiger):

| Crate | Fan-in | Bedeutung |
|---|---|---|
| `contextra-types` | 30 | Domänenmodelle, von allen Produkten getragen |
| `contextra-ports` | 25 | Traits für Austauschbarkeit (Ports und Adapter) |
| `contextra-crypto` | 8 | AES-256-GCM-SIV, HMAC-Ketten, Löschnachweis |
| `contextra-rank` | 8 | Rangfusion und Kalibrierung |
| `contextra-store` | 6 | LSM, WAL, Verschlüsselung im Ruhezustand |
| `contextra-mvcc` | 5 | Snapshot-Isolation |
| `contextra-sys` | 5 | Plattform-Schicht, `mlock`, `mmap` |

Sammelstellen (Fan-out):

| Crate | Abhängigkeiten | Bewertung |
|---|---|---|
| `contextra-db` | 13 | Legacy-Fassade, Auflösung geplant (Strangler) |
| `contextra-engine` | 12 | Orchestrator, enthält Erasure-Module und Chunker |
| `contextra-mcp` | 12 | Protokoll-Adapter, zieht Inferenz, Lizenz, Wire |
| `contextra` | 12 | Gesamtfassade mit `builder()` |

Blattknoten ohne interne Abhängigkeiten: `contextra-sandbox`, `contextra-sys`, `contextra-types`, `contextra-wire`.

## 5. Bekannte Defekte am Commit

| Nr. | Defekt | Wirkung | Fundstelle | Behebung |
|---|---|---|---|---|
| D1 | `xtask` ist Workspace-Mitglied, Verzeichnis fehlt | `cargo metadata` schlägt fehl, kein Build möglich | `Cargo.toml` | PLAN K0.1 |
| D2 | Feature `onnx` ohne `dep:contextra-infer-onnx` | Build mit `--features onnx` schlägt fehl | `contextra-engine/Cargo.toml`, `lifecycle.rs:188` | PLAN K0.2 |
| D3 | Feature `adaptive-candidate-pool-sizing` ohne `dep:contextra-adapt` | Build mit diesem Feature schlägt fehl | `contextra-engine/Cargo.toml`, `builder.rs:45` | PLAN K0.3 |
| D4 | Dateisystem-I/O in Ring 0 | Ringinvariante verletzt | `contextra-vector`, 28 Treffer | PLAN K0.7 |
| D5 | 20 Stellen mit `#[ignore]` im Workspace | Testabdeckung kleiner als die gezählten 4.729 Testfunktionen | verteilt, u. a. `contextra-store` | Prüfung in K0.5 |
| D6 | `contextra-durable-fs` hat keinen Eintrag in `capabilities.toml` | Ring und Reifegrad unbekannt, Invariante A1 wird für diese Crate nicht geprüft | `crates/contextra-durable-fs/` | Eintrag ergänzen, Ring festlegen (PLAN K2.5) |

## 6. Unsafe-Fläche

Gezählt als Treffer des Wortes `unsafe` in `src/`, nicht als Funktionen:

| Crate | Treffer | Deklariert als Insel |
|---|---|---|
| `simd` | 271 | ja |
| `wire` | 46 | ja |
| `sys` | 21 | ja |
| `infer-onnx` | 5 | nein |
| `crypto` | 5 | nein |
| `ingest` | 10 | nein |
| `text` | 8 | nein |

Gesamt: 383 Treffer. Die Zahl ist eine obere Schranke, da Kommentare mitgezählt sind. Ein Audit-Schritt muss die Treffer nach `unsafe {`, `unsafe fn` und `unsafe impl` trennen.

## 7. Datenfluss

Die Beschreibung des Datenflusses (Schreibpfad über WAL und LSM, Lesepfad über HNSW, BM25 und Rangfusion) stammt aus der Crate-Dokumentation und dem README. Am Commit ist sie nicht durch Tests belegt. Sie wird in K0.5 mit den Tests abgeglichen.

Schreibpfad (laut Fassung 3.1, nicht verifiziert): Dokument → Chunking (`ingest`/`engine`) → Vektor (`vector`) und Volltextindex (`text`) → Speicher (`store`, WAL mit HMAC-Kette, Verschlüsselung).

Lesepfad (laut Fassung 3.1, nicht verifiziert): Anfrage → BM25 (`text`) und Vektorsuche (`vector`) → Rangfusion (`rank`) → Mandantenfilter.

## 8. Dokumentation

| Datei | Inhalt | Normativ |
|---|---|---|
| `docs/SPEC.md` | Scope, Build-Vertrag, Invarianten | ja |
| `docs/PLAN.md` | Umsetzungsschritte | nein, Plan |
| `docs/ARCHITECTURE.md` | diese Datei | nein |
| `docs/decisions/` | ADRs | nur bestätigte |
| `docs/analysis/BEFUNDE_2026-10-31.md` | Messungen mit Befehlen | nein, Beleg |
| `docs/spec/archiv/` | Fassungen 3.1, 4.0 | nein |
