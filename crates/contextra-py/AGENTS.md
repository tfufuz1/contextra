# AGENTS.md — `contextra-py`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-py` · **Ring:** 4 · **Version:** 0.1.0
- **Beschreibung:** Python bindings for Contextra using PyO3
- **src:** 16 Dateien, 2235 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 3):** `contextra`, `contextra-types`, `contextra-wire`
- **Interne Dev/Build-Abhängigkeiten (0):** –
- **Externe Abhängigkeiten (normal, 7):** `flatbuffers`, `numpy`, `parking_lot`, `pyo3`, `pythonize`, `serde_json`, `tokio`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra`, `contextra-types`; eingehend –
- **Features (4):** `default`, `docid-128`, `pyo3`, `test-hooks`

### Öffentliche API (grob, aus src/)
- struct (8): `PyCollection`, `PyContextra`, `PyDbStats`, `PyDocument`, `PyRuntimeState`, `PySearchResult`, `PyStorageStats`, `PyVectorIndexStats`
- fn (53): `_trigger_panic_for_test`, `check_subinterpreter_guard`, `collection`, `contextra_err`, `delete`, `dict_to_json`, `doc_to_py`, `drop_collection`, `flush`, `get`, `get_kv`, `get_links`, `get_runtime`, `hybrid_search`, `hybrid_search_fb` … (+38)
- const (7): `MAX_BATCH_SIZE`, `MAX_COLLECTION_NAME_LENGTH`, `MAX_ID_LENGTH`, `MAX_LABEL_LENGTH`, `MAX_WORKER_THREADS`, `MIN_WORKER_THREADS`, `VERSION`
- mod (14): `bindings`, `collection`, `common`, `crud_macros`, `db`, `db_stats`, `document`, `functions`, `hyperedge`, `kv_links`, `runtime_state`, `search_result`, `storage_stats`, `vector_index_stats`
- pub use: `bindings::*`, `collection::PyCollection`, `common::*`, `db::PyContextra`, `db_stats::PyDbStats`, `document::PyDocument`, `functions::*`, `runtime_state::*`, `search_result::PySearchResult`, `storage_stats::PyStorageStats` … (+1)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
