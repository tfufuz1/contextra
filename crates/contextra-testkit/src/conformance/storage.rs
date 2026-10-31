//! # Konformitätsprüfsuite für den `StorageEngine`-Port (`conformance/storage`)
//!
//! Dieses Modul testet die vertraglichen Zusicherungen (Contract Invariants)
//! für Speicher-Engines des [`contextra_ports::StorageEngine`]-Ports.
//!
//! ## Getestete Verträge
//! 1. **Put-Commit-Get-Delete:** Gespeicherte Werte sind nach `commit` abrufbar und nach `delete` + `commit` getilgt.
//! 2. **Überschreiben (Overwrite):** Neuer `put` auf denselben Schlüssel überschreibt den Altwert.
//! 3. **Lexikografische Scan-Ordnung:** `scan_prefix` und `scan` liefern Einträge strikt lexikografisch sortiert zurück.
//! 4. **Mandanten-Präfix-Isolation (Regel 7):** Scans nach `t1:` isolieren vollständig gegen `t2:`.
//! 5. **Große Werte (Large Values):** 1 MB große Werte werden fehlerfrei persistiert und gelesen.
//! 6. **Grenzfälle (Edge Cases):** Leere Schlüssel (`b""`) und leere Werte (`b""`) werden korrekt unterstützt.
//! 7. **Atomare Batches:** `put_batch`, `delete_many`, `delete_prefix` verhalten sich atomar.
//! 8. **Transaktions-Isolation & Rollback:** Nicht bestätigte Mutationen sind unsichtbar; nach `rollback` verworfen.

use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::StorageEngine;
use contextra_types::TxId;
use std::future::Future;

// ANCHOR[contextra-testkit:CONF-STORAGE-SUITE-001]
/// Führt die vollständige Konformitätsprüfung für eine [`StorageEngine`]-Implementierung durch.
///
/// **Parameter:**
/// - `factory`: Eine Closure, die für jeden isolierten Testfall eine frische Storage-Instanz asynchron erzeugt.
///
/// **Ablauf:**
/// 1. Test 1: Basis-Lebenszyklus (`put` -> `commit` -> `get` -> `delete` -> `commit` -> `get`).
/// 2. Test 2: Überschreiben bestehender Schlüssel.
/// 3. Test 3: Lexikografische Sortierordnung bei Präfix-Scans.
/// 4. Test 4: Isolation von Mandanten-Präfixen (`t1:` vs `t2:`).
/// 5. Test 5: Speichern und Lesen von 1-MB-Großwerten.
/// 6. Test 6: Grenzfälle leerer Schlüssel und leerer Nutzdaten.
/// 7. Test 7: Batch-Operationen (`put_batch`, `delete_many`, `delete_prefix`).
/// 8. Test 8: Uncommitted Isolation und Transaktions-Rollback.
///
/// **Rückgabe:** `Ok(())` bei Vertragskonformität, andernfalls `Err(ConformanceFailure)`.
pub async fn run_storage_engine_suite<F, Fut, S>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = S>,
    S: StorageEngine,
{
    let suite = "StorageEngine";

    // Test 1: Put, Get, Delete
    {
        let engine = factory().await;
        let tx1 = TxId::new(1);
        if let Err(e) = engine.put(tx1, b"k1", b"v1").await {
            return Err(ConformanceFailure::new(
                suite,
                "put_get_delete",
                format!("put failed: {e:?}"),
            ));
        }
        if let Err(e) = engine.commit(tx1).await {
            return Err(ConformanceFailure::new(
                suite,
                "put_get_delete",
                format!("commit failed: {e:?}"),
            ));
        }
        match engine.get(b"k1").await {
            Ok(Some(val)) if val.as_ref() == b"v1" => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("expected Some(v1), got {other:?}"),
                ))
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("get failed: {e:?}"),
                ))
            }
        }

        let tx2 = TxId::new(2);
        if let Err(e) = engine.delete(tx2, b"k1").await {
            return Err(ConformanceFailure::new(
                suite,
                "put_get_delete",
                format!("delete failed: {e:?}"),
            ));
        }
        if let Err(e) = engine.commit(tx2).await {
            return Err(ConformanceFailure::new(
                suite,
                "put_get_delete",
                format!("commit delete failed: {e:?}"),
            ));
        }
        match engine.get(b"k1").await {
            Ok(None) => {}
            Ok(Some(val)) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("expected None after delete, got Some({val:?})"),
                ))
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "put_get_delete",
                    format!("get after delete failed: {e:?}"),
                ))
            }
        }
    }

    // Test 2: Overwrite
    {
        let engine = factory().await;
        let tx1 = TxId::new(10);
        let _ = engine.put(tx1, b"k", b"v1").await;
        let _ = engine.commit(tx1).await;

        let tx2 = TxId::new(11);
        let _ = engine.put(tx2, b"k", b"v2").await;
        let _ = engine.commit(tx2).await;

        match engine.get(b"k").await {
            Ok(Some(val)) if val.as_ref() == b"v2" => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "overwrite",
                    format!("expected v2, got {other:?}"),
                ))
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "overwrite",
                    format!("get failed: {e:?}"),
                ))
            }
        }
    }

    // Test 3: Scan ordering
    {
        let engine = factory().await;
        let tx = TxId::new(20);
        let _ = engine.put(tx, b"prefix:c", b"val_c").await;
        let _ = engine.put(tx, b"prefix:a", b"val_a").await;
        let _ = engine.put(tx, b"prefix:b", b"val_b").await;
        let _ = engine.commit(tx).await;

        let res = match engine.scan_prefix(b"prefix:").await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "scan_ordering",
                    format!("scan_prefix failed: {e:?}"),
                ))
            }
        };
        if res.len() != 3 {
            return Err(ConformanceFailure::new(
                suite,
                "scan_ordering",
                format!("expected 3 items, got {}", res.len()),
            ));
        }
        let k0 = res.first().map(|(k, _)| k.as_slice());
        let k1 = res.get(1).map(|(k, _)| k.as_slice());
        let k2 = res.get(2).map(|(k, _)| k.as_slice());
        if k0 != Some(b"prefix:a".as_slice())
            || k1 != Some(b"prefix:b".as_slice())
            || k2 != Some(b"prefix:c".as_slice())
        {
            return Err(ConformanceFailure::new(
                suite,
                "scan_ordering",
                format!("items not in lexicographical order: {res:?}"),
            ));
        }
    }

    // Test 4: Tenant prefix separation
    {
        let engine = factory().await;
        let tx = TxId::new(30);
        let _ = engine.put(tx, b"t1:k1", b"v1").await;
        let _ = engine.put(tx, b"t2:k2", b"v2").await;
        let _ = engine.commit(tx).await;

        let res_t1 = match engine.scan_prefix(b"t1:").await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "tenant_prefix_separation",
                    format!("scan_prefix t1 failed: {e:?}"),
                ))
            }
        };
        let first_k = res_t1.first().map(|(k, _)| k.as_slice());
        if res_t1.len() != 1 || first_k != Some(b"t1:k1".as_slice()) {
            return Err(ConformanceFailure::new(
                suite,
                "tenant_prefix_separation",
                format!("expected [t1:k1], got {res_t1:?}"),
            ));
        }
    }

    // Test 5: Large values (1MB)
    {
        let engine = factory().await;
        let tx = TxId::new(40);
        let large_val = vec![0x42u8; 1_000_000];
        let _ = engine.put(tx, b"large_key", &large_val).await;
        let _ = engine.commit(tx).await;

        match engine.get(b"large_key").await {
            Ok(Some(val)) if val.as_ref() == large_val.as_slice() => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "large_values",
                    format!("large value mismatch: expected 1MB, got {other:?}"),
                ))
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "large_values",
                    format!("get large value failed: {e:?}"),
                ))
            }
        }
    }

    // Test 6: Empty keys and values
    {
        let engine = factory().await;
        let tx = TxId::new(50);
        let _ = engine.put(tx, b"", b"empty_key_val").await;
        let _ = engine.put(tx, b"key_empty_val", b"").await;
        let _ = engine.commit(tx).await;

        match engine.get(b"").await {
            Ok(Some(val)) if val.as_ref() == b"empty_key_val" => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "empty_keys_values",
                    format!("expected empty_key_val, got {other:?}"),
                ))
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "empty_keys_values",
                    format!("get empty key failed: {e:?}"),
                ))
            }
        }
        match engine.get(b"key_empty_val").await {
            Ok(Some(val)) if val.as_ref() == b"" => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "empty_keys_values",
                    format!("expected empty val, got {other:?}"),
                ))
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "empty_keys_values",
                    format!("get empty val failed: {e:?}"),
                ))
            }
        }
    }

    // Test 7: Atomic Batches
    {
        let engine = factory().await;
        let tx1 = TxId::new(60);
        let batch = vec![
            (b"b:1".to_vec(), b"v1".to_vec()),
            (b"b:2".to_vec(), b"v2".to_vec()),
        ];
        if let Err(e) = engine.put_batch(tx1, &batch).await {
            return Err(ConformanceFailure::new(
                suite,
                "atomic_batches",
                format!("put_batch failed: {e:?}"),
            ));
        }
        let _ = engine.commit(tx1).await;

        let res = match engine.scan_prefix(b"b:").await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "atomic_batches",
                    format!("scan_prefix failed: {e:?}"),
                ))
            }
        };
        if res.len() != 2 {
            return Err(ConformanceFailure::new(
                suite,
                "atomic_batches",
                format!("expected 2 items from batch, got {}", res.len()),
            ));
        }

        let tx2 = TxId::new(61);
        let _ = engine.delete_many(tx2, vec![b"b:1".to_vec()]).await;
        let _ = engine.commit(tx2).await;
        match engine.get(b"b:1").await {
            Ok(None) => {}
            Ok(other) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "atomic_batches",
                    format!("expected None after delete_many, got {other:?}"),
                ))
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "atomic_batches",
                    format!("get failed: {e:?}"),
                ))
            }
        }

        let tx3 = TxId::new(62);
        let _ = engine.delete_prefix(tx3, b"b:").await;
        let _ = engine.commit(tx3).await;
        let res_after = match engine.scan_prefix(b"b:").await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "atomic_batches",
                    format!("scan_prefix failed: {e:?}"),
                ))
            }
        };
        if !res_after.is_empty() {
            return Err(ConformanceFailure::new(
                suite,
                "atomic_batches",
                format!("expected empty after delete_prefix, got {res_after:?}"),
            ));
        }
    }

    // Test 8: Uncommitted isolation and rollback
    {
        let engine = factory().await;
        let tx1 = TxId::new(70);
        let _ = engine.put(tx1, b"uncommitted_key", b"val").await;
        match engine.get(b"uncommitted_key").await {
            Ok(None) => {}
            Ok(Some(val)) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "uncommitted_isolation",
                    format!("uncommitted key visible before commit: {val:?}"),
                ))
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "uncommitted_isolation",
                    format!("get failed: {e:?}"),
                ))
            }
        }

        let _ = engine.rollback(tx1).await;
        let _ = engine.commit(tx1).await;
        match engine.get(b"uncommitted_key").await {
            Ok(None) => {}
            Ok(Some(val)) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "uncommitted_isolation",
                    format!("rolled back key visible after commit: {val:?}"),
                ))
            }
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "uncommitted_isolation",
                    format!("get failed: {e:?}"),
                ))
            }
        }
    }

    Ok(())
}
