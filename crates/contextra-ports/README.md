# contextra-ports

`contextra-ports` definiert die abstrakten Trait-Interfaces für alle Contextra-Subsysteme (Ring 0).

## Zweck

Entkoppelt Ring 0/1/2/3-Komponenten durch `dyn`-kompatible Trait-Schnittstellen (Storage, VectorIndex, TextIndex, GraphIndex, Embedder, Clock, Rng, MetricsSink).

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Synchroner Ports-Kern)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Futures/Streams:** `BoxFuture`, `BoxStream`
- **Port Submodule:** `storage`, `vector_index`, `text_index`, `graph_index`, `embedding`, `checkpoint`, `lifecycle`, `observability`

## Architektur & Verweise

Details zur Entkopplung (P27) finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §3 und §4.2.
