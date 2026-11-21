# CLAUDE.md — contextra-crypto

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Alle kryptographischen Primitive: AES-256-GCM-SIV (AEAD), HKDF (Schlüsselableitung), Argon2id, KeyTable, Anker, `RecordSealer`, `SegmentSealer`, `KeyDestroyer`, Wurzel-Backends. **Kryptoprimitive nur hier** (R4). Schlüsselhierarchie: SPEC §5.4–5.8.

## Eckdaten (Messung 21.11.2026)
- Ebene: **E1** (SPEC §3.1, Kapitel 5).
- Umfang: 6.511 Zeilen `src`, 26 Testdateien, 251 `#[test]` (viele Tests, gute Basis).
- Features: `default`, `std`, `tracing`, `test-utils`.
- Build: ok.
- Genutzt von: `contextra-db`, `contextra-mcp`, `contextra-store`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-crypto
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-crypto --no-default-features   # G-01
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-crypto -- -D warnings
```

## Ist-Abweichungen zur Spec
- Ziel laut Spec: ~4.000 Zeilen (Ist 6.511).
- **R2-Verstoß über `store`:** `contextra-store` (E1) hängt von diesem E1-Crate ab. Spec-Ziel: `store` hat keine Krypto-Abhängigkeit (Kapitel 3.1).
- Annahme für Änderungen: Schlüssel- und Nonce-Erzeugung nur über `OsRng` (R10). Keine eigenen Krypto-Implementierungen in anderen Crates.
- `contract.rs` (1 Datei) und 34 `ANCHOR`/`FILE-CONTEXT`-Dateien sind Selbstdeklarationen, die laut REM-14 entfallen sollen.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
