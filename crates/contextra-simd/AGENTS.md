# AGENTS.md — contextra-simd
> Ring 0 · stable · Quelle: capabilities.toml · Spec: K.25

## 1. Zweck

SIMD-beschleunigte Distanzberechnungen (Cosine, L2/Euklidisch, Dot Product) und Vektor-Operationen für f32, u8 und SQ8-Quantisierung. Der Crate kapselt hardware-spezifische Kernel (AVX2, AVX-512, NEON, Scalar) sowie dynamischen CPU-Dispatch in einer Unsafe-Insel und stellt eine 100% sichere öffentliche API bereit.

## 2. Modul-Karte

| Datei/Verzeichnis | Verantwortung |
|---|---|
| `src/lib.rs` | Öffentliche safe API-Schnittstelle, Re-Exports und Top-Level Doku |
| `src/dispatch.rs` | Hardware-Feature-Erkennung und dynamische CPU-Dispatching-Logik |
| `src/kernels/mod.rs` | Modul-Deklarationen für Kernel und gemeinsame Typen/Hilfsfunktionen |
| `src/kernels/avx2.rs` | AVX2-beschleunigte F32- und U8-Distanzkernel |
| `src/kernels/avx512.rs` | AVX-512 / VNNI-beschleunigte Kernel für F32 und U8 |
| `src/kernels/neon.rs` | ARM NEON SIMD-Kernel für AArch64 |
| `src/kernels/scalar.rs` | Skalare Fallback-Implementierungen aller Distanz- und Ähnlichkeitsfunktionen |

## 3. Invarianten

- `INV-SIMD-SAFE-BOUNDARY`: Die öffentliche API ist 100% safe. Alle `unsafe`-Blöcke und `unsafe fn` sind intern gekapselt und erfüllen `#![deny(unsafe_op_in_unsafe_fn)]`.
- `INV-SIMD-FALLBACK-PARITY`: Jeder SIMD-Kernel MUSS identische Ergebnisse wie der skalare Fallback in `scalar.rs` liefern (validiert via SIMD Cross-Validation Workflow).
- `INV-SIMD-BOUNDS-CHECK`: Puffer-Längen und Ausrichtungen MÜSSEN vor dem Aufruf intrinsischer Instruktionen geprüft werden, um Out-of-Bounds-Zugriffe zu verhindern.

## 4. Verboten / Anti-Patterns

- Unkontrollierte Panics bei NaN-Werten in Float-Arrays (Behandlung gemäß `cargo xtask check-nan-hot-loop`).
- Kopieren von Sicherheitsregeln aus `rules/simd_safety.md` — stattdessen direkt auf `rules/simd_safety.md` referenzieren.

## 5. Nebenläufigkeit, Async- und Lock-Regeln

- Kernel sind reine CPU-bound synchrone Berechnungsfunktionen ohne Zustände, Locks oder Async/tokio (Ring 0, Rule P26).

## 6. Verifikation

```bash
cargo test -p contextra-simd
cargo xtask check-nan-hot-loop
cargo xtask check-agents-integrity
cargo xtask check-unsafe-islands
cargo xtask check-ring0-async-purity
```

## 7. Bekannte Lücken / SOLL

- AVX-512 VNNI und NEON-Pfade erfordern spezifische Target-Features und fallen bei Nichtverfügbarkeit auf AVX2 oder Scalar zurück.

