# contextra-erasure

Entkoppelter Löschkern, Saga-Zustandsmaschine, Journal, Tresor, Messung und Bericht.

- **Ring:** Ring 3 (Löschkern)
- **Normativ:** [docs/SPEC.md §8.9](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-erasure` ist der zentrale, entkoppelte Löschkern von Contextra. Er kennt **keine Ring-2-Crates** (Kopplung ausschließlich über `contextra-ports`):
- `scope`: Auflösung `EraseTarget` → `Matcher` und `doc_tag`-Menge.
- `journal`: Append-only `erasure/journal.log` mit Hash-Kette und AEAD-Verschlüsselung (`K_journal`).
- `erased_set`: `erasure/erased.set` verwaltet aktive Gates und implementiert `ReadGate`.
- `vault`: Daten-Tresor (`vault/vault.dat`) für DEK-Slots; implementiert `RecordSealer`.
- `saga`: Deterministische Lösch-Saga: `Requested → Gated → Drained → Rewritten → Cleaned → Measured → Completed`.
- `measure`: Koordiniert E1/E2/E3-Messungen über `ErasureLayer::scan_residue` und Positivkontrollen (`positive_control`).
- `classify`: Berechnet die kanonische `ErasureClass` (G/K/P-Verband) aus den Zeugen.
- `report`: Erstellt und signiert den Löschbericht `ctx-erasure-report/1` via Ed25519.
- `ledger`: Persistiert abgeschlossene Löschungen für den Restore-Guard (`erasure/ledger.log`).
- `recover`: Stellt unvollständige Löschsagas nach Systemabstürzen wieder her.

## Invarianten
- `#![forbid(unsafe_code)]`
- Kennt keine Ring-2-Crates (`store`, `text`, `vector`, `mvcc`).
- Unmittelbares Gating: Sobald `Gated` persistiert ist, blockiert `ErasedSet` fail-closed alle Lese- und Suchpfade.
- Messen vor Behaupten: Berichte werden erst nach verifizierten E1/E2/E3-Scans ausgestellt.
