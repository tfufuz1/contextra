# API-Spezifikation: contextra-store

## Übersicht
`contextra-store` exportiert das Speichertriebwerk (`LsmStorage`), das Write-Ahead-Log (`Wal`), SSTables (`SstableReader`, `SstableBuilder`), das Manifest (`Manifest`) sowie Mandantentrennung (`TenantKeyCodec`, `TenantScopedStorage`).

## Öffentliche Hauptkomponenten

### 1. LSM Storage Engine
- `LsmStorage::new(config: LsmConfig) -> Result<Self>`: Öffnet oder stellt eine LSM-Instanz am konfigurierten Pfad wieder her.
- `LsmStorage::open(config: LsmConfig) -> Result<Self>`: Alias für `new`.
- `LsmStorage::close(&self) -> Result<()>`: Fährt Hintergrundaufgaben herunter und flusht verbleibende Daten sauber auf Platte.
- `LsmStorage::wait_shutdown(&self)`: Wartet auf die Beendigung aller Hintergrundaufgaben.

### 2. Trait-Implementierung `StorageEngine`
`LsmStorage` und `TenantScopedStorage` implementieren den Port `contextra_ports::StorageEngine`:
- `put(&self, tx: TxId, key: &[u8], value: &[u8]) -> BoxFuture<Result<()>>`: Pufferung eines Schlüssel-Wert-Paares in der aktiven Transaktion.
- `delete(&self, tx: TxId, key: &[u8]) -> BoxFuture<Result<()>>`: Pufferung eines Tombstones in der aktiven Transaktion.
- `commit(&self, tx: TxId) -> BoxFuture<Result<()>>`: Dauerhaftes Schreiben und Bestätigen der Transaktionsänderungen im WAL.
- `rollback(&self, tx: TxId) -> BoxFuture<Result<()>>`: Verwerfen der gepufferten Änderungen.
- `get(&self, key: &[u8]) -> BoxFuture<Result<Option<Bytes>>>`: Lesen des neuesten bestätigten Werts.
- `scan(&self, start: Bound<&[u8]>, end: Bound<&[u8]>, limit: Option<usize>) -> BoxFuture<Result<Vec<(Vec<u8>, Vec<u8>)>>>`: Bereichsabfrage.

### 3. Konfiguration: `LsmConfig` & `DurabilityMode`
- `DurabilityMode::Full`: Synchrones WAL-Append mit `fsync` / `fdatasync`. Dauerhaft vor Crash.
- `DurabilityMode::WalNoHmac`: Asynchroner WAL-Flush ohne HMAC-Kettenvalidierung.
- `DurabilityMode::MemoryOnly`: Kein WAL, reine In-Memory Speicherung (flüchtig bei Crash).
