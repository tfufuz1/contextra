# contextra-vault — Dormanzplan

**Normative Basis:** `docs/SPEC.md` §2.2 sowie lokale `AGENTS.md`. Dieses Verzeichnis ist ausdrücklich aus dem Workspace ausgeschlossen; seine frühere Tresor- und Schlüsselvernichtungsfunktion ist in `contextra-erasure::SagaVault` konsolidiert.

1. Keine Reaktivierung, keine Aufnahme als Workspace-Member, keine neue Abhängigkeit und keine Produktänderung ohne erfülltes Gate `G-REACTIVATE`.
2. Belasse Quellcode und `Cargo.toml` als historische Referenz; jede Sicherheitskorrektur erfolgt im aktiven Eigentümer `contextra-erasure`.
3. Vor einem eventuellen Reaktivierungsantrag: formale Architekturentscheidung, Ownership-Nachweis, vollständiger API-/Format-Migrationsplan, Erasure-Conformance und Sicherheitsreview vorlegen.

**Abnahme dieses Plans:** Das Crate bleibt in `exclude`; kein aktives Crate referenziert es, und kein `PLAN.md`-Schritt suggeriert eine implizite Reaktivierung.
