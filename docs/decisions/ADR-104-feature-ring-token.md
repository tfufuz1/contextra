# ADR-104: Feature-Ring-Token (`AuthorizedRing`) und Unumgehbarkeit des Lizenz-Gates

- **Status:** Proposed
- **Datum:** 2026-09-28
- **Kontext:** Bisher prüfte `ContextraBuilder::build()` das Lizenz-Gate nur, wenn explizit `with_performance_profile` aufgerufen wurde. Bei Direktaufruf von `contextra::open()` oder `contextra::open_with_config()` wurde das Lizenz-Gate komplett umgangen.
- **Entscheidung:**
  1. Einführung des unkopierbaren/unimitierten Tokens `AuthorizedRing` in `contextra-ports::license` mit crate-internem Konstruktor.
  2. Erweiterung des `LicenseGate`-Traits um `authorize(&self, requested: FeatureRing) -> Result<AuthorizedRing, LicenseError>`.
  3. `ContextraBuilder::build()` fordert IMMER ein `AuthorizedRing`-Token an (Default: `FeatureRing::Fast`).
  4. Direkte Aufrufe von `contextra::open()` und `open_with_config()` delegieren an den Builder und erzwingen `deletion_proof_active = false` unter dem Open-Source-Ring (`FeatureRing::Fast`).
  5. Das aufgelöste `PerformanceProfile` überschreibt `durability_mode`, `deletion_proof_active` und `vector_delete_mode` in der `ContextraConfig`. Explizit widersprechende Nutzerkonfigurationen werfen `ContextraError::PolicyViolation`.
- **Konsequenzen:**
  - Lizenz-Sicherheitsprüfungen sind kryptografisch/typtechnisch an der Crate-Grenze garantiert.
  - Das Lizenz-Gate kann durch direkte Engine-Instanziierung nicht mehr umgangen werden.
