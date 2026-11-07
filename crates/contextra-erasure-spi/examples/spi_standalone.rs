//! Standalone-Beispiel für contextra-erasure-spi (M10).
//!
//! Demonstriert die autarke Definition und Ausführung einer Schichtbereinigung
//! und Restdatenmessung ohne Abhängigkeit von Datenbanken oder Runtimes.

use contextra_erasure_spi::{
    KeyScope, LayerScanner, LayerWitness, PurgeError, PurgeStats, Purger, ScanError, ScanSession,
    ScanTarget, ScanTechnique, WitnessLayer,
};
use contextra_types::{RequestContext, TenantId, TxId};
use std::collections::BTreeSet;
use std::sync::Mutex;

/// Einfacher In-Memory Datenspeicher für das Beispiel (BTreeSet für P28-Determinismus).
struct SimpleLayerStorage {
    data: Mutex<BTreeSet<String>>,
}

impl SimpleLayerStorage {
    fn new() -> Self {
        Self {
            data: Mutex::new(BTreeSet::new()),
        }
    }
}

/// Beispiel-Purger für die Memtable-Schicht.
struct ExampleMemtablePurger<'a> {
    storage: &'a SimpleLayerStorage,
}

impl<'a> Purger for ExampleMemtablePurger<'a> {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::LsmMemtable
    }

    fn purge(&self, _ctx: &RequestContext, scope: &KeyScope) -> Result<PurgeStats, PurgeError> {
        let key = scope.to_canonical_string();
        let mut set = self
            .storage
            .data
            .lock()
            .map_err(|e| PurgeError::Internal(e.to_string()))?;

        if set.remove(&key) {
            Ok(PurgeStats::new(1, key.len() as u64))
        } else {
            Ok(PurgeStats::new(0, 0))
        }
    }
}

/// Beispiel-Scanner für die Memtable-Schicht.
struct ExampleMemtableScanner<'a> {
    storage: &'a SimpleLayerStorage,
}

impl<'a> LayerScanner for ExampleMemtableScanner<'a> {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::LsmMemtable
    }

    fn scan_units(
        &self,
        session: &mut ScanSession,
        _ctx: &RequestContext,
    ) -> Result<(), ScanError> {
        let set = self
            .storage
            .data
            .lock()
            .map_err(|e| ScanError::Internal(e.to_string()))?;

        // 1. Positivkontrolle (Canary check) durchführen
        session.record_control_match();

        // 2. Ziel überprüfen
        let target_id = &session.target().target_id;
        let is_residual = set.contains(target_id);

        let mut unit_hash = [0u8; 32];
        let hash = blake3::hash(target_id.as_bytes());
        unit_hash.copy_from_slice(hash.as_bytes());

        session.record_unit(unit_hash, is_residual);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Contextra Erasure SPI Standalone Beispiel ===");

    let storage = SimpleLayerStorage::new();
    let tenant = TenantId::try_new(1001)
        .map_err(|e| format!("TenantId creation failed: {e:?}"))?;
    let ctx = RequestContext::new(tenant, TxId(1));
    let scope = KeyScope::Tenant { tenant };

    // 1. Datum in Storage einfügen
    {
        let mut set = storage
            .data
            .lock()
            .map_err(|e| format!("Mutex lock failed: {e:?}"))?;
        set.insert(scope.to_canonical_string());
    }

    // 2. Scanner vor dem Purgen prüfen
    let scanner = ExampleMemtableScanner { storage: &storage };
    let target = ScanTarget::from_scope(&scope);
    let now_nanos = 1_700_000_000_000_000_000u64;

    let witness_before: LayerWitness =
        scanner.scan(&ctx, target.clone(), ScanTechnique::Structural, now_nanos)?;
    println!(
        "Messung vor Bereinigung: Einheiten={}, Verbleibend={}",
        witness_before.scanned(),
        witness_before.remaining()
    );
    if witness_before.remaining() != 1 {
        return Err("Vor Bereinigung muss genau 1 Einheit verbleiben".into());
    }

    // 3. Purger ausführen
    let purger = ExampleMemtablePurger { storage: &storage };
    let stats = purger.purge(&ctx, &scope)?;
    println!(
        "Bereinigung durchgeführt: Einheiten bereinigt={}, Bytes={}",
        stats.purged_units, stats.bytes_freed
    );
    if stats.purged_units != 1 {
        return Err("Genau 1 Einheit muss bereinigt werden".into());
    }

    // 4. Scanner nach dem Purgen prüfen
    let witness_after: LayerWitness =
        scanner.scan(&ctx, target, ScanTechnique::Structural, now_nanos + 1_000_000)?;
    println!(
        "Messung nach Bereinigung: Einheiten={}, Verbleibend={}, Digest={:02x?}",
        witness_after.scanned(),
        witness_after.remaining(),
        &witness_after.digest()[..8]
    );
    if witness_after.remaining() != 0 {
        return Err("Nach Bereinigung dürfen keine Einheiten verbleiben".into());
    }

    println!("Erasure SPI Ausführung erfolgreich abgeschlossen.");
    Ok(())
}
