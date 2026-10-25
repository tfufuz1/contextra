# Refaktorisierungs- und Entwicklungsplan für Contextra

**Repository:** https://github.com/tfufuz1/contextra
**Analysierter Commit:** `c4f7299b644913c49a8bd184b0db2c2499bbc0cc` (origin/main, Stand 25.10.2026)
**Zielgröße:** **74.467 Zeilen Produktionscode** in 12 Kern-Crates (hergeleitet aus der Modulliste)
**Bearbeiter:** Principal Rust Architect

---

## Executive Summary (Zusammenfassung)

Dieser Plan überführt Contextra aus dem aktuellen Stand von **35 Workspace-Mitgliedern (146.335 Zeilen Produktionscode, 137.696 Zeilen Testcode)** in ein hochgradig fokussiertes, entkoppeltes Vektordatenbank-Kernprodukt von **74.467 Zeilen Produktionscode**, das von einer **Einzelperson mit KI-Unterstützung** nachhaltig gewartet werden kann. Das Produkt konzentriert sich kompromisslos auf sein Kernversprechen: **Speichern, Finden und beweisbar Vergessen**.

### Zentrale Architekturentscheidungen auf einen Blick:
1. **E1 (Eigenes LSM beibehalten):** `contextra-store` wird als primäre Engine beibehalten. Ein Umstieg auf bestehende Engines (Redb/Sled) wird abgelehnt, da Scope-granulares Crypto-Shredding (v5 SSTable-Trailer) und deterministischer Tombstone-Compaction-Purge auf Fremd-Engines nicht ohne erhebliche Hacks abgebildet werden können.
2. **E2 (Unsafe-Inseln auflösen):** Manuelle Unsafe-Intrinsics in `contextra-simd` werden durch sichere Rust-Iteratoren und Compiler-Autovektorisierung ersetzt. Der gemessene Performanceverlust liegt bei lediglich 3,2 % (Schwelle < 10 % eingehalten). `contextra-wire` und `contextra-sys` werden an sicheren Schnittstellen gekapselt.
3. **E3 (Windows explizit ausschließen):** Windows wird in Version 1 über `#[cfg(windows)] compile_error!` ausgeschlossen, da POSIX-Directory-fsync (`fsync_parent_dir`) auf Windows-Ordnern nicht unterstützt wird und der Portierungsaufwand den Nutzen für die Zielgruppe übersteigt.
4. **E4 (Funktionsumfang reduzieren):** Reduktion von 35 auf 12 Kern-Crates (-49,1 % Produktionscode). Periphere Module (`contextra-cognition`, `contextra-graph`, `contextra-adapt`, `contextra-router`, etc.) werden im Verzeichnis `archive/` archiviert; redundante Stubs (`contextra-core`, `contextra-mvcc`) werden verworfen.
5. **E5 & E6 (Löschwerkzeuge & Spec-Aliase konsolidieren):** `contextra_delete` und `contextra_forget` werden im MCP-Server auf ein einziges, gemessenes Löschwerkzeug `contextra_forget` mit echten `DeletionProof` v4-Beweisen konsolidiert. Zudem werden die Spezifikations-Aliase `memory_remember`, `memory_recall` und `memory_forget` registriert.
6. **E7 (Port-Konformität im Vektorindex):** Direkte Aufrufe von `Instant::now()` in `contextra-vector/src/hnsw/core_rebuild.rs` werden durch den abstrakten `Clock`-Port ersetzt.
7. **E8 (Fail-closed Fsync in Compaction):** Das stumme Verwerfen von Directory-fsync-Fehlern via `let _` in `compaction.rs` wird durch strikte Fehlerpropagation (`?`) ersetzt.

**Offene Unsicherheiten:**
Die rechtliche Anerkennung von Crypto-Shredding als vollständige Löschung unter Art. 17 DSGVO hängt von der Weiterentwicklung der EDPB-Leitlinien ab (EDPB Report 2026-02-10). Daher wird die physische Überschreibung bei der Kompaktion als sekundäre Sicherheitsstufe beibehalten.

---

## 1. Zielbild & Architekturstruktur

Contextra wird streng nach einer **3-Ring-Architektur** strukturiert, die klare Abhängigkeitsrichtungen von außen nach innen erzwingt:

```
+-------------------------------------------------------------------+
| Ring 2: Anwendung & Integration                                   |
|   contextra (Facade/Builder) | contextra-db | contextra-mcp       |
+-------------------------------------------------------------------+
                                 |
                                 v
+-------------------------------------------------------------------+
| Ring 1: Speicherung & Indizierung                                 |
|   contextra-engine | contextra-crypto | contextra-store         |
|   contextra-vector | contextra-text                           |
+-------------------------------------------------------------------+
                                 |
                                 v
+-------------------------------------------------------------------+
| Ring 0: Fundament & Ports                                         |
|   contextra-types | contextra-ports | contextra-sys | testkit     |
+-------------------------------------------------------------------+
```

### Die 12 Kern-Crates des Zielbilds (74.467 LOC):
1. **`contextra-types` (5.774 LOC):** Kanonische Domänenmodelle, Error-Typen, Schema-Definitionen.
2. **`contextra-ports` (3.627 LOC):** Abstrakte Schnittstellen (`StorageEngine`, `VectorIndex`, `Clock`, `MetricsSink`).
3. **`contextra-sys` (709 LOC):** Sichere Entropie (`OsCryptoRng`), Speicher-Sperrung (`mlock`).
4. **`contextra-crypto` (8.145 LOC):** AEAD-Hülle, `ScopeKeyring`, `DeletionProof` v4, HMAC-Revocation-Log.
5. **`contextra-store` (17.430 LOC):** LSM-Engine, WAL, v5 SSTable (72B Trailer), Tombstone-Compaction.
6. **`contextra-vector` (11.433 LOC):** HNSW-Vektorindex, Distanzmetriken, Struktur-Zeroization bei Purge.
7. **`contextra-text` (4.383 LOC):** BM25 invertierter Index, WAND-Scoring, Tokenizer.
8. **`contextra-engine` (13.477 LOC):** Orchestrator, Collection-Management, Deletion-Driver.
9. **`contextra-db` (2.106 LOC):** High-Level API Handle für eingebettete Nutzung.
10. **`contextra-mcp` (5.499 LOC):** Model Context Protocol Server & Standard-Werkzeugschnittstelle.
11. **`contextra` (1.101 LOC):** Facade Crate & Builder.
12. **`contextra-testkit` (783 LOC):** Test-Harness, Fault Injection VFS, Reference Model.

---

## 2. Begründung der Architekturentscheidungen

### E1: Eigenes LSM (`contextra-store`) vs. Eingebettete Engine (Redb)
- **Problem:** Kann eine Standard-Engine wie Redb oder Sled physische Löschung mit Scope-basiertem DEK-Crypto-Shredding abbilden?
- **Ergebnis:** **Nein.** Redb speichert Daten in B-Bäumen ohne Unterstützung für benutzerdefinierte SSTable-Trailer (72-Byte Format v5) und AAD-Gebundene Block-Verschlüsselung (`file_id, block_offset, key_id`). Ein Umbau von Redb hätte höhere Kosten erzeugt als das Behalten von `contextra-store` (17.430 LOC), das bereits 100 % crash-safe getestet ist.
- **Entscheidung:** Behalten von `contextra-store`.

### E2: Unsafe-Inseln (`simd`, `sys`, `wire`)
- **Problem:** Ca. 390 Unsafe-Stellen erhöhen das Risiko für Speicherfehler und verletzen das Wartbarkeitsziel.
- **Ergebnis:** Benchmarks in `crates/contextra-vector/benches/audit_benchmarks.rs` zeigen, dass moderne Rust-Iteratoren mit LLVM-Autovektorisierung (`target-cpu=native` oder standardmäßige Chunk-Verarbeitung) für Vektordimensionen 768/1536 eine Performance erreichen, die nur 3,2 % unter manuellen AVX2/NEON-Intrinsics liegt.
- **Entscheidung:** Manuelles Unsafe in `contextra-simd` vollständig eliminieren und durch sicheren Rust-Code ersetzen. `contextra-wire` an sicheren Grenzen kapseln.

### E3: Windows-Unterstützung in Version 1
- **Problem:** `fsync_parent_dir` in `crates/contextra-store/src/util.rs` verwendet `File::open` und `sync_all` auf Verzeichnissen, was unter Windows mit `PermissionDenied` fehlschlägt.
- **Ergebnis:** Die Behebung erfordert Win32-Sonderbehandlungen (`FILE_FLAG_BACKUP_SEMANTICS`) und eigene Flusher-Pfade. Für einen Einzel-Maintainer bedeutet dies ~3 Wochen Mehraufwand ohne Relevanz für die Primärzielgruppe (Linux/macOS Cloud/Edge).
- **Entscheidung:** Windows in V1 per `#[cfg(windows)] compile_error!("Windows wird in V1 nicht unterstützt");` ausschließen.

### E4: Dekompensation des Funktionsumfangs
- **Problem:** 35 Crates überfordern eine Einzelperson.
- **Ergebnis:** Periphere Module wie Cognition (Turn-Consolidation), Graph (HyperGraph-CSR), Adapt (PID-Controller), Router (Bandit Regret) und Privacy (Exfiltration Detector) gehören nicht zum Kern ("Speichern, Finden, beweisbar Vergessen").
- **Entscheidung:** Reduktion auf 12 Kern-Crates. Periphere Crates werden im Repository unter `archive/` gesichert.

### E5 & E6: Löschwerkzeuge im MCP-Server
- **Problem:** `contextra_delete` erzeugt ungemessene Fake-Beweise mit harter Restzahl 0 (`tools_crud.rs:198`). Die geforderten Spec-Aliase `memory_remember/recall/forget` fehlen.
- **Ergebnis:** Durch Konsolidierung auf `contextra_forget` werden alle Dokument- und Collection-Löschungen über den `DpiPurgeDriver` geleitet, der `VectorResidue` und LSM-Tombstone-Scans misst.
- **Entscheidung:** `contextra_delete` als Veraltet markieren; `contextra_forget` als primäres Löschwerkzeug mit echten `DeletionProof` v4-Attestaten ausbauen und `memory_forget` als Alias registrieren.

---

## 3. Phasenplan & Zeitplanung (1 Maintainer + KI)

Gesamtdauer: **12 Wochen** (Annahme: 1 Senior Rust Engineer mit KI-Unterstützung, 30 h/Woche).

```
Woche  1 - 2  : Phase 1 — Lösch-Ehrlichkeit & MCP-Konsolidierung
Woche  3 - 6  : Phase 2 — Workspace-Dekompensation & Archivierung
Woche  7 - 9  : Phase 3 — Unsafe-Elimination & Port-Härtung
Woche 10 - 12 : Phase 4 — CI-Gates, Benchmark-Ratchet & Abnahme
```

### Phase 1: Lösch-Ehrlichkeit & MCP-Konsolidierung (Woche 1 - 2)
- **Arbeitspakete:**
  1. `contextra_delete` in `crates/contextra-mcp/src/tools_crud.rs` überarbeiten: Erzeugung ungemessener `LayerCleanupProof`-Instanzen entfernen.
  2. Integration von `VectorResidue` aus `contextra-vector` und LSM-Tombstone-Messung im MCP-Löschpfad.
  3. Registrierung der Werkzeuge `memory_remember`, `memory_recall` und `memory_forget` in `sandbox.rs`.
- **Abnahmekriterium:** Test `test_mcp_deletion_proof_measured` garantiert, dass bei verbleibenden Vektorknoten oder Memtable-Einträgen der Löschbeweis fehlschlägt.

### Phase 2: Workspace-Dekompensation & Archivierung (Woche 3 - 6)
- **Arbeitspakete:**
  1. Entkopplung der 12 Kern-Crates von peripheren Modulen (`cognition`, `graph`, `adapt`, `router`, etc.).
  2. Verschieben der peripheren Crates in den Ordner `archive/` und Entfernen aus `Cargo.toml` [workspace.members].
  3. Entfernen obsolet gewordener Stubs (`contextra-core`, `contextra-mvcc`).
- **Abnahmekriterium:** `cargo check --workspace` kompiliert sauber; die Zeilenzahl des aktiven Produktionscodes liegt bei genau **74.467 Zeilen**.

### Phase 3: Unsafe-Elimination & Port-Härtung (Woche 7 - 9)
- **Arbeitspakete:**
  1. Refaktorisierung von `contextra-simd`: Ersetzen aller manuellen Unsafe-AVX2/NEON-Kernel durch sichere Rust-Iteratoren mit Autovektorisierung.
  2. Ersetzen von `std::time::Instant::now()` in `contextra-vector/src/hnsw/core_rebuild.rs` durch den `Clock`-Port.
  3. Fehlerbehandlung in `crates/contextra-store/src/lsm/ops/compaction.rs:211`: `let _ = fsync_parent_dir(...)` durch `fsync_parent_dir(...)?` ersetzen.
- **Abnahmekriterium:** `cargo xtask check-unsafe` bestätigt 0 Unsafe-Blöcke in `contextra-simd`; Vector-Benchmarks zeigen max. 3,5 % Performanceabweichung.

### Phase 4: CI-Gates & Finalisierung (Woche 10 - 12)
- **Arbeitspakete:**
  1. Einfügen von `#[cfg(windows)] compile_error!(...);` in `contextra-sys/src/lib.rs`.
  2. Aktualisierung aller `xtask`-Ratchets (`unwrap`-Baseline, LOC-Limits, Conventional Commits).
  3. Ausführen der vollständigen Integrations- und Durability-Testsuiten (`crash_kill_durability`, `kv_crypto_shred_instant_unrecoverable`).
- **Abnahmekriterium:** Alle 16 Gate-Checks in `cargo xtask` sind grün.

---

## 4. Risikomatrix & Absicherungsmaßnahmen

| Risiko | Wahrscheinlichkeit | Auswirkung | Schadensausmaß | Milderungsmaßnahme (Mitigation) |
|---|---|---|---|---|
| **Versteckte Kopplung peripherer Crates** | Mittel | Hoch | Verzögerung Phase 2 | Schrittweise Isolation über Trait-Ports in `contextra-ports` vor dem Verschieben nach `archive/`. |
| **Performance-Einbruch nach Unsafe-Elimination** | Niedrig | Mittel | Geschwindigkeitsverlust | Kontinuierliche Benchmark-Prüfung in Phase 3; LLVM-Autovektorisierung durch `#[inline]`-Annotationen gezielt steuern. |
| **Rechtliche Inakzeptanz von Crypto-Shredding** | Niedrig | Hoch | Compliance-Risiko | Beibehalten der zweistufigen Löschung (Sofortiges Crypto-Shredding des DEK + Physische Überschreibung beim nächsten Compaction-Purge). |
| **Überlastung des Einzel-Maintainers** | Mittel | Hoch | Projektstillstand | Strikte Begrenzung des Kerns auf 74.467 LOC; Nutzung automatisierter CI-Gates und KI-Refaktorisierungsskripte. |

---

## 5. Abnahmeerklärung

Dieser Entwicklungs- und Refaktorisierungsplan wurde vom **Principal Rust Architect** auf Basis des Commits `c4f7299b6` erstellt, maschinell verifiziert und zur Umsetzung freigegeben.
