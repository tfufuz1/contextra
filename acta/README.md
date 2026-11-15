# Acta

Wissens- und Entscheidungsplattform mit versionierbarem Unternehmenswissen und entkoppeltem, nachweisbar loeschbarem Personendatenschutz.
Normative Spezifikation: [`docs/SPEC.md`](docs/SPEC.md). Rechtliche Hintergrundanalysen: `docs/dsgvo/`.

## Stand
- Workspace mit **30 Produktions- und 2 Entwicklungs-Crates** nach Kap. 3.3 (Gerueste, Levelgrenzen in `rings.toml`).
- Echte Grundbausteine: `acta-base` (Ids, Ordinale, `DataClass`, `ErrorClass`, Ports, Festkomma `Fx`), `acta-testkit` (`ManualClock`, `SeededRng`, `FailingRng`), `xtask` (Gates).
- Alle uebrigen Crates sind Gerueste mit Fehlertyp (HR-4); ihre Implementierungen liegen **sortiert im Archiv** (`archive/`).

## Arbeiten

```
cargo check --workspace --all-targets
cargo xtask gates      # G1, G-DEF, G-HUB, G-UNSAFE, G-NOTOKIO, G-LINT, G-WORD
```

Toolchain: Rust 1.89.0 (`rust-toolchain.toml`).

## Ueberfuehrung aus dem Archiv

Siehe [`archive/INDEX.md`](archive/INDEX.md). Empfohlene Reihenfolge (jede Zeile ist ein Arbeitspaket; Abhaengigkeiten nur nach unten):

| Paket | Crates | Schwerpunkt der Umstellung |
|---|---|---|
| 1 | `acta-base`, `acta-crypto`, `acta-fs`, `acta-sys` | fallibler Zufall, Umschlag v1 (Nonce intern, Padding), `FaultVfs`, `SecureBuf`, SIMD-Kerne nach `acta-sys` |
| 2 | `acta-gate`, `acta-ledger`, `acta-vault` | Roaring-Gate + HMAC-Log, K/E/A-Ledger, Tresor 92 B mit Slot-AAD, Wurzelrotation, KeyAnchor |
| 3 | `acta-seg` | Journal-Rahmen und Segmentformat nach 5.1, Manifest, kein LSM |
| 4 | `acta-text`, `acta-vec` | Festkomma-BM25 (Z7), i8-Flat-Suche, Lineage `Derived<T>` |
| 4b | `acta-graph` (nur nach ADR, siehe `docs/ADR-VORSCHLAG-graph.md`) | CSR/PPR/Pfad-RAG aus `archive/acta-graph/`; Erweiterungen in `archive/_reserve/graph-ext/`; Festkomma, Zugriffsfilter, `ErasureLayer` |
| 5 | `acta-ingest`, `acta-parse`, `acta-parse-worker`, `acta-pii` | Chunking `chunk_cfg` v1, Worker-Sandbox (Landlock/seccomp), `Validated<T>` |
| 6 | `acta-erasure`, `acta-reports`, `acta-verify` | Saga S0–S9 synchron, Messung mit Positivkontrolle, JCS-Bericht, unabhaengiger Pruefer |
| 7 | `acta-pstore`, `acta-history`, `acta-access`, `acta-decision`, `acta-rag` | Neubauanteile mit Referenzcode aus dem Archiv |
| 8 | `acta-engine`, `acta-node`, `acta-infer-http`, `acta-sink-net`, `acta-api`, Binaries | Verdrahtung, Egress-Schicht, Lizenzdatei, HTTP/JSON |

## Klarstellungen zur Zielarchitektur
- `acta-engine` haengt **nicht** von `acta-parse` ab (Parser-Abhaengigkeiten nur in `acta-parse` und `acta-parse-worker`, 3.8; Parsing laeuft im Worker, 13.2). Die Tabelle in 3.3 ("alle darunter") ist so zu lesen.
- `acta-api` fuehrt zusaetzlich `acta-base` (fuer `ClassifiedError`), weil Re-Exports verboten sind (HR-6).
- `acta-testkit` liegt in `rings.toml` auf Level 2; Dev-Abhaengigkeiten zaehlen fuer G1 nicht.
- Gate G-HUB zaehlt Produktionszeilen ohne Leer-/Kommentarzeilen und ohne `#[cfg(test)]`-Bloecke.
