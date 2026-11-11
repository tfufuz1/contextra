# Entkopplungs-Architektur & Gate-Infrastruktur

Normativ: `docs/SPEC.md` (v3), `docs/decoupling/BAUKASTEN_LEITFADEN.md`, `docs/decoupling/BAUKASTEN_ARCHITEKTURENTSCHEIDUNG.md`, `docs/decoupling/BAUKASTEN_UMSETZUNGSPLAN.md`.

## Dokumentations-Übersicht für Implementierer
- **Leitfaden:** [`docs/decoupling/BAUKASTEN_LEITFADEN.md`](BAUKASTEN_LEITFADEN.md) – Verbindliche 10 Gebote, Schichtenmodell E0–E3, Prüfbefehle und Autonomieregeln.
- **Architekturentscheidungen (AD-1 bis AD-10):** [`docs/decoupling/BAUKASTEN_ARCHITEKTURENTSCHEIDUNG.md`](BAUKASTEN_ARCHITEKTURENTSCHEIDUNG.md) – Analysebefunde, Seams (`ids`/`env`), Fehlerstrategie, Crate-Schnitte.
- **Umsetzungsplan (WP-00 bis WP-91):** [`docs/decoupling/BAUKASTEN_UMSETZUNGSPLAN.md`](BAUKASTEN_UMSETZUNGSPLAN.md) – Detaillierte Arbeitspakete, Spuren A–D, Abhängigkeiten und Abnahmekriterien.
- **Prompt-Katalog:** [`docs/prompts/MODULAR_DECOUPLING_PROMPTS.md`](../prompts/MODULAR_DECOUPLING_PROMPTS.md) – Autarke Prompts für Coding-Agenten je Crate.

## 1. Stufenplan (Roadmap)
- **S-00 / S-01 (WP-F0):** Baseline-Erhebung, Gate-Infrastruktur (`gate_layering.py`, `isolation_proof.py`, `feature_audit.py`), CI-Härtung, Root-Metadaten (`capabilities.toml` Schema v3, `ratchet.toml`), Workspace-Gerüste (`prompt-guard`, `vault`).
- **S-10 / S-11 (FND-1):** Autarke Mikro-Kerne `contextra-ids` und `contextra-env`.
- **S-12:** Feature-Hygiene, Defaults und ungenutzte Features bereinigen.
- **S-20 bis S-39:** Entkopplung der Einheiten (E0/E1 Bausteine: `crypto`, `mvcc`, `text`, `vector`, `store`, etc.).
- **S-50 / S-51:** Typ- und Port-Migrationen (`ports` -> `license`, `erasure`, `testkit`).
- **S-60 bis S-64:** Integration & Orchestrierung (`adapt`, `engine`, `db`, `mcp`, `py`).
- **S-70:** Bereinigung deprecated Re-Exports und veralteter API-Pfade.

---

## 2. Eigentumsmatrix (Governance & Schichten)

| Ebene / Gruppe | Crates | Eigentümer / Regeln |
|---|---|---|
| Foundation (E0) | `ids`, `env`, `sys`, `durable-fs`, `simd`, `wire`, `adapt`, `infer-onnx`, `ingest`, `privacy`, `sandbox`, `prompt-guard` | Keine internen Abhängigkeiten. Autark isolierbar (G-L4). |
| Foundation & Storage (E1) | `mvcc`, `crypto`, `store`, `vector`, `text`, `graph`, `rank`, `checkpoint`, `kvcache`, `infer-ollama`, `infer-candle`, `vault` | Abhängigkeiten nur auf E0 (und lokale Ausnahmen wie `store` auf E0+E1). |
| Contracts & Integration (E2) | `types`, `ports`, `router`, `agent`, `cognition`, `engine`, `db`, `contextra` | Zusammensetzung und Cross-Crate Schnittstellen. |
| Applications & Test (E3) | `mcp`, `py`, `testkit`, `bench` | Endanwender, Bindings und Test-Harnesses. |
| Compliance (Frozen) | `erasure`, `license`, `audit-export`, `avv-generator` | Tier `frozen` gemäß Entscheidung D-08. Keine Rechtszusicherungen. |

---

## 3. Kanonische Gate-Befehle

| Gate | Befehl | Zweck |
|---|---|---|
| **G-L1** | `python3 tools/gate_layering.py --all`<br>`just gate-layering --all` | Prüft Kern-Closure, Feature-Closure, Ebenen $E_n \to E_m$ ($m \le n$), dev-Abhängigkeiten. Exit 0 = grün, 1 = Verstoß, 2 = Werkzeugfehler. |
| **G-L2** | `python3 tools/count_bridge_refs.py -c <crate>` | Null-Prüfung für direkte Bridge-Referenzen (`types|ports|testkit`) außerhalb von `compat/`. |
| **G-L3** | `cargo check --workspace --all-targets --locked`<br>`cargo check -p <crate> --features docid-128 --locked` | Workspace-Kompilierung und ID-Breiten-Matrix. |
| **G-L4** | `python3 tools/isolation_proof.py -c <crate> --modes no-default,default,each-feature`<br>`just isolation <crate>` | Vollständige Isolationsprüfung im temporären Workspace. |
| **G-U1** | `governance/ratchet.toml` | Zähler für `#![forbid(unsafe_code)]` (aktuell: 32). |
| **Feature-Audit** | `python3 tools/feature_audit.py --fail-on-unused`<br>`just feature-audit` | Erkennt ungenutzte Features ohne Code-Präsenz und ohne Weiterleitung. |
