# contextra-db — Implementierungsspezifikation zum SOLL-Zustand

Stand: 17. Nov 2026 · Autor: Principal Rust Engineer (Claude) · Bezug: `docs/SPEC.md` v6 (16.11.2026, „Personendaten, Löschkette, Zeitreise") · Gegenstand: Crate `crates/contextra-db`

---

## 0 Status, Grundlage, Grenzen

### 0.1 Grundlage

| Punkt | Inhalt |
| --- | --- |
| Geprüfter Stand | Repository `tfufuz1/contextra`, Klon vom 17.11.2026, HEAD `24487fa` („Shell-Commit", flacher Klon, Tiefe 1). Die Spezifikation und die Berichte nennen andere Hashes (`fcd387021`, `5137db4ab`, `f0eda1cb9`). Alle Zahlen hier gelten für `24487fa`. |
| Methode | Lesen von `docs/SPEC.md` (vollständig), `docs/01_schluesselmodell.md`, `docs/02_loeschung.md`, `docs/Umsetzbarkeit_Loeschung_Contextra.md` (gezielt), `reports/*.md`, der crate-lokalen Dokumente und des Quelltexts von `contextra-db` (Module, Schnittstellen, Pfade); `grep`, Dateilisten und ein kleines Zählskript. |
| **Grenze** | **Nichts wurde gebaut, getestet oder ausgeführt.** Im Klon fehlt das Workspace-Wurzelmanifest `Cargo.toml` (ebenso `justfile`, `governance/`, `.github/`, `docs/decisions/`, `capabilities.toml`), und die Toolchain 1.89.0 ist hier nicht ladbar (`static.rust-lang.org` nicht erreichbar). Jede Zahl ist eine statische Näherung. |
| Belegmarken | **[V]** am Klon per Datei, `grep` oder Zählskript geprüft · **[V\*]** eigenes Skript, grob · **[R]** aus `reports/` oder `docs/` übernommen, nicht neu geprüft · **[S]** aus `docs/SPEC.md` übernommen · **[E]** Einschätzung · **[D]** Festlegung oder Vorschlag dieser Spezifikation |
| Rangfolge | `docs/SPEC.md` v6 und das Entscheidungsregister bleiben normativ (§1.2 der SPEC). Dieses Dokument ist die crate-lokale Umsetzungsspezifikation und ersetzt `crates/contextra-db/SPEC.md` und `PLAN.md` (beide veraltet, siehe DB-F03). Widersprüche zur SPEC sind als Entscheidungsbedarf (§5) markiert, nicht still aufgelöst. |
| Normative Sprache | MUSS, DARF NICHT, SOLL nach RFC 2119. Dieses Dokument enthält keine Compliance-Aussagen (INV-7): „gelöscht" steht nur zusammen mit einer gemessenen Stufe. |

### 0.2 Aufbau

§1 Zusammenfassung · §2 Was die SPEC für `db` verlangt · §3 Ist-Zustand · §4 Lückenanalyse · §5 Entscheidungen · §6 Zielentwurf · §7 Umsetzungsplan · §8 Tests und Gates · §9 Risiken · §10 Offene Punkte · Anhänge A bis D.

---

## 1 Zusammenfassung

`contextra-db` ist am Klon **nicht** das Crate, das die SPEC beschreibt. Die SPEC sieht `db` als schlanke Integrationsschicht über einem getrennten `engine`-Crate (Kern: `engine`, `types`, `ports`, `vault`). Tatsächlich ist `engine` gemäß ADR-0122 vollständig in `db` aufgegangen; `contextra-engine` ist eine veraltete Fassade mit einer Zeile Code (`pub use contextra_db::*`) \[V\]. `db` ist damit der Motor: 94 Dateien, 23.313 Zeilen (≈ 13.500 ohne Testcode), 336 Testmarker, 10 direkte interne Kernabhängigkeiten \[V\].

Die Arbeit gliedert sich in vier Blöcke:

1. **Ausgangslage herstellen.** Das Crate ist laut Berichten nicht test-baubar (10 Test-Targets und 1 Example mit Fehlern \[R\]). Zwei Test-Dateien importieren ein Modul `contextra_db::memory_consolidation`, das nur in `contextra-cognition` existiert \[V\]. Das Wurzelmanifest fehlt im Klon \[V\]. Ohne grünen Build ist keine Zusage aus §11.5 der SPEC prüfbar.
2. **Strukturbereinigung nach SPEC §9** (S-22, S-60, S-61, S-63, S-64, S-12a, S-70): `vault`-Reste, Regler-Kopien, Chunker-Dublette, Cognition-Inversion, Feature-Namen, tote Abhängigkeiten, aufwärts gerichtete Dev-Kante auf `testkit`, Zyklus `db` ↔ `cognition`.
3. **Neue Fähigkeiten nach SPEC Kap. 11 bis 14**, soweit sie `db` betreffen: Klassentrennung (R-CLS), Lesesperre in allen Lesepfaden (Z-E1, Z-E8), Löschkette als Teilnehmer (L2, L5, Drop), Zeitreise mit zwei unabhängigen Achsen (Kap. 12), Simulation (Kap. 13). Heute enthält `db` **nichts** davon: kein Gate, keine Klasse, keinen `PersonalRef`, keine Schlüsselvernichtung im Löschpfad \[V\].
4. **Abschluss:** Entfernen der Übergangspfade (S-70), Berichte, Entnehmbarkeitsnachweis.

**Die fünf wichtigsten Entscheidungen, die vor Phase 1 fallen müssen (§5):**

| ID | Frage | Empfehlung |
| --- | --- | --- |
| DB-D01 | Wird der `engine`-Merge (ADR-0122) bestätigt und die SPEC angepasst? | Ja. Rückbau wäre teurer und widerspricht dem Repo-Stand. |
| DB-D02 | Darf `db` Features `erasure` und `rules` führen, obwohl §4.1 sie nicht nennt? | Ja, als benannte Features; SPEC §4.1 entsprechend ändern. |
| DB-D04 | Wie wird R-CLS (Klasse je Schreibvorgang) mit INV-2 (Pfadgleichheit) vereinbart? | `DataClassPolicy::{Legacy, Enforced}`; Aussagen zu Z-E5 nur unter `Enforced`. |
| DB-D06 | Wie wird Z-E8 (Rangfolge unabhängig von unzugänglichen Dokumenten) gegenüber BM25-Korpusstatistik erfüllt? | Spike vor Festlegung (DB-S23); bis dahin Zusage auf Trefferebene. |
| DB-D08 | Gilt Z-T1 (byte-identisch, ohne Gleitkomma) auch für ANN-Dokumentensuche? | Nein; Z-T1 gilt für `rules` und die Zeitauswahl, für ANN gilt INV-5. |

**Kritischer Pfad:** DB-S00 → DB-S01 → (DB-D01/D02 entschieden) → DB-S15 → [erasure S-101/S-102/S-103, `ids` S-100] → DB-S22 → DB-S31 → DB-S32. Zeitreise und Simulation hängen zusätzlich an `rules` (S-104, S-105, S-107). Aufwand grob 45 bis 60 Personentage für `db` allein, ohne die Vorarbeiten in `erasure`, `ids`, `privacy`, `rules`, `types` \[E\].

---

## 2 Was die SPEC für `contextra-db` verlangt

### 2.1 Zielzustand (Zusammenstellung)

| Anforderung | Quelle | Inhalt |
| --- | --- | --- |
| Ebene E2 | §2.2, §4.1 | Integration; darf E0, E1, E2 nutzen, nie E3. |
| Kern direkt | §4.1 | `engine, types, ports, vault` (siehe DB-D01 zu `engine`). |
| Features (Ziel) | §4.1 | `cognition`→cognition; `exp-diskann`→vector-diskann (nach S-80). |
| Default | §4.1, R10 | leer. |
| Fehlertyp | §4.1 | `ContextraError` (kein eigener Typ). |
| Vault | S-22, D-14 | `volatile_vault.rs` ist Re-Export; `vault_sys.inc`/`vault_compat.inc` entfallen; `vault` wird Workspace-Mitglied. |
| Regler | S-60 | `decay_controller`, `homeostat`, `pid_latency_controller` als `pub use` aus `adapt` nach Differenztest (bitgleich, `ManualClock`). |
| Chunker | S-61 | `engine::chunker` über `ingest::chunking` anbinden; `contextra::chunker::MarkdownChunker` bleibt API-stabil. |
| Cognition | S-63 | Inversion: `cognition` definiert lokale Ports, `db` liefert die Brücke; `cognition` hängt nicht mehr an `engine` und `graph`. |
| Lizenz | S-64, S-50 | `license`-Seam auf Crate `license`; ohne Feature `license` gilt `OpenFastGate`-Semantik, per Test belegt. |
| Experimente | S-12a | `experimental-*` → `exp-*`; `*-compat` als Alias kennzeichnen; Features ohne `cfg` und ohne Weiterleitung entfernen. |
| Zeitachsen | §12.2, S-105 | `db/src/temporal_filter.rs` auf `rules::ValidityWindow` umstellen; Golden für Intervallgrenzen. |
| Klassen | §11.2, R-CLS, R-HANDLE | Jeder Schreibvorgang deklariert K, P, D oder E; K nur über `Validated<T>`; Verweise auf P nur als `PersonalRef`. |
| Lesesperre | §11.4.2, Z-E1, Z-E8 | Prüfung vor Entschlüsselung und vor jeder Ausgabe, O(1), ohne I/O; gleiches Fehlerbild wie bei nicht vorhandenem Ziel. |
| Löschkette | §11.4 | L2 (Bereinigung: Indexsegmente, Caches, Sitzungen, Chunk-Cache, Temp, Journal), L5 (Messung), Drop-Pfad. |
| Simulation | §13, Z-S1 | Szenarien nur auf `sim/<id>`, kein Schreibzugriff auf `main`, P, Rechte. |
| Allgemeine Invarianten | INV-1 bis INV-16 | u. a. INV-2 (Pfadgleichheit), INV-5 (Determinismus), INV-11 (`forbid(unsafe_code)`), INV-12 (keine Panics in Nicht-Test-Code), INV-13 bis INV-16. |
| Entnehmbarkeit | §8.4 | Gilt für E0/E1; für `db` (E2) wird die Closure nicht budgetiert, aber von G-L4 berechnet. |

### 2.2 Zusagen, die `db` mit trägt (Auswahl)

Z-E1 (Lesepfade), Z-E2 (Verschlüsselung und Metadaten), Z-E5 (K ohne Personendaten), Z-E6 (IDs unverändert), Z-E7 (keine Roh-Kennungen in Ausgaben), Z-E8 (Zugriff vor Suche und Statistik), Z-E9 (fail-closed), Z-T2 (vollständige Historie), Z-T5 (Richtlinie nur im aktuellen Stand), Z-S1 (Simulationsisolation). Die Testzuordnung steht in §8.

---

## 3 Ist-Zustand (am Klon `24487fa`)

### 3.1 Kennzahlen

| Messgröße | Wert | Beleg |
| --- | --- | --- |
| Quelldateien `src/` | 94 | \[V\] |
| Zeilen `src/` gesamt / ohne Testcode | 23.313 / ≈ 13.492 | \[V\*\] Skript, schneidet ab erstem `#[cfg(test)]` und schließt `tests.rs`, `unit_tests.rs`, `*/tests/*` aus |
| Einträge in `tests/` | 53 (davon 1 `golden/`) | \[V\] |
| Testmarker (`#[test]`, `#[tokio::test]`) in `src/` und `tests/` | 336 | \[V\] |
| Größte Dateien | `lib.rs` 1.205 · `collection/maintenance.rs` 972 · `collection/mod.rs` 730 · `contextra_impl/lifecycle.rs` 600 · `collection/search/hybrid/query.rs` 590 · `collection/crud/insert.rs` 565 | \[V\] |
| Features | 17 (inklusive `default = []`) | \[V\] `Cargo.toml` |
| Interne Kernabhängigkeiten | 10 fest (`vault`, `types`, `ports`, `mvcc`, `store`, `vector`, `text`, `checkpoint`, `rank`, `graph`) + 4 optional (`cognition`, `sandbox`, `infer-onnx`, `adapt`) | \[V\] |
| Interne Dev-Abhängigkeiten | 8: `vector`, `testkit`, `store`, `graph`, `text`, `adapt`, `checkpoint`, `ingest` | \[V\] |
| Reverse-Abhängigkeiten | `contextra`, `contextra-agent`, `contextra-bench`, `contextra-engine` (Fassade), `contextra-cognition` (nur dev) | \[V\] |
| `from_key(`-Aufrufe außerhalb von Tests | 35 in 14 Dateien (`collection/relate.rs` 8, `export.rs` 4, `search/hybrid/query.rs` 4, `maintenance.rs` 4, weitere ≤ 2) | \[V\] |
| `unwrap`/`expect`/`panic!`/`todo!` im Nicht-Testteil | 0 Treffer | \[V\*\] gleiches Skript; `unwrap-gate` bleibt maßgeblich |
| `unsafe` | `#![forbid(unsafe_code)]` in `lib.rs` | \[V\] |
| `cfg(not(loom))` | 62 Stellen; `lib.rs` blendet fast alles unter Loom aus | \[V\] |
| `#[allow(deprecated)]` | 40 dokumentiert in `DEPRECATED_DEBT.md`, dessen Pfade (`collection/search.rs`, `query_builder.rs`, `tests.rs`) heute Verzeichnisse sind | \[V\] |
| Golden-Korpus | nur `tests/golden/vault_errors.json` (+ Manifest, `regen.sh`) | \[V\] |
| Crates `rules`, `verify`, `vector-diskann`, `graph-research` | existieren nicht | \[V\] |

### 3.2 Abhängigkeiten: Soll und Ist

```
SOLL (SPEC §4.1)                      IST (Cargo.toml)
db ─► engine ─► {mvcc,store,vector,   db ─► mvcc, store, vector, text, graph,
                 text,graph,rank,            rank, checkpoint, vault, types, ports
                 checkpoint}                 (+opt: cognition, sandbox, infer-onnx, adapt)
   ─► types, ports, vault             engine ─► db            (deprecated Fassade)
   ─► cognition (Feature)             cognition ─(dev)► db    ◄── Zyklus mit db ─(opt)► cognition
                                      db ─(dev)► testkit (E3) ◄── Aufwärtskante
```

### 3.3 Modullandkarte (Ist)

| Bereich | Pfad | Zeilen (Prod. ohne Tests, grob) | Rolle |
| --- | --- | --- | --- |
| Fassade | `src/lib.rs`, `src/contextra_impl/{lifecycle,collections,crud,diagnostics,wiring}.rs` | ≈ 1.400 | `Contextra`, `ContextraConfig`, Re-Exports, `SandboxBridge` |
| Collection | `src/collection/{mod,kv_lock,tx,relate,maintenance}.rs`, `crud/*`, `search/*`, `query_builder/*` | ≈ 5.000 | CRUD, Suche, Hybrid, Links, Wartung, Drop |
| Transaktionen | `src/transaction/*` | ≈ 1.200 | 4-Index-2PC, `CommitIntent`, Kompensation, Recovery |
| Zeitfilter | `src/temporal_filter.rs` | ≈ 190 | Bitemporaler Nachfilter |
| Chunker | `src/chunker/*` | ≈ 700 | Dublette zu `ingest::chunking::prose` |
| Regler | `src/{decay_controller,homeostat,pid_latency_controller}.rs`, `multistep.rs`, `multistep_pid.inc` | ≈ 800 | Kopien zu `adapt` |
| Hintergrund | `src/background_workers/*` | ≈ 500 | Expiry, Orphan, Hyperedge, Decay |
| Extraktion | `src/extraction/*` (Feature `exp-extraction`) | ≈ 400 | Regelbasierte/Open-IE-Extraktion |
| Export/Import | `src/export.rs`, `src/import.rs` | ≈ 520 | JSON-Format v1 |
| Brücken/Nähte | `src/adapters/*`, `src/cognition/*`, `src/license_seam.rs`, `src/backends.rs`, `src/volatile_vault.rs`, `src/*.inc` | ≈ 200 | siehe Befunde |
| Sonstiges | `fusion.rs`, `filter.rs`, `metrics_names.rs`, `merge_operator_adapter.rs`, `orchestration/mod.rs` | ≈ 1.100 | |

### 3.4 Befunde

Schweregrade: **K** kritisch (blockiert Prüfbarkeit oder Zusage), **H** hoch, **M** mittel, **N** niedrig.

| ID | Befund | Beleg | Sev. | Schritt |
| --- | --- | --- | --- | --- |
| DB-F01 | Wurzelmanifest `Cargo.toml`, `justfile`, `governance/`, `.github/`, `docs/decisions/`, `capabilities.toml` fehlen im Klon. Kein Build, kein Gate lokal reproduzierbar. | `ls`, `git ls-files` | K | DB-S00 |
| DB-F02 | `engine` ist in `db` aufgegangen (ADR-0122); die SPEC (§2.2, §4.1, §9.2, §9.4) führt `engine` weiter als eigenes E2-Crate. ADR-0122 liegt nirgends im Repo (nur Verweise in `lib.rs`-Kommentaren und `engine/src/lib.rs`). `db/README.md` behauptet, `db` gehe in `contextra` auf; die SPEC lässt `contextra` von `db` abhängen. | `engine/Cargo.toml` (`[dependencies] contextra-db`), `engine/src/lib.rs`, `db/README.md` | K | DB-D01, DB-S03 |
| DB-F03 | Crate-lokale `SPEC.md`/`PLAN.md`/`AGENTS.md` beziehen sich auf SPEC v2 und HEAD `f0eda1cb9`, nennen „1797 Zeilen in 10 Dateien" und Kanten „engine, types, ports"; viele Modulköpfe sagen „Gehört zur Crate `contextra-engine`". | `SPEC.md` §3/§4, Modulköpfe in `temporal_filter.rs`, `delete.rs`, `export.rs`, `backends.rs` | M | DB-S02 |
| DB-F04 | Tests nicht kompilierbar: `tests/docid_128_consolidation.rs` und `tests/consolidation_double_trigger_test.rs` importieren `contextra_db::memory_consolidation`; das Modul liegt in `contextra-cognition/src/memory_consolidation.rs`. Laut Berichten zusätzlich Typfehler in `tests/fault_injection_2pc.rs` und 10 Test-Targets + 1 Example betroffen. Der Fix-Branch `fix/test-targets` fehlt im Klon. | `grep`, `reports/AUDIT_VERIFIKATION_2026-11-16.md` §2 | K | DB-S01 |
| DB-F05 | Kantenverstöße: `db` führt dev-Abhängigkeit auf `contextra-testkit` (E3), die in `src/`, `tests/` und `examples/` **nicht benutzt** wird (0 Treffer); `contextra-ingest` ebenfalls ungenutzt (0 Treffer). `cognition` führt dev-Abhängigkeit auf `db`, während `db` das optionale Feature `cognition` führt: Zyklus im Kantengraph (R13, D-05). | `Cargo.toml` beider Crates; `grep` | H | DB-S12, DB-S15 |
| DB-F06 | Die Cognition-Brücke ist leer: `src/cognition/bridge.rs` (15 Zeilen) definiert `DbDocumentSource` ohne Trait-Implementierung. `cognition` definiert die Ports `DocumentSource`, `ConsolidationSink`, `HyperedgeSink`, `DecayPolicy`, `CollectionOps` und einen eigenen `CommitIntent`-Ersatztyp (`ports.rs:16-21`) und führt noch vier `*_local.rs`-Dateien sowie die Abhängigkeit auf `graph`. | `bridge.rs`, `contextra-cognition/src/ports.rs`, `Cargo.toml` | H | DB-S15 |
| DB-F07 | Tote Vault-Dateien: `src/vault_sys.inc` (34 Zeilen) und `src/vault_compat.inc` (2 Zeilen) werden nirgends per `include!` eingebunden (einzig `multistep_pid.inc` wird eingebunden). `volatile-vault` ist ein leeres Feature, obwohl `contextra-vault` feste Abhängigkeit ist. `volatile_vault.rs` ist Re-Export. | `grep include!`, `Cargo.toml` | M | DB-S10 |
| DB-F08 | Regler-Kopien weichen von `adapt` ab: `db::decay_controller` hat `with_clock`, `*_ms`, `*_wall_clock`, die `adapt` fehlen; `db::homeostat::RerankDeadline` (`Instant::now()`, `is_expired`, `remaining`) und `RerankPidController::new(target_ms, initial_pool)` unterscheiden sich von `adapt::homeostat::{RerankDeadline<C: Clock>, RerankPidController::new(kp,…)}`; `db::PidLatencyController` (Skalierungsfaktor `f64`) ist semantisch nicht `adapt::PidController` (Poolgröße `usize`). Ein bloßes `pub use` (S-60) bricht die API. `homeostat.rs` nutzt `Instant::now()` (INV-5). | Vergleich der `pub`-Signaturen | H | DB-S13 |
| DB-F09 | Chunker-Dublette: `src/chunker/*` (≈ 1.100 Zeilen inklusive `unit_tests.rs` 411) neben `ingest::chunking::prose`. Der in der SPEC genannte Paritätstest `engine/tests/proof_chunker_vs_ingest.rs` existiert nicht; vorhanden ist `ingest/tests/chunk_prose_parity.rs`. `ingest` ist in `db` nur dev-Abhängigkeit und ungenutzt. | Dateilisten, `grep` | M | DB-S14 |
| DB-F10 | Zeitfilter vermengt die Achsen: `HybridQueryBuilder::as_of(u64)` ruft `apply_temporal_validity_filter_at`, das den **einen** Wert als `TxId::new(ts)` **und** als Geschäftszeit verwendet (`temporal_filter.rs`, `builder_exec.rs:175`). Geschäftszeit ist `i64` „ms/Sekunden" (Einheit ungeklärt); die SPEC verlangt `i32` Kalendertage seit 1970-01-01 und ein explizites „fehlt" statt `0`. Der Filter läuft **nach** der Fusion auf Ergebnis-Metadaten (kann Treffer unter `k` drücken \[E\]). Metadaten-Schlüssel sind mehrdeutig (`valid_from`, `business_valid_from`, `valid_to`, `valid_until`, `tx_valid_to`, `tx_valid_until`). Kein Golden für Intervallgrenzen. Fehlende Metadaten gelten als „gültig" (Fail-Open). | `temporal_filter.rs:120-170`, `builder_exec.rs:160-190` | H | DB-S40 bis DB-S42 |
| DB-F11 | Löschpfad ohne Löschkette: `delete_op` setzt Tombstones für `user_key` und `doc_key`, stagt Text- und Graph-Delete und ruft `index.delete`; kein Gate, keine Schlüsselvernichtung, keine Lineage, keine Klasse. `drop_collection` löscht Präfixe in Storage und Text, sonst nichts (kein HNSW-/Graph-/KV-Cache-Pfad sichtbar, kein KEK-Widerruf). `deletion_proof_active` wird in allen drei Öffnungspfaden auf `false` gesetzt. T-DEL-1 schlägt laut Bericht reproduzierbar fehl; `T-DEL-2` bis `T-DEL-8` fehlen. | `collection/crud/delete.rs`, `collection/maintenance.rs:852-870`, `contextra_impl/lifecycle.rs:26,42,60`, `docs/02_loeschung.md` §2.9 \[R\] | K | DB-S30 bis DB-S33 |
| DB-F12 | Keine Lesesperre: In `Collection::{get, get_at_snapshot, scan, scan_prefix, search*, hybrid_search*, query()→execute}`, `Contextra::*`, `export`, `SandboxBridge` gibt es keinen Gate-Aufruf. `RequestContext` wird in `db` nur re-exportiert, nicht benutzt. | `grep RequestContext`, Funktionslisten | K | DB-S22 |
| DB-F13 | Keine Klassentrennung, kein `PersonalRef` (0 Treffer im gesamten Repo). `Document { id: String, metadata: Option<Value> }` nimmt beliebiges JSON. `insert*`/`upsert*`/`update`/`put_kv`/`link_memories`/`relate*`/`import` deklarieren keine Klasse. | `grep -rn PersonalRef`, `lib.rs` | K | DB-S20, DB-S21 |
| DB-F14 | Deterministische Pseudonyme in Schlüsseln: `user_key = namespaced_key(id.as_bytes(), 0)` legt die vom Aufrufer gewählte ID im Klartext in den Schlüssel; `doc_key` nutzt `DocId::from_key(id)` (BLAKE3-Präfix, deterministisch). Für Klasse P widerspricht das Z-E2 („keine dauerhaften deterministischen Personenpseudonyme"). | `delete.rs:38-52`, `collection/mod.rs:522` | H | DB-S20 |
| DB-F15 | Klartext im Normalfall: `ContextraConfig::default()` setzt `encryption_passphrase: None`; `ContextraConfig` führt die Passphrase als `Option<String>` (kein `Secret`/`Zeroize`); `zeroize` ist als Abhängigkeit deklariert, wird in `src/` und `tests/` nirgends verwendet. `db` nutzt `kv_shredding` aus `erasure` nicht. | `lib.rs:225,309`, `grep zeroize`, `docs/01_schluesselmodell.md` §1.6 | H | DB-S25 |
| DB-F16 | `export.rs` exportiert Inhalte und Embeddings aller Collections ohne Gate und ohne Klassenfilter. | `export.rs` Kopf und Strukturen | H | DB-S24 |
| DB-F17 | Ungenutzte oder fehl platzierte Abhängigkeiten: `zeroize`, `wat`, `sha2` ohne einen Treffer; `blake3` nur in `tests/` (1 Treffer), nicht in `src/`; `criterion` in `[dev-dependencies]` ohne `benches/`-Verzeichnis und mit höchstens einem Treffer. Bestätigung durch Build offen. | `grep -w` | N | DB-S11 |
| DB-F18 | Feature-Hygiene (R10): `bench`, `edge-reinforcement-learning` und `license` haben keine einzige `cfg(feature = …)`-Stelle in `src/`, `tests/`, `examples/` und leiten nichts weiter (`engine` leitet `edge-reinforcement-learning` weiter, `license` niemand). `experimental-diskann` statt `exp-diskann`; `volatile-vault` leer; `exp-extraction`/`entity-extraction`/`auto-extraction-opt-out` ohne klare Besitzerdokumentation. | `grep`, `Cargo.toml` | M | DB-S11 |
| DB-F19 | `lib.rs` (1.205 Zeilen) mischt Fassaden-Struktur, Konfiguration, `SandboxBridge`-Implementierung, Prelude/Advanced-Re-Exports und Tests. `Contextra` dupliziert die komplette `Collection`-Suchoberfläche (`contextra_impl/crud.rs`, 406 Zeilen) als deprecated Weiterleitungen. | Dateien | M | DB-S17 |
| DB-F20 | `SandboxBridge::db_insert` fällt bei nicht-JSON-Werten still auf `{"raw_data": lossy-UTF-8}` zurück (`unwrap_or`) und ruft `insert` ohne Klasse und Gate. Fail-Open im Sinne von Z-E9. | `lib.rs` (Feature `sandbox`) | M | DB-S21 |
| DB-F21 | Abhängigkeiten in anderen Crates, die `db` blockieren: `erasure::gate::erased_set::ErasedSet::check` prüft per `iter().any(...)` linear, die SPEC verlangt O(1) (§11.4.2), der Doc-Kommentar behauptet O(log n); doppelte `FenceGate`/`WalSink` (F-17); `erasure` ist nicht in `default-members` (F-18); `ContextraError` kennt keine Varianten `Erased`, `Restricted`, `Quarantine`, `ClassViolation` (0 Treffer in `types/src/error.rs`); die Compaction ist über die Fassade nicht erreichbar (`StorageEngine` bietet nur `flush`). | `grep` | H | §7 Abhängigkeiten |
| DB-F22 | `Collection::new` bestimmt den Druckempfänger per `Any`-Downcast auf `LsmStorage`; die `Backends`-Abstraktion wird dadurch umgangen. `Contextra` bindet konkret `LsmStorage` und `HnswIndex`. | `collection/mod.rs:369-411`, `lib.rs` | N | DB-S17 |
| DB-F23 | `multistep.rs:434` dedupliziert mit `std::collections::HashSet`; die Ausgabereihenfolge folgt dem Vektor, die Menge dient nur der Existenzprüfung (voraussichtlich unkritisch für INV-5 \[E\]); `lifecycle.rs:255` nutzt `Instant::now()` für Zeitmessung. | `grep` | N | DB-S13 |

Positiv am Ist: `forbid(unsafe_code)`, kein Panic-Treffer im Produktivteil, saubere 2PC-Dokumentation mit Intent-Log und Idempotenz-Invarianten (`transaction/mod.rs`), injizierbare `Clock` und `MetricsSink` in `Collection`, bestehender Muster-Mechanismus `TenantPolicy::{Optional, Required}` (`lib.rs`), der sich für die neuen Richtlinien wiederverwenden lässt, und mehrere Mehr-Index-Konsistenz-Tests (`fault_injection_2pc.rs`, `atomic_commit.rs`, `split_brain_vector_orphan_test.rs`).

---

## 4 Lückenanalyse (Soll gegen Ist)

| Soll (Quelle) | Ist | Lücke | Befund |
| --- | --- | --- | --- |
| `db` Kern: `engine, types, ports, vault` (§4.1) | Kern: 10 Bausteine direkt, kein `engine` | SPEC und Code widersprechen sich | F02 |
| Default leer (R10) | leer | erfüllt | — |
| Features nach R10 / S-12a | 17, davon 3 ohne `cfg`, Altnamen `experimental-*` | Bereinigung | F17, F18 |
| Kantenrichtung nur abwärts (INV-8, G-L1) | dev→`testkit` (E3); Zyklus mit `cognition` | Entfernen bzw. invertieren | F05 |
| S-22 `vault` | Re-Export + tote `.inc`-Dateien | Löschen, Feature klären | F07 |
| S-60 Regler als `pub use` aus `adapt` | Kopien, API weicht ab | Paritätsarbeit in `adapt` oder Beibehalten mit ADR | F08 |
| S-61 Chunker über `ingest` | Dublette, Paritätstest fehlt | Paritätstest, dann Umstellung | F09 |
| S-63 Cognition-Inversion | Brücke leer, Zyklus, Test-Importe falsch | Implementieren, Tests verschieben | F04, F05, F06 |
| S-64 Lizenz | `license_seam.rs` re-exportiert `ports::license`; Feature `license` leer | Umstellung nach S-50 | F18 |
| S-105 `rules::ValidityWindow` | eigener `ValidityWindow` mit `i64`/`TxId` | Neu typisieren, zwei Achsen trennen | F10 |
| R-CLS, R-HANDLE (§11.2) | keine Klassen, kein `PersonalRef` | Neu | F13, F14 |
| Z-E1, §11.4.2 Lesesperre | keine | Neu, an allen Lesepfaden | F12, F16, F21 |
| §11.3 Schlüsselhierarchie, Z-E2 | globale Passphrase oder Klartext | Umschlagverschlüsselung je Version über Port | F14, F15 |
| §11.4 L2/L5, Drop | Tombstones, Präfix-Löschung | Teilnehmer der Saga, Lineage | F11 |
| §12 Zeitreise (`as_of`, `valid_at`) | vermengte Achsen | Neu typisieren | F10 |
| §13 Simulation | nicht vorhanden | Neu | — |
| Berichte, REPORT.md, `capabilities.fragment.toml` für `db` | `db` hat weder `REPORT.md` noch `capabilities.fragment.toml` | Anlegen | — |

---

## 5 Entscheidungen

Jede Entscheidung braucht ein ADR in `docs/decisions/` (SPEC §10). Die Empfehlungen sind \[D\].

### DB-D01 — `engine`-Merge bestätigen

- **Lage:** ADR-0122 hat `engine` in `db` aufgelöst; die SPEC (§2.2, §4.1) und `AGENTS.md` kennen `engine` noch.
- **Option A (empfohlen):** Merge bestätigen. SPEC §4.1: Zeile `engine` streichen, Zeile `db` auf Kern `ids?, types, ports, vault, mvcc, store, vector, text, graph, rank, checkpoint` ändern; `contextra-engine` bleibt als `#[deprecated]`-Fassade bis S-70 (INV-2) und wird dort entfernt; ADR-0122 nachträglich einchecken. Folgen: `db` hat eine Closure von 13 bis 16 Mitgliedern (E2 ist nicht budgetiert, §4.1), S-60 bis S-64 werden Arbeit **innerhalb** von `db`.
- **Option B:** `engine` neu herauslösen. Verworfen: widerspricht dem Repo-Stand, ca. 13.500 Produktivzeilen müssten zurückwandern, alle Reverse-Abhängigkeiten (`contextra`, `agent`, `bench`) würden erneut umgestellt, kein Nutzen für die Zusagen aus Kap. 11 bis 14.
- **Folgeänderungen der SPEC:** §2.2, §4.1, §9.2 (S-60 bis S-64 Ortsangaben), §9.4 (Zeile „router, engine, db, …"), Anhang C. `db/README.md` („geht in `contextra` auf") ist zu korrigieren oder die SPEC zu ändern; die SPEC gewinnt.

### DB-D02 — Features `erasure` und `rules` in `db`

- `db` braucht für die Lesesperre und die Löschkette Typen aus `erasure` (E0) und für die Zeitachsen Typen aus `rules` (E1a). Beides erlaubt die Ebenenregel (E2 darf E0 und E1 nutzen); §4.1 nennt die Features nicht.
- **Empfehlung:** Zwei benannte Features: `erasure` (→ `contextra-erasure`, Default-Features aus) und `rules` (→ `contextra-rules`). Default bleibt leer. Die **Traits** (`ReadGuard`, `DocumentSealer`, siehe §6) liegen lokal in `db` und sind ohne diese Features nutzbar; die Features liefern nur die Adapter. §4.1 ist entsprechend zu ergänzen.
- **Alternative:** Kernabhängigkeit auf `erasure`. Verworfen: Dann wäre `db` ohne Löschkette nicht mehr baubar, und `erasure` (F-18) ist noch nicht im Default-Workspace-Build.

### DB-D03 — Zeitpunkt der Gate-Pflicht

- Ein `GuardPolicy::{Optional, Required}` (Muster `TenantPolicy`). Bei `Required` scheitert `Contextra::open*` mit einem Fehler, wenn kein `ReadGuard` und kein `DocumentSealer` gesetzt ist; es gibt keinen Standardschlüssel und keinen Fallback (§11.4.4). Das Profil `EnterpriseRegulated` in `contextra` (`collection_profile.rs`) MUSS `Required` setzen (Befund P1 aus `docs/Umsetzbarkeit_Loeschung_Contextra.md` \[R\]; Umsetzung liegt außerhalb von `db`).

### DB-D04 — R-CLS und INV-2

- R-CLS verlangt Abbruch bei fehlender Klasse, INV-2 verlangt gültige alte Importpfade. Alte Methoden (`insert`, `upsert`, …) tragen keine Klasse.
- **Empfehlung:** `DataClassPolicy::{Legacy, Enforced}` in `ContextraConfig`. Unter `Legacy` bleiben die alten Methoden gültig; sie gelten dokumentiert als „Klasse nicht deklariert" und es werden **keine** Aussagen zu Z-E5/G-ER1 gemacht. Unter `Enforced` brechen alte Methoden mit `Invalid` ab (R-CLS), neue Methoden (`writer(DataClass)`) sind der Weg. Der Wechsel des Defaults auf `Enforced` erfolgt in S-70 oder früher über Profile. Das ist eine bewusste, ADR-pflichtige Ausnahme von INV-2, die erst mit S-70 endet.

### DB-D05 — Ort von `Validated<T>`

- R-CLS verlangt, dass der K-Schreiber nur `Validated<T>` entgegennimmt, gebildet über `privacy::entity_recognizer`. `rules` (S-106) und `db` brauchen denselben Typ.
- **Empfehlung:** `Validated<T>` mit privatem Konstruktor in `contextra-privacy` (E0). `db` nimmt `privacy` hinter dem Feature `rules` **oder** als schmale Kernabhängigkeit (E0, erlaubt). `db/src/adapters/privacy.rs` ist heute nur ein Tenant-Konverter und ändert daran nichts.

### DB-D06 — Z-E8 und BM25-Korpusstatistik

- Z-E8 verlangt, dass „Rangfolge, Zählwerte und Fehlerbilder sich nicht durch unzugängliche Dokumente ändern". Die Fusion (BM25 mit IDF, HNSW-Kandidatenpool, RRF) nutzt heute Korpusstatistik über **alle** Dokumente der Collection. Ein Nachfilter genügt dafür nicht.
- **Optionen:** (1) Statistik je Zugriffsbereich (Collection pro Bereich oder Statistikpartitionen), (2) Statistik zur Abfragezeit aus der zugänglichen Teilmenge (Kosten), (3) Z-E8 auf „Treffermenge" einschränken und die Abweichung der Statistik als dokumentiertes Restrisiko führen (SPEC-Änderung).
- **Empfehlung:** Spike DB-S23 (Messung von Rangänderung und Kosten), dann Entscheidung. Bis dahin implementiert DB-S22 die Zusage auf Trefferebene (Kandidaten- und Ausgabefilter mit Auffüllen auf `k`), und der Test Z-E8 wird als erwarteter Fehlschlag mit Verweis auf DB-D06 geführt (kein stilles Abschwächen, INV-10).

### DB-D07 — Kanonische Gültigkeitsdarstellung und Migration

- Neue kanonische Metadatenfelder: `valid_from_day`, `valid_until_day` (`i32`, Tage seit 1970-01-01, fehlend = offen). Alte Schlüssel (`valid_from`, `valid_to`, `valid_until`, `business_valid_*`, `tx_valid_*`) werden **weiter gelesen** (INV-1: kein persistiertes Byte ändert sich; kein Umschreiben). Einheiten werden **nicht** geraten: Die Einheit der Altfelder setzt eine Collection-Einstellung `legacy_validity_unit ∈ {Days, Seconds, Millis}`; ohne Einstellung gelten Altfelder als „Einheit unbekannt" und der Filter lehnt die Abfrage mit `Invalid` ab, sofern ein Altfeld getroffen wird, statt Fail-Open zu raten. Das ändert das heutige Verhalten und ist als INV-2-Ausnahme zu führen.
- **Offen:** Welche Einheit die Bestandsdaten und die Tests (`valid_until: 100/200`) meinen, ist am Code nicht erkennbar \[E\]; ein Eigentümer muss sie festlegen.

### DB-D08 — Reichweite von Z-T1

- Z-T1 verlangt byte-identische Ausgaben auf x86_64 und aarch64 und „Rangfolgen ohne Gleitkommaarithmetik". Für `rules` (Fristen, Kalender, Festkomma) ist das erfüllbar. Die Dokumentensuche in `db` nutzt f32-Distanzen, SIMD-Pfade und BM25-Gleitkomma; Byte-Identität über Architekturen ist dort nicht zusagbar.
- **Empfehlung:** SPEC §12.7 präzisieren: Z-T1 gilt für `rules`-Ausgaben und die **Zeitauswahl** (welche Dokumentversionen sichtbar sind); für ANN- und BM25-Rangfolgen gilt INV-5 (gleiche Plattform, gleiche Eingabe, Tie-Break nach ID aufsteigend). `db` stellt das mit Golden-Tests je Plattform für die Zeitauswahl sicher.

### DB-D09 — Regler: Parität in `adapt` oder Ausnahme

- Siehe DB-F08. **Empfehlung:** Zuerst Differenzanalyse; `adapt` übernimmt die fehlenden Methoden additiv (`with_clock`, `*_ms`, `*_wall_clock`; `RerankDeadline` mit `Clock`-Parameter bleibt, `db` nutzt `SystemClock`); `db::homeostat` wird Wrapper mit alter Signatur auf `adapt` und `#[deprecated]`. `PidLatencyController` (Skalierungsfaktor) bleibt in `db` oder zieht als eigener Typ nach `adapt::pid` um (kein `pub use` auf einen semantisch anderen Typ). ADR nötig, weil S-60 „bitgleich" verlangt und die Typen verschieden sind.

---

## 6 Zielentwurf für `contextra-db`

Alle Typnamen sind Vorschläge \[D\]; öffentliche Signaturen werden in S-00 mit `cargo public-api` gegengeprüft.

### 6.1 Crate-Grenzen und Features (Ziel)

```toml
[features]
default = []                                   # R10
cognition  = ["dep:contextra-cognition"]       # SPEC §4.1
erasure    = ["dep:contextra-erasure"]         # DB-D02 (neu, SPEC ergänzen)
rules      = ["dep:contextra-rules", "dep:contextra-privacy"]  # DB-D02, DB-D05
exp-diskann = ["contextra-vector/exp-diskann"] # später → vector-diskann (S-80)
sandbox, onnx, reranking, background-maintenance, graph-connectivity-health,
coherence-bonus-fusion, adaptive-candidate-pool-sizing, volatile-vault,
exp-extraction, entity-extraction, auto-extraction-opt-out   # bleiben; Besitzer je Feature dokumentieren
# Aliase bis S-70: experimental-diskann
# entfallen (R10): bench, edge-reinforcement-learning, license — sofern S-64 `license` nicht als echtes Feature braucht
```

Abhängigkeiten, die verschwinden (nach Build-Bestätigung): `zeroize` (wird in DB-S25 wieder benutzt, sonst entfernen), `wat`, `sha2`, `criterion` (nur mit `benches/`), dev-`testkit`, dev-`ingest`. `blake3` wandert nach `[dev-dependencies]`.

### 6.2 Modulstruktur (Ziel, ohne Crate-Zerlegung — D-01, §9.3-Nicht-Ziel)

```
src/
  lib.rs            Re-Exports, prelude, advanced (Ziel < 300 Zeilen; INV-2 Pfade bleiben)
  config.rs         ContextraConfig, EmbeddingBackend, TenantPolicy, GuardPolicy, DataClassPolicy
  contextra_impl/   Contextra (unverändert, nutzt config.rs)
  collection/       wie heute; neu: view.rs (ReadOnlyCollection), classed.rs (ClassedWriter)
  guard/            NEU: ReadGuard, GuardTarget, GuardVerdict, DocumentSealer, Lineage
    erasure_adapter.rs   (Feature `erasure`)
  erase/            NEU (Feature `erasure`): DbPurgeTarget, DbScanLayer, Drop-Pfad
  temporal/         NEU: window.rs, query.rs, filter.rs  (ersetzt temporal_filter.rs; Alt-Pfad = Re-Export)
  sim/              NEU (Feature `rules`): SimSession, SimStorage, SimRun
  cognition/        Brücke (Feature `cognition`): Implementierungen der cognition-Ports
  transaction/      unverändert; Erweiterung um Gate-/Lineage-Hooks
```

Entfallen: `chunker/` (nach S-61), `decay_controller.rs`/`homeostat.rs`/`pid_latency_controller.rs` (als Re-Exports, DB-D09), `volatile_vault.rs` (Re-Export bleibt bis S-70), `vault_sys.inc`, `vault_compat.inc`, `multistep_pid.inc` (prüfen, ob `include!` noch nötig ist).

### 6.3 Klassenmodell und Schreibpfad (R-CLS, R-HANDLE)

```rust
#[non_exhaustive] pub enum DataClass { K, P, D, E }

pub struct ClassedWriter<'c, S, V> { /* Borrow auf Collection, Klasse, Zweck */ }
impl Collection<S, V> {
    pub fn writer(&self, class: DataClass) -> Result<ClassedWriter<'_, S, V>>;
}
impl ClassedWriter {
    // K: nimmt nur Validated<KPayload>
    pub async fn put_k(&self, key: KKey, v: Validated<KPayload>) -> Result<()>;
    // P: Schlüssel ist PersonalRef (128 Bit, zufällig), Inhalt wird versiegelt
    pub async fn put_p(&self, subject: SubjectRef, doc: &[u8], ctx: &WriteCtx) -> Result<PersonalRef>;
}
```

Regeln:

1. Unter `DataClassPolicy::Enforced` brechen alle bisherigen Schreibmethoden mit `ContextraError::InvalidInput` (bzw. `ClassViolation`, siehe §7 Vorbedingung V4) ab; es wird nichts geschrieben (R-CLS).
2. K-Schreiben ruft den Validator (`privacy::entity_recognizer`, `recognize_bounded`) **vor** dem ersten `storage`-Zugriff und scheitert bei Treffer (fail-closed, Z-E5). Der K-Pfad nimmt `Validated<T>`; ein `put_k` mit rohem `T` kompiliert nicht (G-ER1: Compile-Fail-Test mit `trybuild`).
3. P-Schlüssel sind `PersonalRef` (S-100). Die deterministische Ableitung `DocId::from_key(id)` darf für P **nicht** verwendet werden (DB-F14). Das Verzeichnis `PersonalRef → (Betroffener, Dokument, Version)` liegt in P (verschlüsselt).
4. D-Schreiben (Indizes, Embeddings, Caches) trägt immer `Lineage` (§6.5).
5. E-Schreiben (Journal, Ledger, Bericht) liegt in `erasure`; `db` schreibt nie E-Klasse selbst.
6. `SandboxBridge::db_insert` nutzt den Pfad `writer(K)` mit Validator; nicht-JSON-Werte werden **abgelehnt**, nicht umgedeutet (DB-F20).

### 6.4 Lesesperre (Z-E1, Z-E8, §11.4.2)

```rust
pub trait ReadGuard: Send + Sync {
    /// O(1), ohne I/O, darf nicht blockieren (§11.4.2)
    fn check(&self, t: &GuardTarget) -> GuardVerdict;      // Open | Erased | Restricted
}
pub struct GuardTarget { pub tenant: TenantId, pub collection: CollectionId,
                         pub doc: GuardDoc /* DocId | PersonalRef */, pub version: Option<u64>,
                         pub purpose: Purpose /* Normal | Export | Simulation(CaseRef) | Timetravel */ }
```

- Injektion: `Contextra::set_read_guard(Arc<dyn ReadGuard>)` und Feld in `Collection` (Muster `set_clock`, `set_metrics_sink`). Unter `GuardPolicy::Required` ist ein fehlender Guard ein Öffnungsfehler.
- **Prüfpunkte (vollständig, jeweils ein Test, siehe §8):**

| Lesepfad | Datei/Funktion | Stelle der Prüfung |
| --- | --- | --- |
| Einzelabruf | `collection/crud/read.rs`: `get`, `get_at_snapshot`, `get_tracked`, `get_at_seq_tracked`, `get_doc_tx` | vor dem Entschlüsseln und vor dem Rückgeben |
| KV | `collection/crud/kv.rs`: `get_kv` | wie oben |
| Scan | `read.rs`: `scan`, `scan_prefix` | je Eintrag vor Ausgabe |
| Vektor-/Textsuche | `collection/search/{basic,filtered}.rs` | auf Kandidatenebene vor Fusion/Rerank, mit Auffüllen auf `k` |
| Hybrid | `search/hybrid/{mod,query}.rs`, `query_builder/builder_exec.rs` | wie oben; Reranker bekommt nur freigegebene Kandidaten |
| Links/Graph | `crud/links.rs`: `get_links`; `search/hybrid/mod.rs`: `traverse_links` | je Kante und Zielknoten |
| Multistep | `multistep.rs` | über die darunterliegenden Pfade |
| Zeitreise | `temporal/*`, `with_pinned_checkpoint*` | gleiche Prüfung, ältere Versionen eingeschlossen (Z-E1) |
| Simulation | `sim/*` | `purpose = Simulation`, Vorgangsbindung |
| Export | `export.rs` | je Eintrag; gesperrte Einträge werden ausgelassen und im Bericht nur gezählt |
| Cache/Vorschau/Sitzung | `collection/mod.rs` (Importance-/KV-Caches), `agent`-seitig | Lineage-gestützte Bereinigung (L2), Prüfung beim Lesen |
| Sandbox | `lib.rs`: `SandboxBridge::{db_search, db_get}` | wie Einzelabruf/Suche |

- **Fehlerbild:** Für Berechtigte `Erased`; für Unberechtigte dasselbe Fehlerbild wie bei nicht vorhandenem Ziel (SPEC §11.4.2). Entschlüsselungsfehler sind nach außen nicht unterscheidbar (`docs/01_schluesselmodell.md` §1.3).
- **Zählwerte (Z-E8):** `Collection::len`, `stats`, `is_empty`, `Contextra::stats` dürfen keine gesperrten Ziele zählen, oder die Zusage wird ausdrücklich eingeschränkt (DB-D06).

### 6.5 Versiegelung und Lineage (Z-E2, §11.3, D-Klasse)

```rust
pub trait DocumentSealer: Send + Sync {
    fn seal(&self, plain: &[u8], aad: &Aad /* tenant, collection, doc/PersonalRef, version */)
        -> Result<Envelope, SealError>;                       // fallibler Zufall, kein Ersatzschlüssel
    fn open(&self, env: &Envelope, aad: &Aad) -> Result<Zeroizing<Vec<u8>>, OpenError>; // Erased/Integrity einheitlich
}
```

- Die Implementierung liefert `erasure` (`kv_shredding`: `GroupEpochKey`, `RecordDek`; DEK je Version nach §11.3) über einen Adapter im Feature `erasure`. `db` implementiert **keine** eigene Kryptographie.
- Reihenfolge im Schreibpfad: Gate-Prüfung → Validator/Klasse → `seal` → Storage → Indizes (D) mit `Lineage`. Der Klartext verlässt den Speicher von `db` nicht vor `seal` (`Zeroizing`).
- `ContextraConfig::encryption_passphrase: Option<String>` wird zu `Option<SecretString>` (Zeroize on Drop, kein `Clone`, kein `Debug`-Klartext) und bleibt als Store-weite Verschlüsselung erhalten; sie ersetzt **keine** Löschung je Version (DB-F15).
- **Lineage (D):** Jedes abgeleitete Objekt einer Version (HNSW-Knoten, Posting-Einträge, Graph-Entitäten, KV-Cache-Segmente, Konsolidierungsausgaben, Chunk-Cache, Sitzungs-Kontexte) trägt einen `LineageRef { source: DocId|PersonalRef, version }` oder liegt in einem Segment, das genau diese Quelle trägt. `LineageIndex::derived_of(source, version) -> Iterator<DerivedRef>` ist die Grundlage für L2 (Bereinigung) und L5 (Messung). Ohne Lineage keine Aussage „Bereinigt".
- **Grenze:** Dass Postings und Vektoren selbst „vor dem Schreiben unter AEAD" liegen (Z-E2 für D-Klasse), erfordert Änderungen in `text`, `vector`, `store` (S-32, S-36, S-37). `db` liefert dafür Schnittstelle, Lineage und den Canary-Test; die Zusage Z-E2 gilt für `db` erst, wenn diese Crates mitziehen.

### 6.6 Löschkette: Teilnahme von `db` (§11.4)

| Stufe | Rolle von `db` | Umsetzung |
| --- | --- | --- |
| L0/L1 | Keine eigene Logik; Gate-Wirkung über `ReadGuard` sofort nach Rückkehr von L1 | `ReadGuard` aus `erasure::gate` (Feature `erasure`) |
| L2 Bereinigung | `DbPurgeTarget` implementiert die Purge-Schnittstelle von `erasure` für: Storage-Schlüssel (user/doc/links), Text-Postings (`stage_text_delete`), HNSW (`index.delete` plus Neuaufbau-Hinweis), Graph (`stage_graph_entity_delete`), KV-Caches, Chunk-Cache, Sitzungen, Temp-Dateien | je Segment-/Manifest-Batch `fsync` über `store`; Idempotenz wie `recover_pending_intents` |
| L3/L4 | Keine Logik in `db`; `db` darf versiegelte Daten nach Schlüsselvernichtung nicht fallback-lesen | Z-E9, Fehlerinjektionstest |
| L5 Messung | `DbScanLayer` implementiert die Scanner-Schicht (`ports::scanner::layer`) für Text-, Vektor-, Graph-, Cache-Schicht mit Positivkontrolle (Canary in `db`-Pfad einfügen, Scanner muss ihn finden) | `NotImplemented` für jede Schicht ohne Verfahren, nie „reached" (Z-E10) |
| Drop einer Collection | Statt reinem Präfix-Delete: Saga mit KEK-Widerruf der Gruppe und anschließender Bereinigung aller Indizes; neue Collection gleichen Namens erhält neue Epoche (`docs/02_loeschung.md` §2.6, Befund N11 \[R\]) | `Collection::drop_collection` bleibt als Pfad bestehen, ruft unter Feature `erasure` die Saga, sonst bisheriges Verhalten mit dokumentierter Stufe „nur logisch entfernt" |
| Physische Spuren in SSTables | Tombstone allein entfernt Klartext nicht (T-DEL-1). Unter Z-E2 liegt kein Klartext vor, sodass Schlüsselvernichtung genügt; ohne Versiegelung braucht es eine erreichbare Compaction | Vorbedingung V5 (`store`/`ports`), nicht in `db` lösbar |

`delete`/`delete_op` bleiben als **logische** Entfernung (Stufe „Gesperrt" nur zusammen mit Gate-Eintrag). Sie behaupten nichts weiter; Berichte nennen die erreichte Stufe (INV-7).

Crash-Invarianz (Z-E4): Alle `db`-Schreibstellen, die an L2 teilnehmen (`DbTransaction`-Phasen, Intent-Log `__tx_intent:<tx>` bzw. `__col:<name>:\x03<tx>`), werden in die Crash-Matrix (§14.2) aufgenommen; physische Tatsachen haben Vorrang vor dem Journalstand.

### 6.7 Zeitreise (Kap. 12)

```rust
pub struct ValidDay(pub i32);                 // Tage seit 1970-01-01
pub enum  AsOf { Latest, Seq(u64), TxTime(u64 /*ms*/) }
pub struct TemporalQuery { pub as_of: AsOf, pub valid_at: Option<ValidDay> }
pub struct ValidityWindow { pub from: Option<ValidDay>, pub until: Option<ValidDay> } // [from, until); None = offen
```

- `ValidityWindow` kommt aus `rules` (S-104/S-105); `db::temporal` hält nur die Anbindung an Dokumentmetadaten und Collection-Sicht.
- **Zwei Achsen getrennt:** `as_of` wählt den MVCC-Stand (`snapshot_seq`, `get_at_snapshot`, `with_pinned_checkpoint`); `valid_at` filtert Gültigkeit. `TxTime → Seq` über einen je Mandant dichten `tx_seq`-Index mit `tx_time` (Klemmen/`clamped`, Fehler über `clock_skew_max` = 5 min, §12.2); `db` nutzt ihn nur lesend, der Besitzer ist `rules` (S-104).
- **Vorfilter statt Nachfilter:** Die Gültigkeitsprüfung wirkt auf Kandidatenebene (vor Fusion/Rerank) und füllt auf `k` auf. Der heutige Nachfilter wird für Alt-Aufrufer als Pfad erhalten (INV-2) und `#[deprecated]` markiert.
- **Fehlend ist ein Zustand:** `Option::None` bedeutet „offen", nie `0`. Alte Metadatenschlüssel gemäß DB-D07 gelesen; ohne bekannte Einheit Abfragefehler statt Raten.
- **Altpfad:** `HybridQueryBuilder::as_of(u64)` bleibt mit dokumentierter Altsemantik (ein Wert für beide Achsen) und `#[deprecated(note = "as_of_seq / valid_at_day verwenden")]`. Neue Methoden: `as_of_seq(u64)`, `as_of_tx_time(u64)`, `valid_at_day(i32)`.
- **Z-T5:** Die Zugriffsrichtlinie wird aus dem **aktuellen** Stand ausgewertet, auch bei `as_of` in der Vergangenheit (Guard liest nie historische Rechte).
- **Z-T2:** Spätere Schreibvorgänge ändern frühere `as_of`-Antworten nicht; Eigenschaftstest über zufällige Schreibfolgen.

### 6.8 Simulation (Kap. 13)

- `Contextra::simulation(base: CommitRef, scenario: ScenarioRef) -> Result<SimSession>` (Feature `rules`). Der Szenario-Branch `sim/<id>` und die Commits gehören `rules` (S-107); `db` liefert die **Laufumgebung**.
- `SimStorage<S>` umhüllt `StorageEngine`: Lesen wie `main`; **jeder Schreib- oder Löschzugriff** außerhalb des Namensraums `__sim:<id>:` liefert einen Fehler. Es gibt keinen Zugriff auf Gate-Register, Policy oder Schlüssel mit Schreibrecht.
- `ReadOnlyCollection` bietet nur Lesemethoden (typseitig kein `insert`/`delete`); Kandidaten laufen durch `ReadGuard` mit `purpose = Simulation(case_ref)`; P nur mit Zugriffsrecht und Vorgangsbindung (§13.3).
- Ändert ein Szenario Chunking oder Embedder, entsteht ein temporärer Index unter `__sim:<id>:`; `retire` entfernt Namensraum und Index.
- Ergebnisse mit Personenbezug werden versiegelt (`DocumentSealer`, Klasse E/P-Ableitung) und tragen eine Ablauffrist; `SimRun` enthält Basis-Commit, Szenario-Commit, Parameter, Ergebnisse und den Hinweis „Entscheidungshilfe" (Z-S3).
- Keine Löschwirkung: `SimSession` besitzt keine Referenz auf `delete`, `drop_collection` oder die Saga (INV-16).

### 6.9 Cognition-Brücke (S-63)

- `db::cognition` implementiert für `Collection` die Ports von `cognition`: `DocumentSource` (`fetch_turns`), `ConsolidationSink` (`apply_intent`), `CollectionOps` (`name`, `namespaced_key`, `user_key_prefix`, …), und, soweit nötig, `HyperedgeSink` und `DecayPolicy`.
- `CommitIntent`: `cognition` führt einen eigenen Ersatztyp (`ports.rs:16-21`); `db::transaction::CommitIntent` bleibt der persistierte Typ. Die Abbildung liegt in `db::cognition` (SPEC-Risiko §10: „Port scheitert ⇒ eigener Typ in `cognition`, Abbildung in der `db`-Brücke" ist eingetreten). Abbildung tabellengetrieben testen, inklusive `Consolidation` und `Failed { reason }`.
- `cognition` verliert dev-Abhängigkeit auf `db`; Tests, die beide brauchen, ziehen nach `testkit/tests/cognition/` (E3 → E2, erlaubt) um, oder `db`-Tests nutzen `cognition` als Feature. Die beiden Tests mit `contextra_db::memory_consolidation` werden auf `contextra_cognition::memory_consolidation` umgestellt und in `cognition` verortet.

### 6.10 Fehler

`db` behält `ContextraError` (SPEC §4.1: „Fehler —"). Benötigt werden neue Varianten (nur additiv, `#[non_exhaustive]` prüfen): `Erased`, `Restricted(Reason)`, `Quarantine`, `ClassViolation`. Sie liegen in `types` (Vorbedingung V4). Display-Texte enthalten keine Roh-Kennungen (Z-E7). `Erased` und „nicht gefunden" erzeugen für Unberechtigte dasselbe äußere Bild.

---

## 7 Umsetzungsplan

### 7.1 Spielregeln (aus SPEC §9.1)

Jeder Schritt ist ein Commit-Paar `DB-Sxx A: …` (verschieben/umbenennen) und `DB-Sxx B: …` (umbauen), einzeln revertierbar (INV-3, D-08). Der Schritt ist fertig, wenn: `cargo check --workspace --all-targets --locked` grün (INV-4), die genannten Gates grün, Score und Brückenzählung neu erzeugt, keine neue Verletzung von R10 bis R14, der Revert-Commit baut, Verweise in geänderten Dokumenten lösen auf (G-DOC1) und der Eintrag in `crates/contextra-db/SPEC.md`/`PLAN.md` aktualisiert ist. Kein Schritt ändert persistierte Formate (INV-1), außer ausdrücklich genannt.

Aufwand: S ≤ 1 Tag, M 2 bis 4 Tage, L 5 bis 9 Tage, XL ≥ 10 Tage \[E\].

### 7.2 Vorbedingungen in anderen Crates

| ID | Vorbedingung | Besitzer | SPEC | Blockiert |
| --- | --- | --- | --- | --- |
| V1 | `PersonalRef` (128 Bit, zufällig, serde, Golden) | `ids` | S-100 | DB-S20 |
| V2 | `FenceGate`/`WalSink` je ein Besitzer; `erasure` in `default-members`, Check grün; Lebenszyklustabelle §11.4 gegen Code; G-ER2 | `erasure` | S-101, S-102, S-103 | DB-S22 bis DB-S33 |
| V2a | Gate-Prüfung in O(1) statt linearer Suche (`ErasedSet::check`) | `erasure` | neu (an S-103 anhängen) | DB-S22 |
| V3 | `rules` (E1a) mit `ValidityWindow`, `tx_seq`-Index, Branches, Takedown | `rules` | S-104, S-105, S-107, S-108 | DB-S40 ff., DB-S50 ff. |
| V3a | `Validated<T>` mit privatem Konstruktor | `privacy` | S-106 (erweitern) | DB-S21 |
| V4 | Fehlervarianten `Erased`, `Restricted`, `Quarantine`, `ClassViolation` | `types` | neu | DB-S21, DB-S22 |
| V5 | Erreichbare Compaction/Purge für Storage über die Fassade (oder Verzicht dank Versiegelung) | `store`, `ports` | S-37 (erweitern) | DB-S31 (Teilziel physische Spuren) |
| V6 | Text-/Vektor-/Store-Strukturen für D-Klasse unter AEAD | `text`, `vector`, `store` | S-32, S-36, S-37 (erweitern) | Z-E2 für D-Klasse |
| V7 | `adapt` nimmt fehlende Regler-Methoden additiv auf | `adapt` | S-60 | DB-S13 |
| V8 | Dubletten `store/scanner/*` neben `compat/` und `vector/hnsw/*` neben `compat/` geklärt | `store`, `vector` | S-36, S-37 | Entnehmbarkeit (indirekt) |

### 7.3 Phasenübersicht

| Phase | Ziel | Schritte | Abhängig von |
| --- | --- | --- | --- |
| 0 | Prüfbarkeit herstellen | DB-S00 bis DB-S03 | — |
| 1 | Strukturbereinigung (SPEC §9, Welle 1 bis 4) | DB-S10 bis DB-S18 | Phase 0, DB-D01, DB-D02 |
| 2 | Klassentrennung, Lesesperre, Versiegelung | DB-S20 bis DB-S25 | Phase 1, V1, V2, V2a, V3a, V4 |
| 3 | Löschkette | DB-S30 bis DB-S34 | Phase 2, V2, V5 |
| 4 | Zeitreise | DB-S40 bis DB-S43 | Phase 1, V3 |
| 5 | Simulation | DB-S50 bis DB-S52 | Phase 2, Phase 4, V3 |
| 6 | Abschluss | DB-S60 bis DB-S63 | alle |

Phasen 2 und 4 sind nach Phase 1 parallel möglich (verschiedene Dateien: `guard/` und `collection/*` gegen `temporal/`).

### 7.4 Schrittkatalog

#### Phase 0 — Prüfbarkeit

**DB-S00 Build-Fähigkeit und Baseline** · M · Risiko M · Voraussetzung: —
- Wurzelmanifest und Gate-Werkzeuge im Arbeitsklon wiederherstellen (DB-F01); Klärung, ob die fehlenden Dateien (`Cargo.toml`, `justfile`, `governance/`, `.github/`) in der Quelle vorhanden sind.
- Toolchain 1.89.0 bereitstellen (conda-forge/apt-Weg laut `CLAUDE.md`; INV-9).
- Ausführen und festhalten: `cargo check -p contextra-db --no-default-features --locked`, `--all-targets`, `cargo hack check -p contextra-db --each-feature --no-dev-deps`, `cargo tree -p contextra-db -e normal,dev --no-default-features`, `cargo public-api -p contextra-db`.
- Zählung der 35 `from_key`-Stellen (mechanisch via `?` / nicht-mechanisch per Musterabgleich); erwartet: 0 Musterabgleiche im Produktivteil \[E\]; Rückfall §9.5 der SPEC nur bei > 20.
- **Abnahme:** `docs/decoupling/baseline/db/` mit HEAD-Hash und Datum; Liste aller Kompilierfehler je Target.

**DB-S01 Tests wieder baubar** · M · Risiko M · Voraussetzung: DB-S00
- `tests/docid_128_consolidation.rs`, `tests/consolidation_double_trigger_test.rs` auf `contextra_cognition::memory_consolidation` umstellen oder nach `cognition` verschieben (Zusammenspiel mit DB-S15).
- `tests/fault_injection_2pc.rs` Typfehler beheben (Ursache zuerst am Baseline-Log feststellen); die zehn Test-Targets und das Example einzeln durchgehen.
- Branch `fix/test-targets` klären (fehlt im Klon).
- **Abnahme:** `cargo test -p contextra-db --locked` kompiliert; rote Tests sind einzeln als Befunde erfasst, keine Abschwächung von Assertions (INV-10 sinngemäß).
- **Rollback:** Revert; Tests bleiben im Ausgangszustand.

**DB-S02 Dokumentation neu aufsetzen** · S · Risiko N · Voraussetzung: DB-S00, DB-D01
- `crates/contextra-db/SPEC.md` und `PLAN.md` durch dieses Dokument (gekürzt) ersetzen; `AGENTS.md` regenerieren (`tools/crate_inventory.py`); `README.md` korrigieren (kein „geht in `contextra` auf"); `DEPRECATED_DEBT.md` auf aktuelle Pfade bringen; Modulköpfe „Gehört zur Crate `contextra-engine`" korrigieren; `capabilities.fragment.toml` und `docs/decoupling/REPORT.md` anlegen.
- **Abnahme:** G-DOC1 grün.

**DB-S03 ADR und SPEC-Änderungen** · S · Risiko N · Voraussetzung: DB-D01 bis DB-D09
- ADR-0122 (Merge) nachträglich; ADRs zu DB-D02, D04, D05, D06, D07, D08, D09; SPEC-Änderungsantrag (§2.2, §4.1, §9.2, §9.4, §12.7, Anhang C) mit Label `protected-path`, soweit Governance-Dateien berührt sind (INV-10).

#### Phase 1 — Strukturbereinigung

**DB-S10 Vault (S-22)** · S · Risiko N · Voraussetzung: DB-S00
- `vault_sys.inc` und `vault_compat.inc` löschen (nach `grep`-Nachweis, dass sie nirgends eingebunden sind; DB-F07); `PurgeReceipt`-Alias aus `vault_compat.inc` nur übernehmen, wenn ein Verbraucher ihn braucht (`agent`: Feldname `reset_receipt` laut Fix-Report \[R\]); `volatile-vault`-Feature: entweder `dep:`-Gating (Abhängigkeit optional machen) oder Feature entfernen, mit `agent` (Feature-Verbraucher) abstimmen.
- **Abnahme:** `grep -rn 'volatile_vault' crates --exclude-dir=contextra-vault` nur Re-Exports; `tests/golden_vault.rs` grün; GL.

**DB-S11 Feature- und Abhängigkeitsbereinigung (S-12a)** · M · Risiko M · Voraussetzung: DB-S00, DB-D02
- `experimental-diskann` → `exp-diskann` mit Alias; `bench`, `edge-reinforcement-learning`, `license` entfernen oder begründen (Weiterleitung in `engine` beachten, solange die Fassade existiert); Besitzerdokumentation je Feature in `README.md`; ungenutzte Abhängigkeiten (`zeroize` bis DB-S25 behalten, `wat`, `sha2`, `criterion`) nach Build-Bestätigung (`cargo udeps` oder Entfernen und Bauen) streichen; `blake3` nach dev; `version` an Pfad-Abhängigkeiten ergänzen (F-12).
- Features `erasure` und `rules` anlegen, zunächst leer-leitend (kompilieren ohne Verwendung), damit spätere Schritte additiv bleiben.
- **Abnahme:** GH; `cargo hack check -p contextra-db --each-feature`; `cargo tree -p contextra-db -e features` zeigt keinen verbotenen Default.

**DB-S12 Kantenbereinigung** · S · Risiko N · Voraussetzung: DB-S00
- dev-Abhängigkeit `contextra-testkit` und `contextra-ingest` entfernen (ungenutzt, DB-F05); die dev-Abhängigkeiten auf `store`, `vector`, `text`, `graph`, `adapt`, `checkpoint` bleiben (E1 nach E2, abwärts); doppelte Einträge (`vector`, `store`, `graph` in `[dependencies]` **und** `[dev-dependencies]`) vereinheitlichen.
- **Abnahme:** G-L1 (normal, dev, Feature) für `db` grün.

**DB-S13 Regler (S-60, DB-D09)** · M · Risiko M · Voraussetzung: DB-S00, V7
- Differenzanalyse der drei Paare; Differenztest (Commit A) mit `ManualClock`, bitgleich in f64/f32 wie die SPEC verlangt (§6.4); additive Ergänzungen in `adapt`; `db::{decay_controller, homeostat}` werden Wrapper/Re-Exports mit alter Signatur und `#[deprecated]`; `PidLatencyController` nach Entscheidung. `Instant::now()` in `homeostat.rs` entfällt (injizierte `Clock`, INV-5).
- **Abnahme:** GD (bitgleich), G-A1 ohne ungewollte Diffs; `tests/pid_pool_production_caller.rs` unverändert grün.
- **Rollback:** B revertieren; A (reiner Differenztest) bleibt.

**DB-S14 Chunker (S-61)** · M · Risiko M · Voraussetzung: DB-S00
- Paritätstest `db/tests/chunker_vs_ingest.rs` auf Chunk-Folge **samt Breadcrumbs** und Reihenfolge (Korpus aus `ingest/tests/corpus`); erst bei Gleichheit `db::chunker` auf `ingest::chunking::prose` umstellen; `contextra::chunker::MarkdownChunker` bleibt API-stabil. `ingest` wird normale, nicht-optionale Abhängigkeit (E0, erlaubt) oder Feature; Entscheidung im ADR.
- **Abnahme:** GD; `mcp`-Tests unverändert grün; die 411 Zeilen `unit_tests.rs` wandern als Regressionstests mit.

**DB-S15 Cognition-Inversion (S-63)** · L · Risiko H · Voraussetzung: DB-S01, DB-S12, DB-D01
- Implementierungen nach §6.9; `cognition`-Seite: `*_local.rs` löschen, Abhängigkeit auf `graph` und dev-`db` entfernen (Besitzer: `cognition`, Schritt wird dort gespiegelt); Test-Umzug; Abbildungstest `CommitIntent`.
- **Abnahme:** `cargo tree -p contextra-cognition -e normal` ohne `contextra-engine` und ohne `contextra-db`; G-L1 ohne Zyklus; `consolidation_*`-Tests grün; GL.
- **Risiko:** Der Port-Zuschnitt scheitert an Typen, die `cognition` heute aus `engine` zieht; Rückfall wie in der SPEC beschrieben (eigener Typ in `cognition`, Abbildung in `db`).

**DB-S16 Lizenz (S-64, nach S-50)** · S · Risiko N · Voraussetzung: S-50, S-20
- `license_seam.rs` auf `contextra-license` zeigen lassen; Test „ohne Feature `license` gilt `OpenFastGate`-Semantik".
- **Abnahme:** `grep -rn 'contextra_ports::license' crates/contextra-db/src` nur im Seam; GL.

**DB-S17 `lib.rs` entflechten (INV-2-neutral)** · M · Risiko M · Voraussetzung: DB-S00
- `config.rs` abspalten (Konfigurationstypen), `SandboxBridge`-Implementierung nach `sandbox/`, Prelude/Advanced bleiben als Re-Export; `Collection::new`-Downcast auf `LsmStorage` durch eine Methode am `StorageEngine`-Port oder einen Konstruktor-Parameter ersetzen (DB-F22).
- **Abnahme:** G-A1: keine Differenz der öffentlichen Pfade; `lib.rs` < 300 Zeilen.

**DB-S18 Vorbereitung S-70** · S · Risiko N · Voraussetzung: DB-S11 bis DB-S17
- Liste aller `#[deprecated]`-Re-Exports und `#[allow(deprecated)]`-Stellen (40) erneuern; `tests/no_blanket_allow_deprecated.rs` bleibt grün.

#### Phase 2 — Klassen, Lesesperre, Versiegelung

**DB-S20 `PersonalRef` und Schlüsselpolitik** · M · Risiko H · Voraussetzung: V1, DB-S11
- Schlüsselraum für P: `PersonalRef`-basierte Schlüssel (`__p:` Präfix, Namensraum neben `__col:`); keine deterministische ID-Ableitung für P (DB-F14); Verzeichnis `PersonalRef → (SubjectRef, DocRef, Version)` in P.
- K-Pfade verweisen auf P nur über `PersonalRef` (R-HANDLE).
- **Abnahme:** Test „Z-E6" (Vorher/Nachher-Vergleich: Objekt-IDs, Commit-Hashes, K-Ledger unverändert nach P-Löschung); Test Metadaten: Schlüssel und Dateien enthalten keine ID-Klartexte und keine stabilen Pseudonyme für P (Canary-Suche).

**DB-S21 Klassen und Schreibpfad (R-CLS)** · L · Risiko H · Voraussetzung: V3a, V4, DB-D04, DB-D05
- `DataClass`, `DataClassPolicy`, `ClassedWriter`, `Validated<T>`-Anbindung; alte Schreibmethoden prüfen die Policy; K-Validator vor jedem Storage-Zugriff; `SandboxBridge::db_insert` ohne Umdeutung.
- Compile-Fail-Tests (`trybuild`) für G-ER1 und G-PII1-Mustertests mit synthetischen Canaries (Namen, Anschriften, Kennungen, E-Mail, Telefon).
- **Abnahme:** G-ER1, G-PII1; Test „fehlende Klasse bricht ab, nichts geschrieben" (Hash des Datenverzeichnisses vorher/nachher gleich).
- **Rollback:** B revertieren; Policy-Default bleibt `Legacy`.

**DB-S22 Lesesperre (Z-E1)** · XL · Risiko H · Voraussetzung: V2, V2a, V4, DB-D03, DB-S21
- `ReadGuard`, `GuardTarget`, `GuardPolicy`; Injektion; Prüfpunkte laut §6.4; Auffüllen auf `k`; Fehlerbild-Gleichheit; `Collection::len`/`stats` ohne Leck.
- **Abnahme:** Je Lesepfad ein Test (siehe §8, Tabelle Z-E1); `loom`-Linearisierungstest für „Sperre sichtbar sofort nach Rückkehr von L1" (`tests/loom_read_gate.rs`, im Pfadfilter von `loom-gate.yml`); Leistungsmessung: Gate-Prüfung O(1) ohne I/O (Mikrobenchmark, Gate G-C1 unberührt).
- **Hinweis:** `loom` blendet in `lib.rs` fast alles aus (`cfg(not(loom))`, 62 Stellen); der Test braucht ein eigenes, kleines Modell des Gate-Pfads.

**DB-S23 Spike Z-E8** · M · Risiko H · Voraussetzung: DB-S22 (Teilziel Kandidatenfilter)
- Messung: Wie ändert ein unzugängliches Dokument Rang und Scores (BM25-IDF, HNSW-Pool, RRF)? Kosten der drei Optionen aus DB-D06; Bericht mit Zahlen; Empfehlung an den Entscheider.
- **Abnahme:** Bericht unter `docs/decoupling/`; Test `z_e8_rank_unchanged` existiert und ist markiert, bis DB-D06 entschieden ist.

**DB-S24 Export und Import** · M · Risiko M · Voraussetzung: DB-S22
- `export.rs`: Gate je Eintrag, Klassenfilter (P nur auf ausdrückliche Anforderung mit Berechtigung und Zweck); gesperrte Einträge im Bericht nur gezählt. `import.rs`: Klassendeklaration beim Import; K-Validator; Schema bleibt `1.0` (Formatgleichheit INV-1), neue Felder nur additiv und optional.
- **Abnahme:** Round-Trip-Test unverändert grün; neuer Test „Export enthält keine gesperrten Ziele".

**DB-S25 Versiegelung (Z-E2, Z-E9)** · L · Risiko H · Voraussetzung: V2, DB-S20
- `DocumentSealer` und Adapter über `erasure::kv_shredding`; `SecretString` für Passphrase; `zeroize` wird benutzt; Schlüsselerzeugung über fallible Zufallsquelle, Fehler ergibt weder Umschlag noch Ersatzschlüssel; AAD `(tenant, collection, doc/PersonalRef, version)` (schließt Befund N8 \[R\] auf der `db`-Seite).
- **Abnahme:** Canary-Test: Byte-Suche über das gesamte Datenverzeichnis nach Insert, Flush, Compaction findet keinen Klartext der P-Payloads (Positivkontrolle eingeschlossen); Fehlerinjektion Z-E9 (fehlender Schlüssel ergibt Integritätsfehler, nie Klartext oder `Erased`-Verwechslung, §11.4.4).

#### Phase 3 — Löschkette

**DB-S30 Lineage** · L · Risiko H · Voraussetzung: DB-S21
- `LineageIndex`, Zuordnung aus den Schreibpfaden (HNSW, Text, Graph, KV-Cache, Konsolidierung, Chunk-Cache, Sitzungen).
- **Abnahme:** Eigenschaftstest: Für jede geschriebene Version liefert `derived_of` genau die Menge der erzeugten Ableitungen (Vergleich mit Orakel über Datenverzeichnis-Diff).

**DB-S31 L2: `DbPurgeTarget` und Drop-Pfad** · XL · Risiko H · Voraussetzung: V2, DB-S30, DB-S25
- Implementierung gemäß §6.6; `drop_collection` unter Feature `erasure` über die Saga; Epochenrollover für gleichnamige Neuanlage.
- **Abnahme:** `T-DEL-2` (nach Löschung, Flush, Compaction kein Canary, Lesepfad `Erased`), `T-DEL-4` (nach Neustart bleibt `Erased`), `T-DEL-6` (weder Volltext- noch Vektorsuche finden das Dokument), `T-DEL-8` (Drop widerruft KEK); `T-DEL-1` bleibt rot, bis V5 geklärt ist, und wird nicht umgedeutet.

**DB-S32 Crash-Matrix (db-Anteil)** · L · Risiko H · Voraussetzung: DB-S31, `testkit::FaultVfs`
- Szenarien an jeder `db`-Schreibstelle der Löschteilnahme: Intent-Marker, LSM-Commit, HNSW, Text, Graph, KV-Cache, Manifest. Invarianten I1 bis I6 (SPEC §14.2).
- **Abnahme:** Matrix grün; Testliste je Zeile aus §11.4 (G-ER2 für `db`).

**DB-S33 L5: `DbScanLayer`** · M · Risiko M · Voraussetzung: DB-S30
- Scanner-Schichten Text, Vektor, Graph, Cache mit Positivkontrolle; `NotImplemented` für Lücken.
- **Abnahme:** Test mit absichtlich verstümmeltem Scanner scheitert (Z-E10); `T-DEL-7` (Positivkontrolle findet eingebettetes Muster).

**DB-S34 Restore-Schutz am Öffnen (§11.4.5, D-32)** · S · Risiko M · Voraussetzung: V2
- `Contextra::open*` fragt `erasure::restore::guard` ab; ohne unabhängiges Register oder außerhalb des Sicherungshorizonts: Zustand `Quarantine`, keine Datenlieferung.
- **Abnahme:** Z-E11 Restore-Tests mit und ohne Senke, außerhalb des Horizonts.

#### Phase 4 — Zeitreise

**DB-S40 Typen und API** · M · Risiko M · Voraussetzung: V3, DB-D07, DB-D08
- `temporal/{window,query}.rs`; `ValidityWindow` aus `rules`; neue Builder-Methoden; Altpfade `#[deprecated]`; `legacy_validity_unit` je Collection (additive Konfiguration, kein Formatbruch).
- **Abnahme:** `tests/integration_temporal_filter.rs` unverändert grün; neue Tests für getrennte Achsen.

**DB-S41 Vorfilter und Auffüllen** · M · Risiko M · Voraussetzung: DB-S40, DB-S22 (Kandidatenebene)
- Gültigkeit vor Fusion/Rerank; Auffüllen auf `k`; Verhalten bei Altmetadaten laut DB-D07.
- **Abnahme:** `tests/truncation_filter_recall_test.rs` und `tests/search_result_bound.rs` grün; neuer Test „weniger als `k` nur, wenn der Korpus nicht mehr hergibt".

**DB-S42 Golden-Korpus Zeitauswahl (G-TT1, db-Anteil)** · M · Risiko M · Voraussetzung: DB-S40, S-02
- `tests/golden/temporal/` mit `MANIFEST.sha256`: Intervallgrenzen (`valid_from` inklusive, `valid_until` exklusiv), offene Enden, Schaltjahre, `i32`-Randwerte, Tage vor 1970; erzeugt auf x86_64 und aarch64, Hash-Vergleich in CI (DB-D08: gilt für die Zeitauswahl, nicht für ANN-Rangfolge).
- **Abnahme:** G-F1, G-TT1.

**DB-S43 Zusagen Z-T2 und Z-T5** · S · Risiko N · Voraussetzung: DB-S40, DB-S22
- Eigenschaftstest über zufällige Schreibfolgen (spätere Commits ändern frühere Antworten nicht); Test mit älterem Stand und erweiterter Richtlinie (Guard liest nur aktuelle Rechte).

#### Phase 5 — Simulation

**DB-S50 `SimStorage`, `ReadOnlyCollection`, `SimSession`** · L · Risiko H · Voraussetzung: V3 (S-107), DB-S22
- Gemäß §6.8; Namensraum `__sim:<id>:`; temporärer Index; `retire`.

**DB-S51 Isolationsbeweis (G-SIM1, Z-S1)** · M · Risiko M · Voraussetzung: DB-S50
- `proptest` über Operationsfolgen (Branch, Commit, Lauf, Vergleich, `retire`, Abbruch): Hash über `main`-Schlüsselraum, P-Register/Gate-Register und Policy vorher gleich nachher. Negativtest: Eine absichtlich eingebaute Schreibstelle muss den Test brechen.

**DB-S52 Ergebnisse und Hinweis (Z-S2, Z-S3)** · S · Risiko N · Voraussetzung: DB-S50
- `SimRun` mit Pflichtfeld „Entscheidungshilfe"; Vorlagentest; Ergebnisse mit Personenbezug versiegelt und mit Ablauf; Golden für Reproduzierbarkeit (Z-S2) im Rahmen von DB-D08.

#### Phase 6 — Abschluss

**DB-S60 S-70: Aufräumen** · M · Risiko M · Voraussetzung: alle Schritte
- `#[deprecated]`-Re-Exports entfernen; `*-compat`/`experimental-*`-Aliase entfernen; `DataClassPolicy::Enforced` als Default (ADR); `contextra-engine` entfernen oder weiter als Fassade führen (Entscheidung im ADR zu DB-D01).
- **Abnahme:** alle Gates; G-A1 zeigt nur beabsichtigte Diffs.

**DB-S61 Messbericht (S-111, db-Anteil)** · M · Risiko N
- Leistungsziele aus §14.5: L1-Rückkehr einer Sperre bei 1.000 Versionen (p99 < 50 ms, Grenze 2 s), Stufe „Bereinigt" für 1.000 Dokumente, Zeitreiseabfrage auf 1 Mio. Blobs (p95 < 200 ms, hier: Dokumentversionen); Werte nur im generierten Bericht (D-15).

**DB-S62 Entnehmbarkeits- und Isolationsnachweis** · S · Risiko N
- `docs/decoupling/REPORT.md` (HEAD-Hash, Datum), `capabilities.fragment.toml` mit `level`, `sublevel`, `core_deps`; `tools/isolation_proof.py --crate contextra-db --modes no-default,default,each-feature`.

**DB-S63 Rückblick und Restbefunde** · S · Risiko N
- Offene Befunde mit Frist; SPEC-Anhang A um `DB-F…` ergänzen.

### 7.5 Abhängigkeitsgraph (verkürzt)

```
DB-S00 ─► DB-S01 ─► DB-S15 ──────────────────────────┐
   │        └──► DB-S02 ─► DB-S03 (ADR-Paket)         │
   ├─► DB-S10, S11, S12, S13, S14, S17 ──► DB-S18 ────┤
   │                                                   ▼
   │     V1,V3a,V4 ─► DB-S20 ─► DB-S21 ─► DB-S22 ─► DB-S24
   │     V2,V2a ───────────────────────────┘   │
   │                                 DB-S23 ◄───┤
   │                         DB-S25 ◄─ V2, S20  │
   │                              └─► DB-S30 ─► DB-S31 ─► DB-S32 ─► DB-S33, S34
   │     V3 ─► DB-S40 ─► DB-S41 ─► DB-S42, S43
   │                  └────────────────────► DB-S50 ─► DB-S51, S52
   └──────────────────────────────────────────────────────────► DB-S60 ─► S61, S62, S63
```

### 7.6 Zeitplan (Wellen, \[E\])

| Welle | Inhalt | Dauer bei 2 Personen |
| --- | --- | --- |
| 0 | DB-S00 bis DB-S03 | 1 Woche |
| 1 | DB-S10 bis DB-S18 | 2 Wochen |
| 2 | DB-S20 bis DB-S25 (ohne Wartezeit auf V1 bis V4) | 4 bis 5 Wochen |
| 3 | DB-S30 bis DB-S34 | 4 Wochen |
| 4 (parallel zu 2/3) | DB-S40 bis DB-S43 | 2 Wochen |
| 5 | DB-S50 bis DB-S52 | 2 Wochen |
| 6 | DB-S60 bis DB-S63 | 1 bis 2 Wochen |

Die Wartezeit auf Vorbedingungen in `erasure`, `rules`, `ids`, `privacy`, `types` ist nicht eingerechnet und liegt auf dem kritischen Pfad.

---

## 8 Tests und Gates

### 8.1 Zuordnung der Zusagen zu `db`-Tests

Testdateien liegen in `crates/contextra-db/tests/` (Namen sind Vorschläge \[D\]). Jede Zusage hat mindestens einen automatisierten Test (Gate G-ER3).

| Zusage | Test (Datei :: Name) | Kern der Prüfung |
| --- | --- | --- |
| Z-E1 | `zusagen_ze1.rs :: read_paths_*` (ein Test je Zeile der Tabelle §6.4); `loom_read_gate.rs` | Nach L1 liefert kein Lesepfad Klartext, auch nicht `Superseded`/ältere Versionen; Schreibzugriffe verweigert; Linearisierung |
| Z-E2 | `zusagen_ze2.rs :: canary_bytes_*`, `metadata_no_pseudonyms` | Byte-Suche über alle Dateien mit Canary-Kennungen (nach Insert, Flush, Compaction, Löschung) mit Positivkontrolle; keine deterministischen Pseudonyme und keine exakten Längen in Metadaten |
| Z-E3 | in `erasure` (Trockenlauf); `db`: Test, dass `db` bei Konflikt keine Löschung auslöst | kein `DeleteAll` ohne Review |
| Z-E4 | `crash_matrix_db.rs` | I1 bis I6 an jeder `db`-Schreibstelle |
| Z-E5 | `class_k_validation.rs`, `compile_fail/*.rs` (`trybuild`) | K-Schreiben mit Personenbezug scheitert; `put_k` ohne `Validated` kompiliert nicht |
| Z-E6 | `zusagen_ze6.rs :: erase_leaves_ids_unchanged` | Objekt-IDs, Commit-Hashes, K-Ledger vorher/nachher identisch |
| Z-E7 | `zusagen_ze7.rs :: no_raw_identifiers_in_outputs` | Canary-Suche in Logs (`tracing`-Capture), Metriken, Fehlertexten |
| Z-E8 | `z_e8_rank_unchanged.rs` | Rangtest mit unzugänglichem Dokument; Status nach DB-D06 |
| Z-E9 | `fail_closed.rs` | Fehlerinjektion in Schlüssel-, Zufalls-, Siegelpfad: kein Ersatzwert |
| Z-E10 | in `erasure`/`DbScanLayer`: `scanner_positive_control.rs` | verstümmelter Scanner scheitert |
| Z-E11 | `restore_guard.rs` | mit/ohne Senke, außerhalb Horizont: `Quarantine` |
| Z-E12 | außerhalb `db` (`T-VENDOR-BUNDLE`) | `db` sendet keine Telemetrie; `grep`-Gate auf Netzwerk-Crates in `db`-Closure |
| Z-T1 | `golden_temporal.rs` | Zeitauswahl byte-identisch auf x86_64/aarch64 (DB-D08) |
| Z-T2 | `time_travel_history.rs` (`proptest`) | spätere Schreibfolgen ändern frühere `as_of`-Antworten nicht |
| Z-T3 | in `rules` | Takedown erhält IDs/Kette; `db`-Test: Lesepfad liefert nach Takedown nichts |
| Z-T5 | `policy_current_only.rs` | älterer Stand + erweiterte Richtlinie erweitert keine Rechte |
| Z-S1 | `sim_isolation.rs` (`proptest`) | `main`, P, Rechte vorher gleich nachher |
| Z-S2 | `sim_golden.rs` | reproduzierbar im Rahmen von DB-D08 |
| Z-S3 | `sim_template.rs` | Hinweis „Entscheidungshilfe" in jedem Ergebnis |
| T-DEL-1…8 | `deletion_*` (`docs/02_loeschung.md` §2.8; Namen müssen den Kennungen entsprechen) | siehe DB-S31, DB-S33 |

### 8.2 Bestehende Tests, die erhalten bleiben (Regressionsschutz)

`atomic_commit`, `fault_injection_2pc`, `split_brain_vector_orphan_test`, `sprint2_acid_tests`, `transaction_isolation`, `snapshot_api`, `snapshot_recovery`, `hnsw_delete_and_backfill`, `graph_entity_delete_test`, `supersedes_cascading_tombstone_test`, `truncation_filter_recall_test`, `search_result_bound`, `integration_temporal_filter`, `proptest_search_invariants`, `loom_relate_n_ary`, `no_blanket_allow_deprecated`, `signal_kind_no_new_variant`, `golden_vault`. Testanzahl vorher und nachher gleich, abgesehen von bewusst verschobenen Tests (Kriterium aus S-12b).

### 8.3 Gates je Phase

| Phase | Pflichtgates | Zusätzlich |
| --- | --- | --- |
| 0 | INV-4 (G-L3), G-DOC1 | Baseline-Bericht |
| 1 | G-L1 (normal, dev, Feature, Zyklen), G-L3 (`--each-feature`, `--no-default-features`), G-H1, G-A1, G-D1 (Regler, Chunker) | Score/Brückenzählung neu |
| 2 | G-ER1, G-ER3 (db-Anteil), G-PII1, G-C1 (Loom Gate-Pfad) | `unwrap-gate` (INV-12) auf neuem Code |
| 3 | G-ER2 (db-Anteil), G-F1 (keine Formatänderung), G-C1 | Crash-Matrix |
| 4 | G-TT1, G-F1 | Golden je Architektur |
| 5 | G-SIM1 | Negativtest zum Gate |
| 6 | alle | `tools/isolation_proof.py`, `cargo package`-Probe (SOLL §8.4 Punkt 6) |

Neue Gates werden **nicht** in `ratchet.toml › not_run_baseline` aufgenommen (D-04, D-23). Jeder Gate-Schritt in CI ohne `|| true` und ohne `continue-on-error`.

### 8.4 Reproduzierbare Prüfbefehle (Auswahl)

```bash
# Baseline und Kanten
cargo check -p contextra-db --no-default-features --locked
cargo hack check -p contextra-db --each-feature --no-dev-deps --locked
cargo tree -p contextra-db -e normal,dev --no-default-features --locked
python3 tools/gate_layering.py --all && python3 tools/isolation_proof.py --crate contextra-db --modes no-default,default,each-feature
# Dubletten und Reste
grep -rn 'volatile_vault' crates --include=*.rs --exclude-dir=contextra-vault
grep -rEn 'contextra_engine' crates/contextra-db/src
grep -rn 'include!' crates/contextra-db/src
# Zeitreise-Altpfade
grep -rn 'apply_temporal_validity_filter' crates --include=*.rs
```

---

## 9 Risiken und Rückfall

| ID | Risiko | Wahrsch. | Wirkung | Gegenmaßnahme / Rückfall |
| --- | --- | --- | --- | --- |
| R-1 | Z-E8 lässt sich mit gemeinsamer BM25-Statistik nicht erfüllen | hoch | Zusage nicht prüfbar | DB-S23 früh; DB-D06 entscheiden; sonst SPEC-Änderung statt stiller Abschwächung (INV-10) |
| R-2 | Gate-Prüfung ist nicht O(1) (`ErasedSet::check` linear) | mittel | Verfehlen §11.4.2 und §14.5 | V2a vor DB-S22; sonst Zusage mit Messwert statt „O(1)" |
| R-3 | Gate an allen Lesepfaden verändert Latenz und Rangfolge (Auffüllen auf `k`) | mittel | Regressionen in Such-Tests | Differenztests (G-D1) vor und nach; Auffüllen deterministisch (Tie-Break nach ID) |
| R-4 | R-CLS bricht Aufrufer (`contextra`, `agent`, `mcp`, `py`) | mittel | große Umstellung | `DataClassPolicy::Legacy` als Übergang (DB-D04); Verbraucherliste per `grep` vor DB-S21 |
| R-5 | Versiegelung verlangt Änderungen in `text`, `vector`, `store` | hoch | Z-E2 für D-Klasse nicht allein durch `db` erfüllbar | V6 explizit planen; Zusage in `db` bis dahin nur für Primärdaten |
| R-6 | Regler-Typen nicht bitgleich zu `adapt` | mittel | S-60 scheitert | DB-D09: Wrapper mit alter Signatur; Abweichung als Befund, Toleranz nicht anpassen (§9.5 Punkt 3) |
| R-7 | Cognition-Inversion stößt auf zyklische Typen (`CommitIntent`) | mittel | DB-S15 verzögert | Rückfall: eigener Typ in `cognition`, Abbildung in `db` (bereits teilweise vorhanden) |
| R-8 | Tests bleiben rot aus Gründen außerhalb `db` (`store` 19, `vector` 22 Targets \[R\]) | hoch | INV-4 nicht grün | Reihenfolge mit den Besitzern abstimmen; `db`-Schritte, die den Gesamt-Build brauchen, warten |
| R-9 | Fehlende Governance-Dateien im Klon (DB-F01) | hoch | Gates nicht ausführbar | DB-S00 klärt Quelle; ohne Governance keine Merge-Freigabe |
| R-10 | Zeitreise-Einheit der Altdaten unbekannt (DB-D07) | mittel | falsche Filterung nach Migration | Keine Heuristik; Eigentümerentscheidung; Abfragefehler statt Raten |
| R-11 | Crash-Matrix deckt `db`-Teile nicht vollständig ab | mittel | Z-E4 nur teilweise | G-ER2 verlangt je Zeile einen Test; fehlende Zeile ist rot |
| R-12 | Rechtliche Einordnung (§11.6) wird in Code und Doku als Aussage missverstanden | niedrig | INV-7 verletzt | G-DOC1 Wortfilter; Berichte nennen Stufen mit Messergebnis |

**Rückfallverfahren** (SPEC §9.5) gelten unverändert: Gate nach Commit B rot und nicht innerhalb eines Arbeitstags behebbar → B revertieren; Differenztest rot → Schritt abbrechen, Toleranz nicht anpassen; Format-Hash weicht ab → Abbruch.

---

## 10 Offene Punkte und Annahmen

1. Ob der Klon `24487fa` dem Stand entspricht, den die SPEC und die Berichte meinen, ist unklar (anderer Hash, „Shell-Commit", fehlende Governance-Dateien). Alle Aussagen zu CI, `ratchet.toml`, `gates.toml` und `docs/decisions/` sind deshalb nicht prüfbar.
2. Die Kompilierfehler stammen aus `reports/` (HEAD `5137db4ab`); am Klon wurde nicht gebaut. Die Liste der betroffenen Test-Targets ist in DB-S00 neu zu erheben.
3. Die Einheit der Altgültigkeitsfelder (`valid_from`, `valid_until`) und die Semantik von `as_of(u64)` in Bestandsdaten und Aufrufern (`contextra`, `mcp`, `agent`) sind nicht belegt \[E\].
4. Ob `HashSet`-Nutzung in `multistep.rs` die Ausgabereihenfolge beeinflusst, wurde nur am Codeausschnitt beurteilt (Reihenfolge folgt dem Vektor) \[E\].
5. Die Abhängigkeit `contextra-vault` ist in der SPEC (§4.1 Anmerkung) als Kernabhängigkeit begründet („db re-exportiert den Typ"); ob `exclude` im Wurzelmanifest noch besteht, ist wegen des fehlenden Manifests unbekannt (D-14, F-11).
6. Dubletten in `store` und `vector` (F-15) beeinflussen die Closure-Aussagen für `db` nur indirekt; sie sind nicht Teil dieses Plans.
7. Die Aufwandsangaben sind Einschätzungen ohne Build- und Messbasis.
8. Rechtsfragen (§11.6 der SPEC) sind nicht Gegenstand dieses Dokuments; es enthält keine Rechtsberatung.

---

## Anhang A — Migrationskarte (Datei → Ziel)

| Heute | Ziel | Schritt |
| --- | --- | --- |
| `src/volatile_vault.rs` | bleibt als Re-Export bis S-70 | DB-S10, DB-S60 |
| `src/vault_sys.inc`, `src/vault_compat.inc` | löschen | DB-S10 |
| `src/chunker/*` | `ingest::chunking::prose` (Re-Export mit Wrapper) | DB-S14 |
| `src/decay_controller.rs`, `src/homeostat.rs` | `adapt` (Wrapper, `#[deprecated]`) | DB-S13 |
| `src/pid_latency_controller.rs` | Entscheidung DB-D09 | DB-S13 |
| `src/temporal_filter.rs` | `src/temporal/*`; Alt-Pfad Re-Export, `#[deprecated]` | DB-S40, DB-S41 |
| `src/cognition/bridge.rs` | Implementierung der `cognition`-Ports | DB-S15 |
| `src/license_seam.rs` | `contextra-license` | DB-S16 |
| `src/lib.rs` (Konfiguration, `SandboxBridge`) | `config.rs`, `sandbox/` | DB-S17 |
| `src/export.rs`, `src/import.rs` | Gate und Klassen | DB-S24 |
| `collection/crud/delete.rs`, `collection/maintenance.rs::drop_collection` | Teilnahme an der Saga | DB-S31 |
| neu | `src/guard/*`, `src/erase/*`, `src/sim/*`, `collection/{view,classed}.rs` | Phasen 2 bis 5 |
| `crates/contextra-engine` | `#[deprecated]`-Fassade, später entfernen | DB-S60 |

## Anhang B — Befundregister (`DB-F`, Auszug der Zuordnung)

| Befund | Schritt(e) | Status |
| --- | --- | --- |
| DB-F01 | DB-S00 | offen |
| DB-F02 | DB-D01, DB-S03 | offen |
| DB-F03 | DB-S02 | offen |
| DB-F04 | DB-S01, DB-S15 | offen |
| DB-F05 | DB-S12, DB-S15 | offen |
| DB-F06 | DB-S15 | offen |
| DB-F07 | DB-S10 | offen |
| DB-F08 | DB-S13 | offen |
| DB-F09 | DB-S14 | offen |
| DB-F10 | DB-S40, DB-S41, DB-S42 | offen |
| DB-F11 | DB-S30, DB-S31, DB-S32, DB-S33 | offen |
| DB-F12 | DB-S22 | offen |
| DB-F13 | DB-S20, DB-S21 | offen |
| DB-F14 | DB-S20 | offen |
| DB-F15 | DB-S25 | offen |
| DB-F16 | DB-S24 | offen |
| DB-F17 | DB-S11 | offen |
| DB-F18 | DB-S11 | offen |
| DB-F19 | DB-S17 | offen |
| DB-F20 | DB-S21 | offen |
| DB-F21 | V2, V2a, V4, V5 | offen (fremde Crates) |
| DB-F22 | DB-S17 | offen |
| DB-F23 | DB-S13 | offen |

## Anhang C — Verbraucher von `contextra-db` (Auswirkungsprüfung vor API-Änderungen)

| Verbraucher | Art | Features | Relevanz |
| --- | --- | --- | --- |
| `contextra` | normal | — | Fassade; `collection_profile` (Z-E-Profile, DB-D03) |
| `contextra-agent` | normal | `volatile-vault` | nutzt Vault-Typen; Feld `reset_receipt`/`purge_receipt` |
| `contextra-bench` (`benchmarks/`) | normal | — | Benchmarks, E3 |
| `contextra-engine` | normal | leitet 11 Features weiter | veraltete Fassade; Features müssen bis S-70 erhalten bleiben |
| `contextra-cognition` | dev | — | Zyklus (DB-F05), nach DB-S15 entfällt |
| `contextra-mcp`, `contextra-py` | indirekt über `contextra` | — | Schreib-/Suchpfade (R-CLS-Umstellung, DB-S21) |

Vor jeder öffentlichen Änderung: `cargo public-api -p contextra-db` und `grep` über diese Verbraucher; Entfernen nur in S-70 (INV-2).

## Anhang D — Definition of Done je Schritt (Checkliste)

1. Commit-Paar `DB-Sxx A/B` (INV-3). 2. `cargo check --workspace --all-targets --locked` grün (INV-4). 3. Gates der Phase grün (§8.3). 4. Score und Brückenzählung neu erzeugt, kein Crate schlechter als vorher. 5. `crates/contextra-db/SPEC.md`/`PLAN.md` und Bericht aktualisiert. 6. Keine neue Verletzung von R10 bis R14, keine neuen `unwrap`/`expect`/`panic!` (INV-12), kein neues `unsafe` (INV-11). 7. Revert-Commit baut. 8. Verweise in Doku lösen auf (G-DOC1); keine Compliance-Aussagen (INV-7). 9. Bei Persistenzbezug: Golden-Korpus unverändert (INV-1), sonst Ausnahme dokumentiert.
