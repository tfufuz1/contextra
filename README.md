# Contextra

> **Passphrase.** Contextra hat keinen Schlüssel-Escrow. Ist die Passphrase verloren, sind die Daten nicht wiederherstellbar. Auch der Hersteller kann sie nicht zurückholen (SPEC HR-3).

## Außenaussage

> **DE:** Contextra entfernt die Daten eines Löschziels aus dem Datenverzeichnis, aus den Indexen und aus den gespeicherten Zeitreise-Versionen und weist den Zustand je Schicht messtechnisch nach. Für jedes Ziel wird zusätzlich der zugehörige Schlüssel aus dem Schlüsseltresor entfernt; ob und wie das rechtlich zu werten ist, entscheidet der Betreiber. Nicht erfasst sind externe Sicherungen, Flash-Remanenz, Dateisystem-Journale, Prozessspeicher und externe Verarbeiter. Der Bericht listet diese Restrisiken.
>
> **EN:** Contextra removes the data of an erasure target from the data directory, from its indexes and from stored time-travel versions, and provides per-layer, measured evidence of the resulting state. For each target the associated key is additionally removed from the key vault; whether and how this is to be assessed legally is for the operator to decide. Not covered are external backups, flash remanence, file-system journals, process memory and external processors. The report lists these residual risks.

Dieser Wortlaut ist die einzige zulässige Außenaussage zum Löschen (SPEC 6.4).

## Zweck

Eingebettete Rust-Bibliothek (Single-Node, kein Server) für die lokale Speicherung von Wissen und Gedächtnis von Agenten- und RAG-Systemen: Dokumente, Volltextsuche (BM25), Vektorsuche (HNSW), Hybridsuche (RRF) und Zeitreisen über MVCC-Versionen.

Kurzformel: „SQLite für Agenten, die gezielt und nachweisbar löschen können.“

## Status

Entwicklungsstand, **kein Release**. Die Spezifikation [`docs/SPEC.md`](docs/SPEC.md) (Version 4.2.0) beschreibt den Sollzustand für Version 1.0.0. Derzeit baut der Workspace nicht, und der Löschkern ist noch nicht implementiert. Die Bibliothek ist nicht für den produktiven Einsatz freigegeben.

## Löschen

- `erase(Ziel)` ist ein eigener Vorgang. Er liefert einen signierten Bericht, der je Schicht ausweist, was entfernt, gemessen und nicht erfasst wurde.
- `Collection::soft_delete` setzt einen logischen Tombstone und ist **keine** Löschaussage. Für Löschpflichten `erase` verwenden (SPEC D-51).
- Versionen werden nach dem Ablauf von `history_horizon` durch die Kompaktion physisch entfernt. Das ist keine Löschaussage.

## Grenzen

- **Plattformen:** Linux x86_64/aarch64 auf ext4 oder XFS (Tier 1); macOS mit APFS sowie Linux mit btrfs oder ZFS (Tier 2). Windows, NFS, SMB, FUSE und overlayfs werden nicht unterstützt. Die Startprobe verweigert den Start auf nicht unterstützten Dateisystemen (SPEC D-56).
- **Netzwerk:** Keine Netzwerkfunktion, keine Telemetrie (SPEC HR-1, HR-2).
- **Nicht in v1:** Server, Sharding, Replikation, MCP-Server, Python-Bindings, Erzeugung von Embeddings, LLM-Aufrufe (SPEC 1.4).
- **Restrisiken:** Die Liste mit Bedingungen steht in SPEC Anhang D und im Bericht jedes Löschvorgangs.

Keine Rechtsberatung. Die rechtliche Wertung obliegt dem Betreiber.

## Dokumentation

- [`docs/SPEC.md`](docs/SPEC.md): normative Spezifikation
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md), [`docs/TYPE_REGISTRY.md`](docs/TYPE_REGISTRY.md), [`docs/VETOES.md`](docs/VETOES.md)
- [`docs/COMPILATION.md`](docs/COMPILATION.md), [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md), [`docs/SECURITY.md`](docs/SECURITY.md)

## Lizenz

MIT OR Apache-2.0. Siehe [`LICENSE-MIT`](LICENSE-MIT) und [`LICENSE-APACHE`](LICENSE-APACHE).
