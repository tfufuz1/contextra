# Technischer Auditbericht: Löschprozesse & Schichtmodell

> **Rolle:** Leitender Ingenieur für technische Löschprozesse mit Datenschutzrecht-Verständnis (Art. 5 Abs. 2, 17, 25, 32 DSGVO; keine Rechtsberatung).
> **Normative Grundlage:** `docs/SPEC.md` v (insb. §§ 6, 11, 12, 13, 16 und Anhänge B, D, F).
> **Prüfstand:** Workspace HEAD (Stand: 2026-11-02).
> **Evidenz-Prinzip:** Aussagen sind EV0 bis zum Beleg durch Quelltext (EV1) und lokale Ausführung (EV2). Zero-Trust: Fehlende Schichten gelten als „nicht erfasst“.

---

## 1. Urteil

Der Löschkern ist an HEAD durch statische Moduldoppelungen in `contextra-erasure` nicht kompilierbar (B12).
Zentrale Löschmechanismen wie Surrogat-Injektion und Kantenreparatur im Vektorindex sowie physischer Purge im Store sind nicht implementiert oder reine Stubs; der Textindex exponiert rohe DocIds.
Der Löschprozess erreicht derzeit für den Gesamtablauf weder Stufe EV3 noch EV4 und ist in der Schichtmatrix überwiegend als „nicht erfasst“ einzustufen.

---

## 2. Befundtabelle

| ID | Datei:Zeile | Soll (SPEC-ID) | Ist | Evidenz [V/M] | Folgetest |
|---|---|---|---|---|---|
| BEF-01 | `crates/contextra-erasure/src/lib.rs:12-21` | Saubere Modulstruktur nach Ring 3 (AR-3, WP-4.1..4.3); Crate muss fehlerfrei kompilieren. | Kompilationsfehler Rustc E-0761: Moduldateien existieren redundant als `src/<mod>.rs` und `src/<mod>/mod.rs` für 10 Module. | [V] EV2 (`cargo check -p contextra-erasure`) | `cargo check -p contextra-erasure` nach Dateibereinigung. |
| BEF-02 | `crates/contextra-erasure/src/measure/positive_control.rs:17-23` | D-72, ER-25, I-19: Typisierte Positivkontrolle muss Kontrollmuster in Scratch-Datei schreiben, scannen und Treffer nachweisen. | Reine Stub-Implementierung: Gibt bedingungslos `Some(ControlEvidence::observed())` zurück, sobald `pattern` nicht leer ist. | [V] EV1 (Quelltext-Inspektion) | `T-RAW-2`: Negativkontrolle und fehlschlagendes Muster müssen `None` liefern. |
| BEF-03 | `crates/contextra-store/src/purge/*.rs`, `erase/*.rs`, `health/*.rs` | SPEC §8.8, §11.4: Purge-Orchestrator mit Memtable-Drain, SSTable-Rewrite, WAL-Floor-Versiegelung und FileKeyring-Vernichtung. | Sämtliche Dateien in `purge/` (6 Dateien), `erase/` (2 Dateien) und `health/` (2 Dateien) sind reine Modul-Einzeiler ohne Code. | [V] EV1 (Quelltext-Inspektion) | `T-CRASH-2`, `T-RAW-1`: Physische Umschreibung und Zeugenausstellung im Store prüfen. |
| BEF-04 | `crates/contextra-vector/src/hnsw/core_rebuild.rs:394-474` | D-77, SPEC §11.2, T-VEC-DEL-1..2: Vor Gate Surrogat-Injektion, im Purge-Fenster Kantenreparatur (FreshDiskANN/Malkov). | Reines Tombstone-Bit (`deleted_nodes.insert`). Weder Surrogat-Vektoren noch Kantenreparatur verbleibender Nachbarn vorhanden. | [V] EV1 (Quelltext-Inspektion & `docs/VECTOR_INDEX_AUDIT.md`) | `T-VEC-DEL-1`: Recall@10-Gate nach Löschung von 5 % und 20 % der Knoten. |
| BEF-05 | `crates/contextra-text/src/wand.rs`, `stream.rs`, `posting_list.rs` | D-08, D-75, SPEC §11.1: Trennung von Speicher-DocId und externer Kennung; Zuordnung über verschlüsselte Tags (`DocTag`). | Direkte Verwendung unmaskierter `DocId` (u64) in Inverted Index, Cursors, WAND und Posting-Listen. | [V] EV1 (Quelltext-Inspektion) | `T-RAW-6`: Needlescan auf Roh-DocIds in Textsegmenten nach Löschung. |
| BEF-06 | `crates/contextra-engine/src/collection/crud/delete.rs:28` | SPEC B-13, §14.2: Exklusiver KV-Lock muss über die gesamte Löschtransaktion bis Commit/Rollback gehalten werden. | `drop(_guard)` erfolgt unmittelbar nach Erwerb vor Beginn der Transaktion; TOCTOU-Race bei parallelen Schreibzugriffen. | [V] EV1 (Quelltext-Inspektion & `docs/ENGINE_REVIEW.md`) | `T-CONC-1`: Paralleles Schreiben und Löschen derselben Dokument-ID unter Last. |
| BEF-07 | `Cargo.toml:4-21` | AR-4, SPEC §15.4: Unabhängige Verifikationswerkzeuge (`contextra-lab`, `ctx-verify`) müssen im Workspace integriert sein. | `crates/contextra-lab` fehlt in `members` und `default-members` von `Cargo.toml`. | [V] EV1 (Quelltext-Inspektion `Cargo.toml`) | `cargo check --workspace` inklusive `contextra-lab`. |

---

## 3. Schichtmatrix des technischen Löschprozesses

Gemäß Zero-Trust-Vorgabe: Jede Schicht, die im Code nicht funktionsfähig implementiert ist, wird als **„nicht erfasst“** ausgewiesen. Vektor- und Textindex gelten erst nach gemessenem Nachweis als erfasst.

| Schicht | Mechanismus | Zeuge | MK-Klasse | Restrisiko nach SPEC §6.4 & Anhang D | Status | Test- / Messverweis |
|---|---|---|---|---|---|---|
| **Store (LSM & SST)** | Memtable-Drain, SSTable-Rewrite ohne Zielfamilien, WAL-Floor-Versiegelung, Dateischlüssel-Vernichtung (`F`-Slot) | `LayerWitness` (Store-Scanner) | MK1, MK2 | `FlashRemanence`, `FsJournalOrCow`, `WrappedKeyRemanence` | **nicht erfasst** (Module in `contextra-store/src/purge` sind Ein-Zeiler) | `T-RAW-1`, `T-RAW-2`, `BM-ERASE-WA` |
| **MVCC-Versionen** | Snapshot-Widerruf, Floor-Fortschreibung in `SequenceLog`, Pruning historischer Versionen | `LayerWitness` (MVCC-Scanner) | MK1 | `ProcessMemory` (flüchtige Snapshots im Heap) | **entfernt** (in `contextra-mvcc` grundgelegt; Pruning aktiv) | `tests/t_snap_phase_a.rs`, `tests/isolation_tests.rs` |
| **Textindex (BM25)** | Textsegment-Rewrite im ersten Fenster (D-69), Entfernen entkoppelter Postings, Neuberechnung von Korpusstatistiken (E-L5) | `LayerWitness` (Text-Scanner) | MK1, MK2 | `TextIndexUntilPurge`, `WrappedKeyRemanence`, `ProcessMemory` | **nicht erfasst** (rohe `DocId` im Index geführt; Rewrite-Orchestrierung fehlt) | `T-RAW-4`, `BM-ERASE-WA-TEXT` |
| **Vektorindex (HNSW)** | Surrogat-Injektion vor Sperre (D-77), Nachbarschafts-Kantenreparatur (Orphan-Reconnect), Rebuild nach Quote | `LayerWitness` (Vektor-Scanner) | MK1 | `IndexTopologyInference`, `ProcessMemory` | **nicht erfasst** (nur Tombstones vorhanden, keine Kantenreparatur, kein Surrogat) | `T-VEC-DEL-1`, `T-RAW-3`, `BM-VEC-DEL` |
| **Journal & Gate-Log** | Append-Log mit Hash-Kette (`gates.log`), Journal-Reduktion nach Abschluss (R13), Epochenschlüssel-Rotation (D-48) | `LayerWitness` (Journal-Hash) | MK1 | `PseudonymCommitment` | **nicht erfasst** (Build-Blockade in `contextra-erasure`) | `T-GATE-1`, `T-JRN-3`, `BM-ERASE-LAT` |
| **Ledger (Restore-Guard)** | HMAC-Kette außerhalb Datenverzeichnis, Bereichsspeicherung (`lo`..`hi`), Erkennung von Zeitregression und Datei-Wiedergängern | `LedgerEntry`-Signatur | MK1 | `BackupContentsOutside` (externe Sicherungen des Betreibers) | **nicht erfasst** (Schnittstelle in `contextra-ports` definiert, Backend-Orchestrierung blockiert) | `T-RESTORE-1..3` |
| **Vault (Schlüsseltresor)** | O(1)-Schlüsselvernichtung: Slot-Nullung (`T`/`C`/`S`/`D`/`F`), Grabstein `0x02`, `sync_data`, Zero-Verification vor Freigabe | `LayerWitness` (Vault-Slot-Prüfung) | MK3 | `WrappedKeyRemanence` (Remanenz alter Slot-Bytes vor Rehash) | **gemessen** (in `contextra-erasure/src/vault/table.rs` und `slot.rs` als In-Memory-Modell implementiert) | `T-VAULT-1..5`, `BM-ERASE-O1` |

---

## 4. Datenschutzrechtliche Bewertung: Pseudonym-Tags & Verknüpfbarkeit (Art. 5 Abs. 2, 17, 25, 32 DSGVO)

> *Hinweis:* Die folgenden Ausführungen stellen eine technische Übersetzung rechtlicher Rahmenbedingungen dar und dienen der Rechenschaftslegung (Art. 5 Abs. 2 DSGVO); sie stellen **keine Rechtsberatung** dar.

### 4.1 Bewertung der Pseudonym-Tags nach Schlüsselvernichtung (E-L2, E-L3, E-L4)

1. **Tag-Konstruktion:**
   Tags (`TenantTag`, `CollTag`, `SubjectTag`, `DocTag`) werden über BLAKE3-Keyed-Hashing (`TagPreimage`) abgeleitet. Nach Vernichtung des Masterschlüssels bzw. des hierarchischen Slots (`T`/`C`/`S`/`D`) ist die mathematische Rekonstruktion des Preimages (personenbezogene Kennung) durch die Einwegfunktion und den Verlust des Schlüsselmaterials rechnerisch unterbunden.
2. **Relative Personenbeziehbarkeit (ErwGr. 26 DSGVO, EuGH C-413/23 P, EDPB 02/2026 Entwurf):**
   - Gemäß der konservativen Auslegung (SPEC §6.6, E-L2, E-L4) behauptet Contextra **keine** automatische Einstufung der Schlüsselvernichtung als vollendete Löschung oder Anonymisierung.
   - Solange pseudonyme Tags im Ledger oder Journal überdauern, weisen sie auf Existenz und Umfang früherer Datenbestände hin.
   - Die Verknüpfbarkeit von Ledger-Bereichen (`lo`..`hi`) zu neu angelegten Datensätzen ist durch die Einweg-Eigenschaft und isolierte Schlüsselsalze für unbefugte Dritte ausgeschlossen, bleibt jedoch für den Store-Betreiber im Rahmen der Zweckbindung „Restore-Schutz“ prüfbar.

### 4.2 Datenminimierung & Zweckbindung (Art. 5 Abs. 1 lit. b, c DSGVO)

- **Journal:** Zwischenzustände werden nach Erreichen von `Completed` bereinigt (Journal-Reduktion, SPEC §11.6, R13). Die Commitments rotieren mit der `commit_epoch` (Standard: 365 Tage), nach deren Ablauf sie invalidiert werden.
- **Ledger:** Verbleibt außerhalb des regulären Datenverzeichnisses (`LedgerSink`). Die Aufbewahrungsdauer ist strikt an `backup_retention` gekoppelt (D-74). Das Restrisiko `PseudonymCommitment` wird im Bericht transparent ausgewiesen.

---

## 5. Nicht geprüft / Restrisiken (SPEC §6.4 & Anhang D)

Folgende Restrisiken verbleiben konstruktionsbedingt außerhalb des Einflussbereichs der lokalen Bibliothek und werden im standardisierten Löschbericht ausgewiesen:

1. **Externe Sicherungen (`ExternalCopies` / `BackupContentsOutside`):** Plattform-Backups und Snapshot-Kopien enthalten Zielrohdaten bis zur manuellen Bereinigung oder zum Einspielen (welches den Restore-Guard triggert).
2. **Flash-Remanenz (`FlashRemanence`):** Physische Speicherblöcke auf SSDs/NVMe-Medien können durch Wear-Leveling und Over-Provisioning alte Ciphertext-Fragmente vorhalten, selbst wenn logische Blöcke überschrieben wurden.
3. **Dateisystem-Journale & CoW (`FsJournalOrCow`):** Dateisysteme wie Btrfs, ZFS oder APFS legen Metadaten und modifizierte Datenblöcke an neuen Stellen ab; In-Place-Zeroing greift dort nicht zuverlässig.
4. **Prozessspeicher (`ProcessMemory`):** Flüchtige Kopien im RAM von Host-Prozessen (z. B. unbereinigte Buffers, Embeddings-Zwischenspeicher vor Zeroization).
5. **Externe Verarbeiter (`ExternalProcessors`):** Vektoren oder Chunks, die an externe Inferenzdienste oder RAG-Caches übertragen wurden.
6. **Schlüsselremanenz im Tresor (`WrappedKeyRemanence`):** Durch Open-Addressing-Rehash entstandene Altdateien des Tresors, sofern sie vor dem Unlink nicht physisch genullt werden konnten (D-83).
7. **Topologische Inferenz (`IndexTopologyInference`):** Verbleibende Graphkanten im Vektorindex vor vollständigem deterministischem Rebuild.
8. **Textindex bis Purge (`TextIndexUntilPurge`):** Nicht bereinigte Postings in Textsegmenten vor Durchlauf des ersten Segment-Rewrites.

---

## 6. Getroffene Entscheidungen mit SPEC-Verweis

Die Implementierung und dieses Audit folgen der konservativsten Auslegung der technischen und rechtlichen Schutzvorschriften:

- **E-L1 (Doppelte Säule):** Physische Umschreibung (Klasse P) und Schlüsselvernichtung (Klasse K) sind kumulativ erforderlich; keine Maßnahme gilt für sich allein als ausreichend.
- **E-L2 (Keine Konformitätsbehauptung):** Contextra beansprucht für die Schlüsselvernichtung weder den Status „Löschung“ noch „Anonymisierung“. Der Bericht belegt ausschließlich gemessene technische Zustände.
- **E-L3 (Embeddings personenbezogen):** Embeddings und HNSW-Topologien werden vorsorglich als personenbezogene Daten behandelt. Surrogat-Injektion und Kantenreparatur sind Pflicht.
- **E-L4 (Pseudonymität im Ledger):** Der Ledger führt nur Commitments, Bereichs-Tags und Sequenznummern; seine Vorhaltezeit ist strikt an `backup_retention` gebunden.
- **E-L5 (Textstatistiken):** Korpusstatistiken im Textindex dürfen keine gelöschten Term-Frequenzen überdauern und sind beim Rewrite deterministisch neu aufzubauen.
- **E-L11 (Reichweite der Signatur):** Die Ed25519-Signatur des Berichts bestätigt Herkunft und Integrität aus der lokalen Installation, nicht die materielle Wahrheit oder rechtliche Konformität.
- **E-L13 (Wortlaut-Restriktion):** Das Löschergebnis wird ausschließlich nach den Vorgaben aus SPEC §6.4 beschrieben. Alle nach T-LINT-1 untersagten Begriffe für das Löschergebnis sind strikt ausgeschlossen.
- **E-L15 (Entkoppelte Rückstände):** Entkoppelte Postings und unreparierte Kanten gelten bis zum Vollzug des Purge-Fensters als personenbezogene Rückstände mit auszuweisendem Restrisiko.
- **E-L16 (Pseudonyme Dateinamen):** Datei- und Pfadnamen verwenden ausschließlich kryptografische Tags und Hashes (D-75). Klartext-Kennungen sind verboten.
