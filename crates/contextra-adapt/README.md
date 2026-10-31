# contextra-adapt

Anpassungs- und Parameteradaptions-Layer.

Normativitätsklasse: **Rand `[R]`** (Ring 0, `experimental`; §6.1, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-adapt` regelt Laufzeitparameter (wie Search-Budget oder PID-Latenzsteuerung) deterministisch ohne eigene Hintergrund-Threads.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Parameteradaption)
- **Klasse:** Rand `[R]`
- **Reifegrad:** `experimental`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Spec §6.9, `capabilities.toml`)
