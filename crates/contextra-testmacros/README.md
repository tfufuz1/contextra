Proc-Macros für Contextra-Tests und Benchmarks (`ctx_test`, `ctx_bench`).

Bietet die Attribute `#[ctx_test(id = "T-...")]` und `#[ctx_bench(id = "BM-...")]` zur statischen Verifikation von SPEC-IDs zur Compile-Zeit (T-TRACE-1, D-100).

## Verwendung

Das Crate wird ausschließlich als `[dev-dependencies]` in Test- und Benchmark-Suites eingebunden:

```toml
[dev-dependencies]
contextra-testmacros = { workspace = true }
```

### Tests

```rust
use contextra_testmacros::ctx_test;

#[ctx_test(id = "T-TRACE-1")]
fn test_sync() {
    assert_eq!(2 + 2, 4);
}

#[ctx_test(id = "T-TRACE-1")]
async fn test_async() {
    assert_eq!(2 * 2, 4);
}
```

- Für synchrone Funktionen emittiert das Makro standardmäßig `#[test]`.
- Für `async fn` emittiert das Makro `#[tokio::test]`.
- Ist bereits ein Test-Attribut vorhanden (z. B. bei Integration mit externen Testframeworks oder proptest), wird kein doppeltes Attribut erzeugt.
- Das Makro emittiert eine maschinenlesbare Dokumentationsmarkierung `#[doc = "SPEC-ID: ..."]`.

### Benchmarks

```rust
use contextra_testmacros::ctx_bench;

#[ctx_bench(id = "BM-WRITE-1")]
fn bench_iteration() {
    // Benchmark implementation
}
```

- Emittiert `#[doc = "SPEC-BENCH-ID: ..."]`.

## Validierungsregeln

- Test-IDs müssen mit `T-` beginnen und dürfen nur Großbuchstaben, Ziffern, `-` und `_` enthalten.
- Benchmark-IDs müssen mit `BM-` beginnen und dürfen nur Großbuchstaben, Ziffern, `-` und `_` enthalten.
- Leere IDs, Kleinbuchstaben und doppelte `id`-Attribute werden zur Compile-Zeit mit genauer Span-Position abgewiesen.
