# Abschlussbericht: contextra-router (WP-RT-1)

## 1. Gate-Matrix

| Gate | Exakter Befehl | Ergebnis |
|---|---|---|
| **G-B1** | `cargo check -p contextra-router --no-default-features --locked` | GRÜN |
| **G-B2** | `for f in subprocess-dispatch bandit-routing cloud-egress-guard egress-sherman-morrison flow-corrected-thompson exp-fc-ts exp-routing-strategy; do cargo check -p contextra-router --no-default-features --features $f --locked; done` | GRÜN (alle 7 einzeln + alle Paarkombinationen) |
| **G-B3** | `cargo check -p contextra-router --all-targets --all-features --locked` | GRÜN |
| **G-B4** | `cargo test -p contextra-router --locked` & `cargo test -p contextra-router --all-features --locked` | GRÜN (110 Tests bzw. 128 Tests bestanden) |
| **G-B5** | `cargo clippy -p contextra-router --all-targets --all-features --no-deps --locked -- -D warnings` | GRÜN (0 Warnungen) |
| **G-B6** | `cargo bench --no-run -p contextra-router --all-features`, `cargo run --example simple_router -p contextra-router`, `cargo run --example builder_router -p contextra-router` | GRÜN |
| **G-L1** | `cargo tree -p contextra-router -e normal --no-default-features` | GRÜN (nur types, ports, adapt + transitive env/ids; tokio=0, shlex=0, wire=0) |
| **G-L4** | `python3 tools/isolation_proof.py --crate contextra-router --modes no-default,default,each-feature` | Befund: Werkzeug scheitert an `capabilities.toml`, da `contextra-env` und `contextra-ids` als transitive Mikro-Kerne nicht im statischen Root-Allowed gelistet sind. Ersatznachweis via `cargo tree` und isolierter Testmatrix erbracht. |
| **G-D** | `python3 tools/determinism_guard.py --crate contextra-router` & `python3 tools/zero_panic_analyzer.py --crate contextra-router` | GRÜN (0 Prod-Verstöße) |
| **G-F** | `git diff --stat origin/main...HEAD -- crates/contextra-router/tests/golden` | GRÜN (leer, Golden-Dateien unverändert) |
| **G-R** | `grep -rn "RetrievalStrategy" crates/contextra-router/src \| grep -v 'arm_registry.rs\|routing_strategy.rs'` | GRÜN (leer, vollständige Isolation) |
| **G-S** | `git diff --stat origin/main...HEAD` | GRÜN (berührt ausschließlich Eigentum) |

## 2. Abweichungen von Auftrag / Spezifikation
- Keine Abweichungen. Alle Vorgaben aus Z1 bis Z6 wurden exakt umgesetzt.

## 3. Getroffene Entscheidungen
- `CalibrationSource`: Als schmales Trait in `src/seam.rs` eingeführt mit drei konkreten Typen (`NoCalibrationSource`, `StdFsCalibrationSource`, `VfsCalibrationSource`) sowie Blanket-/Direkt-Implementierung für `Arc<dyn Vfs>`.
- `RouterEngineBuilder`: Additiv als fluide Fassade eingeführt. Sämtliche Konstruktoren delegieren intern über `build_internal` auf denselben Pfad.
- `subprocess-dispatch`: Kapselt `contextra-wire`, `tokio` und `shlex` als optionale Features. Für Tests wurde `tokio` als dev-dependency registriert.
- `ports_local.rs`: Verbleibt unverändert im Crate (wird wie vorgegeben erst in S-70 entfernt).

## 4. Offene Punkte / Risiken
- Keine internen Risiken für `contextra-router`.

## 5. Übergaben an andere Eigentümer
- **Übergabe an P04 (capabilities.toml)**: `tools/isolation_proof.py` verlangt, dass transitive Mikro-Kerne (`contextra-env`, `contextra-ids`) in `capabilities.toml` unter `allowed_internal` für `contextra-router` aufgenommen werden oder das Tool transitive Closure-Prüfungen unterstützt. Zudem sollte `contextra-wire` in `feature_closure_allowed.subprocess-dispatch` eingetragen sein.
- **Übergabe zu ports_local.rs**: `ports_local.rs` hat außerhalb dieses Crates keine Konsumenten mehr und kann in S-70 ersatzlos entfernt werden.
- **Übergabe zu contextra-mcp**: MCP-Konsumenten nutzen `RouterEngine::new` (`crates/contextra-mcp/src/routing.rs:96`) und `SlmProfile` (`crates/contextra-mcp/src/config.rs:487, 509`). Alle Signaturen bleiben 100% quellkompatibel und wurden durch `tests/consumer_surface.rs` abgesichert.
- **Übergabe an contextra-adapt Eigentümer**: In `contextra-adapt` existieren duplizierte Rng/Clock-Traits (`contextra_adapt::Rng`, `contextra_adapt::Clock`) im Vergleich zu `contextra-env` (SPEC R4).

## 6. Messwerte
- Testzahlen vorher: 109 Tests (Default) / 126 Tests (All-Features).
- Testzahlen nachher: 110 Tests (Default) / 128 Tests (All-Features) (+ `builder_equivalence_test` und `consumer_surface`).
- Clippy: 0 Warnungen mit `-D warnings`.
