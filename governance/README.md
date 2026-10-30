# governance/ — Single Source of Truth für Gates & Policies

> **Status 30.10.2026:** Gate-Katalog enthält Gates für entfernte Komponenten. Angleichung in `docs/PLAN.md` Phase R2.

<!-- doc-stamp: 2026-10-26 | Normativ: docs/SPEC.md §17, §18.2, AGENTS.md §3 -->

> [!IMPORTANT]
> Dieses Verzeichnis ist ein **geschützter Pfad** (protected-paths.toml).
> Änderungen erfordern ADR-Trailer im Commit und das PR-Label `protected-path`.

## Dateien

| Datei | Zweck | Normativ | WP |
|-------|-------|----------|-----|
| `gates.toml` | Gate-Katalog — alle Gates aus SPEC §17.3 + Repo-Gates | SPEC §17, REQ-QA-010 | WP 0.4 |
| `ratchet.toml` | Qualitäts-Baselines (nur sinkend) | SPEC §17, REQ-QA-005, P7 | WP 0.4 |
| `protected-paths.toml` | Geschützte Dateipfade | REQ-GOV-002, §18.2 | WP 0.4 |
| `security_ports.toml` | Pflicht-Sicherheitsports (Kern + Rand) | INV-WIRING-2, PLAN.md WP 0.4 | WP 0.4 |
| `key_purposes.toml` | HKDF-Labels und Schlüssel-Zweckbindung | INV-KEYPURPOSE-1, SPEC §12.3 | WP 0.4 |
| `clippy.toml` | Workspace Clippy: disallowed-methods/-types (P28) | SPEC §17.3, PLAN.md K-13 | WP 0.4 |
| `determinism-allow.toml` | Erlaubte Nicht-Determinismus-Ausnahmen | P28, ADR-098 | WP 0.4 |

## Rangfolge (AGENTS.md §1 + SPEC §18.7)

1. `docs/SPEC.md` — normativ, einzige Wahrheit
2. `docs/VETOES.md` — Vetos und Architekturprinzipien
3. `capabilities.toml` — Crate-Ring-Zuordnung
4. `governance/gates.toml` — Gate-Definitionen (Single Source of Truth)
5. `AGENTS.md` — Agenten-Betriebsanleitung
6. `governance/ratchet.toml` — Qualitäts-Baselines

## Gate-Klassifikation

| Tier | Trigger | Blockiert | Beispiele |
|------|---------|-----------|-----------|
| `pr` | Pull Request | Ja (Merge verhindert) | ring-dag, panic-surface, clippy-lintset |
| `merge` | Merge-Queue | Ja (nach PR-Approval) | deny, public-api, feature-matrix |
| `nightly` | Scheduled (täglich) | Nein (informativ) | audit, sbom, evidence-complete |
| `heavy` | Scheduled (heavy) | Nein | fuzz-matrix, miri, loom-inventory, mutation |

## Gate-Aliase (K-14)

SPEC §17.3 verwendet teilweise andere Namen als der Gate-Katalog:

| SPEC-Name | gates.toml Name | Feld `aliases` |
|-----------|-----------------|----------------|
| `ring-layering` | `ring-dag` | `["ring-layering"]` |
| `ratchet-panic` | `panic-surface` | `["ratchet-panic"]` |
| `ring0-async-purity` | `async-io` | `["ring0-async-purity"]` |

## Konsumenten

- `cargo xtask` — **entfernt** (Commit `068261abd`). Die Auswertung von `gates.toml`, `ratchet.toml`, `security_ports.toml` und `key_purposes.toml` ist ungeklärt (D-R2 in `docs/SPEC.md` §15).
- `justfile` — Ziel nach PLAN R2.4: direkte `cargo`-Befehle statt Delegation an `xtask`
- `.github/workflows/merge-gate.yml` — blockierend, liest gates.toml
- `.github/workflows/deep-gates.yml` — nicht blockierend
- `clippy.toml` → wird von `cargo clippy` im Workspace-Root konsumiert

## Änderungsprotokoll

Alle Änderungen an diesem Verzeichnis erscheinen in `docs/decisions/` als ADR.
Quality Gates dürfen niemals geschwächt werden (REQ-QA-010).
