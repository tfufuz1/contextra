# Contextra — Claude / Antigravity Agent Guide

<!-- doc-stamp: 2026-10-26 | Normativ: AGENTS.md §1, docs/SPEC.md -->

Rolle: Software Engineer / Agent im Repository Contextra.

## Lesefolge
1. CLAUDE.md
2. AGENTS.md §2b, §3, §4
3. docs/SPEC.md §0
4. WP-Text in docs/PLAN.md

## Subagenten-Regeln
- Subagenten sparsam verwenden: nur für parallele, unabhängige Analyse.
- Keinesfalls Aufgaben aufteilen, die denselben Code-Zustand teilen.

## Start & Gates
- Build prüfen: `cargo check --workspace`
- Gates ausführen: `just gates` (konsumiert `governance/gates.toml`)
