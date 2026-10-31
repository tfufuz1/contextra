# Contextra Deletion Pipeline Architecture Reference

Diese Referenz dokumentiert die normative Spezifikation des Löschsystems aus SPEC §12 und PLAN §10.

## 1. Wirkungsklassen

Contextra unterscheidet vier messbare Klassen:

- **Klasse L (Logisch gesperrt)**:
  - Der Scope ist im `ErasedSet` registriert und per `fsync` persistent.
  - Alle Lese- und Suchpfade weisen Anfragen auf den Scope sofort ab (`Fail-Closed`).
- **Klasse P (Klartext bereinigt)**:
  - Ebene-A-Purge ist auf allen flüchtigen und speicherinternen Strukturen (Memtable, HNSW, BM25 Postings, Caches) abgeschlossen.
  - Der Canary-Scan bestätigt die Abwesenheit von Klartextmustern.
- **Klasse C (Persistente Reste unlesbar)**:
  - Der dokumentspezifische `DocDek` wurde im `KeyTable` vernichtet.
  - Das Chiffrat auf persistenten Medien (SSTables, WAL) ist mathematisch unentschlüsselbar.
- **Klasse E (Technisch vollständig)**:
  - Klasse P und Klasse C sind lückenlos für alle im Scope deklarierten Schichten vollzogen.
  - Jede Schicht hat einen gültigen `LayerWitness` geliefert.

## 2. Invarianten des Löschpfads

- **INV-DEL-ORDER-1**: Ebene A (Klartext-Purge) MUSS vor der Schlüsselvernichtung (Ebene B) abgeschlossen sein. Purger müssen noch entschlüsseln können, um Multi-Dokument-Blöcke neu zu schreiben.
- **INV-NO-RESURRECTION-1**: Ein gelöschter Scope darf nach einem Crash, Restore oder Replay niemals wieder lesbar werden.
- **INV-CLASS-MEASURED-1**: Ein Bericht darf niemals eine Klasse ausweisen, deren Voraussetzungen nicht von einem Schicht-Scanner gemessen wurden.
- **VETO-F02**: HNSW-Löschung erfolgt durch Markierung, Slot-Nullung (`zeroize_and_free_node`) und Rebuild. Es erfolgt KEIN dynamisches Rewiring der Nachbarkanten.

## 3. Schlüsselhierarchie

```
Root-Passphrase
   │
   ▼ Argon2id (m >= 64 MiB, t >= 3, p >= 1)
KEK_root
   │
   ▼ HKDF-SHA256 (Context || Tenant || Collection)
KEK_coll
   │
   ▼ Wrap
DocDek (Dokument-Schlüssel, 32 Byte CSPRNG)
   │
   ├── Chunk-Text (AEAD)
   ├── Vektoren (AEAD)
   └── Metadaten (AEAD)
```

Jedes Chiffrat bindet zusätzliche authentifizierte Daten (AAD) an `(Tenant, Collection, DocId, KeyGeneration, SuiteId)`.
