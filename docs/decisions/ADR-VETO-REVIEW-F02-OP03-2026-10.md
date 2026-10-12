# ADR-VETO-REVIEW-F02-OP03-2026-10: Überprüfung und Bestätigung der Veto-Sperren F-02 und OP-03

* **Status:** Accepted (Bestätigt)
* **Datum:** 2026-10-12
* **Target Path:** VETOES.md, docs/decisions/ADR-VETO-REVIEW-F02-OP03-2026-10.md
* **Kontext / Auslöser:** Fällige Governance-Überprüfung der bedingt akzeptierten Veto-Sperren VETO-F02 (Frist: 2026-10-07) und VETO-OP03 (Frist: 2026-10-07) gemäß Governance-Regeln und `xtask check-vetoes`.

---

## 1. VETO-F02: HNSW Partial-Rebuild / Tombstone-Pruning

### Entscheidungsstand
**Bestätigt (Unverändert in Kraft)**

### Begründung
Die Überprüfung des Quellcodes und der Test-Suites im Workspace bestätigt, dass die Vorgaben von VETO-F02 strikt eingehalten werden:
1. **Quellcode (`crates/contextra-vector/src/hnsw/core_rebuild.rs`):**
   Die Methode `rebuild_region_sync` führt ausschließlich ein reines Filtern/Pruning von Tombstones aus den Adjazenzlisten in `arena` und `counts` durch (`if !tombstoned_set.contains(&neighbor_u32)`). Es findet kein partielles Rewiring, Nachbarschafts-Searching oder Neu-Verknüpfen von HNSW-Kanten statt.
2. **Sicherheits- & Regressions-Tests:**
   - `crates/contextra-vector/tests/partial_rebuild_veto_f02_guard.rs`: Verifiziert explizit, dass `rebuild_region` ausschließlich Tombstones entfernt und gelöschte Dokumente vollständig isoliert.
   - `crates/contextra-vector/tests/partial_rebuild_recall_regression.rs`: Testet `test_partial_rebuild_node_degree_no_isolation` und garantiert invariant, dass `rebuild_region` zwar Tombstones prunt, aber keine neuen Kanten knüpft.

Die Sperre gegen partielles HNSW-Rewiring / Teilgraph-Rebuilding bleibt zur Vermeidung von Recall-Kollaps und RwLock-Contention uneingeschränkt aufrecht erhalten.

---

## 2. VETO-OP03: Realtime-Audio / Voice / Speech-to-Text / Jarvis

### Entscheidungsstand
**Bestätigt (Unverändert in Kraft)**

### Begründung
Eine vollständige Workspace-Prüfung aller Crates (`crates/*`) ergab:
1. Es existieren keinerlei Subsysteme, Treiber oder Abstraktionen für Speech-to-Text (STT), Text-to-Speech (TTS), Realtime-Audio-Streaming, Voice-Assistenten oder "Jarvis"-Features.
2. Die einzige Erwähnung von Audio im Codebase betrifft rein textuelle Metadaten-Repräsentationen (`SignalModality::AudioTranscript` in `crates/contextra-db/src/volatile_vault.rs`), welche lediglich Text-Transkripte als bi-temporale Speicherobjekte kennzeichnen.

Die Einbindung von Echtzeit-Audio-Verarbeitung widerspricht der Produktvision eines deterministischen bi-temporalen Speichersubstrats (ADR-077) und bleibt daher strikt untersagt.

---

## 3. Konsequenzen & Wiedervorlage
- In `VETOES.md` wird der Verweis `adr_ref` für VETO-F02 und VETO-OP03 auf `docs/decisions/ADR-VETO-REVIEW-F02-OP03-2026-10.md` aktualisiert.
- Nächste ordentliche Wiedervorlage/Review-Frist für beide Vetos: **2027-10-12**.
