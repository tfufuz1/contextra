# AGENTS.md — `contextra-simd`

## 1. Rolle
`contextra-simd` kapselt hardwarenahe SIMD-Vektordistanzberechnungen in einer genau auditierten, sicheren API-Insel.

## 2. Ring, Klasse, Reifegrad & `may_use`
- **Ring:** Ring 0 (Unsafe-Insel 1 von 3)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **`may_use` (Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)

## 3. Garantien & Anforderungen (§33, §6.7)
- **REQ-SMD-001:** Exponiert ausschließlich **sichere** öffentliche APIs.
- **REQ-SMD-002:** Jeder `unsafe`-Block trägt ein explizites `// SAFETY:` Kommentar.
- **Durchsetzungsmechanismus (§27.6):** Miri-Ausführung, Property-Tests gegen sichere Skalar-Referenzimplementierungen.

## 4. Verbotene Muster
- Exponieren unsicherer Funktionen in der öffentlichen Schnittstelle.
- `unsafe`-Blöcke ohne `// SAFETY:` Begründung.

## 5. Implementierte & konsumierte Ports (Anhang A)
- Exponiert direkte Vektor-Distanzfunktionen (L2, Cosine, DotProduct, Hamming) für `contextra-vector`.

## 6. Tests & Gates
- **Miri Gate:** Miri-Verifikation aller SIMD Intrinsics / Pointer-Operationen.
- **Property-Tests:** Bit-exakte Gleichheit gegen reine Skalar-Funktionen.

## 7. Offene Punkte
- keine.
