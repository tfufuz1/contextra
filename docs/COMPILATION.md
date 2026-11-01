# Contextra – Implementierer-Kompilier- und Bauanleitung

<!-- doc-stamp: 2026-11-01 | Normativ: docs/SPEC.md §5.1, PLAN.md -->

Dieses Dokument dient als operative Anleitung für Entwickler und Agenten, um den standardkonformen Cargo-Workspace von Contextra schrittweise zu kompilieren, zu testen und Quality-Gates auszuführen.

---

## 1. Systemvoraussetzungen

- **Rust-Toolchain:** Version 1.89+ laut [`rust-toolchain.toml`](../rust-toolchain.toml) (Edition 2021).
- **Komponenten:** `rustfmt`, `clippy`.
- **Cargo:** Resolver v2.
- **Plattform:** Linux (x86_64 / aarch64), macOS.
- **Single-Node-Umgebung:** Lokales Dateisystem (POSIX-kompatibel für `fsync` und `sync_dir`).

---

## 2. Der standardkonforme Workspace (SPEC §5.1)

Der Soll-Workspace besteht aus genau 14 Produktionscrates und 4 Hilfs-/Test-Crates:

```
Ring 4:  contextra (Fassade)
Ring 3:  contextra-erasure · contextra-engine
Ring 2:  contextra-mvcc · contextra-store · contextra-text · contextra-vector · contextra-rank
Ring 1:  contextra-sys [unsafe] · contextra-simd [unsafe] · contextra-durable-fs · contextra-crypto
Ring 0:  contextra-types · contextra-ports
dev:     contextra-testkit · contextra-lab · ctx-verify · tests/arch
```

> [!NOTE]
> Alle historischen Crates (`graph`, `checkpoint`, `ingest`, `privacy`, `mcp`, `infer-*`, `candle`, `wire`, `license`, `db`) sind gemäß SPEC §5.1 nicht Teil des v1-Kerns und werden nicht im Produktions-Workspace geführt.

---

## 3. Bau- und Prüfschritte

### 3.1 Standard-Build & Check
```bash
# Workspace-Bibliotheken prüfen
cargo check --workspace

# Alle Targets (inkl. Tests, Beispiele, Benches) prüfen
cargo check --workspace --all-targets
```

### 3.2 Lints & Zero-Panic Prüfung (P7, SPEC §11.1)
Contextra erzwingt strikte Deny-Lints auf allen Produktivpfaden:
```bash
cargo clippy --workspace --all-targets -- \
  -D warnings \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::panic \
  -D clippy::indexing_slicing
```

### 3.3 Test-Ausführung
```bash
# Unit- und Integrationstests
cargo test --workspace --locked

# Architektur- und Unsafe-Insel-Gates
cargo test -p tests-arch
```

### 3.4 Unsafe-Audit & Miri (SPEC §8.3, §8.4)
```bash
# Miri auf den genau zwei Unsafe-Inseln
cargo miri test -p contextra-sys
cargo miri test -p contextra-simd
```
