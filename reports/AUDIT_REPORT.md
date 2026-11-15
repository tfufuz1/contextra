# Contextra – Audit-Report (Messstand 15.11.2026)

## 1. Umfang und Methodik

Geprüft wurde das Repository `tfufuz1/contextra` (Commit des Klons, Shallow-Clone mit 50 Commits). Toolchain: Rust 1.89.0 (rustc, cargo, clippy, rustfmt), bezogen über micromamba/conda-forge, da `static.rust-lang.org` nicht erreichbar war. Hardware: 1 vCPU, 3,9 GB RAM, Ubuntu 24.04.

Drei Messarten wurden verwendet. Erstens eine statische Inventur (Zeilen, `unsafe`, `unwrap`/`expect`, `panic!`, Testattribute). Zweitens `cargo check --all-targets --workspace --keep-going` für eine vollständige Kompilierliste. Drittens `cargo test --locked -p <crate> --no-fail-fast` für alle 28 Default-Member, mit 25 Minuten Zeitlimit pro Crate. Zusätzlich wurden das Benchmark-Harness und ein verkleinerter Skalierungslauf ausgeführt.

Eine wichtige Einschränkung gilt für alle Testzahlen: Cargo führt bei einem Kompilierfehler in einem Target keine Tests des gesamten Crates aus. Crates mit solchen Fehlern zeigen daher 0 Tests, was nicht „keine Tests vorhanden“ bedeutet, sondern „nicht ausgeführt“.

## 2. Kompilierstatus

Alle 39 Workspace-Mitglieder kompilieren als Library. Die Fehler liegen ausschließlich in Test-, Bench- und Example-Targets sowie in einem Dead-Code-Lint im Lib-Test-Target von `contextra-graph`.

Sechs Crates sind betroffen. Die Anzahl der Fehler beträgt jeweils (Summe über alle Targets aus dem Workspace-Check, 294 Fehler gesamt):

| Crate | Fehlerhafte Targets | Befund |
|---|---|---|
| contextra-erasure | 2 Proof-Tests (78 Fehler) | Methoden wie `current_phase`, `advance_to_fence`, `advance_to_wal` und Enum-Varianten wie `PurgePhase::Completed` existieren nicht. Die Tests sind gegen eine ältere API geschrieben. |
| contextra-db | 10 Test-Targets, 1 Example (ca. 40 Fehler) | Fehlendes Modul `memory_consolidation`, Typfehler in `fault_injection_2pc`. |
| contextra-store | 19 Test-Targets (ca. 60 Fehler) | Chaos- und Compaction-Tests, u. a. `chaos_bitflip_sstable` (13 Fehler). |
| contextra-vector | über 25 Test- und Bench-Targets (ca. 100 Fehler) | `GenericHnswIndex` hat keine Methoden `insert`, `search`, `commit`, die die Tests erwarten. Zusätzlich brechen Benchmarks wie `overfetch_selectivity`. |
| contextra-graph | Lib-Test (1 Fehler, 8 Warnungen), dazu Test und Bench | `#![deny(dead_code)]` in `lib.rs` bricht den Build, weil Felder `data` und `last_tx` nie gelesen werden. |
| contextra-agent | Lib-Test (1 Fehler) | Feld `purge_receipt` auf `ScratchpadCheckpoint` fehlt. Das Struct hat `reset_receipt`. Vermutlich unvollständige Umbenennung. |
| contextra-checkpoint | Test `checkpoint_clock_determinism` (35 Fehler) | `InMemoryStorageEngine` implementiert `SnapshotSource` nicht, die Methoden `create_checkpoint`, `with_clock`, `commit` sind dadurch nicht verfügbar. |
| contextra-text | Test `lib_scorer_lifecycle_test` (5 Fehler) | Die Dev-Abhängigkeiten `contextra_ports` und `contextra_types` sind nur über das Feature `types-compat` aktiv. Der Test ist nicht feature-gated. |

## 3. Testergebnisse

Die Tabelle zeigt die ausgewerteten Zahlen aus `cargo test --no-fail-fast`. Die Crates mit Kompilierfehlern sind mit „nicht ausgeführt“ markiert.

| Crate | Bestanden | Fehlgeschlagen | Ignoriert | Status |
|---|---|---|---|---|
| contextra-ids | 17 | 0 | 0 | grün |
| contextra-env | 19 | 0 | 0 | grün |
| contextra-durable-fs | 43 | 0 | 0 | grün |
| contextra-wire | 111 | 0 | 0 | grün |
| contextra-sys | 48 | 0 | 0 | grün |
| contextra-simd | 98 | 0 | 0 | grün |
| contextra-types | 187 | 0 | 1 | grün |
| contextra-crypto | 234 | 0 | 1 | grün |
| contextra-ports | 36 | 0 | 1 | grün |
| contextra-mvcc | 127 | 0 | 1 | grün |
| contextra-testkit | 88 | 0 | 2 | grün |
| contextra-adapt | 68 | 0 | 0 | grün |
| contextra-kvcache | 44 | 0 | 0 | grün |
| contextra-rank | 121 | 0 | 1 | grün |
| contextra-router | 114 | 0 | 1 | grün |
| contextra-infer-onnx | 4 | 0 | 0 | grün |
| contextra-infer-ollama | 25 | 0 | 2 | grün |
| contextra-cognition | 108 | 1 | 0 | rot |
| contextra-engine | 110 | 1 | 0 | rot |
| contextra-mcp | 129 | 4 | 0 | rot |
| contextra-store | 284 | 16 | 0 | rot, dazu Timeout im Stresstest |
| contextra | 49 | 2 | 2 | rot |
| contextra-vector | nicht ausgeführt | – | – | Kompilierfehler |
| contextra-db | nicht ausgeführt | – | – | Kompilierfehler |
| contextra-graph | nicht ausgeführt | – | – | Kompilierfehler |
| contextra-agent | nicht ausgeführt | – | – | Kompilierfehler |
| contextra-checkpoint | nicht ausgeführt | – | – | Kompilierfehler |
| contextra-text | nicht ausgeführt | – | – | Kompilierfehler |

Über die 22 Crates mit ausgeführten Tests ergeben sich 2 064 bestandene, 24 fehlgeschlagene und 12 ignorierte Tests. Die sechs Crates mit Kompilierfehlern (agent, checkpoint, db, graph, text, vector) gehen mit 0 Tests in diese Summe nicht ein.

Nicht Teil des Testlaufs ist `contextra-erasure`, weil es kein Default-Member ist. Sein Kompilierfehler stammt aus dem Workspace-Check (78 Fehler in zwei Proof-Tests).

### 3.1 Fehlgeschlagene Tests und Befunde

Der sicherheitsrelevanteste Befund ist der Test `test_tdel_1_canary_bytes_erased_after_compaction` in `crates/contextra/tests/deletion_e2e.rs`. Er schlägt mit `left: 1, right: 0` fehl. Nach Schreiben, Löschen und Flush werden Canary-Bytes in einer Datei des Datenverzeichnisses gefunden. Das Projekt beansprucht Löschbelegbarkeit (README, Abschnitt Sicherheit). Dieser Test belegt das Gegenteil, zumindest für den getesteten Pfad.

Weitere Befunde: Der Test `test_consolidation_engine_fault_injection_resume` in `contextra-cognition` erwartet nach einem Fehler und Wiederaufnahme 1 synthetisiertes Meta-Chunk, es entstehen aber 0. Das ist ein Wiederanlauf-Defekt in der Konsolidierung.

In `contextra-mcp` brechen vier Stdio-Tests, darunter `test_e2e_stdio_demo_flow`, `test_max_rpc_bytes_overflow_and_line_draining_stdio` und `test_slowloris_stdio_attack_simulation`. Damit sind die Grenzen für Nachrichtengröße und das Schutzverhalten gegen langsame Clients nicht durch Tests bestätigt.

In `contextra-engine` bricht `test_durability_after_wal_commit_and_simulated_crash`. Das ist der Test, der die Durabilität nach einem simulierten Absturz nach WAL-Commit prüft.

In `contextra-store` schlagen 16 Tests fehl. Beispiele sind `test_retain_versions_unpinned_floor_max`, `test_delete_memory_capacity_exceeded` und `test_compaction_backpressure_timeout_exceeded`. Außerdem blockiert der Stresstest `compaction_stress_and_gc` über das Zeitlimit hinaus.

In `contextra` schlagen zwei Tests fehl: `open_compliance_profile_succeeds_without_license` und die Löschprüfung oben.

### 3.2 Ungeklärte Punkte

Die Logs sind vollständig, aber die Ursache einzelner Fehlschläge ist nicht in jedem Fall geklärt. Für `contextra-store` und `contextra-mcp` wurden die Assertion-Meldungen nicht vollständig ausgelesen. Eine Ursachenanalyse für jeden der 16 Store-Fehler ist eine eigene Arbeit.

## 4. Statische Inventur

Die Zahlen unten sind grob, weil `unwrap`, `expect` und `panic!` auch in Testcode gezählt werden. Der Produktivcode ist damit nicht sauber getrennt.

Der größte Crate nach Zeilenzahl ist `contextra-store` mit ca. 58 900 Zeilen in 330 Dateien. Danach folgen `contextra-db` (ca. 34 900), `contextra-graph` (ca. 34 400), `contextra-vector` (ca. 33 800) und `contextra-erasure` (ca. 32 400). Die höchste Zahl an `unsafe`-Stellen hat `contextra-simd` mit 291 Vorkommen, gefolgt von `contextra-sys` (25) und `contextra-vector` (10). Bei `contextra-simd` ist das plausibel, da es sich um SIMD-Code handelt. Der Schutz gegen unbegründetes `unsafe` ist hier nicht gemessen worden.

Die Zahl der `unwrap()`-Aufrufe ist mit 1 540 in `contextra-store` am höchsten. Ein Großteil davon liegt vermutlich im Testcode. Die Zahl der Tests mit `#[test]` liegt bei `contextra-store` mit 651 Attributen am höchsten. Die Anzahl der `todo!`/`unimplemented!`-Makros ist in allen Crates 0, es gibt also keine offenen Platzhalter, die als Stub kompiliert würden. Die Kommentar-Marker TODO/FIXME/XXX/HACK sind mit 2 in `contextra-graph` und 5 in `contextra-cognition` sehr selten.

## 5. Benchmarks

### 5.1 Retrieval-Harness (`contextra-bench`)

Das Haupt-Binary `contextra-bench` lässt sich nicht bauen. Der Build verlangt das Feature `datasets`, das laut Kommentar in `Cargo.toml` die Dateien `src/locomo.rs` und `src/long_mem_eval.rs` benötigt. Diese Dateien fehlen im Repository. Der Build bricht mit `unresolved import crate::locomo` und `crate::long_mem_eval` ab.

Ohne dieses Binary erzeugt auch `compare-baseline` keine Ergebnisse. Das Programm liest `benchmarks/results/current_metrics.json`, die nie erzeugt wird. Die Qualitätsmessung (Recall, nDCG, MRR) ist daher in diesem Stand nicht reproduzierbar.

Die vorhandenen Ergebnisdateien in `benchmarks/results/` sind historisch. Die Vergleichsmatrix in `2026-11-09.md` vermerkt selbst, dass sie „nicht ausgeführt“ ist.

### 5.2 Skalierungslauf (`scale-100k`, verkleinert)

Gemessen wurde mit 2 000 Dokumenten, Dimension 128 und Schrittgröße 500, Release-Profil, 1 vCPU. Die Werte sind für diese VM gültig, nicht für Produktion.

| Dokumente | Insert (docs/s) | Suche p50 | Suche p95 | Suche p99 | RSS |
|---|---|---|---|---|---|
| 500 | 54,4 | 0,42 ms | 4,47 ms | 5,13 ms | 21 MB |
| 1 000 | 35,9 | 0,35 ms | 4,41 ms | 4,57 ms | 48 MB |
| 1 500 | 19,8 | 0,73 ms | 4,80 ms | 5,10 ms | 87 MB |
| 2 000 | 12,8 | 0,55 ms | 4,67 ms | 5,06 ms | 143 MB |

Die Insert-Rate fällt von 54 auf 13 Dokumente pro Sekunde, während der Speicher wächst. Die Suchlatenz p50 bleibt grob stabil, der p95-Wert liegt konstant bei etwa 4,5 bis 4,8 ms. Die Tail-Latenz ist also dominierend und gleichbleibend hoch, nicht zunehmend. Das stützt die README-Aussage zu konstanten Antwortzeiten nur eingeschränkt, da hier nur 2 000 Dokumente gemessen wurden und der Lauf nicht bis zu Produktionsgrößen reicht.

Ein früherer 100k-Lauf (siehe `benchmarks/results/summary.md`) brach im Repository selbst bei 0 verarbeiteten Intervallen wegen eines Speicherlimits von 2048 MB ab. Dieser Befund wurde nicht wiederholt, weil die Laufzeit auf einer vCPU den Rahmen sprengt.

### 5.3 Nicht gemessen

Die in der README beschriebenen Kennzahlen zu RAM-Reduktion (auf ein Viertel bzw. ein Zweiunddreißigstel) wurden nicht gemessen. Der zugehörige Test `ram_reduction` in `contextra-vector` kompiliert nicht. Ebenso nicht gemessen: Cross-Encoder-Reranking, Ingest-Durchsatz und Speichertrennung zwischen Mandanten.

## 6. Bewertung

Funktionierende Komponenten sind die Bausteine mit grünen Tests und stabilen Schnittstellen: Identifikatoren, Umgebungsparameter, durable Dateisystemoperationen, Wire-Format, Systemschnittstelle, SIMD-Hilfen, Typen, Kryptographie, Ports, MVCC und Testkit. Diese Crates haben zusammen über 1 000 bestandene Tests. Die Kryptographie-Crate mit 234 bestandenen Tests ist der größte grüne Block.

Unreife Logiklücken sind an den Stellen zu finden, an denen Tests gegen eine ältere API geschrieben sind, oder an denen Tests Verhalten prüfen, das der Code nicht zeigt. Das betrifft vor allem die Löschbelegbarkeit (T-DEL-1), die Konsolidierung nach Fehlern (cognition), die Durabilität nach Absturz (engine), die Stdio-Schutzmechanismen (mcp) und die Compaction-Pfade (store). Zusätzlich sind Erasure, Vector und DB zu großen Teilen durch Testcode blockiert, dessen Tests die aktuelle API nicht mehr treffen.

Der Gesamtstand ist daher: Die Library-Schicht kompiliert, aber die Verifikation der Kernpfade ist unvollständig und teils widerlegt.

## 7. Empfehlungen

Die dringendste Maßnahme ist die Behebung des T-DEL-1-Fehlers, weil er einen Löschanspruch betrifft. Danach sollten die sechs nicht kompilierenden Crates repariert werden, damit ihre Tests überhaupt laufen. Das `purge_receipt`-Problem in `contextra-agent` ist dafür ein kleiner Einstieg. Die fehlenden Dateien `locomo.rs` und `long_mem_eval.rs` müssen ergänzt oder das Feature `datasets` entfernt werden, damit die Qualitätsmessung wieder läuft. Der Speicherverbrauch im Skalierungslauf sollte bei größeren Mengen erneut gemessen werden, mit mehr Zeit und Speicher als hier verfügbar.

## 8. Rohdaten

Die Logs liegen unter `/home/claude/audit/logs/`, die Auswertung unter `/home/claude/audit/summary.tsv`, der Skalierungslauf unter `/home/claude/audit/scale_2k.json`.
