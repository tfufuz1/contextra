# Decoder-Fuzzing

Besitzer: SP-VERIFY. Jedes untrusted-byte Decoderformat erhält ein eigenes `cargo-fuzz`-Target. Fuzzing ergänzt Golden-/Property-Tests und Crash-Sweeps; es ersetzt sie nicht. Die Target-Dateien in `fuzz_targets/` sind derzeit noch Scaffold-Dateien und zählen nicht als Fuzz-Evidenz.

## Zielzuordnung

| Target | Decoder / wesentliche Properties |
|---|---|
| `decode_varint` | kanonisch ≤10 Byte, Überlauf/Überlänge ablehnen |
| `decode_redo_frame` | Grenzen vor Allokation, Frame-/Op-Tags, reservierte Bits |
| `decode_catalog_record` | Tag-/Versionsprüfung, Recordlänge, CRC/MAC-Eingabegrenzen |
| `decode_run_header` | Magic/Version/Featurebits/Dateilimits |
| `decode_run_block` | Prefix-Restarts, Sortierung, Feldlängen und Gesamtlänge |
| `decode_run_subindex` | Blockgrenzen, Aggregate, Stamp-/Weight-Felder |
| `decode_run_filter` | nur unterstützter Filtertyp, bounded Bitset und Parameter |
| `decode_run_footer` | monotone, nicht überlappende Offsets innerhalb Dateilänge |

Jedes Target soll beliebige Bytes direkt an genau einen Decoder geben, keine Dateisystem-/Providerzustände benötigen und Panics, UB, unbounded allocation oder nicht-deterministische Hänger als Fehler behandeln. Decoderfehler sind zulässig und erwartet.

## Implementierungsvertrag

Fuzz-Paket wird separat vom Workspace geführt. Cargo-Fuzz-/libFuzzer-Version, Rust-Toolchain und Aufruf werden im CI-Job fixiert. Corpus und Regressionseingaben enthalten keine Produktionsdaten. Nach jedem Fund wird der minimale Input als dauerhaftes Regressionstest-Fixture eingecheckt und die betroffene Invariante referenziert. `max_len`/Timeouts verhindern Ressourcenerschöpfung im Fuzzrunner; diese Limits dürfen nicht als Ersatz für Decoder-internes Bounds-Checking dienen.

## Gate-Evidenz

CI startet jedes Target mit festgelegtem Zeit-/Executionsbudget und archiviert Toolchain, Revision, Corpus-Hash und Ergebnis. G-F benötigt mindestens ein Target pro eingefrorenem Decoderformat; G-P benötigt Dauer-Fuzzing und keine offenen reproduzierbaren Korruptionsbefunde. Aktuelle Platzhalter-Targets sind als **nicht implementiert** zu führen.
