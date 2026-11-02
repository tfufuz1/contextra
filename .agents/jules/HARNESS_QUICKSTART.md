# Schnellstart für Jules-Sitzungen

Nicht normativ. Die Befehle stehen in [`docs/COMPILATION.md`](../../docs/COMPILATION.md).

## Verlässliche Befehle

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo test -p contextra-arch-tests --locked
```

Der Workspace baut derzeit nicht vollständig (SPEC F-32). Bis die Behebung abgeschlossen ist, meldet ein Lauf der ersten Befehle diese Fehler. Sie sind keine Regression der Aufgabe und werden im PR als Vorbestand vermerkt.

## Just-Rezepte

Das Justfile enthält unter anderem `check`, `test`, `gates`, `pre-push`, `crate`, `loom`, `bootstrap`, `coverage`, `loom-store` und `session-context`. Es ist ein optionaler Wrapper. Den Inhalt von `gates` und `session-context` prüfe ich in dieser Revision nicht. Das Rezept `gates` verweist laut Vorfassung auf `tests/layering.rs`; diese Datei existiert nicht. Verlässlich sind die Architekturtests mit `cargo test -p contextra-arch-tests --locked`.

## Eskalation

1. **Lokal stoppen.** Derselbe Compiler- oder Testfehler nach zwei Iterationen: Vorgehen überprüfen und Sitzung anhalten.
2. **Aufgabe teilen.** Wird der Plan wiederholt abgelehnt, wird die Aufgabe kleiner geschnitten.
3. **Mensch entscheidet.** Der Mensch prüft das Log und passt Aufgabe und Scope an.

Die Regeln der Vorfassung zu Exit-Codes und einem Stoppbefehl sind nicht geprüft und gelten nur, soweit sie in der Umgebung tatsächlich vorhanden sind.
