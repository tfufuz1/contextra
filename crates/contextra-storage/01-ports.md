# SP-PORTS — `contextra-storage-ports` (Verträge, Werttypen, Fehler)

| | |
|---|---|
| Crate | `crates/contextra-storage-ports` (keine Logik, `#![forbid(unsafe_code)]`) |
| Besitzt | alle Werttypen und Traits, die ≥ 2 Komponenten teilen; das `Storage`-Fassadenprofil (Signaturen, nicht Rümpfe) |
| Konsumiert | nichts (nur `bytes`, `zeroize`, `thiserror`) |
| Spec-Bezug | R8 §3.2, §3.5, §4, §12, §15 |
| Phase | P0, Gate G-0 (API-Review-Freeze) |

## 1 Auftrag und Abgrenzung
Einzige Quelle für Typen und Traits. Jede andere Komponente importiert von hier und definiert **keine** konkurrierenden Typen.
**Darf nicht:** Logik, Serialisierung (→ SP-CODEC), I/O, Abhängigkeit auf `contextra-ports` (Altstand). Der Altstand-Crate `contextra-ports` wird **nicht** re-exportiert; er bleibt bis zum Cutover unverändert (siehe Master G-03).

## 2 Öffentliche Schnittstelle
```rust
// Identität (Debug redigiert, kein Display)
pub struct ClusterId([u8; 16]);  pub struct SpaceId(pub u16);  pub struct BranchId(pub u32);
pub type Stamp = u64;            pub const STAMP_MAX: Stamp = (1 << 56) - 1;
pub struct TombId([u8; 32]);     pub struct IdemToken([u8; 16]);   pub struct Ticket([u8; 16]);

pub enum Protection { Sealed, Plain }
pub enum Durability { Durable, Relaxed }
pub enum Origin { User, Maintenance, Replication, Migration }       // NEU (E-04), 1 Byte im Frame
pub struct CallerMeta { pub origin: Origin, pub caller_seq: u64 }    // NEU (E-04), z. B. TxId des Aufrufers

pub struct Change { pub ops: Vec<Op>, pub guards: Vec<Guard>, pub idem: Option<IdemToken>,
                    pub durability: Durability, pub allow_mixed: bool, pub caller: CallerMeta }
pub enum Op { Put { space: SpaceId, cluster: ClusterId, branch: BranchId, key: Bytes, header: Bytes,
                    body: Bytes, weights: [u32; 4] },
              Delete { .. }, RangeDelete { .. /* start,end exklusiv */ } }
pub enum Guard {
    Key   { space: SpaceId, cluster: ClusterId, branch: BranchId, key: Bytes, pre: Precondition },
    Range { space: SpaceId, cluster: ClusterId, branch: BranchId, start: Bytes, end: Bytes, since: Stamp }, // NEU (E-05)
}
pub enum Precondition { Absent, Version(Stamp), Unchanged { since: Stamp } }
pub struct Receipt { pub stamp: Stamp, pub durable: bool }
```
**Traits** (Signaturen in R8 §3.2/§9.4/§12.1; Abweichungen/Ergänzungen hier):
- `Fs`/`File`: `read_at`, `append_prealloc`, `preallocate` (**Nullen schreiben**, nie sparse), `sync_data/sync_all/sync_dir`, `rename`, `lock`, `list`, `free_space`, `caps() -> FsCaps`, optional `link/reflink/punch_hole`. Jede Operation trägt einen `IoCtx` (Datei-Rolle, Zweck) für `FaultFs`-Indizierung.
- `KeyProvider`: R8 §9.4 **plus** `create_cluster_keys(&self, &[ClusterId], &dyn Entropy) -> Result<Vec<u32>, KeyError>` (**Batch, ein Durabilitätspunkt**, E-02).
- `Anchor`, `Clock`, `Entropy`, `Cipher`, `Mac`, `Kdf`, `Checksum`, `Authority`, `Metrics`, `Observer`, `ColdTier`, `Spawner`.
- `Memtable` (Skiplist hinter Trait), `SegmentOwner`, `ExtensionBuilder`, `ConsumerFeed` (Verträge §12).
- `Error` (Klassen F-1…F-9) mit `hint() -> &'static str` und `class() -> ErrorClass`; Texte ohne Schlüssel/Nutzdaten (Test mit Canaries).

**Fassadenprofil** `Storage` (Rümpfe verteilt per `impl Storage` auf Komponenten; Besitzer in Klammern):
`open/close` (SP-CATALOG) · `apply` (SP-COMMIT) · `view/view_at/view_durable` (SP-READ) · `clusters()`, `spaces()`, `fork/branch_drop/merge/diff` (SP-CLUSTER) · `eraser()` (SP-ERASE) · `consumers()`, `segments()` (SP-EXT) · `checkpoint()/restore()` (SP-CKPT) · `inspector()` (SP-VERIFY, nur `testing`).

## 3 Regeln
1. Jede Änderung an einem Trait nach G-0 braucht Review durch SP-PORTS-Owner und alle Konsumenten (RFC im PR).
2. Typen sind `#[non_exhaustive]` nur dort, wo R8 Erweiterung vorsieht (Op-Kinds, Fehler); Wire-Formate hängen nie an `repr` dieser Typen (→ SP-CODEC).
3. `Debug` für `ClusterId`, `Key32`, Körper ist `Redacted<T>`.
4. Zeitquellen: kein `SystemTime`/`Instant` in Signaturen; nur `Clock`.

## 4 Fehlerverhalten
Fehlerklassen und Reaktion exakt nach R8 §3.5. `Error::Poisoned` ist klebrig und trägt Ursache (`PoisonCause`) ohne Inhalte.

## 5 Invarianten / Tests
`INV-CLUSTER-2` (kein Konstruktor aus Kennung), `INV-LIM-1` (Limit-Konstanten), Redaktions-Test (Canary in Debug/Display/Error → 0 Treffer), Compile-fail-Doctests: `ClusterId` nicht aus `&[u8]` ableitbar außer `from_wire` (nur `pub(crate)`-Pfad via SP-CODEC-Feature `wire`).

## 6 Lücken / Errata (Beschlüsse)
- **E-02** Batch-Schlüsselanlage im `KeyProvider` (sonst 1 Fsync je Cluster; Ingest von 10⁶ Dokumenten unbrauchbar).
- **E-04** `CallerMeta` im Frame-Kopf: trägt die TxId des Aufrufers (Altstand-Abbildung `last_tx_id`, `rollback_to_tx`, Observer-`WriteOrigin`). **Muss vor G-F** entschieden sein, sonst Migration.
- **E-05** `Guard::Range` (Phantom-Schutz für `scan_prefix_tracked`).
- Zwei `Clock`-Traits im Altstand (`clock.rs`, `persistence.rs`): hier genau einer.

## 7 Arbeitspakete
| ID | Titel | PW | Abhängig von | Phase | Abnahme |
|---|---|---|---|---|---|
| WP-01.1 | Werttypen, IDs, `Redacted`, `Change/Op/Guard`, `Error` + `hint()` | 2.0 | — | P0 | Redaktions-/Compile-fail-Tests grün |
| WP-01.2 | Port-Traits (Fs, Clock, Entropy, Anchor, KeyProvider inkl. Batch, Cipher/Mac/Kdf/Checksum, Authority, Metrics, Observer, ColdTier, Spawner) | 2.0 | WP-01.1 | P0 | Mock-Impl je Trait kompiliert; Layering-Test |
| WP-01.3 | Erweiterungstraits + `Storage`-Fassadenprofil + API-Review-Freeze | 1.5 | WP-01.2 | P0 | zwei Reviewer, G-0 |

## 8 Abnahme
Crate baut ohne Abhängigkeit auf Altstand-Crates; `cargo doc` vollständig; `layering`-Test grün; jede in R8 §3.2 genannte Port-Fähigkeit hat genau einen Trait.
