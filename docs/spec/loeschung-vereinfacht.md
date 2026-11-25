# Spezifikation: Vereinfachte synchrone Löschung

Status: Verbindlich für die Umsetzung. Grundlage: ADR-115 (`docs/decisions/ADR-115-vereinfachte-loeschung-ohne-loeschnachweis.md`).
Ersetzt: die Spezifikation „Entfernung der Löschkette“ (v1). Diese ist überholt.

---

## 1. Vertrag der Löschung

### 1.1 Operation

```
delete_document(id: DocId) -> Result<DeleteStatus, DeleteError>
```

- Atomar: Entweder sind alle abhängigen Strukturen entfernt oder keine.
- Synchron: Der Aufrufer erhält erst nach Abschluss die Antwort.
- Idempotent: Ein erneuter Aufruf für eine bereits gelöschte ID liefert `DeleteStatus::AlreadyDeleted`, keinen Fehler.

### 1.2 Entfernte Strukturen (in einer Transaktion)

| Struktur | Crate | Hinweis |
|---|---|---|
| Dokument und Text-Index-Einträge | `contextra-text`, `contextra-engine` | |
| Vektoreinträge (HNSW) | `contextra-vector` | Tombstone intern, Rebuild bei Schwelle |
| Graph-Kanten und Hyperedges mit Bezug zum Dokument | `contextra-graph` | synchrone Kaskade |
| KV-Einträge und Segmente | `contextra-kvcache`, `contextra-store` | `purge` nur für betroffene Einträge |
| Provenienz-Einträge | `contextra-graph` | |

### 1.3 Rückgabe

```
DeleteStatus::Deleted { removed: RemovalCounts }
DeleteStatus::AlreadyDeleted
DeleteError::{ NotFound, Busy, Storage(...) }
```

`RemovalCounts` enthält Anzahlen je Struktur. Diese dienen der Fehlersuche, nicht als Nachweis.

Keine Signatur, kein Beweisobjekt, kein Beweis-Schlüssel.

---

## 2. Entfallende Komponenten (Umsetzung Phase 3)

- `contextra-crypto`: `deletion_proof.rs`, `deletion_proof_typestate.rs`, `ed25519_proof.rs`, `kv_shredding.rs`, Funktionen `derive_deletion_proof_key`, `create_deletion_proof`, `compute_wal_delete_receipt`, `verify_wal_delete_receipt`.
- `contextra-engine`: `generate_kv_deletion_proof`, `DeletionProof`-Re-Exports in `lib.rs`, `LayerCleanupProof`, `DeletionLayer`, `DeletionScope`, `purge_tenant_segments` (durch Standard-Löschpfad ersetzt).
- `contextra-mcp`: `proof_key.rs`, `proof_key_env.rs`, Proof-Ausgabe im Handler.
- `contextra-store`: `kv/delete_mode.rs` (`KvDeleteMode`), `generate_deletion_proof` in `kv/segment.rs`.
- `contextra-kvcache`: `ShreddableSegmentKey`, `shred`, `is_shredded`.
- `contextra-db`: `PurgeReceipt` und `purge` in `volatile_vault.rs` (sofern nur für Löschung genutzt).
- `contextra-privacy`: `pii_vault_forces_crypto_shred`, `resolve_effective_kv_delete_mode`.
- `contextra-audit-export`: `DeletionProofSummary` und zugehörige Art.-30-Felder.
- `contextra-avv-generator`: bleibt bei der Löschfrist (`with_deletion_sla_days`), da die Frist vertraglich relevant ist.
- Entfällt: asynchroner Worker `start_hyperedge_cascade_deferred_worker` (wenn synchrone Kaskade läuft).

Vor jeder Entfernung: Prüfen, ob die Komponente außerhalb der Löschung genutzt wird (`grep` über alle Crates). Nur reine Löschkomponenten entfernen.

Bleibt unverändert: `INV-VAULT-1/2/3`, `INV-WAL-LEGACY-KEY-1`, `INV-WAL-TRUNCATION-1`, Verschlüsselung at rest.

---

## 3. Ablauf je Phase

Jede Phase endet mit grünen Gates und einem eigenen Commit.

| Phase | Inhalt | Akzeptanz |
|---|---|---|
| 0 | ADR-115 (erledigt), Spec v2 (dieses Dokument), `AGENTS.md`-Markierung, `CLAUDE.md`-Anpassung | Dokumentationsänderung, keine Code-Änderung |
| 1 | Einheitliche Operation `delete_document` mit Vertrag aus Abschnitt 1, parallel zum Alt-Pfad | `cargo test -p contextra-engine --locked` |
| 2 | Synchrone Kaskade in Graph und Vektor, Hyperedge-Worker als Ausnahme oder entfernt | `cargo test -p contextra-graph --locked`, `cargo test -p contextra-vector --locked` |
| 3 | Entfernung der Komponenten aus Abschnitt 2, Invarianten aus `AGENTS.md` entfernen | `cargo test -p contextra-crypto --locked`, `cargo xtask doctrine-scan`, `cargo xtask dependency-graph-audit` |
| 4 | WAL-Löschverhalten (siehe Abschnitt 4), Snapshot-Kürzung | `cargo test -p contextra-store --locked` |
| 5 | Dokumentation: Doku der Crates (`API.md`, `PLAN.md`, `SPEC.md`), `contextra_delete` in Schnittstellenspezifikation | `cargo xtask check-doc-references`, `cargo xtask check-manifest-completeness` |

Build-Hinweis: Die VM hat 1 vCPU und 3,9 GB RAM. Builds nur pro Crate. `--locked` immer. `Cargo.lock` darf sich nicht ändern.

Keine Gate-, Baseline- oder Schwellenabschwächung. Wenn ein Gate durch die Entfernung rot wird, ist die Ursache zu beheben oder die Entscheidung im ADR zu aktualisieren.

---

## 4. Offene technische Frage: WAL

Löschte Daten bleiben im WAL, bis dieses nach einem Snapshot gekürzt wird. Beim Neustart würden sie wieder erscheinen, wenn das WAL ungefiltert repliziert wird.

Empfehlung (zu bestätigen in Phase 4):

- Eine Löschung schreibt einen WAL-Eintrag `Delete(DocId, TxId)`.
- Beim Replay wird jeder `Delete` angewendet. Ein Dokument mit späterem `Delete` wird nicht wiederhergestellt.
- Die WAL-Kürzung erfolgt nach dem nächsten Snapshot. Vorher bleiben Daten physisch im WAL.

Diese Empfehlung löst die Wiederherstellung beim Neustart. Sie löst nicht die physische Entfernung aus dem WAL-Dateiinhalt. Das ist ausdrücklich kein Versprechen. Die Dokumentation an Kunden muss das so benennen.

Sollte eine physische Entfernung aus dem WAL nötig sein, ist eine erzwungene Kürzung (Snapshot plus Truncation) als Operation `compact_now()` anzubieten. Das ist Teil von Phase 4 und wird dort entschieden.

---

## 5. Kundendokumentation

Contextra liefert:

- Einen Rückgabestatus pro Löschaufruf.
- Ein Protokoll der Löschaufrufe (ohne Inhalt, mit Zeitstempel und ID), das der Kunde für seine Dokumentation nutzen kann.

Contextra liefert nicht:

- Kryptografische Löschbeweise.
- Garantien über Backups, Kopien außerhalb von Contextra oder Modelle, die Daten zur Laufzeit gesehen haben.

Die Kundendokumentation muss diese Grenzen ausdrücklich nennen. Die Rechtsbewertung liegt beim Kunden.

---

## 6. Offene Punkte

1. Soll `compact_now()` in Phase 4 Standard sein oder nur auf Anforderung?
2. Wie lange wird das Protokoll der Löschaufrufe aufbewahrt, und wer kann es exportieren?
3. Ist `contextra-erasure` in `CLAUDE.md` ein geplantes Crate oder veraltet? Empfehlung: entfernen, da es im Repository nicht existiert.
