# OPEN_ISSUES (WP-12 / P30)

## Externe Verwendungen von archivierten Beweis-Symbolen in Fremd-Crates [V]

Gemäß Besitzprinzip (Regel 2) und Zwei-Phasen-API-Regel (Regel 4) dürfen Dateien außerhalb von BESITZ (`crates/contextra-crypto/`) nicht modifiziert werden.

Folgende Alttests in fremden Crates referenzieren noch archivierte Beweis-Symbole aus `contextra-crypto`:
1. `crates/contextra-store/tests/campaign_j42_audit.rs`:
   - Referenziert `DeletionProofKeyPair` aus `contextra_crypto`.
   - Maßnahme: Eintrag in `PENDING_API_CHANGES.md`. In Phase 4 (P40) auf `contextra-erasure` oder lokalen Mock umstellen.
2. `crates/contextra-db/tests/deletion_proof_integration.rs` & `e2e_compliance_scenario.rs`:
   - Referenzieren `contextra_crypto::deletion_proof::*`.
   - `contextra-db` gehört nicht zu BESITZ.
   - Maßnahme: Eintrag in `PENDING_API_CHANGES.md`.
3. `crates/contextra-engine/tests/erasure_report_honesty.rs` & `proof_verify_not_constant.rs`:
   - Referenzieren `contextra_crypto::deletion_proof::*`.
   - `contextra-engine` gehört nicht zu BESITZ.
   - Maßnahme: Eintrag in `PENDING_API_CHANGES.md`.

## Vorherige Workspace-Fehler [V]
- `crates/contextra-graph` hat Namenskollision `contract`, wodurch abhängige Crates (`engine`, `db`, `mcp`, `py`, `agent`, `cognition`) in `BUILD_LADDER.md` als rot vermerkt sind.
- `crates/contextra-infer-candle`: fehlt `thiserror` (dokumentiert in `P00/BUILD_LADDER.md`).
