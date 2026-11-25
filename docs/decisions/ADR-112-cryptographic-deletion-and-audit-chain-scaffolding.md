# ADR-112: Status und Architekturanalyse des Sovereign Deletion Proof v3, Audit Chains & Key Shredding Scaffolding-Codes

* **Datum**: 2026-10-04
* **Status**: ⏳ Proposed / Pending Architectural Decision
* **Betroffene Komponenten**: `contextra-crypto` (`crypto.rs`, `deletion_proof.rs`, `wal_crypto.rs`, `kv_shredding.rs`, `audit_chain.rs`, `revocation_log.rs`, `kdf.rs`)
* **Spezifikationsreferenzen**: `CONTEXTRA_VOLLSTAENDIGE_FEATURE_SPEZIFIKATION.md` (§10.1, INV-DELETION-1, INV-DELETION-2), `docs/TYPE_REGISTRY.md`

---

## 1. Ausgangslage & Befund (Codebase Audit)

Im Rahmen des Auditings wurden 17 öffentliche Kryptografie-Funktionen in `contextra-crypto` identifiziert, die aktuell keine Aufrufer außerhalb des Crates haben.

### Kernbefunde:
1. **Löschbeweis v3 & Receipt Verification (`deletion_proof.rs`, `wal_crypto.rs`)**:
   `create_v3_with_audit_position`, `verify_wal_delete_receipt`, `set_last_seq_no` und `last_seq_no_snapshot` dienen der Erzeugung und Verifikation von Ed25519-signierten `DeletionProof` v3 Objekten (§10.1).
   *Architektur-Bezug*: Invarianten `INV-DELETION-1` und `INV-DELETION-2` schreiben vor, dass Löschbeweise erst nach physischer Bereinigung erzeugt werden dürfen und keine Geisterzeiger im Index verbleiben.
2. **Key Shredding & Scoped Key Derivation (`kv_shredding.rs`, `crypto.rs`, `kdf.rs`)**:
   `with_revocation_log`, `get_wrapped_kek`, `get_wrapped_dek`, `is_record_active`, `derive_segment_key`, `derive_deletion_proof_key`, `derive_kv_key_scoped`, `cipher_for_scoped` und `generate_default` bilden das kryptografische Fundament für Crypto-Shredding via KEK/DEK Key-Hierarchien.
3. **Audit Chains & Revocation Log (`audit_chain.rs`, `revocation_log.rs`)**:
   `sign_head`, `verify_head_signature`, `new_in_memory` und `verify_integrity` ermöglichen unveränderbare Audit-Ketten für Compliance-Nachweise.

---

## 2. Architekturanalyse & Kontext

`contextra-crypto` ist ein Ring-0-Crate mit `#![forbid(unsafe_code)]`. Der Code bildet das Alleinstellungsmerkmal von Contextra als "kryptografisch beweisbare Memory-Engine" im Sovereign-Ring ab. Er ist zu 100% unit- und property-getestet (u.a. gegen Timing-Seitenkanäle via `ed25519_dalek`). P28 verlangt, dass kryptografisches Schlüsselmaterial echten CSPRNG verwendet.

---

## 3. Handlungsoptionen für die menschliche Entscheidung

### Option A: Vollständige Anbindung der Audit Chain Head Signatures im Sovereign-Ring
* **Beschreibung**:
  Integrieren von `sign_head` und `create_v3_with_audit_position` in den automatischen Transaktionsabschluss des `Sovereign`-Rings in `contextra-engine`.
* **Aufwandsschätzung**: **3 bis 4 Personenwochen (120 - 160 Stunden)**
  *(Inkl. Audit Position Verification, Cross-Segment Validation und Verifikations-Benchmarks)*.
* **Pro**:
  - Lückenlose kryptografische Audit-Kette für Aufsichtsbehörden und B2B2G-Kunden.
  - Höchste Stufe der Fälschungssicherheit.
* **Contra**:
  - Geringfügiger Overheads bei Transaktions-Commits im Sovereign-Ring.

### Option B: Rückbau von v3 Audit Positions & Revocation Log Scaffolding
* **Beschreibung**:
  Entfernen der `create_v3_with_audit_position`-Spezialpfade und Vereinfachung von `audit_chain.rs`.
* **Aufwandsschätzung**: **1 bis 2 Personentage (8 - 16 Stunden)**
  *(Inkl. Bereinigung von contextra-crypto)*.
* **Pro**:
  - Reduzierter Krypto-Codeumfang.
* **Contra**:
  - Verlust von erweiterten Audit-Positions-Nachweisen.

### Option C: Beibehaltung des Ist-Zustands als Krypto-Scaffolding (Status Quo / Empfehlung)
* **Beschreibung**:
  Belassen der Methoden in `contextra-crypto`. Die Primitive bleiben vollständig verifizierbar und betriebsbereit.
* **Aufwandsschätzung**: **0 Stunden**
* **Pro**:
  - Erfüllung aller Invarianten (INV-DELETION-1/2).
  - Volle kryptografische Abdeckungsgarantie ohne Laufzeitkosten im Fast-Ring.
* **Contra**:
  - Vorhandensein von ungenutzten Krypto-Hilfsmethoden.

---

## 4. Empfehlung des Architects

1. **Kurz- bis Mittelfristig (Status Quo - Option C)**:
   Belassen der Methoden im Crate. Sie gewährleisten die Verifizierbarkeit des Sovereign-Rings.

2. **Langfristig**:
   Anbindung der Audit-Head Signatures im Rahmen von Compliance-Behörden-Piloten (Option A).
