# CLAUDE.md — contextra-sys

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Low-Level-Systemabstraktion und FFI, **eine von zwei erlaubten `unsafe`-Inseln** (zusammen mit `contextra-simd`). Enthält laut Spec: mlock, Direct-IO, Dumpable, Win32-ACL (in Übergangsphase noch in `store`, siehe `tests/unsafe_islands.rs`).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E0** (SPEC §3.1, ADR-41). Keine internen Abhängigkeiten.
- Umfang: 2.401 Zeilen `src`, 3 Testdateien, 47 `#[test]`.
- Features: keine.
- Build: ok.
- Genutzt von: `contextra-store`.
- `unsafe`-Lint: Ausnahme von R5. Alle anderen Crates haben `forbid(unsafe_code)`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-sys
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-sys -- -D warnings
RUSTUP_TOOLCHAIN=stable cargo test --test unsafe_islands   # G-02 (im Root-Paket)
```

## Hinweise
- Jeder `unsafe`-Block braucht eine Begründung (Sicherheitsargument) im Code. Das gilt auch für neue Stellen.
- Ziel laut Spec: ~2.500 Zeilen. Der Umfang steigt, wenn Win32-ACL aus `store` übernommen wird. Das vorher mit dem Nutzer klären.
- 9 Dateien mit `ANCHOR`/`FILE-CONTEXT` (REM-14: Selbstdeklarationen sollen weg), 1 `contract.rs`.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
