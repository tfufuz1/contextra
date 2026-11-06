# Contextra Baukasten – Refaktorisierungsplan

Begleitdokument zur „Contextra Baukasten – Spezifikation v3.0“ · Stand 06.11.2026 · Basis: `tfufuz1/contextra`

Dieser Plan legt Reihenfolge, Arbeitspakete, Gates und Risiken zur Transformation des Gesamtsystems in ein entkoppeltes, modulares Baukastensystem (Building Blocks) fest. Aufwände sind als relative Größen (S/M/L/XL) angegeben.

## 1. Leitprinzipien des Baukastensystems

1. **Modulare Autarkie & Isolation:** Jeder Crate und jede Komponente muss architektonisch sauber entkoppelt sein und isoliert funktionieren, um Bausteine (z. B. Storage, Vector-Index, BM25-Text, MVCC, Crypto) ohne transitive Altlasten in Fremdprojekten einsetzen zu können.
2. **Immer baubar:** Jeder Merge hinterlässt einen grünen `cargo build --workspace` und grüne Tests. Große Schnitte werden in wohldefinierte Teil-PRs zerlegt.
3. **Entkopplung der Löschkette (Modularisierung in `contextra-erasure`):** Die historische Multi-Layer-Löschkette mit Deletion-Proofs, Audit-Ketten, DSGVO-/Gesetzesbezügen und Nachweislogik wird vollständig aus dem Speicherkern (Store, Vector, Text, Crypto, DB, MCP etc.) entfernt und als eigenständiger, isolierter Baustein im Crate `contextra-erasure` modularisiert. Kern-Datenhalter implementieren lediglich generische, schlanke Bereinigungs-Ports (`Purger`, `LayerScanner`).
4. **Schnittstellen via Ports/Traits:** Direkte Abhängigkeiten zwischen konkreten Engines werden eliminiert. Abstraktionen liegen in `contextra-ports` oder generischen Parametern; interne Implementierungstypen lecken nicht über Modulgrenzen.
5. **DAG-Architektur & Zero-Dead-Code:** Ring-Struktur (Ringe 0 bis 4) und Abhängigkeitsrichtung werden maschinell validiert. Unnötige Querabhängigkeiten und toter Code werden eliminiert.

## 2. Überblick über die Phasen

| Phase | Inhalt | Größe | Gate |
| --- | --- | --- | --- |
| 0 | Baseline: Build, Tests, Entkopplungs-Inventar, Tag | M | G0 |
| 1 | Ring-Layout, DAG-Erzwingung und Entkopplungsprüfung | L | G1 |
| 2 | Ports v2 konsolidieren (Vfs, Purger, Storage, Vector, Text, Graph) | L | G2 |
| 3 | Löschkette entkoppeln & in `contextra-erasure` modularisieren | XL | G3 |
| 4 | Kerne entkoppeln: Append-friendly Store, Vector, Text, Graph | XL | G4 |
| 5 | Feingranulare Modularisierung, Abstraktion & Zero-Copy-Härtung | L | G5 |
| 6 | Produktschnitte, Baustein-Export & isolierte Beispiele | M | G6 |
| 7 | Dokumentation, Governance-Bereinigung und CI-Absicherung | S | G7 |

Phase 3 und 4 vollziehen die zentrale Modularisierung: Sie befreien den Speicherkern von der monolithischen Löschkette und kapseln die DSGVO-/Proof-Verantwortung autark in `contextra-erasure`.

## 3. Phase 0 – Baseline

**Ziel:** Ein belegter Ausgangszustand. Alles Weitere wird dagegen gemessen.

| WP | Arbeit | Ergebnis |
| --- | --- | --- |
| 0.1 | Rust-Toolchain installieren (`rust-toolchain.toml`, Min-Version 1.89), `cargo metadata` ausführen | nachweisen, ob D1 (`xtask`) noch gilt, beheben falls ja |
| 0.2 | `cargo build --workspace --locked`, `cargo test --workspace --locked` | Liste roter Tests; die 20 `#[ignore]`-Stellen erfassen |
| 0.3 | Befunde D2 bis D4 und D6 prüfen | jeder Befund: Status verifizieren und Behebung dokumentieren |
| 0.4 | Benchmarks aus `benchmarks/contextra-bench` laufen lassen | `baseline_20261106.json` im Repo |
| 0.5 | Kompilierzeit und Binary-Größe je Crate erfassen | Baseline-Metriken |
| 0.6 | `git tag pre-baukasten`, Branch `baukasten/main` | Rückfalloption |
| 0.7 | Lösch- und DSGVO-Inventar konsolidieren (Audit-Abgleich mit `docs/MODULAR_DECOUPLING_AUDIT.md`): Fundstellen außerhalb von `contextra-erasure` nach Crate und Datei erfassen | `docs/refactor/erase-inventory.md` |

**Gate G0:** Build und Tests grün (oder dokumentierte Ausnahmeliste), Baseline-Dateien und Inventar eingecheckt.

## 4. Phase 1 – Ring-Layout und DAG-Erzwingung

**Ziel:** Strikte Durchsetzung des 5-Ring-Schichtenmodells und maschinelle Prüfung der Abhängigkeitsrichtung.

| WP | Arbeit |
| --- | --- |
| 1.1 | `capabilities.toml`: Ring, Klasse und `may_depend_on` für alle Crates sauber definieren; `contextra-durable-fs` und `contextra-erasure` normativ festlegen. |
| 1.2 | Pfade und Workspace-Manifeste konsolidieren: `Cargo.toml`, `capabilities.toml`, `CODEOWNERS`, `justfile`, CI-Workflows. |
| 1.3 | DAG-Prüfung (`check-dag`) in CI einbinden: verifiziert Ring-Regeln und `capabilities.toml`-Einschränkungen. Bricht bei aufwärts gerichteten Kanten und unzulässigen Querabhängigkeiten. |
| 1.4 | `xtask/dag-exceptions.toml` für temporäre Übergangs-Ausnahmen anlegen (mit festem Ablauf-Meilenstein). |
| 1.5 | `deny.toml` aktualisieren zur Absicherung der Kern- vs. Extension-Grenzen. |

**Gate G1:** `check-dag` ist aktiv und schützt den Workspace; alle Kanten sind deklariert und legitimiert.

## 5. Phase 2 – Ports v2 (Vertragsschicht der Bausteine)

**Ziel:** Die Schnittstellen-Traits in `contextra-ports`, an denen alle Subsysteme hängen, werden konsolidiert und als stabiler Vertrag eingefroren.

| WP | Arbeit |
| --- | --- |
| 2.1 | Konsolidierung des `Vfs`-Ports: Vereinheitlichung auf `contextra_ports::Vfs` mit Streaming/Handle-Operationen (`read_at`, `write_at`, `sync_data`, `sync_all`); `contextra-durable-fs` implementiert `RealFs` gegen diesen Standard-Trait. |
| 2.2 | Bereinigungs- und Scan-Ports (`Purger`, `LayerScanner`) in `contextra-ports`: Datenhalter bieten rein mechanische Bereinigungsmethoden an, ohne Kenntnis von Sagas, DSGVO-Regelwerken oder Berichtsformaten. |
| 2.3 | Core Storage & Index Ports: `StorageEngine`, `VectorIndex`, `TextIndex`, `GraphIndex` bereinigen; Entkopplung von monolithischen Ausführungsannahmen; native async-Methoden bzw. sauber getrennte synchrone Kerne. |
| 2.4 | Egress- und Security-Ports: Neuer Port `EgressPolicy` in `ports`, damit z. B. `contextra-router` ohne direkte Bindung an `contextra-privacy` agieren kann. |
| 2.5 | Contract-Tests (`ports/contract_tests.rs`) aktualisieren und auf alle Ports erweitern. |

**Gate G2:** Contract-Tests grün; Ports v2 in `contextra-ports` abgenommen; keine Lecks interner Crate-Typen in Port-Signaturen.

## 6. Phase 3 – Löschkette entkoppeln & in `contextra-erasure` modularisieren

**Ziel:** Vollständige Entfernung des DSGVO-, Löschketten-, Nachweis- und Saga-Ballasts aus dem Speicherkern. Kapselung der gesamten Multi-Layer-Löschlogik in `contextra-erasure`.

| Schritt | Crate | Entkopplung / Modularisierung | Größe |
| --- | --- | --- | --- |
| 3.1 | `contextra-crypto` | Aufspaltung: `contextra-crypto` behält ausschließlich krypto-technische Grundbausteine (`wal_crypto`, `kdf`, `cipher`, `audit_chain`, `anti_tamper`). Alle Lösch- und Nachweiskomponenten (`deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs`) wandern vollständig in `contextra-erasure`. | L |
| 3.2 | `contextra-store` | Löschketten- und DSGVO-Relikte entfernen: `compaction/engine.rs` und `adaptive.rs` entflechten; `kv/segment.rs` und `kv/delete_mode.rs` von `kv_shredding` entkoppeln; Store implementiert nur `Purger`/`LayerScanner` für `contextra-erasure`. | XL |
| 3.3 | `contextra-vector` | DSGVO- und Deletion-Proof-Begriffe aus `hnsw/deletion.rs`, `DiskAnn::delete`, `VectorIndex` entfernen; `VectorScanner`/`Purger` implementieren; Entkopplung von `crypto::wal_crypto`. | L |
| 3.4 | `contextra-text` | Bereinigung von 17 Dateien mit Löschketten-Bezug; `TextScanner`/`Purger` implementieren; Index-Bereinigung als autarker Baustein. | M |
| 3.5 | `contextra-engine` | `contextra_impl/erasure.rs` vollständig aus der Engine entfernen und in `contextra-erasure` überführen; Background-Workers und Decay-Controller entkoppeln; Engine orchestriert Backends nur noch über Ports. | L |
| 3.6 | `contextra-db`, `-agent`, `-cognition` | Entfernen monolithischer Löschkaskaden; DB fungiert als saubere Kompositionsreferenz ohne direkte Bindung an Löschketten-Interna. | M |
| 3.7 | `contextra-mcp` | Löschwerkzeuge und Erasure-spezifische IPC-Handler herauslösen; Werkzeuge greifen bei Bedarf über den modularen `contextra-erasure`-Baustein zu; Lizenz-Gates entkoppeln. | M |
| 3.8 | `contextra` (Fassade) | DSGVO- und Erasure-APIs (`erase`, `erase_status`, `report`) herauslösen oder als optionale Fassaden-Brücke zu `contextra-erasure` bereitstellen. | S |
| 3.9 | `contextra-erasure` | Finalisierung des eigenständigen Löschkern-Bausteins: Kapselt Deletion Saga, Vault (16 Shards), GateLog, Ledger, Restore-Guard, Witness-Erfassung und kryptografisch signierte Deletion-Berichte. `contextra-erasure` steuert die Bereinigung der Datenhalter ausschließlich über `Purger` und `LayerScanner`. | L |

**Gate G3:** Alle 664 verstreuten DSGVO-/Löschketten-Fundstellen außerhalb von `contextra-erasure` bereinigt; Kern-Crates (`store`, `vector`, `text`, `crypto`, `engine`) sind frei von Deletion-Proof- und Rechtslogik; `contextra-erasure` baut und testet autark als eigenständiger Baustein.

## 7. Phase 4 – Kerne entkoppeln: Append-friendly Store, Vector, Text, Graph

**Ziel:** Autarke, hochperformante Speichermodule mit klarer Supersession- und Append-Semantik.

| WP | Crate | Arbeit |
| --- | --- | --- |
| 4.1 | `store` | LSM-Storage auf modularen Betrieb ausrichten: Compaction als entkoppelter Level-Merge; saubere Block-Cache-Abstraktion (`PinnedBlock`); allokationsfreier Lesepfad via `with_value`. |
| 4.2 | `sys` | `mmap`-Abstraktion mit versiegelten Speicherbereichen (`SealedMap`), robuster Signal- und Fehlerbehandlung ohne Panics. |
| 4.3 | `graph` | CSR-Graph vollständig entkoppeln: Delta-Puffer und lock-freier Lesepfad; `adapt`-Querabhängigkeit eliminieren; Dev-Abhängigkeit zu `store` entfernen. |
| 4.4 | `vector` | HNSW-Index isolieren: Supersession-Bitmap (`RoaringBitmap`), Entkopplung von Dateisystem-Direktaufrufen via `Vfs`-Port. |
| 4.5 | `text` | Inverted-Index autark machen: BM25/BM25F mit Block-Max-WAND; Datentyp `Language` nach `types` verlagern; keine Querkopplungen zu höheren Ringen. |
| 4.6 | `checkpoint` | Backup- und Snapshot-Handling isolieren, Bereinigung von Legacy-Löschmarken, autarker Wiederanlauf-Test. |

**Gate G4:** Alle Ring-1-Crates (`store`, `vector`, `text`, `graph`, `checkpoint`) bauen unabhängig; Benchmarks p50/p99 mindestens auf Baseline-Niveau; keine direkten `std::fs`-Aufrufe im Index-Kern (vollständig über `Vfs`).

## 8. Phase 5 – Feingranulare Modularisierung, Abstraktion & Zero-Copy-Härtung

| WP | Arbeit |
| --- | --- |
| 5.1 | Backend-Bündel in `contextra-engine`: Einführung von `trait Backends` (Storage, Vector, Text, Graph); Engine wird vollständig generisch und verzichtet auf feste Verdrahtung konkreter Index-Typen. |
| 5.2 | Dynamische Abstraktion prüfen (`dyn` vs. Generics): Heißpfade (Storage-Read, Vektor-Scan) monomorphisieren, Kaltpfade (Sagas, Reporting, Worker) schlank über `dyn` halten. |
| 5.3 | Quer-Kanten restlos schneiden: `graph` → `adapt`, `vector` → `mvcc`/`crypto`, `mcp` → Inferenz-Backends via Features entkoppeln. |
| 5.4 | `contextra-ingest` autark machen: Abhängigkeit zu `engine` durch Extraktion von `contextra-chunker` auflösen. |
| 5.5 | `contextra-core` (tote Kompatibilitätsfassade) vollständig dekommissionieren. |
| 5.6 | Unsafe-Inseln auditieren: `forbid(unsafe_code)` in allen Crates bis auf die 3 genehmigten Ausnahmen (`simd`, `sys`, `wire`). |

**Gate G5:** Keine verbotenen Querabhängigkeiten; Engine vollständig generisch über `Backends`-Trait; `ingest` baut eigenständig ohne `engine`.

## 9. Phase 6 – Produktschnitte, Baustein-Export & isolierte Beispiele

| WP | Arbeit |
| --- | --- |
| 6.1 | Erstellung isolierter Baustein-Beispiele unter `examples/`: <br>- `example-store`: Nutzung von `contextra-store` als autarker Key-Value-Speicher.<br>- `example-vector`: Autarker HNSW-Vektorindex mit `contextra-vector`.<br>- `example-text`: BM25-Volltextsuche mit `contextra-text`.<br>- `example-erasure`: Anbindung des modularen `contextra-erasure`-Bausteins an eigene Datenstrukturen.<br>- `example-rag`: Schlanke RAG-Pipeline mit entkoppelten Bausteinen. |
| 6.2 | CI-Job `building-blocks-matrix`: Validiert, dass jeder Crate isoliert in einem sauberen Verzeichnis ohne Workspace-Monolith baut (`cargo build -p <crate> --no-default-features`). |
| 6.3 | SemVer- und Versionierungs-Entkopplung: Crates erhalten eigenständige Versionslinien, um unabhängiges Einbinden in Drittprojekten zu ermöglichen. |

**Gate G6:** Alle Baustein-Beispiele bauen und laufen isoliert; Building-Blocks-CI-Matrix grün.

## 10. Phase 7 – Dokumentation, Governance-Bereinigung und CI-Absicherung

- `docs/SPEC.md` als maßgebliche Baukasten-Spezifikation v3.0 führen; Veraltete Monolith-Zusagen in `docs/` archivieren.
- `docs/ARCHITECTURE.md` auf den Baukasten-Zustand aktualisieren.
- `CHANGELOG.md`: Dokumentation der Entkopplung und Ausgliederung der Löschkette in `contextra-erasure`.
- `capabilities.toml`, `deny.toml` und Quality Gates final auf den modularen Zustand arretieren.

**Gate G7:** Vollständige Dokumentationskonsistenz; keine irreführenden DSGVO- oder Löschzusagen im Speicherkern; saubere Baukasten-Dokumentation.

## 11. Risikoregister

| ID | Risiko | Wirkung | Gegenmaßnahme |
| --- | --- | --- | --- |
| R1 | Baseline rot (Ist-Stand baut nicht fehlerfrei) | Verzögerung bei Phase 0 | G0 ist hartes Gate; Baseline-Fehler vor Umbauten isolieren und beheben |
| R2 | Semantikverlust bei Entflechtung von Update/Supersession und Erasure | Regressionen im Retrieval oder bei Dokument-Ersetzungen | Charakterisierungstests in Phase 2 vor Beginn der Schnittarbeiten |
| R3 | Unzureichende Port-Abstraktion bricht Fremdnutzung | Baustein bleibt implizit verdrahtet | Spike-Implementierung in Phase 2 und Contract-Tests |
| R4 | Erhöhte Kompilierzeit durch Generics in `engine` | Längere CI-Zyklen | Pragmatische Aufteilung: Heißpfade generisch, Kaltpfade über dyn Trait-Objekte |
| R5 | Drittnutzer benötigen Löschfunktionalität | DSGVO-Anforderungen in Kundenprojekten | `contextra-erasure` ist als fertiger, modularer Baustein direkt zuschaltbar |

## 12. Definition of Done (gesamt)

1. Gates G0 bis G7 bestanden und in CI automatisiert überwacht.
2. Jeder Kern-Crate (`store`, `vector`, `text`, `graph`, `crypto`, `mvcc`) baut und testet autark und kann isoliert in Drittprojekte eingebunden werden.
3. Sämtliche DSGVO-, Löschketten-, Saga- und Audit-Logiken sind vollständig und exklusiv in `contextra-erasure` gekapselt.
4. Öffentliche Schnittstellen lecken keine internen Fremdtypen (Modularitäts-Score >= 80 für alle Kernbausteine).
5. Isolierte Produkt- und Baustein-Beispiele in `examples/` beweisen die universelle Verwendbarkeit.

---

# Contextra Baukasten – Implementierungsspezifikation

Stand 06.11.2026 · Basis `tfufuz1/contextra` · Refaktorisierung zur modularen Bausteinarchitektur

## 0. Entscheidung und Architektur-Transformation

**Entscheidung:** Das Contextra-Ökosystem wird von einem monolithisch verknüpften Spezialsystem zu einem **hochgradig modularen Baukastensystem (Building Blocks)** umgebaut.
- **Entkopplung aller Crates:** Jeder Crate wird als isolierter, in Fremdprojekten wiederverwendbarer Baustein realisiert. Direkte Abhängigkeiten zwischen den Modulen werden durch generische Typparameter oder Traits in `contextra-ports` ersetzt.
- **Vollständige Kapselung der Löschkette in `contextra-erasure`:** Die Multi-Layer-Löschkette mit Deletion Proofs, AuditChains, Gesetzesbezügen (DSGVO Art. 17 etc.) und der Löschsaga wird vollständig aus den Speicher- und Index-Crates (`store`, `vector`, `text`, `crypto`, `engine`, `db`, `mcp`) entfernt. Die gesamte Nachweis- und Löschorchestrierung wird in `contextra-erasure` gekapselt. Datenhaltende Crates implementieren ausschließlich minimale, neutrale Bereinigungs-Ports (`Purger`, `LayerScanner`).

## 1. Status der technischen Befunde

| ID | Befund | Analyse & Korrekturmaßnahme | Phase |
| --- | --- | --- | --- |
| D1 | `xtask`-Integration | Workspace-Gates konsolidieren; sicherstellen, dass DAG- und Port-Checks in CI automatisiert greifen. | Phase 1 |
| D2, D3 | Optionale Feature-Abhängigkeiten | Features sauber deklarieren; keine impliziten Schwergewichte in Default-Features. | Phase 1 |
| D4 | Direkte I/O-Aufrufe in Indizes | Dateizugriffe in `vector` und `store` auf den `Vfs`-Port umstellen zur Gewährleistung von Sandbox- und In-Memory-Fähigkeit. | Phase 4 |
| D5 | Ignorierte Tests (`#[ignore]`) | Auditieren, beheben oder dokumentieren; keine unklaren Lücken in der Testabdeckung. | Phase 0 |
| D6 | `durable-fs` & `capabilities.toml` | Alle Crates inkl. `contextra-durable-fs` und `contextra-erasure` mit klar definierten Ring- und `may_depend_on`-Vorgaben erfassen. | Phase 1 |
| D7 | Vfs-Konsolidierung | Vereinheitlichung der zwei historischen Vfs-Konzepte in `contextra-ports::Vfs` mit Streaming/Handle-Support. | Phase 2 |
| D8 | Layering-Validierung | Statische Ring- und Schichtentests zur Durchsetzung der strikten Abwärtsabhängigkeiten. | Phase 1 |
| D9 | Tote Abhängigkeiten | Unnötige Crate-Abhängigkeiten bereinigen (u. a. `crypto` in `db`, `sys` in `engine`). | Phase 5 |
| D10 | Unsafe-Code-Disziplin | Strikte Beschränkung von `unsafe` auf `simd`, `sys` und `wire`; `#![forbid(unsafe_code)]` in allen anderen 32 Crates. | Phase 5 |
| D11 | Dekommissionierung `contextra-core` | Legacy-Fassade auflösen und entfernen. | Phase 5 |

## 2. Baukasten-Kopplungsregeln & Entkopplungsziel

Die statische Analyse (siehe `docs/MODULAR_DECOUPLING_AUDIT.md`) zeigte 664 verstreute Löschketten-/DSGVO-Relikte und 307 fehlende Port-Abstraktionen. Ziel ist das Erreichen eines Modularitäts-Scores von > 80 für alle Kernbausteine.

**Entkopplungs-Maßnahmen je Kernkomponente:**
- **`contextra-store`:** Entfernen von `kv_shredding`, Deletion-Proof- und DSGVO-Referenzen. Fokus auf schnelles LSM, Append-Only, Block-Cache und WAL-Durability. Bereitstellung von `Purger`- und `LayerScanner`-Implementierungen.
- **`contextra-vector`:** Entfernen von `wal_crypto::WalHmac` und Deletion-Proof-Logiken. Eigenständige HNSW- und Vektor-Primitive.
- **`contextra-text`:** Vollständig entkoppelter BM25-Index; `Language`-Typ nach `contextra-types` auslagern.
- **`contextra-graph`:** Lock-freier Lesepfad, CSR-Segmente; Entfernen harter Abhängigkeiten zu `store` und `adapt`.
- **`contextra-crypto`:** Aufspaltung: Reine kryptografische Primitive (AEAD, KDF, Cipher, WAL-HMAC, Anti-Tamper) verbleiben in `contextra-crypto`. Alle Deletion-Proof-Strukturen (`deletion_proof/`, `ed25519_proof.rs`, `kv_shredding.rs`, `revocation_log.rs`) werden in `contextra-erasure` modularisiert.
- **`contextra-engine`:** Umbau von statischer Bindung auf das generische `Backends`-Bündel. Beseitigung monolithischer Löschkaskaden (`contextra_impl/erasure.rs` wandert nach `contextra-erasure`).
- **`contextra-erasure`:** Der zentrale, modulare Baustein für erweiterte Lösch-, Audit- und DSGVO-Anforderungen. Kapselt Saga-Orchestrierung, Vault, GateLog, Restore-Guard und Signierung.

## 3. Der Baukasten-Vertrag (Building Block Rules)

Jeder Baustein erfüllt folgende verbindliche Architekturregeln:

1. **M1 Strikte Abwärtsrichtung:** Abhängigkeiten zeigen ausschließlich auf niedrigere Ringe (`capabilities.toml`).
2. **M2 Autarke Baubarkeit:** Jeder Baustein lässt sich via `cargo build -p <crate> --no-default-features` isoliert bauen und testen.
3. **M3 Schnittstellenreinheit:** Öffentliche Signaturen nutzen ausschließlich Typen aus `types`, `ports` oder dem Crate selbst; keine internen Implementierungstypen fremder Crates.
4. **M4 Invertierte Umweltabhängigkeiten:** Umweltinteraktionen laufen über standardisierte Ports: I/O über `Vfs`, Zeit über `Clock`, Entropie über `Rng`, Identifikatoren über `IdGen`, Metriken über `MetricsSink`.
5. **M5 Schlanke Default-Features:** Additive Feature-Struktur; externe Runtimes und Frameworks (`onnx`, `candle`, `wasmtime`, `tokio`) sind niemals in Standard-Minimal-Builds erzwungen.
6. **M6 Isolation von Test-Abhängigkeiten:** Keine Dev-Abhängigkeiten auf höhere Architekturschichten.
7. **M7 Entkoppelte Bereinigung:** Datenhalter implementieren rein funktionale Bereinigung (`Purger`, `LayerScanner`). Kaskadierung, Nachweiserzeugung, DSGVO-Reporting und Auditierung liegen exklusiv in `contextra-erasure`.
8. **M8 Asynchrone Neutralität im Fundament:** Fundamentbausteine (Ring 0) und Speicherkerne halten ihre Kernlogik synchron und frei von festen Async-Runtime-Zwängen.
9. **M9 Isolierte Unsafe-Inseln:** Unsafe ist verboten (`forbid(unsafe_code)`), außer in den drei explizit deklarierten Low-Level-Inseln (`simd`, `sys`, `wire`), jeweils mit formalem `// SAFETY:`-Nachweis.
10. **M10 SemVer-Autonomie:** Jeder Baustein ist unabhängig versionierbar.
11. **M11 Integrations-Dokumentation:** Jeder Crate verfügt über eine README mit Entkopplungsprofil, Schnittstellenbeschreibung und Minimalbeispiel für den Einsatz in Fremdprojekten.

## 4. Modulkatalog des Baukastensystems (Zielzustand)

| Modul | Ring | Rolle im Baukastensystem | Ziel-Abhängigkeiten (intern) | Entkopplungs-Status |
| --- | --- | --- | --- | --- |
| `contextra-types` | 0 | Kanonische Grundtypen, IDs, Fehlerklassen | – | Autark |
| `contextra-ports` | 0 | Zentrale Port- und Trait-Definitionen | `types` | Konsolidiert |
| `contextra-sys` | 0 | Unsafe-Betriebssystemprimitive, mmap, Memory-Lock | – | Autarke Insel |
| `contextra-simd` | 0 | SIMD-Vektorbeschleunigung (AVX2, AVX-512, NEON) | `types` | Autarke Insel |
| `contextra-wire` | 0 | Zero-Copy-Serialisierung & FlatBuffers | – | Autarke Insel |
| `contextra-durable-fs` | 0 | Fsync-, atomic-rename- und Flush-Dateisystemprimitive | `sys`, `ports` | Konsolidiert mit Vfs |
| `contextra-mvcc` | 0 | Multi-Version Concurrency Control & SSI | `ports`, `types` | Autark |
| `contextra-crypto` | 0 | Krypto-Kern: AEAD, KDF, WAL-HMAC, Anti-Tamper | `ports`, `types` | Bereinigt um Erasure |
| `contextra-rank` | 0 | Rangfusion (RRF) & Score-Kalibrierung | `ports`, `types` | Autark |
| `contextra-adapt` | 0 | Adaptive Algorithmen (Bandits, Drift-Watcher) | `ports`, `types` | Autark |
| `contextra-store` | 1 | LSM-Tree Storage Engine & Write-Ahead Log | `ports`, `types`, `mvcc`, `crypto`, `sys`, `durable-fs` | Bereinigt um DSGVO/Erasure |
| `contextra-vector` | 1 | HNSW-Vektorindex & Vektorsuche | `ports`, `types`, `mvcc`, `simd`, `sys` | Bereinigt um DSGVO/Erasure |
| `contextra-text` | 1 | BM25/BM25F-Volltextindex | `ports`, `types` | Bereinigt um DSGVO/Erasure |
| `contextra-graph` | 1 | CSR-Graphindex für semantische Netze | `ports`, `types` | Entkoppelt von Store/Adapt |
| `contextra-checkpoint` | 1 | Point-in-Time-Snapshots & Restore-Guard | `ports`, `types`, `mvcc` | Autark |
| `contextra-kvcache` | 1 | Verschlüsselter KV-Cache für LLM-Zustände | `ports`, `types`, `crypto` | Autark |
| `contextra-infer-onnx` | 2 | ONNX-Embedding- & Reranking-Adapter | `ports`, `types`, `rank` | Autark |
| `contextra-infer-candle` | 2 | Candle/GGUF-Embedding-Adapter | `ports`, `types`, `rank` | Autark |
| `contextra-infer-ollama` | 2 | Ollama-Netzwerkadapter für Embeddings | `ports`, `types` | Autark |
| `contextra-sandbox` | 2 | WASM-Sandbox (Wasmtime) für sichere Ausführung | – | Autark |
| `contextra-engine` | 3 | Generische Orchestrierung über `Backends`-Trait | `ports`, `types` | Entkoppelt von konkreten Indizes |
| `contextra-erasure` | 3 | **Modularer Löschkern:** Saga, Vault, GateLog, Nachweise, DSGVO-Reporting | `ports`, `types`, `crypto` | Vollständig modularisiert |
| `contextra-cognition` | 3 | Konsolidierung & Langzeit-Gedächtnis | `engine`, `ports`, `types` | Basiert auf Ports |
| `contextra-router` | 3 | Adaptives Query-Routing | `ports`, `types`, `wire`, `adapt` | Nutzt `EgressPolicy`-Port |
| `contextra-privacy` | 3 | PII-Redaktion & Datenschutzfilter | `ports`, `types` | Implementiert `EgressPolicy` |
| `contextra-ingest` | 3 | Dokumenten-Parsing & Text-Extraktion | `types` (via `contextra-chunker`) | Entkoppelt von Engine |
| `contextra-agent` | 3 | Workflow- & Graph-Agenten | `engine`, `ports`, `types` | Komposition |
| `contextra-db` | 3 | Referenz-Komposition für Single-Node-DB | diverse Bausteine via Ports | Referenzkomposition |
| `contextra` | 4 | Öffentliche Hauptfassade (High-Level-API) | Bausteine nach Bedarf | Schlanke Fassade |
| `contextra-mcp` | 4 | Model Context Protocol Server (stdio) | `contextra`, `types`, `ports` | Autarke Oberfläche |
| `contextra-py` | 4 | Python-Bindings (PyO3) | `types`, `ports`, `contextra` | Autarke Oberfläche |

## 5. Detaillierte Schnittstellen-Spezifikation für die Entkopplung

### 5.1 Generisches Backend-Bündel in `contextra-engine`

Um zu verhindern, dass `contextra-engine` starre Abhängigkeiten auf konkrete Speicher- und Index-Crates erzwingt, wird das Backend-Bündel eingeführt:

```rust
pub trait Backends: Send + Sync + 'static {
    type Storage: StorageEngine;
    type Vector: VectorIndex;
    type Text: TextIndex;
    type Graph: GraphIndex;

    fn storage(&self) -> &Self::Storage;
    fn vector(&self) -> &Self::Vector;
    fn text(&self) -> &Self::Text;
    fn graph(&self) -> &Self::Graph;
    fn purgers(&self) -> Vec<Arc<dyn Purger>>;
}

pub struct Collection<B: Backends = DefaultBackends> {
    backends: Arc<B>,
    // Weitere generische Komponenten...
}
```

Die Standard-Backends (`LsmStorage`, `HnswIndex`, `InvertedIndex`, `CsrGraph`) liegen hinter dem Feature `default-backends`. Andere Projekte können eigene Speicher- und Index-Kombinationen ohne `contextra-store` oder ohne Vektorindex instanziieren.

### 5.2 Konsolidierter Vfs-Port (`contextra-ports`)

Alle Subsysteme abstrahieren Dateisystemzugriffe über einen einzigen Port:

```rust
#[async_trait::async_trait]
pub trait Vfs: Send + Sync + 'static {
    async fn open(&self, path: &Path, opts: OpenOptions) -> Result<Box<dyn VfsFile>, VfsError>;
    async fn read(&self, path: &Path) -> Result<Vec<u8>, VfsError>;
    async fn write_atomic(&self, path: &Path, data: &[u8]) -> Result<(), VfsError>;
    async fn remove(&self, path: &Path) -> Result<(), VfsError>;
    async fn sync_dir(&self, path: &Path) -> Result<(), VfsError>;
}

#[async_trait::async_trait]
pub trait VfsFile: Send + Sync + 'static {
    async fn read_at(&self, buf: &mut [u8], offset: u64) -> Result<usize, VfsError>;
    async fn write_at(&self, buf: &[u8], offset: u64) -> Result<usize, VfsError>;
    async fn sync_data(&self) -> Result<(), VfsError>;
    async fn sync_all(&self) -> Result<(), VfsError>;
}
```

Damit entfällt die Kopplung an das Betriebssystem-Dateisystem; Speicher- und Indexbausteine sind in reinen In-Memory- oder WASM-Umgebungen lauffähig.

### 5.3 Neutrale Bereinigungs- und Scan-Ports (`Purger`, `LayerScanner`)

Speicher- und Index-Crates kennen weder DSGVO noch Nachweisketten. Sie bieten lediglich neutrale Schnittstellen an:

```rust
pub trait Purger: Send + Sync + 'static {
    fn purge_scope(&self, scope: &KeyScope) -> Result<PurgeOutcome, PurgeError>;
}

pub trait LayerScanner: Send + Sync + 'static {
    fn scan_for_residual(&self, scope: &KeyScope) -> Result<ScanOutcome, ScanError>;
}
```

`contextra-erasure` konsumiert diese Traits, um Sagas auszuführen, Restbestände zu prüfen und kryptografische Deletion-Proof-Zertifikate auszustellen.

### 5.4 Modularisierung von `contextra-erasure`

`contextra-erasure` bündelt:
- **`FileVault`:** 16 Shards, Verwahrung von Epochen- und Scope-Schlüsseln, physisches Schreddern von DEKs.
- **`ReadGate` & `GateLog`:** Sofortige Lesesperren über RCU-Intervalle (`ErasedSet`).
- **`ErasureJournal` & `DeletionSaga`:** Zustandssaga (`Requested -> Gated -> KeyDestroyed -> Purging -> Measured -> Completed`).
- **`Scanner` & `Witness`:** Positive Canary-Kontrolle und Messung verbliebener Restdaten.
- **`DeletionReport`:** Ed25519-signierte Nachweise mit kanonischem JSON-Export und vollständiger DSGVO-Nachweisführung.
- **`RestoreGuard`:** Schutz gegen unbemerktes Wiederherstellen gelöschter Daten aus Alt-Backups.

Damit ist die Löschkette ein hochspezialisierter, separat lizenzierbarer oder einspeisbarer Baustein, während die Kerne leichtgewichtig und maximal performant bleiben.
