# Governance Rule: SIMD Safety & Unsafe Islands Isolation (`rules/simd_safety.md`)

> **Normative Vorgabe nach docs/SPEC.md §5.2, §8.3, §8.4 und D-29.**

## Gilt für
- `crates/contextra-simd`
- `crates/contextra-sys`
- Gesamter Workspace (Durchsetzung des `#![forbid(unsafe_code)]` Verbots)

## Pflichtregeln
1. **Unsafe-Insel-Doktrin (D-29, P2)**:
   - `unsafe` Rust Code ist im gesamten Workspace **ausschließlich in zwei benannten Inseln** erlaubt:
     1. `crates/contextra-simd` (AVX2 / AVX-512 f32 / NEON Distanzkernel)
     2. `crates/contextra-sys` (Low-Level OS-Primitive, mlock, Dateisystem-Sync)
   - Alle anderen 12 Produktivcrates erzwingen strikt `#![forbid(unsafe_code)]`.
2. **Lückenlose `// SAFETY:` Dokumentation**:
   - Jeder einzelne `unsafe`-Block erfordert zwingend eine präzise Begründung, die Alignment, Slice-Bounds und Pointer-Gültigkeit nachweist.
3. **Genauigkeits- & Tail-Disziplin in SIMD (SPEC §8.4)**:
   - Jeder Kernel muss für identische Eingaben das Ergebnis der Skalar-Referenz innerhalb einer relativen Fehlertoleranz von $\epsilon = 10^{-4}$ (für Norm $\le 10^3$) einhalten.
   - Vektorlängen, die kein Vielfaches der Registerbreite sind, werden deterministisch über den Skalar-Rest berechnet. Keine Out-of-Bounds Lesezugriffe!
4. **Verbot von u8-VNNI (D-28)**:
   - Der historische u8-VNNI-Pfad ist defekt und gestrichen. In v1 existieren ausschließlich f32-Kernel.

## Verweise
- `docs/SPEC.md` §8.3, §8.4
- `CONSTITUTION.md` §1.1
- `crates/contextra-simd/AGENTS.md`
- `crates/contextra-sys/AGENTS.md`
