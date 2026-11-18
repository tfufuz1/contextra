# contextra-ids — Crate-Spezifikation

**Ebene:** E0 (Kern-Closure leer)  
**Bindung:** `docs/SPEC.md` §2, §3, §4

---

## 1 Zweck

Identitätstypen für das gesamte System: `DocId`, `TenantId`, `CollectionId`, `SubjectRef`, `PersonalRef`, `ErasureId`, `LayerId`, `FileId`, `GateId`, `Seq`, `Timestamp`.

## 2 Verantwortlichkeit

- **Besitzer von `SubjectRef`:** 128-Bit-Zufalls-ID für Betroffene (SPEC §3.2). MUSS über `OsRng::try_fill_bytes` erzeugt werden (fehlbar).
- **Besitzer von `PersonalRef`:** 128-Bit-Zufalls-ID als Verweis auf P-Daten in K-Dokumenten.
- **Besitzer von `DocId`:** Dokumentidentifikation. BLAKE3-Hash-Präfix. Bei Kollision (64-Bit-Präfix): keine stille Schlüsselüberschreibung — Kollision MUSS erkannt und mit Fehler abgewiesen werden.
- **Besitzer von `ErasureId`:** 128-Bit-Zufalls-ID für Lösch-Sagas.

## 3 Abhängigkeiten

Keine internen Abhängigkeiten. Nur `blake3`, `rand`, `serde`.

## 4 Regeln

- Kein `Clone` für `SubjectRef` und `PersonalRef` (kein versehentliches Kopieren von Identitäten).
- `Eq`, `Hash`, `Ord` für alle ID-Typen über constant-time-Vergleich nicht nötig (IDs sind nicht geheim), aber `SubjectRef` MUSS `ZeroizeOnDrop` implementieren, da es als Schlüsselableitung dient.
- Keine `Display`-Implementierung, die den vollen Wert zeigt, für `SubjectRef`.
