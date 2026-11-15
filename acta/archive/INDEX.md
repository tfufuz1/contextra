# Archiv: Extraktion nach Ziel-Crate

Alle Dateien sind nach der **Ziel-Crate der Zielarchitektur** sortiert (`archive/<ziel>/<altquelle>/…`). Namen und Importe sind auf Acta umgeschrieben; Alt-Kennungen und Verweise auf frueheren Spezifikationsstand sind entfernt. Das Archiv ist **kein** Teil des Cargo-Workspace: es wird Crate fuer Crate in `crates/<ziel>/src` ueberfuehrt und danach geloescht.

| Ziel | Level | Dateien | LOC | Budget | Altquellen ausserhalb der Whitelist |
|---|---:|---:|---:|---:|---|
| [`acta-base`](acta-base/MANIFEST.md) | L0 | 13 | 3725 | 2500 | blake3 |
| [`acta-crypto`](acta-crypto/MANIFEST.md) | L1 | 28 | 5955 | 4200 | aes-gcm-siv, bincode, blake3, rand |
| [`acta-fs`](acta-fs/MANIFEST.md) | L1 | 29 | 5400 | 2500 | crc32fast, parking_lot |
| [`acta-sys`](acta-sys/MANIFEST.md) | L1 | 35 | 9896 | 1600 | memmap2, tracing |
| [`acta-gate`](acta-gate/MANIFEST.md) | L1 | 12 | 2497 | 1200 | parking_lot |
| [`acta-access`](acta-access/MANIFEST.md) | L1 | 4 | 748 | 3000 | |
| [`acta-infer`](acta-infer/MANIFEST.md) | L1 | 6 | 1152 | 700 |  |
| [`acta-pii`](acta-pii/MANIFEST.md) | L1 | 11 | 3400 | 4000 | tracing |
| [`acta-ingest`](acta-ingest/MANIFEST.md) | L1 | 45 | 11353 | 3000 | blake3 |
| [`acta-parse`](acta-parse/MANIFEST.md) | L1 | 126 | 21530 | 5000 | blake3, calamine, cfb, csv, encoding_rs, html5ever, lopdf, mail-parser, quick-xml, toml, yaml-rust2, zip |
| [`acta-ledger`](acta-ledger/MANIFEST.md) | L2 | 16 | 3268 | 2800 | bincode, blake3, futures-util, parking_lot |
| [`acta-vault`](acta-vault/MANIFEST.md) | L2 | 21 | 6797 | 5500 | aes-gcm-siv, ahash, bincode, blake3, futures-util, parking_lot, rand |
| [`acta-seg`](acta-seg/MANIFEST.md) | L2 | 144 | 29896 | 6500 | ahash, blake3, bytes, crc32fast, lru, parking_lot, quick_cache, rand, tracing |
| [`acta-history`](acta-history/MANIFEST.md) | L3 | 39 | 12629 | 6500 | ahash, blake3, bytes, parking_lot, tracing |
| [`acta-text`](acta-text/MANIFEST.md) | L3 | 60 | 14313 | 5000 | bincode, blake3, bytes, futures, itoa, parking_lot, tracing |
| [`acta-vec`](acta-vec/MANIFEST.md) | L3 | 39 | 6413 | 4000 | ahash, blake3, parking_lot, tracing |
| [`acta-erasure`](acta-erasure/MANIFEST.md) | L4 | 76 | 16365 | 9000 | bincode, blake3, futures-util, parking_lot, tracing |
| [`acta-decision`](acta-decision/MANIFEST.md) | L4 | 1 | 465 | 4500 | ahash |
| [`acta-rag`](acta-rag/MANIFEST.md) | L4 | 54 | 9797 | 7000 | ahash, tracing |
| [`acta-reports`](acta-reports/MANIFEST.md) | L5 | 33 | 5566 | 3500 | blake3, futures-util |
| [`acta-engine`](acta-engine/MANIFEST.md) | L6 | 22 | 8139 | 12000 | blake3, bytes, parking_lot, rand, scc, tokio-util, tracing |
| [`acta-node`](acta-node/MANIFEST.md) | L7 | 16 | 3425 | 3000 | bincode, blake3, tracing |
| [`acta-infer-http`](acta-infer-http/MANIFEST.md) | L8 | 17 | 5078 | 3000 | futures-util, parking_lot, reqwest, tracing |
| [`acta-api`](acta-api/MANIFEST.md) | L8 | 2 | 1337 | 6000 |  |
| [`acta-parse-worker`](acta-parse-worker/MANIFEST.md) | L9 | 11 | 2241 | 600 | anyhow, bytes, flatbuffers, tracing, wasmtime |
| [`acta-verify`](acta-verify/MANIFEST.md) | L9 | 7 | 3637 | 1200 | bincode, blake3, rand |
| [`acta-testkit`](acta-testkit/MANIFEST.md) |  | 16 | 4235 |  | bytes, parking_lot |
| [`acta-graph`](acta-graph/MANIFEST.md) | L3 (ADR) | 89 | 19801 | 4000 (Vorschlag) | ahash, arc-swap, bincode, blake3, bytes, parking_lot, scc, tracing |
| [`_reserve/graph-ext`](_reserve/graph-ext/MANIFEST.md) |  | 52 | 13188 |  | wie `acta-graph` |
| [`_reserve/ann-backend`](_reserve/ann-backend/MANIFEST.md) |  | 103 | 17980 |  | ahash, blake3, memmap2, parking_lot, rand, tracing |

**Summe:** 1127 Dateien, 250226 Zeilen. Zusaetzlich `xtask/` (19 Werkzeuge), [NICHT_EXTRAHIERT.md](NICHT_EXTRAHIERT.md) und [ALTLASTEN.md](ALTLASTEN.md).

## Regeln fuer die Ueberfuehrung

1. Level-weise von unten nach oben (L0 → L9); jedes Crate gruen (`cargo check`, `cargo test`, `cargo xtask gates`), bevor das naechste beginnt.
2. Marker `TODO(acta-hr3)`: Import stammt aus einem aufgeloesten Sammel-Crate; Typ dem Crate zuordnen, das ihn **braucht** (HR-2/HR-3).
3. Befund-Spalten der Manifeste abarbeiten (Festkomma statt Gleitkomma, Plattenformate ohne `serde`, kein `tokio` in L0–L7, `unsafe` nur in `acta-sys`, kein `std::fs` ausserhalb `acta-fs`).
4. Golden-Dateien der Altquelle tragen alte Magic-Bytes/Domaenenpraefixe: neu erzeugen, nicht uebernehmen. Domaenenpraefixe nur aus Kap. 4.1.
5. Tests sind Orakel-Material (RFC-Vektoren, Absturz-, Eigenschafts- und Fuzz-Tests). Jede Zusage Z1–Z19 braucht danach einen gruenen Test (G-ZUSAGE).
6. Marker `TODO(acta-altlast)` und die Punkte in [ALTLASTEN.md](ALTLASTEN.md) vor dem Gruenstellen des Crates abarbeiten.
7. Wenn ein Crate vollstaendig ueberfuehrt ist: Ordner `archive/<ziel>/` loeschen.
