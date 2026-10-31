# Governance Rule: Zero-Panic & Error Handling (`rules/zero_panic.md`)

> **Normative Vorgabe nach docs/SPEC.md §0.2 (Regel 2) und P7.**

## Gilt für
- Alle Crates in `crates/*/src/` (Produktionspfade außerhalb von `#[cfg(test)]`)

## Pflichtregeln
1. **Absolutes Verbot von Panics**:
   - `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()`, `assert!()` und `std::process::exit` sind in Produktivpfaden verboten.
2. **Kein ungeprüftes Index-Slicing**:
   - Direkte Indexzugriffe der Form `slice[i]` sind verboten. Verwende `.get(i).ok_or(...)` oder sichere Iteratoren.
3. **Einheitlicher Fehlerbaum (A6)**:
   - Alle Fehler müssen strukturiert über `ContextraError` oder domänenspezifische Fehler-Enums propagiert werden.
4. **Deny-Lints**:
   - Folgende Lints sind workspaceweit auf `deny` konfiguriert:
     - `clippy::unwrap_used`
     - `clippy::expect_used`
     - `clippy::panic`
     - `clippy::indexing_slicing`

## Verweise
- `docs/SPEC.md` §0.2, §3.3 (P7), §13.6
- `.github/unwrap_baseline.txt`
