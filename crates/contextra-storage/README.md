# contextra-storage

Eingebetteter, synchroner, crash-sicherer Key-Value-Kern nach `SPEC.md`. `#![forbid(unsafe_code)]`, nur `std` plus `contextra-storage-ports`.

**Status:** im Umbau, nicht produktiv. Es gibt keine öffentliche Facade, der Redo-Pfad im Committer ist ein Stub (IMPL S-1). Modulstand: `../../ARCHITECTURE.md` §2.

## Features

- `testing` (Default, S-9 offen): Testhilfen in `src/testing`. Zieht Testhaken in Release-Builds, soll künftig nicht Default sein.
- `cfg(loom)`: Nebenläufigkeitsmodell, `make loom` aus dem Repo-Root.

## Lints

Workspace-Lints verbieten `unwrap`, `expect`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`. `unsafe_code = forbid`.

## Tests

```bash
make test-target T=redo        # aus dem Repo-Root
```

Verfügbare Targets: `ls tests`. `tests/engine` kompiliert derzeit nicht. Unit-Tests liegen in den Modulen.

## Regeln für Änderungen

`../../CLAUDE.md` §4 und `../../CONTRIBUTING.md`.
