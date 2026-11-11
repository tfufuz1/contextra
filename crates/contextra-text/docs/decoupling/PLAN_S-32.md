# Implementierungsplan S-32: contextra-text entkoppeln & Hohlkern auflösen

## Ziel
Entkopplung von `crates/contextra-text` zu einem autarken Ring-0/Klasse-A Baustein. Auflösung des Hohlkerns, sodass `inverted`, `stream` und `wand` im Kern ohne Feature-Gate (`types-compat`) kompilieren. Kern-Closure beschränkt sich auf `contextra-ids` und `contextra-env`. Alle Anbindungen an `contextra-types` und `contextra-ports` liegen hinter dem Feature `workspace` in `src/compat/`.

## Arbeitspakete & Gates

1. **WP0 Baseline**: Erhebung aller Baseline-Werte, Verifikation des Hohlkerns, Ablage von `BASELINE.md` und `PLAN_S-32.md`.
2. **WP1 Charakterisierung**: Golden-Fixtures für BM25, WAND und Posting-Listen (`tests/golden/`).
3. **WP2 Fehler**: Eigener `TextError`, tabellarischer Test in `compat/error.rs` für bitgleiche Konvertierung.
4. **WP3 Nähte**: Inventar `StorageEngine`, `PostingStore`-Trait, `StorageBridge<S>(pub S)`.
5. **WP4 Hohlkern auflösen**: `stream`, `inverted`, `wand` ungaten, Re-Exports und Port-Impls nach `compat/`.
6. **WP5 Ids & Env**: Differenztests, Migration von `DocId`, `Clock`/`unix_ms`, `docid-128`.
7. **WP6 Referenz & Standalone**: `tests/core_smoke.rs`, `examples/standalone.rs`, Quickstart-Doctest.
8. **WP7 Doku & Bericht**: Bereinigung `AGENTS.md`, Abschlussbericht `REPORT.md`.
