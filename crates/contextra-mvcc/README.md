# contextra-mvcc

Multi-Version Concurrency Control, Sequenzsemantik, Snapshot-Isolation und Snapshot-Widerruf.

- **Ring:** Ring 2 (Transaktionsisolation)
- **Normativ:** [docs/SPEC.md §8.7](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-mvcc` stellt deterministische Transaktionsisolation und MVCC bereit:
- `seq`: Monotone Sequenzen `Seq(NonZeroU64)` ohne Sentinel `u64::MAX`.
- `snapshot_registry`: Snapshot-Registrierung, `min_active()`, Snapshot-Widerruf (`revoke_covering(&Matcher)`).
- `ssi`: `SsiValidator` für Serializable Snapshot Isolation ohne Write-Skew.
- `tx_buffer`: Transaktionspuffer zur Isolation uncommitteter Änderungen.

## Invarianten
- `#![forbid(unsafe_code)]`
- `prune_through` mit gesetztem Tombstone-Bit ist ein Fehler, kein stilles Maskieren (D-C02 Bugfix).
- Snapshot-Widerruf (`revoke_covering`): Snapshots, die gelöschte Sequenzen sehen könnten, werden als widerrufen markiert; Folgeoperationen scheitern fail-closed mit `SnapshotRevoked`.
