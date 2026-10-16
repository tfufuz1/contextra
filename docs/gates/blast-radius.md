# Harness Command: blast-radius

`blast-radius` ermittelt den Auswirkungsbereich von Änderungen über den Rückwärts-Abhängigkeitsgraph aller Crates.

## CLI-Syntax

```bash
cargo run --manifest-path xtask/Cargo.toml -- blast-radius [--base <rev>] [--head <rev>] [--json]
```

## Funktionsweise
- Liest `capabilities.toml` (`may_depend_on`) und berechnet alle transitiv betroffenen Crates und deren Tests.
- Bricht mit Status `error` ab, falls Zyklen im Graph erkannt werden.
