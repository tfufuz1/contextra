# Contextra Python Bindings (`contextra-py`)

Official Python bindings for Contextra — an embedded 4-signal hybrid-search vector database built with Rust and PyO3.

## Purpose

Brücke zwischen Rust-Kern und Python-Ökosystem. Ermöglicht NumPy-Zero-Copy-Umschlag, Python-Collection-Management und FFI-Anbindung.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 4 (Ränder / FFI Bindings)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Installation

> **Hinweis zur Distribution:** Eine PyPI-Distribution dieses Pakets existiert aktuell noch nicht und wird nachgereicht, sobald die Namensfrage für das Gesamtprojekt geklärt ist.

## Quick Start

```python
import numpy as np
import contextra

# Initialize database
db = contextra.open("./data", dimension=128)
collection = db.collection("documents")

# Insert document with vector and metadata
vector = np.random.rand(128).astype(np.float32)
collection.insert("doc_1", vector, metadata={"text": "Contextra provides high-performance embedded vector search."})

# Perform hybrid search
results = collection.hybrid_search("vector search", vector, k=5)
for res in results:
    print(res.id, res.score, res.metadata)
```

## Öffentliche API-Übersicht

- **Runtime State:** `PyRuntimeState`
- **PyO3 Types:** `PySearchResult`, `PyDocument`, `PyVectorIndexStats`, `PyStorageStats`, `PyDbStats`
- **CRUD Operations:** `insert`, `get`, `update`, `upsert`, `delete`

## Development & Publishing

Refer to [PUBLISHING.md](PUBLISHING.md) for instructions on local building, testing, and release management.

## Architektur & Verweise

Details zu FFI-Invarianten und Python-Anbindung finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) sowie `README.md` §9.4.
