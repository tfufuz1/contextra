# ADR-102: Mandantenisolation via `TenantScopedStorage` Wrapper

* **Status:** Proposed
* **Datum:** 2026-09-28
* **Anforderung / Referenz:** Mandantenfähigkeit als Produktversprechen (Option A, AGENTS.md Abschnitt 7, INV-TENANT-2)

## Kontext & Problemstellung
`TenantKeyCodec` in `contextra-store` bietet Präfixcodierung und Dekodierung für Mandanten-Schlüssel (`t:{tenant_id}:...`). Bisher hatte dieser Codec jedoch außerhalb seiner eigenen Datei keinen Aufrufer, da in der Engine alle Mandanten denselben unstrukturierten Schlüsselraum nutzten.

Gemäß den Governance-Regeln in `AGENTS.md` (Abschnitt 7) ist Mandantenisolation (`TenantId`) absolut, um DSGVO-Löschgarantien und KV-Cache-Sicherheit zu gewährleisten. Gemäß Option A wird die Mandantenfähigkeit im Produktversprechen durch einen transparenten Wrapper auf Speicherebene umgesetzt.

## Entscheidung

1. **`TenantScopedStorage<S: StorageEngine>` Wrapper in `contextra-store` (`src/tenant_codec.rs`):**
   - Ein Wrapper-Typ `TenantScopedStorage<S>`, der eine innere `S: StorageEngine` kapselt und an eine unveränderliche `TenantId` gebunden ist.
   - Jede schlüsselbasierte Lese-, Schreib-, Lösch- und Scan-Operation transformiert logische Schlüssel automatisch über das Mandantenpräfix (`t:{tenant_id}:`).
   - Scan-Ergebnisse werden gefiltert, sodass nur Keys des eigenen Mandanten zurückgegeben werden. Das Präfix wird bei der Rückgabe transparent wieder entfernt.
   - Der inneren StorageEngine wird kein direkter Zugriff oder Getter gewährt (`no raw inner storage accessor`), und die `TenantId` ist nach der Konstruktion unveränderlich (`immutable`).
   - Transaktions- und Lifecycle-Methoden (`commit`, `rollback`, `flush`, `stats`, `last_seq_no`, `last_tx_id`, `pin_checkpoint`, `unpin_checkpoint`) delegieren direkt an den inneren Storage.

2. **Umgang mit Bestandsdaten ohne Präfix (Legacy Un-prefixed Data):**
   - Bestandsdaten ohne `t:`-Präfix gehören semantisch dem Default-Mandanten (`TenantId::SYSTEM` / `TenantId(1)`).
   - Eine spätere Aufgabe wird beim Starten der Engine ein Manifest-Migrations-Marker (`legacy_tenant_migrated: true`) setzen und Alt-Keys in das Mandanten-Präfix-Format migrieren. Dieser Marker wird in diesem Schritt vorbereitend beschrieben, aber noch nicht im Manifest angelegt.

3. **Verhältnis zu Verschlüsselung und Löschbeweis (Logical vs. Cryptographic Isolation):**
   - Ohne mandantenspezifisches Schlüsselmaterial (per-tenant encryption keys / salts) bietet `TenantScopedStorage` eine **reine logische Isolation** (Namespace-Prefix-Isolation).
   - Es verhindert Cross-Tenant-Datenaustausch auf API- und Speicherzugriffsebene, stellt jedoch ohne separate Kryptoschlüssel pro Mandant keine kryptografische Mandantentrennung dar. Cryptographic Shredding / Deletion Proofs erfordern weiterhin das gezielte Löschen oder Verwerfen des mandantenspezifischen Key-Materials.

## Konsequenzen & Sicherheitsgarantien
- **Garantierte Isolation (INV-TENANT-2):** Kein Mandant kann Schlüssel anderer Mandanten lesen, überschreiben oder löschen.
- **Transparenz:** Höhere Schichten (Engine, KV-Cache) können `TenantScopedStorage` als gewöhnliche `StorageEngine` nutzen, ohne manuell Präfixe verwalten zu müssen.
- **Zero-Performance-Cost for Base Engine:** Die grundlegende LSM-Engine bleibt mandantenagnostisch und performant.
