# contextra-wire

FlatBuffers Wire-Protokolle und Serialisierungs-Formate.

Normativitätsklasse: **Kern `[K]`** (Ring 0 Unsafe-Insel, `stable`; §6.1, §6.7, §6.9, `capabilities.toml`).

## Rolle in einem Satz
`contextra-wire` verwaltet performante Zero-Copy FlatBuffers Wire-Protokolle und Deserialisierungs-Primitiven.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Unsafe-Insel 3 von 3, FlatBuffers FFI)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types` (Spec §6.9, `capabilities.toml`)
