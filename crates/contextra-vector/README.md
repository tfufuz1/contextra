# contextra-vector

Vektorsuche und Indexierung (HNSW, SQ8 Quantisierung).

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, §9.2, §9.3, `capabilities.toml`).

## Rolle & Zweck
`contextra-vector` ist der autarke Vektorindex-Baustein des Contextra-Ökosystems. Er bietet speichereffiziente Vektorsuche und Indexierung (Hierarchical Navigable Small World, HNSW) mit skalarer Quantisierung (SQ8) und Cosine-/Dot-Produkt-Distanzen.

Der Baustein kann isoliert in beliebigen Rust-Systemen als eigenständige Vektor-Engine betrieben werden.

## Baukasten-Einordnung & Status
- **Ring:** Ring 0 (Vektorindexierung)
- **Status:** 🟡 In Migration (Bereinigung von Löschketten-Relikten gemäß `PLAN.md` Phase 4)
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`
- **Erlaubte Abhängigkeiten:** `contextra-types`, `contextra-ports`, `contextra-simd`.

## Deprecated Debt & Entkopplung
- **Löschketten-Verdrahtung:** Historische Nachweis- und Deletion-Proof-Logiken sind als *Deprecated Debt* (siehe [`DEPRECATED_DEBT.md`](../../DEPRECATED_DEBT.md)) deklariert.
- **Zielarchitektur:** `contextra-vector` beschränkt sich auf Mandantenisolation und Tombstone-Pruning via `LayerScanner`/`Purger`-Ports. Sämtliche Nachweis- und Signaturaufgaben liegen in [`contextra-erasure`](../contextra-erasure/).
