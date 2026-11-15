# Archiv: Extraktion nach Ziel-Crate

Alle Dateien sind nach der **Ziel-Crate der Zielarchitektur** sortiert (`archive/<ziel>/<altquelle>/…`). Namen und Importe sind auf Acta umgeschrieben; Alt-Kennungen und Verweise auf frueheren Spezifikationsstand sind entfernt. Das Archiv ist **kein** Teil des Cargo-Workspace: es wird Crate fuer Crate in `crates/<ziel>/src` ueberfuehrt und danach geloescht.

| Ziel | Level | Dateien | LOC | Budget | Altquellen ausserhalb der Whitelist |
|---|---:|---:|---:|---:|---|
| [`acta-base`](acta-base/MANIFEST.md) | L0 | 10 | 1752 | 2500 | blake3 |
| [`acta-crypto`](acta-crypto/MANIFEST.md) | L1 | 26 | 5690 | 4200 | aes-gcm-siv, bincode, blake3, rand |
| [`acta-fs`](acta-fs/MANIFEST.md) | L1 | 28 | 5025 | 2500 | crc32fast, parking_lot |
| [`acta-sys`](acta-sys/MANIFEST.md) | L1 | 35 | 9931 | 1600 | memmap2, tracing |
| [`acta-gate`](acta-gate/MANIFEST.md) | L1 | 8 | 1884 | 1200 | parking_lot |
| [`acta-infer`](acta-infer/MANIFEST.md) | L1 | 5 | 448 | 700 |  |
| [`acta-pii`](acta-pii/MANIFEST.md) | L1 | 10 | 3012 | 4000 | tracing |
| [`acta-ingest`](acta-ingest/MANIFEST.md) | L1 | 44 | 11235 | 3000 | blake3 |
| [`acta-parse`](acta-parse/MANIFEST.md) | L1 | 112 | 21329 | 5000 | blake3, calamine, cfb, csv, encoding_rs, html5ever, lopdf, mail-parser, quick-xml, toml, yaml-rust2, zip |
| [`acta-ledger`](acta-ledger/MANIFEST.md) | L2 | 10 | 1855 | 2800 | bincode, blake3, futures-util, parking_lot |
| [`acta-vault`](acta-vault/MANIFEST.md) | L2 | 14 | 5274 | 5500 | aes-gcm-siv, ahash, bincode, blake3, futures-util, parking_lot, rand |
| [`acta-seg`](acta-seg/MANIFEST.md) | L2 | 133 | 29356 | 6500 | ahash, blake3, bytes, crc32fast, lru, parking_lot, quick_cache, rand, tracing |
| [`acta-history`](acta-history/MANIFEST.md) | L3 | 16 | 6499 | 6500 | ahash, blake3, bytes, parking_lot, tracing |
| [`acta-text`](acta-text/MANIFEST.md) | L3 | 47 | 10850 | 5000 | bincode, blake3, bytes, futures, itoa, parking_lot, tracing |
| [`acta-vec`](acta-vec/MANIFEST.md) | L3 | 40 | 5645 | 4000 | ahash, blake3, parking_lot, tracing |
| [`acta-erasure`](acta-erasure/MANIFEST.md) | L4 | 58 | 12015 | 9000 | bincode, blake3, futures-util, parking_lot, tracing |
| [`acta-decision`](acta-decision/MANIFEST.md) | L4 | 1 | 466 | 4500 | ahash |
| [`acta-rag`](acta-rag/MANIFEST.md) | L4 | 35 | 5837 | 7000 | ahash, tracing |
| [`acta-reports`](acta-reports/MANIFEST.md) | L5 | 31 | 5272 | 3500 | blake3, futures-util |
| [`acta-engine`](acta-engine/MANIFEST.md) | L6 | 19 | 7225 | 12000 | blake3, bytes, parking_lot, rand, scc, tokio-util, tracing |
| [`acta-node`](acta-node/MANIFEST.md) | L7 | 10 | 2358 | 3000 | bincode, blake3, tracing |
| [`acta-infer-http`](acta-infer-http/MANIFEST.md) | L8 | 13 | 2872 | 3000 | futures-util, parking_lot, reqwest, tracing |
| [`acta-api`](acta-api/MANIFEST.md) | L8 | 1 | 739 | 6000 |  |
| [`acta-parse-worker`](acta-parse-worker/MANIFEST.md) | L9 | 11 | 2252 | 600 | anyhow, bytes, flatbuffers, tracing, wasmtime |
| [`acta-verify`](acta-verify/MANIFEST.md) | L9 | 6 | 3538 | 1200 | bincode, blake3, rand |
| [`acta-testkit`](acta-testkit/MANIFEST.md) |  | 13 | 2415 |  | bytes, parking_lot |
| [`_reserve/ann-backend`](_reserve/ann-backend/MANIFEST.md) |  | 101 | 17494 |  | ahash, blake3, memmap2, parking_lot, rand, tracing |

**Summe:** 837 Dateien, 182268 Zeilen. Zusaetzlich `xtask/` (19 Werkzeuge) und [NICHT_EXTRAHIERT.md](NICHT_EXTRAHIERT.md).

## Regeln fuer die Ueberfuehrung

1. Level-weise von unten nach oben (L0 → L9); jedes Crate gruen (`cargo check`, `cargo test`, `cargo xtask gates`), bevor das naechste beginnt.
2. Marker `TODO(acta-hr3)`: Import stammt aus einem aufgeloesten Sammel-Crate; Typ dem Crate zuordnen, das ihn **braucht** (HR-2/HR-3).
3. Befund-Spalten der Manifeste abarbeiten (Festkomma statt Gleitkomma, Plattenformate ohne `serde`, kein `tokio` in L0–L7, `unsafe` nur in `acta-sys`, kein `std::fs` ausserhalb `acta-fs`).
4. Golden-Dateien der Altquelle tragen alte Magic-Bytes/Domaenenpraefixe: neu erzeugen, nicht uebernehmen. Domaenenpraefixe nur aus Kap. 4.1.
5. Tests sind Orakel-Material (RFC-Vektoren, Absturz-, Eigenschafts- und Fuzz-Tests). Jede Zusage Z1–Z19 braucht danach einen gruenen Test (G-ZUSAGE).
6. Wenn ein Crate vollstaendig ueberfuehrt ist: Ordner `archive/<ziel>/` loeschen.
