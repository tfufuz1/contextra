# Contextra – Architektur-Dokumentation

| Feld | Wert |
|---|---|
| Stand | 2026-11-02 |
| Normative Referenz | [`docs/SPEC.md`](SPEC.md) (Fassung 4.0.0 für Release 1.0.0) |
| Ring-Definition | SPEC §5.1, §5.2, §5.3 |
| Sicherheits-Doktrin | Single-Node, Safe Rust (2 Unsafe-Inseln), Zero Panic, Fail-Closed, Pflichtverschlüsselung, absolute Mandantentrennung, 3 Säulen der Löschung |

---

## 1. Überblick & Mission

Contextra ist ein lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust (Single-Node, Library-First) für Agenten- und RAG-Systeme:
- **Speichern**: Transaktionaler LSM-Tree-Speicher mit Write-Ahead-Log (WAL) als einziger Quelle der Wahrheit (I-1: WAL → Memtable → Sichtbarkeit; WAL wird nie umgeschrieben, D-73).
- **Finden**: Hybrides Retrieval (HNSW-Vektorsuche + segmentierter BM25-Volltextsuche + RRF-Fusion), strikt mandantengrenzenbeachtend.
- **Löschen (Art. 17/18 DSGVO)**: Entkoppelter, gemessener Löschprozess auf drei Säulen (D-01, ADR-002):
  1. *Gate und physischer Purge:* Sofortige Lesesperre (`ReadGate`) und physisches Umschreiben betroffener Dateien (LSM-SSTable, HNSW-Batch-Kantenreparatur mit Surrogaten und deterministischem Rebuild D-77, Textsegment-Rewrite D-71).
  2. *Kryptografische Schlüsselvernichtung:* Hierarchischer Tresor (`FileVault`, D-79) vernichtet Mandanten- (`T`), Collection- (`C`), Betroffenen- (`S`) oder Dokument-Slots (`D`), wodurch untergeordnete DEKs sofort unableitbar werden; Dateischlüssel-Vernichtung (`F`-Slots, D-85) entwertet freigegebene Blöcke.
  3. *Unabhängige Messung & Nachweis:* Typisierter Zeuge `LayerWitness` (Messklassen MK1, MK2, MK3-Feld), Ed25519-signierter Bericht und unabhängige Feldprüfung via `ctx-verify scan/report` und Erasure Lab.

---

## 2. Ringmodell & Crate-Topologie (SPEC §5.1, §5.2)

Das Gesamtsystem ist als strikter, azyklischer gerichteter Graph (**DAG**) in 5 Ringen organisiert:

```
Ring 4  contextra                         (Fassade)
Ring 3  contextra-erasure   contextra-engine
Ring 2  contextra-mvcc  contextra-store  contextra-text  contextra-vector  contextra-rank
Ring 1  contextra-sys  contextra-simd  contextra-durable-fs  contextra-crypto
Ring 0  contextra-types  contextra-ports
dev     testkit  lab  ctx-verify  tests/arch
```

### Die 14 Produktions-Crates (SPEC §5.1)

| Crate | Ring | Rolle |
|---|---|---|
| `contextra-types` | 0 | Reine Typen (IDs, blinde Tags, Schlüssel, Fehler, Löschtypen, typisierter Zeuge). Kein I/O, kein `async`. |
| `contextra-ports` | 0 | Traits: `StorageEngine`, `TextIndex`, `VectorIndex` (statisch dispatcht), `Vfs` (einziges VFS-Trait), `Clock`, `Rng`, `IdGen`, `ErasureLayer`, `ReadGate`, `RecordSealer`, `FileKeyring`, `LedgerSink`, `Metrics`. |
| `contextra-sys` | 1 | Unsafe-Insel für OS-Primitive: `fsync`, `flush_dir`, `rename_durable`, `mlock`, POSIX-Dateirechte. Windows-Code ist entfernt. |
| `contextra-simd` | 1 | Unsafe-Insel: f32-Distanzkerne (Dot, L2, Cosine) mit Skalar-Referenz und automatischem Laufzeit-Dispatch. |
| `contextra-durable-fs` | 1 | `Vfs`-Implementierung (`RealVfs`), atomarer Austausch, optionales Scrubbing (D-83), Startprobe (D-56). |
| `contextra-crypto` | 1 | AES-256-GCM-SIV, Argon2id mit strengen Grenzen (D-06), HKDF-SHA256, keyed BLAKE3, WAL-HMAC-Kette, Ed25519. |
| `contextra-mvcc` | 2 | Sequenzen, Snapshots, SSI-Validierung, Snapshot-Registry nach `tenant_tag`. Kein `u64::MAX`-Sentinel. |
| `contextra-store` | 2 | LSM-Tree: WAL (HMAC-Kette, nie umgeschrieben), Memtable, SSTable mit Block-AEAD, kooperative Kompaktion (D-78), Purge-Primitive. |
| `contextra-text` | 2 | Segmentierter BM25-Index (D-20, D-71, D-81): Textpuffer, Segmente, Merge, Rewrite je Zielsegment. Implementiert `TextIndex` und `ErasureLayer`. |
| `contextra-vector` | 2 | HNSW mit Arena (D-86), Surrogat-Zeroing, Batch-Kantenreparatur, deterministischer Rebuild (D-77), verschlüsselter Snapshot nur über `Vfs`. Implementiert `VectorIndex` und `ErasureLayer`. |
| `contextra-rank` | 2 | RRF-Fusion (`k_rrf = 60`), deterministische Tie-Breaks nach `doc_tag`. |
| `contextra-erasure` | 3 | Löschkern: Saga-Zustandsmaschine, `ErasedSet`/Gate-Log, Journal, Tresor (`FileVault`, `RecordSealer`, `FileKeyring`), Messung (`Scanner<Armed>`), Berichtsbau, Ledger, Restore-Guard. Hängt nur an `types`, `ports`, `crypto`. |
| `contextra-engine` | 3 | Orchestrierung: `TenantScopedStorage`, Collections, Transaktionen, Suche, Hintergrundtasks, Startsequenz. Enthält keine Löschlogik. |
| `contextra` | 4 | Fassade, Einstiegspunkt, `system_defaults()`, Re-Exports. |

---

## 3. Architekturregeln (AR-1 bis AR-9, SPEC §5.2)

1. **AR-1 (DAG-Fluss):** Abhängigkeiten zeigen nur auf den gleichen oder einen niedrigeren Ring; innerhalb Ring 2 nur laut Matrix SPEC §5.3.
2. **AR-2 (Store-Isolation):** `store` kennt weder `text`, `vector` noch `erasure`.
3. **AR-3 (Erasure-Entkopplung):** `erasure` kennt **keine** Ring-2-Crates, nur `ports`-Traits, `types` und `crypto`.
4. **AR-4 (Unabhängige Prüfer):** `lab` und `ctx-verify` hängen weder an `store`, `engine`, `erasure` noch an Produktions-Decodern von `crypto`. Sie besitzen unabhängige Format-Parser nach SPEC Anhang F.
5. **AR-5 (Types-Reinheit):** `contextra-types` ist frei von `async` und I/O.
6. **AR-6 (Mandantenschranke):** Nur `TenantScopedStorage` darf den rohen Speicher ansprechen.
7. **AR-7 (Single-Node-Garantie):** Keine Netzwerk-Crates im Produktionsbaum.
8. **AR-8 (Unsafe-Inseln):** `unsafe` ausschließlich in `sys` und `simd`; alle anderen 12 Produktivcrates deklarieren `#![forbid(unsafe_code)]`.
9. **AR-9 (Feature-Hygiene):** Jedes `cfg(feature)` ist in `Cargo.toml` deklariert; kein `allow(unexpected_cfgs)`. Keine Features schwächen Sicherheitsfunktionen (D-84).

---

## 4. Löschcluster-Schlüsselraum (SPEC §9.1, D-31, D-70)

Schlüssel werden zur Unterstützung von O(1)-Pruning und gezieltem Umschreiben nach Löschclustern partitioniert:
- **Dokument-Identität:** `doc_tag = tag("ctx/doc/v2", tenant_tag ‖ coll_tag ‖ subject_tag_oder_0¹⁶ ‖ doc_key)`. Der Betroffene ist Teil der Identität (`DocRef`, D-70).
- **Cluster-Präfix:** Schlüssel gruppieren Datensätze desselben Betroffenen (`cluster_tag = subject_tag`), sodass ein Subject-Erase nur zusammenhängende SSTables betrifft und der Schreibaufwand proportional zum Ziel bleibt (P11, BM-ERASE-WA).
