# Contextra – Audit-Verifikation und Produktionsreife (Stand 16.11.2026)

Ergänzt `reports/AUDIT_REPORT.md` (15.11.2026) und `reports/FIX_REPORT_2026-11-16.md`. Dieser Bericht prüft, welche Aussagen des bisherigen Audits am aktuellen HEAD noch gelten.

## 1. Umgebung und Methode

- Repository: `tfufuz1/contextra`, Branch `main`, HEAD `5137db4ab`. Arbeitsverzeichnis sauber, keine Änderungen am Repo durch diesen Audit.
- Toolchain: Rust 1.89.0 (cargo, rustc, clippy 0.1.89, rustfmt 1.8.0) aus conda-forge über micromamba, da `static.rust-lang.org` nicht erreichbar ist.
- Hardware: 1 vCPU, ca. 4 GB RAM, Ubuntu 24.04. Das Dateisystem war während der Läufe zeitweise voll.
- Abweichungen von den Projektvorgaben, die für die Ergebnisse relevant sind:
  - Test-Builds liefen mit `CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_PROFILE_DEV_DEBUG=0` und `CARGO_INCREMENTAL=0`. Das ändert nur Debug-Informationen und Inkrementell-Caching, nicht das Laufzeitverhalten. Grund: Speicherplatz.
  - Ausgeführt wurden `cargo check` über den gesamten Workspace und `cargo test --locked -p <crate>` für einzelne Crates. Der volle `just check`/`just test`-Lauf wurde nicht ausgeführt.

## 2. Kompilierstatus (verifiziert)

`cargo check --locked --all-targets --workspace --keep-going` ergibt 290 Fehlerzeilen. Der Audit nennt 294. Die Fehler verteilen sich so:

| Crate | Betroffene Targets | Befund |
|---|---|---|
| contextra-vector | 22 | Tests rufen `insert`, `commit`, `search_filtered`, `inspect_residual` auf `GenericHnswIndex` auf. Diese Methoden fehlen im Typ. |
| contextra-store | 19 | Chaos-, Compaction- und WAL-Tests, u. a. `chaos_bitflip_sstable` (13 Fehler), `compaction_merge_operator` (11 Fehler). |
| contextra-db | 10 Tests, 1 Example | Fehlendes Modul `contextra_db::memory_consolidation`, Typfehler in `fault_injection_2pc`. |
| contextra-graph | Lib-Test (3 Fehler, 282 Warnungen), `ppr_candidate_stream`, `csr_traversal_bench` | Typ- und Feature-Fehler. Der Lib-Test-Build bricht trotz des `deny(dead_code)`-Befunds aus dem Audit an anderen Stellen. |
| contextra-erasure | 2 Proof-Tests (78 Fehler) | Tests gegen eine ältere API: `current_phase`, `advance_to_fence`, `PurgePhase::Completed` existieren nicht. |
| contextra-text | `lib_scorer_lifecycle_test` (5 Fehler) | `contextra_text::compat` ist nicht verfügbar, weil das Modul hinter einem Feature liegt. `contextra_ports` und `contextra_types` sind nicht als Dev-Dependencies gesetzt. |
| contextra-checkpoint | `hardlink_cloner_tests` (2 Fehler) | `?` konvertiert `CheckpointError` nicht in `ContextraError`. Dieser Fehler stand nicht im Audit. |

Bereits im HEAD behoben oder nicht mehr betroffen:

- `contextra-agent` kompiliert im HEAD. Der Fehler `purge_receipt` aus dem Audit ist nicht mehr vorhanden.
- `contextra-text` und `contextra-checkpoint` kompilieren als Library. Die Test-Fehler sind oben genannt.

Der Fix-Report beschreibt Änderungen auf einem Branch `fix/test-targets`. Dieser Branch ist im Klon nicht vorhanden. Im Remote gibt es mehrere `audit-*`-Branches, aber keinen mit diesem Namen. Der Stand im HEAD ist deshalb nicht der Stand, den der Fix-Report beschreibt. Das muss geklärt werden.

## 3. Testergebnisse (verifiziert)

| Crate | Ergebnis | Anmerkung |
|---|---|---|
| contextra (Test `deletion_e2e`) | 1 bestanden, 1 fehlgeschlagen | T-DEL-1 schlägt reproduzierbar fehl, siehe Abschnitt 4. |
| contextra-agent (Lib) | 47 bestanden | Gilt auch für den Audit-Befund `purge_receipt`, der nicht mehr besteht. |
| contextra-agent (Integrationstests) | 22 Test-Targets vollständig grün | Ausnahmen: `persistence_test` (1 Fehler, DirLock, Zeile 120) und `session_concurrency_test` (siehe nächste Zeile). |
| contextra-agent (`session_concurrency_test`) | 2 fehlgeschlagen | `test_parallel_agent_sessions_isolation` und `test_parallel_tool_execution_failure_dlq_routing`. Im Audit nicht erfasst. |
| contextra-text | 0 ausgeführt | Kompilierfehler in einem Test-Target blockiert den ganzen Crate-Testlauf. |
| contextra-checkpoint | 0 ausgeführt | Kompilierfehler in `hardlink_cloner_tests` blockiert den ganzen Crate-Testlauf. |
| contextra-vector, contextra-db, contextra-store, contextra-graph, contextra-erasure | 0 ausgeführt | Kompilierfehler. |

Die Angaben oben beziehen sich auf die Läufe, die ich tatsächlich beobachtet habe. Ein Lauf mit `checkpoint` und `text` nach der Speicherbereinigung brach vor dem Testlauf ab, wie oben beschrieben.

Nicht verifiziert in dieser Runde: die Befunde des Audits zu `contextra-engine` (Durabilität nach WAL-Commit und Crash), `contextra-mcp` (4 Stdio-Tests), `contextra-cognition` (Konsolidierung nach Fehler) und `contextra-store` (16 fehlgeschlagene Tests). Sie sind aus dem Audit übernommen und nicht nachgeprüft.

## 4. T-DEL-1: Löschbelegbarkeit

Der Test `test_tdel_1_canary_bytes_erased_after_compaction` in `crates/contextra/tests/deletion_e2e.rs` schlägt reproduzierbar fehl:

```
assertion `left == right` failed: T-DEL-1: Canary-Bytes wurden im Datenverzeichnis (1 Datei(en)) nach Schreiben, Löschen und Flush gefunden
  left: 1
 right: 0
```

Befund aus einer temporären Diagnose (Test danach auf HEAD zurückgesetzt, Repo sauber):

- Nach Schreiben, Flush, Löschen und erneutem Flush liegen zwei SST-Dateien vor. `sst-…-000015` enthält den Canary (120 685 Bytes). `sst-…-000027` enthält ihn nicht (120 312 Bytes).
- Die alte Datei mit dem Canary bleibt im Datenverzeichnis. Der Flush erzeugt eine neue Datei ohne den gelöschten Datensatz, entfernt aber die alte nicht.
- Der Test heißt „nach Kompaktion“ und dokumentiert „Vollkompaktion“, ruft aber keine Kompaktion auf. Die öffentliche Fassade in `crates/contextra/src/lib.rs` bietet keine Compact-Funktion. `contextra-store` hat `compact_all` und `compact_range` auf der internen Ebene, sie sind über die Fassade nicht erreichbar.

Bewertung:

- Die Fassade hat keinen öffentlichen Pfad, der physisches Löschen erzwingt. Die Aussage in der README zur Löschbelegbarkeit ist deshalb für den öffentlichen API-Pfad nicht belegt.
- Der Test selbst ist unvollständig, weil er seine eigene Vorbedingung nicht erfüllt. Ob der Test korrigiert oder die fehlende Funktion ergänzt werden soll, ist eine Designentscheidung und wurde nicht getroffen.
- Ob die alte SST-Datei noch vom Manifest referenziert wird, habe ich nicht geprüft. Das ist der nächste Schritt für diesen Befund.

## 5. Weitere Befunde

- `persistence_test` (contextra-agent, Zeile 120) meldet `Datenverzeichnis bereits in Benutzung`. Der Fix-Report führt das auf den Lebenszyklus des `DirLock` zurück: `LsmStorage::drop` wartet nicht auf die Compaction-Task, die den Manifest-Lock hält. Der Befund ist plausibel, aber nicht im Code selbst nachgeprüft.
- `session_concurrency_test` in contextra-agent hat zwei Fehlschläge, die im Audit nicht stehen. Die Ursache ist nicht untersucht.
- `hardlink_cloner_tests` in contextra-checkpoint hat einen Konvertierungsfehler im Test. Das ist ein Test-Defekt, kein Produktfehler.

## 6. Benchmarks und Messwerte

- `contextra-bench` lässt sich nicht bauen: das Feature `datasets` verlangt `src/locomo.rs` und `src/long_mem_eval.rs`, die im Repo fehlen. Die Qualitätsmessung (Recall, nDCG, MRR) ist damit nicht reproduzierbar.
- Der verkleinerte Skalierungslauf (2 000 Dokumente, Dimension 128, 1 vCPU) aus dem Audit ist nicht wiederholt. Die Ergebnisse gelten nur für diese VM und diesen Korpus. Die Insert-Rate fällt von 54 auf 13 Dokumente pro Sekunde, während der RSS auf 151 MB steigt.

## 7. Produktionsreife

Einschätzung: Contextra ist derzeit nicht produktionsreif.

Gründe, die direkt belegt sind:

1. Sieben Crates haben Targets, die nicht kompilieren. Darunter sind Vector, Store und DB, also die Kernpfade für Speicherung und Suche. Ihre Tests laufen daher gar nicht.
2. Der Löschanspruch ist für den öffentlichen API-Pfad nicht belegt. T-DEL-1 schlägt fehl, und die Fassade hat keine Compact-Funktion.
3. Der Lock-Lebenszyklus (DirLock) verhindert ein Wiederöffnen des Datenverzeichnisses nach dem Schließen. Das betrifft Neustart und Wiederherstellung.
4. Mehrere Tests in agent und mcp schlagen fehl, darunter Parallelität und DLQ-Routing im Agent.

Was gut aussieht: Die Basisbausteine (ids, env, durable-fs, wire, sys, simd, types, crypto, ports, mvcc, testkit, adapt, kvcache, rank, router, infer-*) laufen mit grünen Tests laut Audit. Ein Nachweis in dieser Runde fehlt, die Zahlen stammen aus dem Audit.

## 8. Empfohlene Reihenfolge

1. Klären, warum der Fix-Stand (`fix/test-targets`) nicht im Klon liegt, und ob er gemergt werden soll.
2. T-DEL-1 zuerst: Entscheiden, ob eine öffentliche Compact-/Purge-Funktion nötig ist oder ob die README-Aussage angepasst wird. Danach den Test korrigieren.
3. DirLock-Lebenszyklus beheben, bevor Neustart-Tests ausgewertet werden.
4. Kompilierfehler in contextra-vector, contextra-store, contextra-db, contextra-graph und contextra-erasure beheben. Erst danach sind deren Tests aussagekräftig.
5. Feature-Gating für contextra-text und die Fehlerkonvertierung in contextra-checkpoint korrigieren.
6. Fehlende Benchmark-Dateien ergänzen oder das Feature `datasets` entfernen, damit die Qualitätsmessung wieder läuft.
7. Die im Audit übernommenen Befunde zu engine, mcp, cognition und store nachprüfen.

## 9. Offene Punkte für den Auftraggeber

- Soll die Löschfunktion über die öffentliche API physisch erzwingbar sein? Das ist eine Produktentscheidung.
- Wo liegt der Fix-Branch `fix/test-targets`?
- Die Umgebung hatte nur 1 vCPU und wenig Speicher. Die Build-Zeit war erheblich. Ein vollständiger Testlauf auf stärkerer Hardware wird empfohlen.
