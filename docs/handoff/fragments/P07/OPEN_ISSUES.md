# Offene Punkte P07 (Fremdfehler, Unklares, [H]-Punkte)

- [V] Cargo.lock ist im Workspace dirty bei Cargo-Ausführungen ohne `--locked`; wird vor jedem Commit mittels `git checkout -- Cargo.lock` bereinigt.
- [V] AVX-512 Hardware: Host-Maschine besitzt keine AVX-512-Flags; AVX-512-Pfade werden softwareseitig/syntaktisch geprüft, Benchmarks können AVX-512 nicht auf nativer Hardware ausführen.
- [V] Miri: Komponente `cargo-miri` ist in der bereitgestellten Toolchain nicht installiert.
