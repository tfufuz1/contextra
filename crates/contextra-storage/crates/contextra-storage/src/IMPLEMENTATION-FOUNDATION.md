# R8 Code-Fundament und Implementierergrenzen

Normative Quelle: `crates/contextra-storage/SPEC.md`; SP-Verträge: `crates/contextra-storage/*.md`. Dieses Dokument beschreibt, wie die Rust-Module zusammengebaut werden. Es autorisiert keine Abweichung von den SPs und behauptet nicht, dass ein als Scaffold aufgeführtes Modul implementiert sei.

## 1. Kernpfad und Vertrauensgrenzen

```text
öffentliche synchrone Fassade
  ├─ Config/MemoryBudget/Ports prüfen, bevor I/O beginnt
  ├─ RECOVERY: FORMAT → KEYS → CATALOG/Anchor/Ledger → Redo → Gate-Reconcile
  ├─ COMMIT: validate/admit → sequencer → durable Redo → publish MVCC roots
  └─ READ: immutable View → point/range/branch resolution → Gate/Restriction filter
       → key generation check → decode/header/body → result

Persistente Wahrheit: Redo + Catalog + Ledger + External Anchor
Abgeleitete Dateien: Runs + Blobs + Extension-Segmente (Watermark/Attestation)
```

Jede Komponente liest/ändert nur ihren eigenen Zustand über den eigenen Adapter. Durable Dateien sind die Recovery-Wahrheit; Caches, Memtables, Registry-Wurzeln und Observer sind abgeleiteter oder flüchtiger Zustand. Kein Cache darf eine fehlende durable Bestätigung ersetzen.

## 2. Besitz- und Modulplan

| Teilbaum/Dateien | Besitzer | Implementierer-Vertrag |
|---|---|---|
| `codec/*` | CODEC | reine bounded Funktionen; kanonische Bytes; keine I/O-/Zeit-/KeyProvider-Aufrufe |
| `catalog/*` | CATALOG | serialisierte Edits, Reserve-High-Water-Marks, CURRENT/Rollover, Graveyard; Edit wird vor Sichtbarkeit synchronisiert |
| `store/redo/*` | REDO | genau ein Writer; versiegelte Ops, durable Gruppen, poisoned nach Sync-/harter Append-Störung; Decoder akzeptiert nur vollständige Präfixe |
| `store/run/*` | RUN | immutable Dateien, Section-Directory, SubIndex/Filter, bounded Reader/FD-Pins; keine Nutzlast-Entsiegelung im Raw-Copy-Pfad |
| `store/blob/*` | BLOB | stabile `record_uid`-AAD; Inkarnation und GC nur nach Nullreferenzbeweis |
| `engine/memtable.rs`, `version.rs`, `registry.rs` | MEMVER | clusterpartitionierte Memtable; immutable MVCC roots; Zeroizing-Lifetime; getrennte visible/durable roots |
| `engine/committer.rs`, `guard_window.rs`, `idem.rs`, `volatile.rs` | COMMIT | ein sequenzierter Writer; Guard-Prüfung und Stamp-Reserve; durable append vor Publication; aktive Idem-Ergebnisse unabhängig vom replay floor |
| `engine/views.rs`, `cursor.rs`, `page.rs`, `cache.rs` | READ | Snapshot-Basis, Stale-/Gate-Fencing, token-budgetiertes Paging; nie Body-I/O für Skip-Entscheidungen |
| `erase/*` | CLUSTER/ERASE/LEDGER | monotone Lifecycle-Zustände, Restriction/Closure, Intent→Gate→Anchor→KeyDestroy→Purge→Receipt |
| `maint/*` | MAINT/PURGE | bounded Scheduler-Lanes, Claims, eigene Scratchreserve, pro-Datei PurgeEdit, Reader-Drain, UnlinkAck |
| `recovery/*`, `checkpoint/*` | RECOVERY/CKPT | keine Nutzer-View vor vollständigem Restore-Reconcile; gleiche InstanceId benötigt externes Writer-Fencing |
| `engine/{segments,extension,feed}.rs` | EXT | nur `durable_stamp` bestätigen; Gate-Filter vor Query-Ausgabe; fehlende Attestation verhindert systemweiten Erfolg |
| `env/*`, `sync/*` | ENV | Fail-injectable Ports, explizite I/O-Rollen, Lock-Rang und Epochen; keine implizite Plattformgarantie |
| `testing/*`, `formal/`, `fuzz/`, `tools/reference/` | VERIFY | unabhängige Modelle/Orakel/Decoderprüfer; Produktionscode darf nicht vom Testorakel abhängen |
| `contextra-storage-bridge` | BRIDGE | async Blocking-Pool und v15 API/DeletionProof-Abbildung außerhalb des synchronen Kerns |

## 3. Schreibpfadvertrag

Ein Commit implementiert diese Grenze, ohne sie zu verkürzen:

1. Eingabegrenzen, Space-/Clusterzustand, Protection, Restrictions, Guards und Idem-Ergebnis prüfen; Speicherbudget vor Allokation reservieren.
2. Alle benötigten IDs/Stamps über dauerhaft bestätigte Reservierungen beziehen. Space-IDs sind kanonisch `u64`; Systemzone 0…255 wird von öffentlicher API abgewiesen.
3. Sealed-Nutzlasten und clustergebundene Systemmetadaten vor Rang 1 versiegeln. Unter Rang 1 weder Provider- noch Observer-Rückruf.
4. Sequencer serialisiert Konfliktprüfung und finalen Stamp; ein dauerhafter Change wird als vollständige Redo-Gruppe geschrieben und synchronisiert.
5. Erst nach dem Redo-Ack Memtable/immutable Version-Root veröffentlichen. `durable_stamp` hebt sich bei Relaxed nie über durable Redo.
6. Bei Write-/Sync-Fehler Writer vergiften und Ergebnis nach Error-Vertrag als nicht angenommen/indeterminate behandeln; niemals denselben Writer fortsetzen.

Publication-Fehler nach einem dauerhaften Redo-Ack ist kein gewöhnlicher Commitfehler: Handle wird vergiftet, kein weiterer Commit wird angenommen, Recovery muss den durable Change wiederherstellen. Die öffentliche Receipt-Semantik muss diesen Zustand explizit als indeterminate abbilden und darf nicht „nicht geschrieben“ behaupten.

## 4. Read- und Löschpfadverträge

- `view()` kann Relaxed-Zustand sehen; `view_durable()` darf ausschließlich einen Root des durable Präfixes halten. `view_at(S)` prüft History-Floor und sichtbaren Head.
- Point lookup ist nur ein Kandidat. Range-Delete/EraseRange, Branch-Vererbung und Lifecycle-Masken werden vor Rückgabe aufgelöst.
- Gate-Generation wird vor Zugriff geprüft und vor Rückgabe erneut validiert. Das Gate ist sofort logisch wirksam, unabhängig von Hintergrund-Purge.
- Crypto-Erasure zählt erst nach durablem `KeyDestroyIntent` und Providerbeleg `probe == Absent`. `Disabled`, Timeout, `Unavailable` oder `Lost` entsprechen nicht `Absent`.
- Purge schreibt pro betroffener Datei ein `PurgeBatchEdit`; entlinkt erst nach Reader-Drain und `sync_dir`; `UnlinkAck` räumt den Graveyard. Nur der finale Edit nach allen Owner-Attesten darf `Purged/Done` setzen.
- `CompleteWithResidual` beschreibt abgeschlossene Arbeit mit explizit außerhalb des Vertrags liegender Restexposition. Fehlender erforderlicher Owner-/Scope-Beleg ist `Incomplete`, retrybarer sicherer Halt `Blocked`.

## 5. Implementierungsreihenfolge

1. **G-0:** Ports/Fehler/Config/Ressourcenbudget, Filesystem-Capabilities, Lock-Ränge, Zeit-/Entropy-Ports. `SpaceId u16` im Code ist ein Blocker, bis alle Ports/Codecs `u64` abbilden.
2. **Durable Basis:** Format/Siegel → Redo-Codec/Writer/Reader → Catalog/Reserve/Graveyard → Recovery/Open. FaultFs-Sweeps gehen mit jedem Writepfad mit.
3. **Write/Read:** Memtable/VersionRegistry → Sequencer/Guards/Idem → public Views → Run/Blob-Reader und Flush/Builder.
4. **Erasure:** Ledger + Anchor → Gate/Restrictions/Links → KeyProvider → Purge/Scratch/Checkpoint/Restore → Extension-Atteste/Receipt-Bridge.
5. **Produktintegration:** Branches, Volltext/Vektor/Graph-Owner, gemeinsame RSS-/Disk-Budgets, Migration, Soak und G-CUTOVER.

Keine höhere Schicht darf eine unfertige niedrigere Schicht durch direkte Dateizugriffe umgehen. Stub-Module bleiben explizit nicht-produktionsfähig; insbesondere dürfen leere Trait-Implementierungen oder erfolgreiche Dummy-Receipts nicht eingeführt werden.

## 6. Implementierer-Abnahme pro Modul

Ein Modul gilt erst als geliefert, wenn es (a) Besitzer-SP und API abbildet, (b) Grenzen vor jeder Allokation prüft, (c) Fehler ohne Klartext/Secrets meldet, (d) alle persistierenden Zustandsübergänge in FaultFs-Sweeps hat, (e) zuständige `INV-*`/A-IDs in Tests und Modellen abdeckt, (f) mit Referenzvektoren kompatibel ist und (g) Ressourcen-/Liveness-Grenzen dokumentiert. Ein Scaffold oder erfolgreicher Build schließt keine Abnahmebedingung.
