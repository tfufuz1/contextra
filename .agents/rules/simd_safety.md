# Governance Rule: SIMD Safety & Unsafe Islands Isolation (`rules/simd_safety.md`)

## Gilt für
- `crates/contextra-simd`
- `crates/contextra-sys`
- `crates/contextra-wire`
- `crates/contextra-crypto`

## Pflichtregeln
1. **Unsafe Isolation Invariante**: `unsafe` Rust Code darf ausschließlich in den vier freigegebenen Unsafe-Inseln (`contextra-simd`, `contextra-sys`, `contextra-wire`, `contextra-crypto`) existieren. Alle anderen Crates erzwingen `#![forbid(unsafe_code)]` oder `#![deny(unsafe_code)]` (Quelle: `CONSTITUTION.md`, `capabilities.toml`).
2. **`// SAFETY:` Dokumentationspflicht**: Jeder `unsafe`-Block muss eine explizite, unmissverständliche `// SAFETY:` Begründung enthalten, welche Invarianten (Alignment, Bounds, Pointer Validity) eingehalten werden (Quelle: `CONSTITUTION.md`, `crates/contextra-simd/src/lib.rs`).
3. **64-Byte Alignment für Vector-Slabs**: SIMD-Kernels und Arena-Allocators verlangen striktes Memory Alignment von 64 Byte (`ARENA_ALIGNMENT_BYTES = 64`), um UB und Alignment Faults bei AVX-512/NEON Vektor-Loads zu verhindern (Quelle: `crates/contextra-vector/src/hnsw/arena.rs`).

## Häufige Fehler
- Fehlen des `// SAFETY:`-Kommentars bei Zeiger-Deref.
- Ausführen von unaligned Reads auf SIMD-Puffern.
- Einschleusen von `unsafe`-Blöcken in Ring-0/Ring-1 Kernmodule außerhalb der Inseln.

## Verweise
- `CONSTITUTION.md`
- `capabilities.toml`
- `crates/contextra-simd/src/lib.rs`
