# RA-1 Re-Audit

- **Datum:** 2026-10-05
- **Commit-SHA:** `e31ed5c4cbb91598c0cdb8d87e4c70159cbd25cd`

---

## Einzelprüfungen (Checkliste)

### 1. `trait KvLifecycleHooks` Deklaration und Implementierer
- **Frage:** Deklariert `trait KvLifecycleHooks` in `crates/contextra-ports/src/kv.rs` die Methoden `on_rollback`, `remove_doc_segments` und `purge_tenant`? Welche Typen implementieren das Trait?
- **Antwort:** BESTÄTIGT
- **Belege:** `crates/contextra-ports/src/kv.rs:161-170`, `crates/contextra-kvcache/src/store.rs:759`, `crates/contextra-engine/src/collection/mod.rs:269`
- **Anmerkung:** `KvLifecycleHooks` deklariert `on_rollback` (Z. 163), `remove_doc_segments` (Z. 166) und `purge_tenant` (Z. 169). Die Implementierer im Workspace sind `contextra_kvcache::TenantIsolatedKvStore` (`crates/contextra-kvcache/src/store.rs:759`) und `CryptoKvStoreAdapter` (`crates/contextra-engine/src/collection/mod.rs:269`). In Testcode existiert zudem `MockKvLifecycleHooks` (`crates/contextra-engine/tests/kv_lifecycle_hooks_rollback.rs:22`).

### 2. Exakter Typ von `Collection::kv_store`
- **Frage:** Was ist der exakte Typ von `Collection::kv_store` in `crates/contextra-engine/src/collection/mod.rs`? Ist er `Option<Arc<dyn KvLifecycleHooks>>`?
- **Antwort:** NICHT_BESTÄTIGT
- **Belege:** `crates/contextra-engine/src/collection/mod.rs:302`, `crates/contextra-engine/src/collection/mod.rs:304`
- **Anmerkung:** Der exakte Typ von `Collection::kv_store` ist `Option<Arc<contextra_crypto::TenantIsolatedKvStore>>` (gated unter `#[cfg(feature = "encryption-at-rest")]`, Z. 302). Das Trait-Objekt `Option<Arc<dyn KvLifecycleHooks>>` ist stattdessen im benachbarten Feld `kv_hooks` abgelegt, welches den Typ `parking_lot::RwLock<Option<Arc<dyn KvLifecycleHooks>>>` hat (Z. 304).

### 3. Existenz von `KvCacheRuntime` / `kv_cache_runtime` / `KvRuntime`
- **Frage:** Existiert irgendwo ein Typ namens `KvCacheRuntime` (oder ein Äquivalent)?
- **Antwort:** NICHT_BESTÄTIGT
- **Belege:** Workspace-weiter `grep -rniE "KvCacheRuntime|kv_cache_runtime|KvRuntime" .` (0 Treffer)
- **Anmerkung:** Ein Typ oder Modul mit dem Namen `KvCacheRuntime`, `kv_cache_runtime` oder `KvRuntime` existiert im gesamten Repository nicht.

### 4. Produktionsaufrufe von `on_rollback` und Erreichbarkeit
- **Frage:** Existiert ein Produktionsaufruf (Nicht-Test) von `on_rollback`? Prüfe `crates/contextra-engine/src/transaction/db_transaction_rollback.rs` und `crates/contextra-engine/src/transaction/compensating_actions.rs`. Ist dieser Code von einem öffentlichen Einstiegspunkt aus erreichbar?
- **Antwort:** BESTÄTIGT
- **Belege:** `crates/contextra-engine/src/transaction/db_transaction_rollback.rs:24`, `crates/contextra-engine/src/transaction/compensating_actions.rs:274`
- **Anmerkung:** Im Produktionsquellcode rufen `DbTransaction::trigger_kv_store_rollback` (`db_transaction_rollback.rs:24`) und `CompensateHnswAction::execute` (`compensating_actions.rs:274`) `kv_hooks.on_rollback(...)` auf. Der Code ist strukturell über die öffentliche Methode `DbTransaction::rollback` (`db_transaction_rollback.rs:218`) sowie über die 2PC-Kompensationslogik in `DbTransaction::commit` (`db_transaction_commit.rs:316`) erreichbar. Allerdings ist im Produktionsbetrieb zur Laufzeit `collection.kv_hooks` stets `None`, da keine Produktions-Initialisierung `set_kv_hooks` oder `set_kv_store` aufruft.

### 5. `KeyRegistry::revoke_group` und Purge-Hooks
- **Frage:** Ruft `KeyRegistry::revoke_group` in `crates/contextra-crypto/src/kv_shredding.rs` irgendeinen Purge-Hook (`purge_tenant`, `remove_doc_segments`) auf?
- **Antwort:** BESTÄTIGT
- **Belege:** `crates/contextra-crypto/src/kv_shredding.rs:241-270`
- **Anmerkung:** Wie erwartet ruft `KeyRegistry::revoke_group` keine Purge-Hooks auf. Es beschränkt sich auf das Anhängen an das `revocation_log` (sofern konfiguriert), das Einfügen der `group_id` in `revoked_groups` und das Zeroizen/Entfernen der KEK-Einträge in `groups`.

### 6. Konstruktion von `KvCipher` mit `with_revocation_log` im Produktionscode
- **Frage:** Konstruiert Produktionscode ein `KvCipher` mit `with_revocation_log`?
- **Antwort:** BESTÄTIGT
- **Belege:** `crates/contextra-crypto/src/kv_cipher.rs:116`, `crates/contextra-crypto/src/kv_shredding.rs:132`
- **Anmerkung:** Es gibt KEINE Produktionsaufrufstelle im gesamten Repository, die `KvCipher` oder `KvSegmentCipher` mit `with_revocation_log` instanziiert. Die Methode ist nur in Unittests/Integrationstests (`crates/contextra-crypto/tests/kv_revocation_wiring.rs:32`) angebunden.

### 7. Aufruf von `was_group_ever_registered` in `generate_deletion_proof` und Testabdeckung der `Err`-Path
- **Frage:** Wird `was_group_ever_registered` in `generate_deletion_proof` in `crates/contextra-store/src/kv/segment.rs` aufgerufen? Gibt es einen Test, der den `Err`-Pfad für eine nie registrierte Gruppen-ID abdeckt?
- **Antwort:** BESTÄTIGT
- **Belege:** `crates/contextra-store/src/kv/segment.rs:163`, `crates/contextra-store/tests/kv_deletion_proof_registry.rs:14`
- **Anmerkung:** `was_group_ever_registered` wird in `generate_deletion_proof` auf Zeile 163 aufgerufen. Der `Err`-Pfad für nie registrierte Gruppen-IDs wird explizit im Test `test_generate_deletion_proof_behavior` in `crates/contextra-store/tests/kv_deletion_proof_registry.rs:14-38` geprüft.

### 8. Symbolzählung Cluster WP-A mit Produktionsaufrufer
- **Frage:** Wie viele Symbole des Workspaces gehören zum Cluster WP-A und besitzen einen Produktionsaufrufer?
- **Antwort:** 11 von 34 Symbolen (32,4 % verdrahtet, 67,6 % unverdrahtet).
- **Belege:** Audit-Synthese `docs/analysis/kampagne/M00.md:52` (Abschnitt 2.3)
- **Anmerkung:** Die Workspace-weite Verdrahtungsmatrix kategorisiert 34 Symbole in Cluster WP-A (KV-Cache). Davon weisen 11 Symbole direkte Produktionsaufrufer auf (z. B. `KvLayout`, `PrefixKey`, `KvBlock`, `KvPrefixHit`, `KvPrefixStore` in `contextra-ports`, `TenantIsolatedKvStore` in `contextra-kvcache`), während 23 Symbole (u. a. `remove_doc_segments`, `purge_tenant`, `KvSegmentManager`, `KvBridgeAdapter` Injektionen) ohne Produktionsaufrufer verbleiben.

---

## Korrekturen zum Plan

1. **Behaupteter Typ von `kv_store` vs. Realität:**
   - *Annahme im Plan:* `Collection::kv_store` hat den Typ `Option<Arc<dyn KvLifecycleHooks>>`.
   - *Code-Realität:* `Collection::kv_store` hat den Typ `Option<Arc<contextra_crypto::TenantIsolatedKvStore>>` (gated unter `#[cfg(feature = "encryption-at-rest")]`). Das Trait-Objekt `Option<Arc<dyn KvLifecycleHooks>>` ist stattdessen im Feld `kv_hooks` in einem `parking_lot::RwLock` verankert (`crates/contextra-engine/src/collection/mod.rs:302,304`).

2. **Existenz von `KvCacheRuntime`:**
   - *Annahme im Plan:* Ein Runtime-Typ `KvCacheRuntime` orchestriert den KV-Cache.
   - *Code-Realität:* Es existiert weder ein Typ `KvCacheRuntime` noch `kv_cache_runtime` oder `KvRuntime` im gesamten Workspace.

3. **Laufzeit-Wirksamkeit von `on_rollback`:**
   - *Annahme im Plan:* Transaktions-Rollbacks lösen im Produktionsbetrieb KV-Cache-Rollbacks aus.
   - *Code-Realität:* Der Aufruf `kv_hooks.on_rollback(...)` steht zwar im Quellcode von `db_transaction_rollback.rs` und `compensating_actions.rs`, wird zur Laufzeit in Produktion jedoch nie ausgeführt, da `kv_hooks` bei der Instanziierung von `Collection` immer `None` bleibt und kein Produktionscode `set_kv_hooks` aufruft.

4. **RevocationLog-Anbindung bei `KvCipher`:**
   - *Annahme im Plan:* `KvCipher` wird in Produktions-Pfaden mit `with_revocation_log` gekoppelt.
   - *Code-Realität:* `KvSegmentCipher::with_revocation_log` wird in keinem Produktionspfad aufgerufen, sondern lediglich in Tests.

---

## Offene Lücken

- `Collection::kv_hooks` wird im Produktions-Bootstrapping (`Contextra::open_authorized` / `Collection::new`) nie initialisiert und bleibt permanent `None`.
- `KvLifecycleHooks::remove_doc_segments` wird aus keinem CRUD-Löschpfad (`Collection::delete`, `delete_by_id`) aufgerufen.
- `KvLifecycleHooks::purge_tenant` wird aus keinem Purge-Pfad (`Contextra::purge_tenant`) aufgerufen.
- `contextra-store::kv::segment::KvSegmentManager` besitzt außerhalb von Unittests keinen einzigen Produktionsaufrufer in den Crates.
- `KvSegmentCipher::with_revocation_log` ist im Produktionscode unverdrahtet.
- `setup_kv_bridge` in `contextra-mcp` gibt stets `None` zurück und bindet den `KvBridgeAdapter` nicht an den Inferenz-Pfad an.
- `EvictionWorker` in `contextra-kvcache` wird in `lifecycle.rs:116` gestartet, arbeitet aber auf einer verworfenen lokalen `kv_store`-Instanz.
