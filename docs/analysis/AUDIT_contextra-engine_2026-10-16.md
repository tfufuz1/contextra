# AUDIT-Bericht: Scope S9 — Collection-Orchestrierung & CRUD-Pfade (contextra-engine)

**Rolle:** S9 — Spezialist für die Orchestrierung der Collections, CRUD-Pfade und Mandantenisolation
**Scope:** `contextra-engine` (ohne Commit- und Rollback-Pfad [S3] und ohne Löschkern [S4])
**Basis / Vorgänger:** `docs/analysis/AUDIT_collection-cognition-router-agent-db_DEEP_2026-10-13.md`
**Datum:** 2026-10-16
**Commit:** `22e599bdfb73b9d85e3333f1a21a11c9c5dcdf9b`

---

## 1. Zusammenfassung (Ampel je Prüfbereich)

| Prüfbereich | Status | Grund / Kurzzusammenfassung |
|---|---|---|
| **Mandantenisolation** | 🟢 GRÜN | Jeder öffentliche Pfad leitet den Mandanten aus dem Kontext ab (`TenantPolicy::Required` / `TenantIsolatedKvStore`). Kein Fallback auf Standardmandant wenn Policy `Required`. |
| **Konsistenz der Indizes** | 🟡 GELB | Multi-Index-Schreibvorgänge laufen über `DbTransaction` / CommitLedger. Bitemporale Filterung in `apply_temporal_validity_filter_at` verwechselte Unix-Zeitstempel mit System `TxId` (durch `apply_temporal_validity_filter_at_bitemporal` korrigiert/resolviert). |
| **Collection-Lebenszyklus** | 🟢 GRÜN | `drop_collection` bereinigt LSM-Präfixe, HNSW/BM25/Graph RAM-Zustände und erzeugt Ed25519 `DeletionProof`. Wiederanlegen unter gleichem Namen ist isoliert. |
| **Fehlerbehandlung & Retry** | 🟢 GRÜN | Keine stummen Fehler im Collection-Scope. Idempotente Operationsmuster eingehalten; Fehler werden als typisierte `ContextraError` fortgepflanzt. |
| **Locks & Concurrency** | 🟢 GRÜN | Lock-Hierarchie (`collections` -> `kv_locks` -> `embedder`) strikt eingehalten. Keine synchronen Lock-Guards über `.await`-Punkte. |
| **Typzustände & Invarianten** | 🟢 GRÜN | Zero-Panic im Produktionscode. Keine `unwrap`/`expect`/`panic!` in `src/`. `#![forbid(unsafe_code)]` workspace-weit eingehalten. |
| **Konstante Platzhalter** | 🟡 GELB | `init_embedding_backend` liefert `Ok(())` (legitimer No-Op Stub für dynamisches Laden). `no_graph_stubs.rs` Stubs aktiv bei fehlendem `graph` Feature. |
| **Zeitachsen & Bitemporalität** | 🟢 GRÜN | Systemzeit (`TxId`) und Geschäftszeit (`valid_from`/`valid_to`) sind in `temporal_filter.rs` sauber getrennt. |
| **Hintergrundprozesse** | 🟢 GRÜN | `TaskTracker` und `CancellationToken` in `Contextra` steuern Expiry-/Orphan-Worker geordnet ohne Datenverlust beim Herunterfahren. |

---

## 2. Delta zum letzten Bericht (`2026-10-13`)

- **Neu / Geändert seit 2026-10-13:**
  - `refactor(text,engine): eliminate u64::MAX read timestamps for MVCC snapshot isolation (K1.13)` (`22e599bdfb73b9d85e3333f1a21a11c9c5dcdf9b`): In `search/filtered.rs`, `query_builder/builder_exec.rs` und `search/hydrate.rs` wurden unbegrenzte `u64::MAX` Lese-Zeitstempel durch explizite MVCC-Snapshots ersetzt.
  - `ENG-FILTER-001` (Bitemporale Filterung): `apply_temporal_validity_filter_at_bitemporal` wurde eingeführt, um `as_of_tx` (TxId) und `as_of_business` (u64) strikt zu trennen. Die alte Funktion `apply_temporal_validity_filter_at` wurde als `#[deprecated]` markiert und korrigiert.
- **Behoben:**
  - Eliminierung von `u64::MAX` Read-Timestamps in `search/filtered.rs` und `search/hydrate.rs`.
  - Abspaltung veralteter WasmMergeOperator-Compaction-Tests in eigenständige Integrationsprüfungen.
- **Offen:**
  - Unverdrahtete Deletion-Komponenten `DpiPurgeDriver` und `reverse_sweep` (`ENG-DEL-001`) haben im Produktionspfad von `contextra-engine` weiterhin keinen direkten Aufrufer (gehören zum Deletion-Purge-Sweep S4).
  - Veralteter Kompilierfehler in `contextra-store` (`crates/contextra-store/src/erasure_ledger/mod.rs:938` unclosed delimiter) blockiert Crate-übergreifende `cargo test`-Läufe.

---

## 3. Detaillierte Befunde

### FINDING-01: Inkorrekte Zeitstempel-Interpretation in `apply_temporal_validity_filter_at`
- **ID:** `ENG-FILTER-001`
- **Schwere:** P2 (Doku-Drift / API-Klarheit)
- **Datei:Zeile:** `crates/contextra-engine/src/temporal_filter.rs:163`
- **Beleg:**
  ```rust
  #[deprecated(since = "0.1.0", note = "Use apply_temporal_validity_filter_at_bitemporal with explicit TxId and business time instead")]
  pub fn apply_temporal_validity_filter_at(results: Vec<crate::SearchResult>, as_of_timestamp: u64) -> Vec<crate::SearchResult> { ... }
  ```
- **Risiko:** Wenn Aufrufer die veraltete Funktion nutzen, könnten Geschäfts-Zeitstempel fälschlicherweise als System-Transaktions-IDs eingegeben werden.
- **Empfehlung:** Migration aller Aufrufer in `contextra-engine` und `contextra-db` auf `apply_temporal_validity_filter_at_bitemporal`.
- **Aufwand:** S

### FINDING-02: Blockierter Crate-Build von `contextra-store` beeinträchtigt globale Integrationstests
- **ID:** `ENG-STORE-001`
- **Schwere:** P1 (Verschluckter / blockierter Testlauf)
- **Datei:Zeile:** `crates/contextra-store/src/erasure_ledger/mod.rs:938`
- **Beleg:**
  `cargo test -p contextra-engine` schlägt fehl mit `error: this file contains an unclosed delimiter` in `crates/contextra-store/src/erasure_ledger/mod.rs:938`.
- **Risiko:** Integrationstests gegen den realen LSM-Store können auf Workspace-Ebene nicht durchgeführt werden, solange `contextra-store` einen Syntaxfehler enthält.
- **Empfehlung:** Behebung der schließenden Klammer in `contextra-store/src/erasure_ledger/mod.rs` durch den zuständigen S3/S4 Role Owner.
- **Aufwand:** S

---

## 4. Testlücken

1. **Zwei-Mandanten-Matrix:**
   - *Vorschlag:* Test für `Collection::insert`, `get`, `delete` und `hybrid_search` unter zwei isolierten Mandanten (`TenantId(1)` und `TenantId(2)`).
   - *Erwartete Invariante:* Keine Daten von Tenant 1 dürfen bei Abfragen von Tenant 2 sichtbar sein, selbst wenn identische Document IDs genutzt werden.
2. **Fehlerinjektion zwischen Indexschritten:**
   - *Vorschlag:* Abbruch/Panic-Injektion nach dem LSM-Schreiben, aber vor der HNSW-Vektor-Aktualisierung.
   - *Erwartete Invariante:* `DbTransaction` / `CommitLedger` führt beim Rollback zur vollständigen Bereinigung aller vorbereiteten Index-Änderungen.

---

## 5. Technische Schulden

| Typ | Ort | Aufwand | Priorität / Reihenfolge |
|---|---|---|---|
| Stub-Impl | `crates/contextra-engine/src/contextra_impl/lifecycle.rs:178` (`init_embedding_backend`) | S | 3 (Niedrig) |
| Orphan Symbol | `crates/contextra-engine/src/deletion/purge_driver.rs:38` (`DpiPurgeDriver`) | M | 2 (Mittel, S4 Scope) |
| Deprecated API | `crates/contextra-engine/src/temporal_filter.rs:163` (`apply_temporal_validity_filter_at`) | S | 1 (Hoch) |

---

## 6. Hypothesen

- *Hypothese 1:* Die parallele Vektor- und BM25-Suche in `Collection::hybrid_search` könnte bei hoher Last durch Asynchronität auf Tokio-Worker-Thread-Ebene beschleunigt werden (aktuell sequentiell).
  - *Klärender Test:* Latency Benchmark über `tokio::join!` vs. sequentielle Awaits unter 100 concurrent requests.

---

## 7. Quellenkonflikte und Doku-Drift

- **Doku-Drift:** `AGENTS.md` in `contextra-engine` beschreibt `ENG-026 Sperrenhierarchie` (`collections` -> `kv_locks` -> `embedder`). Im Code wird diese Hierarchie eingehalten, jedoch fehlen explizite Lock-Trace-Logs bei Rekursion.

---

## 8. Ausgeführte Befehle und Exit-Codes

| Befehl | Exit-Code | Status / Bemerkung |
|---|---|---|
| `cargo xtask harness-list` | 0 | Harness-Kommandos gelistet |
| `cargo xtask crate-context contextra-engine` | 0 | Context gelesen |
| `cargo xtask stub-impls --crate contextra-engine` | 0 | 1 Stub gefunden (`init_embedding_backend`) |
| `cargo xtask orphan-symbols` | 0 | Symbole gescannt |
| `cargo xtask unwired-ports` | 0 | Keine unverdrahteten Ports im Scope |
| `cargo xtask check-no-downcast` | 0 | Zero Downcasts |
| `cargo xtask lock-audit` | 0 | Keine Locks über `.await` |
| `cargo xtask doctrine-scan --crate contextra-engine` | 1 | Status-Widerspruch wegen unclosed delimiter in externer Crate `contextra-store` |

---

## 9. Nicht Geprüftes und Gründe

- **Volle E2E-Cargo-Testausführung für `contextra-engine`:** Blockiert durch Syntaxfehler in `crates/contextra-store/src/erasure_ledger/mod.rs:938` (`unclosed delimiter`), was eine externe Abhängigkeit außerhalb des Scopes S9 darstellt.
