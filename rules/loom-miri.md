# Governance Rule: Concurrency & Memory Safety Verification (`rules/loom-miri.md`)

## Gilt für
- Unsafe-Inseln (`contextra-simd`, `contextra-sys`, `contextra-wire`, `contextra-crypto`)
- Lock-Hierarchien, Group-Commit, Lock-Free Datenstrukturen und Ring-0 Synchronisation (`contextra-store`, `contextra-mvcc`)

## Pflichtregeln
1. **Verpflichtender Loom-Einsatz**: Änderungen an Group-Commit Handoffs, atomaren Sequence Counters, Snapshot Registry Floor-Berechnungen oder Lock-Abläufen müssen zwingend mit Loom-Modellprüfungen verifiziert werden (Quelle: `xtask/src/loom_run.rs`).
2. **Loom Test-Isolierung**: Loom-Tests unter `crates/*/tests/loom_*.rs` erzwingen `RUSTFLAGS="--cfg loom"` und sequenzielle Ausführung (`--test-threads=1`).
3. **Bekannte Loom-Ausnahmen**: Tests, die unter der Loom Mock-Runtime nicht lauffähig sind (z. B. `loom_relate_n_ary` wegen `#![cfg(not(loom))]` in Engine oder `loom_quantizer_race_test` wegen Tokio Runtime Builder), sind in `xtask/src/loom_run.rs` zentral dokumentiert und werden vom Harness automatisch übersprungen.
4. **Verpflichtender Miri-Einsatz**: Jede Überarbeitung von Pointer-Arithmetik, SIMD Memory Slabs, Alignment oder FFI-Typen in Unsafe-Inseln muss frei von Undefined Behavior (UB), Strict Aliasing und Data Races unter Miri sein (Quelle: `xtask/src/harness/unsafe_audit.rs`).
5. **Miri Isolation Isolation Flag**: Miri-Ausführungen erzwingen `MIRIFLAGS="-Zmiri-disable-isolation"`.

## Befehle

```bash
# Loom Concurrency Exploration Suite über xtask ausführen
cargo run -q --manifest-path xtask/Cargo.toml -- loom-run

# Spezifischen Store Group Commit Loom-Test via just ausführen
just loom-store

# Unsafe-Audit inklusive Miri-Überprüfung ausführen
cargo run -q --manifest-path xtask/Cargo.toml -- unsafe-audit --miri

# Miri Testlauf manuell ausführen (für ein bestimmtes Unsafe-Island Crate)
MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test -p contextra-sys --locked
```

## Verbotenes Verhalten
- Umgehen von Miri UB-Befunden durch Unterdrückung von Warnungen ohne Behebung der echten Memory Safety Lücke.
- Ausführen von Loom-Tests ohne die Flagge `RUSTFLAGS="--cfg loom"`.
- Hinzufügen von echten Thread-Sleeps (`std::thread::sleep`) in Loom-Modelltests.

## Häufige Fehlerbilder
- `Miri toolchain missing or cargo +nightly miri execution failed`: Nightly Miri-Komponente nicht im System verfügbar.
- `Undefined Behavior: Out-of-bounds pointer arithmetic`: Pointer-Offsets in Unsafe-Inseln überschreiten die zugewiesene Slab-Grenze.
- `Data race detected by Loom`: Fehlende Memory Barriers / Atomic Ordering Invarianten bei nebenläufigen Mutationen.

## Verweise
- `xtask/src/loom_run.rs`
- `xtask/src/harness/unsafe_audit.rs`
- `crates/contextra-store/AGENTS.md`
- `.github/workflows/rust-ci.yml`
