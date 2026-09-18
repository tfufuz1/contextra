# Diagnose-Report: `memfuse-core[docid-128]` (feature-check)

Fehler: **7**  |  Warnungen: **0**

## `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs`  (4 Diagnose(n))

- **ERROR [E0308]** @ `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs:797:21`: mismatched types
- **ERROR [E0308]** @ `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs:797:21`: mismatched types
- **ERROR [E0308]** @ `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs:797:21`: mismatched types
- **ERROR [E0308]** @ `/home/jules/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/proptest-1.11.0/src/sugar.rs:797:21`: mismatched types

## `/rustc/48a229ceaefd4985c50990b14116b6d856af0985/library/core/src/macros/mod.rs`  (2 Diagnose(n))

- **ERROR [E0277]** @ `/rustc/48a229ceaefd4985c50990b14116b6d856af0985/library/core/src/macros/mod.rs:46:32`: can't compare `u64` with `u128`
  - _Hinweis:_ the trait `PartialEq<u128>` is not implemented for `u64`
  - _Hinweis:_ `u64` implements trait `PartialEq<Rhs>`
- **ERROR [E0308]** @ `/rustc/48a229ceaefd4985c50990b14116b6d856af0985/library/core/src/macros/mod.rs:46:35`: mismatched types

## `crates/memfuse-core/src/types/domain.rs`  (1 Diagnose(n))

- **ERROR [E0308]** @ `crates/memfuse-core/src/types/domain.rs:1821:34`: mismatched types
  - _Hinweis:_ associated function defined here
  - _Hinweis:_ you can convert a `u64` to a `u128`
