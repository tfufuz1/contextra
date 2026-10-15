# Audit-Bericht: `cloud_query` und `egress` Befund

## Übersicht
Gemäß TEIL B des Audit-Auftrags wurde das Repository (`crates/` und `Cargo.toml`-Dateien am HEAD) bezüglich der Existenz des MCP-Werkzeugs `contextra_cloud_query` sowie Egress-Guard Feature-Gating untersucht.

---

## 1. Befund zur Werkzeug-Existenz (`contextra_cloud_query`)

| Prüfung | Ergebnis | Details |
|---|---|---|
| Existenz von `contextra_cloud_query` | **NICHT VORHANDEN** | Eine case-insensitive Volltextsuche über alle Source-Dateien in `crates/` und Manifest-Dateien `Cargo.toml` ergab **0 Treffer** für `contextra_cloud_query` im produktiven Code am HEAD. |
| Ursache / Historie | In der Spezifikation genannt | Historische Spezifikationsdokumente (`SPEZIFIKATION_EP6_CONTEXTRA_GUARD.md`, `ARCHITEKTUR_MASTERPLAN_v2.md`) verwiesen auf `crates/contextra-mcp/src/server_dispatch.rs:297` und `server.rs`. Im aktuellen Codebase-HEAD wurde der MCP-Server refaktoriert; das Werkzeug existiert dort derzeit nicht mehr. |

---

## 2. Befund zu Egress- und Cloud-Features in `crates/`

| Crate / Datei | Feature / Symbol | Status / Beschreibung |
|---|---|---|
| `crates/contextra-router/Cargo.toml` | `cloud-egress-guard = []` | Feature-Flag vorhanden. |
| `crates/contextra-router/src/transport.rs` | `#[cfg(feature = "cloud-egress-guard")] HttpCloud` | `HttpCloud`-Transportvariante ist strikt hinter `#[cfg(feature = "cloud-egress-guard")]` abgesichert. |
| `crates/contextra-router/src/lib.rs` | `#[cfg(feature = "cloud-egress-guard")] pub use transport::HttpCloud;` | Exporte sind feature-gated. |
| `crates/contextra-engine/src/collection/search/hybrid/query.rs` | `HybridSearchStage::EgressFilter` | Internes Hybrid-Search Pipeline-Enum für Egress-Filterung. |
| `crates/contextra-mcp/src/sandbox.rs` | `allow_cloud_egress: bool` | Policy-Feld zur Steuerung von Cloud-Egress (Standard: `false`). |

---

## 3. Fazit & Maßnahme

Gemäß Vorgabe in Teil B (3):
> "Nur wenn es existiert und ohne Feature "guard" registriert ist: hinter das Feature stellen. Kein Eingriff, wenn es nicht existiert."

Da das Tool `contextra_cloud_query` am HEAD in `crates/contextra-mcp` nicht existiert, erfolgt **kein Code-Eingriff**.
