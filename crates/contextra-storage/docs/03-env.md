# SP-ENV — `env` + `sync` (Fs-Implementierungen, Simulation, Anker, Sperrränge)

| | |
|---|---|
| Modul | `contextra-storage/src/env/` (`memfs`, `realfs`, `faultfs`, `sim`, `anchor`, `governor`), `src/sync/` (`ranked`, `epoch`) |
| Besitzt | alle **Implementierungen** der Ports `Fs`, `Clock`, `Entropy`, `Anchor` (Mem/Datei); `FaultFs`; Sim-Scheduler; `Governor`; `sync::Ranked` |
| Konsumiert | SP-PORTS (Traits, `IoCtx`, `FsCaps`) |
| Spec-Bezug | R8 §3.1, §3.2, §3.4, §6.6 (AN-10), §11.2, §13.1, §14 P0; INV-LOCK-1, INV-DISK-1 (Fs-Teil), INV-ANCH-1 (Monotonie der Anker-Impl.) |
| Phase | P0 (MemFs, FaultFs, Ranked, Sim-Kern), P1 (RealFs, Anker), P5 (Governor), P8 (Windows) |

## 1 Auftrag und Abgrenzung
Jeder Byte-Zugriff und jede Zeitablesung des Kerns läuft über diese Schicht, damit `FaultFs` **jede** Operation sieht (K-11). Keine Kernlogik, keine Formate. **Darf nicht:** `std::fs`/`SystemTime`/`rand` außerhalb von `realfs`/Adaptern (Lint `disallowed_methods`); `mmap`/`io_uring`/`O_DIRECT` im Kern (nur als optionale `Fs`-Variante, nie Standard).
**Wiederverwendung Altstand (G-10):** `contextra-durable-fs` liefert geprüfte Plattform-Primitive (`platform.rs`: Datei-/Verzeichnis-Sync) und `scrub.rs`. Sie werden in `RealFs` **hinter** dem neuen `Fs`-Trait verwendet (Review auf Fehlerklassen F-1/F-3, kein Retry nach Fsync-Fehler). Dessen eigenes `Vfs`-Trait und `fault_injector` werden **nicht** übernommen: genau ein Fehlerinjektor (`FaultFs`).

## 2 Schnittstelle (zusätzlich zu den Port-Traits)
```rust
pub struct MemFs;            // deterministisch, Crash-Modell (siehe 3.1), Konformanz-Referenz
pub struct RealFs;           // Unix; Windows siehe E-22
pub struct FaultFs<F: Fs> { .. }
impl<F: Fs> FaultFs<F> {
    pub fn new(inner: F, plan: FaultPlan) -> Self;
    pub fn op_count(&self) -> u64;                      // Index der nächsten Operation
    pub fn crash_image(&self, at: CrashPoint) -> MemFs; // Zustand nach Absturz bei Op-Index i
    pub fn ghost_journal(&self) -> &GhostJournal;       // alle je geschriebenen Seiten (Orakel O-Device)
    pub fn bytes_read(&self, role: FileRole) -> u64;    // Byte-Zähler (INV-ERA-11)
}
pub enum Fault { TornWrite{keep:usize}, LostFsync, Eio, Enospc, RenameReorder, ShortRead, BitFlip{off:u64} }
pub struct FaultPlan { pub at_op: OpSel, pub fault: Fault }  // OpSel: Index | (Rolle, Zweck, n-tes Vorkommen)
pub struct Sim { .. }  // seeded Scheduler (shuttle-kompatibel), SimClock, SimEntropy, SimKms-Stub
pub struct Governor { .. } // IO-Budget je Lane, free_space-Beobachtung, Pressure → Rückdruck-Signal
pub mod sync { pub struct Ranked<T, const R: u8>; pub struct RankStack; pub struct Epoch; }
```

## 3 Regeln
1. **MemFs-Crash-Modell:** Schreibvorgänge landen in einem flüchtigen Puffer je Datei; `sync_data/all` macht sie durabel, `sync_dir` macht Namensoperationen (create/rename/unlink) durabel. Ein Absturz verwirft Nicht-Durables; nicht synchronisierte Seiten dürfen **beliebig teilweise** erhalten bleiben (Seitengranularität 512 B konfigurierbar, AN-02). `rename` ist atomar (AN-04), aber ohne `sync_dir` nicht dauerhaft.
2. **`preallocate` schreibt Nullen** (nie sparse, AN-10): `ENOSPC` darf ausschließlich hier auftreten. `append_prealloc` schreibt nur in vorallokierte Bereiche. `FaultFs` meldet Verstoß als Testfehler.
3. **FaultFs:** Jede Op trägt `IoCtx`; Indizierung nach globalem Op-Index (Crash-Sweep: für i in 0..N Abbild bauen, Recovery laufen lassen, Orakel prüfen) und nach `(Rolle, Zweck, n)`. `LostFsync` simuliert Fsync, das Erfolg meldet, aber nicht durabel ist — nur in einem separaten Testmodus (verletzt AN-01, prüft Erkennung, keine Garantie). Fsync-Fehler (`Eio`) werden vom Kern **nie** wiederholt (AN-03). Wiederholte Syncs desselben Handles sind zulässig, wenn seit dem vorigen Sync Daten/Metadaten geschrieben wurden; unmittelbar redundante Syncs ohne Mutation dürfen als Ineffizienz gemeldet, aber nicht als Korrektheitsverstoß klassifiziert werden.
4. **Ghost-Journal** hält jede je geschriebene Seite (auch überschriebene/entlinkte) für O-Device; es ist reine Testinfrastruktur (`testing`-Feature), nie in Produktion.
5. **RealFs:** Implementierung liegt in `contextra-storage-fs` und verwendet v1 ausschließlich sichere `rustix`-/Standardbibliothekswrapper; eigener `unsafe`-Code ist nicht zulässig. Exklusiver Prozesslock, private Modi 0600/0700, `sync_dir`, `free_space`, `read_at`/`read_ranges` und Rename-Semantik werden durch Fs-Konformanz geprüft. `punch_hole` und `reflink` sind optionale Fähigkeiten und nie Voraussetzung für Erase/Durability. Ein `Durable`-Store wird nur geöffnet, wenn `caps()` Datei-Sync, Verzeichnis-Sync, atomaren Rename und exklusiven Lock bestätigt; andernfalls nur ReadOnly oder rein Volatile. `acknowledged_weak_durability` darf keine Durable-Receipt erzeugen.
6. **Sim:** Ein Scheduler, der alle Threads des Kerns (über `Spawner`) deterministisch verschränkt; Seed reproduziert Lauf bit-genau. `SimClock` kann UTC rückwärts oder springen lassen, aber monotone Laufzeit läuft innerhalb eines Boots nie rückwärts; Reboot setzt nur die monotone Epoche neu. `SimEntropy` ist ein seeded CSPRNG (nur Tests; Produktions-`Entropy` = OS-CSPRNG, Prüfung auf Fork-Sicherheit).
7. **sync::Ranked:** Sperren tragen Rang 1…8 (R8 §11.2); `RankStack` (thread-local, nur `debug_assertions`) bricht bei Rangverletzung ab. Einzige Doppelhaltung: Rang 1+3 im Reservierungs-Fallback. Kein `std::sync::Mutex` außerhalb. `Epoch` = Generationszähler-Hilfen (`AtomicU64 state:8|gen:56`) für Cluster-/Cache-Generationen (SD-17).
8. **Anker:** `MemAnchor` (Test, `Strong`), `FileAnchor` (`Weak`, CAS über atomares Ersetzen + `sync_dir`, in separatem Verzeichnis außerhalb `<root>`). `advance(expect,next)` ist monoton und liefert bei Verlust des Wettlaufs `Err(Cas)`. Wert: `(erase_epoch, ledger_head, catalog_epoch)` kanonisch kodiert (SP-CODEC, Freeze G-F).
9. **Governor:** Token-Bucket je Lane (`maint_io_fraction` 0,5); `urgent` ist ungedrosselt. Liefert `DiskFull`-Hysterese-Signal (`free_space < erase_reserve_bytes`, Ausstieg bei 2×) an SP-COMMIT; kennt keine Kernlogik.

## 4 Fehlerverhalten
`Fs`-Fehler werden klassifiziert (F-1 `EINTR`/`EAGAIN` → Backoff; F-2 `ENOSPC` nur bei `preallocate`; F-3 `EIO`/Sync-Fehler → Handle vergiftet, nie wiederholt; sonst `Io{kind}` ohne Pfadinhalte jenseits der Dateirolle). Pfade in Fehlern nur als Rolle+ID, nicht als Nutzdaten.

## 5 Invarianten / Tests
`INV-LOCK-1` (loom + Rangstack-Test mit absichtlicher Verletzung), Fs-Konformanzsuite (identisch gegen MemFs, RealFs, FaultFs-Passthrough): Rename-Atomarität, `sync_dir`-Semantik, Prealloc-Nullen, Lock-Exklusivität, `list`-Ordnung unabhängig. Selbsttest `FaultFs`: bekannte Absturzpunkte reproduzieren; `crash_image(i)` für alle i eines Referenzworkloads (Atomar-Ersetzen) liefert nur „alt“ oder „neu“. Anker: Monotonie, CAS-Wettlauf, Persistenz über Neustart.

## 6 Lücken / Errata
- **E-21 (Zwei Fehlerinjektoren):** Altstand hat `durable-fs::fault_injector`; Beschluss: eine Quelle (`FaultFs`), Altstand-Injektor entfällt beim Cutover (SP-CUTOVER).
- **E-22 (Windows):** Windows `RealFs` ist v1-Release-Gate G-P, keine optionale Spike-Entscheidung. `FlushFileBuffers`, atomarer Replace/Rename, dauerhafte Verzeichnisänderungen und Locking müssen dieselbe Fs-Konformanz-/Crash-Suite wie Unix bestehen. Bis dahin lehnt `open(Durable)` auf Windows ab; weak durability darf keine Durable-Zusage ausgeben.
- **E-23 (`read_ranges`):** Beschlossen: verpflichtende Trait-Methode mit Default als geordnete Schleife über `read_at`; Adapter dürfen gebündelte Reads optimieren. Semantik und Fehlerreihenfolge bleiben identisch; die Methode ist Teil von G-0 und formatunabhängig.
- **E-24 (mmap für Segmente):** Obere Schichten (DiskANN) nutzen mmap. Erlaubt nur als `Fs`-Variante `MmapFs` für **Segmentdateien Plain/entschlüsselt-im-RAM**, nie für Runs/Redo; Sealed-Segmente werden blockweise entsiegelt (→ SP-EXT E-15).
- Entropie: Fork-Sicherheit (`getrandom` nach `fork` in Python-Bindings `contextra-py`) prüfen.

## 7 Arbeitspakete
| ID | Titel | PW | Abhängig von | Phase | Abnahme |
|---|---|---|---|---|---|
| WP-03.1 | `MemFs` inkl. Crash-Modell + Fs-Konformanzsuite | 2.0 | WP-01.2 | P0 | Suite grün, Crash-Modell dokumentiert |
| WP-03.2 | `FaultFs` (Op-Index, Torn/Eio/Enospc/Rename, Ghost-Journal, Byte-Zähler) | 3.0 | WP-03.1 | P0 | Selbsttests, Sweep-Treiber läuft |
| WP-03.3 | `RealFs` Unix (+ Wiederverwendung `durable-fs`-Primitive) | 2.5 | WP-03.1 | P1 | Konformanz auf ext4/xfs/tmpfs |
| WP-03.4 | `sync::Ranked`, `RankStack`, `Epoch`, loom-Gerüst | 1.5 | WP-01.1 | P0 | loom-Test Rangverletzung |
| WP-03.5 | `Sim` (Scheduler, SimClock, SimEntropy, SimKms-Stub) | 3.0 | WP-03.2 | P1 | Seed-Reproduzierbarkeit |
| WP-03.6 | `MemAnchor`, `FileAnchor` | 1.0 | WP-03.1 | P1 | Monotonie-/CAS-Tests |
| WP-03.7 | `Governor` (Lane-Budgets, DiskFull-Hysterese) | 1.0 | WP-03.3 | P5 | Budget-Tests |
| WP-03.8 | Windows-`RealFs`-Spike + Entscheidungsvorlage | 2.0 | WP-03.3 | P8 | Bericht, Go/No-Go |

## 8 Abnahme
Kein `std::fs`/`SystemTime`/`rand` im Kern (Lint grün); Konformanzsuite auf allen Fs-Varianten identisch; Crash-Sweep-Treiber nutzbar für SP-REDO ab P1.

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §3.2 (`HostProbe`), §9.7 (Geräte-IDs), §9.9; E-78, E-81.

- `RealFs`/Linux-Adapter implementieren `HostProbe`: Swap aktiv/verschlüsselt (soweit ermittelbar), `core_pattern`/`RLIMIT_CORE`, `RLIMIT_MEMLOCK`, Dateisystemtyp je Verzeichnis, `st_dev` von Daten- und Ledger-Verzeichnis. Nicht ermittelbare Werte werden `Unknown`, nie geraten. `MemFs`/`FaultFs` liefern konfigurierbare Reports für Tests (gleiche/getrennte Geräte, Swap an/aus).
- Der Kern ruft `probe` nur bei `open` (Diagnose, `LedgerSameDevice`-Prüfung in `OpenProfile::Production`) und beim Receipt-Bau auf; kein Pfad hängt für Korrektheit vom Ergebnis ab.

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-03.9 | `HostProbe` für Linux-`RealFs` und Test-Fs | 1.0 | WP-03.3, WP-01.5 | P5 | Unit-Tests mit `FaultFs`-Reports; `Unknown` bleibt `Unknown`; kein Probe-Aufruf unter Kernsperren (Lock-Rang-Test) |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

Der Notpool ersetzt jede Rang-1/Rang-3-Doppelhaltung. `Fs`-Konformanz und Open-Admission prüfen zusätzlich FD-/Dateigrenzen, vorallokierte Purge-Scratch-Slots sowie CoW-/Quota-Eignung, wenn eine harte DiskFull-Purge-Zusage konfiguriert wird. Recovery- und Race-Bezeichner sind `OP*` beziehungsweise `RACE-*`.
