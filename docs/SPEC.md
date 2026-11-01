# Contextra v1 – Spezifikation (SPEC)

| Feld | Wert |
|---|---|
| Spezifikationsversion | 2.0.0 (SOLL-Zustand für Release 1.0.0) |
| Stand | 2026-11-01 |
| Status | Final. Alle Entscheidungen sind getroffen, es bestehen keine offenen Fragen. |
| Basis der statischen Analyse | `github.com/tfufuz1/contextra`, `main` @ `722d45aa8998142ede259d12fa4414823dd5f454` |
| Prüfmodus | Rein statisch: Quelltext gelesen, Manifeste ausgewertet, Muster gezählt. **Nichts gebaut, nichts ausgeführt.** |
| Geltung | Einzige normative Beschreibung des Zielzustands. Jede Abweichung des Quellcodes von diesem Dokument ist ein Defekt des Quellcodes. |
| Begleitdokument | `PLAN.md` (Arbeitspakete, Gates, Reihenfolge). Beide Dokumente sind in sich vollständig. |

> **Keine Rechtsberatung.** Kein Softwareprodukt kann „100 % Rechtskonformität“ zusagen, denn Rechtskonformität ist ein Merkmal des Betriebs durch den Verantwortlichen, nicht des Codes. Dieses Dokument legt deshalb fest, was Contextra technisch maximal leistet, wie jede Leistung gemessen wird, und was Contextra wahrheitsgemäß behaupten darf (Abschnitt 6). Wo eine Rechtsfrage die Technik berührt, ist sie in Abschnitt 6.6 **konservativ entschieden**: Es gilt immer die strengste Auslegung. Eine spätere juristische Prüfung kann dadurch nur Aussagen lockern, nie technische Maßnahmen nachfordern.

---

## 0. Konventionen

- **MUSS / DARF NICHT / SOLL** gelten im Sinne von RFC 2119.
- **IDs:** `P` Prinzip, `D` Entscheidung, `HR` Harte Regel, `R` Rechtsanforderung, `E` Rechtsentscheidung, `I` Invariante, `T` Test, `B` Benchmark, `FC` Fehlerklasse, `FI` Funktionsinventar, `L` Schicht (Layer).
- **Evidenzmarken:** **[V]** am Klon `722d45aa8` statisch geprüft (Datei, Zeile oder Zählung). **[E]** Entscheidung, Rechnung oder Zielwert dieser Spezifikation. Zielwerte sind erst mit einem Benchmarklauf (Abschnitt 15) belegt und bis dahin Gate-Schwellen, keine Messungen.
- `grep`-Zählungen enthalten Inline-Testmodule und sind **Obergrenzen** für Produktionscode.
- Alle Byte-Layouts sind big-endian, sofern nicht anders angegeben. Alle Zeitangaben sind UTC aus dem Port `Clock`.

---

## 1. Produkt

### 1.1 Mission

Contextra ist eine **eingebettete Rust-Bibliothek** (Single-Node, kein Server) zur lokalen Speicherung von Wissen und Gedächtnis für Agenten- und RAG-Systeme: Dokumente (Textchunks, Metadaten, Zuordnung zu einer betroffenen Person), Volltextsuche (BM25), Vektorsuche (HNSW), Hybridsuche (RRF) und Zeitreisen über MVCC-Versionen.

**Alleinstellungsmerkmal:** Löschen ist ein **eigener, entkoppelter, gemessener Prozess**. Ein Betreiber ruft `erase(Ziel)` auf und erhält einen signierten, von einem unabhängigen Programm prüfbaren Bericht, der je Schicht ausweist, was entfernt, was gemessen und was nicht erfasst wurde. Alles andere ist Dienst am Löschen: Keine Funktion DARF Daten an einem Ort ablegen, den der Löschkern nicht erreicht.

### 1.2 Zielnutzer

Entwickler und Betreiber von Agenten- und RAG-Systemen im EU-Raum, die Betroffenenrechte (Art. 17 und 18 DSGVO) technisch umsetzen müssen und einen prüfbaren Nachweis brauchen.

### 1.3 Produktstufen (jede einzeln lieferbar)

| Stufe | Inhalt | Lieferbares Ergebnis |
|---|---|---|
| **S0** | Gehärteter LSM-Speicher mit Löschcluster-Schlüsselraum, Pflichtverschlüsselung, Tresor, Gate, Saga, Purge, Journal, Bericht, Lab, Verifier. Dokumente, Zeitreisen, Mandanten. | Verschlüsselter Dokumentenspeicher mit belegbarer Löschung. Vollwertiges Produkt (Agentengedächtnis ohne Suche). |
| S1 | Textsuche als abgeleiteter, segmentierter, verschlüsselter Index | Volltextsuche mit denselben Nachweisen |
| S2 | Vektorsuche (HNSW) mit Slot-Zeroing, verschlüsseltem Snapshot, Batch-Rebuild | RAG-Kern |
| S3 | Hybridsuche (RRF), API-Politur, Dokumentation, Release-Candidate | v1.0.0 |

Reihenfolge-Begründung: S0 beweist das Alleinstellungsmerkmal auf kleinstem Umfang. Jede weitere Stufe fügt eine Löschschicht und neue Fehlerquellen hinzu. Scheitert eine spätere Stufe, bleibt S0 ein vollwertiges Produkt.

### 1.4 Nicht-Ziele v1

Netzwerkdienst, Server-Socket, Sharding, Replikation, P2P; Erzeugung von Embeddings und LLM-Aufrufe (Vektoren und Text kommen vom Aufrufer); Dateiformat-Parsing; Graph und Relationen; Checkpoints, Forks, Hardlink-Klone; persistente Caches; automatische Entitäts- oder Metadatenextraktion; Lizenz- und DRM-Schicht; MCP-Server und Python-Bindings; Mehr-Betroffenen-Dokumente (ein Dokument gehört zu höchstens einem Betroffenen); Schlüsselrotation; Telemetrie; Schlüssel-Escrow; Löschung außerhalb von Contextra (Backups, Kopien beim Betreiber, Hardware-Ebene).

### 1.5 Prüfbare Zusagen (Leistungsbeschreibung, Anhang B)

1. Ein Löschauftrag wirkt ausschließlich auf sein typisiertes Ziel; alle Nicht-Ziel-Daten bleiben bitgleich erhalten (T-ISO-1).
2. Ab persistierter Sperre liefert jeder Lesepfad `Erased`; Zeitreisen eingeschlossen (T-VER-1).
3. Für jedes Ziel wird der dokumentspezifische Schlüssel aus dem Tresor entfernt und das Fehlen geprüft (T-VAULT-1..4).
4. Nach abgeschlossener Löschung enthält kein Datenverzeichnis-File, kein Index und kein von Contextra kontrollierter Speicher einen gemessenen Rest des Ziels (T-RAW-1..5, Messklassen E1/E2/E3).
5. Unterbrechungen (Absturz, I/O-Fehler, Abbruch des Aufrufers) führen nach Neustart zur Vollendung oder zu sichtbarem Status `Pending`. Stille Teilzustände existieren nicht (T-CRASH-1..4, T-CANCEL-1).
6. Der Bericht nennt, was gemessen wurde, und listet alle zutreffenden Restrisiken (T-REPORT-1, T-PROOF-1).

### 1.6 Zusagen, die Contextra nicht macht

„DSGVO-konform“, „rechtskonform“, „garantiert gelöscht“, „unwiederbringlich“, Löschung in Backups, auf Flash-Ebene, in Dateisystem-Journalen, in Swap oder bei externen Verarbeitern (Abschnitt 6.4).

---

## 2. Leitprinzipien (normativ)

| ID | Prinzip | Folge |
|---|---|---|
| P1 | **Wirkung vor Behauptung** | Ein Status oder Zähler „gelöscht“ entsteht nur aus einem Zeugen (`LayerWitness`) eines Scanners. |
| P2 | **Fail-closed** | Unbekannter Zustand, korrupte Daten, abgelaufene Fristen: Verweigerung oder `Pending`, nie Weiterlaufen. |
| P3 | **Ein Weg** | Ein Löschpfad, ein Format, eine Fehlerbehandlung je Konzept. Keine Legacy-Lesepfade. |
| P4 | **Ziel-Isolation** | Jede mutierende Operation hat ein typisiertes Ziel. Nachbardaten sind bitgleich unverändert (messbar). |
| P5 | **Keine Schalter für Sicherheit** | Kein `cfg(feature)` auf Verschlüsselung, Integrität, Löschmessung, Mandantenprüfung. |
| P6 | **Keine Panics, keine verschluckten Fehler** | Im Produktionscode kein `unwrap`, `expect`, `panic!`, `unreachable!`, `let _ =` auf `Result`, kein `unwrap_or_default` auf I/O. |
| P7 | **Determinismus über Ports** | Zeit (`Clock`), Zufall (`Rng`), Dateisystem (`Vfs`), IDs (`IdGen`). Ausnahme: Schlüssel-, Salt- und Nonce-Material aus dem OS-CSPRNG. |
| P8 | **Kleinster Kern** | Ein Crate oder Modul ist nur im Kern, wenn ein Kernkonsument es braucht (`cargo tree`-belegt). |
| P9 | **Messen vor Optimieren** | Jedes Leistungsgate hat eine Basislinie auf dokumentierter Hardware. Ohne Basislinie kein Gate. |
| P10 | **Verifizierbarkeit vor Eleganz** | Jede Entscheidung nennt den Test, der sie belegt. Ohne Test keine Entscheidung. |
| P11 | **Löschbarkeit als Strukturprinzip** | Der Schlüsselraum, das Dateiformat und die Indexe sind so gebaut, dass jedes Löschziel einen zusammenhängenden, kleinen Bereich belegt. Löschaufwand skaliert mit dem Ziel, nicht mit der Datenbankgröße. |
| P12 | **Entkopplung** | Der Löschkern kennt nur Traits aus `ports`. Speicher, Textindex und Vektorindex implementieren `ErasureLayer`. Ein Austausch einer Schicht ändert den Löschkern nicht. |
| P13 | **Datensparsamkeit im Nachweis** | Journal, Bericht und Ledger enthalten nur Commitments, Zähler und Zeiten, nie Inhalte, Schlüssel oder Klartext-Kennungen. |

---

## 3. Ausgangszustand am Klon (statisch verifiziert)

Dieser Abschnitt beschreibt das Delta zwischen Ist und Soll. Er ist Arbeitsgrundlage der Umsetzung, nicht Teil des Produkts.

### 3.1 Stand des Repositories

Der Klon `722d45aa8` ist ein **halb ausgeführter Rückbau**. Die Verzeichnisse von 21 Crates sind gelöscht, die Manifeste und Quelltexte zeigen aber weiter auf sie.

| ID | Befund [V] | Beleg | Soll-Maßnahme |
|---|---|---|---|
| B-01 | Workspace-`members` listet 37 Einträge, davon **21 ohne Verzeichnis** (`wire`, `core`, `checkpoint`, `privacy`, `graph`, `infer-onnx`, `infer-ollama`, `infer-candle`, `mcp`, `agent`, `router`, `sandbox`, `py`, `adapt`, `kvcache`, `cognition`, `audit-export`, `avv-generator`, `license`, `ingest`, `benchmarks/contextra-bench`). `default-members` (27) enthält 12 fehlende. | `Cargo.toml` | Manifest auf 14 Mitglieder reduzieren (Abschnitt 5.1) |
| B-02 | `crypto/src/lib.rs` deklariert `deletion_proof`, `ed25519_proof`, `kv_cipher`, `kv_segment`, `kv_shredding`, `revocation_log`. Die Dateien fehlen. Das Crate kompiliert nicht. | `crypto/src/lib.rs:47–54,64–79`, Verzeichnisliste | Crate auf Zielumfang bringen (8.6) |
| B-03 | `ports/src/lib.rs` deklariert `pub mod scanner`, `checkpoint`, `graph`, `graph_index`, `license`; `scanner` fehlt als Datei. `types/src/lib.rs:60` deklariert `pub mod witness`; Datei fehlt. | `ports/src/lib.rs:33–92`, `types/src/lib.rs:60,70` | Ports und Types auf Zielumfang (8.1, 8.2) |
| B-04 | Pfadabhängigkeiten auf entfernte Crates: `db` (cognition, checkpoint, graph, adapt), `engine` (sandbox, infer-onnx, adapt, checkpoint, graph), `rank` (adapt), `store` Dev-Abhängigkeit (graph, sandbox), `contextra` (license, privacy, audit-export, avv-generator, infer-candle), `store/fuzz` und `db/fuzz` (core). Quelltext der Engine nutzt `contextra_graph` an 103 Stellen. | `crates/*/Cargo.toml`, `grep` | Abhängigkeiten und Aufrufe entfernen (8.12) |
| B-05 | Der alte Löschpfad ist im Store bereits entfernt (`scanner/`, `kv/`, `engine/src/contextra_impl/erasure.rs` fehlen). Es existiert **kein** neuer Löschpfad. | Verzeichnislisten | Neubau (Abschnitte 8.9, 11) |

### 3.2 Sicherheits- und Korrektheitsbefunde im Kern

| ID | Befund [V] | Beleg | Soll |
|---|---|---|---|
| B-10 | **WAL-Integrität abgeschaltet:** `#![allow(unexpected_cfgs)]` hebt das Workspace-`deny` auf; `feature = "wal-integrity"` ist 7-mal in `wal/replay.rs` geprüft und nirgends deklariert. `encryption-at-rest` ist ebenfalls undeklariert (u. a. `wal/tests*`, `tests/no_encryption_feature.rs`). | `store/src/lib.rs:45`, `Cargo.toml:194`, `store/Cargo.toml [features]` | Gates entfernen, Integrität unbedingt (D-05, I-12) |
| B-11 | **Snapshot-Sentinel:** `prune_bound = min_read_snapshot().unwrap_or(u64::MAX).min(min_active_seqno())`; `prune_through` maskiert das Tombstone-Bit und `fetch_max` macht den Wert permanent. Ein Sentinel `u64::MAX` wird als Sequenz verwendet. | `store/src/lsm/ops/compaction.rs:225–231`, `mvcc/src/ssi.rs:640–652` | `Option<Seq>`, kein Maskieren (8.7, T-SEQ-2) |
| B-12 | **`Vfs` im Store: 0 Treffer.** Direkte `tokio::fs`/`std::fs`-Zugriffe im Store: 150 (Obergrenze). Zwei `Vfs`-Traits existieren (`ports/src/vfs.rs:29`, `durable-fs/src/vfs.rs:54`). | `grep` | Ein `Vfs` (D-04) |
| B-13 | **Delete-Race:** `Collection::delete` erzeugt den Lock und gibt ihn sofort frei (`let _guard = …lock_for(id).await; drop(_guard);`). | `engine/src/collection/crud/delete.rs:16–17` | Lock über die ganze Operation (8.12) |
| B-14 | **Verschlüsselung optional:** `SstableBuilder` hält `key_manager: Option<Arc<KeyManager>>`; `create` ruft `create_with_key_manager(path, None)`. `encryption_mode: Option<EncryptionMode>` im Builder. | `store/src/sstable/builder.rs:215,229`, `contextra/src/builder.rs:86` | Pflichtverschlüsselung (D-05) |
| B-15 | **Argon2-Untergrenze** m = 19.456 KiB, t = 2. | `crypto/src/kdf.rs:36,38` | m ≥ 65.536 KiB, t ≥ 3 (D-06) |
| B-16 | **Mandantenpolitik:** `TenantPolicy` hat `#[default] Optional`. | `engine/src/lib.rs:205–210` | nur `Required` (D-07) |
| B-17 | **Automatische Extraktion:** `AutoExtractionMode` existiert mit `Enabled`/`Disabled`. | `types/src/types/domain/misc.rs:576` | entfernen (D-27) |
| B-18 | **Schlüsselcodec ist Text:** `t:{tenant}:{collection}:{doc_type}:{doc_id}` mit Präfixsemantik. | `store/src/tenant_codec.rs` | blinder Cluster-Codec (D-31) |
| B-19 | **`DocId(pub u64)`** (Feature `docid-128` schaltet auf `u128`), unkeyed. | `types/src/types/domain/ids.rs:173,180` | 16-Byte-Tags, keyed (D-08) |
| B-20 | Snapshot- und Arena-Persistenz im Vektorcrate nutzt `std::fs`/`memmap2` (37 Treffer), keine Verschlüsselung gefunden. | `vector/src/persistence/mmap.rs` | verschlüsselt, über `Vfs` (D-19) |
| B-21 | Textindex ist termgeordnet im LSM (Schlüssel `(term, doc)`, Batch-Posting-Listen, Vorwärtsindex, Tombstones). | `text/src/inverted/index_struct.rs` | abgeleiteter Segmentindex (D-20) |
| B-22 | `block_on` an 7 Stellen im Bibliothekscode. | `grep` | async durchgängig (D-03) |
| B-23 | CI: 12× `continue-on-error: true` in `rust-ci.yml`. Vorhanden und nutzbar: `chaos`, `loom-gate`, `fuzz-extended`, `simd-cross-validation`, `tsan`, `unwrap-gate`, `proof-correctness`, `coverage`, `deny`, `bench`. | `.github/workflows/` | Blockierend herstellen (15.9) |
| B-24 | Testinfrastruktur vorhanden: `testkit` (`fault_vfs.rs`, `reference_model.rs`, `manual_clock.rs`, In-Memory-Ports, Conformance-Suiten), Store: 124 Testdateien, 6 Benchmarks (`redb_comparative_bench`, `lsm_concurrency_bench`, `wal_bench`, `sstable_bench`, `block_cache_bench`, `memtable_bench`), 8 Fuzz-Ziele. | Verzeichnislisten | erweitern statt neu bauen (Abschnitt 15) |
| B-25 | Ein Löschfehler im Altbestand (Purger ignorierte das Ziel und verwarf alle Mandantendaten) war ein Datenvernichtungsfehler Dritter. Die Klasse „falsches Ziel“ ist deshalb Release-Blocker (HR-7). | Historie des Repos | T-ISO-* |

### 3.3 Größenbasis (Zeilen `src`, Obergrenzen) [V]

`store` 27.057 · `engine` 20.863 · `vector` 13.630 · `text` 7.158 · `types` 7.259 · `mvcc` 4.033 · `ports` 3.862 · `testkit` 3.601 · `rank` 3.445 · `crypto` 3.293 · `simd` 3.062 · `db` 2.407 · `contextra` 1.870 · `sys` 1.860 · `durable-fs` 1.593. Summe ca. 105.000 Zeilen in 15 Crates.

### 3.4 Nicht geprüft (bewusst nicht behauptet)

Build- und Teststatus; Inhalt von `contextra-db` über die Dateiliste (`homeostat`, `multistep`, `volatile_vault`) hinaus; Einfügepfad der Engine im Detail; ob der Blockcache entschlüsselte Blöcke hält; HNSW-Snapshot-Format im Detail; Kompaktions-Retention-Semantik. Jede Lücke hat ein Arbeitspaket mit Messung.

---

## 4. Entscheidungsregister (final)

| ID | Entscheidung | Begründung | Test / Beleg |
|---|---|---|---|
| D-01 | **Löschen in drei Säulen, alle in v1:** (1) Gate und physischer Purge, (2) Schlüsselvernichtung je Dokument über den Tresor, (3) unabhängige Messung mit signiertem Bericht. | Überschreiben garantiert auf Flash nichts; Schlüsselvernichtung ist das einzige Mittel, das bei Dateisystem-Remanenz wirkt. Nachrüsten würde das Datenformat brechen. | T-VAULT-*, T-RAW-*, T-PROOF-1 |
| D-02 | **Eigener LSM-Tree bleibt, aber nur mit Löschcluster-Schlüsselraum (D-31).** Gate G0 (11.9) entscheidet verbindlich mit Zahlen. Rückfallebene: redb-Adapter hinter `StorageEngine`. | Der LSM-Vorteil ist die Dateigranularität. Mit verstreuten Hash-Schlüsseln läge ein Dokument in ca. 26 % aller Dateien (Rechnung 11.9), ein Purge schriebe fast die ganze Datenbank um. | B-ERASE-WA |
| D-03 | Ports sind `async` (`BoxFuture`). Kein `block_on` im Bibliothekscode. | Synchrone Ports über asynchroner I/O erzwingen `block_on` und Deadlocks (B-22). | T-LINT-2 |
| D-04 | **Ein** `Vfs`-Trait in `ports`, Implementierung in `durable-fs`. Store, Vektor, Text und Erasure nutzen ausschließlich `Vfs`. | B-12. Fehler- und Crashinjektion setzt `Vfs` voraus. | T-LINT-2, T-CRASH-* |
| D-05 | **Verschlüsselung und WAL-Integrität sind nicht abschaltbar.** Kein `Option<KeyManager>`, kein Klartextmodus, kein Feature `wal-integrity`/`encryption-at-rest`. | B-10, B-14. Ein Opt-out erzeugt Parallelbestände. | T-DEF-1, T-CFG-1 |
| D-06 | Passphrase nur über Argon2id, Minimum m = 64 MiB, t = 3, p = 1; Standard m = 128 MiB, t = 3, p = 1. Unterschreiten ist Fehler. HKDF nur für Subkeys. | B-15. | T-KDF-1 |
| D-07 | `TenantContext` ist Pflicht. Es gibt keine mandantenlose Zugriffsfunktion. | B-16. | T-API-1 (trybuild) |
| D-08 | Dokumentidentität: `doc_tag = BLAKE3-keyed(K_idx, "ctx/doc/v1" ‖ tenant ‖ coll ‖ doc_key)[0..16]`. Kollision mit anderem `doc_key` → `Conflict::IdCollision`. | B-19. 128 Bit keyed, nicht aus Nutzerkennung berechenbar. | T-ISO-3 |
| D-09 | Alle Schlüsselkomponenten sind **blinde Tags** (keyed BLAKE3, `K_idx`). Nutzerkennungen stehen nur verschlüsselt im Wert. | Der Indexbereich einer SSTable enthält Schlüssel im Klartext (B-14). Blinde Schlüssel halten ihn frei von Nutzerdaten. | T-RAW-4 |
| D-10 | Matcher sind **Schlüsselbereiche fester Tag-Länge** plus **exakte Schlüssel**. Keine String-Präfixe, keine variablen Präfixe. | Variable Präfixe waren Ursache falscher Ziele (B-25). | T-ISO-2, T-KEY-1 |
| D-11 | Zeitreisen bleiben. Purge entfernt alle Versionen. Offene Snapshots, die ein Ziel berühren, werden **widerrufen** (`SnapshotRevoked`). `history_horizon` (Standard 24 h) begrenzt die Versionsmenge. | Warten auf Snapshots macht Löschfristen unbegrenzt. | T-VER-1 |
| D-12 | **ReadGate** vor Purge, auch für Zeitreisen. | Sofortwirkung für Nutzer. | T-VER-1 |
| D-13 | Bericht: Ed25519-signiert, kanonisches JSON, ein Format, keine Legacy-Versionen. Signatur beweist Integrität und Herkunft aus der Installation, nicht die Wahrheit des Inhalts. | Symmetrische Beweise sind durch den Prüfer fälschbar. | T-PROOF-1 |
| D-14 | Betroffenen-Löschung über `subject_tag` = Cluster (D-31). | Ohne Zuordnung findet Contextra eine Person nicht. | T-SUBJ-1 |
| D-15 | `RetentionAttestation` ist Pflicht. **`Restricted` (Art. 18) ist Gate-Variante in v1**; Automatik ist kein v1-Umfang. | Das Gate existiert ohnehin; Art. 17 Abs. 3 und Art. 18 sind reale Rechtslagen. | T-API-2, T-RESTR-1 |
| D-16 | Keine Checkpoints, Forks, Hardlink-Klone. | Jede Kopie liegt außerhalb der Löschgrenze. | T-ARCH-3 |
| D-17 | Graph und Relationen entfernt, nicht per Feature abgeschaltet. | P5; Kantenreparatur pro Löschung ist fehleranfällig. | T-ARCH-3 |
| D-18 | Vektorlöschung: Slot-Zeroing sofort, Kanten bleiben bis **Batch-Rebuild** bei Tombstone-Quote ≥ 0,2 oder spätestens zum nächsten Purge-Fenster-Ende nach `erase_sla`. | Kanten enthalten nur Slot-Nummern. Rewiring pro Löschung ist teuer und riskant. | T-VEC-*, T-RAW-3 |
| D-19 | Vektor-Snapshots: Payload je Slot unter dem DEK des Dokuments, Graphkanten unter dem Store-Schlüssel, ausschließlich über `Vfs`. | B-20. Embeddings gelten vorsorglich als personenbezogen (E-L3). | T-RAW-3 |
| D-20 | Textindex: **abgeleiteter, segmentierter, verschlüsselter Index** (9.7). Der LSM hält keine Postings. | B-21. Per-Term-Schlüssel streuen jedes Dokument über den Schlüsselraum. | T-TEXT-*, B-ERASE-WA |
| D-21 | `tenant_tag`, `coll_tag`, `cluster_tag`, `doc_tag`, `term_tag` sind keyed BLAKE3 mit festen Labels (Anhang C). | Index enthält keine Klartexte. | T-RAW-4 |
| D-22 | Entfernte Crates liegen im Branch `archive/v0-full`. | Historie auffindbar. | PLAN WP-1.2 |
| D-23 | Lizenz-/DRM-Schicht entfällt. | Geschäftsmodell liegt außerhalb des Speicherkerns. | T-ARCH-3 |
| D-24 | Ingest, Privacy, Agent, Cognition, Router, Adapt, KV-Cache, Inference, Sandbox, Audit-Export, AVV-Generator, `contextra-db` entfallen. | Nicht im Kern. | T-ARCH-3 |
| D-25 | MCP-Server und Python-Bindings sind kein v1-Umfang. Sie entstehen nach API-Freeze als Adapter. | Löschwerkzeuge dürfen erst nach der Saga exponiert werden. | – |
| D-26 | `rank` enthält nur RRF-Fusion. | Hybridsuche ist Kern, der Rest nicht. | T-RANK-1 |
| D-27 | Keine automatische Entitäts- oder Metadatenextraktion. `AutoExtractionMode` entfällt. | B-17. Erzeugt Parallelbestände. | T-ARCH-3 |
| D-28 | SIMD: u8-VNNI-Kernel entfällt; f32-Kernel plus Skalar-Referenz. Jeder Kernel hat Cross-Check. | Falsche Operandentypen bei `_mm512_dpbusd_epi32` (u8×u8 statt u8×i8) am Muster belegt (`simd/src/kernels/avx512.rs`). | T-SIMD-1 |
| D-29 | `unsafe` nur in `contextra-sys` und `contextra-simd`. | Kleinste Auditfläche. | T-UNSAFE-1 |
| D-30 | `erase` kehrt nach persistiertem Gate und vernichtetem Schlüssel zurück (Status `Gated`), Ziel p50 ≤ 10 ms. `erase_sla` ist Pflichtparameter ohne Standardwert. | Fail-closed ohne Blockieren. | T-SLA-1, B-ERASE-LAT |
| D-31 | **Löschcluster-Schlüsselraum:** `region ‖ tenant_tag ‖ coll_tag ‖ cluster_tag ‖ doc_tag ‖ family ‖ sub` (9.1). Jedes Löschziel ist ein zusammenhängender Bereich. | P11, Rechnung 11.9. | B-ERASE-WA |
| D-32 | **Record-Envelope:** Jeder Wert liegt unter dem DEK seines Dokuments; zusätzlich Block-AEAD unter `K_enc`. | Entkoppelt Schlüsselvernichtung von der Blockstruktur: Ein Block mit Daten vieler Dokumente bleibt für andere lesbar. | T-ENC-*, T-VAULT-* |
| D-33 | **Tresor:** Slot je Dokument mit 32-Byte-Salt; `DEK = HKDF-SHA256(K_unit, salt, "ctx/dek/v1" ‖ doc_tag)`. Löschen = Slot nullen, `fsync`, Fehlen prüfen. | Kleinster löschbarer Anker. | T-VAULT-1..4 |
| D-34 | **`contextra-erasure`** hängt nur an `types`, `ports`, `crypto`. Schichten implementieren `ErasureLayer`. | P12. | T-ARCH-3 |
| D-35 | **Messung E1/E2/E3** (12.1) mit unabhängigem Lab. Ein Rohscan auf Klartext allein ist keine Messung. | In einem verschlüsselten Store ist er für Dateischichten fast immer leer. | T-RAW-2 |
| D-36 | **Klassenverband G, K, P** statt linearer Skala (12.5). | Die Aussagen sind unabhängig. | T-CLASS-1 |
| D-37 | **Restore-Guard** über Ledger (11.8). | Restore aus Backup kann gelöschte Daten zurückbringen. | T-RESTORE-1 |
| D-38 | `EraseTarget::Tenant` (Offboarding). | Häufigster realer Löschfall beim Betreiber. | T-TENANT-1 |
| D-39 | **Purge-Fenster:** Gate und Schlüsselvernichtung sofort, physischer Purge gebündelt bis höchstens `batch_window` (Standard 60 s) bzw. spätestens `erase_sla`; `Timing::Now` erzwingt Sofortlauf. | Bündelung senkt die Zahl berührter Dateien (11.9). | B-ERASE-BATCH |
| D-40 | Ein Dokument gehört zu höchstens einem Betroffenen. Der Betroffene ist **unveränderlich** (`Conflict::SubjectImmutable`). Mehrere Betroffene: Anwendung teilt in getrennte Dokumente. | Sonst ist der Cluster nicht eindeutig. | T-SUBJ-2 |
| D-41 | `scan` iteriert in `doc_tag`-Reihenfolge (Locator-Region), nicht in Nutzerschlüssel-Reihenfolge. | Folge von D-09 und D-31. | T-SCAN-1 |
| D-42 | Jeder Header (WAL, SSTable, Manifest, Tresor, Journal, Ledger, Snapshot, Textsegment, Bericht) trägt `format_version` und Algorithmus-IDs. | Krypto-Obsoleszenz planbar. | T-FMT-1 |
| D-43 | Kein Netzwerk, keine Telemetrie, kein Escrow im Produktionsbuild. | HR-1..3. | T-ARCH-4 |
| D-44 | Rot-vor-Grün; Evidenzbündel je Arbeitspaket. | Verifizierbar. | PLAN §1 |
| D-45 | `contextra-db` entfällt. Seine Module (`homeostat`, `multistep`, `volatile_vault`) sind nicht im Kern. | Features leiten an entfernte Crates. | T-ARCH-3 |
| D-46 | Tests dürfen den Master-Schlüssel direkt injizieren (`EngineBuilder::master_key_for_testing`) unter Feature `testing`, das **nur** den Argon2-Schritt ersetzt, nie die Verschlüsselung. CI prüft, dass kein Release-Build `testing` aktiviert. | Schnelle Tests ohne Sicherheitsschalter. | T-CFG-2 |
| D-47 | **Locator-Region:** Je Dokument ein Locator-Eintrag `LOC ‖ tenant ‖ coll ‖ doc_tag → sealed(cluster_tag)`. | `get(doc_key)` kennt den Cluster nicht. Locator-Schlüssel sind dicht und werden mit dem Dokument als exakter Schlüssel entfernt. | T-KEY-2 |
| D-48 | Commitment-Epoche und Ledger-Lebensdauer: 365 Tage; danach Rotation und Entfernen der Ledger-Einträge. | Pseudonymrest begrenzen (E-L4). | T-JRN-3 |
| D-49 | Textsegmente: Zielgröße 32 MiB, ein Dokument liegt zu jedem Zeitpunkt in genau einem Segment oder im Textpuffer. | Rewrite je Erase = ein Segment. | B-ERASE-WA |
| D-50 | Ein Datenverzeichnis hat genau einen schreibenden Prozess (exklusiver Verzeichnis-Lock über `Vfs`). | Journal- und Manifest-Konsistenz. | T-LOCK-1 |

---

## 5. Gesamtarchitektur

### 5.1 Workspace (Soll)

14 Mitglieder, dazu zwei dev-only Hilfs-Crates:

| Crate | Ring | Rolle |
|---|---|---|
| `contextra-types` | 0 | Reine Typen (IDs, Tags, Fehler, Löschtypen). Kein I/O, kein `async`. |
| `contextra-ports` | 0 | Traits: `StorageEngine`, `TextIndex`, `VectorIndex`, `Vfs`, `Clock`, `Rng`, `IdGen`, `ErasureLayer`, `ReadGate`, `RecordSealer`, `LedgerSink`, `Metrics`. |
| `contextra-sys` | 1 | `unsafe`-Insel: `fsync`, `flush_dir`, `rename_durable`, `mlock`, Dateirechte. |
| `contextra-simd` | 1 | `unsafe`-Insel: f32-Distanzkernel mit Skalar-Referenz. |
| `contextra-durable-fs` | 1 | `Vfs`-Implementierung (`RealVfs`), `scrub` best-effort. |
| `contextra-crypto` | 1 | AEAD, KDF, HKDF, WAL-HMAC, Hash-Kette, Ed25519. |
| `contextra-mvcc` | 2 | Snapshots, SSI, Sequenzen. |
| `contextra-store` | 2 | LSM-Tree (WAL, Memtable, SSTable, Manifest, Kompaktion, Purge-Primitive). |
| `contextra-text` | 2 | BM25-Segmentindex. |
| `contextra-vector` | 2 | HNSW, Arena, Snapshot. |
| `contextra-rank` | 2 | RRF-Fusion. |
| `contextra-erasure` | 3 | Löschkern: Saga, Gate, Journal, Tresor, Messung, Bericht, Ledger. |
| `contextra-engine` | 3 | Orchestrierung: Collections, Suche, Transaktionen, Verdrahtung. |
| `contextra` | 4 | Fassade, einzige öffentliche Oberfläche. |
| `contextra-testkit` | dev | Fixtures, `FaultVfs`, `ReferenceModel`, `ManualClock`, In-Memory-Ports. |
| `contextra-lab` | dev | Unabhängige Prüfinstanz (12.3). |
| `ctx-verify` | dev/bin | Eigenständiger Berichtsprüfer (12.7). |
| `tests/arch` | dev | Architektur- und Lint-Tests. |

Workspace-`members` = diese Liste (14 Produktions- plus Hilfs-Crates). `default-members` = die 14 Produktionscrates ohne `lab` und `ctx-verify`. Alle anderen Einträge, Pfadabhängigkeiten und `[workspace.dependencies]` auf entfernte Crates entfallen.

### 5.2 Ringe und Abhängigkeitsregeln

```
Ring 4  contextra                         (Fassade)
Ring 3  contextra-erasure   contextra-engine
Ring 2  contextra-mvcc  contextra-store  contextra-text  contextra-vector  contextra-rank
Ring 1  contextra-sys  contextra-simd  contextra-durable-fs  contextra-crypto
Ring 0  contextra-types  contextra-ports
dev     testkit  lab  ctx-verify  tests/arch
```

| Regel | Inhalt | Test |
|---|---|---|
| AR-1 | Abhängigkeiten zeigen nur auf gleichen oder niedrigeren Ring; innerhalb Ring 2 nur laut Matrix 5.3. | T-ARCH-3 |
| AR-2 | `store` kennt weder `text`, `vector` noch `erasure`. | T-ARCH-3 |
| AR-3 | `erasure` kennt **keine** Ring-2-Crates, nur `ports`-Traits, `types` und `crypto`. | T-ARCH-3 |
| AR-4 | `lab` und `ctx-verify` hängen nicht an `store`, `engine`, `erasure`, `crypto`-Produktions-Decoder. Sie haben eigene Implementierungen der Formate und Primitive (unabhängiger Prüfer). | T-ARCH-3 |
| AR-5 | `types` ohne `async` und ohne I/O. | T-ARCH-1 |
| AR-6 | Nur `TenantScopedStorage` ruft den rohen Store. | T-ARCH-2 |
| AR-7 | Kein Netzwerk-Crate im Abhängigkeitsbaum der Produktionscrates. | T-ARCH-4 |
| AR-8 | `unsafe` nur in `sys` und `simd`; alle anderen `#![forbid(unsafe_code)]`. | T-UNSAFE-1 |

### 5.3 Abhängigkeitsmatrix (erlaubte Kanten)

| von \ nach | types | ports | sys | simd | dfs | crypto | mvcc | store | text | vector | rank | erasure |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ports | ✓ | | | | | | | | | | | |
| sys | ✓ | | | | | | | | | | | |
| simd | ✓ | | | | | | | | | | | |
| durable-fs | ✓ | ✓ | ✓ | | | | | | | | | |
| crypto | ✓ | ✓ | | | | | | | | | | |
| mvcc | ✓ | ✓ | | | | | | | | | | |
| store | ✓ | ✓ | ✓ | | | ✓ | ✓ | | | | | |
| text | ✓ | ✓ | | | | ✓ | | | | | | |
| vector | ✓ | ✓ | | ✓ | | ✓ | | | | | | |
| rank | ✓ | ✓ | | | | | | | | | | |
| erasure | ✓ | ✓ | | | | ✓ | | | | | | |
| engine | ✓ | ✓ | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

`store` darf `Vfs` nur als Trait aus `ports` nutzen (nie `durable-fs` direkt); die konkrete `RealVfs` wird in `engine`/`contextra` verdrahtet.

### 5.4 Laufzeitmodell

- **Runtime:** Tokio multi-thread. Die Bibliothek erzeugt keine eigene Runtime, außer in `EngineBuilder::open` wird ein `tokio::runtime::Handle` übergeben (Pflicht); alle Hintergrundtasks laufen auf diesem Handle.
- **Cancel-Safety (I-14):** Jede mutierende Operation läuft in einer vom Aufrufer unabhängigen Task (`spawn`); der Aufrufer wartet auf ein `oneshot`. Abbruch des Aufruferfutures ändert den Ausgang nicht.
- **Hintergrundtasks:** WAL-Flusher, Memtable-Flush, Kompaktion, Purge-Scheduler, Vektor-Rebuild, Textsegment-Merge, Ledger-Rotation. Alle besitzen Stop-Flag und `JoinHandle`; `close()` beendet sie geordnet.
- **Sperren:** Pro Dokument ein asynchroner Lock (Writes, Erase, Update teilen ihn); eine Purge-Barrier (`RwLock`) zwischen Kompaktion (shared) und Purge (exklusiv). Lockreihenfolge: Dokumentlock → Barrier → Journal. Keine Sperre über `.await` einer fremden Task.
- **Keine Blockierung in `Drop`** (14.3).

### 5.5 Datenflüsse

**Schreiben (Insert/Update):**
`Collection::insert` → TenantContext prüfen → ReadGate prüfen (Ziel nicht `Restricted`/`Gated`) → Dokumentlock → `doc_tag`, `cluster_tag` ableiten → Tresor: Slot anlegen (neu) oder DEK holen → Envelope sealen (Doc, DocMap, Manifest, Locator) → Textpuffer und Vektorpuffer vormerken → **eine** Store-Transaktion (WAL-Frame, HMAC, `fsync` nach Gruppen-Commit) → Sichtbarkeit.

**Lesen:** `get(doc_key)` → `doc_tag` → ReadGate → Locator lesen → Cluster → Record lesen → DEK (Tresor/LRU) → Envelope öffnen. Fehlt der Tresor-Slot: `Erased`.

**Suchen:** Text/Vektor liefern `doc_tag`-Treffer → ReadGate-Filter → RRF → Dokumente laden.

**Löschen:** `erase` → Validierung, Retention → Journal `Requested` → Dokumentlocks → Gate persistieren → Tresor-Slots nullen → Rückgabe `Gated` → Purge-Fenster → Drain → Rewrite → Unlink → Indexbereinigung → Messung → Bericht → `Completed` (11.2).

### 5.6 Verzeichnislayout

```
<data>/                       0700
  LOCK                        exklusiver Prozess-Lock (D-50)
  HEADER                      Format-Version, KDF-Parameter, Salt[32], Store-ID, Algorithmus-IDs
  wal/        *.wal           HMAC-Kette, verschlüsselt
  sst/        *.sst           SSTables (Block-AEAD), Level-Struktur
  MANIFEST/   *.log           Manifest-Log (Datei-Mengen, Schlüsselbereiche, Seq)
  text/       *.seg           Textsegmente (verschlüsselt)
  vector/     *.snap          Vektorsnapshots (verschlüsselt)
  vault/      vault.dat       Tresor (Slots)
  erasure/    journal.log     Löschjournal (Hash-Kette, verschlüsselt)
              erased.set      aktive Gates (Bereiche, Commitments)
              ledger.log      abgeschlossene Löschungen (Restore-Guard)
              scope/          Auflösung je laufender Löschung (verschlüsselt)
              report/         signierte Berichte
  keys/       0700, Dateien 0600 (Signaturschlüssel-Seed verschlüsselt)
  tmp/        Waisen werden beim Start entfernt
```

Außerhalb des Datenverzeichnisses: das Ledger-Exemplar des Betreibers (`LedgerSink`, 11.8).

### 5.7 Konfiguration (`EngineBuilder`)

| Parameter | Pflicht | Standard | Prüfung |
|---|---|---|---|
| `data_dir` | ja | – | existiert oder wird mit 0700 angelegt |
| `passphrase` | ja | – | Länge ≥ 12 Bytes |
| `erase_sla` | ja | – | 1 min ≤ sla ≤ 30 d |
| `runtime` (Handle) | ja | – | – |
| `vfs`, `clock`, `rng` | ja | – | Traits |
| `kdf` | nein | m = 128 MiB, t = 3, p = 1 | ≥ Minimum (D-06) |
| `erase_return_timeout` | nein | 60 s | 1 s … 10 min |
| `batch_window` | nein | 60 s | 0 s … `erase_sla` |
| `history_horizon` | nein | 24 h | 0 … 365 d |
| `max_doc_bytes` | nein | 4 MiB | ≤ 64 MiB |
| `vector_rebuild_ratio` | nein | 0,2 | 0,05 … 0,5 |
| `commit_epoch` | nein | 365 d | 30 d … 3 Jahre |
| `ledger_sink` | nein | Datei außerhalb des Datenverzeichnisses (`<data>/../.contextra-ledger/<store_id>`) | Pfad ≠ Unterpfad von `data_dir` |

Fehlende Pflichtparameter oder Werte außerhalb der Grenzen sind `ConfigError` beim `open`. Es gibt keine stillen Korrekturen.

### 5.8 Budgets (CI-blockierend)

| Budget | Wert | Messung |
|---|---|---|
| Erase-Produktcode (`contextra-erasure`, `store/src/erase`, `types/src/erasure`, `crypto/src/vault_cipher.rs`) | ≤ 6.000 Zeilen | `tokei` |
| `unwrap`/`expect`/`panic!`/`let _ =`/`#[allow]` im Produktionscode | nur sinkend; Ziel 0 in allen Kern-Crates ab Gate G-C | `governance/ratchet.toml` |
| Clippy | 0 Meldungen mit `-D warnings` | CI |
| Produktcode Kern gesamt | Reduktion ≥ 40 % gegenüber der in WP-0.1 gemessenen Basislinie | `tokei` |
| `unsafe` | nur `sys`, `simd` | `forbid(unsafe_code)` sonst |

---

## 6. Rechts- und Haftungsrahmen als technische Anforderung

### 6.1 Rollen

Verantwortlicher im Sinne der DSGVO ist der **Betreiber**, der Contextra in seine Anwendung einbaut. Ihn treffen die Pflichten aus Art. 17, 18, 25, 32. Contextra haftet nicht für diese Pflichten, sondern ausschließlich dafür, (a) **was es zusagt**, (b) **ob es als Produkt fehlerhaft ist**, und (c) falls es selbst personenbezogene Daten erhielte. Punkt (c) wird **konstruktiv ausgeschlossen**: Contextra sieht nie Daten des Betreibers, weil es lokal läuft und nichts nach außen sendet (HR-1..4). Dadurch ist Contextra bei bestimmungsgemäßem Betrieb weder Verantwortlicher noch Auftragsverarbeiter (E-L6).

### 6.2 Haftungsköpfe und Designwirkung

| ID | Haftungskopf | Auslöser | Designmaßnahme | Test |
|---|---|---|---|---|
| H1 | Irreführende Werbung (§ 5, § 5a UWG) | Aussagen wie „DSGVO-konform“, „garantiert gelöscht“ in README, Doku, Berichten | Wortlaut-Lint blockierend; Außenaussagen nur aus 6.4 und Berichtsklassen | T-LINT-1 |
| H2 | Beschaffenheitsvereinbarung (§ 434, §§ 280 ff. BGB) | Werbung und Dokumentation werden Vertragsinhalt | Leistungsbeschreibung (Anhang B) schmal; jede Zeile hat Test und Evidenzlauf | T-LEIST-1..12 |
| H3 | **Produkthaftung** (ProdHaftG nach Umsetzung der RL (EU) 2024/2853, Software ist Produkt, Anwendung ab 9.12.2026) | Falsches Löschziel, Datenverlust nach Absturz, unbemerkte Korruption | T-ISO, Crash-Matrix, WAL-HMAC, Block-AEAD sind haftungskritisch; Release-Blocker HR-7; Update- und Sicherheitsupdate-Prozess (`SECURITY.md`) | T-ISO-*, T-CRASH-*, T-WAL-1 |
| H4 | Cyber Resilience Act (VO (EU) 2024/2847) | Bereitstellung eines Produkts mit digitalen Elementen | SBOM (`cargo cyclonedx`) je Release, `SECURITY.md` mit Meldeweg und Fristen, benannter Support-Zeitraum, signierte Releases | T-SUPPLY-1 |
| H5 | DSGVO direkt | Contextra-Seite erhält Daten (Telemetrie, Crash-Upload, Lizenzserver, Escrow) | HR-1..HR-4 | T-ARCH-4, T-LOG-1 |
| H6 | Berufsgeheimnis (§ 203 StGB) | Einsatz bei Kanzlei, Arzt, Steuerberater | Kein Zugriff durch Contextra; Support nur mit Berichten ohne Inhalt | HR-4 |
| H7 | Verlässlichkeit der Nachweise | Betreiber verlässt sich auf einen falschen Bericht | Fail-closed, Positivkontrolle, Mutationstests auf `classify`, unabhängiger Verifier | T-RAW-2, T-PROOF-1, T-MUT-1 |
| H8 | Krypto-Obsoleszenz (Art. 32) | Algorithmus gebrochen oder Parameter veraltet | Algorithmus-ID und Format-Version in jedem Header; jährliche Neubewertung (Release-Checkliste) | T-FMT-1 |

### 6.3 Harte Regeln

| ID | Regel | Durchsetzung |
|---|---|---|
| HR-1 | Kein Netzwerkzugriff im Produktionsbuild. Keine HTTP-, Socket- oder DNS-Bibliothek im Abhängigkeitsbaum der Produktionscrates. | `deny.toml [bans]` (`reqwest`, `hyper`, `ureq`, `curl`, `isahc`, `surf`, `attohttpc`, `tonic`), `cargo tree`-Test, Quellscan nach `std::net`, `tokio::net` |
| HR-2 | Keine Telemetrie, kein Crash-Upload, kein Phone-Home. | wie HR-1 |
| HR-3 | **Kein Schlüssel-Escrow, keine Wiederherstellung.** Passphrase verloren heißt Daten verloren; dies steht an erster Stelle der Dokumentation. | Doku-Test, Review |
| HR-4 | Fehler- und Lognachrichten enthalten nie Inhalte, Nutzerkennungen, Schlüssel oder Pfade mit Personenbezug. | T-LOG-1 |
| HR-5 | Außenaussagen ausschließlich aus 6.4. | T-LINT-1 |
| HR-6 | Jede Zeile der Leistungsbeschreibung hat mindestens einen automatisierten Test und einen Evidenzlauf. | CI-Job `leistung` |
| HR-7 | Ein Test, der zeigt, dass ein Erase Daten außerhalb des Ziels verändert, ist **Release-Blocker ohne Ausnahme**. | G-D, G-H |
| HR-8 | Sicherheitslücken werden innerhalb dokumentierter Fristen bearbeitet (kritisch 7 Tage, hoch 30 Tage); unterstützte Versionen sind benannt. | `SECURITY.md`, Release-Checkliste |
| HR-9 | Haftungsausschlüsse der Lizenz (MIT/Apache-2.0) werden **nicht** als Schutz verbucht. Für Vorsatz, grobe Fahrlässigkeit und Produkthaftung gelten zwingende Grenzen. | Prozess |

### 6.4 Zulässige Außenaussage (einzige)

> Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis, aus den Indexen und aus den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Für jedes Ziel wird zusätzlich der dokumentspezifische Schlüssel aus dem Schlüsseltresor entfernt; ob und wie das rechtlich zu werten ist, entscheidet der Betreiber. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter. Der Bericht listet diese Restrisiken.

**Verbotene Begriffe** (Wortlaut-Lint über Quelle, Doku, Berichtsvorlagen, Marketing): „garantiert“, „100 %“ (im Zusammenhang mit Löschung), „DSGVO-konform“, „rechtskonform“, „unwiederbringlich“, „unwiderruflich“, „informationstheoretisch“, „kryptografisch gelöscht“, „sicher gelöscht“ ohne Berichtsbezug, „vollständig gelöscht“ ohne Klassenangabe, „zertifiziert“, „von einer Aufsichtsbehörde geprüft“.

### 6.5 Rechtsanforderungsmatrix

| # | Anforderung | Mechanismus | Test | Restrisiko im Bericht |
|---|---|---|---|---|
| R1 | Unverzüglich, Monatsfrist (Art. 17 Abs. 1, Art. 12 Abs. 3) | Saga mit Wiederholung, `erase_sla`, Status, Alarm | T-CRASH-1, T-SLA-1 | `ReplicaWindow` |
| R2 | Unkenntlichmachen, nicht reproduzierbar (SDM-Baustein 60, ErwGr 26) | Klassen K und P | T-RAW-1..5 | `FlashRemanence`, `WrappedKeyRemanence` |
| R3 | Einfache OS-Löschbefehle genügen nicht (SDM-Baustein 60) | Rewrite, Unlink, Fehlen geprüft, Tresor-Slot genullt | T-CRASH-1, T-RAW-1 | `FsJournalOrCow` |
| R4 | Alle Versionen und Kopien (SDM M60.P14) | Purge aller Versionen, Snapshot-Widerruf, Index-Segmente | T-VER-1 | `ExternalCopies` |
| R5 | Rechenschaft (Art. 5 Abs. 2, Art. 24) | signierter Bericht, Audit-Kette | T-PROOF-1, T-REPORT-1 | `PseudonymCommitment` |
| R6 | Datenschutz durch Technikgestaltung und Voreinstellung (Art. 25) | Verschlüsselung und Mandantenpflicht ohne Opt-out | T-DEF-1, T-API-1 | – |
| R7 | Sicherheit der Verarbeitung (Art. 32) | Argon2id, WAL-HMAC, AEAD mit AAD | T-KDF-1, T-WAL-1, T-ENC-2 | Schlüsselkompromittierung |
| R8 | Information von Empfängern (Art. 17 Abs. 2, Art. 19) | Bericht führt `ExternalProcessors`; Checkliste (16) | – | Verantwortung Betreiber |
| R9 | Aufbewahrungspflichten (Art. 17 Abs. 3), Einschränkung (Art. 18) | `RetentionAttestation`, `Restricted` | T-API-2, T-RESTR-1 | Prüfung beim Betreiber |
| R10 | Embeddings vorsorglich personenbezogen | Vektoren im Umfang, Snapshot verschlüsselt, Slot-Zeroing | T-RAW-3, T-RAW-5 | `ProcessMemory`, `GraphRebuildPending` |
| R11 | Nach Restore erneut löschen (SDM M60.S10, M60.P08) | Restore-Guard (Ledger) | T-RESTORE-1 | Backup-Inhalte außerhalb |
| R12 | Löschbarkeit auf Datensatz- und Gruppenebene mit beherrschbarem Aufwand (SDM M60.D01) | Löschcluster-Schlüsselraum, Segmentindex | B-ERASE-WA | – |
| R13 | Datensparsames Löschprotokoll (SDM M60.S05/S06) | Journal-Reduktion, Commitments mit Epochenrotation | T-REPORT-1, T-JRN-3 | `PseudonymCommitment` |
| R14 | Regeln zu Schlüsseln zu löschender Daten (SDM M60.P09/P10) | Tresor-Protokoll (11.5) | T-VAULT-1..4 | `WrappedKeyRemanence` |
| R15 | Algorithmenwechsel planbar | Algorithmus-ID in Headern | T-FMT-1 | – |
| R16 | Werbeaussagen belegbar (§ 5 UWG) | Wortlaut-Lint | T-LINT-1 | – |

### 6.6 Rechtsentscheidungen (technisch konservativ, endgültig)

Jede Frage, die die Technik berührt, ist hier entschieden. Maßstab ist immer die **strengste** Auslegung.

| ID | Frage | Entscheidung | Folge |
|---|---|---|---|
| E-L1 | Genügt physische Umschreibung samt Entfernen der Altdateien? | Sie ist Pflichtmaßnahme (Klasse P), wird aber **nie allein** als ausreichend dargestellt. Der Bericht führt `FlashRemanence` und `FsJournalOrCow` immer, wenn zutreffend. | Säule 1 plus 2 |
| E-L2 | Ist die Vernichtung des Dokumentschlüssels (Klasse K) Löschung, Anonymisierung oder Einschränkung? | Contextra **behauptet keine** dieser Wertungen. Der Mechanismus läuft immer. Der Bericht weist `key_destroyed` als Tatsache aus und nennt `WrappedKeyRemanence`. Die Wertung trifft der Betreiber. | Bericht, 6.4 |
| E-L3 | Sind Embeddings, HNSW-Slotnummern und blinde Tags personenbezogen? | **Ja, vorsorglich.** Embeddings werden zeroisiert, Snapshots verschlüsselt, Tags im Purge entfernt. Der Graph-Rest (Slot-Nummern ohne Inhalt) wird bis zum Rebuild als `GraphRebuildPending` ausgewiesen und ist spätestens nach `erase_sla` beseitigt. | D-18, D-19 |
| E-L4 | Sind Bericht, reduziertes Journal und Ledger personenbezogen? | **Pseudonym ja, daher begrenzt:** nur Commitments, Bereichs-Tags und Zeiten, nie Kennungen; Lebensdauer = Commitment-Epoche (365 d), danach Entfernen. | D-48 |
| E-L5 | Sind aggregierte Textstatistiken (Dokumentzahl, Termfrequenzen je Segment) personenbezogen? | Sie enthalten keine Dokumentbezüge. Sie werden dennoch **nach jedem Segment-Rewrite exakt neu berechnet**; kein Zähler überlebt die Löschung seines Beitrags. | 9.7 |
| E-L6 | Rolle von Contextra? | Weder Verantwortlicher noch Auftragsverarbeiter, solange HR-1..4 gelten. Dokumentation weist darauf hin; Betreiber prüft selbst. | HR |
| E-L7 | Aufbewahrungspflichten? | Contextra führt kein Fristenwerk. Der Betreiber erklärt per `RetentionAttestation`. Bei Pflicht: `Refused(RetentionObligation)`, oder `Restricted` als Gate-Variante ohne Vernichtung. | D-15 |
| E-L8 | Löschcheckliste und Berichtstext? | Wortlaut ist in Abschnitt 16 und Anhang D **endgültig festgelegt**; sie sind kein Rechtsrat und enthalten diesen Hinweis. | Lint |
| E-L9 | ProdHaftG/CRA bei Open-Source-Bibliothek? | Contextra behandelt sich als **dem CRA und dem ProdHaftG unterliegend** (strengste Annahme): SBOM, Meldeweg, Support-Zeitraum, Sicherheitsupdates. | H3, H4 |
| E-L10 | Wirkung von Haftungsausschlüssen? | Werden nicht als Schutz verbucht (HR-9). Die Leistungsbeschreibung ist schmal und testbar gehalten. | Anhang B |
| E-L11 | Reichweite der Signatur? | Die Signatur beweist Integrität und Herkunft aus der Installation, nicht die Wahrheit des Inhalts. Dieser Satz steht im Bericht. | D-13 |
| E-L12 | Liste der `RetentionCode`s? | Festgelegt in 7.2 (Enum ohne Freitext). | T-API-2 |

---

## 7. Öffentliche API (Rust, Crate `contextra`)

Alle öffentlichen Typen sind `#[non_exhaustive]`, außer reinen Konfigurationsbausteinen. Keine öffentlichen `String`-Felder für Schlüssel oder Passphrasen. Kein öffentlicher Zugriff auf `store`, `vector`, `text`, `erasure` oder `engine`.

### 7.1 Öffnen, Schließen

```rust
pub struct EngineBuilder { /* privat */ }
impl EngineBuilder {
    pub fn new(data_dir: impl AsRef<Path>) -> Self;
    pub fn passphrase(self, secret: SecretBytes) -> Self;            // Pflicht
    pub fn erase_sla(self, sla: Duration) -> Self;                    // Pflicht, kein Standard
    pub fn runtime(self, handle: tokio::runtime::Handle) -> Self;     // Pflicht
    pub fn vfs(self, vfs: Arc<dyn Vfs>) -> Self;                      // Pflicht (Fassade: system_defaults())
    pub fn clock(self, clock: Arc<dyn Clock>) -> Self;                // Pflicht
    pub fn rng(self, rng: Arc<dyn Rng>) -> Self;                      // Pflicht
    pub fn kdf(self, p: KdfParams) -> Self;                           // optional, >= Minimum
    pub fn erase_return_timeout(self, t: Duration) -> Self;           // 60 s
    pub fn batch_window(self, w: Duration) -> Self;                   // 60 s
    pub fn history_horizon(self, h: Duration) -> Self;                // 24 h
    pub fn vector_rebuild_ratio(self, r: f32) -> Self;                // 0.2
    pub fn commit_epoch(self, d: Duration) -> Self;                   // 365 d
    pub fn ledger_sink(self, s: Arc<dyn LedgerSink>) -> Self;         // Datei außerhalb data_dir
    pub fn system_defaults(self) -> Self;                             // RealVfs, SystemClock, OsRng
    pub async fn open(self) -> Result<Contextra, ContextraError>;
}

impl Contextra {
    pub async fn close(self) -> Result<(), ContextraError>;           // definierter Pfad (14.3)
}
```

`open` führt `recover()` (11.7) und den Restore-Guard (11.8) vollständig aus, **bevor** irgendein Lesepfad freigegeben wird. Fehlt ein Pflichtparameter: `ContextraError::Config`.

### 7.2 Mandanten, Collections, Dokumente

```rust
pub struct TenantContext { /* TenantId, validiert */ }
impl TenantContext { pub fn new(tenant: TenantId) -> Self; }        // TenantId: 1..=128 Byte UTF-8

pub struct CollectionSpec {
    pub vector_dim: Option<u16>,        // 16..=4096, fest pro Collection
    pub text_index: bool,
}

impl Contextra {
    pub async fn create_collection(&self, ctx: &TenantContext, name: &CollectionName, spec: CollectionSpec)
        -> Result<Collection, ContextraError>;
    pub async fn collection(&self, ctx: &TenantContext, name: &CollectionName)
        -> Result<Collection, ContextraError>;
}
// Ohne TenantContext existiert kein Weg zu Collection (T-API-1).

pub struct NewDocument {
    pub key: DocKey,                    // 1..=512 Byte, vom Aufrufer
    pub subject: Option<SubjectId>,     // 1..=256 Byte; unveränderlich (D-40)
    pub text: Option<String>,           // <= 64 KiB, wird indexiert
    pub payload: Bytes,                 // gesamt <= max_doc_bytes (4 MiB)
    pub metadata: Metadata,             // <= 64 Einträge, <= 16 KiB, Werte: Text, i64, bool
    pub vector: Option<Vec<f32>>,       // Länge == vector_dim, alle Werte endlich
}

impl Collection {
    pub async fn insert(&self, doc: NewDocument) -> Result<(), ContextraError>;          // existiert: Update
    pub async fn insert_batch(&self, docs: Vec<NewDocument>) -> Result<(), ContextraError>; // <= 1.000, atomar
    pub async fn get(&self, key: &DocKey) -> Result<Option<Document>, ContextraError>;
    pub async fn get_at(&self, key: &DocKey, at: Seq) -> Result<Option<Document>, ContextraError>;
    pub async fn scan(&self, after: Option<ScanCursor>, limit: u32) -> Result<Page<Document>, ContextraError>;
    pub async fn scan_at(&self, at: Seq, after: Option<ScanCursor>, limit: u32) -> Result<Page<Document>, ContextraError>;
    pub async fn delete(&self, key: &DocKey) -> Result<(), ContextraError>;              // LOGISCH (D-51)
    pub async fn search_text(&self, q: &TextQuery) -> Result<Vec<Hit>, ContextraError>;
    pub async fn search_vector(&self, q: &VectorQuery) -> Result<Vec<Hit>, ContextraError>;
    pub async fn search_hybrid(&self, q: &HybridQuery) -> Result<Vec<Hit>, ContextraError>;
    pub fn snapshot(&self) -> Result<Snapshot, ContextraError>;                          // `at`-Quelle, widerrufbar
}
```

**D-51 `delete` ist logisch.** Es schreibt einen Tombstone; ältere Versionen bleiben bis `history_horizon` lesbar (Zeitreise). Das Rustdoc und die Dokumentation sagen an erster Stelle: „Für Löschpflichten `erase` verwenden.“ Nach Ablauf von `history_horizon` entfernt die Kompaktion Versionen und Tombstones physisch; dies ist keine Löschaussage.

**`RetentionCode`** (abschließend, ohne Freitext):

```rust
#[non_exhaustive]
pub enum RetentionCode {
    CommercialRecordRetention,   // handels-/steuerrechtliche Aufbewahrung
    TaxRecordRetention,
    OtherStatutoryObligation,    // Art. 17 Abs. 3 lit. b (Rechtspflicht)
    PublicInterestTask,          // Art. 17 Abs. 3 lit. b (öffentliche Aufgabe)
    PublicHealth,                // Art. 17 Abs. 3 lit. c
    ArchivingResearchStatistics, // Art. 17 Abs. 3 lit. d
    LegalClaims,                 // Art. 17 Abs. 3 lit. e
    FreedomOfExpression,         // Art. 17 Abs. 3 lit. a
}
```

### 7.3 Löschen

```rust
#[non_exhaustive]
pub enum EraseTarget {
    Document   { collection: CollectionName, doc: DocKey },
    Subject    { subject: SubjectId },                  // über alle Collections des Mandanten
    Collection { collection: CollectionName },
    Tenant,                                             // Offboarding: alles dieses Mandanten
}

pub enum Timing { Windowed, Now }                       // D-39

pub struct EraseRequest {
    pub target: EraseTarget,
    pub retention: RetentionAttestation,                // Pflicht
    pub timing: Timing,
}

pub enum RetentionAttestation {
    NoObligationConfirmed,                              // Betreiber bestätigt
    ObligationDeclinesErasure { code: RetentionCode },  // Löschung wird verweigert
    ObligationRestricts { code: RetentionCode },        // Gate ohne Vernichtung (Restricted)
}

impl Contextra {
    pub async fn erase(&self, ctx: &TenantContext, req: EraseRequest) -> Result<ErasureHandle, ContextraError>;
    pub async fn erasure_status(&self, ctx: &TenantContext, id: ErasureId) -> Result<ErasureStatus, ContextraError>;
    pub async fn lift_restriction(&self, ctx: &TenantContext, id: ErasureId) -> Result<(), ContextraError>;
    pub async fn verify(&self, ctx: &TenantContext, target: &EraseTarget) -> Result<VerificationReport, ContextraError>;
    pub async fn export_report(&self, ctx: &TenantContext, id: ErasureId) -> Result<Vec<u8>, ContextraError>; // kanonisches JSON
    pub fn erasure_checklist(&self) -> String;                                           // Abschnitt 16
}

pub struct ErasureHandle { /* … */ }
impl ErasureHandle {
    pub fn id(&self) -> ErasureId;
    pub async fn status(&self) -> Result<ErasureStatus, ContextraError>;
    pub async fn wait(&self, timeout: Duration) -> Result<ErasureStatus, ContextraError>;
}

#[non_exhaustive]
pub enum ErasureStatus {
    Gated,                                  // Sperre persistiert, Schlüssel vernichtet, Purge geplant/läuft
    Pending { attempt: u32, next_retry_at: Timestamp },   // Fehler in Phase, Wiederholung geplant
    Restricted,                             // Einschränkung aktiv (kein Schlüsselvernichten, kein Purge)
    Completed(ErasureReport),               // gemessen, signiert
    Refused { reason: RefusalReason },      // terminal, ändert keinen Datenbestand
}
pub enum RefusalReason { RetentionObligation(RetentionCode), TargetUnknown, Closed }
```

**Regeln (normativ):**

1. `erase` kehrt zurück, sobald Gate und Schlüsselvernichtung persistiert sind (`Gated`); p50 ≤ 10 ms. Die Fertigstellung ist asynchron.
2. `Completed` nur nach Messung **aller** Pflichtschichten (12.4).
3. `Refused` ist terminal und verändert keinen Datenbestand. Ein **unbekanntes Ziel** ist **kein** Fehler: `Completed` mit leerer Trefferliste, gemessen (damit Wiederholung idempotent ist).
4. `ObligationRestricts` setzt `Restricted`: Gate ohne Schlüsselvernichtung und ohne Purge. `lift_restriction` hebt auf; eine anschließende `erase` ist möglich.
5. Wiederholter `erase` mit gleichem Ziel ist idempotent (gleiche `ErasureId`, solange nicht `Completed`).
6. `erase_sla` überschritten: Alarm (Metrik `erase_sla_breached` und Journal-Eintrag), kein Abbruch.

### 7.4 Fehlertaxonomie

```rust
#[non_exhaustive]
pub enum ContextraError {
    Config(ConfigError),            // fehlende Pflichtangabe, zu schwache KDF-Parameter, Grenzwert verletzt
    Auth(AuthError),                // falsche Passphrase
    Integrity(IntegrityError),      // WAL-HMAC, AEAD, Manifest, Journal-Kette, Tresor, LegacyFormat
    Io { op: IoOp, kind: io::ErrorKind },   // ohne Pfad und ohne Inhalt
    Conflict(ConflictError),        // IdCollision, SubjectImmutable, SSI-Schreibkonflikt
    SnapshotRevoked { seq: Seq },
    Erased,                         // Ziel gesperrt (Gate) oder Schlüssel vernichtet
    Limit(LimitError),              // Ressourcengrenze (14.6)
    Unsupported(&'static str),
    Closed,
}
```

Fehlermeldungen enthalten **nie** Schlüssel, Inhalte, Nutzerkennungen oder Pfade mit personenbezogenen Namen (T-LOG-1). `Display` liefert feste Texte.

---

## 8. Crate-Spezifikationen

Jedes Crate nennt: Zweck, Ring, erlaubte Abhängigkeiten, Zielmodule, öffentliche Oberfläche, Invarianten, Fehlerverhalten, Tests, Benchmarks, Delta zum Ist.

### 8.1 `contextra-types` (Ring 0)

**Zweck:** Reine Typen. **Verboten:** `async`, I/O, Zeit, Zufall, Threads. **Abhängigkeiten:** `serde`, `thiserror`, `zeroize`, `bytes`.

| Modul (Soll) | Inhalt |
|---|---|
| `ids` | `TenantId`, `CollectionName`, `DocKey`, `SubjectId` (validierte Newtypes mit Längengrenzen); `TenantTag`, `CollTag`, `ClusterTag`, `DocTag`, `TermTag` (je `[u8;16]`, `Copy`, `Ord`, kein `Display` mit Inhalt); `Seq(NonZeroU64)`, `TxId(u64)`, `ErasureId([u8;16])`, `SegmentId(u64)`, `Slot(u32)` |
| `key` | `Region` (`Data=0x01`, `Loc=0x02`, `Meta=0x03`), `Family` (Anhang C), `KeyBuf` (Aufbau/Parse nach 9.1), `KeyRange { start, end_exclusive }`, `Matcher { ranges: Vec<KeyRange>, exact: BTreeSet<KeyBuf> }` |
| `erasure` | `EraseTarget`, `ErasureState`, `LayerId`, `LayerStatus`, `ErasureClass` (Verband G/K/P), `ResidualRisk`, `ErasureReport`-Struktur, `RetentionAttestation`, `RetentionCode`, `RefusalReason`, `NeedleSet`, `LayerScan`, `Evidence` |
| `error` | `ContextraError` und Teilfehler (7.4) |
| `secret` | `SecretBytes` (`Zeroizing<Vec<u8>>`, kein `Clone`, kein `Debug`-Inhalt, kein `Serialize`), `SecretKey32` |
| `time` | `Timestamp(i64)` (µs seit Unix-Epoche), `Duration`-Wrapper für Konfiguration |
| `limits` | Konstanten aus 14.6 |

**Invarianten:** (a) Alle Tag-Typen sind unveränderlich und implementieren kein `Display`, nur `Debug` mit gekürzter Hex-Darstellung der ersten 2 Bytes. (b) `Seq` kennt keinen Sentinel; „kein Snapshot“ ist `Option<Seq>` (B-11). (c) `ErasureClass::display()` liefert ausschließlich die Texte aus 12.5.
**Tests:** T-ARCH-1; Property-Tests für `KeyBuf`-Roundtrip (alle Längen), `Matcher`-Mengenlogik (disjunkt, sortiert, normalisiert), `Seq`-Konvertierungen.
**Delta:** `DocId(u64|u128)`, `witness.rs`-Deklaration, `AutoExtractionMode` und alle Graph-/Checkpoint-/Lizenz-Typen entfernen; `types/src/erasure`, `key`, `secret` neu.

### 8.2 `contextra-ports` (Ring 0)

**Zweck:** Alle Traits, über die Ring 1–3 gekoppelt sind. **Abhängigkeiten:** `types`, `futures` (nur `BoxFuture`).

| Trait | Methoden (Kern) | Implementierer |
|---|---|---|
| `Vfs` / `VfsFile` | `open(path, OpenOptions)`, `read_at`, `write_at`, `append`, `set_len`, `sync_data`, `sync_all`, `rename`, `remove_file`, `exists`, `create_dir_all`, `read_dir`, `sync_dir`, `lock_exclusive`, `set_permissions`, `metadata` | `RealVfs` (durable-fs), `FaultVfs`, `MemVfs` (testkit) |
| `Clock` | `now() -> Timestamp`, `monotonic() -> u64` | `SystemClock`, `ManualClock` |
| `Rng` | `fill(&mut [u8])` (nur für Nicht-Schlüsselmaterial); `OsEntropy::fill` separat für Schlüssel, Salt, Nonce | `OsRng`-Wrapper, `SeededRng` |
| `IdGen` | `erasure_id()`, `report_id()` | – |
| `StorageEngine` | `get`, `get_at_seq`, `put`, `put_batch`, `delete`, `commit`, `rollback`, `scan_range`, `scan_range_at`, `flush`, `last_seq`, `pin_snapshot`/`unpin_snapshot`, `stats` – alle `async`, alle auf `KeyBuf`/Bytes | `LsmStorage`, `InMemoryStore` (testkit), redb-Adapter (nur Rückfall) |
| `TextIndex` | `add(doc_tag, tokens)`, `remove_docs(set)`, `search(coll, query, k)`, `flush`, `merge`, `segments_of(doc_tag)` | `contextra-text` |
| `VectorIndex` | `insert(slot, vec)`, `search`, `zero_slot`, `rebuild`, `snapshot`, `restore` | `contextra-vector` |
| **`ErasureLayer`** | `layer_id()`, `plan(&Matcher) -> LayerPlan`, `purge(&PurgePlan, &Deadline) -> PurgeOutcome`, `scan_residue(&NeedleSet) -> LayerScan`, `positive_control(&Canary) -> bool` | `store`, `text`, `vector` |
| `ReadGate` | `blocked(&KeyBuf) -> bool`, `blocked_range(&KeyRange) -> bool`, `epoch() -> u64` | `ErasedSet` (erasure) |
| `RecordSealer` | `seal(doc_tag, aad, plaintext) -> Result<Bytes>`, `open(doc_tag, aad, sealed) -> Result<Zeroizing<Vec<u8>>>`, `forget(doc_tag)` | Tresor (erasure) |
| `LedgerSink` | `append(&LedgerEntry)`, `read_all() -> Vec<LedgerEntry>` | `FileLedger` (erasure), `MemLedger` (testkit) |
| `Metrics` | `counter`, `histogram`, `gauge` (statische Namen, keine ID-Labels) | – |

`ErasureLayer`-Vertrag (normativ): `purge` ist **idempotent**, **kooperativ abbrechbar** (Deadline), verändert nur Daten innerhalb des Matchers (P4) und meldet jeden Fehler als `Err`; `scan_residue` darf Daten nur lesen; `positive_control` beweist, dass der Scanner seinen Canary findet (12.2). Die Conformance-Suite `ports::contract_tests` (Bestand) wird um je eine Suite für `ErasureLayer`, `ReadGate`, `RecordSealer`, `Vfs` erweitert und läuft gegen **jede** Implementierung (echt, Fixture, Fault).
**Delta:** entfernen `attention`, `checkpoint`, `embedding`, `graph`, `graph_index`, `license`, `plugin`, `query_rewriter`, `reranker`, `scanner` (Deklaration), `lifecycle` (nur falls es nicht `Closeable` definiert; sonst auf `Closeable` reduzieren); neu `vfs` (konsolidiert, Bestand `ports/src/vfs.rs` als Basis), `erasure`, `gate`, `sealer`, `ledger`. Verbleibende Reste aus `observability` gehen in `metrics`.
**Tests:** Conformance je Trait; T-ARCH-1.

### 8.3 `contextra-sys` (Ring 1, `unsafe`)

**Zweck:** Plattformprimitive. **Module:** `fsync` (`fsync_file`, `fdatasync`, `F_FULLFSYNC` auf macOS), `dir` (`flush_dir`, `rename_durable` mit Verzeichnis-`fsync`), `mem` (`mlock_best_effort`, `munlock`), `perm` (`set_mode`, `check_mode`). **Invarianten:** Jede `unsafe`-Zeile trägt `// SAFETY:` mit Bounds/Alignment/Gültigkeit; öffentliche `fn` ohne `unsafe` darf keinen unsafe-Inhalt mit offenen Vorbedingungen haben (die bekannten Fälle `mmap_readonly`, `reopen_and_dup2` werden `unsafe fn` oder entfernt). **Fehler:** `fsync`-Fehler werden **immer** propagiert. **Tests:** T-UNSAFE-1, `miri` auf Insel-Tests, Fehlerinjektion über Wrapper. **Delta:** `wire`-bezogene Teile entfernen.

### 8.4 `contextra-simd` (Ring 1, `unsafe`)

**Zweck:** Distanzkernel für `vector`. **Umfang:** f32 `l2`, `dot`, `cosine` für Skalar, AVX2, AVX-512 (f32), NEON; Laufzeit-Dispatch über `is_x86_feature_detected!`. **Entfernt:** u8-VNNI-Pfad (`_mm512_dpbusd_epi32`). **Invarianten:** Jeder Kernel liefert für identische Eingaben das Ergebnis der Skalar-Referenz innerhalb ε = 1e-4 (relativ) bei Vektoren mit Norm ≤ 10³; Längen ungleich 8/16 werden mit Skalar-Rest behandelt; keine Out-of-Bounds-Lesezugriffe (Tail-Handling). **Tests:** T-SIMD-1 (Property, alle Dispatch-Pfade, 10⁵ Zufallsvektoren, Längen 1…4096), `miri`. **Benchmarks:** B-SIMD (Durchsatz je Kernel, Regression ≤ 5 %).

### 8.5 `contextra-durable-fs` (Ring 1)

**Zweck:** `RealVfs`, die einzige produktive Implementierung von `Vfs`. **Module:** `real_vfs`, `file` (`VfsFile`), `atomic` (Temp → `fsync` Datei → Rename → `fsync` Verzeichnis), `scrub` (best-effort Überschreiben mit `ZeroFill`, danach `fsync`, danach Unlink; **dokumentiert die Grenze**: auf Flash/COW keine Garantie), `probe` (Startprobe: atomares Rename, Verzeichnis-`fsync`, `O_EXCL`-Lock). **Invarianten:** (a) `rename_durable` ist erst nach Verzeichnis-`fsync` abgeschlossen; (b) kein `let _ =` auf `sync_*`; (c) Dateirechte werden beim Anlegen gesetzt (Dateien 0600, Verzeichnisse 0700) und beim Öffnen geprüft. **Tests:** Conformance `Vfs`, Fehlerinjektion, T-PERM-1. **Delta:** zwei `Vfs`-Traits zusammenführen; `PendingDeleteJournal` nur behalten, wenn `erasure` ihn nicht ersetzt (Entscheidung: **entfernen**, das Löschjournal liegt in `erasure`).

### 8.6 `contextra-crypto` (Ring 1)

**Zweck:** Alle kryptografischen Primitive in einer Typfamilie. **Abhängigkeiten:** `aes-gcm-siv`, `argon2`, `hkdf`, `hmac`, `sha2`, `blake3`, `ed25519-dalek`, `subtle`, `zeroize`, `rand` (nur `OsRng`).

| Modul (Soll) | Inhalt |
|---|---|
| `kdf` | `KdfParams { m_kib, t, p }`, Minimum (65.536, 3, 1), `derive_master(passphrase, salt, params) -> SecretKey32`, `KdfHeader` (serialisiert mit ID und Version) |
| `keys` | `KeySet { k_enc, k_idx, k_wal, k_journal, k_vault, k_unit, k_meta, k_hdr, k_sign_seed }` per HKDF aus `K_master` mit festen Labels (13.1); `commit_key(epoch)` |
| `aead` | `Aead::seal(key, nonce, aad, pt)`, `open(...)`; AES-256-GCM-SIV; Nonce 96 Bit aus `OsRng`; Zähler je Schlüssel (2³² Grenze) |
| `envelope` | `DocDek::derive(k_unit, salt, doc_tag)`, `Envelope::seal/open` (Layout 9.3) |
| `tag` | `blind_tag(k_idx, label, parts) -> [u8;16]` (keyed BLAKE3, Längenpräfix je Teil gegen Mehrdeutigkeit) |
| `wal_crypto` | HMAC-SHA256-Kette je Frame, Sequenzprüfung (Bestand, bereinigt) |
| `audit_chain` | Hash-Kette für Journal und Ledger (Bestand `audit_chain.rs`, auf BLAKE3 und Commitments reduziert) |
| `sign` | Ed25519 `sign`, `verify`; Seed aus `K_sign` |
| `anti_tamper`, `wal_completeness` | Bestand, auf Soll prüfen; behalten, falls von `store` genutzt |

**Invarianten:** (a) Kein Schlüsselmaterial in `Clone`/`Debug`/`Serialize`. (b) Alle Schlüsselpuffer `Zeroizing`. (c) `blind_tag` ist injektiv über Längenpräfixe: `blind_tag(l, [a,b]) ≠ blind_tag(l, [ab])`. (d) Nonce-Wiederverwendung ist strukturell ausgeschlossen (Zufall 96 Bit, Zähler; bei Zählergrenze Fehler). (e) Keine HKDF-Ableitung aus Passphrase (B-15). **Tests:** T-KDF-1, T-ENC-1/2, T-TAG-1, Known-Answer-Tests (offizielle Vektoren für AES-GCM-SIV, HKDF, BLAKE3, Ed25519, Argon2id), `cargo-fuzz` für Envelope- und Header-Parser. **Benchmarks:** B-CRYPTO (seal/open je 1 KiB/64 KiB, Argon2-Dauer bei Standardparametern, `blind_tag`). **Delta:** Module `deletion_proof`, `ed25519_proof`, `kv_cipher`, `kv_segment`, `kv_shredding`, `revocation_log` sind bereits entfernt, **die Deklarationen in `lib.rs` und die Re-Exports (Z. 47–54, 64–79) sind zu streichen**; `KeyManager` auf `KeySet` umbauen; `lru`, `regex`, `bincode`, `ahash` aus den Abhängigkeiten streichen, soweit nach dem Umbau ungenutzt; Features `kv-encryption`, `cloud-egress-guard`, `test-utils` entfernen.

### 8.7 `contextra-mvcc` (Ring 2)

**Zweck:** Snapshots, Sequenzen, SSI-Validierung. **Module:** `seq` (monotone `Seq`), `snapshot_registry` (`register`, `release`, `min_active() -> Option<Seq>`, `revoke_covering(&Matcher)`), `ssi` (`SsiValidator`, `prune_through(bound: Seq) -> Result<usize, MvccError>`), `tx_buffer`. **Invarianten:** (a) kein Sentinel `u64::MAX` als Sequenz (B-11); (b) `prune_through` mit Tombstone-Bit im Eingabewert ist **Fehler**, kein stilles Maskieren; (c) `pruned_through` steigt monoton und überschreitet nie die höchste committete Sequenz; (d) `revoke_covering` markiert alle Snapshots, die einen Matcher-Schlüssel sehen könnten (konservativ: alle Snapshots mit `seq` ≤ `erased_through_seq`), als widerrufen; deren nächste Operation liefert `SnapshotRevoked`. **Tests:** T-SEQ-1/2, Regression „Flush ohne Snapshot, danach Commit gelingt“, loom für Registry, Property gegen `ReferenceModel`. **Delta:** Fix in `ssi.rs:640–652` und Aufrufer `store/src/lsm/ops/compaction.rs:225–231`.

### 8.8 `contextra-store` (Ring 2)

**Zweck:** Transaktionaler LSM-Tree mit WAL, Memtable, SSTables, Manifest, Kompaktion und **Purge-Primitiven**. Der Store ist **blind**: Er kennt weder Dokumente, Texte, Vektoren noch Löschziele, nur `KeyBuf`, `Seq`, Werte-Bytes und `Matcher`.

| Modul (Soll) | Inhalt |
|---|---|
| `codec` | Cluster-Codec (9.1): `encode_data_key`, `encode_loc_key`, `encode_meta_key`, `parse`, `range_of(Target)`; ersetzt `tenant_codec.rs` |
| `scoped` | `TenantScopedStorage` (einziger Zugang, T-ARCH-2): erzwingt Mandanten-Tag, prüft `ReadGate` in `get`, `get_at`, `scan_range`, `scan_range_at` |
| `wal` | `frame` (Länge, Typ, Seq, TxId, Nutzlast, CRC32, HMAC), `io` (Append, Rotate, Seal), `flusher` (Gruppen-Commit), `replay` (**eine** Implementierung), `hmac` |
| `memtable` | Skiplist mit allen Versionen; `remove_range(&Matcher)` (entfernt alle Versionen) |
| `sstable` | `builder`, `reader`, `index`, `bloom`, `block_cache`, Format 9.2 |
| `manifest` | Append-Log mit Einträgen `AddFile`, `RemoveFile`, `ReplaceFiles{old,new}`, `SetSeq`; Schlüsselbereiche je Datei; `fsync` je Eintrag |
| `compaction` | Leveled; `CompactionFilter` (verwirft Einträge in gegateten Bereichen, Port `ReadGate`); `Barrier` |
| `lsm` | `LsmStorage` (implementiert `StorageEngine`, `ErasureLayer`); `ops::{read, write, compaction}` |
| `erase` | `plan(&Matcher)` → Kandidatendateien; `drain`; `rewrite_file`; `swap`; `unlink_verify`; Implementierung von `ErasureLayer` |

**Format-Eigenschaften (normativ):**

- Schlüsselordnung: lexikographisch auf `KeyBuf` (Bytefolge 9.1). Alle Familien eines Dokuments liegen hintereinander.
- Jeder Datenblock ist ein AEAD-Block (AES-256-GCM-SIV unter `K_enc`, AAD = `file_id ‖ block_index ‖ format_version`). Es gibt **keinen** Klartext-Blockpfad. Der Indexbereich (letzter Schlüssel je Block) und der Bloom-Filter enthalten nur blinde Tags und stehen ebenfalls unter AEAD; nur Header und Footer sind Klartext (Magic, Version, Algorithmus-IDs, Dateigröße, MAC über Footer).
- Werte sind Envelopes (9.3); der Store behandelt sie als opake Bytes.
- WAL-Frames: HMAC-SHA256-Kette (`K_wal`), Sequenzprüfung, CRC32 für Torn-Write-Erkennung. `WalVersion::V1` (ohne HMAC) ist `Integrity::LegacyFormat`.

**Schreibpfad:** `begin` → `put*` → `commit`: Frame(s) → Gruppen-Commit (Leader/Follower, I-14) → `sync_data` WAL → Memtable → Sichtbarkeit. Reihenfolge ist WAL → Memtable → Sichtbarkeit (I-1). Der Follower-Commit ist cancel-sicher: Der Commit läuft in einer eigenen Task, der Aufrufer hält ein `oneshot` (T-CANCEL-1).

**Purge-Primitive (Funktionsinventar FI-S20..S27, Ablauf in 11.4):**

1. `Barrier::exclusive(deadline)` zwischen Kompaktion (shared) und Purge; Warten ist fristgebunden.
2. `drain`: `rotate_and_seal`, Flush aller (auch immutable) Memtables, Entfernen versiegelter WAL-Dateien, deren Inhalt vollständig in SSTables **und** in allen abgeleiteten Indexen (`derived_durable_through`) liegt; `Vfs::exists` prüft das Fehlen.
3. `plan`: Kandidaten = Dateien, deren Schlüsselbereich einen Matcher-Bereich oder einen exakten Schlüssel überlappt; Bloom nur als Vorfilter, exakte Bestätigung per Stream.
4. `rewrite_file`: Streaming (Speicher ≤ 4 × Blockgröße), `create_like` (gleiche Formatversion, Schlüsselkontext), alle Einträge im Matcher (alle Versionen, Tombstones) verworfen, übrige Einträge **mit Originalwerten** `seq`, `tx_id`, `flags` (I-2). Schreibfehler → Abbruch, Temp-Datei als Waise, alter Reader bleibt gültig.
5. `swap`: Temp-Datei `sync_all` → Manifest `ReplaceFiles{old,new}` + `sync_data` → Verzeichnis-`sync_dir` → Reader tauschen → Blockcache-Einträge der alten Datei entfernen und `zeroize`n → alte Datei entfernen → `sync_dir` → Fehlen prüfen (I-3, I-4).
6. Ergebnis `PurgeOutcome { files_rewritten, bytes_written, wal_segments_removed, residue_hint }`.

**Aufräumen (Delta):** Entfernen: `#![allow(unexpected_cfgs)]` (`lib.rs:45`), alle sieben `wal-integrity`-Gates in `wal/replay.rs`, alle `encryption-at-rest`-Gates und die zugehörigen Tests (`tests/no_encryption_feature.rs`), Features `legacy-wal-key`, `docid-128`, `block-cache-v2`, `sieve-cache` (entweder festlegen oder entfernen: **entfernen**, der Standard-Cache bleibt), `memory-only-storage` (Ersatz: `InMemoryStore` in `testkit`), Dev-Abhängigkeiten auf `graph` und `sandbox`; `Option<Arc<KeyManager>>` im `SstableBuilder` wird `Arc<KeySet>`; `HashMap/HashSet` in `manifest/core.rs` (Z. 284, 338–342) auf `BTreeMap/BTreeSet`; `expect` im Scanner entfällt mit dem Scanner. **Prüfauftrag (WP-2.9):** `compaction/adaptive.rs`, `system_pressure.rs`, `lsm/observer.rs`, `engine/memory_only.rs` bleiben nur, wenn `cargo tree` und Aufrufanalyse einen Kernkonsumenten belegen; sonst entfernen.
**Fehlerverhalten:** Jede I/O-Fehlerstelle propagiert `Io`. CRC-Mismatch am WAL-Ende ist Torn-Write (Abschneiden bis zum letzten gültigen Frame, protokollieren); Mismatch mitten in der Datei ist `Integrity`, kein Abschneiden.
**Ursache CRC-Fehler Offset 4 (verschlüsselter Replay):** Beim WAL-Replay mit Verschlüsselung tritt laut Befundlage ein CRC-Fehler bei Offset 4 auf; die Ursache ist **unbekannt** und wird in einer Zeitbox von 2 Wochen lokalisiert (PLAN WP-2.4). Der Soll-Zustand ist unabhängig davon: Frame-Layout 9.2 mit einheitlicher Reihenfolge Längenfeld → Nutzlast → CRC → HMAC, Test T-WAL-1 an jedem Byte-Offset.
**Tests:** T-DIFF-1, T-CRASH-1..4, T-WAL-1/2, T-SEQ-1/2, T-CANCEL-1, T-CONC-1, T-ENC-1/2, T-KEY-1, T-COV-1, T-DROP-1, Fuzz (WAL-Frame, SSTable-Index, Manifest, Codec), loom (Barrier, Gruppen-Commit). **Benchmarks:** B-INSERT, B-GET, B-SCAN, B-WAL, B-SST, B-ERASE-WA, B-RECOVER, B-VS-REDB.

### 8.9 `contextra-erasure` (Ring 3)

**Zweck:** Der entkoppelte Löschkern. **Abhängigkeiten:** `types`, `ports`, `crypto`, `serde_json`, `tokio` (Tasks, Timer). **Kennt nicht:** `store`, `text`, `vector`, `mvcc`.

| Modul | Inhalt | Funktionen |
|---|---|---|
| `scope` | Auflösung `EraseTarget` → `Matcher` (Bereiche + exakte Schlüssel) und `doc_tag`-Menge über `StorageEngine`-Port (Locator-/Cluster-Lesen) | FI-E01 |
| `journal` | Append-only `journal.log`: Länge, Typ, Nutzlast (AEAD `K_journal`), CRC, Vorgänger-Hash; `fsync` je Eintrag; Reduktion nach `Completed` | FI-E02..E04 |
| `erased_set` | `erased.set`: aktive Gates (Bereiche, Commitment, `gated_seq`); atomarer Austausch; implementiert `ReadGate` | FI-E05..E07 |
| `vault` | Tresor: Slots, Memory-Index, DEK-LRU, `create_slot`, `dek`, `destroy`, `compact`; implementiert `RecordSealer` | FI-E08..E13 |
| `saga` | Zustandsmaschine, Schrittausführung, Idempotenz, Wiederholung | FI-E14..E17 |
| `scheduler` | Purge-Fenster, Deduplikation nach Commitment, `Timing::Now`, SLA-Alarm | FI-E18 |
| `measure` | Orchestrierung von E1/E2/E3 über `ErasureLayer::scan_residue`, `positive_control` | FI-E19..E21 |
| `classify` | Reine Funktion `Evidence → ErasureClass` | FI-E22 |
| `report` | Bericht bauen, kanonisieren, signieren | FI-E23..E25 |
| `ledger` | Abgeschlossene Löschungen für den Restore-Guard; `LedgerSink` | FI-E26..E28 |
| `recover` | Wiederanlauf (11.7) | FI-E29 |
| `checklist` | Textexport (Abschnitt 16) | FI-E30 |

**Invarianten:** I-1 … I-14 (11.10). **Fehlerverhalten:** Jeder Fehler in einer Phase wird als `Pending { attempt, next_retry_at }` persistiert (Backoff 1 s, 2 s, 4 s … Obergrenze 15 min), nie verschluckt. **Tests:** T-VAULT-*, T-CRASH-*, T-REPORT-1, T-PROOF-1, T-CLASS-1, T-MUT-1, loom (Gate/Journal), Fuzz (Journal-Parser, Ledger-Parser, Report-Parser). **Benchmarks:** B-ERASE-LAT, B-VAULT, B-READGATE, B-REPORT.

### 8.10 `contextra-vector` (Ring 2)

**Zweck:** HNSW-Vektorsuche mit löschbarer Persistenz. **Module (Soll):** `arena` (Slots fester Größe, f32), `hnsw` (Aufbau, Suche; `deletion.rs` und altes `purge.rs` entfallen), `docmap` (`doc_tag ↔ slot`), `snapshot` (Chunk-Dateien, 9.8), `rebuild`, `layer` (`ErasureLayer`), `distance` (nutzt `simd`). Entfernen: `acorn`, `diskann`, `quantize*`, `partial_rebuild`, `candidate_stream`, `compute_pool` (nur behalten, falls ein Kernkonsument belegt ist; Standard: entfernen).

**Verhalten:** Beim Erase eines Dokuments: `zero_slot` überschreibt die Slot-Bytes mit `zeroize`, markiert Tombstone, entfernt `DocMap`-Eintrag; Suche überspringt Tombstones. Das Purge-Fenster schreibt die betroffenen Snapshot-Chunks und die Graph-Datei (gefiltert um Tombstone-Slots) neu. Rebuild bei Tombstone-Quote ≥ `vector_rebuild_ratio`. **Invarianten:** (a) Vektoren verlassen den Prozess nur verschlüsselt; (b) Slot-Wiederverwendung erst nach Zeroing **und** Snapshot-Rewrite; (c) Suchergebnisse enthalten nie Tombstones; (d) `DocMap` und Arena sind nach Erase und Rebuild konsistent (T-DOCMAP-1). **Tests:** T-VEC-1 (Recall@10 ≥ 0,95 gegen Brute-Force auf Testkorpus), T-VEC-2 (Erase: Slot nullen, Chunk-Rewrite, Rohscan), T-RAW-3, T-SIMD-1, T-DOCMAP-1, Fuzz (Snapshot-Parser). **Benchmarks:** B-VEC-BUILD, B-VEC-SEARCH, B-VEC-ERASE.

### 8.11 `contextra-text` (Ring 2)

**Zweck:** BM25-Segmentindex (D-20). **Module (Soll):** `tokenizer` (NFKC, Kleinschreibung, Unicode-Wortgrenzen UAX #29), `buffer` (Textpuffer), `segment` (Format 9.7, `SegmentWriter`, `SegmentReader`), `merge`, `bm25`, `wand` (Bestand, bleibt nur bei Gleichheit mit exhaustivem BM25 im Test T-TEXT-2), `layer` (`ErasureLayer`), `docseg` (Zuordnung `doc_tag → segment_id`). Entfernen: `morphology`, `morph_index`, `data`, `domain`, `posting_list` und `stream` (sofern nicht vom Segmentformat genutzt), alle LSM-Schlüssel (`plb`, `fw`, `tbs`).

**Verhalten:** Neue Dokumente gelangen in den Textpuffer, dessen Flush ein Segment schreibt (Zielgröße 32 MiB). Segmente sind unveränderlich; `merge` fasst kleine Segmente zusammen. Erase: Dokument aus Puffer entfernen, betroffene Segmente neu schreiben ohne die Ziel-Dokumente, Statistik neu berechnen, Manifest tauschen, Alt-Segment entfernen und Fehlen prüfen. **Invarianten:** (a) kein Termklartext auf Platte (nur `term_tag`); (b) jedes Segment ist AEAD-verschlüsselt, Segmentschlüssel `HKDF(K_enc, "ctx/text/seg/v1" ‖ segment_id)`; (c) globale Statistik (N, Σlen, df) wird aus Segmentheadern berechnet, es gibt keine persistenten globalen Zähler; (d) ein Dokument liegt in höchstens einem Segment. **Tests:** T-TEXT-1 (BM25 gegen Referenzimplementierung, Rangfolge identisch), T-TEXT-2 (WAND = exhaustiv), T-TEXT-3 (Erase: Segment-Rewrite, Rohscan E1/E2, Statistik exakt), T-RAW-4, Fuzz (Segment-Parser). **Benchmarks:** B-TEXT-BUILD, B-TEXT-QUERY, B-TEXT-ERASE.

### 8.12 `contextra-engine` (Ring 3)

**Zweck:** Orchestrierung ohne eigene Löschlogik. **Module (Soll):**

| Modul | Inhalt |
|---|---|
| `wiring` | Baut aus `store`, `text`, `vector`, `erasure`, `vfs`, `clock`, `rng` den Laufzeitgraphen; startet Hintergrundtasks; implementiert `open`/`close` |
| `collection` | `Collection`: `insert`, `insert_batch`, `get`, `get_at`, `scan`, `scan_at`, `delete` (logisch), Suche; Dokumentlock über die **gesamte** Operation (behebt B-13) |
| `doc` | Dokumentzerlegung in Records (`Doc`, `DocMap`, `Manifest`, `Locator`), Envelope-Sealing |
| `search` | `search_text`, `search_vector`, `search_hybrid` (ReadGate-Filter, RRF, Metadatenfilter nachgelagert) |
| `transaction` | `DbTransaction`: Store-Transaktion + Tresor-Slots + Puffer-Vormerkung atomar; Rollback |
| `recovery` | Wiederanlauf der abgeleiteten Indexe aus dem WAL-Rest (`derived_durable_through`) |
| `erase_bridge` | Verdrahtet `ErasureCore` mit den Schichten (`LsmStorage`, `TextIndex`, `VectorIndex`) und stellt `erase`, `erasure_status`, `verify` bereit |
| `metrics_names`, `diagnostics` | Metriken ohne IDs |

**Entfernen:** `chunker`, `extraction`, `export`, `import`, `decay_controller`, `temporal_filter`, `merge_operator_adapter`, `background_workers` (Ersatz: in `wiring`), `contextra_impl/*` (Ersatz: `wiring`, `collection`), alle 103 Verwendungen von `contextra_graph`, `community_detection`, `checkpoint`, `privacy`, `license`, `cognition`; Features `graph-connectivity-health`, `sandbox`, `onnx`, `adaptive-candidate-pool-sizing`; `TenantPolicy` (nur noch Pflicht, D-07). **Invarianten:** (a) Ein Insert schreibt `Doc`, `DocMap`, `Manifest`, `Locator` in **einer** Store-Transaktion; Tresor-Slot ist vorher dauerhaft (11.5 Schritt 0); (b) jeder Lesepfad prüft `ReadGate` vor Entschlüsselung; (c) `delete` und `erase` teilen den Dokumentlock. **Tests:** T-ISO-*, T-VER-1, T-COV-1, T-DOCMAP-1, End-to-End-Szenarien (Anhang B), T-CONC-1. **Benchmarks:** B-HYBRID, B-E2E-INSERT.

### 8.13 `contextra-rank` (Ring 2)

**Zweck:** RRF. **API:** `rrf_fuse(lists: &[Vec<Hit>], k: u32 = 60, top: usize) -> Vec<Hit>`; deterministische Tie-Breaks nach `doc_tag`. Entfernen: `IsotonicCalibrator`, Ranking-Strategien, Adaptivität, Abhängigkeit auf `adapt`. **Tests:** T-RANK-1 (Property: Permutationsinvarianz der Eingabelisten, Determinismus; Referenzwerte per Hand). **Benchmarks:** B-RRF.

### 8.14 `contextra` (Ring 4, Fassade)

**Zweck:** Einzige öffentliche Oberfläche (Abschnitt 7). Re-exportiert nur API-Typen. `system_defaults()` liefert `RealVfs`, `SystemClock`, `OsRng`-basierten `Rng`. **Entfernen:** Abhängigkeiten auf `license`, `privacy`, `audit-export`, `avv-generator`, `infer-candle`; Features `audit-export`, `avv-generator`, `candle`; `performance_profile`, `collection_profile`, wenn sie Entfernte referenzieren (sonst auf `CollectionSpec` reduzieren). **Tests:** T-API-1 (trybuild), Doc-Tests, `examples/quickstart.rs`, `examples/collections.rs`, `examples/hybrid_search.rs`, **neu** `examples/erase_and_verify.rs`, `examples/governance_fixtures.rs` (Bestand, an Soll anpassen).

### 8.15 `contextra-testkit` (dev)

**Zweck:** Wiederverwendbare Testbausteine. **Bestand:** `fault_vfs.rs`, `reference_model.rs`, `manual_clock.rs`, `in_memory_*`, `mocks.rs`, `conformance/`. **Soll:** `FaultVfs` erweitert um: Absturz nach Operation *n* (alle `Vfs`-Methoden), abgerissener Write an Byte *b*, `ENOSPC`, `EIO` bei `sync_*`, verzögerte Sichtbarkeit (Simulation Seitencache), Aufzeichnung jeder Operation; `ReferenceModel` (BTreeMap-Orakel mit Zeitreisen und Löschsemantik); `RecordingStorage` (protokolliert jeden geschriebenen Schlüssel je öffentlicher Operation, T-COV-1); `NeighborDigest` (Hash aller Nicht-Ziel-Paare über alle Dateien und Versionen); `LeakyLayer` (absichtlich fehlerhafte `ErasureLayer`-Implementierung für T-RAW-2); `MemVfs`, `MemLedger`; `CanaryFactory` (hochentropische Needles pro Lauf, Seed protokolliert, nur Testmandant). Entfernen: `in_memory_graph`.

### 8.16 `contextra-lab` (dev, unabhängige Prüfinstanz)

**Zweck:** Beweist die Löschung **ohne Vertrauen in den Produktionscode**. Eigene, vom Produktionscode getrennte Implementierungen (AR-4): Formatleser für WAL-Frames, SSTable-Blöcke, Textsegmente, Vektor-Chunks, Journal, Tresor; eigene AES-256-GCM-SIV-Entschlüsselung (anderes Crate als die Produktion, z. B. RustCrypto `aes-gcm-siv` in eigener Version oder ein zweiter Implementierungsweg über `ring`-freie Referenz). **Module:** `decoder`, `scan_e1` (Rohscan), `scan_e2` (Decoder-Scan), `scan_e3` (Schlüsselabwesenheit und Entschlüsselungsprobe), `heap` (Linux: `/proc/self/mem`), `scenario` (Workloads: Insert, Update, Erase, Restart, Crash), `report_check`. **Aufruf:** nur in Tests, Benchmarks und der CI-Messkampagne. **Tests:** Selbsttest T-RAW-2 (`LeakyLayer` wird gefunden, sauberes System besteht), Kreuzprüfung Lab-Decoder gegen Produktions-Reader auf 10⁴ zufälligen Dateien (muss identische Klartexte liefern, sonst ist der Lab-Decoder defekt).

### 8.17 `ctx-verify` (Binary)

**Zweck:** Eigenständiger Berichtsprüfer. **Abhängigkeiten:** nur `ed25519-dalek`, `serde_json`, `blake3`; **keine** Contextra-Crates. **Funktion:** `ctx-verify <report.json> [--pubkey <hex>]` prüft Signatur, Feldschema (unbekannte Felder abgelehnt), kanonische Serialisierung, Klassenableitung aus den Zeugen (eigene Implementierung von `classify`), Restrisikoliste vorhanden, Pflichtschichten vorhanden. Exit-Code 0 = gültig, 1 = ungültig, 2 = Eingabefehler. Testvektoren im Repo (`testvectors/reports/`). **Tests:** T-PROOF-1 (jedes Byte), Kreuzprüfung `classify` Produktion gegen `ctx-verify` auf 10⁵ Zufalls-Evidenzen.

### 8.18 `tests/arch`

Architektur- und Lint-Tests: T-ARCH-1..4, T-LINT-1/2, T-UNSAFE-1, T-CFG-1/2, T-SUPPLY-1. Parst `cargo metadata` (Ringe, Matrix 5.3), `cargo tree` (Netzwerk-Crates), Quelltext (verbotene Muster in Produktionspfaden, undeklarierte `feature = "…"`, `#[allow]` ohne Ledgereintrag, Wortlaut). Bestand `tests/layering.rs` und `tests/unsafe_islands.rs` werden integriert.

---

## 9. Datenmodell und Formate

Alle Formate tragen `magic`, `format_version: u16` und Algorithmus-IDs (D-42). Unbekannte Version oder ID ist `Integrity::UnknownFormat` (fail-closed). Ein älteres Format wird **nicht** gelesen (P3).

### 9.1 Schlüsselraum (Löschcluster-Schlüsselraum, D-31)

```
DATA = 0x01 ‖ tenant_tag[16] ‖ coll_tag[16] ‖ cluster_tag[16] ‖ doc_tag[16] ‖ family[1] ‖ sub[0..]
LOC  = 0x02 ‖ tenant_tag[16] ‖ coll_tag[16] ‖ doc_tag[16]
META = 0x03 ‖ tenant_tag[16] ‖ coll_tag[16] ‖ name_tag[16] ‖ family[1]
```

| Tag | Berechnung (`BLAKE3-keyed(K_idx, label ‖ lp(teil₁) ‖ lp(teil₂) …)[0..16]`, `lp` = 4-Byte-Längenpräfix) |
|---|---|
| `tenant_tag` | `"ctx/tenant/v1"`, tenant |
| `coll_tag` | `"ctx/coll/v1"`, tenant, collection |
| `doc_tag` | `"ctx/doc/v1"`, tenant, collection, doc_key |
| `cluster_tag` (Betroffener) | `"ctx/cluster/subject/v1"`, tenant, subject_id (gleich über alle Collections; `coll_tag` steht davor) |
| `cluster_tag` (ohne Betroffenen) | `"ctx/cluster/solo/v1"`, tenant, collection, doc_key |
| `name_tag` | `"ctx/name/v1"`, tenant, Namensteil |
| `term_tag` | `"ctx/term/v1"`, tenant, collection, term |

`coll_tag = 0x00…00` ist der mandantenweite META-Namensraum (Collection-Verzeichnis, Betroffenenverzeichnis).

**DATA-Familien (`family`):**

| Code | Name | Inhalt (vor Sealing) | Wert |
|---|---|---|---|
| 0x01 | `Doc` | `doc_key`, `subject_id?`, `text?`, `payload`, `metadata`, `created_at`, `updated_at` | Envelope (DEK) |
| 0x02 | `DocMap` | `slot: u32?`, `text_segment: u64?`, `version: u32` | Envelope (DEK) |
| 0x03 | `Manifest` | Liste aller exakten Schlüssel des Dokuments (Prüfzweck, T-COV-1) | Envelope (DEK) |

**LOC:** Wert = Envelope(DEK) über `cluster_tag[16]`. **META-Familien:** `0x10 CollectionSpec` (`vector_dim`, `text_index`, Format), `0x11 CollectionList`, `0x20 SubjectDir` (`name_tag = subject_tag`; Wert: sortierte Liste `coll_tag` + Zähler), `0x30 StoreStats`; Werte unter `K_meta`.

**Löschbereiche je Ziel (alle Matcher bestehen aus Bereichen fester Tag-Länge und exakten Schlüsseln, D-10):**

| Ziel | Bereiche | Exakte Schlüssel |
|---|---|---|
| Document | `DATA ‖ t ‖ c ‖ cluster ‖ doc` | `LOC ‖ t ‖ c ‖ doc` |
| Subject | je Collection aus `SubjectDir`: `DATA ‖ t ‖ c ‖ cluster` | je Dokument `LOC ‖ t ‖ c ‖ doc` (aus `DocMap`-Scan des Clusters), `META ‖ t ‖ 0 ‖ subject_tag ‖ 0x20` |
| Collection | `DATA ‖ t ‖ c`, `LOC ‖ t ‖ c`, `META ‖ t ‖ c` | `SubjectDir`-Einträge werden aktualisiert (Zähler), bei Zähler 0 entfernt |
| Tenant | `DATA ‖ t`, `LOC ‖ t`, `META ‖ t` | – |

`scan` iteriert über die LOC-Region in `doc_tag`-Reihenfolge (D-41).

**Insert-Ablauf:** (1) `doc_tag` bilden; (2) `LOC` lesen: existiert und entschlüsselter `doc_key` (aus `Doc`) ≠ neuer Schlüssel → `Conflict::IdCollision`; gleich → Update (Betroffener gleich, sonst `SubjectImmutable`); (3) `cluster_tag` bestimmen; (4) Tresor-Slot anlegen (neu) und **dauerhaft machen**; (5) eine Store-Transaktion mit `Doc`, `DocMap`, `Manifest`, `LOC`, ggf. `SubjectDir`; (6) Text- und Vektorpuffer vormerken. Ein Dokument ohne vollständiges `Manifest` und `LOC` existiert für das Löschen nicht und wird beim Recovery verworfen.

### 9.2 WAL und SSTable

**WAL-Datei:** Header `magic "CTXWAL01" | format_version u16 | aead_id u8 | mac_id u8 | wal_id u64 | first_seq u64 | prev_file_chain[32]`. Frame:

```
len u32 | type u8 | seq u64 | tx_id u64 | payload (AEAD, K_enc, AAD = wal_id‖seq) | crc32 u32 (über type..payload) | hmac[32] = HMAC-SHA256(K_wal, prev_hmac ‖ len‖type‖seq‖tx_id‖payload‖crc)
```

Frame-Typen: `TxBegin`, `Put`, `Delete`, `TxCommit`, `TxAbort`, `SetSeq`. Sichtbarkeit entsteht erst mit `TxCommit`. Torn-Write-Regel: ungültiger letzter Frame wird abgeschnitten; ein ungültiger Frame vor gültigen Frames ist `Integrity`.

**SSTable-Datei:** `Header (klar): magic "CTXSST01" | format_version | aead_id | hash_id | file_id u64` → `Datenblöcke` → `Indexblock` → `Bloomblock` → `Footer`. Block = `nonce[12] | ciphertext | tag[16]`, Klartextgröße 16 KiB, AAD = `file_id ‖ block_index ‖ format_version`. Eintrag im Block: `klen u16 | key | seq u64 | tx_id u64 | flags u8 | vlen u32 | value`. `flags`: Bit 0 Tombstone. Footer (AEAD): Blockoffsets, Indexoffset, `min_key`, `max_key`, Seq-Spanne, BLAKE3 über alle Klartextblöcke. Magic, Version, IDs und `file_id` sind die einzigen Klartextfelder.

**Manifest-Log:** Einträge `AddFile{id,level,min,max,seq_range}`, `RemoveFile{id}`, `ReplaceFiles{old[],new[]}`, `SetSeq`; je Eintrag `len | AEAD(K_enc) | crc | chain = BLAKE3(prev ‖ entry)`; `sync_data` je Eintrag.

### 9.3 Record-Envelope (D-32)

```
Envelope = version u8 (=1) | aead_id u8 (=1) | nonce[12] | ciphertext | tag[16]
AAD      = format_version ‖ vollständiger Schlüssel (key_bytes) ‖ family
DEK      = HKDF-SHA256(ikm = K_unit, salt = vault_salt[32], info = "ctx/dek/v1" ‖ doc_tag)
```

Der AAD bindet den Wert an seinen Schlüssel: Vertauschen von Werten zwischen Schlüsseln, Familien oder Mandanten schlägt fehl (T-ENC-2). Über dem Envelope liegt das Block-AEAD (Doppelschicht).

### 9.4 Tresor (`vault/vault.dat`, D-33)

```
Header (64 B): magic "CTXVLT01" | format_version u16 | aead_id u8 | slot_size u16 (=80) | slot_count u64 | epoch u64 | header_mac[16]
Slot (80 B):   state u8 (0 Free, 1 Active) | reserved[3] | doc_tag[16] | nonce[12] | wrapped_salt[32] | tag[16]
               wrapped_salt = AES-256-GCM-SIV(K_vault, nonce, AAD = slot_index ‖ doc_tag ‖ epoch, salt[32])
```

Im Speicher: `BTreeMap<DocTag, u32>` (Slot-Index), freie Slots als Liste, DEK-LRU (4.096 Einträge, `Zeroizing`). Slot nullen heißt: alle 80 Byte = `0x00`.

### 9.5 Journal, Gate-Menge, Ledger

**Journal** (`erasure/journal.log`): Eintrag `len u32 | type u8 | erasure_id[16] | seq u64 | ts i64 | payload (AEAD K_journal, AAD = Header) | crc32 | chain[32] = BLAKE3(prev_chain ‖ header ‖ payload)`. Typen: `Requested`, `Gated`, `KeyDestroyIntent`, `KeyDestroyed`, `Drained`, `Rewritten`, `Unlinked`, `IndexesCleaned`, `Measured`, `Completed`, `PendingSet`, `Refused`, `Restricted`, `Lifted`, `SnapshotRevoked`, `SlaBreached`, `RestoreReapplied`, `EpochRotated`. Nach `Completed` wird der Eintragssatz einer Löschung auf die **Minimalform** reduziert: `erasure_id`, `completed_at`, `class`, `commitment`. Reduktion = atomarer Austausch der Datei (Temp → `sync_all` → Rename → `sync_dir`), danach Entfernen der alten Datei und Fehlen prüfen.

**Gate-Menge** (`erased.set`): AEAD-Datei (`K_journal`) mit sortierten Einträgen `{erasure_id, kind: Gated|Restricted, ranges[], exact[], commitment, gated_seq, erased_through_seq}`; Austausch atomar. Enthält **nur aktive** Gates. Ein Gate wird mit `Completed` entfernt, ein `Restricted`-Gate mit `Lifted`.

**Ledger** (`erasure/ledger.log` und `LedgerSink`): Hash-gekettete Einträge `{generation u64, completed_at, commitment, ranges[] (Tags), epoch}`; AEAD (`K_journal`). Lebensdauer = Commitment-Epoche (D-48).

**Commitment:** `commitment = BLAKE3-keyed(K_commit(epoch), canonical(target_desc))[0..16]` mit `K_commit(e) = HKDF(K_master, "ctx/commit/v1" ‖ e)`; `epoch = floor(completed_at / commit_epoch)`.

### 9.6 Bericht

Kanonisches JSON (UTF-8, Schlüssel sortiert, keine Gleitkommazahlen, keine Leerzeichen), Schema in Anhang D. Dateiname `erasure/report/<report_id>.json`.

### 9.7 Textsegment (`text/<coll_tag_hex>/seg-<id>.seg`, D-20, D-49)

```
Header (klar):  magic "CTXSEG01" | format_version | aead_id | segment_id u64
Körper (AEAD, K_seg = HKDF(K_enc, "ctx/text/seg/v1" ‖ segment_id), 64-KiB-Blöcke):
  DocTable : n × { doc_tag[16], doc_len u32 }                (sortiert nach doc_tag)
  TermDict : m × { term_tag[16], df u32, postings_off u64 }  (sortiert nach term_tag)
  Postings : je Term Blöcke zu 128 Einträgen { doc_idx (Delta, varint), tf (varint) }, je Block max_tf (für WAND)
  Stats    : doc_count u64, total_len u64
Footer (AEAD): Abschnittsoffsets | BLAKE3 über Klartextabschnitte
```

Das Segmentverzeichnis je Collection (`text/<coll_tag_hex>/SEGMENTS`) ist ein AEAD-Log wie das Manifest. `DocMap.text_segment` verweist auf das Segment des Dokuments. Globale BM25-Werte entstehen zur Abfragezeit aus den Segment-Stats (Summe über Segmente). Kein persistenter globaler Zähler existiert.

### 9.8 Vektorsnapshot (`vector/<coll_tag_hex>/`, D-19)

```
chunk-<n>.vec : Header (klar: magic "CTXVEC01", format_version, aead_id, dim u16, chunk_no u32, slots_per_chunk=4096)
                je Slot: slot u32 | doc_tag[16] | nonce[12] | ciphertext (dim×4 Byte, unter DEK(doc_tag)) | tag[16]
graph.hnsw    : AEAD (K_enc): Ebenen, Nachbarlisten (nur Slot-Nummern), Einstiegspunkt, M, ef_construction
SNAPSHOT      : AEAD-Log: Chunk-Liste, Tombstone-Bitmap, generation, Prüfsummen
```

Atomarer Austausch je Chunk wie bei SSTables (Temp → `sync_all` → Log-Eintrag → `sync_dir` → Rename/Unlink → Fehlen prüfen). Die Arena im RAM hält Klartextvektoren (für Distanzen); beim Laden entschlüsselt der Index jeden Slot mit dem DEK aus dem Tresor. Fehlt der Tresor-Slot, bleibt der Slot Tombstone (Klasse K wirkt sofort).

### 9.9 Datenverzeichnis-Header (`HEADER`)

`magic "CTXHDR01" | format_version | kdf_id | m_kib u32 | t u32 | p u32 | salt[32] | store_id[16] | created_at | aead_id | hash_id | check (AEAD(K_hdr, "ctx/check")) | header_mac[16]`. Falsche Passphrase: `check` schlägt fehl → `Auth`. KDF-Parameter unterhalb des Minimums: `Config`.

---

## 10. Funktionsinventar

Jede Zeile ist eine testbare Funktion mit ID. **Pre** = Vorbedingung, **Post** = Nachbedingung, **Err** = Fehlerverhalten. Alle Funktionen sind `async`, wo sie I/O ausführen. Das Inventar ist abschließend: Funktionen, die hier nicht stehen, sind nicht Teil des v1-Produkts.

### 10.1 `types` und `crypto`

| ID | Funktion | Pre / Post / Err | Test |
|---|---|---|---|
| FI-T01 | `KeyBuf::data/loc/meta(parts) -> KeyBuf`, `parse(bytes) -> Result<Parsed>` | Roundtrip bijektiv; falsche Länge → `Err(KeyFormat)` | T-KEY-1 |
| FI-T02 | `Matcher::normalize()` | Bereiche sortiert, disjunkt, exakte Schlüssel nicht in Bereichen enthalten | T-KEY-1 |
| FI-T03 | `Matcher::contains(&KeyBuf)` | genau dann wahr, wenn in Bereich oder exakt | T-ISO-2 |
| FI-T04 | `ErasureClass::display()` | nur Texte aus 12.5 | T-LINT-1, T-CLASS-1 |
| FI-C01 | `derive_master(pass, salt, params)` | Parameter ≥ Minimum, sonst `Config`; Ergebnis `Zeroizing` | T-KDF-1 |
| FI-C02 | `KeySet::from_master(k)` | feste Labels, 9 Schlüssel, deterministisch | T-KEY-3 |
| FI-C03 | `Aead::seal/open` | `open` mit falschem AAD/Schlüssel → `Err(Integrity)` | T-ENC-1/2 |
| FI-C04 | `DocDek::derive(k_unit, salt, doc_tag)` | deterministisch; verschiedene Salts → verschiedene DEKs | T-VAULT-1 |
| FI-C05 | `Envelope::seal/open(dek, key_bytes, family, pt)` | AAD gebunden an Schlüssel | T-ENC-2 |
| FI-C06 | `blind_tag(k_idx, label, parts)` | injektiv über Längenpräfixe, Ausgabe 16 Byte | T-TAG-1 |
| FI-C07 | `WalChain::next(frame)` / `verify(frame)` | Sequenz- und HMAC-Prüfung; Abweichung → `Integrity` | T-WAL-1 |
| FI-C08 | `HashChain::append/verify` | jede Manipulation bricht die Kette | T-JRN-1 |
| FI-C09 | `sign(seed, bytes)`, `verify(pk, bytes, sig)` | Ed25519; Bytefehler → `false` | T-PROOF-1 |

### 10.2 `durable-fs`, `mvcc`, `store`

| ID | Funktion | Pre / Post / Err | Test |
|---|---|---|---|
| FI-F01 | `Vfs::write_atomic(path, bytes)` | Temp → `sync_all` → Rename → `sync_dir`; Fehler propagiert | T-CRASH-2 |
| FI-F02 | `Vfs::lock_exclusive(dir)` | zweiter Prozess → `Err(Locked)` | T-LOCK-1 |
| FI-F03 | `probe(dir)` | atomares Rename und Verzeichnis-`fsync` funktionieren, sonst `Err` | T-CRASH-3 |
| FI-F04 | `check_permissions(dir)` | Verzeichnis 0700, Dateien 0600, sonst `Err(Permissions)` | T-PERM-1 |
| FI-M01 | `SnapshotRegistry::register/release/min_active` | `Option<Seq>`, kein Sentinel | T-SEQ-2 |
| FI-M02 | `SnapshotRegistry::revoke_covering(&Matcher, erased_through)` | alle betroffenen Snapshots widerrufen | T-VER-1 |
| FI-M03 | `SsiValidator::prune_through(bound)` | Tombstone-Bit gesetzt → `Err`; monoton | T-SEQ-2 |
| FI-S01 | `LsmStorage::open(cfg)` | Replay, Manifest-Check; Integrität unbedingt | T-WAL-1, T-CFG-1 |
| FI-S02 | `begin/put/put_batch/delete/commit/rollback` | WAL → Memtable → Sichtbarkeit; Abbruch des Aufrufers ändert nichts | T-CANCEL-1 |
| FI-S03 | `get`, `get_at(seq)` | `ReadGate` geprüft; gegatet → `Erased` | T-VER-1 |
| FI-S04 | `scan_range`, `scan_range_at` | Reihenfolge nach `KeyBuf`; Gate-Filter | T-SCAN-1 |
| FI-S05 | `flush()` | Memtable → SSTable, Manifest, `sync` | T-CRASH-1 |
| FI-S06 | `compact_step()` | Barrier shared; `CompactionFilter` aktiv | T-CONC-1 |
| FI-S07 | `pin_snapshot/unpin_snapshot` | Registry-Eintrag | T-SEQ-1 |
| FI-S08 | `close()` | Tasks beenden, WAL schließen, Lock lösen | T-DROP-1 |
| FI-S20 | `Barrier::exclusive(deadline)` | Frist überschritten → `Err(Timeout)` → `Pending` | T-CONC-1 |
| FI-S21 | `drain()` | alle Memtables geflusht, WAL-Segmente entfernt, Fehlen geprüft | T-CRASH-1 |
| FI-S22 | `erase::plan(&Matcher)` | Kandidatenmenge ⊇ tatsächlich betroffene Dateien | T-COV-1 |
| FI-S23 | `erase::rewrite_file(src, &Matcher)` | Nicht-Ziel-Einträge bitgleich inkl. `seq`, `tx_id`, `flags` | T-ISO-1, T-VER-2 |
| FI-S24 | `erase::swap(old, new)` | Manifest `ReplaceFiles`, Reihenfolge I-4 | T-CRASH-1 |
| FI-S25 | `erase::unlink_verify(paths)` | Datei weg, sonst `Err` | T-CRASH-1 |
| FI-S26 | `BlockCache::evict_file(id)` | Einträge entfernt und `zeroize`d | T-RAW-3 |
| FI-S27 | `LsmStorage::scan_residue(&NeedleSet)` | Decoder-Scan aller Dateien, Offsets | T-RAW-1/2 |

### 10.3 `erasure`

| ID | Funktion | Pre / Post / Err | Test |
|---|---|---|---|
| FI-E01 | `scope::resolve(target) -> Scope` | Matcher und `doc_tag`-Menge; unbekannt → leer | T-ISO-*, T-SUBJ-1 |
| FI-E02 | `Journal::append(entry)` | `sync_data` vor Rückkehr | T-CRASH-2 |
| FI-E03 | `Journal::open_verify()` | Kette geprüft; defekt → `Err(Integrity)` | T-JRN-1 |
| FI-E04 | `Journal::reduce_completed(id)` | Minimalform; Fehlen der alten Datei geprüft | T-JRN-3 |
| FI-E05 | `ErasedSet::add(entry)` | atomarer Austausch vor Rückkehr | T-CRASH-2 |
| FI-E06 | `ErasedSet::remove(id)` | atomar | T-CRASH-2 |
| FI-E07 | `ErasedSet::blocked(&KeyBuf)` (`ReadGate`) | O(log n); Rückgabe wahr, wenn in Bereich oder exakt | B-READGATE |
| FI-E08 | `Vault::open(path, k_vault)` | Header- und Slot-Prüfung; Index aufbauen | T-VAULT-1 |
| FI-E09 | `Vault::create_slot(doc_tag)` | Salt aus OS-CSPRNG; `sync_data` vor Rückkehr | T-VAULT-2 |
| FI-E10 | `Vault::dek(doc_tag)` | Slot fehlt → `Err(Erased)` | T-VAULT-3 |
| FI-E11 | `Vault::destroy(&[DocTag])` | nullen, `sync_data`, Rücklesen = Nullen, Index und LRU bereinigt | T-VAULT-1..4 |
| FI-E12 | `Vault::compact()` | neue Datei, Austausch, Alt entfernt, Fehlen geprüft | T-VAULT-4 |
| FI-E13 | `RecordSealer::seal/open/forget` | `open` nach `forget` → `Err(Erased)` | T-VAULT-3 |
| FI-E14 | `Saga::start(req)` | Retention prüfen; `Requested` → `Gated` → `KeyDestroyed`; Rückgabe `Gated` | T-SLA-1 |
| FI-E15 | `Saga::step(state)` | idempotent; Fehler → `Pending` | T-CRASH-1 |
| FI-E16 | `Saga::resume_all()` | offene Läufe in ihrem Zustand fortsetzen | T-CRASH-1 |
| FI-E17 | `Saga::restrict/lift` | `Restricted` ohne Vernichtung | T-RESTR-1 |
| FI-E18 | `Scheduler::plan_window()` | bündelt nach Commitment; `Now` sofort | B-ERASE-BATCH |
| FI-E19 | `measure::run(layers, scope)` | E1/E2/E3, Positivkontrolle zuerst | T-RAW-1..5 |
| FI-E20 | `measure::positive_control(layer)` | Canary im Testbereich gefunden, sonst Lauf ungültig | T-RAW-2 |
| FI-E21 | `measure::key_absence(vault, tags)` | Slot null, Entschlüsselungsprobe scheitert | T-VAULT-3 |
| FI-E22 | `classify(evidence) -> ErasureClass` | rein, monoton | T-CLASS-1, T-MUT-1 |
| FI-E23 | `report::build(evidence)` | enthält Restrisiken, keine Inhalte | T-REPORT-1 |
| FI-E24 | `report::canonicalize(&Report) -> Vec<u8>` | deterministisch | T-PROOF-1 |
| FI-E25 | `report::sign(seed, bytes)` | Ed25519 | T-PROOF-1 |
| FI-E26 | `Ledger::append/read_all` | Hash-Kette, `LedgerSink` gespiegelt | T-RESTORE-1 |
| FI-E27 | `Ledger::rotate_epoch()` | Einträge älter als Epoche entfernt | T-JRN-3 |
| FI-E28 | `Ledger::reapply(entries)` | Restore-Guard (11.8) | T-RESTORE-1 |
| FI-E29 | `recover()` | Reihenfolge 11.7 | T-CRASH-1..4 |
| FI-E30 | `checklist()` | Text laut Abschnitt 16, Lint-sauber | T-LINT-1 |

### 10.4 `text`, `vector`, `rank`, `engine`, Fassade

| ID | Funktion | Pre / Post / Err | Test |
|---|---|---|---|
| FI-X01 | `Tokenizer::tokens(text)` | NFKC, lowercase, UAX #29, deterministisch | T-TEXT-1 |
| FI-X02 | `TextBuffer::add/remove` | Puffer je Collection | T-TEXT-3 |
| FI-X03 | `SegmentWriter::write(buffer) -> SegmentId` | AEAD, Stats, Manifest-Eintrag, `sync` | T-TEXT-1 |
| FI-X04 | `Bm25::search(q, k)` | Rangfolge = Referenz; Gate-Filter außerhalb | T-TEXT-1/2 |
| FI-X05 | `TextLayer::purge(plan)` | betroffene Segmente neu, Stats exakt, Alt entfernt | T-TEXT-3 |
| FI-X06 | `TextLayer::scan_residue(needles)` | Decoder-Scan Segmente | T-RAW-4 |
| FI-V01 | `Hnsw::insert(slot, vec)` / `search(q, k)` | Recall ≥ 0,95 | T-VEC-1 |
| FI-V02 | `Hnsw::zero_slot(slot)` | Bytes `zeroize`d, Tombstone, `DocMap` bereinigt | T-VEC-2 |
| FI-V03 | `Snapshot::write_chunk(n)` / `rewrite_chunk(n, &tomb)` | AEAD je Slot unter DEK | T-VEC-2 |
| FI-V04 | `Graph::write_filtered(tombstones)` | keine Tombstone-Slots in der Datei | T-RAW-3 |
| FI-V05 | `Hnsw::rebuild()` | Quote ≥ Schwelle | T-DOCMAP-1 |
| FI-V06 | `VectorLayer::purge/scan_residue` | wie Textschicht | T-RAW-3 |
| FI-R01 | `rrf_fuse(lists, k, top)` | deterministisch | T-RANK-1 |
| FI-G01 | `Collection::insert/insert_batch` | 9.1 Insert-Ablauf; Limits 14.6 | T-ISO-3, T-COV-1 |
| FI-G02 | `Collection::get/get_at` | Gate vor Entschlüsselung; `Erased` | T-VER-1 |
| FI-G03 | `Collection::scan/scan_at` | `doc_tag`-Reihenfolge, Cursor | T-SCAN-1 |
| FI-G04 | `Collection::delete` | logisch, Lock über gesamte Operation | T-CONC-1 |
| FI-G05 | `search_text/vector/hybrid` | Gate-Filter, RRF, Metadatenfilter nachgelagert | T-HYB-1 |
| FI-G06 | `Contextra::erase/erasure_status/lift_restriction/verify/export_report/erasure_checklist` | API 7.3 | T-API-2, T-LEIST-* |
| FI-G07 | `EngineBuilder::open/close` | Pflichtparameter, Recovery vor Freigabe | T-DEF-1, T-SLA-1, T-DROP-1 |

---

## 11. Löschmodell

### 11.1 Begriffe

| Begriff | Definition |
|---|---|
| **Ziel** | Typisierter `EraseTarget`, aufgelöst zu `Matcher` (Bereiche, exakte Schlüssel) und `doc_tag`-Menge |
| **Gate** | Persistierter Eintrag in `erased.set`. Ab hier liefert jeder Lesepfad `Erased` |
| **Schlüsselvernichtung** | Nullen und Fehlprüfung der Tresor-Slots aller Ziel-Dokumente |
| **Purge** | Physisches Entfernen aller Ziel-Einträge (alle Versionen) aus Dateien und Indexen |
| **Schicht (`LayerId`)** | Ort, an dem Ziel-Daten liegen können (12.4) |
| **Zeuge (`LayerWitness`)** | Ergebnis eines Scanners für eine Schicht |
| **Klasse** | Aussagestufe des Berichts (12.5) |

### 11.2 Zustandsmaschine

Jeder Übergang wird **vor** der Aktion mit `sync_data` im Journal festgehalten (I-1). Jeder Schritt ist idempotent (I-11).

| Zustand | Dauerhafte Wirkung | Nächster Schritt | Bei Fehler |
|---|---|---|---|
| `Requested` | Journal `Requested` (Ziel verschlüsselt, Commitment) | Gate setzen | `Pending` |
| `Gated` | `erased.set` enthält Gate; Journal `Gated` | Schlüssel vernichten (entfällt bei `Restricted`) | `Pending` (Gate bleibt) |
| `KeyDestroyed` | Tresor-Slots genullt, Fehlen geprüft; DEK-LRU bereinigt | Rückgabe an Aufrufer (`Gated`); Purge-Fenster abwarten | `Pending` |
| `Drained` | Memtables und abgeleitete Puffer geflusht, versiegelte WAL entfernt, Fehlen geprüft | Rewrite | `Pending` |
| `Rewritten` | Betroffene Dateien neu geschrieben, Austausch committed (Store, Textsegmente, Vektor-Chunks, Graph-Datei) | Unlink | `Pending` |
| `Unlinked` | Alte Dateien entfernt, Verzeichnis `fsync`, Fehlen geprüft | Indexbereinigung | `Pending` |
| `IndexesCleaned` | Blockcache und Arena bereinigt, Snapshots widerrufen | Messung | `Pending` |
| `Measured` | Zeugen aller Pflichtschichten vorhanden | Bericht | `Pending` (Messung wiederholbar) |
| `Completed` | Bericht signiert, Gate entfernt, Ledger-Eintrag, Journal reduziert | terminal | – |
| `Restricted` | Gate ohne Vernichtung und ohne Purge | `lift_restriction` oder Erase | – |
| `Refused` | nichts | terminal | – |

**Pending** trägt `attempt` und `next_retry_at` und meldet nie Erfolg. Backoff 1 s, 2 s, 4 s … bis 15 min. Überschreitung von `erase_sla` erzeugt Alarm (Metrik `erase_sla_breached`, Journal `SlaBreached`), keinen Abbruch.

**Begründung der Reihenfolge (Schlüssel früh):** Durch das Record-Envelope (D-32) liegt jeder Wert unter dem Schlüssel genau seines Dokuments. Das Vernichten des Schlüssels macht die Daten dieses Dokuments sofort unlesbar, ohne andere Dokumente zu berühren. Der Purge benötigt keine Entschlüsselung der Ziel-Werte, weil er über Schlüsselbereiche arbeitet.

### 11.3 Gate und Lesepfade

- Der `ErasedSet`-Eintrag wird **vor** der Rückkehr von `erase` persistiert (`sync_data`).
- `TenantScopedStorage` ist der einzige Zugang zum Store (T-ARCH-2). Er prüft `ReadGate` in `get`, `get_at`, `scan_range`, `scan_range_at`. Die Engine prüft zusätzlich vor jeder Entschlüsselung und im Suchfilter (Text, Vektor, Hybrid).
- Treffer liefert `Erased`. **Zeitreisen sind nicht ausgenommen.**
- Snapshots, die eine betroffene Sequenz sehen könnten, werden widerrufen (`SnapshotRevoked`); der Widerruf ist ein Journal-Eintrag. Konservativ werden alle Snapshots mit `seq ≤ erased_through_seq` widerrufen, wenn sie den Matcher-Bereich berühren können.
- `ErasedSet` wird in `recover()` geladen, **bevor** irgendein Lesepfad freigegeben wird.
- Das Gate gilt auch für Schreibzugriffe auf das Ziel: Insert auf ein gegatetes Ziel → `Erased`. Nach `Completed` ist das Gate entfernt; ein neuer Insert mit demselben `doc_key` erzeugt ein neues Dokument mit neuem Salt.

### 11.4 Purge

**Barrier.** Ein `RwLock` trennt Kompaktion (shared) und Purge (exklusiv je Purge-Lauf). Lesen und Schreiben sind nicht blockiert. Warten ist fristgebunden; Überschreitung → `Pending`.

**Drain** (FI-S21): (1) `rotate_and_seal` des aktiven WAL; (2) Flush aller Memtables einschließlich immutable; (3) Flush der abgeleiteten Puffer (Textpuffer, Vektorpuffer) und Fortschreiben von `derived_durable_through`; (4) Entfernen der versiegelten WAL-Dateien, deren Inhalt vollständig in SSTables und abgeleiteten Indexen liegt; (5) Fehlen jeder Datei per `Vfs::exists` prüfen. Eine verbleibende Datei ist `Pending`, nie Warnung.

**Rewrite** (FI-S22..S24): Kandidaten laut `plan`; streamendes Umschreiben ohne Ziel-Einträge; **alle** Versionen und Tombstones der Ziel-Schlüssel entfallen; übrige Einträge mit Originalwerten `seq`, `tx_id`, `flags` (I-2); Speicher ≤ 4 × Blockgröße; Schreibfehler → Abbruch, alter Reader bleibt gültig, Temp-Datei als Waise (beim Start entfernt).

**Atomarer Austausch** (FI-S24): `sync_all` der neuen Datei → Manifest-Eintrag `ReplaceFiles` + `sync_data` → `sync_dir` → Reader tauschen → Blockcache der alten Datei entfernen und `zeroize`n → alte Datei entfernen → `sync_dir` → Fehlen prüfen (I-4).

**Kompaktionsfilter.** Ab Gate verwirft jede laufende oder neue Kompaktion Einträge im gegateten Bereich ebenfalls (zusätzliche, nicht ersetzende Entfernung). Auch Kompaktionen, die vor dem Gate begonnen haben, sind durch die Barrier getrennt: Der Purge startet erst nach deren Ende und erfasst deren Ausgabedateien.

**Matcher.** Menge von Bereichen fester Tag-Länge und exakten Schlüsseln (D-10).

### 11.5 Tresor-Protokoll (Säule 2)

**Anlegen (Insert eines neuen Dokuments), Schritt 0 vor dem WAL-Commit:** Salt (32 Byte) aus OS-CSPRNG → Slot wrappen → Slot schreiben → `sync_data` Tresor → erst dann Store-Transaktion. Absturz dazwischen hinterlässt einen **verwaisten Slot** (harmlos, 80 Byte); `recover()` bereinigt Waisen, wenn der Tresor das Flag `dirty` trägt (gesetzt vor dem ersten Slot einer Batch, gelöscht nach deren Commit): Abgleich gegen die LOC-Region je Collection.

**Vernichten (FI-E11):**
1. Journal `KeyDestroyIntent` (mit Zahl der Slots) `sync_data`.
2. Je `doc_tag`: Slot-Bytes durch 80 × `0x00` ersetzen (`write_at`).
3. `sync_data` des Tresors.
4. **Rücklesen:** Slot-Bytes müssen vollständig `0x00` sein; sonst `Err` → `Pending`.
5. Speicherindex-Eintrag entfernen; DEK-LRU-Einträge `zeroize`n und entfernen.
6. Journal `KeyDestroyed`.
7. **Tresor-Kompaktion** (FI-E12), wenn der Anteil freier Slots ≥ 25 % oder am Ende jedes Purge-Fensters mit Vernichtungen: neue Datei mit nur aktiven Slots (neue `epoch`), `sync_all`, atomarer Austausch, alte Datei entfernen und Fehlen prüfen.

**Ehrliche Grenze (`WrappedKeyRemanence`):** Der Tresor liegt als Datei im Dateisystem. Das Nullen und die Kompaktion beseitigen den Slot logisch und im Dateiinhalt, nicht aber Kopien in Flash-Überbereitstellung, Dateisystem-Journalen oder alten Blöcken. Der gewrappte Salt ist unter `K_vault` verschlüsselt, das auf der Passphrase beruht; ein Angreifer mit Passphrase **und** Altblock könnte den DEK rekonstruieren. Ein Hardware-Anker würde dies lösen und ist kein v1-Umfang. Die Restrisiken `WrappedKeyRemanence` und `FlashRemanence` stehen in jedem Bericht mit Klasse K.

### 11.6 Abgeleitete Schichten

**Textindex (Schicht `TextSegments`):** Erase entfernt Ziel-Dokumente aus dem Textpuffer und schreibt die Segmente neu, die sie enthalten (Zuordnung über `DocMap.text_segment`); Statistik wird aus den neuen Segmenten neu berechnet; Austausch wie 11.4; Alt-Segment entfernen und Fehlen prüfen.

**Vektorindex (Schicht `Vector`):** `zero_slot` (sofort, in RAM), Slot in Tombstone-Liste, `DocMap`-Eintrag entfällt; im Purge-Fenster: betroffene Snapshot-Chunks neu schreiben (Slot-Record entfällt, Tombstone), Graph-Datei gefiltert neu schreiben (keine Tombstone-Slots), alte Dateien entfernen; Rebuild bei Quote ≥ `vector_rebuild_ratio`. Bis zum Rebuild verweisen RAM-Kanten auf Tombstone-Slots (nur Nummern): Restrisiko `GraphRebuildPending`.

### 11.7 Wiederanlauf `recover()`

Reihenfolge, **vor** Freigabe jedes Lesepfads:

1. Dateisystem-Probe (atomares Rename, Verzeichnis-`fsync`). `EIO` oder Abweichung → Start verweigert.
2. Verzeichnis-Lock (D-50), Rechteprüfung (`keys/` 0700, Dateien 0600). Abweichung → Start verweigert.
3. `HEADER` prüfen, Schlüssel ableiten, `check` verifizieren (`Auth`).
4. `erased.set` laden, Gate scharf schalten.
5. Journal lesen, Hash-Kette prüfen. Defekt → Start verweigert (P2).
6. Tresor öffnen, Slots prüfen; `dirty` → Waisenbereinigung.
7. Store-Recovery (WAL-Replay, Manifest), Waisen (`.tmp`, Ausgabedateien ohne Manifest-Eintrag) entfernen.
8. Abgeleitete Indexe aus dem WAL-Rest nachführen (`derived_durable_through`).
9. Restore-Guard (11.8).
10. Offene Sagas in ihrem Zustand fortsetzen (Idempotenz je Schritt).
11. Erst danach Lesen und Schreiben freigeben.

Unklarer Zustand heißt: Öffnen verweigern (P2).

### 11.8 Restore-Guard (D-37)

**Problem:** Wird das Datenverzeichnis aus einer älteren Sicherung wiederhergestellt, kehren gelöschte Daten zurück, und das ebenfalls wiederhergestellte `erased.set` kennt sie nicht.

**Lösung:** Jede abgeschlossene Löschung erhöht `generation` (u64) und erzeugt einen Ledger-Eintrag. Der Eintrag wird **zusätzlich** über den Port `LedgerSink` an einen Ort **außerhalb** des Datenverzeichnisses gespiegelt (Standard: `<data>/../.contextra-ledger/<store_id>`, 0600; der Betreiber kann ihn auf ein anderes Medium legen). Beim Öffnen gilt: `sink.generation > store.generation` → `RestoreDetected`: Alle Ledger-Einträge mit `generation > store.generation` werden als neue Löschaufträge ausgeführt (Bereiche aus dem Ledger, Gate und Purge, Bericht mit Vermerk `restore_reapplied`), **bevor** Lesepfade freigegeben werden. Ist der Sink unlesbar oder inkonsistent, schlägt `open` mit `Integrity` fehl. Einträge älter als die Commitment-Epoche (365 d) sind entfernt; für Sicherungen, die älter sind, wirkt der Guard nicht (`ExternalCopies`, im Bericht und in der Checkliste ausgewiesen).

### 11.9 Aufwandsmodell und Gate G0

**Streumodell (Rechnung, nicht gemessen):** Ein Dokument besitze *K* Schlüssel. Auf einer Ebene liegen *F* gleich große Dateien mit disjunkten Bereichen. Bei gleichverteilten Schlüsseln werden bei *R* unabhängigen Löschbereichen im Mittel *E = F · (1 − e^(−R/F))* Dateien berührt.

| F (Dateien/Ebene) | K = 50 | K = 300 | K = 1000 |
|---|---|---|---|
| 100 | 39 | 95 | 100 |
| 1.000 | 49 | **259** | 632 |
| 10.000 | 50 | 296 | 952 |

Bei 64 MiB je Datei und F = 1.000, K = 300 schriebe ein verstreutes Schema je Erase ca. 16 GiB. Mit Löschcluster ist *R = 1* je Dokument (plus ein LOC-Schlüssel): pro Ebene höchstens eine Datei, bei ca. 6 Ebenen plus bis zu 4 überlappenden L0-Dateien also ≤ 10 Dateien ≈ 640 MiB. Der Gewinn ist der Faktor *K*, nicht die Datenbankgröße. Grenzen: Viele unabhängige Ziele im selben Fenster erhöhen *R*; Bündelung (D-39) und Dateigröße sind Stellhebel. Gleichverteilung ist eine Annahme.

**Gate G0 (verbindlich, in zwei Teilen):** Teil 1 am Ende von Phase 2 (Korrektheit, Crash), Teil 2 am Ende von Stufe 0 (Löschkosten). Messung auf dokumentierter Hardware.

| Prüfung | Schwelle für Verbleib beim LSM |
|---|---|
| T-DIFF-1 | 0 Abweichungen über 10⁶ Operationen gegen `ReferenceModel`; Differential gegen redb ebenfalls 0 |
| T-CRASH-1/2 | 0 verlorene bestätigte Transaktionen, 0 wiederaufgetauchte abgebrochene |
| B-ERASE-WA (Einzeldokument, 10⁶ Dokumente, K ≈ 6 Records inkl. LOC, Body 1 KiB, Store ≥ 8 GiB) | Rewrite-Volumen ≤ 15 % des redb-Vollrewrite-Volumens und absolut ≤ 1 GiB |
| B-ERASE-WA (Betroffener mit 1.000 Dokumenten) | ≤ 25 % des redb-Volumens |
| Restmenge | Positivkontrolle-geprüft 0 Treffer in E1 und E2 nach Erase |
| Aufwand Phase 2 | ≤ 1,5 × Schätzung (≤ 120 Arbeitstage) |

**Ausstiegsauslöser (bindend):** (a) T-CRASH zweimal nicht bestanden nach vier Wochen; (b) Zeitbox WP-2.4 überschritten; (c) B-ERASE-WA verfehlt; (d) Aufwandsgrenze überschritten. Bei Ausstieg: redb-Adapter hinter `StorageEngine` (ca. 25–35 Arbeitstage), Zeitreisen über Versionsschlüssel `(key, seq)` mit eigener GC, Purge als Datei-Rewrite. **`contextra-erasure`, Tresor, Messung und Bericht bleiben unverändert**, weil sie nur `ErasureLayer`, `ReadGate` und `RecordSealer` kennen.

### 11.10 Invarianten

| ID | Invariante | Test |
|---|---|---|
| I-1 | Jede Zustandsänderung wird vor der Aktion im Journal mit `sync_data` festgehalten. | T-CRASH-1 |
| I-2 | Rewrite übernimmt `seq`, `tx_id`, `flags` unverändert. | T-VER-2 |
| I-3 | Der alte Reader bleibt bis zum Manifest-Commit gültig. | T-CRASH-3 |
| I-4 | Unlink nur nach Manifest-`sync` und Verzeichnis-`sync`. | T-CRASH-1 |
| I-5 | Kein Purge ohne Gate; kein Gate ohne persistierten Eintrag. | T-CONC-1 |
| I-6 | Nicht-Ziel-Daten sind bitgleich (NeighborDigest). | T-ISO-1 |
| I-7 | `Completed` nur mit gültigen Zeugen aller Pflichtschichten und geprüfter Schlüsselabwesenheit (bei Vernichtung). | T-PROOF-1, T-RAW-1 |
| I-8 | Kein Fehler wird verschluckt: Jede fallible Operation propagiert oder wird als `Pending` persistiert. | T-LINT-2 |
| I-9 | Tresor-Slot ist vor dem WAL-Commit des Dokuments dauerhaft. | T-VAULT-2 |
| I-10 | Nach `KeyDestroyed` ist kein DEK des Ziels im Speicher des Tresors oder des LRU. | T-VAULT-3, T-SEC-2 |
| I-11 | Jeder Saga-Schritt ist idempotent: zweimaliges Ausführen ergibt denselben Zustand. | T-CRASH-1 |
| I-12 | WAL-Integrität und Verschlüsselung sind in jedem Build aktiv. | T-CFG-1, T-DEF-1 |
| I-13 | Ein Lesepfad liefert für gegatete Ziele nie Daten, auch nicht aus Cache oder Zeitreise. | T-VER-1 |
| I-14 | Abbruch des Aufruferfutures ändert den Ausgang einer mutierenden Operation nicht. | T-CANCEL-1 |

### 11.11 Nebenläufigkeit

- Pro Dokument ein Lock, derselbe für Insert, Update, Delete und Erase. Gate-Setzen und logische Sperre laufen unter diesem Lock (behebt B-13).
- Batch-Fenster (Standard 60 s): Aufträge werden zu einem Purge-Lauf zusammengefasst, dedupliziert nach Commitment. `Timing::Now` erzwingt sofortigen Lauf und weist die Zusatzkosten im Bericht aus (`times.window_bypassed = true`).
- Pro `ErasureLayer` läuft höchstens ein Purge gleichzeitig; Schichten laufen nacheinander in fester Reihenfolge: `Store`, `TextSegments`, `Vector`.
- Rewrite-Kosten skalieren mit der Zahl betroffener Dateien, nicht mit der Datenbankgröße (Gate B-ERASE-WA).

---

## 12. Messung und Nachweis

### 12.1 Messklassen E1, E2, E3 (D-35)

Ein Rohscan auf Klartext-Needles beweist in einem verschlüsselten Store nichts, weil Dateischichten ohnehin keinen Klartext enthalten. Deshalb drei Messklassen:

| Klasse | Prüfung | Beweiswert |
|---|---|---|
| **E1 Rohscan** | Alle Dateien im Datenverzeichnis (und `ledger`-Sink) nach Needles in UTF-8, UTF-16LE, Base64, Hex; je Datei und Offset | Findet Klartextreste (Journal, Temp-Dateien, Logs, nicht verschlüsselte Nebenpfade) |
| **E2 Decoder-Scan** | Entschlüsselt alle Blöcke (Store, Textsegmente, Vektor-Chunks, Journal, Tresor) mit den Testschlüsseln und sucht in den **Klartextblöcken** nach: Inhalts-Needles, blinden Tags (`cluster_tag`, `doc_tag`, `term_tag`), `doc_key`-Needles **und Ciphertext-Markern** (Nonce ‖ erste 16 Byte der Ziel-Envelopes, vor dem Erase aufgezeichnet) | Beweist physische Entfernung auch in verschlüsselten Dateien |
| **E3 Schlüsselabwesenheit** | Tresor-Slot der Ziel-Dokumente ist `0x00`-gefüllt und nicht im Speicherindex; Entschlüsselungsprobe: ein vor dem Erase aufgezeichnetes Ziel-Envelope lässt sich mit keinem ableitbaren DEK mehr öffnen | Beweist Klasse K |

**Zwei Ebenen der Anwendung:**

- **Produktion (im Erase-Ablauf):** `ErasureLayer::scan_residue` jeder Schicht liefert Zeugen (E2 auf eigenen Formaten), der Rohscan `E1` läuft über das Datenverzeichnis, `E3` über den Tresor. Needles sind Blind-Tags, `doc_key`-Hashes und Ciphertext-Marker aus dem Scope; Inhalts-Needles stehen im Erase-Ablauf nicht zur Verfügung (die Inhalte sind nicht mehr zugänglich, sobald das Gate steht).
- **Verifikation (Lab, unabhängig):** `contextra-lab` führt E1, E2, E3 mit eigenem Decoder und Inhalts-Needles aus (Canary-Dokumente). Diese Messung belegt die Produktionsmessung und ist Release-Voraussetzung.

### 12.2 Positivkontrolle

Vor jedem Messlauf wird im Testmandanten ein Canary geschrieben und vom Scanner gefunden (`positive_control`). Nicht gefunden → Lauf ungültig, kein Befund, Zeuge `Invalid`. Der Canary liegt nie im produktiven Mandanten. Im Produktionsablauf läuft die Positivkontrolle gegen einen synthetischen, im Speicher erzeugten Datenblock im jeweiligen Format (ohne Schreiben in den Store).

### 12.3 Lab und `LeakyLayer`

Das Lab erzeugt je Lauf hochentropische Needles (`CanaryFactory`, Seed protokolliert) und führt Szenarien aus: Insert → Update → Erase → Restart → Crash an Punkt *n*. `LeakyLayer` (absichtlich fehlerhaft: überspringt Dateien, behält Versionen, lässt Index-Reste) muss in **jedem** Fehlermodus gefunden werden; ein sauberes System muss bestehen (T-RAW-2). Das Lab-Ergebnis ist maschinenlesbar (`lab-report.json`) und Teil der Evidenz.

### 12.4 Schichten

| LayerId | Inhalt | Mechanismus | Messung | Pflicht |
|---|---|---|---|---|
| `WalSealed` | versiegelte WAL-Dateien | Drain, Entfernen | Dateiliste, E1, E2 | ja |
| `WalActive` | aktive WAL-Datei | Rotate, Drain | Dateiliste, E1, E2 | ja |
| `Memtable` | alle Versionen im RAM | Flush, `remove_range` | HeapScan, E2 nach Flush | ja |
| `Sstable` | Daten, Index, Bloom | Rewrite | E1, E2 | ja |
| `Manifest` | Manifest-Logs | Austausch | E1, E2 | ja |
| `TextSegments` | Textsegmente, Puffer | Segment-Rewrite | E1, E2 | ja bei `text_index` |
| `Vector` | Arena, Chunks, Graph | Zeroing, Chunk- und Graph-Rewrite | E1, E2, HeapScan | ja bei `vector_dim` |
| `Vault` | Tresor-Slots | Nullen, Kompaktion | E3 | ja |
| `BlockCache` | entschlüsselte Blöcke | Eviction, `zeroize` | HeapScan | ja |
| `Journal` | Journal, Scope-Dateien | Reduktion nach `Completed` | E1, E2 (ohne Zielbezug) | ja |
| `Snapshots` | MVCC-Versionen, Snapshot-Register | Widerruf, Purge | Versionsscan je Sequenz | ja |

Eine Schicht, die für das Ziel nicht existiert (Collection ohne Textindex), meldet `NotApplicable` und zählt als gemessen.

### 12.5 Klassenverband und `classify`

Die Aussagen sind unabhängig und bilden einen **Verband** (Menge von Marken):

| Marke | Bedingung | Textbaustein im Bericht (`Display`) |
|---|---|---|
| **G** (Gate) | Sperre persistiert | „Für Lesezugriffe gesperrt.“ |
| **K** (Schlüssel) | Tresor-Slots aller Ziel-Dokumente genullt, Fehlen geprüft, Entschlüsselungsprobe (E3) negativ | „Dokumentschlüssel aus dem Schlüsseltresor entfernt.“ |
| **P** (Physisch) | Alle Pflichtschichten mit gültigem Zeugen ohne Befund (E1, E2) | „In den gemessenen Schichten nicht nachweisbar. Restrisiken laut Liste.“ |

`ErasureClass { marks: {G?,K?,P?}, partial: Vec<LayerId> }`. `Partial` listet Pflichtschichten ohne gültigen Zeugen; dann entfällt **P**. Bei `Restricted` ist nur **G** gesetzt.

`classify(Evidence) -> ErasureClass` ist eine **reine Funktion**, **monoton** (fehlt ein Zeuge, sinkt die Klasse, sie steigt nie) und wird durch Mutationstests (T-MUT-1, ≥ 95 % getötet) und eine zweite, unabhängige Implementierung in `ctx-verify` abgesichert. Es existieren keine Klassen „kryptografisch gelöscht“ oder „garantiert gelöscht“; die `Display`-Texte sind die einzigen zulässigen Formulierungen.

### 12.6 Restrisiken (Pflichtkatalog)

Jeder Bericht listet **alle zutreffenden** Einträge:

| Risiko | Gilt wenn |
|---|---|
| `FlashRemanence` | Speichermedium ist Flash/SSD oder unbekannt |
| `FsJournalOrCow` | Dateisystem mit Journal oder Copy-on-Write oder unbekannt |
| `WrappedKeyRemanence` | immer, wenn Klasse K gesetzt ist |
| `ProcessMemory` | Heap, Swap, Coredump möglich; `mlock` fehlgeschlagen |
| `PageCacheTransient` | Seitencache des Betriebssystems bis zur Eviction |
| `ExternalCopies` | immer (Backups, Snapshots, Kopien beim Betreiber) |
| `ExternalProcessors` | Embeddings oder LLMs außerhalb von Contextra |
| `PseudonymCommitment` | solange die Commitment-Epoche existiert |
| `ReplicaWindow` | Zeitfenster zwischen Gate und Abschluss |
| `UnmeasuredLayer(id)` | Schicht ohne gültigen Zeugen |
| `GraphRebuildPending` | Vektor-Tombstones im RAM-Graph bis Rebuild |
| `RestoreWindow` | Sicherungen älter als die Ledger-Epoche |

Ein Bericht ohne Restrisikoliste ist ungültig (T-REPORT-1).

### 12.7 Bericht, `verify`, Verifier

**Bericht `ctx-erasure-report/1`** (Schema Anhang D): `report_id`, `erasure_id`, `scope_kind` (`document|subject|collection|tenant`), `subject_commitment`, `retention`, `times` (`requested_at`, `gated_at`, `key_destroyed_at`, `drained_at`, `rewritten_at`, `measured_at`, `window_bypassed`), `layers[]` (`layer_id`, `status`, `witness_digest` = BLAKE3 über Scan-Protokoll: Dateizahl, Bytezahl, Needle-Kategorien, `positive_control_found`, `encodings_searched`, `measurement` ∈ {E1,E2,E3}), `store_counters` (Dateien neu geschrieben, Bytes geschrieben, WAL-Segmente entfernt, widerrufene Snapshots, entfernte Tresor-Slots; nur Zahlen), `residual_risks[]`, `provenance` (Contextra-Version, Build-ID, Digest der effektiven Konfiguration ohne Geheimnisse), `class`, `statement` (fester Text laut 6.4 plus „Die Signatur beweist Integrität und Herkunft aus dieser Installation, nicht die Wahrheit des Inhalts.“), `signature`. Der Bericht enthält **nie** Dokumentinhalt, Schlüssel, `doc_id` oder `subject_id` im Klartext.

**`verify(ctx, target)`** läuft in einem **neuen Prozess** ohne Zugriff auf Inhalte: Ziel-Identität (`doc_key`, `subject_id`) vom Aufrufer → Tags und Commitment ableiten → Gate-Menge/Ledger prüfen → E1 und E2 (Tags, `doc_key`-Needles) → E3. **Grenze (ausdrücklich im Bericht):** Inhalts-Needles stehen nur beim Erase zur Verfügung; `verify` prüft Schlüssel-Reste, Identitätsreste und Tresor-Abwesenheit, nicht Inhaltsreste.

**Externer Verifier `ctx-verify`:** siehe 8.17.




---

# Contextra v1 – Spezifikation (SPEC)

| Feld | Wert |
|---|---|
| Status | Zur Freigabe durch den Auftraggeber |
| Spezifikationsversion | 1.0.0-spec.1 |
| Stand | 2026-11-01 |
| Repository | `github.com/tfufuz1/contextra`, `main` @ `76ea331c697d95fae195b1009a737dc2b5d8f486` |
| Geltung | Verbindlich für v1. Ersetzt die Dokumente *Spezifikation v1 Erasure*, *Spezifikation v1 Final*, *Spezifikation und Umsetzungsplan*, *Implementierungsspezifikation SOLL* und *Implementierungsspezifikation Erasure Core* als Arbeitsgrundlage. |
| Verhältnis zu `AGENTS.md` | Teilweise überschrieben durch ADR-001 (Abschnitt 3.3). Bis dahin gilt `AGENTS.md`, soweit nicht im Widerspruch. |
| Begleitdokument | `PLAN.md` (Arbeitspakete, Gates, Reihenfolge) |

> Keine Rechtsberatung. Rechtliche Aussagen sind Arbeitshypothesen für die Prüfung durch Fachanwalt bzw. Datenschutzbeauftragte (Gate G-LEGAL).

---

## 0. Evidenz und Prüfmodus

### 0.1 Kennzeichnung

| Marke | Bedeutung |
|---|---|
| **[V]** | Am Clone-HEAD `76ea331c6` statisch gelesen. Datei und Zeile angegeben. |
| **[U]** | Ergebnis des Build- und Auditlaufs des Auftraggebers (`cargo check --workspace`, `scripts/contextra_full_audit.py --all`). Nicht reproduziert. |
| **[K]** | Aus *Fehlerkatalog Stand 2026-11-01* (Stand `f653f0f73`). Am Clone nicht erneut gelesen. |
| **[D]** | Aus den Dokumenten übernommen, nicht geprüft (insbesondere Rechtssynthese). |
| **[E]** | Entscheidung, Empfehlung oder Schätzung dieser Spezifikation. |

### 0.2 Prüfmodus

Diese Spezifikation wurde ausschließlich **statisch und online** erstellt: Quelltextlektüre, Manifest- und Abhängigkeitsauswertung, Musterzählung per `grep`. Es wurde **nichts gebaut und nichts getestet**. Alle Aussagen über Kompilierbarkeit und Laufzeitverhalten sind [U] oder Erwartungen, bis sie in WP-0.1 gemessen sind.

Zählungen per `grep` enthalten inline-`#[cfg(test)]`-Module und sind deshalb **Obergrenzen** für Produktionscode.

### 0.3 Versionierung der Quellen

Die Quelldokumente beziehen sich auf unterschiedliche Repo-Stände (`9da0eca1b`, `f653f0f73`, `c4e285c`). Der geklonte HEAD ist `76ea331c6`. Die Zahlen der Quellen sind deshalb nur mit Vorsicht übernehmbar. Diese Spezifikation nimmt den Clone-HEAD als Wahrheit (Prinzip P9).

---

## 1. Produkt

### 1.1 Mission

Contextra ist eine **eingebettete Rust-Bibliothek** für lokale Wissens- und Gedächtnisspeicherung von Agenten und RAG-Systemen (Chunks, Volltext, Vektoren, Metadaten, Zuordnung zu betroffenen Personen). Sie löscht personenbezogene Daten, die sie selbst hält, **physisch und gemessen**: im Datenverzeichnis, in allen Indexen, in allen MVCC-Versionen (Zeitreisen) und im Prozessspeicher, soweit steuerbar. Der Löschbericht weist den gemessenen Zustand je Speicherort aus und benennt die Restrisiken.

### 1.2 Zielnutzer

Entwickler und Betreiber von Agenten- und RAG-Systemen im EU-Raum, die Löschpflichten (Art. 17 DSGVO) technisch umsetzen müssen und einen prüfbaren Nachweis brauchen.

### 1.3 Nicht-Ziele v1

- Netzwerkdienst, Server-Socket, Sharding, Replikation, P2P.
- Erzeugung von Embeddings, Aufrufe von LLM-Diensten (Vektoren und Text kommen vom Aufrufer).
- Dateiformat-Parsing (PDF, DOCX, EPUB, ...).
- Graph-Algorithmen und Relationen.
- Schlüsselvernichtung als Löschmechanismus (reserviert, Abschnitt 3, D-01).
- Einschränkung der Verarbeitung (Art. 18) als Speicherzustand.
- Persistente Caches (Embedding-, Query-, KV-Cache).
- Automatische Entitätsextraktion.

### 1.4 Verspricht und verspricht nicht

**Verspricht (prüfbar):**

1. Nach abgeschlossener Löschung enthält kein Datenverzeichnis-File, kein Index und kein Contextra-kontrollierter Speicherbereich einen gemessenen Klartextrest des Ziels. Die Messung ist unabhängig vom Löschmechanismus.
2. Ein Löschauftrag wirkt ausschließlich auf sein typisiertes Ziel. Alle Nicht-Ziel-Daten bleiben bitgleich erhalten (T-ISO-1).
3. Unterbrechungen (Absturz, I/O-Fehler) führen nach Neustart zur Vollendung oder zu sichtbarem Status `Pending`. Stille Teilzustände gibt es nicht.
4. Der Bericht benennt genau, was gemessen wurde, und listet alle Restrisiken.

**Verspricht nicht:**

- „DSGVO-konform“, „rechtskonform“, „garantiert gelöscht“, „unwiederbringlich“, „informationstheoretisch“, „kryptografisch gelöscht“ (Abschnitt 13.2).
- Löschung außerhalb von Contextra (Backups, Volume-Snapshots, Logs fremder Systeme, Kopien beim Betreiber).
- Löschung auf Hardware-Ebene (Flash-Remanenz, Dateisystem-Journale, Swap).

---

## 2. Leitprinzipien (normativ)

| ID | Prinzip | Konkrete Folge |
|---|---|---|
| P1 | **Wirkung vor Behauptung** | Ein Zähler oder Status „gelöscht“ entsteht nur aus einem Zeugen (`LayerWitness`) eines unabhängigen Scanners. |
| P2 | **Fail-closed** | Unbekannter Zustand, korrupte Daten, abgelaufene Fristen: Verweigerung oder `Pending`. Kein Weiterlaufen. |
| P3 | **Ein Weg** | Ein Löschpfad, eine Fehlerbehandlung, ein Format je Konzept. Keine Legacy-Lesepfade. |
| P4 | **Ziel-Isolation** | Jede mutierende Operation hat ein typisiertes Ziel. Nachbarn sind bitgleich unverändert (messbar). |
| P5 | **Keine Schalter für Sicherheit** | Kein `cfg(feature)` auf Verschlüsselung, Integrität, Löschmessung, Mandantenprüfung (AGENTS §2 A1). |
| P6 | **Keine Panics, keine verschluckten Fehler** | Im Produktionscode kein `unwrap`, `expect`, `panic!`, `let _ =` auf Fallible, `unwrap_or_default` auf I/O. |
| P7 | **Determinismus über Ports** | Zeit (`Clock`), Zufall (`Rng`), Dateisystem (`Vfs`), IDs (`IdGen`). Ausnahme: Schlüssel-, Salt- und Nonce-Material aus OsRng. |
| P8 | **Kleinster Kern** | Ein Crate oder Modul ist nur im Kern, wenn ein Kernkonsument es braucht (`cargo tree`-belegt). |
| P9 | **Messen vor Optimieren** | Jedes Performance-Gate hat eine Basislinie auf fester Hardware. Ohne Basislinie kein Gate. |
| P10 | **Verifizierbarkeit vor Eleganz** | Jede Designentscheidung nennt den Test, der sie belegt. Ohne Test keine Entscheidung. |

---

## 3. Entscheidungsregister

### 3.1 Entscheidungen

| ID | Entscheidung | Begründung | Verworfene Alternative | Status |
|---|---|---|---|---|
| **D-01** | **Löschmechanismus v1: physische Purge-Compaction plus Messung.** Schlüsselvernichtung (Klasse C) ist nicht Teil von v1. | Die Rechtswirkung der Schlüsselvernichtung ist offen [D]. Eine Mission „physische Löschung“ darf nicht an dieser Frage hängen. Purge wirkt unabhängig davon und ist messbar. Schlüsselvernichtung braucht außerdem einen löschbaren Anker, den Contextra nicht bieten kann (siehe N10 in Quelldokument Erasure Core). | Zwei-Ebenen-Modell mit DocDek und KeyAnchor (Quelle *Erasure*, *Erasure Core*, AGENTS §4 Ebene B). Entfällt in v1, Schnittstelle bleibt frei (X-1). | [E], Auftraggeber bestätigt |
| **D-02** | **Speicher: eigener LSM-Tree behalten**, hinter dem Port `StorageEngine`. redb ist **Rückfallebene** (Adapter nur bei Ausstieg, Abschnitt 3.2 Gate G0). | Purge-Compaction und atomarer Manifest-Tausch sind LSM-Primitive. Bei redb oder SQLite wüchse die Schreibverstärkung mit der Gesamtdatenbank, nicht mit dem Löschziel [E, nicht gemessen]. Die Begründung „redb kann kein Crypto-Shredding“ trägt nicht und wird nicht verwendet. | redb als Vollersatz (Bericht 3). Würde D-01 ohne Purge-Rewrite nicht erfüllen. | [E], Gate G0 offen |
| **D-03** | **Ports async** (`BoxFuture`), konsistent zu `StorageEngine`. Keine `block_on` im Bibliothekscode. | Synchrone Ports über asynchroner I/O erzwingen `block_on` und sind Ursache der Deadlock-Befunde C-L18/C-L19 [K]. | Synchrone Ports mit Blocking-Adapter. | [E] |
| **D-04** | **Ein `Vfs`-Trait**, in `contextra-ports` definiert, in `contextra-durable-fs` implementiert. Der Store und die Engine nutzen ausschließlich `Vfs`. | Zwei `Vfs`-Traits existieren ([V] `ports/src/vfs.rs:29`, `durable-fs/src/vfs.rs:54`). Der Store nutzt `Vfs` nicht [V: 0 Treffer]. Fehlerinjektion (FaultVfs) und Crash-Tests setzen `Vfs` voraus. | Zwei Traits mit Adapter zwischen beiden. | [E] |
| **D-05** | **Verschlüsselung ist nicht abschaltbar in v1.** Es gibt keinen Klartextmodus im Produktionsbuild. Tests dürfen einen In-Memory-Fixture-Schlüssel nur unter `cfg(test)` nutzen. | Default ist heute unverschlüsselt [V: `contextra/src/builder.rs:147` `encryption_mode: None`]. Ein Opt-out macht jede Erase-Aussage zur Ausnahme. P5 verbietet Schalter. | Default an, Opt-out erlaubt (Final R-SEC-1). Verworfen, weil das Opt-out selbst zum Parallelbestand führt. | [E], Breaking Change |
| **D-06** | **Passphrase nur über Argon2id.** Mindestparameter m ≥ 64 MiB, t ≥ 3, p ≥ 1. Unterschreiten ist Fehler, nicht Warnung. HKDF bleibt nur für Subkey-Ableitung aus dem Master-Key. | HKDF-Ableitung aus Passphrase [V: `crypto/src/crypto.rs:88` `try_new`]. Untergrenze heute 19.456 KiB [V: `crypto/src/kdf.rs:36`]. | Untergrenze 19 MiB beibehalten. | [E] |
| **D-07** | **Mandantenpflicht.** `TenantPolicy::Required` ist der einzige Modus. Operationen ohne `RequestContext` mit Mandanten-ID sind nicht kompilierbar. | Default heute `Optional` [V: `engine/src/lib.rs:207–209`]. VETO-F10 [AGENTS §2 Nr. 7]. | Optional als Default. | [E] |
| **D-08** | **Dokumentidentität: 128 Bit, schlüsselgebunden.** `DocId = BLAKE3-keyed(K_idx, tenant ‖ collection ‖ doc_key)[0..16]`. Beim Insert wird eine Kollision mit anderem `doc_key` als Fehler `IdCollision` abgelehnt. | 64 Bit ohne Schlüssel [V: `types/src/types/domain/ids.rs`, `DocId(pub u64)` ohne Feature] sind Pseudonym und kollisionsanfällig (≈2,7 % bei 10⁹ Dokumenten, Rechnung [E]). Schlüsselbindung verhindert Berechnung aus Nutzerkennung. | 64 Bit mit „ADR-016-Kollisionsprüfung“. ADR-016 ist für DocId nicht belegt [V: nur Decay-Controller]. | [E] |
| **D-09** | **Interne Schlüssel sind blind.** Alle Schlüsselkomponenten außer fester Familienkennung sind keyed Hashes (`K_idx`). Nutzerschlüssel stehen nur verschlüsselt im Wert. Dadurch enthält der Klartext-Index in SSTable-Blöcken keine Nutzerdaten (N2). | Index-Schlüssel werden im Klartext geschrieben [V: `sstable/builder.rs` `flush_block`/`finish`, N2]. Blinde Schlüssel machen Präfix-Matcher überflüssig. | Klartextschlüssel mit Präfix-Matcher (Quelle Erasure Core). | [E], ersetzt Entscheidung E1 der Quelle |
| **D-10** | **Matcher nur exakt.** Präfix-Scans auf Nutzerschlüsseln entfallen. `scan` iteriert die Collection in Schlüsselreihenfolge, nicht nach Nutzer-Präfix. | Präfixlogik ist Ursache von F01/C-L09 (Löschung trifft Mandantendaten) [V: `scanner/purge.rs:36,44`]. Mit D-09 wären Präfixe ohnehin Hashpräfixe ohne fachlichen Wert. | Fixlängen-Präfixe (Quelle Erasure Core 4.4 Nr. 6). | [E], Verhaltensänderung |
| **D-11** | **Zeitreisen bleiben im Umfang.** Purge entfernt alle MVCC-Versionen. Offene Snapshots, die ein betroffenes Ziel berühren, werden **widerrufen** (Fehler `SnapshotRevoked`), nicht abgewartet. | Vorgabe des Auftraggebers. Warten auf Snapshots macht Löschfristen unbegrenzt (C-L17 [K]). Widerruf ist die einzige Variante, die eine feste Frist einhält. | Verzögerter Unlink bis Snapshot-Ende (Quelle Erasure Core 4.5). Verworfen wegen unbegrenzter Restdauer. | [E] |
| **D-12** | **Lesesperre (ReadGate) vor Purge.** Ein Ziel ist ab der persistierten Sperre (`Gated`) für jeden Lesepfad gesperrt, auch für Zeitreise-Lesezugriffe. | Sofortige Wirkung für Nutzer ohne Warten auf den Rewrite. Ohne Gate sind Daten bis zum Rewrite lesbar. | Nur Purge ohne Gate. | [E] |
| **D-13** | **Löschbericht: Ed25519-signiert, kanonisches JSON, Ein Format, keine Legacy-Versionen.** | Externe Prüfbarkeit [D, Synthese]. Symmetrische Beweise sind fälschbar durch den Prüfer [K: B-S10]. V1–V3 werden entfernt. | DeletionProof V1–V3 (HMAC/Legacy). | [E] |
| **D-14** | **Betroffenen-Löschung in v1** über `subject_id` beim Insert. Index `subject_tag → DocIds`. | Ohne Zuordnung kann Contextra eine Person nicht auffinden [D, Final E-SUBJ-1]. | Betroffenen-Löschung erst in v1.1. | [E] |
| **D-15** | **Aufbewahrung: nur Pflichtangabe, kein Scheduler.** `erase` verlangt `RetentionAttestation` (Code-Enum). Der Bericht führt sie. Einschränkung (`Restrict`) entfällt in v1. | Aufbewahrungspflichten sind Betreiberwissen. Eine Pflichtangabe verhindert stilles Löschen, ohne Fristlogik einzuführen [D, Art. 17 Abs. 3]. | `Restrict` in v1 (Quelle Erasure). | [E] |
| **D-16** | **Keine Checkpoints, Forks, Hardlink-Klone in v1.** `contextra-checkpoint` ist aus dem Workspace entfernt. | Jeder Klon ist eine Kopie außerhalb der Löschgrenze. Der Sweep je Klon [K: D-C09, D-C08] wäre fehleranfällig. Zeitreisen bleiben über MVCC erhalten (D-11), nicht über Checkpoints. | Klone mit Sweep (Quelle Erasure Core 4.1 Zeile S12). | [E], Auftraggeber-Entscheidung E-V1-3 |
| **D-17** | **Graph aus v1 entfernt, nicht nur per Feature abgeschaltet.** `contextra-graph` und die Relations-API (`relate`, `__rel:`) entfallen. | Feature-Gates auf Löschpfade sind verboten (P5). Graph-Reparatur pro Löschung ist Veto-Gegenstand (VETO-F02). Nicht im RAG-Kern. | Entkoppeln mit Feature `graph` (Entscheidungsgrundlage W3). Verworfen, weil ein abschaltbarer Graph-Löschpfad denselben Fehler wie `wal-integrity` erzeugt (V-01 [V]). | [E], Owner-Review |
| **D-18** | **Vektorlöschung: Slot-Zeroing plus Batch-Rebuild.** Kein Rewiring der HNSW-Kanten pro Löschung. Rebuild bei Tombstone-Quote ≥ 0,2. | VETO-F02 [VETOES.md]. Kanten enthalten nur Slot-Nummern, keinen Vektorinhalt. | Kantenreparatur pro Löschung (`hnsw/deletion.rs`, 353 Zeilen [V]). | [E] |
| **D-19** | **Vektor-Snapshots und Vektorarena verschlüsselt.** Snapshot-Dateien laufen durch denselben Store-Schlüssel wie SSTables und über `Vfs`. | Vektoren gelten vorsorglich als personenbezogen [D, DSK-OH RAG]. Unverschlüsselte mmap-Snapshots wären ein Parallelbestand. | Snapshots unverschlüsselt (heutiger Zustand, Befund G-03/G-04 [K]). | [E], Implementierung offen |
| **D-20** | **Textindex: Postings als exakte Schlüssel `(term_tag, doc_id)`.** Document Frequency als Zähler je `term_tag`. | Per-Term-Postinglisten über viele Dokumente machen Purge zu Listen-Rewrites [D]. Einzelschlüssel machen Löschung exakt und Matcher trivial. | Postinglisten je Term mit Segment-Rewrite (Quelle Erasure Core 4.7). | [E], Lasttest offen (R-03) |
| **D-21** | **Relations- und Mandanten-Tags sind keyed.** `tenant_tag`, `coll_tag`, `term_tag`, `subject_tag` = BLAKE3-keyed(K_idx, …)[0..16]. | Klartext-Tags würden Mandanten- und Begriffsnamen im Index offenlegen (N5). | Klartextnamen im Schlüssel. | [E] |
| **D-22** | **Entfernung ohne Feature-Gate, mit Archiv.** Entfernte Crates werden in den Branch `archive/v0-full` verschoben, nicht gelöscht. | Historie bleibt auffindbar. Löschung aus dem Hauptzweig macht die Entscheidung prüfbar. | Löschen. | [E] |
| **D-23** | **Lizenz- und DRM-Schicht (`contextra-license`) aus dem Speicherkern entfernt.** | Der Speicher soll keine Lizenzlogik in MIT-Code tragen. Legacy-Lizenzen ohne Installationsbindung sind umgehbar [K: R-03, bestätigt]. Geschäftsmodell ist Auftraggebersache. | Fassade mit `LicenseGate` in v1 (Entscheidungsgrundlage 6.2). | [E], Owner-Entscheidung E-LIC |
| **D-24** | **Ingest, Privacy, Agent, Cognition, Router, Adapt, KV-Cache, Inference, Sandbox, Audit-Export, AVV-Generator aus v1 entfernt.** | Nicht im Kern laut Produktumfang. Ingest parst im Hauptprozess ohne Sandbox [K: B-S25]. Audit-Export hängt an entfernten Beweistypen. AVV-Vorlagen sind nicht fachlich freigegeben [K: G-24/G-25]. | Behalten als Erweiterungen. | [E] |
| **D-25** | **MCP-Server und Python-Bindings sind v1.1-Adapter** auf der stabilen Rust-API. | MCP-Löschwerkzeuge verwerfen `_proof_key` und `_tenant_id` [K: C-L04, bestätigt Zeile `tools_crud.rs`]. Adapter erst nach Freeze der API. | MCP in v1. | [E] |
| **D-26** | **`contextra-rank` auf RRF-Fusion reduziert.** Kalibrierung, Ranking-Strategien und Adaptivität entfallen. | Hybridsuche (HNSW + BM25 + RRF) ist Kern für RAG. Der Rest ist nicht Kern. | Vollständig behalten (Entscheidungsgrundlage 6.2) oder vollständig entfernen. | [E] |
| **D-27** | **Keine automatische Entitäts- oder Metadatenextraktion** in v1. `AutoExtractionMode` entfällt. | Default ist heute `Enabled` [V: `types/src/types/domain/misc.rs:591–597`] und erzeugt ohne Zutun Parallelbestände. Extraktion ist Aufgabe des Aufrufers. | Default `Disabled` mit Feature (AGENTS §5). Verworfen wegen P5. | [E], AGENTS §5 überschrieben |
| **D-28** | **SIMD: u8-VNNI-Kernel entfernt.** Nur f32-Kernel und Skalarpfad. Jeder SIMD-Kernel hat einen Cross-Check-Test gegen Skalar. | Operandentypen des `_mm512_dpbusd_epi32`-Aufrufs sind beide `u8` [V: `simd/src/kernels/avx512.rs:445,565–569`], die Instruktion erwartet u8 × i8. Falsche Distanzen [K: D-V01]. | u8 auf u16 erweitern. Mehr Code für einen Pfad ohne Messgewinn [E]. | [E] |
| **D-29** | **`unsafe` nur in `contextra-simd` und `contextra-sys`.** `contextra-wire` entfällt mit MCP und Python (D-25). | AGENTS §2 Nr. 6 nennt drei Inseln. Eine Insel weniger reduziert Audit-Fläche. | Drei Inseln. | [E], AGENTS-Änderung in ADR-001 |
| **D-30** | **Erase-API synchron mit Frist.** `erase` kehrt nach persistierter Sperre (`Gated`) zurück, nicht nach Ende des Purge. Standardfrist für die Rückgabe 60 s. `erase_sla` (Alarmschwelle für Fertigstellung) ist **Pflichtparameter ohne Standardwert**. | Entscheidungsgrundlage E-10 und Final E-SLA sind vereinbar in dieser Form. Ohne SLA-Wert ist Fail-closed. | Rückgabe erst nach Purge (blockiert Aufrufer unbeschränkt). | [E] |

### 3.2 Gate G0: Speicherentscheidung LSM oder redb

D-02 gilt **vorläufig**. Verbindlich wird sie mit Gate G0 nach Abschluss von WP-2.10 und WP-5.1 (Erasure Lab).

| Prüfung | Vorgabe | Schwelle für Verbleib LSM |
|---|---|---|
| Rohscan nach Erase | identischer Workload (Insert, Update, Erase, Restart, 10⁵ Dokumente) auf LSM und auf redb mit identischer Dokumenthülle | LSM-Rest ≤ redb-Rest; Restmenge nach Purge = 0 Treffer in Positivkontrolle-geprüftem Lauf |
| Schreibverstärkung | geschriebene Bytes je gelöschtem Byte | ≤ 10× Basislinie LSM-ohne-Erase, und ≤ redb-Vergleich |
| Korrektheit | Differential-Test gegen `ReferenceModel` (T-DIFF-1) | 0 Abweichungen über 10⁶ Operationen |
| Crash | Kill-Matrix (T-CRASH-1/2) | 0 verlorene bestätigte Transaktionen |

**Ausstiegsregel:** Verfehlt der LSM die Crash-Matrix (W2.10) zweimal nach einer Frist von vier Wochen oder überschreitet W2.4 seine Zeitbox, startet der redb-Adapter als Spike. Die Erase-Logik (Saga, Gate, Journal, Matcher) bleibt unverändert. Der Purge wird zum Datei-Rewrite der redb-Datei. Die Entscheidung wird mit Zahlen dokumentiert, nicht mit Präferenz.

### 3.3 ADR-001: Änderungen an `AGENTS.md`

`AGENTS.md` ist normativ. Diese Spezifikation ändert sie durch ADR-001 an folgenden Stellen:

| AGENTS-Stelle | Heutiger Wortlaut (Kurz) | Neue Regel |
|---|---|---|
| §1 Löschen | „Ebene B: kryptografische DocDek-Schlüsselvernichtung“ | Ebene B entfällt in v1 (D-01). Löschung = Ebene A (physisch) plus Gate und Messung. |
| §3 Ring-Tabelle | 35 Crates, u. a. `checkpoint`, `ingest`, `privacy` in Ring 1/2 | Ringe laut Abschnitt 5.2 dieser SPEC. |
| §4 Lösch-Saga | Zustände mit `KeyDestroyed` | Zustände laut Abschnitt 8.2. |
| §5 Sicherheits-Defaults | `AutoExtractionMode::Disabled`, `TenantPolicy::Required`, `Verschlüsselung Standard aktiv` mit `EncryptionMode::Required` | Verschlüsselung ohne Opt-out (D-05). `AutoExtraction` entfällt (D-27). |
| §2 Nr. 6 | `unsafe` in `simd`, `sys`, `wire` | `simd` und `sys` (D-29). |

**Widerspruch im Ist-Zustand:** AGENTS §5 beschreibt Defaults, die der Code nicht hat (Abschnitt 4.3). Das ist ein Befund, keine Zustimmung zum Ist-Zustand.

### 3.4 Konflikte zwischen den Quelldokumenten

| # | Konflikt | Auflösung in dieser SPEC | Beleg |
|---|---|---|---|
| K-1 | Entscheidungsgrundlage: 21 Workspace-Mitglieder; Fehlerkatalog: 37 | **37** am Clone | [V] `Cargo.toml` members = 37, default-members = 27 |
| K-2 | Entscheidungsgrundlage V-05: `Collection::delete` gibt Lock nicht frei, „nicht bestätigt“ | **Bestätigt**: Lock wird sofort wieder freigegeben | [V] `engine/src/collection/crud/delete.rs`: `let _guard = …lock_for(id).await; drop(_guard);` |
| K-3 | Final / Entscheidungsgrundlage F6: `scanner` und `hnsw_purger` nicht exportiert, Engine bricht | **Überholt**: Exporte vorhanden | [V] `store/src/lib.rs` `pub mod scanner;`, `vector/src/lib.rs:49` `pub use hnsw::{hnsw_purger, …}` |
| K-4 | Entscheidungsgrundlage: `test-utils` im Store undeklariert | **Nicht bestätigt**; `test-hooks` ist deklariert | [V] `store/Cargo.toml` Features; `encryption-at-rest` (11 Test-Dateien) und `wal-integrity` (7 Gates) sind undeklariert |
| K-5 | Entscheidungsgrundlage N1 „SstableBuilder::create im Produktionscode nicht mehr“ | Aufrufe in `purge.rs` nicht mehr geprüft, Rewrite wird ohnehin neu gebaut (WP-3.8) | [K] |
| K-6 | Fehlerkatalog: Workspace kompiliert nicht (Gruppe A) | **Lib kompiliert**, `--all-targets` mit 26 Fehlern | [U] `cargo check --workspace` fertig ohne Fehler (3 Warnungen); Audit: `check_alle_targets FAIL errors=26` |
| K-7 | Final: DocId 64 Bit mit Kollisionsprüfung nach ADR-016 | Kollisionsprüfung **nicht vorhanden**; ADR-016 betrifft Decay | [V] Suche nach ADR-016: nur Decay-Controller |
| K-8 | AGENTS §5 Default `AutoExtractionMode::Disabled`, `TenantPolicy::Required` | Code: `Enabled`, `Optional` | [V] Abschnitt 4.3 |
| K-9 | Erasure Core / AGENTS: Ebene B mit DocDek | D-01 streicht Ebene B für v1 | Abschnitt 3.1 |
| K-10 | Entscheidungsgrundlage E1 (blinde Nutzer-IDs: v1 nein) vs. Erasure Core E1 (v1 nein) vs. SOLL X-4 (v1.1) | **Ja in v1** (D-09, D-10). Begründung: Klartext-Rest in S4 ist ohne diese Entscheidung nicht ausschließbar. | Abschnitt 3.1 |
| K-11 | Restrict in v1 (Erasure, Erasure Core) vs. v1.1 (Final) vs. Pflichtparameter (SOLL R9) | Pflichtangabe `RetentionAttestation`, kein Restrict (D-15) | Abschnitt 3.1 |
| K-12 | Bericht signiert (Erasure Core, SOLL) vs. Final T-REPORT-2 „falls signiert“ | **Signiert** (D-13) | Abschnitt 3.1 |
| K-13 | Journal im Store (Final) vs. eigene Datei mit Hash-Kette (SOLL 4.10) | Eigene Datei `erasure/journal.log`, verschlüsselt, Hash-Kette, `fsync` je Eintrag (Abschnitt 8.5) | Abschnitt 8.5 |
| K-14 | Aufwand: 13–16 Wochen (Bericht 3) vs. 4–7 Monate Teilzeit (Final) | Eigene Schätzung in PLAN.md, mit Begründung | PLAN.md §6 |

---

## 4. Ausgangszustand (verifiziert)

### 4.1 Workspace und Build

| Punkt | Befund | Beleg |
|---|---|---|
| Mitglieder / Default | 37 / 27. Nicht in default: `privacy`, `bench`, `infer-candle`, `sandbox`, `py`, `audit-export`, `avv-generator`, `license`, `ingest`, `tests/arch` | [V] `Cargo.toml` |
| Toolchain | `1.89.0`, Komponenten `rustfmt`, `clippy` | [V] `rust-toolchain.toml` |
| `cargo check --workspace` | erfolgreich, 3 Warnungen (deprecated `search_filtered_at` im Engine-Hybrid-Query, ungenutzter Import in `ingest`, ungenutztes `PagedChunker`, ungenutzter `PyKeyError`) | [U] |
| `cargo check --workspace --all-targets` | 26 Fehler | [U] Audit `check_alle_targets` |
| `clippy` | Fehler in fast allen Crates, teils >300 pro Crate | [U] Audit `clippy_lib__*` |
| `fmt --check`, `doc` | FAIL | [U] Audit |
| `feature_matrix_hack` | Lauf unvollständig im Log | [U] |
| Audit-Befunde (Auszug) | undefinierte Features (1 Crate), `safe`-Funktionen mit unsafe-Inhalt (2 bekannte Stellen: `mmap_readonly`, `reopen_and_dup2`), 4 Produktivdateien mit `from_entropy` (K14), Default-Members unvollständig | [U] |

### 4.2 Sicherheitsbefunde (Kern, am Clone geprüft)

| ID | Befund | Beleg [V] | Status |
|---|---|---|---|
| **D-C02** | Snapshot-Sentinel: `prune_bound` fällt auf `u64::MAX`, `prune_through` maskiert Tombstone-Bit → `i64::MAX`, `fetch_max` macht es permanent. Danach scheitern SSI-Commits mit „snapshot too old“. | `store/src/lsm/ops/compaction.rs:225–231`; `mvcc/src/ssi.rs:646–652` (`bound_seq & !TOMBSTONE_BIT`, `fetch_max`) | **bestätigt, kritisch** |
| **WAL-Integrität abgeschaltet** | `#![allow(unexpected_cfgs)]` hebt das Workspace-`deny` auf. `wal-integrity` ist nirgends deklariert. Sieben Gates in `wal/replay.rs` (Zeilen 5, 10, 250, 332, 334, 434, 682; Zeile 682 als `not`-Zweig). | `store/src/lib.rs:45`; `store/src/wal/replay.rs`; `Cargo.toml:194` (`unexpected_cfgs = deny`) | **bestätigt, kritisch** |
| **Legacy-WAL akzeptiert** | `WalVersion::V1` (ohne HMAC) wird weiter verarbeitet, mit Hinweis auf öffentlichen Rückfallschlüssel | `store/src/wal/io.rs:43, 267, 284, 294, 415` | bestätigt, hoch |
| **Verschlüsselung Default aus** | `encryption_mode: None`; Store `encryption_passphrase: None` | `contextra/src/builder.rs:147`; `store/src/lsm/config.rs:89,111` | bestätigt, kritisch für Mission |
| **Tenant Default optional** | `#[default] Optional` | `engine/src/lib.rs:207–209` | bestätigt, hoch |
| **AutoExtraction Default an** | `Enabled`, solange Feature `auto-extraction-opt-out` fehlt. Feature in `engine` leer. | `types/src/types/domain/misc.rs:591–597` | bestätigt, hoch |
| **Argon2-Untergrenze** | `MIN_M_COST_KIB = 19456` | `crypto/src/kdf.rs:36` | bestätigt, hoch |
| **HKDF für Passphrase** | `try_new(passphrase, salt)` mit HKDF-SHA256 | `crypto/src/crypto.rs:88` | bestätigt, kritisch |
| **Lock-Bug Delete** | Lock wird vor der Operation freigegeben | `engine/src/collection/crud/delete.rs` | bestätigt, hoch (Race bei Parallel-Write) |
| **Drop ohne Shutdown** | `Drop for LsmStorage` ruft nur `cancel_token.cancel()`; Hintergrundtasks und Verzeichnislock überleben | `store/src/lsm/engine.rs:75–79` | bestätigt, mittel |
| **Follower-Commit nicht cancel-safe** | Timeout-Zweig vorhanden, zweites `rx.await` ohne Timeout; Cancel-Guard nur im Leader-Pfad | `store/src/lsm/ops/write.rs` (Timeout ca. Zeile 524, `rx.await` ca. Zeile 564, `LeaderCancelGuard` ca. Zeile 610) | bestätigt, kritisch (Teilbestätigung, Chaos-Test offen) |
| **Zwei `Vfs`-Traits; Store ohne Vfs** | `ports/src/vfs.rs:29`; `durable-fs/src/vfs.rs:54`; `Vfs` im Store: 0 Treffer | [V] | bestätigt, hoch |
| **Direkte Dateizugriffe im Store** | 83 Treffer `tokio::fs::`/`std::fs::` in 18 Produktionsdateien (grep, inkl. inline Tests) | [V] | bestätigt |
| **Purge ignoriert Ziel** | `LsmPurger::purge(&self, ctx, _target)`; Filter `scan_prefix` auf Mandant | `store/src/scanner/purge.rs:36,44` | bestätigt, kritisch (Löschpfad, entfällt mit Rückbau) |
| **Vorhandene Löschtypen** | `deletion_proof/` (7 Dateien), `kv_shredding.rs`, `kv_segment/`, `kv_cipher.rs`, `revocation_log.rs`, `engine/src/contextra_impl/erasure.rs` (616 Zeilen), `store/src/scanner/` | [V] Existenz | Rückbau (Abschnitt 5) |
| **SIMD u8** | `_mm512_dpbusd_epi32(acc, va, vb)` mit u8/u8; Instruktion erwartet u8×i8 | `simd/src/kernels/avx512.rs:445,565–569` | bestätigt (Muster), falsche Distanzen wahrscheinlich [U nicht getestet] |
| **DocId 64 Bit, unkeyed** | `DocId(pub u64)` ohne Feature `docid-128` | `types/src/types/domain/ids.rs:173` | bestätigt |
| **`expect` im Produktionspfad** | Tokio-Runtime-Erzeugung im Scanner | `store/src/scanner/sstable.rs:35,47` | bestätigt |
| **`HashMap`/`HashSet` mit RandomState** | Manifest-Bereinigung | `store/src/manifest/core.rs:284, 338–342` | bestätigt (P28, Ergebnisse sortiert) |
| **Undeklarierte Feature-Gates unentdeckt** | `feature-matrix` und `cargo-hack-feature-check` laufen mit `continue-on-error: true` | `.github/workflows/rust-ci.yml` (Jobs u. a. `feature-matrix`, `cargo-hack-feature-check`, `supply-chain`, `coverage`, `miri-*`, `asan-*`, `test-py`, `test-cross-platform`) | bestätigt, Ursache für Unentdeckbarkeit |
| **Fehlende Dokumentziele** | `docs/decisions/` fehlt; `docs/PLAN.md` fehlt; `engine/src/collections.rs` fehlt (von Löschdokumenten referenziert) | [V] | bestätigt, hoch (Doc-Drift) |
| **`cargo-deny` Ignorierungen** | 6 Advisories ignoriert (RUSTSEC-2024-0384, -2024-0436, -2026-0176, -2026-0177, -2026-0285, -2026-0205) ohne Ablaufdatum | `deny.toml:9–14` | bestätigt |

### 4.3 Befunde, die überholt sind

| ID | Quelle | Stand am Clone |
|---|---|---|
| F6 | Audit 2 (Build-Blocker `scanner`, `hnsw_purger`) | **überholt**: Exporte vorhanden |
| V-07 | Final (KV-Cache verdrahtet) | `kvcache` ist Workspace-Mitglied, aber nicht in `default-members`; Entfernung in WP-1.2 |
| F-19 | Bericht 3 (`default-members` ≠ `members`) | Unterschied besteht (10 Crates), Bericht veraltet |
| R-04 | Fehlerkatalog (SurrogateVault 16 Bit) | [K] nicht am Clone geprüft; mit `privacy`-Entfernung irrelevant für v1 |

### 4.4 Konsumentenanalyse (Scope-Grundlage)

| Crate | Konsumenten am Clone [V] | Folge |
|---|---|---|
| `contextra-wire` | core, mcp, py, router | entfällt mit MCP/Python/Router (D-25, D-29) |
| `contextra-core` | nur `benchmarks/contextra-bench` | entfällt |
| `contextra-graph` | engine (18 Dateien), cognition, db, agent, router, bench; **store dev-dep** | Relations-API und Graph entfernen (D-17); store-Dev-Abhängigkeit streichen |
| `contextra-checkpoint` | engine (lifecycle, diagnostics), db, agent | Engine-Hooks entfernen (D-16) |
| `contextra-rank` | engine (`fuse_*`, `IsotonicCalibrator`), db, contextra, mcp, py, infer-* | auf RRF reduzieren (D-26) |
| `contextra-privacy` | mcp, router, contextra | entfällt (Fassade entkoppeln) |
| `contextra-license` | mcp, contextra | entfällt (D-23) |
| `contextra-db` | agent, py, router, contextra, bench | prüfen: Inhalt vollständig in Engine/Fassade überführbar (WP-1.1) |
| `contextra-ingest` | nur sich selbst | entfällt, keine Verwender |
| `contextra-testkit` | viele, nur dev | bleibt als Dev-Abhängigkeit |
| `contextra-sys` | db, durable-fs, engine, store, vector, bench | bleibt (unsafe-Insel) |
| `contextra-simd` | simd, vector | bleibt (unsafe-Insel, Kernel-Reduktion D-28) |

Vollständige Entfernung wird durch `cargo tree -p contextra -e normal --prefix none | sort -u` nach jedem Schritt bestätigt (WP-1.1, Abnahme).

---

## 5. Scope v1

### 5.1 Crate-Entscheidungen

| Crate | Entscheidung | Begründung (Verweis) |
|---|---|---|
| `contextra-types` | **behalten**, Löschtypen ohne I/O ergänzen | Typfamilie je Konzept (A2). Keine `async`, keine I/O (T-ARCH-1). |
| `contextra-ports` | **behalten**, `Vfs` konsolidieren | D-04 |
| `contextra-sys` | **behalten** (unsafe-Insel) | fsync-Primitive, `flush_dir`, `rename_durable` |
| `contextra-simd` | **behalten, reduziert** | D-28 |
| `contextra-durable-fs` | **behalten**, Implementierung von `Vfs`; `scrub.rs` und `PendingDeleteJournal` nach Test prüfen | D-04 |
| `contextra-crypto` | **behalten, reduziert**: AEAD, KDF (Argon2id), Schlüsselhierarchie, WAL-HMAC, Hash-Kette, Ed25519-Report | entfernt: `deletion_proof/`, `ed25519_proof.rs`, `kv_*`, `revocation_log.rs` |
| `contextra-mvcc` | **behalten** | Zeitreisen, SSI. D-11, Sentinel-Fix (WP-2.1) |
| `contextra-store` | **behalten, gehärtet** | D-02. Entfernt: `scanner/` (alt), `kv/`. Neu: `erase/`. |
| `contextra-vector` | **behalten, reduziert** | D-18, D-19. Entfernt: `hnsw/deletion.rs`, `hnsw/purge.rs` (alt, ersetzt). |
| `contextra-text` | **behalten, umgebaut** | D-20. Entfernt: `inverted/purge.rs`, `scanner.rs` (alt). |
| `contextra-rank` | **behalten, reduziert auf RRF** | D-26 |
| `contextra-engine` | **behalten, umgebaut** | Saga, Journal, Gate, Subject-Index. Entfernt: `contextra_impl/erasure.rs`, `relate`, Checkpoint- und Graph-Hooks. |
| `contextra-db` | **prüfen (WP-1.1)**, Default: in Fassade überführen | Keine Eigenständigkeit im Kern belegt |
| `contextra` (Fassade) | **behalten** | öffentliche API (Abschnitt 6) |
| `contextra-testkit` | **behalten, dev-only** | Erasure Lab (Abschnitt 9.2) |
| `tests/arch` | **behalten** | Architekturtests T-ARCH-1/2 |
| `contextra-graph` | **entfernen** (Archiv) | D-17 |
| `contextra-checkpoint` | **entfernen** (Archiv) | D-16 |
| `contextra-privacy` | **entfernen** (Archiv) | D-24 |
| `contextra-ingest` | **entfernen** (Archiv) | D-24 |
| `contextra-cognition`, `contextra-adapt`, `contextra-agent`, `contextra-router` | **entfernen** (Archiv) | D-24 |
| `contextra-kvcache` | **entfernen** (Archiv) | D-13, Cache-Verbot |
| `contextra-infer-candle`, `contextra-infer-ollama`, `contextra-infer-onnx` | **entfernen** (Archiv) | D-24 |
| `contextra-sandbox`, `contextra-audit-export`, `contextra-avv-generator` | **entfernen** (Archiv) | D-24 |
| `contextra-license` | **entfernen** (Archiv) | D-23 |
| `contextra-mcp` | **entfernen**, v1.1-Adapter | D-25 |
| `contextra-py` | **entfernen**, v1.1-Adapter | D-25 |
| `contextra-wire`, `contextra-core` | **entfernen** (Archiv) | Konsumentenanalyse 4.4 |
| `benchmarks/contextra-bench` | **entfernen aus Workspace** (Archiv), Store-Benchmarks bleiben in `store/benches` | Nicht kompilierbar [K: A-B06], misst Retrieval-Qualität, nicht den Store |

**Ergebnis:** 13 Produktions-Crates im Workspace (einschließlich `types`, `ports`, `sys`, `simd`, `durable-fs`, `crypto`, `mvcc`, `store`, `vector`, `text`, `rank`, `engine`, `contextra`) plus `testkit` (dev) und `tests/arch`. Gegenüber 37 Mitgliedern entspricht das einer Reduktion um ca. 60 %.

### 5.2 Ringe v1

| Ring | Crates | Regel |
|---|---|---|
| 0 | `types`, `ports` | keine I/O, keine `async` in `types` |
| 1 | `sys` [unsafe], `simd` [unsafe], `durable-fs`, `crypto` | Kein Abhängigkeitspfad nach oben |
| 2 | `mvcc`, `store`, `vector`, `text`, `rank` | `store` kennt weder `vector` noch `text` (LSM-Kern bleibt blind, Quelle Umsetzungsplan) |
| 3 | `engine` | Orchestrierung, Saga, Gate |
| 4 | `contextra` (Fassade) | einzige öffentliche Oberfläche |

Abhängigkeiten zeigen nur nach unten. Verletzung ist Build-Fehler (Architekturtest T-ARCH-3, neu).

### 5.3 Funktionsumfang v1

| Bereich | Funktion |
|---|---|
| Speichern | Collection, Insert, Update, Delete (logisch, mit Löschsemantik siehe 8), Get, Scan (Schlüsselreihenfolge) pro Mandant und Collection |
| Suche | BM25 (Volltext), HNSW (Vektor), RRF (Fusion). Vektoren vom Aufrufer. |
| Betroffene | `subject_id` optional beim Insert, Index `subject_tag → DocIds` |
| Zeitreisen | `get_at_seq`, `scan_at_seq` (Vorgabe) |
| Transaktionen | MVCC, SSI-Commits, atomare Batches |
| Löschung | `erase(EraseTarget)` für Dokument, Betroffener, Collection; `erasure_status`; `verify` |
| Nachweis | `ErasureReport` (signiert), Verifier-Beispiel mit Testvektoren |
| Checkliste | Textexport mit Prüfpunkten außerhalb von Contextra (Abschnitt 13.3) |
| Betrieb | Öffnen mit Passphrase, Recovery beim Start, `close()`, Metriken ohne IDs |

### 5.4 Budgets (CI-Gates)

| Budget | Wert | Messung |
|---|---|---|
| Erase-Produktcode (`store/src/erase`, `engine/src/erasure`, `crypto/src/proof`, `types/src/erasure`) | ≤ 2.500 Zeilen | `tokei` in CI |
| Gesamt-Produktcode Kern (ohne Tests) | Zielreduktion ≥ 40 % gegenüber Baseline WP-0.1 | `tokei` in CI, Baseline in `evidence/` |
| Clippy-Meldungen Kern | 0 | `cargo clippy --workspace --all-targets -- -D warnings` |

---

## 6. Öffentliche API (Rust)

Alle Typen mit `#[non_exhaustive]` außer Konfigurationsbausteinen. Keine öffentlichen `String`-Felder für Schlüssel oder Passphrasen.

```rust
// Öffnen. Pflichtfelder fehlen → Fehler (fail-closed), kein Standardwert.
pub struct EngineConfig { /* privat, über Builder */ }
pub struct EngineBuilder;
impl EngineBuilder {
    pub fn new(data_dir: impl AsRef<Path>) -> Self;
    pub fn passphrase(self, secret: SecretBytes) -> Self;      // Pflicht
    pub fn erase_sla(self, sla: Duration) -> Self;              // Pflicht, kein Standard
    pub fn erase_return_timeout(self, t: Duration) -> Self;     // Standard 60 s
    pub fn batch_window(self, w: Duration) -> Self;             // Standard 60 s
    pub async fn open(self, vfs: Arc<dyn Vfs>, clock: Arc<dyn Clock>) -> Result<Contextra, ContextraError>;
}

pub struct Contextra { /* … */ }
impl Contextra {
    pub async fn collection(&self, ctx: &TenantContext, name: &CollectionName) -> Result<Collection, ContextraError>;
    pub async fn erase(&self, ctx: &TenantContext, req: EraseRequest) -> Result<ErasureHandle, ContextraError>;
    pub async fn erasure_status(&self, id: ErasureId) -> Result<ErasureStatus, ContextraError>;
    pub async fn verify(&self, ctx: &TenantContext, target: &EraseTarget) -> Result<VerificationReport, ContextraError>;
    pub fn erasure_checklist(&self) -> String;
    pub async fn close(self) -> Result<(), ContextraError>;     // explizit, siehe 11.3
}

pub enum EraseTarget {
    Document { collection: CollectionName, doc: DocKey },
    Subject  { subject: SubjectId },
    Collection { collection: CollectionName },
}

pub struct EraseRequest {
    pub target: EraseTarget,
    pub retention: RetentionAttestation,   // Pflicht, D-15
}

pub enum RetentionAttestation {
    /// Keine Aufbewahrungspflicht bekannt. Der Betreiber bestätigt dies.
    NoObligationConfirmed,
    /// Aufbewahrungspflicht geprüft. Die Löschung wird verweigert, solange sie gilt.
    /// Der Code erfasst Gründe als Enum (keine Freitexte, D-15).
    ObligationDeclinesErasure { code: RetentionCode },
}

pub struct ErasureHandle { /* … */ }
impl ErasureHandle {
    pub fn id(&self) -> ErasureId;
    pub async fn status(&self) -> Result<ErasureStatus, ContextraError>;
    pub async fn wait(&self, timeout: Duration) -> Result<ErasureStatus, ContextraError>;
}

pub enum ErasureStatus {
    Gated,                                  // Sperre persistiert, Purge läuft
    Pending { attempt: u32 },               // Fehler in Phase, Wiederholung geplant
    Completed(ErasureReport),               // gemessen, Bericht signiert
    Refused { reason: RefusalReason },      // fail-closed, z. B. NoManifest
}
```

**Regeln:**

- `erase` gibt `Gated` zurück, sobald die Sperre persistiert ist (Latenzziel p50 ≤ 10 ms). Die Fertigstellung erfolgt asynchron.
- `ErasureStatus::Completed` nur nach Messung aller Pflichtschichten (Abschnitt 9).
- `ErasureStatus::Refused` ist terminal und ändert keinen Datenbestand (z. B. `NoManifest` bei Altbestand).
- Ohne `TenantContext` existiert kein Zugriff auf `Collection` (D-07).

### 6.1 Fehlertaxonomie

```rust
#[non_exhaustive]
pub enum ContextraError {
    Config(ConfigError),          // fehlende Pflichtangabe, zu schwache KDF-Parameter
    Auth(AuthError),              // falsche Passphrase, fehlender Mandant
    Integrity(IntegrityError),    // WAL-HMAC, AEAD, Manifest, Journal-Kette
    Io { op: IoOp, source: io::ErrorKind }, // ohne Pfad und ohne Inhalt
    Conflict(ConflictError),      // IdCollision, Schreibkonflikt SSI
    SnapshotRevoked { seq: u64 },
    Erased,                       // Ziel gesperrt (Gate)
    Unsupported(&'static str),
    Closed,
}
```

Fehlermeldungen enthalten **nie** Schlüssel, Inhalte, Nutzer-IDs oder Pfade mit personenbezogenen Namen (VETO-FAILCLOSED, T-LOG-1).

---

## 7. Datenmodell

### 7.1 Schlüsselraum

Jeder Schlüssel ist ein Byte-Array fester Struktur, big-endian, ohne Trennzeichen-Parsing:

```
key   = family:u8 ‖ tenant_tag:[u8;16] ‖ coll_tag:[u8;16] ‖ payload
family ∈ { Doc=0x01, Manifest=0x02, DocMap=0x03, Text=0x04, Df=0x05,
           Subject=0x06, Vec=0x07, Journal=0x08 (außerhalb LSM), Meta=0x09 }
```

| Familie | payload | Wert (verschlüsselt, AEAD) |
|---|---|---|
| `Doc` | `doc_id:[u8;16]` | Dokument-Inhalt, Metadaten, `doc_key` |
| `Manifest` | `doc_id` | Liste aller Schlüssel, die das Dokument besitzt (Abschnitt 7.3) |
| `DocMap` | `doc_id` | `slot:u32`, Version, Subject-Tag |
| `Text` | `term_tag:[u8;16] ‖ doc_id` | `tf:u32` |
| `Df` | `term_tag` | `df:u64` (Zähler) |
| `Subject` | `subject_tag:[u8;16] ‖ doc_id` | leer |
| `Vec` | `slot:u32` | Vektor-Metadaten (Vektorinhalt liegt in der Arena, D-19) |
| `Meta` | `name_tag` | Collection-Metadaten, Format-Version |

Alle `*_tag` sind `BLAKE3-keyed(K_idx, label ‖ Eingabe)[0..16]`, Labels fest definiert (Konstanten-Enum, kein Freitext).

### 7.2 Dokumentidentität

`doc_id = BLAKE3-keyed(K_idx, "ctx/doc/v1" ‖ tenant ‖ collection ‖ doc_key)[0..16]` (D-08).

Insert-Ablauf: Lese `DocMap[doc_id]` → existiert und `doc_key` (entschlüsselt) ≠ neuer Schlüssel → `IdCollision`. Gleich → Update. Der Vergleich erfolgt im Speicher mit `Zeroizing`-Puffern.

### 7.3 Manifest

Pro Dokument ein `Manifest`-Eintrag, in **derselben Transaktion** wie `Doc`, `DocMap`, `Text`, `Df`, `Subject`, `Vec`. Inhalt: Liste aller Schlüssel dieses Dokuments (exakte Bytes, verschlüsselt). Löschung liest das Manifest **vor** jeder Mutation.

- Fehlt das Manifest (Altbestand), antwortet `erase` mit `Refused(NoManifest)`. Es gibt keinen Vollscan-Fallback (P3). Migration: Abschnitt 15, Frage F-LEG.
- Update entfernt im selben Batch die Schlüssel alter Terme, die nicht mehr vorkommen, und aktualisiert das Manifest.

### 7.4 Text und BM25

- Posting `Text(term_tag, doc_id) → tf`.
- `Df(term_tag)` als Zähler, in derselben Transaktion wie die Postings aktualisiert.
- Suche: für jeden Query-Term Präfix-Scan auf `Text(term_tag, *)`. Dies ist der **einzige** erlaubte Präfix-Scan, weil `term_tag` fixe Länge hat (D-10).
- Aggregierte `Df`-Werte enthalten keine Nutzerinhalte, nur Zählwerte. Ob sie personenbezogen sind, ist Prüfpunkt G-LEGAL (Abschnitt 15).

### 7.5 Vektoren

- HNSW-Arena: Slots fester Größe, Vektor in f32. Slot-Zuordnung über `DocMap`.
- Löschung: Vektorbytes des Slots mit `zeroize` überschreiben, Slot in Tombstone-Liste, `DocMap` entfernt.
- Kanten enthalten nur `slot:u32` (keine Vektorinhalte). Bleiben bis zum Rebuild als Restposten (`GraphRebuildPending`, Abschnitt 9.4).
- Snapshot-Dateien: verschlüsselt (D-19), über `Vfs`, bei Löschung neu geschrieben.

### 7.6 Subject-Index

`Subject(subject_tag, doc_id)`. Löschung nach `EraseTarget::Subject` liest die Einträge, bildet die Dokumentliste und führt je Dokument die Dokumentlöschung aus (ein Batch, D-14).

---

## 8. Löschmodell

### 8.1 Begriffe

| Begriff | Definition |
|---|---|
| **Ziel** | Typisierter `EraseTarget`, aufgelöst zu einer Menge von `doc_id`s und einer Menge exakter Schlüssel (Manifeste) |
| **Gate** | Persistierte Sperre (`ErasedSet`-Eintrag). Ab hier liefert jeder Lesepfad `Erased`. |
| **Schicht** | Ort, an dem Ziel-Daten liegen können (Abschnitt 9.4) |
| **Zeuge** | Ergebnis eines unabhängigen Scanners für eine Schicht (`LayerWitness`) |
| **Klasse** | Aussagestufe des Berichts (Abschnitt 9.3) |

### 8.2 Zustandsmaschine

| Zustand | Dauerhafte Wirkung | Übergang | Bei Fehler |
|---|---|---|---|
| `Requested` | Journal-Eintrag mit Ziel-Commitment (verschlüsselt) | Gate setzen | `Pending` |
| `Gated` | `ErasedSet` enthält Commitment, Journal-Eintrag `gated` | Drain | `Pending` (Gate bleibt) |
| `Drained` | Memtables geflusht, versiegelte WAL-Dateien entfernt, Fehlen geprüft | Rewrite | `Pending` |
| `Rewritten` | betroffene SSTables neu, Manifest „ersetze alt durch neu“ committed | Unlink | `Pending` |
| `Unlinked` | alte SSTables entfernt, Fehlen geprüft | Index-Bereinigung (HNSW-Slots, Blockcache) | `Pending` |
| `IndexesCleaned` | Vektor-Zeroing, Blockcache-Eviction | Messung | `Pending` |
| `Verified` | Rohscan ohne Befund, Positivkontrolle gefunden | Bericht | `Pending` (Messung wiederholbar) |
| `Completed` | Bericht signiert, Journal auf Minimalform (ID, Zeit, Ergebnis) reduziert | terminal | – |

**Pending** ist ein Zustand mit `attempt` und `next_retry_at`. Er meldet nie Erfolg. Überschreitung von `erase_sla` erzeugt Alarm (Metrik und Audit-Eintrag), keinen Abbruch (SOLL 4.11).

**Refused** ist terminal und ändert nichts (`NoManifest`, `ObligationDeclinesErasure`, Ziel unbekannt ohne Fehler: `Completed` mit leerer Trefferliste, gemessen).

### 8.3 Gate und Lesepfade

- Das Gate ist der `ErasedSet`-Eintrag. Er wird **vor** `erase` zurückkehrt persistiert (fsync).
- `TenantScopedStorage` ist der einzige Zugang zum Store (T-ARCH-2). Alle Lesepfade prüfen den `ErasedSet`: `get`, `scan`, `get_at_seq`, `scan_at_seq`, Suche (Text, Vektor).
- Treffer liefert `Erased`. Zeitreise-Lesezugriffe werden nicht ausgenommen (D-11, D-12).
- Snapshots, die eine betroffene Sequenz pinnen, werden widerrufen (`SnapshotRevoked`). Widerruf ist ein Journal-Eintrag.
- `ErasedSet` wird beim Start vor Freigabe der Lesepfade geladen (`recover()`, Abschnitt 8.6).

### 8.4 Purge-Compaction

Modul: `contextra-store::erase`.

**Barrier.** Ein `RwLock` trennt Compaction (shared) und Purge (exklusiv pro Purge-Lauf). Reads und Writes sind nicht blockiert. Warten auf die Barrier ist mit Frist versehen; Überschreitung ist `Pending`.

**Drain.**
1. `rotate_and_seal` des aktiven WAL.
2. Flush aller Memtables (inkl. immutable).
3. Versiegelte WAL-Dateien, deren Inhalt vollständig in SSTables liegt, entfernen.
4. Fehlen jeder Datei per `Vfs::exists` prüfen. Eine verbleibende Datei ist `Pending`, nie Warnung.

**Kandidaten.** SSTables, deren Schlüsselbereich mindestens einen Matcher-Schlüssel enthält. Bloom-Filter als Vorfilter. Exakte Bestätigung per Stream.

**Streamender Rewrite.**
1. Builder aus dem Quell-Reader ableiten (`create_like`): gleicher Schlüsselkontext, gleiche Formatversion, Schlüsselkontext aus dem Store-Schlüssel.
2. Iteration über alle Einträge in Schlüsselreihenfolge. Einträge mit Schlüssel ∈ Matcher werden **alle Versionen und Tombstones** verworfen (D-11).
3. Übrige Einträge mit Originalwerten `seq`, `tx_id`, `flags` übernehmen. Keine Neunummerierung (behebt N1).
4. Kein Vollladen. Speicherbedarf ≤ Konstante × Blockgröße.
5. Schreibfehler: Abbruch. Alter Reader bleibt gültig. Temp-Datei wird als Waise markiert und beim Start entfernt.

**Atomarer Tausch.**
1. `tmp` schreiben, `fsync` Datei.
2. Manifest-Eintrag „ersetze {alt} durch {neu}“ anhängen, `fsync` Manifest.
3. `fsync` Verzeichnis.
4. Reader tauschen.
5. Alte Datei entfernen (`Unlink`), danach `fsync` Verzeichnis und Fehlen prüfen.

**Blockcache.** Einträge ersetzter Dateien werden entfernt und mit `zeroize` überschrieben, vor dem Unlink.

**Matcher.** `Matcher = Set<[u8]>` exakter Schlüssel (D-10). Keine Präfixe, keine Stringvergleiche.

### 8.5 Invarianten

| ID | Invariante | Testbeleg |
|---|---|---|
| I-1 | Jede Zustandsänderung wird vor der Aktion im Journal mit `fsync` festgehalten. | T-CRASH-1 |
| I-2 | Rewrite übernimmt `seq`, `tx_id`, `flags` unverändert. | T-VER-1 |
| I-3 | Alter Reader bleibt bis zum Manifest-Commit gültig. | T-CRASH-3 |
| I-4 | Unlink nur nach Manifest-`fsync` und Verzeichnis-`fsync`. | T-CRASH-1 |
| I-5 | Kein Gate ohne persistierten Eintrag; kein Purge ohne Gate. | T-CONC-1 |
| I-6 | Nachbar-Daten bitgleich (NeighborDigest). | T-ISO-1 |
| I-7 | `Completed` nur mit Zeugen aller Pflichtschichten. | T-PROOF-1, T-RAW-1 |
| I-8 | Kein Fehler wird verschluckt. Jede Fallible-Operation liefert `Result`, das propagiert oder als `Pending` persistiert wird. | Lint T-LINT-2 |

### 8.6 Journal und Dateien

Pfad: `<data>/erasure/`.

| Datei | Inhalt | Lebenszyklus |
|---|---|---|
| `journal.log` | Append-only. Pro Eintrag: Länge, Typ, Nutzlast (verschlüsselt), CRC, Hash der Vorgängerkette. `fsync` je Eintrag. | Nach `Completed` auf Minimalform reduziert (ID, Zeit, Ergebnis, Commitment). Keine Zielkennung mehr. |
| `scope/{erasure_id}` | Auflösung des Ziels (Dokument-IDs, Schlüsselliste), verschlüsselt mit `K_journal` | Nach `Completed` entfernt, Fehlen geprüft |
| `erased.set` | sortierte Commitments mit `erased_through_seq`. Atomar ersetzt (Temp, `fsync`, Rename, Verzeichnis-`fsync`). | dauerhaft (Commitment, kein Klartext) |
| `keys/` | Verzeichnis 0700, Dateien 0600 | Start verweigert bei lockereren Rechten (P2) |
| `report/` | signierte Berichte (`ctx-erasure-report/1`) | dauerhaft, enthält keinen Personenbezug |

**Commitments.** `commitment = BLAKE3-keyed(K_commit_epoch, canonical(target))`. `K_commit_epoch` wird pro Epoche rotiert. Der Epochen-Schlüssel liegt im Schlüsselverzeichnis. Lebensdauer ≥ `erased_set_retention` (Frage F-COMMIT).

### 8.7 Wiederanlauf `recover()`

Reihenfolge, vor Freigabe jedes Lesepfads:

1. Dateisystem-Probe: atomares Rename, Verzeichnis-`fsync`. `EIO` → Start verweigert.
2. Schlüsselverzeichnis prüfen (Rechte, Vorhandensein). Fehlt ein Schlüssel → Start verweigert.
3. `erased.set` laden, Gate scharf schalten.
4. Journal lesen, Hash-Kette prüfen. Defekte Kette → Start verweigert (P2).
5. Waisen entfernen (`.tmp`, Ausgabedateien ohne Manifest-Eintrag).
6. Offene Sagas in ihrem Zustand fortsetzen (Idempotenz je Schritt).
7. Erst danach Lesen und Schreiben freigeben.

Unklarer Zustand heißt: Lesen verweigern (P2, INV-FAILCLOSED-1).

### 8.8 Nebenläufigkeit

- Pro Dokument ein Lock, derselbe für Writes und Erase (behebt N7).
- Gate-Setzen und logische Sperre laufen unter diesem Lock.
- Batch-Fenster (Standard 60 s): Aufträge werden zu einem Purge-Lauf zusammengefasst, dedupliziert nach Commitment. `Timing::Now` erzwingt sofortigen Lauf und weist die Zusatzkosten im Bericht aus.
- Rewrite-Kosten skalieren mit der Zahl betroffener SSTables, nicht mit der Gesamtdatenmenge (Gate, Abschnitt 12.4).

### 8.9 Cancel-Safety

Alle mutierenden Operationen laufen in einer vom Aufrufer unabhängigen Task (`tokio::spawn` auf dem Engine-Runtime-Handle). Der Aufrufer hält ein Handle auf das Ergebnis. Abbruch des Aufruferfutures ändert den Ausgang nicht und lässt keinen Zustand halb zurück (behebt D-C01). Ein Test bricht jeden `await`-Punkt ab (T-CANCEL-1).

---

## 9. Nachweis und Messung

### 9.1 Löschbericht `ctx-erasure-report/1`

Kanonisches JSON (Schlüssel sortiert, UTF-8, keine Gleitkommazahlen). Felder:

| Feld | Inhalt |
|---|---|
| `report_id`, `erasure_id` | 128 Bit, OsRng |
| `scope_kind` | `document` \| `subject` \| `collection` |
| `subject_commitment` | Commitment, keine Kennung |
| `retention` | `NoObligationConfirmed` \| `ObligationDeclinesErasure` mit Code (nur bei Ablehnung) |
| `times` | `requested_at`, `gated_at`, `drained_at`, `rewritten_at`, `measured_at` (aus `Clock`) |
| `layers[]` | je Schicht: `layer_id`, `status`, `witness_digest` (BLAKE3 über Scan-Protokoll: Dateizahl, Bytezahl, Needle-Kategorien, `positive_control_found`), `encodings_searched` |
| `store_counters` | Dateien neu geschrieben, Bytes geschrieben, WAL-Segmente entfernt, widerrufene Snapshots (nur Zahlen) |
| `residual_risks[]` | Liste aus 9.4 |
| `provenance` | Contextra-Version, Build-ID, Digest der effektiven Konfiguration (ohne Geheimnisse) |
| `class` | aus `classify` (9.3) |
| `signature` | Ed25519 über `canonical(report ohne signature)` |

Der Bericht enthält nie Dokumentinhalt, nie Schlüssel, nie `doc_id` im Klartext, nie `subject_id`.

### 9.2 Messverfahren

| Komponente | Zweck | Unabhängigkeit |
|---|---|---|
| `ForensicScanner` | Rohscan aller Dateien im Datenverzeichnis nach Needles (UTF-8, UTF-16LE, Base64, Hex), je Datei und Offset | kein SSTable- oder WAL-Parser, nur Bytes |
| `HeapScanner` (Linux) | `/proc/self/mem` nach Needles | eigener Code |
| `CanaryFactory` | hochentropische Needles pro Lauf (Seed protokolliert) | nur Testmandant |
| Needle-Quellen | Dokumentinhalt, `doc_key`, abgeleitete Blind-Schlüssel (`Text`, `DocMap`, `Subject`, `Manifest`) | Needles nur im Speicher, `zeroize` nach Scan |
| `FaultVfs` | Absturz nach Operation *n*, abgerissener Write an Byte *b*, `ENOSPC`, `EIO` bei `fsync` | Testkit |
| `ReferenceModel` | In-Memory-Orakel (BTreeMap) | Testkit |
| `RecordingStorage` | protokolliert jeden geschriebenen Schlüssel je öffentlicher Operation | Testkit |
| `NeighborDigest` | Hash aller Nicht-Ziel-Paare über alle Dateien und Versionen | Testkit |
| `LeakyPurger` | absichtlich fehlerhafter Purger | Selbsttest (T-RAW-2) |

**Positivkontrolle.** Vor jedem Messlauf wird ein Canary im Testmandanten geschrieben und gefunden. Nicht gefunden → Lauf ungültig, kein Befund. Der Canary liegt nie im produktiven Mandanten (C-L20 [K]).

### 9.3 Klassen und `classify`

| Klasse | Bedingung | Aussage im Bericht |
|---|---|---|
| `GateOnly` | Sperre persistiert, Purge nicht abgeschlossen oder nicht gemessen | „Für Lesezugriffe gesperrt. Physische Entfernung nicht abgeschlossen.“ |
| `PhysicalMeasured` | alle Pflichtschichten mit gültigem Zeugen ohne Befund | „Im Datenverzeichnis und in den gemessenen Schichten nicht nachweisbar. Restrisiken laut Liste.“ |
| `Partial(Vec<LayerId>)` | Pflichtschicht ohne gültigen Zeugen | Liste der nicht gemessenen Schichten |

`classify` ist eine reine Funktion `Evidence → Class`. **Monoton:** fehlt ein Zeuge, sinkt die Klasse, sie steigt nie. Es gibt keine Klasse „kryptografisch gelöscht“ oder „garantiert gelöscht“. Die Verbotsliste (13.2) wird über `Display` der Klassen erzwungen.

### 9.4 Schichten und Restrisiken

**Schichten (`LayerId`, exhaustiver Enum):**

| LayerId | Inhalt | Mechanismus | Messung |
|---|---|---|---|
| `WalSealed` | versiegelte WAL-Dateien | Drain, Entfernen | Dateiliste, Rohscan |
| `WalActive` | aktive WAL-Datei | Rotate, Drain | Dateiliste, Rohscan |
| `Memtable` | Memtable (alle Versionen) | Flush, Schlüsselentfernung | Heap-Scan, Rohscan nach Flush |
| `Sstable` | SSTable-Daten und -Index | Rewrite | Rohscan |
| `Manifest` | Manifest-Logs | Austausch | Rohscan |
| `TextPostings` | `Text`, `Df` | Purge über Matcher | Rohscan |
| `Vector` | Arena, Snapshots | Zeroing, Snapshot-Rewrite | Rohscan, HeapScan |
| `SubjectIndex` | `Subject` | Purge | Rohscan |
| `BlockCache` | entschlüsselte Blöcke | Eviction, zeroize | Heap-Scan |
| `Journal` | Journal | Reduktion nach `Completed` | Rohscan (nach Abschluss ohne Zielbezug) |
| `Snapshots` | MVCC-Versionen, Snapshot-Register | Widerruf, Purge | Versionsscan je Sequenz |

**Restrisiken (Pflichtkatalog, jeder Bericht listet alle zutreffenden):**

| Risiko | Gilt wenn |
|---|---|
| `FlashRemanence` | Speichermedium Flash/SSD |
| `FsJournalOrCow` | Dateisystem mit Journal oder Copy-on-Write |
| `ProcessMemory` | Heap, Swap, Coredump möglich |
| `PageCacheTransient` | Seitencache des Betriebssystems bis zur Eviction |
| `ExternalCopies` | immer (Backups, Snapshots, Kopien beim Betreiber) |
| `ExternalProcessors` | Embeddings oder LLMs außerhalb von Contextra |
| `PseudonymCommitment` | solange `K_commit`-Epoche existiert |
| `ReplicaWindow` | Zeitfenster zwischen Gate und Abschluss |
| `UnmeasuredLayer(id)` | Schicht ohne gültigen Zeugen |
| `GraphRebuildPending` | Vektor-Tombstones bis Batch-Rebuild |

Ein Bericht ohne Restrisikoliste ist ungültig (T-REPORT-1).

### 9.5 Verifikation (`verify`)

`verify(ctx, target)` läuft **in einem neuen Prozess**, ohne Zugriff auf Inhalte:

1. Ziel-Identität wird vom Aufrufer übergeben (`doc_key` bzw. `subject_id`). Daraus werden `doc_id`, Blind-Schlüssel und Commitment abgeleitet.
2. `ErasedSet`-Prüfung: Commitment vorhanden.
3. Rohscan nach Blind-Schlüsseln und `doc_key`-Needles.
4. Ergebnis als `VerificationReport` (gleiche Klassen-Logik).

**Grenze (ausdrücklich):** Inhalts-Needles sind nur beim Erase verfügbar. `verify` prüft Schlüssel-Reste und Identitätsreste, nicht Inhaltsreste. Diese Grenze steht im Bericht.

### 9.6 Verifier (extern)

`examples/verify_erasure_report`: prüft Signatur, Feldschema (unbekannte Felder abgelehnt), Klassenableitung aus den Zeugen. Ohne Zugriff auf Contextra-Interna. Testvektoren im Repo. T-PROOF-1 ändert jedes Byte und verlangt Ablehnung.

---

## 10. Sicherheitsbasis

### 10.1 Schlüsselhierarchie

```
Passphrase ──Argon2id(m≥64MiB, t≥3, p≥1, salt[32])──▶ K_master (32 B)
K_master ──HKDF-SHA256(label)──▶ K_enc      (AEAD für Blöcke, Werte, Journal)
                              ▶ K_idx       (BLAKE3-keyed für doc_id, Tags)
                              ▶ K_wal       (HMAC-Kette WAL)
                              ▶ K_commit[e] (Commitments, je Epoche)
                              ▶ K_sign      (Ed25519-Seed, Berichte)
```

- Labels sind Konstanten (Enum). Keine Ableitung aus Freitext.
- Salt und Argon2-Parameter stehen im Header (Format-Version, KDF-ID). Parameter unterhalb der Untergrenze → `Config`-Fehler beim Öffnen.
- `K_master` lebt in `Zeroizing<[u8;32]>`, nicht kopierbar, nicht `Debug`.

### 10.2 AEAD

- AES-256-GCM-SIV (`aes-gcm-siv`, im Lock [V]).
- Nonce: 96 Bit, OsRng, je Datensatz. Limit je Schlüssel ≤ 2³² Datensätze, durchgesetzt über Zähler im Header. Überschreitung → Fehler (Rotation in v1.1).
- AAD: `family ‖ tenant_tag ‖ coll_tag ‖ format_version`. Vertauschung zwischen Familien oder Mandanten schlägt fehl (T-ENC-2).

### 10.3 WAL

- HMAC-Kette über jeden Frame, Sequenzprüfung, CRC zur Erkennung von Torn-Writes.
- Keine Legacy-Formate. `WalVersion::V1` ist Fehler `Integrity::LegacyFormat` (D-05, R-SEC-6).
- Am Dateiende: CRC-Mismatch ist Torn-Write → Abschneiden bis zum letzten gültigen Frame, protokollieren. Mismatch mitten in der Datei → `Integrity`-Fehler, kein Abschneiden (P2).
- Eine einzige Replay-Implementierung (Abschnitt 11.5, behebt N6).

### 10.4 Geheimnisse

- Passphrase: Typ `SecretBytes` (`Zeroizing<Vec<u8>>`), kein `Clone`, kein `Debug`-Ausdruck, kein `Serialize`. Keine `String`-Kopie.
- Schlüssel im Speicher: `Zeroizing`. Heap-Scan-Test T-SEC-2 prüft Entfernung nach Drop.
- `mlock` für Schlüsselpuffer best-effort. Fehlschlag wird im Bericht als `ProcessMemory` ausgewiesen, nicht verschwiegen.

### 10.5 Dateirechte und Start

- Datenverzeichnis: 0700. Schlüsseldateien: 0600. Start verweigert bei abweichenden Rechten (außer `root`-Test-Ausnahme unter `cfg(test)`; Befund K-Prüfung: Tests als Nicht-root).
- Dateisystem-Probe beim Start (8.7).

### 10.6 Lieferkette

- `cargo deny check` und `cargo audit` sind **blockierend**.
- Jede Advisory-Ignorierung benötigt Ablaufdatum und ADR. Die sechs heutigen Ignorierungen [V] werden in WP-6.5 geprüft.
- `Cargo.lock` mit `--locked` in CI.
- Pinning des Toolchains auf `1.89.0`.

---

## 11. Robustheit und Fehlerverhalten

### 11.1 Grundsatz

Jede Fehlerstelle hat genau eine von drei Reaktionen:
1. **Propagieren** als typisierter Fehler.
2. **Persistieren** als `Pending` mit Wiederholung (nur Löschpfad).
3. **Verweigern** (fail-closed) mit Fehlerzustand.

Verboten: `let _ =` auf `Result`, `unwrap_or_default` auf I/O, `.ok()` ohne Begründung, `unwrap`, `expect`, `panic!`, `unreachable!` im Produktionscode.

### 11.2 Lint-Durchsetzung

Workspace-Lints (blockierend in CI):

```toml
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
indexing_slicing = "deny"
todo = "deny"
unimplemented = "deny"
let_underscore_must_use = "deny"   # neu: verbietet let _ = auf #[must_use]
undocumented_unsafe_blocks = "deny"

[workspace.lints.rust]
unsafe_code = "forbid"            # Ausnahme: simd, sys (eigene Lint-Tabelle)
unexpected_cfgs = "deny"          # ohne #![allow] in irgendeinem Crate
```

`clippy.toml`: `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`, `allow-indexing-slicing-in-tests` = `true`. Außerhalb von `#[cfg(test)]` gilt das Verbot ohne Ausnahme. Ein `#[allow(clippy::…)]` im Produktionscode ist ein CI-Fehler, außer er ist in `governance/allow-ledger.toml` mit ADR und Ablaufdatum geführt.

### 11.3 Lebenszyklus

- `close(self)` ist der definierte Pfad: Hintergrundtasks beenden, Journal schließen, Verzeichnis-Lock lösen, Ergebnis zurückgeben.
- `Drop` ist best-effort und **blockiert nie**: setzt Stop-Flag, `cancel_token.cancel()`, gibt Locks frei, meldet Fehler per Tracing (behebt D-C08). Kein `block_on` in `Drop`.
- Test: Öffnen, Schreiben, Drop, Neuöffnen im selben Prozess ohne Race (Regressionstest zu D-C08).

### 11.4 Konsistenz bei Schreibvorgängen

- Ein Insert legt `Doc`, `DocMap`, `Manifest`, `Text`, `Df`, `Subject`, `Vec` in **einer** Transaktion an.
- Ein Dokument ohne vollständiges Manifest existiert nicht für Löschung. Beim Start werden Teilinserts (kein Commit) per WAL-Replay verworfen, nicht repariert.
- Commit-Reihenfolge: WAL → Memtable → Sichtbarkeit (wie heute, Sequenzsemantik nach WP-2.2).

### 11.5 WAL-Replay

Eine Implementierung. Die heutige Doppelung (`wal/io.rs`, `wal/replay.rs`) wird auf eine reduziert. Paritätstest T-WAL-2 prüft Äquivalenz gegen die Referenz, bis die Doppelung entfernt ist, danach entfällt der Test.

Ursache des CRC-Fehlers bei Offset 4 im verschlüsselten Fall ist **unbekannt** [K, D-C03/D-C04]. Zeitbox in WP-2.4, danach Entscheidung nach Gate G0 (3.2).

### 11.6 Ressourcengrenzen

| Grenze | Wert (Standard, konfigurierbar) |
|---|---|
| Maximale Dokumentgröße | 4 MiB |
| Maximale Chunk-Länge Text | 64 KiB |
| Vektordimension | 16–4096 (fest pro Collection) |
| Maximale Batchgröße Insert | 1.000 Dokumente |
| Maximale Löschbatchgröße | 10.000 Dokumente pro Lauf |
| Speicherbudget Rewrite | Konstante × Blockgröße (Gate 12.4) |
| `erase_return_timeout` | 60 s |

Überschreitung → `Config`- oder `Conflict`-Fehler, nie stilles Kürzen.

### 11.7 Beobachtbarkeit

- Metriken als Zähler und Histogramme ohne Labels mit IDs, Nutzerdaten oder Pfaden.
- Tracing auf Level `info` ohne Inhalte. `debug` darf keine Schlüssel oder Inhalte enthalten (Lint T-LOG-1 über `tracing`-Makros).
- Audit-Kette (`crypto/audit_chain.rs`) für Zustandsänderungen der Löschung, nur Commitments und Zeiten.

---

## 12. Qualitätssicherung

### 12.1 Blockierende CI-Jobs

Alle Jobs mit `continue-on-error` werden in WP-0.2 geprüft. Für v1 blockierend:

| Job | Prüfung |
|---|---|
| `fmt` | `cargo fmt --all -- --check` |
| `check` | `cargo check --workspace --all-targets --locked` |
| `clippy` | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| `test` | `cargo test --workspace --locked` (sequenziell für Crash-Tests) |
| `feature-check` | `cargo hack check --each-feature` plus Test auf undeklarierte Features (ohne `allow`) |
| `deny` | `cargo deny check` |
| `audit` | `cargo audit` |
| `arch` | `tests/arch` (T-ARCH-1..3, T-LINT-1..2) |
| `wording` | Wortlaut-Lint (13.2) über Quelle, Doku, Berichtsvorlagen |
| `policy` | Zählung Panics, `let _`, `allow`, `std::fs`, `HashMap` im Produktionscode auf Obergrenze gemäß `governance/ratchet.toml` (Zähler nur sinkend) |

Nicht blockierend (Tiefenprüfungen): `miri`, `asan`, `coverage`, Fuzz-Dauerlauf, Benchmarks (eigener Lauf mit Basislinie).

### 12.2 Pflicht-Testmatrix

| ID | Prüft | Typ |
|---|---|---|
| T-ISO-1 | Löschung von A lässt `NeighborDigest` bitgleich (andere Collection, anderer Mandant) | Property, ≥ 10⁴ Seeds |
| T-ISO-2 | Nutzerschlüssel als Präfix voneinander (`alice`/`alice2`, `1`/`10`) | gezielt |
| T-ISO-3 | erzwungene `doc_id`-Kollision: Insert abgelehnt, Löschung trifft nur das Ziel | gezielt mit Test-Hook |
| T-VER-1 | Nach Erase liefern alle Lesepfade `Erased` oder `SnapshotRevoked`: `get`, `scan`, `get_at_seq` über **alle** Sequenzen, `scan_at_seq`, Suche | exhaustiv |
| T-RAW-1 | Rohscan nach jedem Saga-Zustand, Abschluss ohne Befund | Rohscan |
| T-RAW-2 | `LeakyPurger` wird gefunden, sauberes System besteht | Selbsttest |
| T-RAW-3 | HeapScan nach Vektor-, Text- und Cache-Löschung | Rohscan RAM |
| T-CRASH-1 | Absturz nach **jeder** Vfs-Operation der Saga; nach `recover()` terminal oder `Pending`, nie Lesezugriff auf `Requested`/`Gated`-Scope außer `Erased` | exhaustiv, enumeriert |
| T-CRASH-2 | abgerissener Write an jedem Byte von Journal, Manifest, `erased.set` | exhaustiv |
| T-CRASH-3 | `ENOSPC` und `EIO` bei `fsync` im Rewrite → `Pending`, Nachbarn unverändert | gezielt |
| T-CRASH-4 | teilindexiertes Dokument nach Absturz verworfen | gezielt |
| T-CANCEL-1 | Abbruch des Aufruferfutures an jedem `await` (Follower-Pfad D-C01) | exhaustiv |
| T-CONC-1 | Erase parallel zu Put desselben Dokuments; mit offenem Snapshot; Barrier gegen Compaction | loom, Stress |
| T-DIFF-1 | Zufallsfolgen (Insert, Update, Flush, Compaction, Restart, Erase) gegen `ReferenceModel` | Differential, ≥ 10⁶ Operationen |
| T-WAL-1 | Abschneiden an jedem Byte-Offset, verschlüsselt und plain: Open gelingt, Präfix wiederhergestellt | exhaustiv |
| T-WAL-2 | Parität der Replay-Pfade bis Entfernen der Doppelung | Differential |
| T-SEQ-1 | Sequenzsemantik (Sichtbarkeit über `TxEnd`) gegen Referenzmodell | Property |
| T-COV-1 | jeder Schlüssel jeder öffentlichen Operation ist im Katalog der Schichten und im Manifest | RecordingStorage |
| T-ENC-1 | neue SSTable mit regulärem Reader lesbar, gleiche Verschlüsselung | Regression |
| T-ENC-2 | Wert unter Familie/Mandant A mit Schlüsselkontext B entschlüsselt nicht | Sicherheit |
| T-KDF-1 | Parameter unterhalb Untergrenze → `Config`-Fehler | gezielt |
| T-DEF-1 | Standard-Builder erzeugt keine Klartextdatei (Rohscan), Öffnen ohne Passphrase schlägt fehl | gezielt |
| T-SEC-2 | Passphrase und `K_master` nach Drop nicht im Heap (HeapScan) | gezielt |
| T-SLA-1 | `erase_sla` fehlt → Builder-Fehler; Überschreitung → Alarm, kein Abbruch | gezielt |
| T-API-1 | Ohne `TenantContext` kein `Collection` | Compile-Fail-Test (trybuild) |
| T-API-2 | `RetentionAttestation` ist Pflicht; `ObligationDeclinesErasure` verhindert Erase | gezielt |
| T-REPORT-1 | Bericht enthält Restrisikoliste, keinen Klartext (Scan des Berichts) | Property |
| T-PROOF-1 | Änderung **jedes** Bytes des Berichts wird abgelehnt; Testvektoren für externen Verifier | Mutation |
| T-LINT-1 | Wortlaut-Lint: keine verbotenen Begriffe (13.2) | Lint |
| T-LINT-2 | kein `std::fs`, `tokio::fs`, `unwrap`, `expect`, `let _ =` in Erase-Modulen | Lint |
| T-LOG-1 | Log-Ausgaben enthalten keine Needles (Canary-Scan über Tracing-Output) | Property |
| T-ARCH-1 | `types` ohne `async` und ohne I/O | Architektur |
| T-ARCH-2 | nur `TenantScopedStorage` ruft den rohen Store auf | Architektur |
| T-ARCH-3 | Ring-Abhängigkeiten nur nach unten | Architektur |
| T-DROP-1 | Drop ohne `close()` blockiert nicht, Neuöffnen im selben Prozess gelingt | gezielt |
| T-SIMD-1 | jeder SIMD-Kernel gleich Skalar (Zufallsvektoren, alle CPU-Dispatch-Pfade) | Property |
| T-DOCMAP-1 | Zuordnung `doc_id ↔ slot` konsistent nach Erase und Rebuild | Property |

### 12.3 Werkzeuge

| Werkzeug | Ziel |
|---|---|
| `loom` | Barrier, ReadGate, Journal, Gate-Set unter Nebenläufigkeit |
| `cargo-fuzz` | Journal-Parser, SSTable-Index, WAL-Frame, Schlüsselcodec, Report-Parser |
| `cargo-mutants` | Saga-Übergänge, `classify`, Matcher, Barrier. Gate ≥ 95 % der Mutanten getötet. |
| `miri` | Simd-Kernel-Inseln (nicht blockierend, CPU-Pfad) |
| `proptest` | T-ISO, T-DIFF, T-SEQ, T-SIMD |

### 12.4 Leistungsziele

Basislinie in WP-6.1 auf dokumentierter Hardware (CPU-Modell, RAM, Speicher, Kernel, Dateisystem, Mount-Optionen). Gates relativ zur Basislinie.

| Messgröße | Methode | Gate |
|---|---|---|
| Erase-Latenz bis `Gated` (p50/p99) | 10⁴, 10⁶ Dokumente | p50 ≤ 10 ms, p99 ≤ 50 ms |
| Erase bis `Completed` (Dokument) | 10⁴, 10⁶, 10⁷ Dokumente | Anstieg proportional zu betroffenen SSTables, nicht zur Gesamtmenge |
| Batch-Erase 1.000 Dokumente | Durchsatz, Anzahl Rewrites | ausgewiesen, Basislinie + 10 % |
| Schreibverstärkung Purge | geschriebene Bytes je gelöschtem Byte | ≤ Basislinie + 5 %, Zielwert je nach Messung festgelegt |
| RSS-Spitze beim Rewrite | während Rewrite | ≤ Konstante × Blockgröße |
| Lese-Overhead `ErasedSet`-Prüfung | `get`, `scan` mit/ohne | ≤ 3 % p50 |
| Insert/Get/Scan/Suche | `lsm_concurrency_bench`, `wal_bench`, `sstable_bench`, `redb_comparative_bench` | Regression ≤ 5 % gegen Basislinie |
| Suche Hybrid (BM25 + HNSW + RRF) | festes Testkorpus | Recall@10 und p95-Latenz, Basislinie |

Benchmarks laufen mit fixen Parametern, mehrfach, mit Standardabweichung. Ein Lauf ohne Basislinie zählt nicht.

---

## 13. Compliance-Leitplanken

### 13.1 Anforderungsmatrix

Die Anforderungen sind Arbeitsgrundlage für G-LEGAL. Artikelangaben sind Orientierung.

| # | Anforderung (Orientierung) | Mechanismus | Test | Restrisiko |
|---|---|---|---|---|
| R1 | Unverzügliche Löschung, Fristen (Art. 17, 12 Abs. 3) | Saga mit Wiederholung, `erase_sla`, Status | T-CRASH-*, T-SLA-1 | Zeitfenster `ReplicaWindow` |
| R2 | Unkenntlichmachen (SDM-Baustein 60) | Physische Entfernung aller Schichten, Messung | T-RAW-1/3 | `FlashRemanence`, `FsJournalOrCow`, `ProcessMemory` |
| R3 | Einfache Löschbefehle genügen nicht | Rewrite, Entfernen, Fehlen geprüft | T-CRASH-1, T-RAW-1 | wie R2 |
| R4 | Alle Versionen und Zeitreisen | Purge aller Versionen, Snapshot-Widerruf | T-VER-1 | – |
| R5 | Rechenschaft (Art. 5 Abs. 2) | signierter Bericht, Audit-Kette ohne Klartext | T-PROOF-1, T-REPORT-1 | `PseudonymCommitment` |
| R6 | Datenschutz durch Technikgestaltung (Art. 25) | Verschlüsselung ohne Opt-out, Mandantenpflicht, keine Extraktion im Default | T-DEF-1, T-API-1 | – |
| R7 | Sicherheit der Verarbeitung (Art. 32) | Argon2id, WAL-HMAC, AEAD mit AAD | T-KDF-1, T-WAL-1, T-ENC-2 | Schlüsselkompromittierung |
| R8 | Information von Empfängern (Art. 17 Abs. 2, Art. 19) | Bericht führt `ExternalProcessors`, `ExternalCopies`; Hook in v1.1 | – | Verantwortung Betreiber |
| R9 | Aufbewahrungspflichten (Art. 17 Abs. 3) | `RetentionAttestation`, kein Scheduler | T-API-2 | Prüfung beim Betreiber |
| R10 | Embeddings vorsorglich personenbezogen [D] | Vektoren im Löschumfang, Snapshots verschlüsselt | T-RAW-3 | Rekonstruktion aus Resten im RAM |
| R11 | Keine „Einschränkung statt Löschung“ in v1 | entfällt | – | – |
| R12 | Werbeaussagen belegpflichtig (§ 5 UWG) [D] | Wortlaut-Lint, Aussagen nur aus Berichtsklasse | T-LINT-1 | – |
| R13 | Backups | außerhalb; im Bericht ausgewiesen | – | Betreiber der Sicherung |

### 13.2 Außenaussagen

**Zulässig:**

> Contextra entfernt die Daten einer betroffenen Person aus dem Datenverzeichnis, den Indexen und den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter.

**Unzulässig (Wortlaut-Lint T-LINT-1, Quelle, Doku, Berichtsvorlagen, Marketing):**

„garantiert“, „DSGVO-konform“, „rechtskonform“, „unwiederbringlich“, „informationstheoretisch gelöscht“, „kryptografisch gelöscht“, „sicher gelöscht“ ohne Berichtsbezug, „vollständig gelöscht“ ohne Klassenangabe.

### 13.3 Löschcheckliste (Textexport)

`erasure_checklist()` liefert einen Text mit Prüfpunkten, die der Betreiber außerhalb von Contextra abarbeitet. Inhalt (Auszug):

- Backups und Snapshots des Datenverzeichnisses: Aufbewahrungsdauer prüfen, Restore-Hinweis beachten (Restore kann gelöschte Daten zurückbringen).
- Anwendungs-, Monitoring- und Tracing-Logs, die IDs oder Inhalte enthalten.
- Eingesetzte Embedding- und Sprachmodelldienste: welche Eingaben dort gespeichert werden.
- Exporte, Berichte, Tickets und Kopien beim Betreiber.
- Swap- und Core-Dump-Konfiguration des Hosts.

Die Checkliste enthält den Hinweis, dass sie **keine Rechtsberatung** ist und vom Betreiber geprüft werden muss. Freigabe durch G-LEGAL (F-CHK).

### 13.4 Rechtsfragen (Gate G-LEGAL)

Ohne schriftliche Antworten darf das Produkt keine Löschaussage außer der Zulässigen (13.2) treffen.

1. Gilt physische Umschreibung mit Entfernen der alten Dateien als Löschung nach Art. 17, auch bei Flash-Medien mit Restrisiko?
2. Sind Embeddings personenbezogene Daten? (Vorsorge: ja [D].)
3. Gelten Slot-Nummern im HNSW und Tags als personenbezogen? Ist ein Graph-Rest bis zum Rebuild zulässig?
4. Dürfen Bericht und reduziertes Journal personenbezogene Bezüge (Commitment, Zeit) enthalten, und wie lange?
5. Sind aggregierte `Df`-Zähler personenbezogen?
6. Rolle von Contextra bei lokal installierter Software: Hersteller, Auftragsverarbeiter, Software ohne Zugriff?
7. Aufbewahrungspflichten je Branche und ihre Wirkung auf die Löschung.
8. Freigabe der Löschcheckliste (13.3).

---

## 14. Grenzen dieser Spezifikation

- **Keine Build- und Testläufe.** Kompilierbarkeit und Verhalten sind [U] oder Erwartung.
- **Nicht gelesen am Clone:** Textindex-Postingformat im Detail, Bloom- und Manifestinhalt (ob Schlüsselbereiche gespeichert werden), Blockcache (ob er entschlüsselte Blöcke hält), HNSW-Snapshot-Format, `kvcache`-Verwendung außerhalb der Entfernungsliste, `contextra-db`-Inhalt (WP-1.1).
- **Nicht nachgezählt:** Testzahlen aus Quelldokumenten. Zeilenzahlen sind `grep`- und `find`-Zählungen mit Obergrenzen.
- **Restrisiken** in 9.4 sind Schlussfolgerungen aus Eigenschaften von Flash und Dateisystemen, keine Messungen.
- **Kollisionswahrscheinlichkeit** der 64-Bit-DocId ist eine Rechnung, keine Messung.
- **Mit dem Clone-HEAD nicht abgeglichen:** die Behauptungen über die Schichtzugehörigkeit von `kvcache` und `graph` im Rückbau-Dokument.

---

## 15. Offene Entscheidungen für den Auftraggeber

| ID | Frage | Empfehlung | Wirkung bei Ablehnung |
|---|---|---|---|
| F-LEG | Existiert Altbestand (ausgelieferte Daten, V1–V3-Beweise, Legacy-WAL)? | Nein annehmen; falls ja: dokumentiertes Offline-Migrationswerkzeug mit Frist, dann Entfernen | Migrationspfad als v1-Scope |
| F-BLOB | Speichert Contextra Originaldokumente? | Nein. Nur Chunks und Metadaten. | Gleiche Löschpflicht, zusätzliche Schicht, Phase 4 |
| F-COMMIT | Lebensdauer `K_commit`-Epoche und `erased.set` | Epoche 365 Tage, danach Rotation; Commitments bleiben im Bericht als Pseudonym | Länger = mehr Pseudonymrest |
| F-LIC | Lizenz- und Vertriebsmodell | Außerhalb des Speicherkerns (D-23) | Lizenzlogik zurück in v1 |
| F-PY | Python-Bindings in v1 oder v1.1 | v1.1 (D-25) | Mehrscope, Verzögerung |
| F-MCP | MCP-Server in v1 oder v1.1 | v1.1 (D-25) | Mehrscope, Löschwerkzeuge erst nach Saga |
| F-GRAPH | Graph-Rückkehr | v1.1, nur mit Ebene-A-Adapter (D-17) | – |
| F-RETENTION | Einschränkung (Art. 18) als Zustand | v1.1 (D-15) | – |
| F-KEYSHRED | Schlüsselvernichtung als Schicht | v1.1 nach G-LEGAL (X-1) | – |
| F-AUDITUSER | Benutzerfreundlichkeit `ObligationDeclinesErasure` (Code-Liste) | Code-Liste gemeinsam mit Fachanwalt festlegen | – |
| F-SLA | Wert von `erase_sla` für Standard-Dokumentation | Vorschlag 24 h bis `Completed`, Alarm nach 1 h Pending | – |
| F-AUTO | Automatische Extraktion im Aufrufer oder außerhalb | Außerhalb | – |

---

## Anhang A: Rückbauliste (Kurzform)

| Pfad | Aktion |
|---|---|
| `crates/contextra-crypto/src/deletion_proof/` | entfernen |
| `crates/contextra-crypto/src/ed25519_proof.rs` | ersetzen durch `proof_v1.rs` (Report) |
| `crates/contextra-crypto/src/{kv_shredding,kv_cipher,revocation_log}.rs`, `kv_segment/` | entfernen |
| `crates/contextra-crypto/src/audit_chain.rs` | behalten |
| `crates/contextra-store/src/scanner/` | entfernen (`scanner/purge.rs`, `scanner/wal.rs` mit Canary im produktiven WAL [K: C-L20]) |
| `crates/contextra-store/src/kv/` | entfernen |
| `crates/contextra-store/src/wal/replay.rs` (7 `wal-integrity`-Gates) | Gates entfernen, Replay vereinheitlichen |
| `crates/contextra-store/src/lib.rs:45` | `#![allow(unexpected_cfgs)]` entfernen |
| `crates/contextra-store/src/lsm/ops/compaction.rs:225–231` | Sentinel-Fix (WP-2.1) |
| `crates/contextra-mvcc/src/ssi.rs:646–652` | Sentinel-Fix, Assertion |
| `crates/contextra-engine/src/contextra_impl/erasure.rs` | entfernen |
| `crates/contextra-engine/src/collection/crud/delete.rs` | umbauen (Lock, Saga-Anbindung) |
| `crates/contextra-vector/src/hnsw/{purge,deletion}.rs` | entfernen, neu als Zeroing (D-18) |
| `crates/contextra-text/src/inverted/{purge,scanner}.rs` | entfernen, neu über Matcher (D-20) |
| `crates/contextra-ports/src/scanner/`, `crates/contextra-types/src/witness.rs` | entfernen, neu in `types/src/erasure` |
| `crates/contextra-mcp/src/{proof_key,proof_key_env}.rs` | entfernen |
| `crates/contextra-simd/src/kernels/avx512.rs` (u8-Pfad) | entfernen (D-28) |
| `crates/contextra-ports/src/vfs.rs` und `crates/contextra-durable-fs/src/vfs.rs` | zusammenführen (D-04) |
| `docs/DELETION-DSGVO.md` | Status „Konzept“, durch diese SPEC ersetzt |
| `docs/SPEC.md` (2.799 Zeilen) | durch diese SPEC ersetzen, alte Fassung nach `archive/` |
| `PLAN.md` (1.397 Zeilen) | durch `PLAN.md` dieser Lieferung ersetzen |
| `docs/decisions/` | anlegen, ADR-001 aufnehmen |
| Workspace `members` / `default-members` | auf Kern laut 5.1 reduzieren |
| `capabilities.toml` | an Ringe 5.2 anpassen |
| `governance/*` | Ratchet-Dateien auf neue Zähler umstellen |

---

## Anhang B: Quellenregister

| Dokument | Basis | Verwendung in dieser SPEC |
|---|---|---|
| Fehlerkatalog Stand 2026-11-01 | `f653f0f73` | Befundliste [K], Priorisierung, D-C01/D-C02/B-S-Befunde |
| Löschpfad: Rückbau und Neuaufbau | `f653f0f73` | Entfernungsumfang, Anforderungen N-01..N-29 (übernommen in 8, 9, 10, 13) |
| Implementierungsspezifikation Erasure Core auf eigenem LSM-Tree | `9da0eca` | Befunde N1–N10 (neu, geprüft), Purge-Algorithmus (übernommen in 8.4) |
| Spezifikation v1 Final | `9da0eca1b` | Produktumfang, Phasen (Vorlage für PLAN.md), Nachweisformat |
| Spezifikation v1 Erasure | `9da0eca1b` | ersetzt durch Final, nur Vergleich |
| Implementierungsspezifikation SOLL (Löschkette, Speicherkern, Verifikation) | `9da0eca1b` | Haftungsrahmen, Anforderungsmatrix R1–R13, Journal-Dateien, Restrisiko-Katalog |
| Spezifikation und Umsetzungsplan | `c4e285c` | Ringe, Graph-Entkopplung (verworfen, D-17), Rückbau-Prüfliste |
| Entscheidungsgrundlage | `c4e285c` | Gates G1–G4 (übernommen als G0), Widerspruchsliste W1–W8 |
| LSM-Audit-Report (vollständig und Kurzfassung) | `9da0eca1b` + lokal | Testbefunde #1–#10 (übernommen in PLAN WP-2.x), Gemini-Teil verworfen (N12 Quelle) |
| `AGENTS.md`, `VETOES.md`, `docs/DELETION-DSGVO.md` | Clone | Normative Regeln, ADR-001 |
| Build-/Auditlog des Auftraggebers | Nutzerlauf | [U]-Befunde |
