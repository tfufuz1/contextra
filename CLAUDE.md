# Contextra — Claude / Antigravity Agent Guide

<!-- doc-stamp: 2026-10-29 10cc17922 | Normativ: CONSTITUTION.md, SPEC.md, AGENTS.md -->

> [!IMPORTANT]
> Claude und Antigravity: Lies AGENTS.md §2b (nicht §2a — das ist nur für Jules).
> Normative Rangfolge: CONSTITUTION.md > SPEC.md > PLAN.md > capabilities.toml > AGENTS.md

## Schnellstart für Claude/Antigravity

```bash
# 1. Build-Grundlage prüfen (AGENTS.md §2b)
cargo check --workspace

# 2. SPEC §0 lesen (LLM-Schnellstart)
# Datei: docs/SPEC.md — Abschnitt §0

# 3. Ring/Klassen-Zuordnung prüfen
cat capabilities.toml
```

## Die 12 Regeln (AGENTS.md §4) — Kurzreferenz

| # | Regel | Verletzung = Gate-Fehler |
|---|-------|--------------------------|
| 1 | Ring und Klasse zuerst | ring-dag Gate |
| 2 | Kein Panic in Produktion | panic-surface Gate |
| 3 | Zeit/Zufall/IDs nur über Ports | determinism Gate |
| 4 | Alles Persistente geht durch WAL | deletion-honesty Gate |
| 5 | I/O nur über `Vfs` | security-ports Gate |
| 6 | Unsafe nur in 2 Inseln | unsafe-inventory Gate |
| 7 | Mandant aus RequestContext | no-literal-secrets Gate |
| 8 | Sicherheit kein Feature-Schalter | no-security-stub Gate |
| 9 | Messen vor Behaupten | deletion-honesty Gate |
| 10 | Fail-Closed | code-lints Gate |
| 11 | Eine Typfamilie je Konzept | symbol-uniqueness Gate |
| 12 | Ein PR = ein WP | scope-guard Gate |

## Baukasten-Crates (Reduktion 1.0)

Das Repository wurde auf 14 Crates unter `crates/` reduziert:
`contextra`, `contextra-crypto`, `contextra-db`, `contextra-engine`, `contextra-mvcc`,
`contextra-ports`, `contextra-rank`, `contextra-simd`, `contextra-store`, `contextra-sys`,
`contextra-testkit`, `contextra-text`, `contextra-types`, `contextra-vector`.

Entfernte Crates (`graph`, `cognition`, `adapt`, `kvcache`, `router`, `agent`, `infer-*`,
`sandbox`, `py`, `license`, `core`, `ingest`, `mcp`, `wire`, `checkpoint`, `privacy`,
`audit-export`, `avv-generator`) liegen in der Git-Historie (Tag `snapshot/pre-cut` bzw.
Commit `eb10652f`) und werden nicht neu angelegt.

## Validierung (Cargo statt xtask)

`cargo xtask` wurde entfernt. Validierungen erfolgen ausschließlich über standardisierte Cargo-Befehle:

```bash
cargo fmt --check
cargo check --workspace
cargo clippy --workspace --all-targets
cargo test --workspace
cargo deny check
```

## Commit-Konventionen

```
WP-x.y(crate): Kurze Beschreibung

- Was geändert und warum (SPEC §x.y)
- Validation-Status vorher → nachher

Closes: #<issue>
```

## Branch-Naming

```
wp/x.y-kurzbeschreibung       # z.B. wp/2.1-refactor
fix/beschreibung              # z.B. fix/engine-dep
```

## Governance-Referenzen

| Datei | Zweck |
|-------|-------|
| `governance/gates.toml` | Gate-Katalog — Single Source of Truth |
| `governance/ratchet.toml` | Qualitäts-Baselines (dürfen nur sinken, REQ-QA-005) |
| `governance/security_ports.toml` | Pflicht-Sicherheitsports (INV-WIRING-2) |
| `governance/key_purposes.toml` | HKDF-Labels und Schlüssel-Zweckbindung (INV-KEYPURPOSE-1) |
| `governance/clippy.toml` | P28-Verbotsliste (disallowed-methods/-types) |
| `governance/determinism-allow.toml` | Erlaubte Nicht-Determinismus-Ausnahmen (ADR-098) |
| `governance/protected-paths.toml` | Geschützte Pfade (REQ-GOV-002) |
| `governance/README.md` | Übersicht und Konsumenten-Dokumentation |
