# MemFuse Python Bindings (`memfuse-py`)

Official Python bindings for MemFuse — an embedded 4-signal hybrid-search vector database built with Rust and PyO3.

## Purpose

Brücke zwischen Rust-Kern und Python-Ökosystem. Ermöglicht NumPy-Zero-Copy-Umschlag, Python-Collection-Management und FFI-Anbindung.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 4 (Ränder / FFI Bindings)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Installation

```bash
pip install memfuse
```

## Quick Start

```python
import memfuse

# Initialize database
db = memfuse.PyMemFuse("./data")
collection = db.collection("documents")

# Insert document
collection.insert("doc_1", "MemFuse provides high-performance embedded vector search.")

# Perform hybrid search
results = collection.hybrid_search("vector search")
for res in results:
    print(res.id, res.score, res.text)
```

## Öffentliche API-Übersicht

- **Runtime State:** `PyRuntimeState`
- **PyO3 Types:** `PySearchResult`, `PyDocument`, `PyVectorIndexStats`, `PyStorageStats`, `PyDbStats`
- **CRUD Operations:** `insert`, `get`, `update`, `upsert`, `delete`

## Development & Publishing

Refer to [PUBLISHING.md](PUBLISHING.md) for instructions on local building, testing, and release management.

## Architektur & Verweise

Details zu FFI-Invarianten und Python-Anbindung finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §9.4.
