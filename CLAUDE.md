# Contextra — Claude / Antigravity Agent Guide

<!-- doc-stamp: 2026-10-29 0796b40ed | Normativ: CONSTITUTION.md, SPEC.md, AGENTS.md -->

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
| 6 | Unsafe nur in 3 Inseln | unsafe-inventory Gate |
| 7 | Mandant aus RequestContext | no-literal-secrets Gate |
| 8 | Sicherheit kein Feature-Schalter | no-security-stub Gate |
| 9 | Messen vor Behaupten | deletion-honesty Gate |
| 10 | Fail-Closed | code-lints Gate |
| 11 | Eine Typfamilie je Konzept | symbol-uniqueness Gate |
| 12 | Ein PR = ein WP | scope-guard Gate |

## Baukasten-Module [M] (SPEC §4.2, §5)

`contextra-graph`, `contextra-cognition`, `contextra-adapt`, `contextra-kvcache`,
`contextra-router`, `contextra-agent`, `contextra-infer-onnx`, `contextra-infer-ollama`,
`contextra-privacy`, `contextra-mcp`, `contextra-py`, `contextra-sandbox`

> [!NOTE]
> Gemäß SPEC 3.0 / ADR-D01 sind diese Crates Baukasten-Module [M] mit eigenem Vertrag und isoliertem Prüfauftrag, kein Archiv.

## Gates (`cargo xtask`)

```bash
# Alle aktiven Gates prüfen
cargo run -p xtask -- check

# Einzelnes Gate prüfen
cargo run -p xtask -- gate check --gate ring-dag

# Liste aller Gates
cargo run -p xtask -- --list-gates
```

## Commit-Konventionen

```
WP-x.y(crate): Kurze Beschreibung

- Was geändert und warum (SPEC §x.y)
- Gate-Status vorher → nachher

Gates: ring-layering=🟢 no-security-stub=🔴→🟡
Closes: #<issue>
```

## Branch-Naming

```
wp/x.y-kurzbeschreibung       # z.B. wp/2.1-archive-graph
fix/gate-name-beschreibung    # z.B. fix/ring-layering-engine-dep
```

## Governance-Referenzen

| Datei | Zweck |
|-------|-------|
| `governance/gates.toml` | Gate-Katalog — Single Source of Truth (50+ Gates) |
| `governance/ratchet.toml` | Qualitäts-Baselines (dürfen nur sinken, REQ-QA-005) |
| `governance/security_ports.toml` | Pflicht-Sicherheitsports (INV-WIRING-2) |
| `governance/key_purposes.toml` | HKDF-Labels und Schlüssel-Zweckbindung (INV-KEYPURPOSE-1) |
| `governance/clippy.toml` | P28-Verbotsliste (disallowed-methods/-types) |
| `governance/determinism-allow.toml` | Erlaubte Nicht-Determinismus-Ausnahmen (ADR-098) |
| `governance/protected-paths.toml` | Geschützte Pfade (REQ-GOV-002) |
| `governance/README.md` | Übersicht und Konsumenten-Dokumentation |
