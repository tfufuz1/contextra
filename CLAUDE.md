# Contextra – Claude / Antigravity Agent Guide

<!-- doc-stamp: 2026-10-30 | Normativ: docs/CONSTITUTION.md, docs/SPEC.md (4.0), AGENTS.md -->

> [!IMPORTANT]
> Build-Status (30.10.2026): **nicht grün.** Vor jeder Änderung prüfen, ob die Aufgabe zu Phase R0 in `docs/PLAN.md` gehört. Andere Arbeit wird bis zum grünen Build zurückgestellt.
>
> Rangfolge: CONSTITUTION > SPEC > PLAN > Quelltext > `capabilities.toml` und `governance/` > AGENTS.md > CLAUDE.md

## Schnellstart

```bash
# 1. Workspace lädt?
cargo metadata --no-deps --format-version 1 > /dev/null

# 2. Kompiliert?
cargo check --workspace --all-targets

# 3. Ringe und Abhängigkeiten
#    docs/SPEC.md §4.1 und §4.2
```

Der Befehl `cargo xtask` existiert seit Commit `068261abd` nicht mehr. Befehle aus älteren Dokumenten sind ungültig.

## Die zwölf Regeln (SPEC §3.2, Kurzreferenz)

| # | Regel | Prüfung (Ist, 30.10.2026) |
|---|---|---|
| 1 | Ring und Klasse zuerst | `tests/layering.rs` (Angleichung R2) |
| 2 | Kein Panic in Produktion | Clippy-Lints `unwrap_used`, `expect_used`, `panic`, `indexing_slicing` |
| 3 | Zeit, Zufall, IDs nur über Ports | Review |
| 4 | Persistentes läuft über das WAL | Tests in `contextra-store` |
| 5 | I/O nur über `Vfs`; `fsync`-Fehler propagieren | Review, Clippy (R1) |
| 6 | Unsafe nur in `contextra-sys` und `contextra-simd` | `tests/unsafe_islands.rs` |
| 7 | Mandant aus `RequestContext` | `contextra/tests/tenant_isolation_facade.rs` |
| 8 | Sicherheit ist kein Feature-Schalter | Test im Default-Build (R0.6) |
| 9 | Messen vor Behaupten | `contextra-bench` |
| 10 | Fail-Closed | Recovery-Tests in `contextra-store` |
| 11 | Eine Typfamilie je Konzept | Doppeltyp-Prüfung (R1.3) |
| 12 | Ein PR = ein Arbeitspaket | Review |

## Crates (Kern v1)

`contextra-types`, `contextra-ports`, `contextra-sys` (U), `contextra-simd` (U), `contextra-mvcc`, `contextra-crypto`, `contextra-vector`, `contextra-text`, `contextra-rank` (Ring 0) · `contextra-store` (Ring 1) · `contextra-engine` (Ring 2) · `contextra-db` (Ring 3) · `contextra` (Ring 4) · `contextra-testkit`, `contextra-bench` (Tooling)

Entfernte Crates (`contextra-graph`, `contextra-adapt`, `contextra-cognition`, `contextra-router`, `contextra-agent`, `contextra-infer-*`, `contextra-mcp`, `contextra-py`, `contextra-privacy`, `contextra-ingest`, `contextra-kvcache`, `contextra-sandbox`, `contextra-wire`, `contextra-checkpoint`, `contextra-license`, `contextra-audit-export`, `contextra-avv-generator`, `contextra-core`): nicht anlegen, nicht referenzieren. Siehe SPEC §5.2.

## Commit-Konventionen

```
WP-R0.3(contextra-engine): Residuen aus relate.rs entfernen

- Was geändert und warum (PLAN §2, SPEC §16 B2)
- Prüfstatus vorher → nachher (cargo check -p <crate>)
```

## Branch-Naming

```
wp/r0.3-engine-residuen      # Arbeitspaket aus PLAN.md
fix/build-xtask-member       # Fehlerbehebung außerhalb eines WP
```

## Governance-Dateien

| Datei | Zweck | Stand |
|---|---|---|
| `governance/gates.toml` | Gate-Katalog | Enthält Gates für entfernte Komponenten, Angleichung in R2 |
| `governance/ratchet.toml` | Qualitäts-Baselines (nur sinkend) | Nach R0 neu setzen |
| `governance/security_ports.toml` | Sicherheitsports | Angleichung in R2 |
| `governance/key_purposes.toml` | Schlüssel-Zweckbindung | Entfernte Zwecke streichen (R2) |
| `governance/clippy.toml` | Verbotsliste | Gültig |
| `governance/determinism-allow.toml` | Erlaubte Ausnahmen | Gültig, Pfade prüfen |
| `governance/protected-paths.toml` | Geschützte Pfade | `xtask/**` entfernen (R2) |
| `capabilities.toml` | Ring- und Reifegrad-Manifest | 98 Verweise auf Entferntes, Neuerzeugung in R2 |
