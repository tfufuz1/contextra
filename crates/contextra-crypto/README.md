# contextra-crypto

Kryptographische Vaults, Zeroize, Egress-Verschlüsselung, Audit-Ketten und Löschnachweise.

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.1, §6.9, §20, `capabilities.toml`).

## Rolle in einem Satz
`contextra-crypto` stellt I/O-freie kryptographische Primitiven, Schlüsselverwaltung, Zeroization, Audit-Chains und DeletionProof-Erzeugung/Verifikation bereit.

## Ring, Klasse & Reifegrad
- **Ring:** Ring 0 (Kryptographie & Nachweislogik, I/O-frei, keine Threads)
- **Klasse:** Kern `[K]`
- **Reifegrad:** `stable`
- **Erlaubte Abhängigkeiten (`may_use` Obergrenze):** `contextra-types`, `contextra-ports` (Persistenz nur über Port `DurableLog`; Spec §6.9, `capabilities.toml`)

*Hinweis zu Abweichungen:* `capabilities.toml` führt aktuell veraltetes `contextra-core` in `may_depend_on` (siehe `docs/status/ABW-L3.md`).
