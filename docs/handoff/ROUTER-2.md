# Handoff ROUTER-2: contextra-router Härtung

Stand: 2026-11-10
Commit-Kennung: RT2
Branch: `jules/router-2-harden`

## 1. Status je Arbeitspaket

| WP | Status | Prüfbefehl | Anmerkung |
|---|---|---|---|
| RT2.0 Baseline und Befund | Grün | `cargo check -p contextra-router --all-targets --locked && test -f docs/decoupling/baseline/router/test-router.txt` | Baseline erfasst in `docs/decoupling/baseline/router/` |
| RT2.1 Golden-/Orakel-Korpus | Grün | `cargo test -p contextra-router --test golden_corpus_test --locked` | Korpus mit MANIFEST.sha256 & Rückwärtslesetest verifiziert |
| RT2.2 wire optional | Grün | `cargo check -p contextra-router --all-targets --no-default-features --locked` | `contextra-wire` optional hinter `subprocess-dispatch` |
| RT2.3 Vfs-Stelle & RetrievalStrategy | Grün | `cargo test -p contextra-router --locked` | `ports::vfs` durch lokale `&Path`-Funktion ersetzt, Differenztest aktiv |
| RT2.4 Experiment-Features | Grün | `cargo check -p contextra-router --all-targets --all-features --locked` | `exp-fc-ts`, `exp-routing-strategy` deklariert |
| RT2.5 Härtung & Zero-Panic | Grün | `python3 tools/zero_panic_analyzer.py --crate contextra-router --fail-on-prod --json` | 0 Panics in Prod, BanditError-Propagierung gefixt |
| RT2.6 Dokumentation & Abschluss | Grün | `cargo clippy -p contextra-router --all-targets -- -D warnings` | AGENTS.md & Handoff aktualisiert, Wortprüfung grün |

## 2. Abweichungen von SPEC-v2

Keine Abweichungen.

## 3. Koordinationsanforderungen

1. `crates/contextra-mcp/Cargo.toml`: Falls `contextra-mcp` zukünftig `dispatch_to_slm` aufruft, muss das Feature `subprocess-dispatch` von `contextra-router` im MCP-Manifest aktiviert werden.
2. Workspace Baseline: `contextra-store` kompiliert im unveränderten HEAD nicht aufgrund von Vorarbeiten anderer Missionsstränge (`sstable/block_cache.rs` und `scanner/mod.rs`). Wurde strikt unberührt gelassen.

## 4. API-Freeze-Tabelle

| Typ / Funktion | Aufrufer extern | Kompatibilitätsstatus |
|---|---|---|
| `contextra_router::RouterEngine` | `contextra-mcp`, `contextra-agent` | Unverändert erhalten |
| `contextra_router::DefaultRouterEngine` | `contextra-mcp`, `contextra-agent` | Unverändert erhalten |
| `contextra_router::SlmProfile` | `contextra-mcp` | Unverändert erhalten |
| `contextra_router::DecisionId` | `contextra-agent` | Unverändert erhalten |
| `contextra_router::RoutingOutcome` | `contextra-agent` | Unverändert erhalten |
| `contextra_router::ports_local::{...}` | `contextra-mcp` | Unverändert erhalten (nicht angefasst) |
| `contextra_router::dispatch_to_slm` | Intern / Tests | Hinter Feature `subprocess-dispatch` |

## 5. Kern-API-Lücken

Keine.

## 6. Messwerte Vorher / Nachher

### Vorher (RT2.0)
- Kern-LOC: 7.784 (mit Abhängigkeiten: 31.969; Standalone Router: 2.247)
- Nullprüfzahlen:
  - `contextra_types`: prod=47, inline_test=1, tests_dir=5
  - `contextra_ports`: prod=11, inline_test=0, tests_dir=8
  - `contextra_testkit`: prod=0, inline_test=0, tests_dir=0
- Zero-Panic: 3 Unwraps in `src/router/tests.rs` (fälschlich als Produktion bewertet vor `#[cfg(test)]`-Dateikopf), 0 echtes Produktions-Panic.
- Feature `bandit-routing`: schlug vor RT2.5 fehl wegen `ContextraError::from(err)` mit `BanditError`.

### Nachher (RT2.6)
- Kern-LOC: 2.280 Zeilen (16 Dateien in contextra-router)
- Nullprüfzahlen:
  - `contextra_types`: prod=47, inline_test=1, tests_dir=5
  - `contextra_ports`: prod=11, inline_test=0, tests_dir=8 (`vfs`-Import im Kern entfernt)
  - `contextra_testkit`: prod=0, inline_test=0, tests_dir=0
- Zero-Panic: 0 Unwraps / Panics im Produktionscode (`zero_panic_analyzer --fail-on-prod` ist grün).
- Alle Features (`subprocess-dispatch`, `bandit-routing`, `flow-corrected-thompson`, `cloud-egress-guard`, `exp-fc-ts`, `exp-routing-strategy`) kompilieren und testen sauber.

## 7. Offene Risiken

Keine offenen Risiken.
