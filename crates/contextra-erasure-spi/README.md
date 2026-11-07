# contextra-erasure-spi

Service Provider Interface (SPI) für die modulare physische Bereinigung (`Purger`) und Restdatenmessung (`LayerScanner`) im Contextra-Baukastensystem.

## Zweck & Architekturrolle (Entscheidung D-E1)

Im Zuge der Transformation zum modularen Baukasten kapselt `contextra-erasure-spi` alle Schnittstellen und Datenstrukturen zur schichtbezogenen physischen Bereinigung und Verifikation:
- **Ring 0 (SPI / Foundation):** Hängt ausschließlich an `contextra-types`.
- **Entkopplung der Datenhalter:** Datenhaltende Crates (`contextra-store`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-mvcc`) hängen **nicht** an `contextra-erasure-spi`.
- **Satelliten-Architektur (`contextra-erasure-<schicht>`):** Satelliten implementieren `Purger` und `LayerScanner` für eine konkrete Schicht und rufen hierfür die generische, neutrale Wartungs-API der Schicht (Kompaktion, Rewrite, Rebuild, Slot-Zeroing) auf.

## Invarianten und Garantien

- **INV-WITNESS-MINT-1:** Ein `LayerWitness` kann ausschließlich über einen kontrollierten Scanpfad (`ScanSession::finish`) ausgestellt werden.
- **Positivkontrolle:** Für jede aktive Messung ist der Nachweis einer funktionierenden Positivkontrolle (Canary Probe) zwingend erforderlich (`control_matches > 0`).
- **Messung vor Behauptung:** `scanned > 0` ist für aktive Techniken verpflichtend.
- **Zero-Panic-Doktrin:** Im gesamten Produktionscode gibt es keine Panics, Unwraps oder ungeprüften Indexierungen (`#![forbid(unsafe_code)]`).
- **Deterministischer Digest:** Der BLAKE3-Digest wird unabhängig von der Einfügereihenfolge der Einheiten deterministisch berechnet.

## Schnittstellen

- `Purger`: Führt physische Bereinigungen für einen gegebenen `KeyScope` aus und meldet `PurgeStats`.
- `LayerScanner`: Führt die Restdatenmessung einer Schicht aus und erzeugt bei erfolgreicher Messung einen fälschungssicheren `LayerWitness`.
- `KeyScope`: Typisierte Bereichsdeskriptoren (`Tenant`, `Collection`, `Document`, `Prefix`).
- `WitnessLayer`: Aufzählung der unterstützten physischen Schichten (`LsmMemtable`, `SsTableAllLevels`, `HnswIndex`, `WalAllSegments`, `CsrGraph`, `KvCacheSegments`, `TextIndex`, `EmbeddingCache`, `Checkpoints`, `Custom`).

## Minimalbeispiel

Ein eigenständiges, lauffähiges Beispiel findet sich unter `examples/spi_standalone.rs`:

```bash
cargo run -p contextra-erasure-spi --example spi_standalone
```
