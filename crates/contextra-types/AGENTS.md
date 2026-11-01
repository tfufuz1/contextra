# AGENTS.md — `contextra-types`

> **Normative Vorgabe gemäß [docs/SPEC.md §8.1](file:///home/freddy/Projekte/contextra/docs/SPEC.md) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md)**

## 1. Rolle & Zweck
`contextra-types` definiert die reinen, zustandslosen, I/O- und runtime-unabhängigen Domänentypen, Identifikatoren, Schlüssel-Buffer, Fehler und Löschtypen für das Gesamtsystem Contextra.

## 2. Ring, Abhängigkeiten & Verbote
- **Ring:** Ring 0 (Fundament)
- **Erlaubte Abhängigkeiten:** Nur externe Utility-Crates (`serde`, `thiserror`, `zeroize`, `bytes`).
- **Verbotene Abhängigkeiten:** Keine Abhängigkeit zu anderen Contextra-Crates!
- **Verbotene Muster:**
  - **Strikt kein `unsafe`** (`#![forbid(unsafe_code)]`).
  - **Kein `async`** (T-ARCH-1).
  - **Kein I/O** (kein Dateisystem, kein Netzwerk, keine Sockets).
  - **Keine Zeit- oder Zufallsabfragen** (`Instant::now`, `SystemTime::now`, `thread_rng`).
  - **Keine Threads oder Tasks**.

## 3. Modul-Inventar (Soll-Zustand laut SPEC §8.1)

| Modul | Inhalt & Verantwortung |
|---|---|
| `ids` | `TenantId`, `CollectionName`, `DocKey`, `SubjectId` (validierte Newtypes mit Längengrenzen); `TenantTag`, `CollTag`, `ClusterTag`, `DocTag`, `TermTag` (je `[u8; 16]`, `Copy`, `Ord`, kein `Display` mit Inhalt); `Seq(NonZeroU64)`, `TxId(u64)`, `ErasureId([u8; 16])`, `SegmentId(u64)`, `Slot(u32)`. |
| `key` | `Region` (`Data=0x01`, `Loc=0x02`, `Meta=0x03`), `Family` (Anhang C), `KeyBuf` (Byte-Layout SPEC §9.1), `KeyRange { start, end_exclusive }`, `Matcher { ranges: Vec<KeyRange>, exact: BTreeSet<KeyBuf> }`. |
| `erasure` | `EraseTarget`, `ErasureState`, `LayerId`, `LayerStatus`, `ErasureClass` (G/K/P-Verband), `ResidualRisk`, `ErasureReport`-Struktur, `RetentionAttestation`, `RetentionCode`, `RefusalReason`, `NeedleSet`, `LayerScan`, `Evidence`. |
| `error` | `ContextraError` und typisierte Sub-Fehlerbäume gemäß SPEC §7.4. |
| `secret` | `SecretBytes` (`Zeroizing<Vec<u8>>`, kein `Clone`, kein `Debug`-Inhalt, kein `Serialize`), `SecretKey32`. |
| `time` | `Timestamp(i64)` (Mikrosekunden seit Unix-Epoche UTC), `Duration`-Wrapper für Konfiguration. |
| `limits` | Normative Ressourcengrenzen und Konstanten aus SPEC §14.6. |

## 4. Kritische Invarianten
1. **Tag-Diskretion:** Alle Tag-Typen (`*Tag`) sind unveränderlich und implementieren **kein** `Display`, sondern ausschließlich `Debug` mit gekürzter Hex-Darstellung der ersten zwei Bytes (z. B. `DocTag(a1b2..)`), um Datenlecks in Logs auszuschließen.
2. **Sentinel-Freiheit (`Seq`):** `Seq` besitzt keinen Sentinel wie `u64::MAX`. Ein abwesender Snapshot wird typisiert als `Option<Seq>` modelliert (Behebung B-11 / D-C02).
3. **Kanonische Klassennamen:** `ErasureClass::display()` liefert strikt die normativen Texte aus SPEC §12.5.
4. **Matcher-Normalisierung:** `Matcher`-Instanzen sind stets normalisiert (sortierte, disjunkte Bereiche).

## 5. Crate-Arbeitspakete zum SOLL-Zustand (PLAN.md)

- **[ ] WP-0.5: ADR-001 Typ-Anpassungen**
  - Entfernen aller Referenzen auf Ebene B (`DocDek`-Revocation-Typen).
- **[ ] WP-1.2: Workspace-Bereinigung & Rückbau**
  - Entfernen veralteter Typen: `DocId(u64|u128)`, `witness.rs`, `AutoExtractionMode` sowie aller Graph-, Checkpoint- und Lizenz-Typen.
- **[ ] WP-2.2: Sequenzsemantik festschreiben**
  - Umstellung auf `Seq(NonZeroU64)`, Entfernen jeglicher Tombstone-Bits im `Seq`-Typ.
- **[ ] WP-3.1: Löschtypen definieren (`types/src/erasure`)**
  - Implementierung von `EraseTarget`, `ErasureState`, `LayerId`, `LayerStatus`, `ErasureClass`, `ResidualRisk`, `ErasureReport`, `RetentionAttestation`, `RefusalReason`, `NeedleSet`, `LayerScan`, `Evidence`.
- **[ ] WP-3.2: Schlüsselcodec & Newtypes**
  - Implementierung von `DocKey`, `SubjectId`, `KeyBuf`, `KeyRange`, `Matcher` nach SPEC §9.1.

## 6. Verifikation & Tests
- **T-ARCH-1:** Kein `async`, kein I/O, keine Zeitabfragen in `contextra-types`.
- **Property-Tests:** `KeyBuf`-Roundtrip für alle zulässigen Schlüsseltypen und Längen; `Matcher`-Mengenalgebra (disjunkt, sortiert, Überlappungsprüfung); `Seq`-Konvertierungen.
