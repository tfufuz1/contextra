# contextra-py

PyO3 Python Bindings für Contextra.

Normativitätsklasse: **Kern `[K]`** (Ring 4, `stable`; §6.0, `capabilities.toml`).

Offizielle Python-Brücke für Contextra — eine eingebettete 4-Signal Hybrid-Search Vektordatenbank.

## Zweck & Baukasten-Profil (M1–M11)

- **Ring & Klasse:** Ring 4 (Ränder / FFI-Bindings).
- **Abhängigkeiten:** `contextra` (Fassade), `contextra-types`, `contextra-wire`.
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]` strikt durchgesetzt (M9).
- **Entkopplung & Zero-Leak (M7):** Keine Löschzusagen, keine DSGVO- oder Deletion-Proof-Relikte im Crate.
- **Additive Features & minimale Defaults (M5):**
  - Standard-Build (`default = []`) ist leichtgewichtig und frei von Schwergewichten.
  - PyO3 C-Extension Bindings liegen hinter dem optionalen Feature `pyo3`.
  - Feature `test-hooks` ist isoliert und niemals in Default- oder Release-Konfigurationen enthalten.
- **FFI-Sicherheit (ADR-N04):** Rust-Panics werden an der Sprachgrenze via `catch_unwind` abgefangen und in strukturierte Python-Exceptions umgewandelt.

## Features

| Feature | Beschreibung | Default |
|---|---|---|
| `default` | Leeres Default-Set für minimale Kompilierzeit | Ja (leer) |
| `pyo3` | Aktiviert PyO3 FFI-Bindings, NumPy-Array-Anbindung und Python-Modul `_contextra` | Nein |
| `docid-128` | 128-Bit Dokumenten-IDs via `contextra-types` | Nein |
| `test-hooks` | Interne Hooks für Panik-Isolations- und Sabotage-Tests | Nein |

## Installation & Kompilierung

```bash
# Minimaler Build ohne PyO3:
cargo build -p contextra-py --no-default-features --locked

# Vollständiger Build mit PyO3 C-Extension Bindings:
cargo build -p contextra-py --features pyo3 --locked
```

## Python-Nutzung

```python
import numpy as np
import contextra

# Datenbank initialisieren
db = contextra.open("./data", dimension=128)
collection = db.collection("documents")

# Dokument mit Vektor und Metadaten einfügen
vector = np.random.rand(128).astype(np.float32)
collection.insert("doc_1", vector, metadata={"text": "Contextra provides high-performance embedded vector search."})

# Hybride Suche ausführen
results = collection.hybrid_search("vector search", vector, k=5)
for res in results:
    print(res.id, res.score, res.metadata)
```

## Öffentliche API-Übersicht

- **Python-Klassen:** `PyContextra`, `PyCollection`, `PySearchResult`, `PyDocument`, `PyVectorIndexStats`, `PyStorageStats`, `PyDbStats`, `PyRuntimeState`.
- **CRUD & Suche:** `insert`, `get`, `update`, `upsert`, `delete`, `search`, `hybrid_search`, `search_fb`.
- **Memory-Links & Relationen:** `link_memories`, `get_links`, `relate_bidirectional`.
- **Key-Value Primitives:** `put_kv`, `put_kv_if_absent`, `get_kv`.

## Standalone-Beispiel (M10)

Ein eigenständiges Anwendungsbeispiel befindet sich unter `examples/standalone_python_bridge.rs`.
