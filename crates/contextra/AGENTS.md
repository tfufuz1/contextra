Zentrale Module: builder (ContextraBuilder), boot, collection_profile, performance_profile, agent_memory, query_rewriter. Dokumentation in `API.md`.

Harte Regeln:
- Die öffentliche API ist ein Vertrag. Änderungen gehen durch `API.md`, Changelog und semver-Prüfung.
- Der Builder validiert früh. Ungültige Konfigurationen schlagen beim Bauen fehl, nicht beim ersten Schreibzugriff.
- Tier-Presets (`deployment_tier_presets_valid`) müssen immer valide sein. Ein Test pro Preset ist Pflicht.
- Regulierte Standardwerte (`regulated_auto_extraction_default`) sind sicherheitsrelevant. Änderungen nur mit ausdrücklicher Begründung.
- Löschung (`deletion_e2e`) und Tenant-Isolation (`tenant_isolation_facade`) sind Kernversprechen. Regressionen sind Blocker.

Verifikation: `cargo test -p contextra --locked`, `cargo test -p contextra --all-features`, `cargo doc -p contextra --no-deps`, Beispiel `examples/embedded_facade.rs` ausführen.

Optimierungsfokus: Einfacher Einstieg mit sinnvollen Defaults, keine versteckten Hintergrund-Tasks ohne Konfiguration, schlanker Standard-Feature-Satz.
