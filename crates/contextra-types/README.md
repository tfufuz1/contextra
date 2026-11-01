# contextra-types

Kanonische Typdefinitionen, Identifikatoren, Schlüssel-Buffer, Fehler und Löschtypen.

- **Ring:** Ring 0 (Typen-Kern, deterministisch, I/O-frei, kein `tokio`)
- **Normativ:** [docs/SPEC.md §8.1](../../docs/SPEC.md)
- **Arbeitsanweisungen:** [AGENTS.md](AGENTS.md)

## Rolle & Zweck
`contextra-types` definiert die reinen, zustandslosen, I/O- und runtime-unabhängigen Domänentypen für das Gesamtsystem Contextra:
- `ids`: `TenantId`, `CollectionName`, `DocKey`, `SubjectId`, blinde Tags (`TenantTag`, `CollTag`, `ClusterTag`, `DocTag`, `TermTag`), `Seq(NonZeroU64)`, `TxId`, `ErasureId`, `SegmentId`, `Slot`.
- `key`: `Region`, `Family`, `KeyBuf`, `KeyRange`, `Matcher` (Löschcluster-Schlüsselraum SPEC §9.1).
- `erasure`: `EraseTarget`, `ErasureState`, `LayerId`, `LayerStatus`, `ErasureClass`, `ResidualRisk`, `ErasureReport`, `RetentionAttestation`, `NeedleSet`, `LayerScan`, `Evidence`.
- `error`: `ContextraError`.
- `secret`: `SecretBytes`, `SecretKey32` (`Zeroizing`).
- `limits`: Normative Obergrenzen aus SPEC §14.6.

## Invarianten
- `#![forbid(unsafe_code)]`
- Kein `async`, kein I/O, keine Zeit- oder Zufallsabfragen.
- Alle Tags implementieren kein `Display`, nur gekürztes Hex in `Debug`.
- `Seq` hat keinen Sentinel (`u64::MAX`).
