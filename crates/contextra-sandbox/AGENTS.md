# AGENTS.md — `contextra-sandbox`

> **Ring 2** · Klasse `[R]` · Reifegrad `stable` · Unsafe: nein · Rolle: WASM-Sandbox (Wasmtime) mit Fuel-Begrenzung, Zulassung und Genehmigung von Modulen.
> Gemeinsame Regeln stehen in der Root-`AGENTS.md`; diese Datei nennt nur Crate-Spezifisches. Stand 2026-11-07, statisch gelesen, **nicht kompiliert**; Zahlen sind grep-Heuristiken.

## 1. Baukasten-Profil

| Aspekt | Wert |
|---|---|
| Standalone-Build | `cargo build -p contextra-sandbox --no-default-features --locked` |
| Contextra-Abhängigkeiten **Ist** (Cargo) | keine |
| Contextra-Abhängigkeiten **Soll** (PLAN §4) | keine |
| Features | keine |
| Lints | `#![forbid(unsafe_code)]` |

## 2. Abhängigkeiten & Schulden

- Keine Contextra-Abhängigkeit (autark). `wasmtime 36` (`async`, `cranelift`), `tokio`.
- `engine` hängt hart an `sandbox` (Ist, Feature `sandbox`), `store` hat Dev-Kante: beides lösen (M5/M6).

## 3. Modul-Karte (am Code verifiziert)

| Pfad | Verantwortung |
|---|---|
| `executor/` (`runner`, `state`) | Ausführung mit Fuel |
| `capabilities.rs`, `approval.rs` | Fähigkeiten, Genehmigungsstufen |
| `wasi/` (`fd`, `sys`), `output.rs`, `merge.rs`, `error.rs` | WASI-Anbindung, Ausgabe, WASM-`MergeOperator` |

## 4. Verträge & Invarianten

1. **Admission (INV-ADMISSION-1):** `AdmittedModule` nur über `ModuleVerifier::verify`; SHA-256 aus den Bytes, nie vom Aufrufer.
2. **Genehmigung (INV-APPROVAL-1):** `High`/`Elevated` laufen nicht vor `Approved`.
3. **Determinismus (INV-WASM-DET-1):** gleiches Modul, gleiche Eingabe, gleicher Fuel ⇒ bit-identisches Ergebnis; Wall-Clock-Unterbrechung nur Sicherheitsnetz, fail-closed.
4. Standard: keine Fähigkeiten (Deny-by-default).

## 5. Ports

**Implementiert:** `ModuleVerifier`, `FilterModule`, WASM-`MergeOperator`.

## 6. Nebenläufigkeit & Performance

- Pro Ausführung eigene Store/Instanz, kein geteilter Zustand; Fuel und Speichergrenzen immer gesetzt.

## 7. Anti-Patterns

- Modul ohne Digest-Prüfung ausführen.
- Host-Funktionen mit Dateizugriff ohne Capability.

## 8. Tests & Messung

```bash
cargo test -p contextra-sandbox --locked
cargo clippy -p contextra-sandbox --all-targets -- -D warnings
```

- Fuzzing der Module-Eingabe, Determinismus-Test x86-64/aarch64.

## 9. Offene Arbeit

| WP | Inhalt |
|---|---|
| 5.3 | Kante `engine → sandbox` hinter Port/Feature |
| M6 | Dev-Kante `store → sandbox` entfernen |
