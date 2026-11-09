# Offene Punkte P07 (Fremdfehler, Unklares, [H]-Punkte)

- [V] Cargo.lock ist im Workspace dirty bei Cargo-Ausführungen ohne `--locked`; wird vor jedem Commit mittels `git checkout -- Cargo.lock` bereinigt.
- [V] AVX-512 Hardware: Host-Maschine besitzt keine AVX-512-Flags; AVX-512-Pfade werden softwareseitig/syntaktisch geprüft, Benchmarks können AVX-512 nicht auf nativer Hardware ausführen.
- [V] Miri: Komponente `cargo-miri` ist in der bereitgestellten Toolchain nicht installiert.
- [V] Fremdfehler CI - Production Unwrap Counter Gate: .github/unwrap_baseline.txt steht auf 1197, aber `main` enthält bereits 1333 Production-Unwraps (keine aus contextra-simd). Liegt außerhalb BESITZ (.github/ darf gemäß Regel 2 nicht geändert werden).
- [V] Fremdfehler CI - proof-tests & llvm-cov: Build-Fehler in contextra-crypto (`crates/contextra-crypto/src/kv_segment/`) in `main`. Liegt außerhalb BESITZ.
- [V] Fremdfehler CI - contextra-py Gate Stack: Build-Fehler in contextra-graph (`crates/contextra-graph/`) durch fehlende contextra_ports/contextra_types Referenzen in `main`. Liegt außerhalb BESITZ.
