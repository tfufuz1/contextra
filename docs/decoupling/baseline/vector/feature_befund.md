# Feature-Befund contextra-vector

| Feature | Deklariert in Cargo.toml | Nutzung in src/tests/benches | Weiterleitung in anderen Crates / CI | Status |
|---|---|---|---|---|
| std | Ja | Ja (types.rs, lib.rs) | Default | Behalten |
| simd | Ja | Optionale SIMD-Distanzberechnung | - | Neu |
| serde | Ja (leer) | Nein (serde Dep ist nicht optional, derive aktiv) | Keine direkte Weiterleitung | Bereinigungskandidat (VC6.6) |
| tracing | Ja (leer) | Nein (tracing Dep ist nicht optional) | Keine | Bereinigungskandidat (VC6.6) |
| inspect | Ja (leer) | Nein | Keine | Bereinigungskandidat (VC6.6) |
| experimental | Ja | Aktiviert Sub-Features | Keine | Behalten |
| mmap | Ja (leer) | Nein (memmap2 ist nicht optional) | Keine | Behalten/Prüfen |
| diskann | Ja | Alias für experimental-diskann | Keine | Alias erhalten |
| quantize-rabitq | Ja | Alias für experimental-rabitq | Keine | Alias erhalten |
| ports-compat | Ja | Ja (schaltet types, ports, mvcc, sys) | Default | Kernkompatibilität |
| docid-128 | Ja | Ja (types.rs, Persistence, Tests) | crates/contextra-vector/Cargo.toml | Behalten |
| experimental-diskann | Ja | Ja (diskann/, Tests, Benches) | contextra-db, contextra-engine | Behalten (blockiert) |
| experimental-rabitq | Ja | Ja (quantize_rabitq.rs) | Keine | Behalten |
| experimental-predicate-augmented-search | Ja | Ja (diskann/) | Keine | Behalten |
| experimental-all | Ja | Bündelt alle Experimente | - | Neu |
| graph | Ja (leer) | Nein | Keine | Bereinigungskandidat (VC6.6) |
| partial-index-rebuild | Ja | Ja (partial_rebuild.rs, VETO-F02) | VETO-F02 Guard | Behalten (Non-Default) |
