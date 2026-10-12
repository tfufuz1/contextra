# Governance Rule: Diff-Scoped Mutation Testing Standards (`rules/mutation-testing.md`)

## Gilt für
- Tier-1 Crates (`contextra-store`, `contextra-crypto`, `contextra-vector`, `contextra-db`)
- Kern-Garantien (G1 Crash/WAL, G2 Löschung/Krypto, G3 Audit)

## Pflichtregeln
1. **Diff-Basiertes Mutation Testing**: Im CI- und Jules-Workflow wird `cargo mutants` ausschließlich diff-basiert über das Harness `mutants-diff` ausgeführt (Quelle: `xtask/src/harness/mutants_diff.rs`).
2. **Umgang mit überlebenden Mutanten (MISSED)**: Überlebt ein Mutant eine Test-Suite, darf der Mutant KEINESFALLS durch Inlining-Attribute, globale Exclude-Filter oder künstliche Ausnahmen stummgeschaltet werden. Die Testabdeckung MUSS durch gezielte, schärfere Invarianten-Tests ergänzt werden.
3. **Konfiguration über `mutants-guarantees.toml`**: Um Laufzeit-Timeouts zu verhindern, werden in `mutants-guarantees.toml` ausschließlich extrem schwere Integrations- und Stress-Tests (z. B. 100MB Payloads, 1M Iterationen) im `additional_cargo_test_args`-Abschnitt explizit übersprungen.
4. **Schwellenwert-Beibehaltung**: Die Mutation-Scores geänderter Crates dürfen die in `docs/mutation_score_history.jsonl` dokumentierten historischen Schwellenwerte (Baseline: >= 60%) nicht unterschreiten.

## Befehle

```bash
# Diff-basiertes Mutation Testing über xtask ausführen
cargo run -q --manifest-path xtask/Cargo.toml -- mutants-diff

# Mutation Testing für Garantie-Gruppe G1 (Crash/WAL) ausführen
cargo mutants --config mutants-guarantees.toml -p contextra-store \
  -f "crates/contextra-store/src/wal/**" \
  -f "crates/contextra-store/src/lsm/recovery.rs" -o target/mutants-g1

# Mutation Testing für Garantie-Gruppe G2 (Löschung/Krypto) ausführen
cargo mutants --config mutants-guarantees.toml -p contextra-crypto \
  -f "crates/contextra-crypto/src/kv_shredding.rs" \
  -f "crates/contextra-crypto/src/deletion_proof/**" -o target/mutants-g2

# Mutation Testing für Tier-0 Crypto via just
just mutants-crypto
```

## Verbotenes Verhalten
- Einfügen von Mutanten-Ausschlüssen in Quellcode oder Konfigurationsdateien zur Umgehung echter Testlücken.
- Deaktivieren des `mutants-diff` Gates bei Codeänderungen in Tier-1 Crates.
- Verändern der Timeout-Multiplikatoren (`timeout_multiplier`) ohne freigegebene ADR.

## Häufige Fehlerbilder
- `MUTATION_SCORE_BELOW_THRESHOLD`: Der erreichte Mutation-Score liegt unter dem geforderten Mindestwert.
- `CARGO_MUTANTS_MISSING`: `cargo-mutants` Executable ist in der aktuellen Toolchain-Umgebung nicht auffindbar.

## Verweise
- `mutants-guarantees.toml`
- `xtask/src/harness/mutants_diff.rs`
- `docs/mutation_score_history.jsonl`
