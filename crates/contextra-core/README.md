# contextra-core

Historische Übergangs-Fassade (Strangler-Muster).

Normativitätsklasse: **Kern `[K]`** (Ring 0, `stable`; §6.0, `capabilities.toml`).

## Rolle & Zweck
`contextra-core` ist eine reine Übergangs- und Strangler-Fassade. Im Zuge der Zerlegung des historischen Monolithen wurden alle Kern-Typen und Port-Schnittstellen nach [`contextra-types`](../contextra-types/), [`contextra-ports`](../contextra-ports/) und [`contextra-mvcc`](../contextra-mvcc/) ausgelagert.

## Baukasten-Einordnung & Status
- **Ring:** Ring 0 (Legacy Facade)
- **Status:** 🟡 Deprecated Debt / Strangler-Fassade
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Deprecated Debt & Migration
- **Veralteter Baustein:** Dieser Crate ist als *Deprecated Debt* (siehe [`DEPRECATED_DEBT.md`](../../DEPRECATED_DEBT.md)) deklariert.
- **Handlungsempfehlung:** Neue Komponenten dürfen nicht von `contextra-core` abhängen, sondern müssen direkt `contextra-types` oder `contextra-ports` einbinden. `contextra-core` wird nach vollständiger Portierung aller Downstream-Abhängigkeiten archiviert.
