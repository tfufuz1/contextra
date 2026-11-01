# Contextra – Agenten-Betriebsanleitung (AGENTS.md)

> **Normative Richtlinie für Antigravity CLI, IDE, Desktop App, Claude & Jules.**
> Diese Datei ist die zentrale Quelle der Arbeitsregeln im Workspace `/home/freddy/Projekte/contextra`.
> Sie bindet alle Agenten und Entwickler ausnahmslos an die Vorgaben aus [docs/SPEC.md](file:///home/freddy/Projekte/contextra/docs/SPEC.md) (Soll-Zustand v2.0.0 für Release 1.0.0) und [PLAN.md](file:///home/freddy/Projekte/contextra/PLAN.md).
> Jede Abweichung des Quellcodes von `docs/SPEC.md` ist ein Defekt des Quellcodes.

---

## 1. Produkt & Mission

**Contextra ist ein lokaler, eingebetteter Wissens- und Gedächtnisspeicher in Rust (Single-Node, Library-First).**
- **Speichern**: Transaktionaler LSM-Tree-Speicher mit Write-Ahead-Log (WAL) als einziger Quelle der Wahrheit (I-1: WAL → Memtable → Sichtbarkeit).
- **Finden**: Hybrides Retrieval (HNSW-Vektorsuche + BM25-Volltextsuche + Reciprocal Rank Fusion / RRF), strikt mandantengrenzenbeachtend.
- **Löschen (DSGVO Art. 17/18)**: Physische und gemessene Bereinigung über spezialisierte Schichten (Ebene A: physisches Purging in Memtable, SSTable-Rewrite/Swap, HNSW-Slot-Zeroing & Batch-Rebuild, BM25-Segment-Rewrite).
  > [!IMPORTANT]
  > Gemäß **ADR-001 (SPEC §3.3, D-01)** entfällt Ebene B (kryptografische DocDek-Schlüsselvernichtung) in v1. Löschung in v1 ist die physische Bereinigung (Ebene A) kombiniert mit fail-closed `ReadGate` (`ErasedSet`), automatischer Residuen-Messung (E1 Rohscan, E2 Decoder-Scan, E3 Schlüsselabwesenheit) und kryptografisch signiertem Ed25519-Löschbericht (`ctx-erasure-report/1`, D-13).
- **Single-Node-Doktrin**: Kein Server-Socket, kein Sharding, kein P2P, keine Netzwerk-Crates im Produktionsbaum, keine Telemetrie nach außen. Öffentlicher Zugang ausschließlich über die Fassaden-Crate `contextra`.

---

## 2. Die 12 unverhandelbaren Kernregeln (SPEC §0.2, §2)

1. **Ring- & Schichten-Disziplin (P1, P30, SPEC §5.2)**:
   - Jede Abhängigkeit zeigt ausschließlich auf den gleichen oder einen niedrigeren Ring (Ring 4 → Ring 3 → Ring 2 → Ring 1 → Ring 0).
   - Innerhalb Ring 2 gilt strikt die Abhängigkeitsmatrix aus SPEC §5.3.
   - Der Kern darf niemals von Rand-, Experimentell- oder entfernten Crates (wie `graph`, `ingest`, `privacy`, `checkpoint`, `license`, `wire`) abhängen.
2. **Zero-Panic in Produktion (P7, SPEC §11.1)**:
   - Strengstens verboten in Produktivpfaden: `unwrap()`, `expect()`, `panic!()`, `todo!()`, `unimplemented!()`, `unreachable!()`, `assert!()`, `std::process::exit` und ungeprüftes Index-Slicing (`slice[i]`).
   - Fehler werden ausnahmslos typisiert über `ContextraError` oder domänenspezifische Fehlerbäume propagiert.
   - Erzwungene Deny-Lints: `clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::indexing_slicing`.
3. **Determinismus & Port-Purity (P28, SPEC §8.2)**:
   - Zeit, Zufall und IDs dürfen ausschließlich über die Ports `Clock`, `Rng` und `IdGen` aus `contextra-ports` bezogen werden.
   - Verboten in Kernlogik: `Instant::now()`, `SystemTime::now()`, `rand::thread_rng()`, `std::env::var()` sowie `HashMap`/`HashSet` mit Standard-RandomState (stattdessen `BTreeMap`/`BTreeSet`).
   - Einzige Ausnahme: Kryptografisches Schlüssel- und Salt-Material aus dem OS-CSPRNG (`OsRng`).
4. **WAL-First Consistency (P24, I-1, SPEC §5.5)**:
   - Schreibreihenfolge ist ausnahmslos: Frame(s) → Gruppen-Commit → `sync_data` WAL → Memtable → Sichtbarkeit.
   - Follower-Commits sind cancel-sicher (I-14, D-C01): Ausführung in eigener Task, Aufrufer wartet auf `oneshot`.
5. **I/O ausschließlich über `Vfs` (P24, D-04, SPEC §8.2, §8.5)**:
   - Kein direkter Aufruf von `std::fs` oder `tokio::fs` in Produktivcrates.
   - Alle Dateioperationen laufen über den `Vfs`-Port (`RealVfs` in `contextra-durable-fs`, `FaultVfs`/`MemVfs` in Tests).
   - `fsync`/`fdatasync`/`sync_data`/`sync_all`-Fehler müssen immer propagiert werden; niemals mit `let _ = ...` verwerfen!
6. **Unsafe ausschließlich in zwei Inseln (P2, D-29, SPEC §5.2)**:
   - `unsafe` Rust-Code ist ausschließlich in `contextra-simd` (Distanzkernel) und `contextra-sys` (Plattformprimitive) erlaubt.
   - Jeder einzelne `unsafe`-Block erfordert zwingend eine präzise `// SAFETY:`-Dokumentation (Alignment, Bounds, Gültigkeit).
   - Alle anderen 12 Produktivcrates erzwingen strikt `#![forbid(unsafe_code)]`.
7. **Mandantentrennung verpflichtend (D-07, SPEC §7.2, §8.8)**:
   - Mandanten-IDs (`TenantId`) stammen aus dem Kontext und sind Pflicht (`TenantPolicy` ohne Opt-out).
   - Speicherzugriff erfolgt ausschließlich über `TenantScopedStorage` (T-ARCH-2).
   - Caches, HNSW-Graphkanten, BM25-Statistiken und SSTables dürfen Mandantengrenzen niemals überschreiten.
8. **Sicherheit ist kein Feature-Schalter (A1, D-05, SPEC §10)**:
   - Verschlüsselung at rest (AES-256-GCM-SIV mit per HKDF abgeleiteten Schlüsseln) ist Pflicht ohne Opt-Out.
   - KDF: Argon2id mit $m \ge 64\,\text{MiB}, t \ge 3, p \ge 1$ (`derive_master`). Kein HKDF für Passphrasen!
   - Kein `cfg(not(feature))` darf Sicherheits- oder Krypto-Typen durch Stubs ersetzen.
9. **Messen vor Behaupten (INV-CLAIMS-1, D-35, SPEC §12)**:
   - Restbestand- oder Lösch-Zähler stammen aus echten Messungen (`scan_residue` über `ErasureLayer`).
   - Keine Behauptung ohne maschinellen Zeugen (`LayerScan`, E1/E2/E3 Klassifikation).
   - Externe Positivkontrolle (Canary) beweist Detektionsfähigkeit vor jeder Messung.
10. **Fail-Closed (INV-FAILCLOSED-1)**:
    - Unbekannter Zustand, korrupte Daten, unerwartete Eingaben oder ablaufende Fristen führen sofort zur geordneten Verweigerung, niemals zu stillschweigendem Weiterlaufen.
11. **Eine Typfamilie je Konzept (A2, SPEC §8.1, §8.6)**:
    - Reine Domänentypen, IDs, Tags und Fehler existieren in `contextra-types`.
    - Kryptografische Typen und Operationen existieren ausschließlich in `contextra-crypto`.
12. **Ein PR = ein Arbeitspaket (WP) = max. ein Crate-Schnitt (REQ-CHG-2, PLAN §1)**:
    - Vor der Implementierung muss ein fehlschlagender Regressionstest (Red Phase) vorliegen.
    - Nach jeder Änderung: Build, Testlauf, Lints und Gate-Prüfung. Evidenz unter `evidence/<WP-ID>/`.

---

## 3. Architektur-Ringe & Crate-Topologie (SPEC §5.1, §5.2)

```
Ring 4  contextra                               (Fassade, Composition Root)
Ring 3  contextra-erasure   contextra-engine    (Löschkern, Orchestrierung)
Ring 2  contextra-mvcc  contextra-store  contextra-text  contextra-vector  contextra-rank
Ring 1  contextra-sys   contextra-simd   contextra-durable-fs  contextra-crypto
Ring 0  contextra-types contextra-ports
dev     contextra-testkit  contextra-lab  ctx-verify  tests/arch
```

| Ring | Crate | Rolle (Soll laut SPEC §8) |
|---|---|---|
| **Ring 0** | `contextra-types` | Reine Typen (IDs, Tags, Fehler, Löschtypen, KeyBuf). Kein `async`, kein I/O. |
| **Ring 0** | `contextra-ports` | Alle Schnittstellen-Traits (`StorageEngine`, `ErasureLayer`, `ReadGate`, `Vfs`, `Clock`, `Rng`, etc.). |
| **Ring 1** | `contextra-sys` | Plattformprimitive (`fsync`, `flush_dir`, `rename_durable`, `mlock`). `unsafe`-Insel 1. |
| **Ring 1** | `contextra-simd` | f32-Distanzkernel (`l2`, `dot`, `cosine`) mit Skalar-Referenz. `unsafe`-Insel 2. |
| **Ring 1** | `contextra-durable-fs` | `RealVfs`-Implementierung, atomare Datei-Operationen, best-effort Scrub. |
| **Ring 1** | `contextra-crypto` | AEAD (AES-256-GCM-SIV), Argon2id KDF, HKDF, WAL-HMAC, Hash-Kette, Ed25519. |
| **Ring 2** | `contextra-mvcc` | Monotone `Seq`, Snapshots, SSI-Validierung, Snapshot-Widerruf (`revoke_covering`). |
| **Ring 2** | `contextra-store` | Transaktionaler LSM-Tree (WAL, Memtable, Leveled SSTables, Purge-Rewrite & Swap). Blind für Dokumentinhalte. |
| **Ring 2** | `contextra-text` | BM25-Segmentindex (verschlüsselte Segmente, Term-Tags, kein Klartext). |
| **Ring 2** | `contextra-vector` | HNSW-Vektorindex (Slot-Zeroing, verschlüsselte Snapshots, Batch-Rebuild). |
| **Ring 2** | `contextra-rank` | RRF-Fusion (`rrf_fuse`) mit deterministischem Tie-Break nach `doc_tag`. |
| **Ring 3** | `contextra-erasure` | Entkoppelter Löschkern: Saga, Journal, ReadGate (`ErasedSet`), Tresor (`vault`), Messung, Bericht. |
| **Ring 3** | `contextra-engine` | Orchestrierung: Collections, Schreib-/Lesepfade, Transaktionen, Verdrahtung. |
| **Ring 4** | `contextra` | Öffentliche Fassade, `EngineBuilder`, Re-Exporte. |
| **dev** | `contextra-testkit` | `FaultVfs`, `ReferenceModel`, `ManualClock`, `InMemoryStore`, `LeakyLayer`. |
| **dev** | `contextra-lab` | Unabhängige Prüfinstanz ohne Produktions-Code (Rohscan E1, Decoder-Scan E2, E3). |
| **dev/bin** | `ctx-verify` | Standalone CLI-Berichtsprüfer (Ed25519-Signatur, Schema, Klassifikation). |
| **dev** | `tests/arch` | Architektur-, Ring-, Lint- und Supply-Chain-Tests. |

---

## 4. Löschpipeline & DSGVO-Vorgaben (SPEC §11, §12, ADR-001)

Contextra implementiert das Löschen als **eigenen, entkoppelten, gemessenen Prozess**:
- **Zustandsmaschine (SPEC §8.2, §11.2)**:
  `Requested → Gated → Drained → Rewritten → Cleaned → Measured → Completed` (Fehlerfall: `Pending` mit Backoff oder `Failed`).
- **ReadGate (`ErasedSet`)**:
  - Unmittelbar nach Zustand `Gated` blockiert das Gate fail-closed jeden Lese- und Suchzugriff auf Zielschlüssel und Tags.
  - Snapshot-Widerruf: Aktive Snapshots, die Zielversionen sehen könnten, werden widerrufen (`SnapshotRevoked`).
- **Physisches Purging (Ebene A)**:
  - **Store**: `drain` versiegelt WAL; `rewrite_file` erzeugt neue SSTables ohne Zieldaten (Original-Sequenzen bleiben erhalten, I-2); atomarer `swap` via Manifest; Block-Cache Eviction mit Zeroize.
  - **Vector**: `zero_slot` nullt Slot-Bytes im Speicher und markiert Tombstone; Snapshot-Chunks werden neu geschrieben; Rebuild ab Schwelle.
  - **Text**: Textpuffer bereinigen; betroffene Segmente ohne Zieldokumente neu schreiben; Alt-Segmente löschen; Term-Statistik neu berechnen.
- **Residuenmessung & Klassifikation (SPEC §12)**:
  - **E1**: Byte-genauer Rohscan des Datenverzeichnisses nach Needles.
  - **E2**: Strukturierter Scan nach Dekodierung aller Container.
  - **E3**: Nachweis der Schlüssel- und Slot-Abwesenheit.
  - Positivkontrolle (Canary) vor jedem Scan obligatorisch.
- **Löschbericht (`ctx-erasure-report/1`, D-13, SPEC §9.1)**:
  - Ed25519-signiert, maschinenlesbar, enthält keine Klartext-IDs oder Personendaten.
  - Verifizierbar durch die unabhängige Binärdatei `ctx-verify`.

---

## 5. Sicherheits-Defaults (SPEC §10)

- **KDF**: Argon2id mit Parametern $m \ge 65.536\,\text{KiB}~(64\,\text{MiB}), t \ge 3, p \ge 1$ (`derive_master`). HKDF-SHA256 für Passphrasen ist strikt verboten (B-15).
- **Verschlüsselung**: Pflicht in jedem Build ohne Opt-Out (`EncryptionMode::Required`, D-05).
- **Mandantentrennung**: Strikt verpflichtend (`TenantPolicy::Required`, D-07).
- **Entitätsextraktion**: Entfällt vollständig in v1 (`AutoExtractionMode` gestrichen, D-27).
- **Geheimnisverwaltung**: Alle Schlüssel und sensiblen Puffer in `Zeroizing`, kein `Clone`, kein `Debug`-Inhalt, kein `Serialize`.

---

## 6. Phasen, Gates & Arbeitspakete (PLAN.md)

Die Umsetzung folgt der verbindlichen Gate-Reihenfolge:
1. **Phase 0: Bestandsaufnahme & Freeze (G-A)**
   - WP-0.1 (Basislinie messen), WP-0.2 (Undeklarierte Features blockieren), WP-0.3 (Erasure-Freeze), WP-0.4 (Doku-Freeze & Wortlaut-Lint), WP-0.5 (ADR-001 einpflegen).
2. **Phase 1: Rückbau & Entkopplung (G-B)**
   - WP-1.1 (Konsumenten entkoppeln), WP-1.2 (Workspace auf Soll-Crates reduzieren), WP-1.3 (Alten Löschpfad entfernen), WP-1.4 (SIMD u8-VNNI entfernen), WP-1.5 (Tests bereinigen), WP-1.6 (Governance bereinigen).
3. **Phase 2: Härtung Speicherkern (G-C → G0)**
   - WP-2.1 (Snapshot-Sentinel D-C02 beheben), WP-2.2 (Sequenzsemantik festschreiben), WP-2.3 (WAL-Integrität & Replay vereinheitlichen), WP-2.4 (CRC Offset 4 Zeitbox), WP-2.5 (Sicherheitsdefaults erzwingen), WP-2.6 (Vfs konsolidieren & Store migrieren), WP-2.7 (Follower-Commit cancel-safe), WP-2.8 (Lebenszyklus & Drop), WP-2.9 (Zero-Panic & BTreeMap im Store), WP-2.10 (Store Crash-Matrix).
4. **Phase 3: Löschung Dokumente & Betroffene (G-D)**
   - WP-3.1 (Typen `types/src/erasure`), WP-3.2 (Schlüsselcodec & blind tags), WP-3.3 (Dokument-Manifest), WP-3.4 (Insert/Update/Delete & Lock), WP-3.5 (Journal & ErasedSet), WP-3.6 (ReadGate in allen Lesepfaden), WP-3.7 (Purge-Barrier & Drain), WP-3.8 (Purge-Rewrite SSTables), WP-3.9 (Snapshot-Widerruf), WP-3.10 (Saga & `recover`), WP-3.11 (Erase-API & Batch), WP-3.12 (Betroffenen-Löschung).
5. **Phase 4: Indizes & Caches (G-E)**
   - WP-4.1 (Textindex-Segmente & Purge), WP-4.2 (HNSW Slot-Zeroing & Rebuild), WP-4.3 (Blockcache-Eviction mit Zeroize), WP-4.4 (Speicherhygiene).
6. **Phase 5: Nachweis & Bericht (G-F → G0-Entscheidung)**
   - WP-5.1 (Erasure Lab), WP-5.2 (Bericht v1 & Verifier `ctx-verify`), WP-5.3 (`verify`-API), WP-5.4 (Löschcheckliste), WP-5.5 (Wortlaut-Lint).
7. **Phase 6: Verifikation & Härtung (G-G)**
   - WP-6.1 (Leistungsgates), WP-6.2 (Fuzzing & Concurrency), WP-6.3 (Mutationstests), WP-6.4 (Unsafe-Audit & Miri), WP-6.5 (Lieferkette), WP-6.6 (Gate G0 Messung).
8. **Phase 7: Freigabe & v1.0.0 (G-H)**
   - WP-7.1 (Externes Review), WP-7.2 (G-LEGAL), WP-7.3 (Dokumentation), WP-7.4 (Release Candidate & v1.0.0).

---

## 7. Antigravity Skills für dieses Projekt

Verwende bei spezifischen Arbeitsaufträgen die dafür eingerichteten Antigravity-Skills:
- `/contextra-doctor`: Workspace-Integrität, Compiler-Fehler (B-01, B-03), Manifest-Reparaturen.
- `/contextra-deletion-verifier`: Löschkette, Saga, Canary-Validierung, Ed25519 Proofs, Mutationstests.
- `/contextra-security-audit`: Sicherheits-Defaults, Argon2id KDF, Zeroize, Tenant-Isolation, Fail-Closed.
- `/contextra-invariants-gate`: Ring-DAG, Zero-Panic-Audit, Unsafe-Inseln-Überprüfung, Vfs-Compliance.
- `/contextra-product-suite`: Produktstufen (S0 bis S3) und Release-Validierung.
