# SP-REDO — `store::redo` (Redo-Log, Frames, Kette, Reserve, Torn-Tail)

| | |
|---|---|
| Modul | `contextra-storage/src/store/redo/` |
| Besitzt | Redo-Segmentdateien, Gruppen-/Frame-Kodierung (Gate-/Control-/Reserve-Bodies), Ketten-MAC-Anwendung, `redo_mark`, Reservesegment, Reader/Replay-Iterator, Torn-Tail-Regel |
| Konsumiert | SP-CODEC (Primitive, SealFrame, Kette), SP-ENV (`Fs`), SP-KEYS (Redo-Schlüssel `KDF(RootKey,"ctxs redo v1")`) |
| Spec-Bezug | R8 §5.5, §6.1 (Schritt d/e), §6.6, §10.1 (R6), §12.3 (Quelle des Feeds); INV-LOG-7, INV-LOG-8 |
| Phase | P1 (provisorisch), P2 (vollständig), G-F (Freeze) |

## 1 Auftrag und Abgrenzung
Das Redo ist die einzige Quelle der Atomarität über Spaces (SD-12) und der Durabilität (Durabilitätspunkt = ein `write` + ein `sync_data` je Gruppe). Es **entscheidet nicht**, was ein Frame bedeutet; Committer (SP-COMMIT) baut Frames, Recovery (SP-RECOVERY) wendet sie an. **Darf nicht:** Stamps vergeben, Schlüssel beschaffen (nur übergebene Handles), Cluster-/Gate-Zustand pflegen. Eigener Frame-Aufbau **nutzt** die Primitive aus SP-CODEC (keine zweite Siegelform).

## 2 Schnittstelle
```rust
pub struct RedoWriter { .. }   // genau ein Nutzer: Committer (Rang 1); Gate-/Control-Frames ebenfalls dort
impl RedoWriter {
    pub fn append_group(&mut self, frames: &[EncodedFrame], d: Durability) -> Result<GroupAck, RedoError>; // ein write + sync
    pub fn rotate_if_needed(&mut self) -> Result<(), RedoError>;      // nutzt vorallokiertes Folgesegment
    pub fn open_reserve(&self) -> ReserveHandle;                      // Gate/Control/Ledger-Frames bei DiskFull (INV-LOG-7)
}
pub struct RedoReader { .. }
impl RedoReader {
    pub fn scan(&self, from: RedoPos, keys: &KeyCache) -> impl Iterator<Item = Result<ScannedFrame, RedoError>>;
    pub fn heads_only(&self, from: RedoPos) -> impl Iterator<Item = Result<FrameHead, RedoError>>; // Durchlauf 1 der Recovery, ohne Entsiegeln
}
pub struct EncodedFrame { pub head: FrameHead, pub ops_sealed: Vec<Bytes>, pub stamp: Stamp }
pub enum FrameKind { Change, Gate, Control, Reserve }
```
`FrameHead` enthält `len, stamp, kind, n_ops, idem_token, epoch, flags, ops_head[]` (Format R8 §5.5, Freeze G-F), **ergänzt um** `caller: CallerMeta` (E-04, Origin 1 Byte + `caller_seq u64`) und `ttl_bucket` für `BranchFork` (E-13).

## 3 Regeln
1. **Segmente:** 64 MiB, vorallokiert mit Nullen (AN-10), `redo_align` 4096; Folgesegment wird **vor** Bedarf vorallokiert (Wartungsthread, `urgent` bei < 25 % Rest), damit `ENOSPC` nie im Commit-Pfad auftritt.
2. **Gruppe:** ≥ 1 Frames, Ketten-MAC pro Frame `H_i = MAC(H_{i-1} ‖ frame_without_chain_mac)` (provisorisches Layout, Freeze G-F). Gruppenkopf `len u32 | magic[4] | frame_bytes u32 | n_frames u16`, gefolgt von Frames und `crc32c`; vor jeder Gruppe Nullpadding bis `redo_align`. Eine Gruppe wird als **ein** `append_prealloc` geschrieben, danach folgt genau ein `sync_data` bei `Durable`. Sync-/harte Append-Fehler ⇒ Writer Poisoned und nimmt nichts mehr an.
3. **`redo_mark`:** alle 8 MiB und bei jedem Siegel schreibt der Writer eine Marke (als Control-Frame-Variante oder Catalog-Edit `redo_mark`, Entscheidung WP-05.2); alles bis zur letzten Marke muss gültig sein, sonst `Corrupt{RedoMidLog}`.
4. **Torn-Tail (INV-LOG-8):** erster ungültiger Gruppenkopf beendet das Log **nur**, wenn dahinter bis zum Segmentende nichts Gültiges folgt (Scan über Rest mit Kettenprüfung ab nächstem ausgerichteten Offset); sonst `Corrupt{RedoMidLog}`. Ehrliche Restgrenze: R8 §10.1 (beschädigter letzter Abschnitt hinter `redo_mark` ohne Folge ist von Torn Write nicht unterscheidbar).
5. **Frame-Kopf im Klartext, kettengeschützt**; `space_id` ist überall kanonisches big-endian `u64` (System-IDs 0…255 reserviert; User-IDs ab 256). Nutzlast je Op versiegelt (Sealed-Cluster unter `Clusterschlüssel(cluster,key_epoch)`, Plain-Cluster gemäß instanzweitem `IntegrityMode`/Space-DEK); RootKey-abgeleiteter Redo-Schlüssel authentifiziert nur die Kette und ersetzt nie den Clusterschlüssel für Sealed-Op-Nutzdaten. AAD `RedoOp{instance,space_id:u64,cluster,key_epoch,op_uid,op_index}` (SD-35/E-57). Gate-/Control-Bodies im Klartext, soweit ihre jeweilige Spezifikation sie nicht versiegelt; personenbezogene Branch-Nutzdaten bleiben im Sealed-Cluster-Schutzbereich.
6. **Zwei Durchläufe** für Recovery: `heads_only` sammelt Gate-/Control-Frames (retired_set), `scan` liefert Ops. Retired Cluster werden **nie** entsiegelt (R6).
7. **Barriere (ER4, nur Plain):** `cut()` schließt das aktuelle Segment (Siegel/Marke) und beginnt ein neues; Segmente vor dem Schnitt gehen in den Graveyard, sobald `replay_floor`/`retain_floor` dahinter stehen (Entscheidung in SP-ERASE, Mechanik hier; INV-LOG-9).
8. **Sichtbarkeit nach Fsync:** `GroupAck` enthält höchsten durablen Stamp; `durable_stamp` wird erst danach vom Committer angehoben (INV-CONS-1).
9. **Feed-Lesen:** Der Consumer-Feed (SP-EXT) liest über `RedoReader` mit gepinnten Segmenten; pinnen verhindert nur Löschung, ändert nie Dateien.

## 4 Fehlerverhalten
F-3 (Sync/Write EIO ⇒ Poisoned, alle Changes der Gruppe `Indeterminate`), F-2 (nur beim Vorallokieren), F-4/Corrupt (Kette/CRC/MAC außerhalb Torn-Tail-Regel), F-5 (`Unsupported` bei unbekannter Version/Kind/Bits). Fehler tragen Segment-ID und Gruppenoffset, nie Inhalte.

## 5 Invarianten / Tests
`INV-LOG-7`, `INV-LOG-8`; Crash-Sweep über jeden `FaultFs`-Index bei laufendem Commit-Workload: Ergebnis ist stets ein gültiger Präfix aus ganzen Gruppen; `proptest`: Frame-Rundreise, zufällige Beschädigung ⇒ nie stille Falschdaten; Torn-Tail-Tabelle (Fehler am Ende vs. mittendrin); Bit-Flip-Differenztest; Fuzz-Ziel `decode_redo_frame`; Goldene Vektoren (Change/Gate/Control/Branch, abgeschnitten, beschädigt) aus SP-VERIFY.

## 6 Lücken / Errata
- **E-04** `CallerMeta` im Frame-Kopf — vor G-F (siehe SP-PORTS).
- **E-13** `ttl_bucket` (Stunden, u32) als Klartextfeld des `BranchFork`-Control-Frames, damit die Wartung Branch-TTL ohne Cluster-Schlüssel terminieren kann (Metadatenleck: nur gröbste Ablaufzeit, dokumentiert in Restexposition §9.9).
- **E-27 (Gruppengröße):** `max_group_bytes` (Standard 4 MiB) begrenzt eine Gruppe, damit ein großer Change die Latenz anderer nicht sprengt; Changes > `max_change_bytes` werden vorher abgewiesen (INV-LIM-1).
- **E-28 (Redo-Wachstum bei Consumer-Lag):** `retain_floor` kann bis `max_lag_bytes` (1 GiB) Segmente pinnen — Plattenbudget in `erase_reserve_bytes`-Rechnung einbeziehen (SP-COMMIT DiskFull).

## 7 Arbeitspakete
| ID | Titel | PW | Abhängig von | Phase | Abnahme |
|---|---|---|---|---|---|
| WP-05.1 | Frame-/Gruppen-Codec + Kette (provisorisch, dann Freeze-fähig) | 2.5 | WP-02.1, WP-02.2 | P1 | Rundreise, Fuzz-Ziel |
| WP-05.2 | Segmentverwaltung (Vorallokation, Rotation, Marke, Reserve) | 2.5 | WP-03.2, WP-05.1 | P1 | Crash-Sweep, INV-LOG-7 |
| WP-05.3 | Reader: Torn-Tail-/Mid-Log-Regel, Zwei-Durchlauf-Iterator | 2.0 | WP-05.1 | P1 | Torn-Tail-Tabelle, Bit-Flip-Test |
| WP-05.4 | Gate-/Control-Bodies, Barriere-`cut`, Graveyard-Anbindung | 1.5 | WP-05.2 | P2 | O-Files nach ER4 (mit SP-ERASE) |

## 8 Abnahme
Crash-Sweep ohne Verstoß; kein `unwrap/expect/panic/indexing`; jeder Decoder mit Fuzz-Ziel; Goldene Vektoren nach G-F.

## R9.1-Synchronisierung

Der Syncer ist der einzige Redo-Schreiber und schreibt/synchronisiert Gruppen strikt in Stamp-Reihenfolge; der Sequencer führt keine E/A aus. Control-Frames (`Link`, Restrict/Lift, Gate und Branch-Ops) werden durch den Sequencer total geordnet. Sealed-Redo-Operations sind unter Cluster-Key-Epoche versiegelt, während der RootKey-Redo-Schlüssel ausschließlich die Ketten-MAC schützt.
