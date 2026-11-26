# IMPL-05 — `contextra-mvcc`: Prolly-Tree, RootSet, MemDelta, Fold, Typestate-Txn, GC

**Phase:** P1/P2 · **SOLL:** SPEC §4, §11, §12.4, INV-1, INV-3, INV-4, INV-7 · **Abhängig von:** IMPL-01, 03, 04 · **Crates:** `contextra-mvcc` (L3), `contextra-checkpoint`

## 1. Ziel

Truth-Layer als **Prolly-Trees** (content-defined chunking, strukturelles Teilen), zusammengefasst in `RootSet`. Schreibpfad: `Txn<ReadWrite>` → `MemDelta` → WAL-Commit → **Fold** in neue Roots → atomares Publizieren eines `Commit`-Objekts im Manifest. Indizes (IMPL-06) sind **abgeleitet** und tragen `source_truth_hash` + `watermark_seq`.

## 2. IST → SOLL

| IST | SOLL |
|---|---|
| MVCC/SSI über Sequenznummern im LSM; `store` hängt von `mvcc` (Inversion) | `mvcc` (L3) **oberhalb** `store`/`wal`; Abhängigkeit nur nach unten |
| Snapshot = Sequenznummer | Snapshot = `RootSetRef` (Hash) + `Seq` |
| Einzelne KV-Ebene | 7 Trees: `primary, sequences, graph, aliases, policies, conflicts, provenance` |
| Konflikte via SSI-Tracking (`ssi_max_tracked_keys`) | Branch-Merge mit Gates G1–G7 (IMPL-08) + 3-Wege-Prolly-Merge |

## 3. Datenstrukturen

```rust
pub struct RootSet {
    pub primary: TreeRoot, pub sequences: TreeRoot, pub graph: TreeRoot,
    pub aliases: TreeRoot, pub policies: TreeRoot, pub conflicts: TreeRoot,
    pub provenance: TreeRoot,
}                                           // kanonisch serialisiert, Hash = root_set_hash
pub struct TreeRoot { pub hash: [u8;32], pub height: u8, pub count: u64 }
pub struct Commit {
    pub parents: SmallVec<[CommitId;2]>, pub root_set: RootSetRef, pub seq: Seq,
    pub hlc: Hlc, pub author: ActorId, pub msg_hash: [u8;32], pub sig: Option<Sig64>,
}                                           // CommitId = BLAKE3(kanonische Bytes)
pub struct IndexManifestRef { pub hash:[u8;32], pub source_truth_hash:[u8;32], pub watermark_seq: Seq }
```

**Prolly-Node:** Boundary-Funktion über Rolling-Hash der Schlüssel (Ziel-Knotengröße 4 KiB ±, min 512 B, max 16 KiB), Blätter `(key, value|ref)`, innere Knoten `(max_key, child_hash)`. Knoten sind Objekte im `ObjectStore` (IMPL-04), adressiert durch Hash. Schlüsselkodierung je Tree fest (z. B. `primary`: `ns|cell_key`; `sequences`: `seq be`; `graph`: `src|kind|dst`).

## 4. Transaktionsmodell (Typestate)

```rust
pub struct Txn<M: Mode> { /* privat */ }
pub enum ReadOnly {} pub enum ReadWrite {}
impl Txn<ReadOnly>  { pub fn get(..); pub fn scan(..); pub fn root_set(&self)->RootSetRef; }
impl Txn<ReadWrite> { pub fn put_cell(..); pub fn put_edge(..); pub fn retire(..); pub fn commit(self, d: Durability)->Result<CommitInfo,TxnError>; pub fn abort(self); }
```

- Schreib-API existiert **nur** auf `ReadWrite` (Compile-Zeit-Garantie; `compile_fail`-Doctests).
- **Sequencer-Actor pro Branch:** einziger Schreiber der Branch-Head-Referenz; sammelt `MemDelta`s, vergibt `commit_seq` (dicht, monoton je Branch), schreibt WAL-`Commit` (Group Commit), danach `Fold`.
- **Optimistische Validierung:** `ReadWrite` merkt `read_snapshot_seq`; Konflikt = anderes Commit hat Schlüssel im Lesebereich geändert → `TxnError::Conflict{keys}`; Wiederholung liegt beim Aufrufer.

## 5. MemDelta und Fold

- `MemDelta`: sortierte Änderungsmenge je Tree (BTreeMap oder sortierter Vec + Bloom), abgefragt als Overlay auf dem Basis-Root (Lesen sieht Basis + Delta).
