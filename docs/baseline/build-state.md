# Build State (Ausgangsstand vor v1-Umstrukturierung)

## Status `contextra-sys`
`cargo check -p contextra-sys --locked --all-targets`: **SUCCESS** (0 Fehler).
`cargo test -p contextra-sys`: **SUCCESS** (50 Tests bestanden).

## Status Workspace Check (`cargo check --workspace --locked --all-targets`)

Erste 200 Zeilen der Ausgabe:

```
    Checking tempfile v3.27.0
    Checking contextra-ports v0.1.0 (/app/crates/contextra-ports)
    Checking bit-set v0.8.0
    Checking rand_chacha v0.3.1
    Checking memmap2 v0.9.11
   Compiling curve25519-dalek v4.1.3
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `strum`
   --> crates/contextra-ports/src/erasure.rs:357:46
    |
357 | #[derive(Debug, Clone, PartialEq, Eq, Error, strum::EnumIter)]
    |                                              ^^^^^ use of unresolved module or unlinked crate `strum`

error[E0433]: failed to resolve: use of unresolved module or unlinked crate `strum`
   --> crates/contextra-ports/src/erasure.rs:413:24
    |
413 | #[derive(Debug, Error, strum::EnumIter)]
    |                        ^^^^^ use of unresolved module or unlinked crate `strum`

warning: unexpected `cfg` condition value: `fault-injection`
   --> crates/contextra-ports/src/erasure.rs:467:21
    |
467 |     #[cfg(any(test, feature = "fault-injection"))]
    |                     ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
    = note: expected values for `feature` are: `backend-impl`, `onnx`, and `test-utils`
    = help: consider adding `fault-injection` as a feature in `Cargo.toml`
    = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
    = note: requested on the command line with `-W unexpected-cfgs`

    Checking rand v0.8.8
   Compiling curve25519-dalek-derive v0.1.1
error[E0599]: the method `as_dyn_error` exists for reference `&std::string::String`, but its trait bounds were not satisfied
   --> crates/contextra-ports/src/erasure.rs:424:9
    |
424 |         source: String,
    |         ^^^^^^ method cannot be called on `&std::string::String` due to unsatisfied trait bounds
    |
    = note: the following trait bounds were not satisfied:
            `std::string::String: StdError`
            which is required by `std::string::String: AsDynError<'_>`
            `&std::string::String: StdError`
            which is required by `&std::string::String: AsDynError<'_>`
            `str: Sized`
            which is required by `str: AsDynError<'_>`
            `str: StdError`
            which is required by `str: AsDynError<'_>`

error[E0599]: the method `as_dyn_error` exists for reference `&std::string::String`, but its trait bounds were not satisfied
   --> crates/contextra-ports/src/erasure.rs:431:9
    |
431 |         source: String,
    |         ^^^^^^ method cannot be called on `&std::string::String` due to unsatisfied trait bounds
    |
    = note: the following trait bounds were not satisfied:
            `std::string::String: StdError`
            which is required by `std::string::String: AsDynError<'_>`
            `&std::string::String: StdError`
            which is required by `&std::string::String: AsDynError<'_>`
            `str: Sized`
            which is required by `str: AsDynError<'_>`
            `str: StdError`
            which is required by `str: AsDynError<'_>`

    Checking signature v2.2.0
    Checking ed25519 v2.2.3
    Checking foldhash v0.2.0
    Checking contextra-sys v0.1.0 (/app/crates/contextra-sys)
    Checking allocator-api2 v0.2.21
    Checking contextra-durable-fs v0.1.0 (/app/crates/contextra-durable-fs)
Some errors have detailed explanations: E0433, E0599.
For more information about an error, try `rustc --explain E0433`.
warning: `contextra-ports` (lib) generated 1 warning
error: could not compile `contextra-ports` (lib) due to 4 previous errors; 1 warning emitted
warning: build failed, waiting for other jobs to finish...
```

## Status Workspace Test-Liste (`cargo test --workspace --no-fail-fast -- --list`)

`cargo test --workspace --no-fail-fast -- --list` schlug fehl:
Die Kompilierung des Arbeitsbereichs scheitert beim Erstellen des Crate `contextra-ports` (Fehler E0433: nicht aufgelöstes Modul/Crate `strum` in `crates/contextra-ports/src/erasure.rs:357`, `413` sowie Fehler E0599: unerfüllte Trait-Bounds für `as_dyn_error` in `crates/contextra-ports/src/erasure.rs:424`, `431`).
Da `contextra-ports` eine Abhängigkeit vieler anderer Crates im Workspace ist, konnte der gesamte Test-Listen-Scan nicht durchgeführt werden.
