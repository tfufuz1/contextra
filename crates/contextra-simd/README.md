# contextra-simd

SIMD-beschleunigte Distanzberechnungen und Vektor-Operationen.

Normativitätsklasse: **Kern `[K]`** (Ring 0 Unsafe-Insel, `stable`; §6.1, §6.7, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-simd` kapselt hardwarenahe SIMD-Vektordistanzberechnungen in einer genau auditierten, sicheren API-Insel.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Unsafe-Insel 1 von 3, `#![deny(unsafe_code)]` ausgenommen)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)
