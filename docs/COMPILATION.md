# Contextra – Implementierer-Kompilier- und Bauanleitung

<!-- doc-stamp: 2026-11-02 | Normativ: docs/SPEC.md §5.1, §14.7, §15 -->

Dieses Dokument dient als operative Anleitung für Entwickler und Agenten, um den standardkonformen Cargo-Workspace von Contextra schrittweise zu kompilieren, zu testen und Quality-Gates auszuführen.

---

## 1. Systemvoraussetzungen

- **Rust-Toolchain:** Entsprechend [`rust-toolchain.toml`](../rust-toolchain.toml) und SPEC §4.2 D-82 (MSRV und Toolchain höchstens 2 Minor-Versionen hinter Stable).
- **Komponenten:** `rustfmt`, `clippy`.
- **Cargo:** Resolver v2.
- **Plattform:** Linux (x86_64 / aarch64), macOS. Windows wird in v1 nicht unterstützt (D-56; Windows-Code ist entfernt).
- **Single-Node-Umgebung:** Lokales Dateisystem (POSIX-kompatibel für `fsync` und `sync_dir`).

---

## 2. Der standardkonforme Workspace (SPEC §5.1)

Der Soll-Workspace besteht aus genau 14 Produktionscrates und 4 Hilfs-/Test-Crates:

```
Ring 4:  contextra (Fassade)
Ring 3:  contextra-erasure · contextra-engine
Ring 2:  contextra-mvcc · contextra-store · contextra-text · contextra-vector · contextra-rank
Ring 1:  contextra-sys · contextra-simd · contextra-durable-fs · contextra-crypto
Ring 0:  contextra-types · contextra-ports
dev:     contextra-testkit · contextra-lab · ctx-verify · tests/arch
```

---

## 3. Kompilier- und Testbefehle

### 3.1 Workspace kompilieren
```bash
cargo check --workspace --all-targets --locked
```

### 3.2 Architektur- und Lint-Tests ausführen
```bash
cargo test -p contextra-arch-tests
```

### 3.3 Kern-Tests ausführen
```bash
cargo test --workspace --locked
```
