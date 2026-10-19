# contextra-sandbox — Agenten-Hinweise

## Zweck
In-Memory WASM-Sicherheits-Sandbox für untrusted Plugins und Algorithmen.

## Ring & Invarianten
- Ring: 2
- Async erlaubt: nein
- Unsafe erlaubt: nein

## Abhängigkeiten (`may_depend_on`, siehe `capabilities.toml`)
- `contextra-types`
- `contextra-ports`

## Bekannte Fallstricke
- WebAssembly Fuel-Grenzwerte müssen deterministisch gesetzt werden.
- Disk-I/O und System-Calls im WASM-Modul müssen blockiert werden (WASI-Restriktionen).

```bash
cargo test -p contextra-sandbox --locked
cargo test -p contextra-sandbox --test wasm_boundary_tests
cargo test -p contextra-sandbox --test pure_merge_operator_capabilities_test
cargo test -p contextra-sandbox --test wasm_wasi_io
cargo xtask check-agents-integrity
cargo xtask doctrine-scan --crate contextra-sandbox
cargo xtask check-unsafe-islands
```


## Verifikation
- `cargo test -p contextra-sandbox --locked`
- `cargo test -p contextra-sandbox --test wasm_boundary_tests`
- `cargo test -p contextra-sandbox --test pure_merge_operator_capabilities_test`
- `cargo test -p contextra-sandbox --test wasm_wasi_io`
