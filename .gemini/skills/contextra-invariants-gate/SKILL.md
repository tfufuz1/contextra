---
name: contextra-invariants-gate
description: >-
  Use this skill to enforce Contextra architectural invariants, ring hierarchy (0-4),
  Zero-Panic deny-lints (unwrap, expect, panic, slice indexing), the 2 Unsafe island bounds, and deterministic port usage.
---

# Contextra Invariants Gate – Architektur- & Qualitätsprüfung

Dieser Skill führt die formalen Gates und Architekturprüfungen nach [docs/SPEC.md §5, §11](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [AGENTS.md §2](file:///home/freddy/Projekte/contextra/AGENTS.md) durch.

---

## 1. Die Invarianten im Detail

### Invariante 1: Ring-Hierarchie (P1, P30, SPEC §5.2)
- Abhängigkeiten dürfen nur abwärts oder innerhalb von Ring 2 gemäß Matrix SPEC §5.3 verlaufen.
- Ring 0 (`contextra-types`, `contextra-ports`) führt weder `tokio` noch I/O- oder Netzwerk-Crates.
- `store` kennt weder `text`, `vector` noch `erasure`.
- `erasure` kennt keine Ring-2-Crates.

### Invariante 2: Zero-Panic Richtlinie (P7, SPEC §11.1)
In Produktionscode (`crates/*/src/` außerhalb von `#[cfg(test)]`):
- Kein `.unwrap()`
- Kein `.expect()`
- Kein `panic!()`
- Kein `todo!()` oder `unimplemented!()`
- Kein `assert!()`
- Kein ungeprüftes Indexing `arr[i]` (stattdessen `.get(i).ok_or(...)`)

Verstöße werden über den Clippy-Lintsatz auf `deny`-Ebene blockiert:
```bash
cargo clippy --workspace --all-targets -- \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::indexing_slicing
```

### Invariante 3: Genau zwei Unsafe-Inseln (P2, D-29, SPEC §5.2)
`unsafe` ist ausschließlich in diesen zwei Crates gestattet:
1. `crates/contextra-simd` (AVX2 / AVX-512 f32 / NEON Distanzkernel)
2. `crates/contextra-sys` (Low-Level OS-Primitive, mlock, sync)

Jeder Block **muss** einen vorangestellten Kommentar `// SAFETY:` besitzen, der Alignment, Bounds und Gültigkeit belegt. Alle anderen 12 Produktivcrates erzwingen strikt `#![forbid(unsafe_code)]`.

### Invariante 4: Port-Purity & Determinismus (P28)
- Zeit, Zufall und IDs stammen aus `Clock`, `Rng` und `IdGen` aus `contextra-ports`.
- Keine direkten Aufrufe von `Instant::now()`, `SystemTime::now()` oder `thread_rng()` in Kernlogik.
- Keine `HashMap`/`HashSet` mit Standard-Hasher in deterministischen Pfaden (stattdessen `BTreeMap`/`BTreeSet`).

### Invariante 5: I/O ausschließlich über `Vfs` (D-04)
- Kein direkter Aufruf von `std::fs` oder `tokio::fs` in Produktivcrates.
- Sämtliche `fsync`- und `sync_*`-Fehler werden ausnahmslos propagiert.
