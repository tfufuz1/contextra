# SP-RUN — `store::run` (Run-Format, Builder, Reader, Rohkopie, Table-Cache)

| | |
|---|---|
| Modul | `contextra-storage/src/store/run/` |
| Besitzt | Run-Format (§5.6): Header, `ClusterSection` (DataBlock, SubIndexBlock, FilterBlock, RangeDelBlock), `ClusterDirectory`, `PackMeta`, `ExtensionBlock`-Slot, Footer; `RunBuilder`, `RunReader`, `copy_sections` (Rohkopie), Table-Cache |
| Konsumiert | SP-CODEC (varint, Tag, `cmp_entry`, SealFrame, Dateikopf), SP-ENV (`Fs`), SP-KEYS (`KeyCache`, nur Handles) |
| Spec-Bezug | R8 §4.4, §5.1, §5.2, §5.6, §7.3, §9.5.1; INV-PL-1…5, INV-SEAL-1, INV-FMT-1 |
| Phase | P2 (Format provisorisch → G-F), P4 (Filter/Skipping), P5 (Table-Cache) |

## 1 Auftrag und Abgrenzung
Unveränderliche, sortierte, clusterreine Dateien. Der Run **kennt Cluster, Branch, Tag, Gewichte**, aber **keine** Lebenszyklen (Catalog), Versionssichtbarkeit (SP-READ) oder Löschentscheidung (SP-PURGE). **Darf nicht:** Entsiegeln ohne übergebenen `Key32Handle`; Cluster-Schlüssel beschaffen; Dateien umbenennen/entfernen (Builder liefert `.building`-Datei; `rename`/Edit macht SP-MAINT).

## 2 Schnittstelle
```rust
pub struct RunBuilder<'a> { .. }
impl RunBuilder<'_> {
    pub fn begin_section(&mut self, c: ClusterId, seal: SealSource<'_>) -> Result<(), RunError>; // clusterrein, aufsteigend
    pub fn add(&mut self, e: &EntryRef<'_>) -> Result<(), RunError>;      // Reihenfolge cmp_entry erzwungen; Branch je Eintrag
    pub fn add_range_del(&mut self, m: &RangeMarker) -> Result<(), RunError>;
    pub fn end_section(&mut self) -> Result<SectionMeta, RunError>;
    pub fn finish(self, pack: PackMeta, ext: &[ExtBlock]) -> Result<BuiltRun, RunError>; // schreibt Directory/PackMeta/Footer, sync_all
}
pub struct RunReader { .. }                      // Verzeichnis im RAM (~48 B/Cluster), Daten per read_at
impl RunReader {
    pub fn directory(&self) -> &ClusterDirectory;
    pub fn get(&self, c: ClusterId, b: BranchId, key: &[u8], cap: Stamp, k: &KeyScope) -> Result<Option<EntryOwned>, RunError>; // Filter → SubIndex → 1 Block
    pub fn iter(&self, c: ClusterId, range: KeyRange, caps: &BranchCaps, k: &KeyScope) -> RunIter<'_>;  // Kopf ohne Körper
    pub fn block_skip_scan(&self, c: ClusterId, ..) -> impl Iterator<Item = BlockSummary>;              // weight_sum, ohne Entsiegeln
}
pub fn copy_sections(src:&RunReader, keep:&[ClusterId], dst:&mut RunBuilder<'_>) -> Result<(), RunError>; // Rohkopie: prüft nur CRC, nie AEAD
pub struct TableCache { .. }                     // FD-LRU, Pin/Refcount, max_open_files
```
`EntryRef { cluster, branch, user_key, tag, header, body_or_ref, weights }`; `PutRef` nur im Run, Körper in Blob (SP-BLOB).

## 3 Regeln
1. **Struktur:** exakt R8 §5.6; ein zusammenhängender Abschnitt je Cluster (INV-PL-2), sortiert nach ClusterId; Offsets **abschnittsrelativ**. Nur `ClusterDirectory`, `PackMeta`, `Footer` hängen von der Datei ab; alles andere ist positionsunabhängig ⇒ `copy_sections` kopiert Bytes unverändert.
2. **DataBlock:** Ziel 16 KiB; Präfixkompression mit Restart-Punkten alle 16 Einträge; Eintrag trägt `branch`; Block versiegelt als **ein** SealFrame (AAD `Block{space,cluster,block_uid}`; `block_uid` 128 Bit Zufall, nie aus Position). Rohkopie ⇒ `block_uid` bleibt.
3. **SubIndex:** je Block `last_key, off, len, entries, weight_sum[W], min/max_stamp, block_uid`; versiegelt unter Clusterschlüssel (INV-SEAL-1, SD-05). **Filter:** Ribbon (Standard) / Bloom über `user_key` ohne Branch; Salt im Block; versiegelt; false-positive-Ziel 1 %. **RangeDelBlock:** Marker `(cluster, branch, start, end, tag)`.
4. **Reader:** Punktabfrage = Directory (Binärsuche ClusterId) → Filter → SubIndex → **genau ein** Datenblock (H-2). Unbekannter Cluster in Datei ⇒ `None` (kein Fehler). Block-CRC/AEAD-Fehler ⇒ `Corrupt{Block}` (F-4: kleinste Einheit; Datei bleibt lesbar für andere Cluster). Entsiegeln nur mit `KeyScope`, der `gen` vor/nach prüft (SP-KEYS INV-KEY-2); der Reader ruft den Provider **nie**.
5. **Branch-Kappen:** `get/iter` nehmen `BranchCaps` (Liste `(branch, stamp_cap)` der Kette); Branch-Semantik selbst (SP-BRANCH) — der Reader filtert nur nach `(branch, tag ≤ cap)`. Bis P7 enthält die Liste genau `main`.
6. **Staging-Run (L0):** wie Packed, Abschnitte beliebig, **nur L0**; `PackMeta.kind = Staging`. Builder erzwingt: Pure ≤ 1 Cluster, Packed ⊆ Packgruppe (Prüfung gegen übergebene `PackKey`).
7. **Rohkopie (SP-PURGE nutzt):** `copy_sections` liest Abschnittsbytes, prüft Block-/Frame-CRC (Scrub-Effekt), kopiert 1:1 in neuen Builder, erzeugt neues Directory/Footer. Kein Schlüssel nötig, kein Entsiegeln (L15, H-10).
8. **Table-Cache (E-36):** Reader-Objekte (Verzeichnis, Filter-Index) im RAM, FD-Handles `max_open_files` per LRU; Pin während Iteratoren leben; Evict schließt FD, nie Verzeichnis. FD-Erschöpfung ⇒ Backoff (F-1), nie Fehler an den Lesenden.
9. **Prüfsumme/Format:** Dateikopf per SP-CODEC; Footer enthält `dir_off, dir_len, pack_off, ext_off, footer_crc, magic`. Metadaten-Hash `blake3` nur über Directory/Footer (INV-PRIV-1).
10. **ExtensionBlock:** opak je Abschnitt (Tag u16, Version u16); Builder reicht Blöcke durch; Inhalt/Rewrite = SP-EXT.
11. **Schreiben:** `.building.<tmp_id>` → `sync_all` → Aufrufer (`flush`) macht `rename` + `sync_dir` + Edit. Run-Schreiber nutzt `append_prealloc`; Vorallokation in 4-MiB-Schritten (`ENOSPC` nur dort).

## 4 Fehlerverhalten
F-4 (`Corrupt{Block|Index|Directory|Footer}` mit Datei-ID, Cluster-Rolle, Offset; nie Schlüssel/Inhalt); F-5 (`Unsupported` bei Version/Bits); F-6 durchgereicht aus `KeyScope` (`Shredded`/`KeyUnavailable`); F-2 nur Vorallokation.

## 5 Invarianten / Tests
INV-PL-1/2/3/5 (Builder verweigert Verstöße; Property), INV-SEAL-1, INV-FMT-1; proptest: Rundreise, `cmp`-Ordnung, Präfixkompression, Filter ohne False Negatives; Bit-Flip-Differenztest (nie stille Falschdaten); **`rawcopy_roundtrip`** (Abschnitt Run A → Run B, Blöcke byte-identisch, Entsiegeln gelingt); Block-Skipping ≡ Vollscan; Fuzz `decode_run_*` (Header, Directory, Block, SubIndex, Filter, Footer); goldene Vektoren (leer, ein Eintrag, versiegelt Cluster-/Space-Quelle, 2 Blöcke, Packed 3 Cluster, Staging, Tombstone, Marker); Byte-Zähler-Test: `copy_sections` liest/schreibt ∝ Abschnittsgröße.

## 6 Lücken / Errata
- **E-36** Table-Cache (INDEX).
- **E-48 (Filterwahl):** Ribbon spart Platz, kostet Bau-CPU; Standard **Bloom** in P2, Ribbon als Feature nach Messung H-2/H-5 (Format trägt `filter_kind u8`, vor G-F festlegen).
- **E-49 (Block-Größenvarianz):** Körper < `value_separation_min` bleiben inline; Block darf bis 2× Ziel wachsen, wenn ein Eintrag es erzwingt (Fortschrittsgarantie `oversize_single`).
- **E-23** `read_ranges` (SP-ENV) wird vom `block_skip_scan`/Purge-Pfad genutzt, falls entschieden.

## 7 Arbeitspakete
| ID | Titel | PW | Abhängig von | Phase | Abnahme |
|---|---|---|---|---|---|
| WP-07.1 | DataBlock + Präfixkompression + Siegel | 2.0 | WP-02.2 | P2 | Rundreise, Bit-Flip |
| WP-07.2 | SubIndex, Filter (Bloom), RangeDel, Directory/PackMeta/Footer | 2.5 | WP-07.1 | P2 | Fuzz, Filter-Property |
| WP-07.3 | `RunBuilder` (Sektionen, Prealloc, Staging/Pure/Packed-Regeln) | 2.0 | WP-07.2, WP-03.2 | P2 | INV-PL, Crash-Sweep Bau |
| WP-07.4 | `RunReader` (get/iter), `copy_sections`, Rohkopie-Test | 2.5 | WP-07.2 | P2 | `rawcopy_roundtrip`, Byte-Zähler |
| WP-07.5 | `block_skip_scan`, `TableCache`, Ribbon-Option | 2.0 | WP-07.4 | P4 | Skipping ≡ Vollscan, FD-Test |

## 8 Abnahme
Keine `unwrap/expect/panic/indexing`; jeder Decoder Fuzz-Ziel; `rawcopy_roundtrip` und goldene Vektoren grün (G-F); H-2/H-3/H-4 messbar.
