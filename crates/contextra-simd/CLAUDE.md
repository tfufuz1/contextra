# CLAUDE.md — contextra-simd

> Crate-Kontext. Gilt zusätzlich zur Root-`CLAUDE.md` und zu `docs/SPEC.md` (maßgeblich bei Widerspruch).

## Rolle
Vektor-Distanzfunktionen (L2, Dot, Cosine) mit SIMD-Kernen und Skalar-Referenz. **Zweite erlaubte `unsafe`-Insel** (neben `contextra-sys`). Die SIMD-Varianten müssen identische Ergebnisse liefern wie die Skalar-Referenz (Selbstprüfung).

## Eckdaten (Messung 21.11.2026)
- Ebene: **E0** (SPEC §3.1, CMP-7).
- Umfang: 3.627 Zeilen `src`, 8 Testdateien, 99 `#[test]`.
- Features: `default`, `std`, `serde`, `workspace`, `types-compat`, `simd-avx512`.
- Build: ok.
- Genutzt von: `contextra-vector`.

## Befehle
```bash
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-simd
RUSTUP_TOOLCHAIN=stable cargo test -p contextra-simd --features simd-avx512   # nur auf Hardware mit AVX-512
RUSTUP_TOOLCHAIN=stable cargo clippy -p contextra-simd -- -D warnings
```

## Ist-Abweichungen zur Spec
- **Abhängigkeit `types`** soll laut Spec-Ziel entfernt werden (Tabelle 3.1). Aktuell vorhanden.
- **`compat/`-Verzeichnis** existiert (`src/compat`). Soll aufgelöst werden (G-11, R7).
- **Features `workspace` und `types-compat`** verletzen R7 (keine Compat-Namen). Entfernen, wenn `compat` aufgelöst ist.
- `unsafe` ist hier erlaubt. Jeder Block braucht ein Sicherheitsargument.

## Sitzungsnotizen
<!-- Nur anhängen. Format: ### YYYY-MM-DD · Branch · Kurzfassung · Offen: … -->
