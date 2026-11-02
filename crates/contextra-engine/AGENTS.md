# AGENTS.md — `contextra-engine`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.12](docs/SPEC.md) und SPEC §15.4**

## 1. Rolle & Zweck
`contextra-engine` orchestriert die Teilsysteme (`contextra-store`, `contextra-text`, `contextra-vector`, `contextra-rank`, `contextra-erasure`) zu einem kohärenten Ganzen. Es stellt Collections, Transaktionen, Dokumentenzerlegung, Hybridsuche und die Laufzeit-Verdrahtung bereit — besitzt jedoch **keine eigene Löschlogik** (diese ist an `contextra-erasure` und die `ErasureLayer` delegiert).

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 3 (Orchestrierung)
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-durable-fs`, `contextra-crypto`, `contextra-mvcc`, `contextra-store`, `contextra-text`, `contextra-vector`, `contextra-rank`, `contextra-erasure`.
- **Verbotene Abhängigkeiten:** Keine Abhängigkeit zu `contextra` (Ring 4) oder externen Graph-/ML-/Netzwerk-Crates.
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - Kein Entschlüsseln von Dokumenten oder Records vor erfolgreicher `ReadGate`-Prüfung.
  - Keine Umgehung der Mandantenisolation (`TenantPolicy` ist Pflicht, kein Optional-Modus; D-07).

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.12)

| Modul | Inhalt & Verantwortung |
|---|---|
| `wiring` | Assemblierung des Laufzeitgraphen (`EngineBuilder::open`), Verwaltung von Hintergrundtasks, geordnetes Beenden via `close()`. |
| `collection` | `Collection`: `insert`, `insert_batch`, `get`, `get_at`, `scan`, `scan_at`, `delete` (logisch), Suchmethoden. Verwaltet asynchrone Dokumentlocks über die **gesamte** Operation (behebt Race-Befund B-13). |
| `doc` | Zerlegung von Dokumenten in atomare Records (`Doc`, `DocMap`, `Manifest`, `Locator`) gemäß SPEC §9.3; Envelope-Sealing. |
| `search` | Suchpfade: `search_text`, `search_vector`, `search_hybrid` mit zwingender Vorab-Filterung durch `ReadGate`, RRF-Fusion und nachgelagertem Metadatenfilter. |
| `transaction` | `DbTransaction`: Koordinierte atomare Transaktion (Store-Transaktion, Tresor-Slots, Puffer-Vormerkungen). |
| `recovery` | Wiederanlauf und Konsistenzabgleich abgeleiteter Indizes aus verbleibenden WAL-Frames (`derived_durable_through`). |
| `erase_bridge` | Verdrahtet `contextra-erasure` mit den `ErasureLayer`-Instanzen (`LsmStorage`, `TextIndex`, `VectorIndex`) und exponiert `erase`, `erasure_status` und `verify`. |
| `metrics_names` | Typisierte Metriknamen ohne personenbezogene IDs in Labels. |

> [!IMPORTANT]
> **Bereinigung veralteter Module (SPEC §8.12):**
> Die Module `chunker`, `extraction`, `export`, `import`, `decay_controller`, `temporal_filter`, `merge_operator_adapter`, alle 103 Verwendungen von `contextra_graph`, `community_detection`, `checkpoint`, `privacy` und `license` werden restlos entfernt.

## 4. Kritische Invarianten
1. **Atomares Insert (SPEC §8.12 Invariante a):**
   - Ein `insert` schreibt `Doc`, `DocMap`, `Manifest` und `Locator` in **einer einzigen** transaktionalen Batch-Operation im Store.
   - Der zugehörige Tresor-Slot in `erasure` muss vor dem Commit dauerhaft gesichert sein (SPEC §11.5).
2. **ReadGate vor Entschlüsselung:** Sämtliche Lese- und Suchpfade (`get`, `scan`, `search_*`) konsultieren das `ReadGate` **vor** jeglicher AEAD-Entschlüsselung. Ist das Ziel gegatet, wird sofort `Erased` zurückgegeben.
3. **Dokumentlock-Disziplin (B-13 Fix):** Lese-, Schreib-, Lösch- (`delete`) und Bereinigungsvorgänge (`erase`) auf demselben Dokument teilen denselben asynchronen Dokument-Lock. Das Lock wird während der gesamten Dauer gehalten.

## 5. Crate-Arbeitspakete zum SOLL-Zustand (SPEC §15.4)

- **[ ] WP-1.1: Konsumenten entkoppeln**
  - Rückbau aller Graph-, NLP-, Privacy- und Checkpoint-Abhängigkeiten in der Engine.
- **[ ] WP-1.2: Veraltete Engine-Module & Features entfernen**
  - Bereinigung toter Module und Features (`graph-connectivity-health`, `sandbox`, `onnx`, etc.).
- **[ ] WP-2.5: Mandantenpflicht durchsetzen (D-07)**
  - Entfernung von `TenantPolicy::Optional`; Mandant ist ausnahmslos Pflicht.
- **[ ] WP-3.3: Dokumentenzerlegung & Manifest pro Dokument**
  - Implementierung von `Doc`, `DocMap`, `Manifest`, `Locator` und Envelope-Codec.
- **[ ] WP-3.4: Atomares Lock für Insert/Update/Delete (B-13)**
  - Asynchroner Document-Lock über die gesamte CRUD-Operation gegen Race-Conditions.
- **[ ] WP-3.6: ReadGate in allen Lese- und Suchpfaden verdrahten**
  - Fail-closed Gate-Prüfung vor Datenabruf.
- **[ ] WP-3.10: Wiederanlauf der abgeleiteten Indizes (`recovery`)**
  - Synchronisation von Text- und Vektorindizes aus WAL-Segmenten beim Start.
- **[ ] WP-3.11: Erase-Bridge implementieren**
  - Anbindung von `ErasureCore` an die Engine-Schichten.

## 6. Verifikation & Benchmarks
- **T-ISO-1..4:** Vollständige Mandantenisolation über alle Operationen hinweg.
- **T-VER-1:** Konsistente Versionierung und Snapshot-Isolation bei parallelen Zugriffen.
- **T-CONC-1:** Race-Free Lock-Verhalten bei konkurrierenden Writes und Deletes.
- **Benchmarks:** B-HYBRID (Durchsatz und Latenz hybrider Suchen), B-E2E-INSERT.
