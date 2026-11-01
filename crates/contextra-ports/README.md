# contextra-ports

Abstrakte Port-Schnittstellen und Trait-Definitionen für deterministische Entkopplung.

- **Ring:** Ring 0 (Port-Fundament)
- **Normativ:** [docs/SPEC.md §8.2](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-ports` definiert sämtliche abstrakten Port-Traits, über die die Ringe 1 bis 3 gekoppelt sind:
- `StorageEngine`: Asynchroner Zugriff auf Key-Value-Speicher auf `KeyBuf`/`Bytes`.
- `ErasureLayer`: Einheitlicher Vertrag für löschbare Schichten (`plan`, `purge`, `scan_residue`, `positive_control`).
- `ReadGate`: Fail-closed Lesesperre (`blocked`, `blocked_range`).
- `RecordSealer`: Lokales Envelope-Sealing und Öffnen via Tresor.
- `LedgerSink`: Append-only Sink für Restore-Guard-Ledger.
- `Vfs` / `VfsFile`: Konsolidierte virtuelle Dateisystemschnittstelle.
- `Clock`, `Rng`, `IdGen`, `Metrics`: Ports für strikten Determinismus (P28).

## Invarianten
- `#![forbid(unsafe_code)]`
- Keine konkreten Produktions-Implementierungen (nur Conformance-Suiten).
- Conformance-Tests in `ports::contract_tests` laufen gegen jede Implementierung.
