# API Snapshots (`api/`)

This directory contains public API snapshots for Contextra core crates to measure interface drift during refactoring and restoration processes.

## Purpose
The API snapshot files capture all public items (`struct`, `enum`, `trait`, `fn`, `const`, `type`, `mod`, `use` re-exports, and `impl` methods) exported by core Ring-0 crates (`contextra-ports` and `contextra-types`).

## Generation
Snapshots are generated deterministically using `api/generate_api_snapshot.py` (standard Python 3):

```bash
python3 api/generate_api_snapshot.py
```

Alternatively, to generate a single crate snapshot:
```bash
python3 api/generate_api_snapshot.py --crate-dir crates/contextra-ports --out api/contextra-ports.txt
```

If `cargo-public-api` and nightly rustdoc JSON are available:
```bash
cargo public-api -p contextra-ports > api/contextra-ports.txt
cargo public-api -p contextra-types > api/contextra-types.txt
```

## Governance Rules
- Any Pull Request that modifies or alters files matching `api/*.txt` **must** list all affected restore cards under `docs/restore/` in the PR description.
- Snapshot generation must produce byte-identical output across repeated runs on identical source code.
