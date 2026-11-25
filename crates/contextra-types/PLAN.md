# Implementierungsplan: contextra-types

**Ebene:** L0 · **Ziel-Zustand:** gemeinsame Typen, Fehler, IDs, ohne Serialisierung im Standardpfad.
**Basis:** SPEC R14 §2.1, §2.5, §2.6, §2.7, §3, §14; R15 §2.3. Quellcode gemessen (Dateien, Greps), Module im Detail noch nicht gelesen.

## 1. Ist-Befund (gemessen)

- 25 Dateien, 8 103 Zeilen. Module: `error`, `error_dto`, `model_fingerprint`, `retrieval_strategy`, `schema`, `tenant_scope`, `tombstone`, `types`, `erasure`, `ids`, `refs`, `witness`.
- 79 Treffer auf verbotene Muster im Produktionscode (`bincode`, `blake3`, `serde`, `sha2`). SPEC erlaubt sie nur hinter Feature `wire-compat`.
- `Cargo.toml` führt `bincode`, `blake3`, `sha2`, `serde`, `serde_json` ohne Feature-Gate.
- `Epoch`, `Stamp`, `ClusterId`, `SpaceId`, `Redacted`, `Protection`, `DataClass`, `Durability`, `Receipt`, `ErasureReceipt` existieren, aber in unterschiedlichen Modulen (`types/domain/ids.rs`, `types/erasure.rs`, `ids`, `tombstone`).
- `TxId` ist in `types/domain/ids.rs:417` definiert. Dieselbe Definition existiert in `graph`, `vector`, `mvcc`, `checkpoint`.

## 2. Entfernen

| Ziel | Aktion | Begründung |
|---|---|---|
| `bincode`, `blake3`, `sha2`, `serde_json` in `[dependencies]` | nach `wire-compat` verschieben (`optional = true`) | SPEC §2.7, `types` nur hinter `wire-compat` |
| `TxId` (`types/domain/ids.rs:417`) | löschen | ersetzt durch `Epoch(u64)` (R14 §13.3) |
| Schema-/Retrieval-Typen, die nur Altpfade bedienen (`schema`, `retrieval_strategy`, `model_fingerprint`), nach Prüfung | löschen, wenn keine Referenz aus L1–L3 | Kernel kennt keine Schemas (NZ-3) |
| `error_dto` | nach `wire-compat` verschieben | DTO ist Serialisierung |

Vor jeder Löschung: `grep -rn '<Typ>' crates/` über alle Crates. Nur Typen ohne Verwender entfernen.

## 3. Ändern

- **`error.rs` (1162 Zeilen):** Fehlerenum auf die Klassen F-1…F-9 (§2.5) abbilden. Jede Variante bekommt `hint() -> &'static str`. Varianten aus der Liste F-7 vollständig: `Conflict`, `Restricted`, `UnknownCluster`, `BelowFloor`, `ReservedSpace`, `SnapshotTooOld`, `ErasePending`, `LedgerUnavailable`, `LedgerSameDevice`, `BackupBeyondHorizon`, `ProviderAssuranceInsufficient`, `IndexLag`, `IdemConflict`, `IdemCapacity`, `InvalidInput`.
- **Fehlertexte:** keine Schlüssel, Nutzerschlüssel, Inhalte, Queries oder Embeddings (§2.5). Ein Test prüft, dass `Display` und `Debug` für jede Variante keine Eingabewerte enthalten, die aus Konstruktorargumenten stammen.
- **`Redacted<T>`:** `Debug` und `Display` geben nur `<redacted>` aus. Gilt für `ClusterId`, Schlüsseltypen, Nutzdaten, Query-Texte (§2.6).
- **`Stamp`:** Obergrenze `2^56` als Konstruktor-Validierung; `Stamp::checked_next()` gibt `Result` zurück, kein `wrapping_add`.
- **`Epoch` und `Stamp`:** derselbe Typ (Glossar §0). Doppelte Definition entfernen, falls vorhanden.

## 4. Implementieren

1. **IDs (`ids`):** `ClusterId([u8; 16])` mit `from_wire`, `SpaceId(u64)` (nie wiederverwendet, Konstruktor nur aus Catalog-Reservierung), `BranchId(u32)`, `TombId`, `InstanceId([u8; 16])`, `Ticket`, `IdemToken`.
2. **Epoch/Stamp:** `Epoch(u64)` mit `Ord`, `checked_add`, `saturating_sub`; `Stamp` als `Epoch`-Alias-Typ im selben Modul, nur ein Typ.
3. **Enumerationen:** `Protection { Sealed, Plain }`, `DataClass { Personal, NonPersonal, … }` (vollständige Liste aus §3.8 übernehmen), `Durability { Durable, Relaxed }`, `EraseLevel { Logical, Cryptographic, Physical }`. Alle `#[non_exhaustive]` **nicht**, weil Z-15 ein geschlossenes Enum verlangt.
4. **`ErasureStatus`:** `Complete`, `CompleteWithResidual`, `Incomplete`, `Blocked`, `Failed` (§8.9). Ein Test prüft die Reihenfolge der Caps.
5. **Kanonische Bytes** für `ErasureReceipt`: Feld-Reihenfolge fest, tag- und längencodiert. Die Codierung liegt hier, nicht in `crypto`, weil `types` die Form besitzt.
6. **`wire-compat`:** serde/bincode-Impls hinter Feature. Ohne Feature kompiliert keine `serde`-Zeile.

## 5. Tests

- Property-Tests (`proptest`): `Stamp` Obergrenze, `ClusterId`-Roundtrip, `ErasureStatus`-Ordnung.
- Golden-Test für kanonische Receipt-Bytes (gegen `tools/reference/generator.py`, Anhang C).
- Leakage-Test: `format!("{:?}", cluster_id)` enthält keine Hexziffern der ID.
- Fuzz-Ziel: kanonischer Decoder von `ErasureReceipt` (Pflicht laut §13.1, ≥ 8 Ziele).

## 6. Abhängigkeiten nach Umbau

Erlaubt: `thiserror`, `bytes`, `zeroize`. Optional: `serde`, `serde_json`, `bincode`, `blake3`, `sha2` nur hinter `wire-compat`.

## 7. Gate und Reihenfolge

1. `Cargo.toml` bereinigen, `cargo check -p contextra-types --locked` grün (Teil von G-0a).
2. `TxId` entfernen; Fehlerliste aus F-7 vollständig machen.
3. Kanonische Codierung und Golden-Test (G-F-Vorbedingung).
4. G-0: Review durch zweiten Reviewer (SPEC §17.3).

**Risiko:** Viele Crates importieren `types`. Eine Umbenennung ohne Migration bricht alle Crates gleichzeitig. Mit `pub use`-Alias nur im Übergang, nicht dauerhaft.
