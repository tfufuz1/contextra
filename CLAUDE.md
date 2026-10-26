# Contextra — Claude / Antigravity Agent Guide

<!-- doc-stamp: 2026-10-26 | Normativ: AGENTS.md §1, docs/SPEC.md -->

> [!IMPORTANT]
> Claude und Antigravity: Lies AGENTS.md §2b (nicht §2a — das ist nur für Jules).
> Normative Rangfolge: SPEC.md > VETOES.md > capabilities.toml > governance/gates.toml > AGENTS.md

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

## Archiv-Crates — NICHT anfassen (WP 2.1)

`contextra-graph`, `contextra-cognition`, `contextra-adapt`, `contextra-kvcache`,
`contextra-router`, `contextra-agent`, `contextra-infer-onnx`, `contextra-infer-ollama`,
`contextra-core`

> [!CAUTION]
> Kein neuer Code für Archiv-Crates. Kein neues Abhängen davon (AGENTS.md §5.4).

## Gates (governance/gates.toml)

```bash
# Alle Gates prüfen
just gates

# Einzelnes Gate erklären
just gate ring-dag

# Pre-Push (entspricht merge-gate)
just pre-push
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
