# contextra-engine

Orchestrierung der Teilsysteme, Collections, Transaktionen, Hybridsuche und Erase-Bridge.

- **Ring:** Ring 3 (Orchestrierung)
- **Normativ:** [docs/SPEC.md §8.12](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-engine` verdrahtet die Speicher- und Indexschichten zu einem kohärenten Ganzen, besitzt jedoch **keine eigene Löschlogik** (diese ist an `contextra-erasure` und die `ErasureLayer` delegiert):
- `wiring`: Assemblierung des Laufzeitgraphen, Verwaltung von Hintergrundtasks, geordnetes `close()`.
- `collection`: `Collection` mit `insert`, `get`, `scan`, `soft_delete` (D-51), `search_hybrid`.
- `doc`: Zerlegung von Dokumenten in Records (Record-Envelope v2 mit `cluster_tag`, D-70) gemäß SPEC §9.3.
- `search`: Vorab-Filterung durch `ReadGate`, RRF-Fusion und nachgelagerter Metadatenfilter.
- `transaction`: Atomare Transaktion über Store, Tresor und Indizes.
- `recovery`: Wiederanlauf der abgeleiteten Indizes aus WAL-Segmenten (`derived_durable_through`).
- `erase_bridge`: Anbindung von `ErasureCore` an die `ErasureLayer`-Instanzen.

## Invarianten
- `#![forbid(unsafe_code)]`
- Atomares Insert: Record-Envelopes werden in einer einzigen Store-Transaktion geschrieben; Tresor-Slot ist vorher dauerhaft.
- ReadGate vor Entschlüsselung: Alle Lese- und Suchpfade prüfen das `ReadGate`, bevor Daten entschlüsselt werden.
- Asynchroner Dokument-Lock über die gesamte Operation (behebt Race-Befund B-13 zwischen Insert, Delete und Erase).
- Mandantentrennung ist verpflichtend (`TenantPolicy::Required`, D-07).
- Veraltete Module (Graph, Extraction, Checkpoint, Privacy, License) sind vollständig entfernt.
