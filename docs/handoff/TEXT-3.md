# Übergabebericht MISSION `TEXT-3` · Invertierter Index, Lebenszyklus, Crate-Hülle

## 1. Status & Soll/Ist-Delta
- **Eingefrorener Vertrag (`src/contract.rs`):** Byte-identisch angelegt mit `Error`, `Result`, `Clock`, `Rng`, `DocKey`, `Epoch`. Re-exportiert in `src/lib.rs`.
- **Thread-sicherer Inverted Index mit Snapshot-Isolation (`src/memory_index.rs`):**
  - Implementiert `Index<DocKey, str>` und `Maintain` (`compact`, `vacuum`).
  - Read-Only Snapshots (`IndexSnapshot`): Leser blockieren Schreiber nicht länger als O(1).
  - Schreiber arbeiten Copy-on-Write (CoW).
  - Snapshot-Konsistenz: Wörterbuch, Posting-Listen und Korpusstatistiken (`N`, `avgdl`, `df`) sind strikt synchronisiert.
  - Opake, monotone `Epoch` als Sichtbarkeitsmarke.
  - Neutrale Löschung über Tombstone und physische Bereinigung per `Maintain::compact()`.
  - Fail-closed Kapazitätslimits (`MAX_DOC_BYTES`, `MAX_DOC_TERMS`, `MAX_POSTINGS_PER_TERM`, `MAX_SEARCH_K`).
  - `inspect`-Sicht unter Feature `inspect` mit `KeyInspection`.
- **Adapter-Übergabe (`docs/handoff/TEXT-3/adapter_src/`):**
  - Port-Brücken (`TextPurger`, `TextScanner`, `Bm25Scorer`) unter `docs/handoff/TEXT-3/adapter_src/` gesichert.
  - Alt-Brücken für externe Consumer hinter `types-compat`/`ports-compat` belassen.
- **Crate-Hülle:**
  - `rust-version = "1.89"`, Standard-Features (`std`, `serde`, `tracing`, `inspect`, `experimental`).
  - `README.md` mit Copy-Paste-Anleitung und autarkem Minimalbeispiel.
  - `SPEC.md` mit Invariantentabelle inklusive Testzuordnungen (`INV-TEXT-1`, `INV-TEXT-WAND-1`, etc.).
  - `LICENSE-MIT` und `LICENSE-APACHE` verifiziert.
- **Tests & Verifikation:**
  - Snapshot-Nebenläufigkeitstest: `tests/snapshot_concurrency_test.rs`
  - Property-Test gegen Referenzmodell-Orakel: `tests/snapshot_oracle_proptest.rs`
  - Fault-Injection und Limit-Tests: `tests/fault_injection_limits_test.rs`
  - Zero-Panic: 0 Prod-Verstöße.
  - Unsafe-Boundary: 0 fehlende `// SAFETY:`-Dokumentationen.
  - Gate G3 Export-Test: Erfolgreich bestanden.
  - Isolation Proof (M2): Erfolgreich bestanden.

## 2. Funktionsinventar-Diff
- Keine Funktionen entfernt (Removed: 0).
- 8 neue Funktionen/Methoden addiert (u. a. `IndexSnapshot::search`, `MemoryTextIndex::snapshot`, `inspect_key`, `current_epoch`).

## 3. Offene Punkte / Externe Abhängigkeiten
- `capabilities.toml` führt `contextra-text` mit `may_depend_on = ['contextra-ports', 'contextra-types']`. Sobald übergeordnete Prompts (z. B. `ENG-2`) die externen Verwendungen (`contextra-engine`, `contextra-agent`) auf die neuen Adapter umgestellt haben, können die transitionalen Feature-Flags `types-compat` und `ports-compat` komplett entfallen.
