# Determinismus Audit Report (P28)

> Generiert durch `tools/determinism_guard.py`.
> Untersucht Nicht-Determinismus in Produktivpfaden (Instant, SystemTime, thread_rng, HashMap/HashSet).

## Crate Übersicht (Produktivpfade)

| Crate | Instant::now | SystemTime::now | thread_rng | Random HashMap/Set | env::var | Gesamt Prod |
|---|---|---|---|---|---|---|
| `contextra-crypto` | 0 | 0 | 0 | 0 | 0 | **0** |
