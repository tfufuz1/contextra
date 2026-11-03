---
name: contextra-invariants-gate
description: >-
  Use this skill to check Contextra architectural invariants: the ring hierarchy (0 to 4), the dependency matrix,
  the zero-panic lints, the two unsafe islands, deterministic ports, naming rules, and crate headers.
---

# Architektur- und Qualitätsgates prüfen

Maßgeblich: [`docs/SPEC.md`](../../../../docs/SPEC.md), Kapitel 2, 5.2, 5.3, 6.3, 13.1 und 15.1 (Architektur und Lint).

## 1. Ringe und Abhängigkeiten

- Abhängigkeiten zeigen nur auf gleiche oder niedrigere Ringe (AR-1). Innerhalb Ring 2 gilt die Matrix SPEC 5.3.
- `store` kennt weder `text` noch `vector` noch `erasure` (AR-2).
- `erasure` kennt keine Ring-2-Crates (AR-3).
- Ring 0 (`types`, `ports`) hat kein I/O, kein `async` und keine Netzwerk-Crate (AR-5).
- Nur `TenantScopedStorage` ruft den rohen Store (AR-6).

Prüfung: `cargo test -p contextra-arch-tests --locked` (T-ARCH-3, T-ARCH-6).

## 2. Zero-Panic (P6)

Im Produktionscode, also `crates/*/src/` außerhalb von `#[cfg(test)]`:

- kein `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()`, `assert!()`
- kein ungeprüftes Indexieren (`arr[i]`); stattdessen `.get(i)` mit Fehlerbehandlung
- kein `let _ = …` auf ein `Result`

Prüfung:

```bash
cargo clippy --workspace --all-targets --locked -- \
  -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic -D clippy::indexing_slicing
```

Der Ratchet in `governance/ratchet.toml` erlaubt nur sinkende Zahlen (SPEC 5.8).

## 3. Genau zwei Unsafe-Inseln (D-29, AR-8)

- `unsafe` nur in `contextra-sys` und `contextra-simd`.
- Jeder Block mit `// SAFETY:`-Kommentar.
- Alle anderen Crates mit `#![forbid(unsafe_code)]`.
- Kein `mmap` (D-94).

Prüfung: `grep -rln "unsafe" crates/*/src` darf nur die beiden Inseln nennen. Die Architekturtests (T-UNSAFE-1) prüfen das automatisch.

## 4. Deterministische Ports (P7)

Zeit, Zufall und IDs nur über `Clock`, `Rng`, `IdGen`. Ausnahme: Schlüssel-, Salt- und Nonce-Material aus dem OS-CSPRNG.

Verboten in der Kernlogik: `Instant::now()`, `SystemTime::now()`, `thread_rng()`, `from_entropy()`.

## 5. Namen, Köpfe und Marker (D-90, D-91, D-97)

- Keine Module `misc`, `util`, `utils`, `common`, `helpers`, keine Endung `_impl`.
- Eine Modulform: `foo.rs` neben `foo/`, kein `mod.rs`.
- Der Zweck-Satz steht wortgleich in `Cargo.toml`, im `//!`-Kopf von `lib.rs` und in der ersten Zeile der `README.md`.
- Keine Agenten-, Sitzungs- oder Zeitstempelmarken, keine absoluten Pfade.

## 6. Befundliste

Melden Sie Verstöße mit der SPEC-ID (z. B. `P6`, `AR-8`, `D-04`), der Datei und der Zeile. Die Behebung folgt dem Arbeitspaket aus SPEC 15.4.
