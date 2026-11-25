# SYS-2 — Datenmodell und Byteformate

Ersetzt R14 §3, §4. Abhängigkeiten: SYS-1 (Typen), SYS-4 (Zahlen). Gate: **G-F** (Formatfreeze). Alle Zahlen in `[ ]` verweisen auf `SYS-4`. Alle Mehrbyteangaben in Dateien sind little-endian, **ausgenommen** die Felder der AAD (big-endian, §3.2) und `ClusterId`/`block_uid`/Tags (rohe Bytes).

## 1. Datenmodell

| Begriff | Definition |
|---|---|
| Space | Benannter, physisch getrennter LSM-Baum; `SpaceId u64`, nie wiederverwendet (`INV-ID-2`). IDs `< 256` sind Kernspaces. Der Name ist ein **opaker Token** (≤ 64 B, `[a-z0-9-]`), vom Aufrufer gewählt; Klartextnamen von Sammlungen stehen nie im Catalog (DEC-61). |
| Cluster | Löschdomäne = Schlüsseldomäne = Versionsdomäne = Platzierungseinheit. `ClusterId` 128 Bit, vom Kern aus `Entropy` erzeugt (`INV-CLUSTER-1/2`). Ohne `clustered` ist der Cluster implizit `NONE` (alle Bytes 0). |
| Branch | clusterlokale Version; `branch u32`, `0 = main`; IDs je Cluster nie wiederverwendet (`INV-BR-4`). |
| Schlüssel | Bytes, `0 < len ≤ max_key_bytes`. Reihenfolge `cmp`: Cluster aufsteigend, Branch aufsteigend, Nutzerschlüssel bytelexikografisch, Tag absteigend. Kein `memcmp` über eine Konkatenation. |
| Tag | `u64 = (stamp << 8) \| kind`; `Put=1, Delete=2, RangeDelete=3, EraseRange=4, PutRef=5 (nur Runs), Merge=6 (reserviert; Decoder ⇒ Unsupported)`; alle anderen Werte ⇒ Formatfehler. |
| Wert | `header ‖ body`, getrennt durch `header_len`; Kern interpretiert den Kopf nie. `weights: [u32; W]`, `W = weight_slots ∈ 0…4`. Körper ≥ `value_separation_min` ⇒ Blob (`PutRef`); Kopf immer inline. |
| Stamp | siehe SYS-1 §2. Erster = 1, streng monoton, nie doppelt (auch nicht nach Absturz), Lücken zulässig, `< 2^56`. |

**Packgruppe (DEC-88).** `PackKey = (tenant_tag[16], due_bucket u32, shard u8)`; `due=None ⇒ due_bucket=0xFFFF_FFFF (NONE)`. `due_bucket` und `shard` werden aus **instanzweiten** Parametern berechnet, die in `FORMAT` stehen und nach dem ersten Cluster unveränderlich sind: `pack_bucket ∈ {Month, Quarter, Year}`, `pack_shards ∈ 1…255`. `due_bucket` = Index des Kalender-Buckets des `due` (UTC, Monat: `jahr·12+monat0`, Quartal: `jahr·4+q0`, Jahr: `jahr`); `shard = siphash13(shard_key, cluster_id) mod pack_shards` mit `shard_key` = 16 B aus `FORMAT`. Damit ist die Packgruppe eine Eigenschaft des **Clusters** (ein 4-B-`pack_group`-Index in `ClusterIndex`), nicht des Space. `SpaceOptions.packing` reduziert sich auf `PackMode ∈ {Unpacked, Packed}`; R14s Space-lokales `PackPolicy{bucket, shards}` entfällt (GAP-15).

## 2. `SpaceOptions` und Gültigkeitsmatrix

```rust
pub struct SpaceOptions {            // unveränderlich nach space_add; kanonisch kodiert (§5.6 Op 0x0001)
    pub class: SpaceClass,           // Durable | Volatile
    pub clustered: bool,
    pub data_class: DataClass,       // NonPersonal | Personal
    pub integrity: IntegrityMode,    // Crc | Authenticated | Encrypted
    pub packing: PackMode,           // Unpacked | Packed
    pub pure_threshold: u64,         // [pure_threshold]
    pub seal: Seal,                  // ByMode | Cluster
    pub weight_slots: u8,            // 0..=4
    pub history: HistoryPolicy,      // KeepLatest | KeepUntilFloor | AppendOnly
    pub value_separation_min: u32,
    pub max_key_bytes: u32,          // ≤ 4096
    pub compression: Compression,    // v1: None
    pub durability_default: Durability, // Durable | Relaxed
}
```

Die Validierung läuft vor jeder E/A in `create_space`. Jede Regel hat einen Negativtest (`space_options_matrix`).

| Regel | Aussage | Fehler |
|---|---|---|
| S-1 | `data_class = Personal ⇒ clustered ∧ integrity = Encrypted`; bei `Durable` zusätzlich `seal = Cluster` | `InvalidInput` |
| S-2 | `seal = Cluster ⇒ clustered ∧ integrity = Encrypted`; Space nimmt nur `Sealed`-Cluster auf | `InvalidInput` |
| S-3 | `seal = ByMode ⇒` nur `Plain`-Cluster (oder implizit `NONE`); Schlüssel = Space-DEK bei `Authenticated`/`Encrypted`, keiner bei `Crc` | `InvalidInput` |
| S-4 | `packing = Packed ⇒ clustered` | `InvalidInput` |
| S-5 | `history = AppendOnly ⇒ data_class = NonPersonal` (`INV-HIST-1`) | `InvalidInput` |
| S-6 | `compression ≠ None ⇒ Unsupported` vor Mutation | `Unsupported` |
| S-7 | `weight_slots ≤ 4`; `max_key_bytes ≤ 4096`; `value_separation_min ∈ [256 B, max_body_bytes]`; `pure_threshold ≥ 1 MiB` | `InvalidInput` |
| S-8 | `class = Volatile`: kein Redo/Run/Blob; im Catalog nur `space_add` (Name + Optionen); `integrity` wird ignoriert (muss `Crc` sein); `Personal` erlaubt (keine Durability, **kein** Löschnachweis, Löschung = Freigabe mit Überschreiben, best-effort) | — |
| S-9 | Ein Space enthält nur Cluster **einer** Schutzklasse (`INV-SEAL-1`); die Engine legt je (Collection, Schutzklasse) einen eigenen Space an | `InvalidInput` |
| S-10 | Ein `Sealed`-Cluster mit `DataClass::NonPersonal` darf in einem `Seal::Cluster`-Space liegen, dessen `data_class` `NonPersonal` ist | — |

**Systemspaces (DEC-57):**

| Name | `SpaceId` | Optionen | Inhalt |
|---|---:|---|---|
| `SYS_BRANCH_S` | 1 | Durable, clustered, `Personal`, `Encrypted`, `Seal::Cluster`, `Packed` | Branch-Metadaten von `Sealed`-Clustern |
| `SYS_IDEM` | 2 | Durable, nicht clustered, `NonPersonal`, `Encrypted`, `Seal::ByMode`, eigener System-DEK, `Unpacked` | Idempotenzrecords (CYC-1 §6) |
| `SYS_BRANCH_P` | 3 | Durable, clustered, `NonPersonal`, `Encrypted`, `Seal::ByMode`, `Packed` | Branch-Metadaten von `Plain`-Clustern |
| reserviert | 0, 4…255 | — | versionierte Kernspaces |

Öffentliche `spaces()`/`apply()`/Drop-Pfade weisen jede ID `< 256` mit `ReservedSpace` ab; nur der Kern greift intern zu. Änderungen an Systemspaces nur per Migration (§9).

## 3. Siegel

### 3.1 Siegelrahmen

```
Encrypted     SealFrame := key_epoch u32 | nonce[24] | ciphertext[n] | tag[16]      overhead 44 B
Authenticated SealFrame := key_epoch u32 | plaintext[n] | tag[16]                    overhead 20 B   (tag = HMAC-SHA256-128)
Crc           SealFrame := plaintext[n] | crc32c u32                                 overhead  4 B
```
Der umgebende Container gibt die Länge (`len u32` vor dem Rahmen). `key_epoch` ist bei `Seal::ByMode` die Space-DEK-Epoche (derzeit stets 0), bei `Seal::Cluster` die Cluster-Schlüsselepoche. Nonces kommen ausschließlich aus `Entropy` (24 B zufällig); es gibt keinen Zähler und keine HKDF-Ableitung je Rahmen. Der Verbrauch je Schlüssel wird gezählt; bei `2^48` Siegeln je (Schlüssel, Epoche) meldet der Kern `rotation_recommended` (Observer), erzwingt aber nichts.

`Authenticated`: `tag = HMAC-SHA256(auth_key, "ctxs-auth-v1" ‖ 0x02 ‖ u32le(len(aad)) ‖ aad ‖ plaintext)[0..16]`, vergleichen konstantzeitlich; `auth_key = HKDF(Space-DEK, "ctxs space auth v1")`. `Crc`: `crc32c(aad ‖ plaintext)` (bindet Position, bietet **keine** Manipulationsauthentizität). `Encrypted`: XChaCha20-Poly1305 mit `aad`.

### 3.2 AAD (Teil der Referenzvektoren, alle Ganzzahlen big-endian)

```
aad := "ctxs-aad-v1" ‖ kind u8 ‖ instance_id[16] ‖ space_id u64 ‖ cluster_id[16] ‖ key_epoch u32 ‖ tail
tail(kind=1 Block)    := block_uid[16]
tail(kind=2 RedoOp)   := op_uid[16] ‖ op_index u16
tail(kind=3 Blob)     := record_uid[16]
tail(kind=4 Section)  := section_uid[16]             (MiniSection und Abschnitts-Metablöcke)
tail(kind=5 PartBlock):= partition_tag[16] ‖ generation u32 ‖ block_uid[16]   (ohne space_id, cluster_id: beide 0)
tail(kind=6 Footer)   := file_id u64
```
**Entscheidung DEC-58:** `block_uid`, `record_uid`, `section_uid`, `op_uid` sind **128 Bit zufällig** (Entropy), bei Erzeugung des Objekts vergeben und im Container im Klartext gespeichert (SubIndex, Record-Kopf, Directory). Sie sind **nicht** abschnittsrelative Indizes. Begründung: ein positionsrelativer Index lässt Blockvertauschung zwischen Dateien desselben Clusters und derselben Epoche unbemerkt; eine Zufalls-UID bindet jeden Block an genau eine Entstehung. Rohkopie (Purge, Compaction ohne Merge) behält die UID und bleibt gültig. Der Leser prüft: die UID im SubIndex-Eintrag **ist** die AAD-UID; ein vertauschter Block scheitert am AEAD. Eine abschnittsrelative Ordnungszahl (`block_ordinal u32`) ist ein getrenntes Feld und geht in keine AAD ein; `ExtensionBuilder::block_boundary` erhält `block_ordinal`, nicht die UID.

### 3.3 Schlüsselquelle

`Seal::Cluster` (Sealed-Cluster): `Clusterschlüssel(cluster, key_epoch)` aus dem `KeyProvider`; im Redo immer die im `ops_head` gebundene `key_epoch`. `Seal::ByMode`: Space-DEK. Redo-Kette: `redo_key = KDF(RootKey, "ctxs redo-auth v1")`; Catalog-Kette: `catalog_key = KDF(RootKey, "ctxs catalog-auth v1")`; Ledger: `ledger_key = KDF(RootKey, "ctxs ledger key v1" ‖ instance_id)`; `file_key = KDF(RootKey, "ctxs file-auth v1")`; `FORMAT-MAC` mit `KDF(RootKey, "ctxs format-auth v1")`. Alle Ketten sind **immer** MAC-gekettet, unabhängig von `integrity` (DEC-59).

**Klartext bleiben ausdrücklich** (Metadaten-Leckagebudget, keine Nutzwerte): `ClusterId`, Op-Tags/Routing-Header, Gate-/Control-Header, Größen, Stamp-Bereiche, Dateistruktur, `tenant_tag`/Packgruppe, `IdemToken`, `SpaceId`, `branch`.

## 4. Dateien und Verzeichnisse

```
<root>/FORMAT  LOCK  CURRENT  CATALOG-<n>  KEYS  RESERVE
       redo/redo-<n>.log      spaces/<space_id>/<s>/<file_id>.run|.blob   (s = file_id mod file_shard_count, 3 Hex)
       segments/<kind>/<id>.seg      ckpt/<name>/…
<ledger_dir>/ERASE-LEDGER/ledger-<n>.log      (Standard <root>/ERASE-LEDGER; Produktion: anderes Gerät)
<keyring_dir>/…                               (nur Referenzprovider, CYC-4 §6; nie unter <root>)
```
Alle Dateien `Perm::Private` (0600, Verzeichnisse 0700). Temporär `.building.<tmp_id>`; `tmp_id` aus `reserve_ids`. Verwaiste Dateien entfernt nach Recovery (`INV-CAT-1`). Kein Recovery-/Restore-Pfad vergibt IDs aus alten Catalog-Ständen erneut (`INV-ID-1`): `file_id`/`tmp_id`/`branch_id`/`SpaceId` kommen aus Reservierungsfenstern (`reserve_*`), und `next_*` nach Absturz ist `max(reserve-limit, größte gesehene ID) + 1`.

### 4.1 Dateikopf (64 B, alle Dateiarten)

| Offset | Feld | Wert |
|---:|---|---|
| 0 | `magic[8]` | `"CTXS"` + 4 ASCII je Art: `FRMT KEYS CATL REDO RUN_ BLOB SEGM LEDG CKPT RSRV` |
| 8 | `file_kind u8` | 1 FORMAT, 2 KEYS, 3 CATALOG, 4 REDO, 5 RUN, 6 BLOB, 7 SEGMENT, 8 LEDGER, 9 CKPT, 10 RESERVE |
| 9 | `format_ver u16` | eine Version je Dateiart (v1 = 1) |
| 11 | `feature_bits u32` | Bit 0 `BRANCHES`, Bit 1 `COMPRESSION` (v1: stets 0), Bit 2 `FILTER_BLOOM` (stets 1), übrige 0 |
| 15 | `file_id u64` | 0 bei FORMAT/KEYS/CURRENT |
| 23 | `instance_id[16]` | |
| 39 | reserviert 21 B | **0, wird geprüft** |
| 60 | `header_crc u32` | CRC32C über Bytes 0…60 |

Längen werden vor dem Allokieren gegen die Limits geprüft; unbekannte Version oder Pflichtbits ⇒ `Unsupported` (`INV-FMT-1`); `BRANCHES` ungesetzt und `branch ≠ 0` ⇒ Formatfehler. `varint` = kanonisches LEB128 (nicht minimal ⇒ Formatfehler).

## 5. Formate im Einzelnen

### 5.1 `FORMAT`, `KEYS`, `CURRENT`, `LOCK`, `RESERVE`

`FORMAT` (unveränderlich): Dateikopf · `created_utc i64` · `pack_bucket u8` · `pack_shards u8` · `shard_key[16]` · `file_shard_count u16` · `block_target u32` · `min_reader_ver u16` · `writer_ver u16` · `kdf_params`(`argon2id`: m,t,p; nur bei `FilePassphrase`) · `format_mac[32]` (HMAC unter `format_key`). `KEYS`: Dateikopf · `kek_id[16]` · `wrapped_root` (SealFrame, Schlüssel = KEK, AAD `"ctxs-aad-v1" ‖ 0x07 ‖ instance_id`) · `n_dek u32` · je Space-DEK `(space_id u64, SealFrame unter RootKey, AAD …‖space_id)` · `keys_crc`. **Clusterschlüssel nie hier.** `CURRENT`: eine Zeile `CATALOG-<n>\n` plus `crc32c`; Ersetzen per `tmp` → `sync_all` → `rename` → `sync_dir`. `LOCK`: leere Datei, `Fs::lock(Exclusive)`; `ReadOnly` hält `Shared`. `RESERVE`: vorallokierte Notreserve (`erase_reserve_bytes`, `fallocate`, nie beschrieben, außer Reservesegmente nach §5.3).

### 5.2 Katalog der Tags

`op_kind` im Redo: `1 Put · 2 Delete · 3 RangeDelete`. `frame.kind`: `1 Change · 2 Gate · 3 Control · 4 Reserve`. `control.op`: `1 Restrict · 2 Lift · 3 Link · 4 BranchFork · 5 BranchDrop · 6 BranchMerge`. `gate.scope`: `1 Clusters · 2 Space · 3 Range · 4 Branch` (DEC-81; R14 nannte nur zwei Scopes, obwohl §8.10 `EraseRange` und Branch-Erase als Gate-Marker verlangt).

### 5.3 Redo

**Segment** (`redo-<n>.log`, vorallokiert `[redo_segment_bytes]`): Dateikopf · `SegmentHead{first_stamp u64, prev_chain_h[32], seg_seq u64}` · Gruppen · `SegmentSeal` (letzte Gruppe, nur beim Rotieren: `last_stamp, n_frames, final_h[32]`) · Nullbereich.

**Gruppe** (beginnt immer auf einer `redo_align`-Grenze (4096), mit Nullen aufgefüllt): `GroupHead{gmagic "GRP1" u32, group_len u32, n_frames u16, first_stamp u64, hcrc u32}` (22 B) · Frames lückenlos. **Entscheidung DEC-81:** die Ausrichtung macht die Torn-Tail-Suche (§5.3.1) eindeutig; die Kosten (≤ 4 KiB Füllung je Fsync-Gruppe) sind kleiner als die ohnehin geschriebene Gerätebockgröße.

**Frame** `head ‖ body ‖ tail`:

| Feld | Typ | Bedeutung |
|---|---|---|
| `len` | u32 | Länge `head‖body‖tail` |
| `stamp` | u64 | Stamp des Frames (Reserve: 0) |
| `kind` | u8 | §5.2 |
| `n_ops` | u16 | ≤ `max_change_ops` (Gate: ≤ 4096 Cluster; Control: 1) |
| `idem_token` | [u8;16] | 0 = keiner |
| `policy_epoch` | u64 | Wert **nach** Anwendung des Frames; Link/Restrict/Lift/Gate erhöhen ihn um 1; alle anderen Frames tragen den aktuellen Wert. Stellt `policy_epoch` über Neustarts monoton wieder her (DEC-81; R14 nannte das Feld `epoch` ohne Bedeutung) |
| `flags` | u8 | Bit 0 `RELAXED`, Bit 1 `MERGE`, Bit 2 `FALLBACK_INTENT` |
| `ops_head[n_ops]` | 53 B je Op | `space_id u64 · cluster[16] · branch u32 · op_kind u8 · key_epoch u32 · op_uid[16] · payload_len u32` |
| `body` | | Nutzlasten, eine `SealFrame` je Op, Reihenfolge wie `ops_head` |
| `crc32c` | u32 | über `head‖body` |
| `chain_mac` | [u8;16] | `H_i[0..16]`, `H_i = HMAC(redo_key, H_{i-1} ‖ head ‖ payload_tags ‖ crc32c)`, `payload_tags` = letzte 16 B jedes Op-Rahmens (Crc-Modus: 4-B-CRC, mit Nullen auf 16 B) |

Op-Nutzlast (unversiegelt): `Put: key_len varint · key · header_len varint · header · body · weights[W]` ; `Delete: key_len · key` ; `RangeDelete: start_len · start · end_len · end` (end exklusiv). Bei `Sealed` ist **die gesamte Nutzlast** unter dem Clusterschlüssel versiegelt (Routing im `ops_head` bleibt klar). `Gate`-Body (klar): `scope u8 · ticket[16] · level u8 · policy_epoch_hull u64 (= P1) · n u16 · cluster[16]×n` bzw. `space_id u64` (Scope 2) bzw. `cluster[16] · space_id u64 · selector_sealed` (Scope 3/4; `selector` = `start/end` bzw. `branch`, versiegelt unter Clusterschlüssel bei `Sealed`, sonst Space-DEK). `Control`-Body: `Restrict: restriction_id u64 · reason u16 · owner[16] · effects u8 · expires i64 · cluster[16]`; `Lift: restriction_id u64 · cluster[16] · principal_sig_len u16 · sig`; `Link: derived[16] · n u8 · sources[16]×n · policy_epoch_after u64`; `Branch*`: Routing klar, Nutzdaten versiegelt (CYC-8 §4). `Reserve`-Frame: Füllrahmen ohne Ops, `stamp = 0`, wird bei Replay übersprungen (Ausrichtung und Reservesegment, `INV-LOG-7`).

`redo_mark` (Catalog-Op 0x0012): `(segment_id, offset, stamp, chain_h[32], clock_floor_utc)`; geschrieben alle `redo_mark_bytes` (8 MiB), bei Siegeln und mindestens alle 60 s bei Schreiblast (DEC-64). Replay beginnt am `replay_floor` (ein `redo_mark`-Wert) mit dessen `chain_h`.

#### 5.3.1 Torn-Tail und Mid-Log-Korruption (`INV-LOG-8`, normativ)

Seien `o` der Offset der ersten Gruppe, deren `GroupHead` oder deren Frames (CRC32C oder Ketten-MAC) ungültig sind, und `M` das Ende des letzten `redo_mark`-geschützten Bereichs.
1. Liegt `o < M` ⇒ `Corrupt{RedoBelowMark}` (alles bis zum letzten `redo_mark` MUSS gültig sein).
2. Sonst scanne jede `redo_align`-Grenze `p > o` bis Segmentende. Gilt für **irgendein** `p`: gültiger `GroupHead` (`gmagic`, `hcrc`), alle Frames CRC-gültig, `first_stamp > letzter gültiger Stamp` ⇒ `Corrupt{RedoMidLog}` (Daten nach der Lücke existieren; **nicht** kürzen).
3. Sonst: das Log endet bei `o`; der Rest wird nur im schreibenden Open mit Nullen überschrieben (Heilung), im `ReadOnly`-Open nie verändert.
Ehrliche Grenze: ein beschädigter, letzter quittierter Gruppenabschnitt hinter dem letzten `redo_mark`, dem nichts Gültiges folgt, ist von einem Torn Write nicht unterscheidbar.

### 5.4 Run

```
Run := Dateikopf · ClusterSection* · ClusterDirectory(Seiten) · PackMeta · ExtensionBlock* · Footer
ClusterSection := FullSection | MiniSection
FullSection := DataBlock* · SubIndexBlock · BranchTableBlock · FilterBlock · RangeDelBlock · ChangeIndexBlock
MiniSection := len u32 · section_uid[16] · SealFrame{ entries · range_dels · change_idx · branch_tab }   (Abschnitt ≤ [mini_section_max_bytes])
```
Alle versiegelten Blöcke tragen `len u32 · uid[16] · SealFrame`. **DataBlock** (Ziel `[block_target]`): `entries · restarts u32[] · n_restarts u32` (Restart alle 16 Einträge), dann versiegelt. Eintrag: `shared varint · unshared varint · key_suffix · tag u64 · branch varint · header_len varint · body_len varint (PutRef: Markerwert 0xFFFF_FFFF, dann blob_id u64 · record_uid[16] · len u32) · weights[W]·u32 · header · body`. Einträge sind innerhalb eines Abschnitts nach `(branch, key, tag↓)` sortiert; Einträge eines Branches bilden einen zusammenhängenden Blockbereich.

**SubIndexBlock** (clustersiegel, Page darf ihn entsiegeln, ohne Body zu lesen): je DataBlock `last_branch varint · last_key · offset u64 · len u32 · block_uid[16] · block_ordinal u32 · entries u32 · weight_sum[W]·u64 · min_stamp u64 · max_stamp u64`. Kumulative Präfixsummen werden beim Laden im RAM berechnet, nie persistiert. **BranchTableBlock**: je Branch `branch u32 · first_block u32 · last_block u32 · n_entries u32 · min/max_stamp · has_range_del u8`. **FilterBlock**: Bloom über `(branch, user_key)`, cluster-gesalzen (`salt = HMAC(file_key, cluster_id)[0..8]`), ca. 10 Bit/Schlüssel. **RangeDelBlock**: sortierte, disjunkte Intervalle je Branch mit Stamp. **ChangeIndexBlock**: nach `(stamp, op_uid)`: `(stamp, branch, op_uid, kind, key/range)` inkl. Punkt-, `RangeDelete`-, `EraseRange`-, Branch-State-Mutationen.

**ClusterDirectory** (Klartext-Seiten à 4096 B, nach `ClusterId` sortiert): `n u16 · entries[≤ 76] · page_mac[16] · page_crc u32`, Rest Nullen. Eintrag **53 B**: `cluster[16] · section_kind u8 · section_off u64 · section_len u32 · n_blocks u32 · n_entries u32 · min_stamp u64 · max_stamp u64`. `page_mac = HMAC(file_key, file_id ‖ page_no ‖ page_bytes)[0..16]` (bei `integrity=Crc`: Nullen und nur CRC). (R14 nannte „≈ 78 Einträge“: korrekt sind 76 mit MAC.) Keine Gewichte, keine Tokenprofile.

**PackMeta**: `pack_key` (Pure: `cluster`, Packed: `PackKey`, Staging: leer) · `n_clusters u32` · `page_firsts[n_pages]` (erste `ClusterId` je Seite, Quelle des RAM-Sparse-Index) · `ClusterFilter` (Bloom über `ClusterId`, ≈ 8 Bit/Cluster). **ExtensionBlock**: `tag u16 · version u16 · cluster[16] · SealFrame` je Cluster-Abschnitt. **Footer** (versiegelt als `kind=6`): `dir_off u64 · dir_len u64 · packmeta_off u64 · ext_off u64 · n_clusters u32 · file_len u64 · content_mac[16]` und danach `footer_tag[16]`. `content_mac = HMAC(file_key, Kopf ‖ alle SealFrame-Tags ‖ Directory-Seiten-MACs)[0..16]` wird beim Schreiben streaming berechnet, beim Öffnen **nicht** gelesen, aber vom Scrub (CYC-5 §7) vollständig geprüft. `footer_tag` wird beim Öffnen einer Datei geprüft und gegen den Catalog-Wert verglichen (DEC-59).

**Reader-Regel:** ein gelesener Block muss mit der im SubIndex stehenden `block_uid` entsiegeln; jede Abweichung ⇒ `Corrupt{Block}` und Block-Vergiftung (F-4).

### 5.5 Blob

`Blob := Dateikopf · ClusterSection*(Record*) · ClusterDirectory · Footer`, `Record := record_uid[16] · len u32 · SealFrame · crc32c u32`; AAD bindet `record_uid`, nicht `blob_id`/Offset. Ziel `≤ [blob_max_bytes]`. Fragmentierungsregel: ein Loch darf bei verfügbarem `punch_hole` dieselbe `blob_id` behalten; überschreitet `hole_ratio` den Wert `[blob_gc_ratio]` oder fehlt `punch_hole`, entsteht eine neue `blob_id` mit gepackten Überlebenden (`blob_replace`, atomar). **Purge DARF NICHT auf `punch_hole` angewiesen sein.** Eine Inkarnation wird nur entfernt, wenn kein lebender Run mehr einen `PutRef` darauf enthält (`INV-BLOB-1`).

### 5.6 Catalog

`CATALOG-<n>`: Dateikopf · Blöcke à 1 MiB (vorallokiert) · Records. Record: `rec_len u32 · rec_kind u8 (1 Snapshot, 2 Edit) · seq u64 · payload · crc32c u32 · chain_mac[16]`, `chain_mac = HMAC(catalog_key, prev_chain_h ‖ seq ‖ payload)[0..16]` (voller Zustand 32 B im RAM; erster Record trägt `prev_chain_h` aus dem Dateikopf-Anhang). Ein Edit-Payload: `n_ops u16 · (op_tag u16 · len varint · body)*`; gekoppelte Zustandsänderungen sind **ein** Edit (`INV-CAT-2`). Gruppen-Commit: wartende Edits ⇒ ein `write` + ein `sync_data`, Entlassung erst danach.

| Tag | Op | Felder (Auszug, kanonisch in dieser Reihenfolge) |
|---|---|---|
| 0x0001 | `space_add` | `space_id u64 · name_len u8 · name · options[64 B kanonisch] · created_epoch` (DEC-47/61) |
| 0x0002 | `space_state` | `space_id · state u8 (Live, Dropping, Dropped)` |
| 0x0003 | `space_remove` | `space_id` |
| 0x0004 | `cluster_register` | `cluster · protection u8 · tenant_tag[16] · due i64 (0=None) · erase_locality u8 · expected_bytes u64 · pack_group u32 · key_epoch u32 · registered_epoch` |
| 0x0005 | `cluster_state` | `cluster · state u8 · stamp S · gen u56` (Live, Retired, Shredded, Purged) |
| 0x0006 | `cluster_trim` | `cluster` |
| 0x0007 | `cluster_link` | `derived · n u8 · sources · stamp · policy_epoch_after` (Spiegel des Control-Frames) |
| 0x0008 | `restriction_set` | `cluster · restriction_id u64 · reason u16 · owner[16] · effects u8 · expires i64 · stamp` |
| 0x0009 | `restriction_lift` | `cluster · restriction_id · stamp` |
| 0x000A | `pack_assign` | `cluster · pack_group u32` (Rebucket) |
| 0x000B | `files_add` | `file_id u64 · kind u8 · level u8 · space_id · pack_ref · file_len u64 · footer_tag[16] · n_clusters u32 · min/max_stamp · floor` (DEC-59) |
| 0x000C | `files_remove` | `file_id` (legt im selben Edit den Graveyard-Eintrag an) |
| 0x000D | `unlink_ack` | `file_id` |
| 0x000E | `blob_replace` | `old_blob · new_blob · live_bytes` |
| 0x000F | `blob_live_delta` | `blob_id · live_delta i64` |
| 0x0010 | `replay_floor` | `redo_mark` |
| 0x0011 | `retain_floor` | `segment_id · offset` |
| 0x0012 | `redo_mark` | `segment_id · offset · stamp · chain_h[32] · clock_floor_utc` |
| 0x0013 | `segment_add` | `kind · id · owner_gen · class · key_scope_digest` |
| 0x0014 | `segment_remove` | `kind · id` |
| 0x0015 | `hold_set` | `hold_id · stamp · expires_mono_utc` |
| 0x0016 | `consumer_ack` | `consumer_id[16] · epoch u64` |
| 0x0017 | `erase_pending_add` | `ticket[16] · gate_stamp · deadline_ns · clock_floor · cluster_count · digest[16]` |
| 0x0018 | `erase_stage` | `ticket · stage u8` |
| 0x0019 | `erase_done` | `ticket` |
| 0x001A | `erased_ranges_add` | `space_id · cluster · erase_stamp · selector_sealed` |
| 0x001B | `erased_ranges_trim` | `…` |
| 0x001C | `key_epoch` | `cluster · epoch u32` |
| 0x001D | `kek_id` | `kek_id[16]` |
| 0x001E | `idem_trim` | `watermark_bucket u32` |
| 0x001F | `reserve_stamps` | `new_limit u64` |
| 0x0020 | `reserve_ids` | `kind u8 (file, tmp, branch) · new_limit u64` |
| 0x0021 | `reserve_space_ids` | `new_limit u64` |
| 0x0022 | `anchor_mark` | `catalog_epoch u64 · ledger_head[32] · erase_epoch u64` |
| 0x0023 | `restore_floor` | `erase_epoch u64` (Tomb-Retire, CYC-6 §7) |
| 0x0024 | `format_floor` | `file_kind u8 · min_ver u16` (Migration, §9) |
| 0x0025 | `hold_drop` | `hold_id` |

Reserviert `0xF000…0xFFFF`; unbekannte Tags ⇒ `Unsupported`. **Der Catalog enthält keine Branch-Namen und keine Nutzdaten.** `files_remove` und `unlink_ack` bilden den persistenten Graveyard. Snapshot-Payload: kanonisch sortierte Zustandsmengen (Spaces, `ClusterIndex`-Einträge à 36 B, Dateien, Restrictions, Links, Holds, offene Tickets, Floors, Reservierungslimits, `chain_h` des Ledger-Ankers). Rollover-Protokoll: CYC-5 §8.

**Catalog-Spiegel der Kontrollpfade (korrigiert R15 DEC-46).** `Restrict/Lift/Link` sind zuerst Control-Frames im Redo (stampgeordnet); ihr Zustand wird zusätzlich über die Ops `0x0007…0x0009` im Catalog gespiegelt, und zwar **bevor** `replay_floor` den Frame überschreiten darf (gleiche Regel wie `erase_pending_add` für Gates, `INV-LOG-5`). Branch-Ops erzeugen keine Catalog-Ops; ihr Zustand liegt in `SYS_BRANCH_*`-Records und Run-`BranchTableBlock`s.

### 5.7 Ledger

`ledger-<n>.log` (16 MiB vorallokiert): Dateikopf · `LedgerHead{prev_hash[32], first_seq u64}` · Records. Record: `seq u64 · prev_hash[32] · clock{utc i64 ns, mono u64} · kind u16 · len u32 · payload · mac[32]`, `mac = HMAC(ledger_key, prev_hash ‖ seq ‖ clock ‖ kind ‖ len ‖ payload)`; `prev_hash` des Folgesatzes = `mac`. Kopf der Kette = `mac` des letzten Records (Wert in `AnchorValue.ledger_head`).

`tomb_id = HMAC(tomb_key(tomb_epoch), "ctxs cluster tomb v1" ‖ instance_id[16] ‖ ClusterId[16])` — **32 B vollständig** (DEC-77); `space_tomb_id = HMAC(ledger_key, "ctxs space tomb v1" ‖ instance_id ‖ be_u64(SpaceId))`. Ledger enthält nie Klartext-ClusterId/SpaceId, Nutzerschlüssel, Nutzdaten oder Personen-IDs (`INV-LED-1`).

| Kind | Ereignis | Nutzlast (Auszug) |
|---|---|---|
| 1 | `EraseIntent` | `ticket[16] · request_id_hmac[32] · request_digest[32] · level u8 · kind u8 (Cluster,Space,Range,Branch,PackExpired) · origin u8 (Normal,Poisoned,Retention,Fallback) · t_requested · purge_deadline_ns u64 · tomb_epoch u32 · n u16 (≤ 4096) · tomb_id[32]×n (sortiert) · scope_ciphertext` (≤ 256 KiB; selektive Selektoren AEAD unter `LedgerScopeKey`, AAD `instance‖ticket‖digest`) |
| 2 | `EraseIntentExtend` | `ticket · n · tomb_id[32]×n` (Hüllenerweiterung, `INV-LED-3`) |
| 3 | `SpaceDropIntent` | `ticket · space_tomb_id · level · deadline` |
| 4 | `SpaceScopeFrozen` | `ticket · count · scope_digest` |
| 5 | `GateApplied` | `ticket · gate_stamp S · gate_utc` |
| 6 | `KeyDestroyIntent` | `ticket · tomb_epoch · n` |
| 7 | `KeysDestroyed` | `ticket · provider_attestation_digest · n` |
| 8 | `KeyLost` | `ticket/tomb_id-Menge` |
| 9 | `PurgeProgress` | `ticket · files_done · bytes` |
| 10 | `UnlinkProgress` | `ticket · acks` |
| 11 | `ExtAck` | `ticket · owner_kind · generation · attestation_digest` |
| 12 | `ExtBlocked` | `ticket · owner_kind · reason · since · retry_at` |
| 13 | `PurgeVerified` | `ticket · verifier_report_digest` |
| 14 | `Completed` | `ticket · status · residual_digest` |
| 15/16 | `RestrictionSet`/`RestrictionLifted` | `tomb_id · reason · effects` |
| 17 | `BranchEraseDone` | `ticket` |
| 18 | `SpaceDropped` | `ticket` |
| 19 | `RestoreReconciled` | `backup_mark · applied_intents` |
| 20 | `ReceiptIssued` | `ticket · receipt_digest` |
| 21–24 | `TombEpochOpened/Closed/RetireIntent/Retired` | `epoch u32 · …` |
| 25 | `LinkFence` | `ticket · hull_digest` |
| 26 | `FallbackMerged` | `from_seq · to_seq` (DEC-71) |
| 27/28 | `TicketBlocked`/`TicketResumed` | `ticket · reason` |
| 29 | `DeadlineBreached` | `ticket · t_over` |
| 30 | `Escalated` | `ticket · from_level · to_level · cause` |

### 5.8 `SYS_IDEM`, `SYS_BRANCH_*`, `head`, Directory

| Record | Schlüssel | Wert (kanonisch) |
|---|---|---|
| `SYS_IDEM` | `HMAC(idem_key, "ctxs idem key v1" ‖ scope ‖ token[16])` (32 B) | `schema u8=1 · status u8 (InFlight, Applied, Failed) · request_fp[32] (HMAC) · expires_bucket u32 (Stunden seit `created_utc`) · result_stamp u64 · scope_digest[16] (sortierte `tomb_id`s, gehasht) · result_kind u8 · result_len varint · result (≤ 256 B)` |
| `SYS_BRANCH_*` | `"br/" ‖ branch_id u32 (BE)` | Kopf: `schema u8 · state u8 (Live, Merged, Dropped) · parent u32 · fork_stamp u64 · ttl_ns u64 · created_by[16] · name_len u8 · name (≤ 64 B)`. Bei `Head` ist der MVCC-Stamp des Erstellungsrecords der `fork_stamp`; kein stampabhängiges Feld im versiegelten Payload. |
| `head` | `"doc/" ‖ doc_id[16] ‖ "/head"` | Kopf: `schema u8=1 · version u32 · chunk_count u32 · flags u8`. **Keine** Inhalts-Hashes (`INV-PRIV-1`). |
| Directory vorwärts | `dir_key` = `keyed_mac(DirectoryKey, tenant_slot ‖ canonical_person_id)` (32 B) | `entry_tag[16] · SealFrame_{entry_key(entry_tag)}(ClusterId[16])` (72 B) |
| Directory rückwärts | `rev_key` = `keyed_mac(ReverseKey, tenant_slot ‖ ClusterId)` (32 B) | `dir_key[32] · entry_tag[16]` unter System-DEK |

Schlüsselschema der oberen Schicht (verbindlich, CYC-7 §2): `doc/<doc_id[16]>/v<be_u32>/c<be_u32>` (Chunk), `…/head`, `emb/<model_id>` (Space), `fw/…`, `edge/…`, `sess/<id>/…`; Zahlen immer big-endian fixbreit, damit `cmp` numerisch ordnet. Zeichen `0x00` ist in Schlüsseln erlaubt.

### 5.9 Segmentdatei (Index-Partition) — nur der Container ist eingefroren (DEC-87)

```
Seg := Dateikopf(kind=SEGM) · PartManifest · PartBlock* · Footer
PartManifest := sealed{ partition_tag[16] · generation u32 · index_class u8 (Corpus, Pseudo) · owner_kind u16 · payload_schema u16 · applied_stamp u64 · instance_id · n_blocks u32 · rotation_seed[16] }
PartBlock := len u32 · block_uid[16] · SealFrame     (AAD kind=5, Schlüssel = partition_key(tag, generation))
Footer := n_blocks u32 · manifest_off u64 · file_tag[16]
```
Payload-Formate der Blöcke (HNSW-Kanten, Postings, CSR) sind je Crate in `contract` versioniert (`payload_schema`) und **nicht** eingefroren: eine Schemaänderung wird durch Neuaufbau der `Rebuildable`-Partition eingelöst, nicht durch Migration. Eingefroren sind Container, AAD, Schlüsselscope `(partition_tag, generation)` und Manifest.

### 5.10 Checkpoint-Manifest

`ckpt/<name>/MANIFEST`: Dateikopf(CKPT) · sealed{ `instance_id · created_utc · erase_epoch · tomb_epoch_open · catalog_epoch · open_tickets_digest · files[(file_id, kind, len, footer_tag)] · retired_not_purged_plain_clusters[…]` } · `mac` (Mac-Port, `file_key`). `BackupMark{instance_id, erase_epoch, created_utc, tomb_epoch}` ist ein Teilfeld und wird auch von externen Backups mitgeführt.

### 5.11 `ErasureReceipt` kanonische Bytes

TLV, Tags aufsteigend, Länge als `varint`, jeder Tag höchstens einmal, Listen sortiert nach (`ledger_seq`, Tag). `canonical_bytes := "CTXRCPT1" ‖ TLV*`. Tags: `1 instance_id · 2 ticket · 3 scope_digest · 4 level · 5 t_requested · 6 t_logical · 7 erase_epoch · 8 t_crypto? · 9 t_physical? · 10 t_verified? · 11 stages[(stage, ledger_seq, evidence_digest)] · 12 key_attestations · 13 provider{destroy, keys_in_backups, assurance} · 14 ext_acks · 15 verifier_report_digest · 16 ledger_head · 17 anchor{strength, value} · 18 restrictions_encountered · 19 status · 20 statuscaps · 21 disclosures · 22 residual · 23 time_trust{Verified, Uncertain}`. Zeiten sind `i64` UTC-Nanosekunden. Unbekannte Tags beim Verifizieren ⇒ `Unsupported`.

## 6. Freeze-Liste und Referenzvektoren (Gate G-F)

Eingefroren: Dateikopf, `FORMAT`, `KEYS`, Redo (Segment, Gruppe, Frame inkl. `policy_epoch`, Gate-Scopes, Control, Kette), Run (Sektionen, Directory, PackMeta, SubIndex, Footer), Siegel und AAD, Pfad-Sharding, Blob-Record, Catalog-Tags und Felder, `AnchorValue`, Ledger (Domain-Separation, `tomb_id`, Eventtags), `cmp` und Tag-Kodierung, Segment-Container (§5.9), `head`-Record, `SYS_IDEM`/`SYS_BRANCH_*`/Directory-Records, Receipt-Kanonik, `BackupMark`. Je Format ein unabhängiger Python-Referenzcodierer (`tools/reference/generator.py`) und goldene Vektoren, geprüft von `tools/reference/verify_goldens.py` und Rust-Test `golden`.

Vektorliste (Pflicht vor G-F): leeres Objekt · ein Eintrag · Eintrag mit versiegeltem Körper (Cluster- und Space-Quelle) · Cluster-Abschnitt mit zwei Blöcken · Packed-Run mit drei Clustern · Staging-Run · `MiniSection` · Abschnitt mit `BranchTable` (drei Branches, einer leer) · Tombstone · Bereichsmarker · Gate-Frame (alle vier Scopes) · Control-Frame `Link`/`Restrict`/`Lift` · Blob mit Loch · Catalog-Kette (Reservierung, Gruppen-Commit, `anchor_mark`, `files_add` mit `footer_tag`) · Ledger-Kette (zwei Tomb-Epochen, ein Retire, `EraseIntent` + `EraseIntentExtend`, `FallbackMerged`) · Redo-Gruppe mit Change/Gate/Control/Branch · abgeschnittener und beschädigter Frame · Mid-Log-Lücke · nicht-kanonisches `varint` · `PartBlock` und `PartManifest` · `head` · Directory vorwärts/rückwärts · `SYS_IDEM` · `SYS_BRANCH_S/_P` · Receipt-Kanonik · AAD je `kind` · **Rohkopie-Test** (Abschnitt aus Run A in Run B; Blöcke byte-identisch; Entsiegeln gelingt) · **Blocktausch-Test** (Block aus Run A in Run B vertauscht ⇒ `Corrupt`). `tools/reference/generator.py` und `verify_goldens.py` existieren im Repository und sind auf diese Liste auszubauen (R14 Anhang C „nicht vorhanden“ ist überholt, FIX-40).

## 7. Größenrechnung (geprüft)

| Größe | Wert | Rechnung |
|---|---:|---|
| Siegelrahmen (Encrypted) | 44 B | 4 + 24 + 16 (R14 R-PL-8 nannte 40 B: korrigiert) |
| Directory-Eintrag | 53 B | 16+1+8+4+4+4+8+8 |
| Einträge je Directory-Seite | 76 | (4096 − 2 − 16 − 4) / 53 |
| `ops_head` je Op | 53 B | 8+16+4+1+4+16+4 |
| Gate-Frame 4096 Cluster | ≈ 64 KiB + Kopf | 4096 × 16 B |
| `EraseIntent` 4096 `tomb_id` | ≈ 128 KiB | 4096 × 32 B (< 256 KiB) |
| Minimaler Overhead eines 1-KiB-Clusters (Mini) | ≈ 9,5 % | (44 + 53) / 1024 (H-11: ≤ 10 % für 1–4 KiB; unter 1 KiB nicht zugesagt) |
| `ClusterIndex`-Eintrag | 36 B (40 B mit Slack) | id 16, `state:8\|gen:56` 8, `pack_group` 4, `hot_ref` 4, flags 4 |

## 8. Platzierung (R-PL-1…8, unverändert in der Aussage)

Jede Datei ist **Pure** (ein Cluster), **Packed** (mehrere Cluster einer Packgruppe) oder **Staging** (nur L0, beliebige Cluster, nur Runs) (R-PL-1). Je Datei ein zusammenhängender Abschnitt je Cluster, nach `ClusterId` sortiert, abschnittsrelative Offsets (R-PL-2). Cluster ≥ `pure_threshold` wird bei der nächsten Compaction Pure (R-PL-3). Compaction mischt nur innerhalb einer Wartungsgruppe (R-PL-4). Ein Flush schreibt je Space eine Staging-Datei plus je großem Cluster eine Pure-Datei (R-PL-5). `rebucket` ist ein atomarer Umzug in einem Edit (R-PL-6). Spill- und `MiniSection`-Regeln: CYC-5 §3.

## 9. Migration und Versionierung (DEC-70)

1. Eine `format_ver` je Dateiart. `FORMAT.writer_ver` ist die höchste je geschriebene Version; `min_reader_ver` die niedrigste, die lesen darf.
2. Ein Build liest die Versionen `N` und `N−1` jeder Dateiart und schreibt nur `N`. Ältere Dateien werden durch normale Compaction/Flush lazy auf `N` umgeschrieben; Catalog-Op `format_floor` hebt den Boden, wenn keine Datei der Art unter `N` mehr lebt.
3. Ein Build verweigert (`Unsupported`) Dateien mit `format_ver > N` oder gesetzten unbekannten Pflichtbits. **Downgrade ist nicht unterstützt**; ein Rollback der Software ist nur über Restore eines Checkpoints vor dem Upgrade möglich.
4. Catalog-Snapshot und Ledger werden bei einer Versionsanhebung zuerst migriert (Rollover erzeugt Snapshot in `N`), Redo-Segmente sind kurzlebig (`redo_max_age`) und bekommen **keine** Rückwärtsleser: ein Upgrade verlangt vorher sauberes `close` mit leerem Replay (`replay_floor` = Ende), sonst verweigert der neue Build das Öffnen mit `MigrationNeedsCleanClose`.
5. Migration von `contextra-store` (NZ-7): kein Dateiformat-Leser; Forward-Export aus dem Altkern, Forward-Ingest in den Neukern (R5). Kein Dual-Write (Z-22).
6. Nach G-F ist jede Formatänderung ein Migrationseintrag mit: Version, Leser, Schreiber, Rewrite-Pfad, Rollback-Hinweis, Goldens für alt und neu.

## 10. Entscheidungen und geschlossene Lücken

DEC-47, DEC-57, DEC-58, DEC-59, DEC-61, DEC-70, DEC-77, DEC-81, DEC-87, DEC-88; GAP-01, GAP-02, GAP-04, GAP-05, GAP-06, GAP-07, GAP-08, GAP-15, GAP-16, GAP-17, GAP-18 (SYS-7 §3).

## 11. Abnahme

`golden` (alle Vektoren), `space_options_matrix` (S-1…S-10 je positiv/negativ), `aad_swap` (Blocktausch, Frame-Tausch zwischen Clustern, Epochentausch), `redo_torn_tail` (alle Fälle §5.3.1 inkl. Mid-Log), `catalog_chain_tamper`, `footer_tag_mismatch`, `canonical_varint_reject`, `reserved_bits_nonzero_reject`, `fuzz_decoder_*` (≥ 8 Ziele, SYS-6 §6), `miri` auf `codec` und Siegel.

**Offene Punkte:** keine.
