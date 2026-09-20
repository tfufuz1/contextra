# memfuse-types

`memfuse-types` definiert die zentralen Domänentypen und Fehler-Enums für MemFuse (Ring 0).

## Zweck

Kannolischer Typen- und Fehlerkatalog des Gesamtsystems: Identifikatoren (`DocId`, `DocIdx`, `TxId`, `TenantId`), `ModelFingerprint`, `MemFuseError`, DTOs und Tombstone-Semantik. Besitzt keine Asynchronität und kein I/O.

## Ring-Zugehörigkeit & Status

- **Ring:** Ring 0 (Typen-Kern, kein `tokio`)
- **Status:** 🟢 Fertig
- **Sicherheits-Invariante:** `#![forbid(unsafe_code)]`

## Öffentliche API-Übersicht

- **Fehler & DTOs:** `MemFuseError`, `Result`, `MemFuseErrorDto`
- **Fingerprints & Schema:** `ModelFingerprint`, `DocIdWidth`, `ManifestSchemaVersion`
- **Tombstones & Types:** `SeqBitTombstone`, `TombstoneSemanticsCheck`, Identifikatoren

## Architektur & Verweise

Details zur Typenhierarchie finden sich in [`ARCHITECTURE.md`](../../ARCHITECTURE.md) (folgt in Kürze) und `README.md` §4.2.
