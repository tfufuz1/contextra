# Governance Rule: WAL & Cryptographic Invariants (`rules/wal_crypto.md`)

> **Normative Vorgabe nach docs/SPEC.md §8.6, §8.8, §10 und PLAN.md Phase 2.**

## Gilt für
- `crates/contextra-store`
- `crates/contextra-crypto`
- `crates/contextra-erasure`

## Pflichtregeln
1. **WAL-First Consistency (I-1, P24)**:
   - Jede persistente Zustandsänderung folgt der strikten Reihenfolge: WAL-Frame erzeugen → Gruppen-Commit → `sync_data` WAL → Memtable eintragen → Sichtbarkeit im Read-Pfad.
2. **Kryptografische WAL-Integrität (SPEC §8.8, §9.2)**:
   - Jeder WAL-Frame besitzt eine kryptografische HMAC-SHA256-Verkettung unter dem Schlüssel `K_wal` (abgeleitet aus `KeySet`).
   - CRC32 dient ausschließlich der schnellen Torn-Write-Erkennung am Dateiende. CRC-Mismatch am Dateiende führt zum kontrollierten Abschneiden; CRC-Mismatch mitten in der Datei ist ein fataler Integritätsfehler.
   - Historische Legacy-Keys ohne HMAC (`WalVersion::V1`) sind verboten und werden nicht als reguläre Writes akzeptiert.
3. **Schlüsselhierarchie & Zeroize (SPEC §10.1, §10.4)**:
   - Master-Key wird zwingend via Argon2id ($m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$) aus der Passphrase abgeleitet.
   - Teilschlüssel (`k_enc`, `k_idx`, `k_wal`, `k_journal`, `k_vault`, `k_unit`, `k_meta`, `k_hdr`, `k_sign_seed`) werden per HKDF abgeleitet.
   - Sämtliche Schlüsselstrukturen implementieren `Zeroizing` und löschen sensible Daten bei Scope-Verlassen aus dem RAM.
4. **Asymmetrisch signierter Löschbericht (D-13, SPEC §9.1)**:
   - Löschnachweise erfolgen über das Format `ctx-erasure-report/1`, signiert per Ed25519.
   - Der Bericht enthält keine Klartext-IDs, personenbezogene Daten oder Dokumentinhalte.

## Verweise
- `docs/SPEC.md` §8.6, §8.8, §10
- `PLAN.md` WP-2.3, WP-2.4, WP-2.5
