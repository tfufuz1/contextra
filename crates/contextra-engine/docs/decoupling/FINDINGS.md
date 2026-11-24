# FINDINGS — contextra-engine (S-64)

Vorbestehende Befunde und Übergaben an Parallelaufträge / Folgeaufträge:

1. **`tests/decay_and_expiry.rs` (ENG-B Eigentum)**:
   - Datei gehört laut Auftragsdefinition ENG-B und darf in ENG-A nicht modifiziert werden.
   - Initialisierer von `DecayControllerConfig` in Zeile 19 fehlt das Feld `base_half_life_ms: None`.
   - Übergabe an ENG-B zur Anpassung.

2. **Root-`capabilities.toml` Abweichungen (F0 / Z Eigentum)**:
   - Root-`capabilities.toml` führt `allowed_internal` für `contextra-engine` mit `contextra-kvcache`, das laut SPEC §3.1 und Manifest keine Abhängigkeit ist.
   - `contextra-testkit` fehlt in `dev_allowed`.
   - Übergabe an F0/Z zur Aktualisierung der Root-Metadaten.

3. **Lizenz-Traits in `contextra-ports`**:
   - `src/lib.rs` und `src/contextra_impl/lifecycle.rs` importieren `LicenseGate` aus `contextra-ports::license`.
   - Feature `license` und Abhängigkeit `contextra-license` können erst nach formaler Abgabe durch `ports` eingeführt werden.

4. **`deletion_proof_active` Bezeichner**:
   - `ContextraConfig.deletion_proof_active` ist ein öffentliches Feld.
   - Um Quellkompatibilität für Downstream-Konsumenten (`contextra-db`, `contextra-cognition`) zu wahren (INV-2), wird der Bezeichner nicht umbenannt, sondern dessen Semantik neutralisiert.
