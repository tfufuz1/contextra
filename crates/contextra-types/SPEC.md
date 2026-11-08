# SPEC: contextra-types (Klasse B)

Normatives Baustein-Spezifikationsdokument gemäß SPEC v4.0 §8, B3–B12.

## 1. Rolle & Geltungsbereich
`contextra-types` ist das fundamentale Typfundament für das gesamte Storage-Bundle (Klasse B: `types`, `ports`, `mvcc`, `store`, `checkpoint`). Es stellt kanonische IDs, domänenspezifische Basisstrukturen, Abfrage-, Budget-, Filter- und Fehlertypen bereit.

- **Klasse:** B (Storage-Bundle)
- **Ring:** 0 (Rechenkern, autonom und I/O-frei)
- **Sicherheitsniveau:** `#![forbid(unsafe_code)]`
- **Zero-Panic:** Keine unhandled Panics, unwrap() oder unchecked Slices im Produktionscode (B9).
- **Determinismus:** Keine Zeiterfassung (`SystemTime::now`), kein Thread-Zufall, keine System-Entropie (B11).

## 2. Invarianten (INV-*)
1. **INV-TENANT-1 (Mandantenisolation):** `TenantId(0)` ist als `TenantId::SYSTEM` reserviert. `TenantId::try_new(0)` schlägt immer mit `ContextraError::InvalidInput` fehl. Kein Typ darf einen Default-Mandanten implizieren.
2. **INV-TXID-1 (Transaktionsbereiche):** Sequentielle Mutations-IDs bewegen sich im Bereich `0..=MAX_COLLECTION_SEQUENCE`. Interne Systemoperationen reservieren `INTERNAL_BASE` (`1 << 62`). Überläufe führen zu `ContextraError::InvalidSequenceNumber`.
3. **INV-DOCID-1 (Dokumentidentifikation):** `DocId` wird entweder als 64-Bit (`u64`) oder per Feature `docid-128` als 128-Bit (`u128`) breiter Typ instanziiert. Schlüssel hashing via BLAKE3 ist deterministisch und panikfrei (`DocId::from_key`).
4. **INV-TOMBSTONE-1 (Tombstone-Disziplin):** Bit 63 einer Sequenznummer ist `TOMBSTONE_BIT` (`1 << 63`). Vergleiche maskieren Bit 63 (`seq & !TOMBSTONE_BIT`).
5. **INV-FORMAT-1 (Formatkompatibilität):** Typen, die via `bincode` oder `serde` serialisiert werden, behalten ihre Feldreihenfolge strikt bei.
6. **INV-A6 (Einheitlicher Fehlerbaum):** `ContextraError` bündelt alle Kernfehler; neue Varianten werden ausschließlich am Ende angehängt.

## 3. Fehlermodell
Alle falliblen Operationen geben `crate::error::Result<T>` zurück, mit `ContextraError` als zentralem Fehlertyp.
- `ContextraError::InvalidInput`
- `ContextraError::NotFound`
- `ContextraError::Conflict`
- `ContextraError::TransactionTimeout`
- `ContextraError::TenantScopeViolation`
- `ContextraErrorDto` stellt eine serialisierbare, sprachunabhängige Repräsentation bereit.

## 4. Löschdomäne & Welle-3 Übergangsstatus
- Typen in `erasure.rs` und `witness.rs` (`EraseTarget`, `ScanMethod`, `SagaRecord`, `ErasureStatus`, etc.) sind als `#[deprecated(note = "moved to contextra-erasure")]` gekennzeichnet.
- Re-Exporte in `lib.rs` verbleiben übergangsweise bis Welle 5/6 für Abwärtskompatibilität downstream.
- Keine direkten Abhängigkeiten zu `contextra-erasure` im Manifest; die Crate bleibt autark (0 interne Abhängigkeiten).
