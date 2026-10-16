# Task-Karten in Contextra

Eine Task-Karte beschreibt ein isoliertes, maschinell prüfbares Arbeitspaket für Jules. Sie verhindert vage Aufträge und setzt klare Scope- und Budgetgrenzen.

## Drei Beispiele guter Task-Karten

### Beispiel 1: WAL-fsync Fehlerbehandlung (`crates/contextra-store`)
```toml
id        = "T-2026-0101"
title     = "WAL: fsync-Fehler propagieren statt verschlucken"
crate     = "contextra-store"
tier      = 1
risk      = "crash"
goal      = "Ersetze ignoriertes fsync in wal/flusher.rs durch fehlerfortpflanzendes Result."
non_goals = ["Keine Änderungen am CompactionEngine", "Keine neuen Dependencies"]
scope     = ["crates/contextra-store/src/wal/flusher.rs", "crates/contextra-store/tests/wal_*.rs"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["INV-WAL-POISON-EXPLICIT", "Zero-Panic"]
acceptance = [
  "cargo test -p contextra-store --test wal_lifecycle_crash_point_matrix"
]
evidence_required = ["Neuer Test für I/O-Fehler beim Flushen"]

[budget]
files = 6
lines = 250
iterations = 6
```

### Beispiel 2: SIMD-Bounds-Prüfung (`crates/contextra-simd`)
```toml
id        = "T-2026-0102"
title     = "SIMD: Slices-Bounds-Check vor Vektorisierung sicherstellen"
crate     = "contextra-simd"
tier      = 1
risk      = "simd"
goal      = "Expliziten Slices-Längencheck vor SIMD-Dot-Product einfügen."
non_goals = ["Keine ungesicherten Memory-Layouts"]
scope     = ["crates/contextra-simd/src/lib.rs"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["Zero-Panic", "Unsafe Islands Only"]
acceptance = [
  "cargo test -p contextra-simd"
]
evidence_required = ["Fuzzing-Test oder Out-of-bounds Testfall"]

[budget]
files = 3
lines = 100
iterations = 4
```

### Beispiel 3: Dokumentations-Fix (`crates/contextra-types`)
```toml
id        = "T-2026-0103"
title     = "Doku: Sync von Domain-Type Invarianten"
crate     = "contextra-types"
tier      = 1
risk      = "none"
goal      = "Aktualisiere Doc-Comments in domain/mod.rs bezüglich Fehlercodes."
non_goals = ["Keine API-Code-Änderungen"]
scope     = ["crates/contextra-types/src/types/domain/mod.rs"]
forbidden = ["xtask/**", ".github/**", "capabilities.toml", "AGENTS.md", "rust-toolchain.toml", "deny.toml", ".jules/setup/**"]
invariants = ["Single Error Root"]
acceptance = [
  "cargo test -p contextra-types"
]
evidence_required = ["Doku-Check bestanden"]

[budget]
files = 2
lines = 50
iterations = 3
```
