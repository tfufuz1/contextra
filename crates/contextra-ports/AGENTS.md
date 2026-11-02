# AGENTS.md — `contextra-ports`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.2](docs/SPEC.md) und SPEC §15.4**

## 1. Rolle & Zweck
`contextra-ports` definiert sämtliche abstrakten Port-Schnittstellen (Traits), über die die Ringe 1, 2 und 3 lose und deterministisch gekoppelt sind.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 0 (Fundament)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `futures` (ausschließlich `BoxFuture` für asynchrone Trait-Methoden).
- **Verbotene Abhängigkeiten:** Keine Abhängigkeiten zu Ring 1–4.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Keine konkreten Produktions-Implementierungen (nur Conformance-Testsuiten).
  - Keine direkten Systemaufrufe (`Instant::now`, `thread_rng` sind strikt verboten).

## 3. Port-Inventar (Soll-Zustand laut SPEC §8.2)

| Trait | Kernmethoden | Soll-Implementierer |
|---|---|---|
| `Vfs` / `VfsFile` | `open`, `read_at`, `write_at`, `append`, `set_len`, `sync_data`, `sync_all`, `rename`, `remove_file`, `exists`, `create_dir_all`, `read_dir`, `sync_dir`, `lock_exclusive`, `set_permissions`, `metadata` | `RealVfs` (`contextra-durable-fs`), `FaultVfs`, `MemVfs` (`contextra-testkit`) |
| `Clock` | `now() -> Timestamp`, `monotonic() -> u64` | `SystemClock`, `ManualClock` |
| `Rng` | `fill(&mut [u8])` (nur Nicht-Krypto); `OsEntropy::fill` für Krypto-Zufall | `SeededRng`, `OsRng`-Wrapper |
| `IdGen` | `erasure_id() -> ErasureId`, `report_id() -> ReportId` | ID-Generatoren |
| `StorageEngine` | `get`, `get_at_seq`, `put`, `put_batch`, `delete`, `commit`, `rollback`, `scan_range`, `scan_range_at`, `flush`, `last_seq`, `pin_snapshot`, `unpin_snapshot`, `stats` (alles `async` auf `KeyBuf`/`Bytes`) | `LsmStorage` (`contextra-store`), `InMemoryStore` (testkit), redb-Adapter (Rückfall) |
| `TextIndex` | `add(doc_tag, tokens)`, `remove_docs(set)`, `search(coll, query, k)`, `flush`, `merge`, `segments_of(doc_tag)` | `contextra-text` |
| `VectorIndex` | `insert(slot, vec)`, `search`, `zero_slot`, `rebuild`, `snapshot`, `restore` | `contextra-vector` |
| **`ErasureLayer`** | `layer_id()`, `plan(&Matcher) -> LayerPlan`, `purge(&PurgePlan, &Deadline) -> PurgeOutcome`, `scan_residue(&NeedleSet) -> LayerScan`, `positive_control(&Canary) -> bool` | `contextra-store`, `contextra-text`, `contextra-vector` |
| `ReadGate` | `blocked(&KeyBuf) -> bool`, `blocked_range(&KeyRange) -> bool`, `epoch() -> u64` | `ErasedSet` (`contextra-erasure`) |
| `RecordSealer` | `seal(doc_tag, aad, plaintext) -> Result<Bytes>`, `open(doc_tag, aad, sealed) -> Result<Zeroizing<Vec<u8>>>`, `forget(doc_tag)` | Tresor (`contextra-erasure`) |
| `LedgerSink` | `append(&LedgerEntry)`, `read_all() -> Vec<LedgerEntry>` | `FileLedger` (`contextra-erasure`), `MemLedger` (testkit) |
| `Metrics` | `counter`, `histogram`, `gauge` (statische Metriknamen, keine IDs in Labels) | Engine/Observability |

## 4. Kritische Invarianten & Verträge
1. **`ErasureLayer`-Vertrag:**
   - `purge` ist **idempotent**, **kooperativ abbrechbar** (Frist via `Deadline`), modifiziert ausschließlich Daten innerhalb des `Matcher` (P4) und meldet jeden Fehler als `Err`.
   - `scan_residue` darf Daten nur lesen und niemals mutieren.
   - `positive_control` beweist maschinell, dass der Scanner seinen eigenen Test-Canary zuverlässig findet (SPEC §12.2).
2. **`ReadGate`-Vertrag:** Fail-Closed; bei unbekanntem Zustand oder Fehlern wird geblockt.
3. **`Vfs`-Vertrag:** Jede Flush- und Sync-Operation liefert ein verbindliches `std::io::Result<()>`, das niemals ignoriert werden darf.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (SPEC §15.4)

- **[ ] WP-1.2: Veraltete Ports entfernen**
  - Streichen von: `attention`, `checkpoint`, `embedding`, `graph`, `graph_index`, `license`, `plugin`, `query_rewriter`, `reranker`, `scanner`.
- **[ ] WP-2.6: Vfs-Port konsolidieren (D-04)**
  - Zusammenführen der zwei bestehenden `Vfs`-Traits zu einer einzigen kanonischen Schnittstelle mit `VfsFile`.
- **[ ] WP-3.5: Ports für Löschinfrastruktur definieren**
  - Definition von `ReadGate` und `LedgerSink`.
- **[ ] WP-3.6 / WP-3.7: `ErasureLayer`-Port definieren**
  - Bereitstellung des normativen `ErasureLayer`-Traits und Integration in die Conformance-Testsuite `ports::contract_tests`.

## 6. Verifikation & Tests
- **Conformance-Testsuite:** `ports::contract_tests` läuft gegen jede Trait-Implementierung (Produktion, Mock, Fault).
- **T-ARCH-1:** Keine unzulässigen Abhängigkeiten oder verbotene Port-Exporte.
