# Contextra – Architektur-Dokumentation

| Feld | Wert |
|---|---|
| Stand | 2026-11-01 |
| Normative Referenz | [`docs/SPEC.md`](SPEC.md) (Fassung 2.0.0 für Release 1.0.0) & [`PLAN.md`](../PLAN.md) |
| Ring-Definition | SPEC §5.1, §5.2, §5.3 |
| Sicherheits-Doktrin | Single-Node, Safe Rust (2 Unsafe-Inseln), Zero Panic, Fail-Closed, Pflichtverschlüsselung, absolute Mandantentrennung |

---

## 1. Überblick & Mission

Contextra ist ein lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust (Single-Node, Library-First) für Agenten- und RAG-Systeme:
- **Speichern**: Transaktionaler LSM-Tree-Speicher mit Write-Ahead-Log (WAL) als einziger Quelle der Wahrheit (I-1: WAL → Memtable → Sichtbarkeit).
- **Finden**: Hybrides Retrieval (HNSW-Vektorsuche + BM25-Volltextsuche + RRF-Fusion), strikt mandantengrenzenbeachtend.
- **Löschen (DSGVO Art. 17/18)**: Eigenständiger, entkoppelter und gemessener Löschprozess gemäß ADR-001 (physische Bereinigung Ebene A + fail-closed `ReadGate` + E1/E2/E3 Messungen + Ed25519-Löschbericht `ctx-erasure-report/1`).

---

## 2. Ringmodell & Crate-Topologie (SPEC §5.1, §5.2)

Das Gesamtsystem ist als strikter, azyklischer gerichteter Graph (**DAG**) in 5 Ringen organisiert:

```
Ring 4  contextra                               (Fassade, Composition Root)
Ring 3  contextra-erasure   contextra-engine    (Löschkern, Orchestrierung)
Ring 2  contextra-mvcc  contextra-store  contextra-text  contextra-vector  contextra-rank
Ring 1  contextra-sys   contextra-simd   contextra-durable-fs  contextra-crypto
Ring 0  contextra-types contextra-ports
dev     contextra-testkit  contextra-lab  ctx-verify  tests/arch
```

### Die 14 Produktionscrates

| Ring | Crate | Rolle | Unsafe |
|---|---|---|:---:|
| **Ring 0** | `contextra-types` | Reine, zustandslose Typen (IDs, Tags, Fehler, Schlüsselcodec, Löschtypen). Kein I/O, kein `async`. | Nein |
| **Ring 0** | `contextra-ports` | Abstrakte Schnittstellen-Traits (`StorageEngine`, `ErasureLayer`, `ReadGate`, `Vfs`, `Clock`, `Rng`, etc.). | Nein |
| **Ring 1** | `contextra-sys` | Plattform- und OS-Primitive (`fsync`, `flush_dir`, `rename_durable`, `mlock`). **Unsafe-Insel 1 von 2**. | **Ja** |
| **Ring 1** | `contextra-simd` | f32-Distanzkernel (`l2`, `dot`, `cosine`) mit Skalar-Referenz. **Unsafe-Insel 2 von 2**. | **Ja** |
| **Ring 1** | `contextra-durable-fs` | `RealVfs`-Implementierung, atomare Dateieretzung, best-effort Zeroize-Scrub. | Nein |
| **Ring 1** | `contextra-crypto` | AEAD (AES-256-GCM-SIV), Argon2id KDF, HKDF, WAL-HMAC, Hash-Ketten, Ed25519-Signatur. | Nein |
| **Ring 2** | `contextra-mvcc` | Monotone `Seq`, Snapshots, SSI-Validierung, Snapshot-Widerruf (`revoke_covering`). | Nein |
| **Ring 2** | `contextra-store` | Transaktionaler LSM-Tree (WAL, Memtable, Leveled SSTables, Purge-Rewrite & Swap). Blind für Dokumentinhalte. | Nein |
| **Ring 2** | `contextra-text` | BM25-Segmentindex (AEAD-verschlüsselte Segmente, Term-Tags, kein Klartext). | Nein |
| **Ring 2** | `contextra-vector` | HNSW-Vektorindex (Slot-Zeroing, verschlüsselte Snapshot-Chunks, Batch-Rebuild). | Nein |
| **Ring 2** | `contextra-rank` | RRF-Fusion (`rrf_fuse`) mit deterministischem Tie-Break nach `doc_tag`. | Nein |
| **Ring 3** | `contextra-erasure` | Entkoppelter Löschkern: Saga, Journal, ReadGate (`ErasedSet`), Tresor (`vault`), Messung, Bericht. | Nein |
| **Ring 3** | `contextra-engine` | Orchestrierung: Collections, Transaktionen, Schreib-/Lesepfade, Verdrahtung. | Nein |
| **Ring 4** | `contextra` | Öffentliche Fassade, `EngineBuilder`, Re-Exporte. | Nein |

---

## 3. Erlaubte Abhängigkeiten (Matrix SPEC §5.3)

| von \ nach | types | ports | sys | simd | dfs | crypto | mvcc | store | text | vector | rank | erasure |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ports | ✓ | | | | | | | | | | | |
| sys | ✓ | | | | | | | | | | | |
| simd | ✓ | | | | | | | | | | | |
| durable-fs | ✓ | ✓ | ✓ | | | | | | | | | |
| crypto | ✓ | ✓ | | | | | | | | | | |
| mvcc | ✓ | ✓ | | | | | | | | | | |
| store | ✓ | ✓ | ✓ | | | ✓ | ✓ | | | | | |
| text | ✓ | ✓ | | | | ✓ | | | | | | |
| vector | ✓ | ✓ | | ✓ | | ✓ | | | | | | |
| rank | ✓ | ✓ | | | | | | | | | | |
| erasure | ✓ | ✓ | | | | ✓ | | | | | | |
| engine | ✓ | ✓ | | | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

**Architektur-Regeln (AR-1 bis AR-8):**
1. Abhängigkeiten zeigen nur auf gleiche oder niedrigere Ringe; innerhalb Ring 2 gilt strikt obige Matrix.
2. `store` kennt weder `text`, `vector` noch `erasure`.
3. `erasure` kennt **keine** Ring-2-Crates, sondern kommuniziert ausschließlich über Ports (`ports`-Traits).
4. `store` nutzt `Vfs` ausschließlich als Port aus `contextra-ports`, niemals direkt aus `contextra-durable-fs`.
5. Keine Netzwerk-Crates im Produktionsbaum (Single-Node-Doktrin).
6. `unsafe` ist strikt auf `contextra-sys` und `contextra-simd` beschränkt. Alle anderen Crates erzwingen `#![forbid(unsafe_code)]`.

---

## 4. Löscharchitektur & DSGVO (ADR-001, SPEC §11, §12)

Das Löschen ist in Contextra ein eigenständiger, entkoppelter und maschinell gemessener Prozess:
- **Keine Ebene B in v1 (ADR-001, D-01):** Die kryptografische Schlüsselvernichtung (`DocDek`) entfällt in v1. Löschung ist die physische Bereinigung (Ebene A) kombiniert mit fail-closed `ReadGate`, Residuenmessung und signiertem Bericht.
- **Zustandsmaschine**:
  `Requested → Gated → Drained → Rewritten → Cleaned → Measured → Completed`
- **ReadGate (`ErasedSet`)**:
  Unmittelbar nach `Gated` blockiert das Gate jeden Lese- und Suchzugriff auf Zielschlüssel fail-closed. Aktive Snapshots, die Zielversionen sehen könnten, werden widerrufen (`SnapshotRevoked`).
- **Physisches Purging (Ebene A)**:
  - **Store**: `drain` versiegelt WAL; `rewrite_file` erzeugt neue SSTables ohne Zieldaten (Originalsequenzen bleiben erhalten); atomarer `swap` via Manifest; Block-Cache Eviction mit Zeroize.
  - **Vector**: `zero_slot` nullt Slot-Bytes und markiert Tombstone; Snapshot-Chunks werden neu geschrieben; Rebuild ab Schwelle.
  - **Text**: Textpuffer leeren; betroffene Segmente ohne Zieldokumente neu schreiben; Alt-Segmente löschen; Term-Statistik neu aggregieren.
- **Residuenmessung (E1, E2, E3)**:
  - **E1**: Byte-genauer Rohscan des Datenverzeichnisses nach Needles.
  - **E2**: Strukturierter Scan nach Dekodierung aller Container.
  - **E3**: Nachweis der Schlüssel- und Slot-Abwesenheit.
- **Löschbericht (`ctx-erasure-report/1`)**:
  Ed25519-signiert, maschinenlesbar, frei von Klartext-IDs oder Personendaten.

---

## 5. Laufzeitmodell & Datenflüsse (SPEC §5.4, §5.5)

- **Runtime:** Tokio multi-thread über den übergebenen `tokio::runtime::Handle` in `EngineBuilder`.
- **Cancel-Safety (I-14):** Mutierende Schreib- und Löschoperationen laufen in entkoppelten Tasks (`spawn`); der Aufrufer wartet auf ein `oneshot`-Signal.
- **Sperren:** Dokument-Lock über die gesamte Operation (verhindert Race-Conditions zwischen Insert, Delete und Erase). Exklusive Purge-`Barrier` zwischen Kompaktion und Purge.
- **Dateisystem-Layout (SPEC §5.6):**
  ```
  <data>/                       0700
    LOCK                        exklusiver Prozess-Lock
    HEADER                      Format-Version, KDF-Parameter, Salt[32], Store-ID
    wal/        *.wal           HMAC-Kette, verschlüsselt
    sst/        *.sst           SSTables (Block-AEAD), Level-Struktur
    MANIFEST/   *.log           Manifest-Log (Datei-Mengen, Bereiche, Seq)
    text/       *.seg           Textsegmente (AEAD-verschlüsselt)
    vector/     *.snap          Vektorsnapshots (AEAD-verschlüsselt)
    vault/      vault.dat       Tresor (Slots)
    erasure/    journal.log     Löschjournal (Hash-Kette, verschlüsselt)
                erased.set      aktive Gates (Bereiche, Commitments)
                ledger.log      abgeschlossene Löschungen (Restore-Guard)
                report/         signierte Berichte
    keys/       0700            Schlüssel-Seeds (0600)
    tmp/                        Waisen-Bereinigung beim Start
  ```
