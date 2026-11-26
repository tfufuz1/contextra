# Versionierter Speicher für Agenten und LLMs: Architektur (Teil II)

| | |
|---|---|
| Status | Entwurf zur Umsetzung, Entscheidungen final (V-xx), Annahmen markiert (AV-xx) |
| Baut auf | [`REFACTORING_PLAN.md`](REFACTORING_PLAN.md) (Teil I, Kernel-Refaktorierung P0–P9) |
| Zielgruppe | Implementierer und Reviewer. Teil I liefert den Kernel, Teil II die Schichten darüber und die Vorleistungen im Kernel (Abschnitt 5). |
| Verifikation | Reiner Entwurf. Leistungsaussagen sind **Hypothesen** (H-xx), die der Prototyp V0 widerlegen darf. |

---

## 0. Kurzfassung

Gewünscht ist ein Fundament mit: atomaren Änderungen, Versionskontrolle ähnlich Git (Commits, Branches, Merge, Diff),
feingranularer atomarer Zeitreise, SQLite-artiger Nutzbarkeit für Agenten/LLMs und Basis für KI-Daten
(Blobs, Embeddings, Volltext).

Zentrale Entscheidungen:

1. **Kein „MVCC mit unendlicher Retention“, sondern persistente, inhaltsadressierte Prolly-Bäume** (V-01).
   Ein Branch ist ein Zeiger, Diff kostet O(Δ·log n), Gleichheit ist ein Hash-Vergleich, Merge arbeitet auf Schlüsseln/Zellen.
2. **Der Kernel (`contextra-store`) bleibt generisch** und kennt weder Commits noch Tabellen. Er liefert
   atomare Multi-Key-Batches **mit Vorbedingungen**, Snapshots, Streaming-Iteratoren und Pins. Darauf liegen
   `contextra-cas` (Objekte/Packs), `contextra-vcs` (Commits, Refs, Merge, Zeitreise), `contextra-table`
   (Schema, Zeilen, Indizes) und Frontends (SQLite-vtab, MCP, CLI, FFI).
3. **Zwei Ebenen von „Änderung“:** *Transaktion* (atomare Einheit, SQL `COMMIT`) und *Commit* (benanntes Versionsereignis).
   Ein Commit enthält ein Journal seiner Transaktionen. Daraus ergibt sich **Zeitreise bis auf einzelne Transaktion
   und Operation**, ohne pro Transaktion einen Baum neu zu schreiben (V-04).
4. **SQL = austauschbares Frontend.** Erstes Frontend ist SQLite über Virtual Tables (der Dialekt, den LLMs kennen),
   nicht SQLite-Seiten-VFS, weil Seiten-Versionierung Diff und Merge zerstört (V-08).
5. **Agenten-Modell: Sitzung = Branch-Arbeitskopie** mit dauerhaftem Overlay, Idempotenzschlüsseln, Quoten und
   Schutzregeln (V-06, V-07).
6. **Datenschutz ist Architektur, nicht Nachtrag:** Schlüsselhierarchie, Crypto-Shredding je Subjekt und
   administratives History-Rewrite (V-11), weil unveränderliche Historie und Löschpflicht kollidieren.
7. **Ein Repository = eine Kernel-Instanz** (Isolation, Schlüssel, Löschbarkeit). Eine Instanz ist **Single-Process**
   (anders als SQLite); Mehrprozess-Zugriff läuft über einen Daemon/MCP-Server (V-13).

---

## 1. Anforderungen und Nicht-Ziele

| ID | Anforderung |
|---|---|
| R-01 | Atomare Änderungen über mehrere Schlüssel, Tabellen und Refs hinweg; Crash-sicher. |
| R-02 | Commits (DAG), Branches, Tags, Merge, Diff, Log, Blame, Revert, Cherry-pick. |
| R-03 | Zeitreise: lesen **und** verzweigen an beliebigem Commit, Zeitpunkt, Transaktion und Operation. |
| R-04 | Fein granular: Diff und Blame auf Zeilen- und Spaltenebene (Zelle). |
| R-05 | SQLite-artig: eingebettet, ohne Konfiguration, ACID, SQL-Dialekt, den LLMs sicher schreiben. |
| R-06 | Agenten-tauglich: parallele Arbeitskopien, Provenienz, Idempotenz bei Retries, strukturierte Fehler, Quoten, Schutzregeln. |
| R-07 | KI-Daten: große Blobs, Vektoren, Volltext, JSON; abgeleitete Indizes ohne die Versionierung zu brechen. |
| R-08 | Sauber entkoppelt: jede Schicht allein testbar, austauschbar, mit eigener Formatversion. |
| R-09 | Mehrmandantenfähig, verschlüsselbar, löschbar (inkl. Historie), prüfbar (Audit). |
| R-10 | Betreibbar: GC, Retention, Backup/Restore, `verify`, Metriken, Upgrade. |
| R-11 | Deterministisch testbar (Uhr, Zufall, I/O über Ports). |

**Nicht-Ziele (bewusst):** verteilter Konsens (nur Einzelknoten; Replikation später über Objekt-Austausch),
Git-Wire-Protokoll-Kompatibilität (ein Export/Import-Pfad ist möglich), eigene SQL-Engine im Kern,
Einzeldatei-Format (Verzeichnis wegen Crash-Sicherheit), SQLite-Seitenkompatibilität auf Dateiebene,
Trigger/Stored Procedures in v1.

---

## 2. Vokabular

| Begriff | Bedeutung |
|---|---|
| **Seq / LSN** | Kernel-Sequenznummer eines Batches. Physisch, linear, pro Kernel-Instanz. |
| **Transaktion (Tx)** | Atomare Menge von Operationen. Kleinste Einheit der Zeitreise. |
| **Objekt** | Unveränderliche, inhaltsadressierte Bytes (`ObjectId`): Blob, Baumknoten, Commit, Tag. |
| **Prolly-Baum (`ProllyMap`)** | Geordnete Map `Bytes → Bytes` als inhaltsadressierter Baum mit inhaltsabhängigen Chunk-Grenzen. |
| **Wurzelkarte (RootMap)** | Kleine Prolly-Map `Name → Wurzel-ID` (Tabellen, Schemas, Metadaten). Der Commit zeigt auf sie. |
| **Commit** | Objekt: RootMap-ID, Eltern, Autor/Prinzipal, Zeit, Nachricht, Metadaten, Journal-ID, optional Signatur. |
| **Ref** | Benannter, veränderlicher Zeiger (`heads/`, `tags/`, `sessions/`) mit Generationszähler. Liegt im Kernel. |
| **Sitzung (Session)** | Arbeitskopie eines Agenten: Basis-Commit + dauerhaftes Overlay. |
| **Keyspace** | Namensraum im Kernel (Präfix `KeyspaceId`), nur Tag, keine Domänenbedeutung. |
| **Pack** | Append-only Datei mit Objekten (Quelle der Wahrheit für Objektexistenz). |
| **Abgeleiteter Index** | Aus einem Baumzustand berechnetes, neu aufbaubares Artefakt (FTS, ANN, Aggregate). |

---

## 3. Entscheidung: Grundlage der Versionierung

| Option | Idee | Vorteile | Nachteile | Urteil |
|---|---|---|---|---|
| **A** MVCC-Retention im Kernel | Alle Versionen je Key behalten, `AS OF seq` | Existiert teilweise (`get_at_seq`, `pin_checkpoint`), keine Schreibverstärkung | Nur **lineare** Historie. Branch/Merge nicht ausdrückbar (Sichtbarkeit je Branch wäre ein Vorfahrentest je Version). Kein O(Δ)-Diff, keine Hash-Gleichheit, Platz wächst mit Update-Rate, Retention verteuert Compaction | Als Hauptmechanismus **abgelehnt** |
| **B** Prolly-/Merkle-Bäume, inhaltsadressiert | Zustand = Wurzel-Hash; Commits bilden DAG | Branch = Zeiger; Diff O(Δ·log n); Merge; Integrität; Deduplikation; strukturelle Teilung; Zustand historienunabhängig | Schreibpfad O(Δ·log n) Knoten; Lesepfad Baumtraversal (Cache); GC über Erreichbarkeit | **Gewählt** (V-01) |
| **C** Hybrid: aktueller Stand im Kernel materialisiert + B für Historie | Schnelle Reads am Head | Schnellster Lesepfad | Zwei Repräsentationen müssen atomar konsistent bleiben; Branch-Wechsel teuer | Rückfalloption, falls V0 die Leseleistung von B verfehlt |
| **D** SQLite-Seiten in versioniertem Speicher | Volle SQLite-Kompatibilität | Kein neuer Query-Stack | Seiten-Diffs sind layout- und historienabhängig: Merge und feingranularer Diff unmöglich | Abgelehnt |

Kernel-`get_at_seq` bleibt nützlich für **nicht versionierte** Keyspaces und für konsistente Lesevorgänge über
Refs + Indexstrukturen zum selben Seq. Es trägt aber nicht die Versionshistorie.

---

## 4. Schichtenmodell und Crate-Schnitt

```
 Frontends  contextra-sqlite (vtab, FFI)  contextra-mcp   ctx (CLI)   contextra-ffi (C/Python/Node)
                 │ nur über Query-/Agent-Ports
 ┌───────────────▼───────────────────────────────────────────────────────────┐
 │ contextra-agent   Sitzungs-API, Tool-Fassade, Fehlerkatalog, Quoten        │
 │ contextra-query   QueryPort-Traits (Scan mit Pushdown, Mutation, AS OF)   │
 │ contextra-index-* Abgeleitete Indizes (fts, vector, graph, aggregate)     │
 │ contextra-table   Schema, Zeilenformat, Sekundärindizes, Constraints      │
 │ contextra-vcs     Objektmodell, ProllyMap, Commits, Refs, Merge, Zeitreise,│
 │                   Sitzungen, GC-Orchestrierung, Hooks, Zugriffsregeln      │
 │ contextra-cas     Objektspeicher: Packs, Index, Kompression, Verschl., GC │
 │ contextra-store   KERNEL (Teil I): Batches mit Vorbedingungen, Snapshots, │
 │                   Iteratoren, Keyspaces, Pins, Vfs/Clock/Entropy/Crypto-Ports│
 └───────────────────────────────────────────────────────────────────────────┘
```

Regeln:

- Abhängigkeiten zeigen **nur nach unten**. Kein Crate kennt ein höheres.
- **`unsafe`/FFI nur in** `contextra-sqlite` und `contextra-ffi`. Der Kernel bleibt `forbid(unsafe_code)`.
- Jede Schicht hat eine eigene Formatkennung und Golden-Tests (Abschnitt 14).
- Jede Schicht hängt an **Traits** der Schicht darunter (Abschnitt 4.1), nicht an deren Strukturen. Dadurch sind
  Fakes (In-Memory-Kernel, `MemVfs`, `ManualClock`) in allen Schichten möglich.
- Domänenbegriffe der Plattform (Tenant, Collection, Embedding-Modell, Lizenz) leben **über** `contextra-table`.

### 4.1 Seam-Traits (Vorgabe)

```rust
// Kernel-Fassade, wie contextra-cas/vcs sie nutzen (Teil I liefert die Implementierung)
pub trait KvStore: Send + Sync {
    type Snap: KvSnapshot;
    fn snapshot(&self) -> Self::Snap;                                   // konsistent über alle Keyspaces
    fn apply(&self, batch: WriteBatch) -> Result<CommitInfo, ApplyError>; // atomar, inkl. Vorbedingungen
    fn pin(&self, seq: Seq, why: PinReason) -> PinGuard;               // Retention
    fn subscribe(&self, l: Arc<dyn CommitListener>);                    // nach Durabilität, in Seq-Reihenfolge
}
pub struct WriteBatch { pub ops: Vec<Op>, pub preconditions: Vec<Precondition> }
pub enum Precondition { Absent(Key), Equals(Key, Bytes), UnchangedSince(Key, Seq) }
pub trait KvSnapshot { fn get(&self, ks: Keyspace, k: &[u8]) -> Result<Option<Bytes>>;
                       fn iter(&self, ks: Keyspace, r: KeyRange, dir: Dir) -> Box<dyn KvIter>; }

// Objektspeicher (contextra-cas)
pub trait ObjectStore: Send + Sync {
    fn put(&self, kind: ObjKind, bytes: &[u8]) -> Result<ObjectId>;     // idempotent (Dedup)
    fn get(&self, id: &ObjectId) -> Result<Bytes>;
    fn has(&self, id: &ObjectId) -> Result<bool>;
    fn flush_durable(&self) -> Result<DurableMark>;                      // fsync der offenen Packs
}

// Baum-Engine (contextra-vcs)
pub trait TreeEngine {
    fn get(&self, root: &ObjectId, k: &[u8]) -> Result<Option<Bytes>>;
    fn range(&self, root: &ObjectId, r: KeyRange, dir: Dir) -> Box<dyn Iterator<Item = Result<(Bytes, Bytes)>>>;
    fn apply(&self, root: &ObjectId, muts: &mut dyn Iterator<Item = Mutation>) -> Result<ObjectId>;
    fn diff(&self, a: &ObjectId, b: &ObjectId, r: KeyRange) -> Box<dyn Iterator<Item = Result<DiffEntry>>>;
    fn merge3(&self, base: &ObjectId, ours: &ObjectId, theirs: &ObjectId, res: &dyn MergeResolver) -> Result<MergeOutcome>;
}

// Erweiterungspunkte
pub trait MergeResolver { fn resolve(&self, c: &Conflict) -> Resolution; }
pub trait CommitHook { fn pre_commit(&self, c: &CommitDraft) -> Result<(), HookRejection>; fn post_commit(&self, c: &Commit); }
pub trait AccessPolicy { fn check(&self, p: &Principal, a: &Action, target: &Target) -> Result<(), Denied>; }
pub trait DerivedIndex { fn id(&self) -> &str; fn update(&self, prev: Option<&ObjectId>, next: &ObjectId, diff: &mut dyn DiffStream) -> Result<()>; }
```

---

## 5. Vorleistungen im Kernel (ändern Teil I)

Diese Punkte müssen **in** der Refaktorierung des Kernels entstehen, sonst erzwingt Teil II später Umbauten.
Einarbeitung in die Phasen von Teil I siehe dort, Abschnitt 12.

| ID | Vorleistung | Begründung / Befund |
|---|---|---|
| K-1 | **Keyspaces** als typisierte Präfix-Tags (`KeyspaceId(u16)`, Registry im Kernel, Namen vergibt die obere Schicht). Kein Formatwechsel. | Heutige `RESERVED_PREFIXES` sind ein ungeplanter Vorläufer. Atomarität über Keyspaces ist damit gratis (ein WAL, eine Seq). |
| K-2 | **`WriteBatch` mit Vorbedingungen**, im Committer atomar geprüft. | Ref-Updates sind Compare-and-Swap. Heute existiert nur `put_if_absent` je Key. Im Single-Writer-Modell (Teil I, D-05) ist das billig. |
| K-3 | **Streaming-Iteratoren** (Grenzen, Richtung, Snapshot) statt `Vec`-Scans. | `scan*` liefert heute vollständig materialisierte `Vec<(Vec<u8>,Vec<u8>)>`. Für Prolly-Traversal und große Tabellen untragbar. |
| K-4 | **Stabile `CommitListener`-API** mit Seq, nach Durabilität, in Reihenfolge. Kein dauerhafter Kernel-Change-Feed. | Abgeleitete Indizes folgen **Commit-Diffs** der VCS-Schicht, nicht dem WAL (das nach Flush gelöscht wird). |
| K-5 | **Retention/Pins** als explizite Policy: `floor = min(Snapshots, Pins)`. Bestehendes `pin_checkpoint(seq)` bleibt kompatibel. | Heute nur über Snapshot-Registry und `pin_checkpoint`. Pins brauchen Grund und Ablauf (Zombie-Pins vermeiden). |
| K-6 | **`JobRunner`-Port** und **Committer als Single-Flight-Job** statt eigener Thread je Instanz; **`ResourceGovernor`** (Cache-, Memtable-, I/O-Budget) über viele Instanzen teilbar. | Korrigiert Teil I D-03/4.1: Tausende Repos mit je 3+ Threads skalieren nicht. Single-Flight (höchstens ein aktiver Committer-Job je Instanz, startet bei nicht leerer Queue) vermeidet den früheren Leader/Follower-Handoff. |
| K-7 | **`STORE_FORMAT`-Deskriptor** (eigene kleine Datei, `required`/`optional` Feature-Bits). Neue Datei, **kein** MANIFEST-Formatwechsel. | Ältere Software muss unbekannte Pflichtfeatures ablehnen. Analog `REPO_FORMAT` in der VCS-Schicht. |
| K-8 | **Ports und Krypto öffentlich und semver-stabil** (`Vfs`, `Clock`, `EntropySource`, `Cipher`, `KeyHierarchy`). | `contextra-cas` nutzt dieselben Ports (Packs, `fsync`-Fehlerpolitik INV-P1, Fault-Injection). Kein zweites VFS. |
| K-9 | **Wertgrößenpolitik:** Kernel lehnt Werte über `max_value_bytes` ab (Default 1 MiB). Große Daten gehören in `contextra-cas`. | Verhindert Write-Amplification großer Blobs im LSM. |
| K-10 | **Instanz je Repository**, Single-Process (`DirLock`). | Isolation, Schlüssel, Löschbarkeit (V-13). |
| K-11 | `rollback_to_tx` und der `__session_dag`-Bereich des Altsystems werden gegen VCS-`reset`/Commit-DAG **abgeglichen** (Teil I A-07). | Überlappende Konzepte vermeiden, Migration planen (V9/Abschnitt 13). |

---

## 6. Objektmodell

### 6.1 ObjectId

`ObjectId = { algo: u8, digest: [u8; 32] }`, Standard `algo = 1` (BLAKE3-256) über `kind_tag ‖ len ‖ canonical_bytes`.

- **Hash-Agilität:** das `algo`-Byte ist Teil jeder ID. Ein späterer Wechsel erfordert keine Umkodierung der Struktur.
- **Verschlüsselte Repos (V-05):** ID = **keyed** BLAKE3 mit einem Repo-`id_key`. Ohne Schlüssel ist weder Existenz noch Gleichheit
  von Inhalten prüfbar (kein Dedup-Orakel). Preis: IDs sind nur innerhalb derselben Schlüsseldomäne portabel;
  Austausch zwischen Domänen erfordert Export mit Neu-Hashing.
- **Kanonische Kodierung (spezifiziert, versioniert, Golden-Tests):** Längenpräfixe als Varint, Maps nach Schlüsselbytes sortiert,
  Floats als total geordnete Bytes (`-0.0` und `NaN` kanonisiert), Text als rohe UTF-8-Bytes ohne Normalisierung,
  keine Zeiger auf Plattformdetails.

### 6.2 Objektarten

| Art | Inhalt |
|---|---|
| `Blob` | Rohbytes bis `max_inline_blob` (z. B. 256 KiB). Größere Werte als `BlobTree` (FastCDC-Chunks, ID = Wurzel). |
| `Node` | Prolly-Knoten (Blatt oder innerer Knoten). |
| `Commit` | Siehe 6.4. |
| `Tag` | Annotierter Zeiger auf Commit (Name, Prinzipal, Nachricht, Signatur). |
| `Journal` | Geordnete Transaktionsaufzeichnung eines Commits (Abschnitt 9). |
| `RepoConfig` | Eingefrorene Parameter (Chunking, Hash, Kodierung). Änderung = neues Baumformat. |

### 6.3 Prolly-Baum

- **Blatt:** sortierte `(key, value)`-Einträge. **Innerer Knoten:** `(max_key, child_id, subtree_count, subtree_bytes)`.
  `subtree_count` ermöglicht Rang, `OFFSET/LIMIT` und exaktes `COUNT(*)` in O(log n).
- **Chunk-Grenzen** hängen deterministisch vom Inhalt der laufenden Eintragsfolge ab (gewichtete Wahrscheinlichkeit
  mit Ziel-, Mindest- und Maximalgröße). Folge: **Historienunabhängigkeit** (gleiche Daten → gleiche Wurzel, egal in welcher
  Einfügereihenfolge). Das macht Gleichheit, Diff und Merge billig und deterministisch.
- **Adversarial-Schutz:** Die Boundary-Funktion nutzt einen **Repo-Seed** (keyed). Maximalgröße begrenzt degenerierte Chunks.
- **Anfangsparameter (Hypothese H-1, Prototyp stimmt ab):** Ziel 4 KiB, min 1 KiB, max 16 KiB.
- **Strukturelle Teilung:** Ein Commit schreibt nur geänderte Pfade (Wurzel bis Blatt), also O(Δ·Höhe) Knoten.

### 6.4 Commit-Objekt

```
Commit {
  format: u16,
  root: ObjectId,                 // RootMap
  parents: [ObjectId],            // 0 (Root), 1, oder 2+ (Merge)
  author: Principal,              // { id, kind: Human | Agent | System, display }
  committer: Principal,
  time_unix_ms: u64,              // aus Clock-Port; informativ, nicht vertrauenswürdig
  message: String,
  metadata: CanonicalMap,         // session_id, tool_call_id, model_id, prompt_hash, idempotency_key, ...
  journal: Option<ObjectId>,      // feingranulare Änderungen (Abschnitt 9)
  signature: Option<Signature>
}
```

Die Metadaten sind **typisiert und begrenzt** (Anzahl, Größe). Sie sind unvertrauenswürdiger Text, wenn sie je an ein
LLM ausgegeben werden (Abschnitt 11.4).

### 6.5 RootMap

Prolly-Map mit Schlüsseln `tables/<table_id>` → Tabellen-Wurzel, `schema/<table_id>` → Schemaobjekt,
`maps/<name>` → beliebige Key-Value-Maps, `meta/…`. Dadurch erkennt ein Diff unveränderte Tabellen in O(1) (gleiche ID).

---

## 7. Refs, Atomarität, Commit-Protokoll

### 7.1 Refs

Im Kernel (Keyspace `ref`): `ref/<name> → { id: ObjectId, gen: u64, updated_seq, updated_by }`.
Namensräume: `heads/`, `tags/`, `sessions/`, `pins/`, `remote/`.

**Reflog** (Keyspace `rlog`): `rlog/<name>/<gen:be> → { old, new, principal, time, reason, prev_hash }`.
`prev_hash` verkettet Einträge (Manipulationserkennung). Zeit wird **monoton erzwungen**: `max(prev.time, now)`.

### 7.2 Commit-Protokoll (zwei dauerhafte Schritte)

```
 1. Overlay + Basisbaum → neue Knoten im Speicher (TreeEngine.apply)           (keine I/O)
 2. Objekte (Knoten, Journal, Commit) in offenes Pack schreiben + fsync       (cas.flush_durable)
 3. EIN Kernel-Batch:                                                          (atomar)
      + Objektindex-Einträge für alle neuen Objekte
      + ref CAS:      Precondition Equals(ref/<branch>, alter Wert)
      + Reflog-Eintrag (gen+1)
      + Idempotenz:   Precondition Absent(idem/<key>) und Eintrag idem/<key> → CommitId
      + Sitzungsstatus (committed / Overlay löschen)
 4. CommitListener / DerivedIndex / post_commit-Hooks (nicht blockierend)
```

**Absturzanalyse:**

| Abbruch nach | Zustand | Folge |
|---|---|---|
| Schritt 1 | nichts persistent | keine |
| Schritt 2 | Objekte im Pack, kein Index, kein Ref | Recovery scannt Pack-Ende, indiziert neu, Objekte sind unerreichbar → GC |
| Schritt 3 (vor Durabilität) | wie nach 2 | wie oben |
| Schritt 3 (durabel) | Ref, Reflog, Index, Idempotenzeintrag sichtbar | Commit existiert vollständig |
| Schritt 4 | Commit existiert, Listener haben nicht gelaufen | Indexe holen nach (siehe 12.2) |

**Invarianten:**

- **INV-V1** Ein Ref/Index-Eintrag wird nie geschrieben, bevor das Pack-Objekt durabel ist.
- **INV-V2** Jeder von einem Commit erreichbare Knoten ist indiziert, wenn der Ref auf den Commit zeigt.
- **INV-V3** `gen` eines Refs wächst streng monoton; Reflog ist lückenlos und verkettet.
- **INV-V4** Der Packinhalt ist Quelle der Wahrheit für Objektexistenz; der Index ist rekonstruierbar.
- **INV-V5** Fehler bei Pack-`fsync` ⇒ Pack und Instanz vergiftet (wie Teil I INV-P1), kein Retry.
- **INV-V6** Ein Commit ist unveränderlich; „Änderungen an der Historie“ erzeugen **neue** Commits.
- **INV-V7** Tree-Wurzel ist eine Funktion des logischen Inhalts (Historienunabhängigkeit).

Atomare Multi-Ref-Updates (z. B. Merge + Tag + Branch-Löschung): ein Kernel-Batch mit mehreren Vorbedingungen.

### 7.3 Merge, Rebase, Cherry-pick, Revert

- **Merge** = `merge3(base, ours, theirs)`. Beide Seiten gleich → Ergebnis; nur eine Seite geändert → diese; sonst
  schlüsselweise, bei Tabellen **zellenweise** (Abschnitt 10).
- **Konfliktmodell:** Ergebnis ist `MergeOutcome::{Clean(root), Conflicted(ConflictSet)}`. `ConflictSet` ist strukturiert
  (Tabelle, Schlüssel, Spalte, base/ours/theirs-Werte). Auflösung über `MergeResolver` (`ours`, `theirs`, `union`,
  Callback) oder explizit durch den Aufrufer. *Spätere Erweiterung:* erstklassige Konflikte im Commit (Jujutsu-Stil).
- **Constraint-Verletzungen nach Merge** (Unique, Not Null, FK) werden als eigene Menge geführt; Standard ist **Abbruch**.
- **Rebase/Cherry-pick:** Wiederanwendung des **Journal-Deltas** eines Commits auf einen anderen Baum (nutzt `merge3` mit
  Base = Elternbaum). **Revert:** Merge mit vertauschten Rollen. Alles erzeugt neue Commits.
- **Linearer Modus:** Pro Branch konfigurierbar `linear` (Sitzungsdelta wird auf den aktuellen Head angewandt, keine Merge-Commits)
  oder `merge-commits` (Basis bleibt sichtbar).

---

## 8. Sitzungen (Agentenmodell)

```
Session { id, base: CommitId, branch: Option<RefName>, isolation, principal, ttl, quota,
          overlay: Keyspace "sess/<id>", staged_root: Option<ObjectId> }
```

- **Overlay im Kernel** (LSM passt zu vielen kleinen veränderlichen Schreibvorgängen). Eine Sitzung überlebt einen Prozessabsturz.
  Überschreitet das Overlay einen Schwellwert, wird es in einen **sitzungslokalen Baum** (`staged_root`) gefaltet (Bulk-Pfad).
- **Lesen** = Merge-Iterator aus Overlay über Baum (Read-your-writes, Snapshot relativ zu `base`).
- **Isolation:**
  - `Snapshot` (Standard): Konflikte nur bei überlappenden Schreibzugriffen (über `merge3`).
  - `Serializable`: zusätzlich Lese-Set-Validierung. Weil der Baum Diffs billig macht, prüft der Commit
    `diff(base, head) ∩ read_set`. Ein Treffer ⇒ Abbruch. Das ersetzt den heutigen SSI-Sequenzlog für versionierte Daten.
- **Commit-Politik bei `head ≠ base`:** `Fail` | `Rebase` (Delta auf Head, Wiederholung bei Ref-CAS-Fehlschlag) | `Merge(resolver)`.
- **Idempotenz (V-07):** `CommitRequest.idempotency_key` ⇒ Wiederholung liefert dieselbe `CommitId`. Schlüssel haben TTL.
- **Lebenszyklus:** `ttl` und Heartbeat; verwaiste Sitzungen werden abgebrochen, ihr Pin wird frei, ihr Overlay gelöscht.
- **Quoten je Prinzipal/Sitzung:** Bytes im Overlay, neue Objekte, Anzahl offener Sitzungen, Commit-Rate.
- **Schutzregeln:** geschützte Branches (kein direkter Commit, nur Merge), Pflicht-Hooks (Schema-/Constraint-/Policy-Prüfung),
  Verbot von History-Rewrite auf geschützten Refs.

---

## 9. Zeitreise (feingranular, atomar)

### 9.1 Granularitäten

| Stufe | Auflösung | Mechanismus | Kosten |
|---|---|---|---|
| G1 | Kernel-Seq (nur nicht versionierte Keyspaces, Kurzzeit) | `get_at_seq` + Pins | gering, Retention-begrenzt |
| G2 | **Commit** | `ObjectId` → RootMap | O(log n) je Zugriff |
| G3 | **Transaktion / Operation innerhalb eines Commits** | `StateAt(commit, i)` = `apply(Elternbaum, Journal[0..i])` | O(i) Wiederholung, Cache möglich |
| G4 | **Zelle** (Zeile × Spalte) | Zeilen-/Spaltendiff über Commits, `blame` | O(Commits·log n) ohne Index, O(Treffer) mit `KeyHistoryIndex` |

### 9.2 Journal (V-04)

Jeder Commit kann ein `Journal`-Objekt tragen: geordnete Liste `TxRecord { tx_seq, time, principal, ops[(table, pk, op, after)] }`.
**Vorher-Werte werden nicht gespeichert** (ergeben sich aus dem Elternbaum). Betriebsarten je Branch/Sitzung:

| Modus | Journal | Granularität | Kosten |
|---|---|---|---|
| `Squash` | keines | G2 | minimal |
| `PerTx` (Standard für Agenten) | je Transaktion | G3 bis Transaktion | Journalgröße ≈ Änderungsvolumen |
| `PerOp` | je Operation | G3 bis Operation | höher |

Damit gilt: **Jede SQL-Transaktion eines Agenten ist einzeln adressierbar**, ohne dass jede einen eigenen Baum-Commit erzeugt.

### 9.3 Adressierung

`main`, `main~3`, `<commit>`, `main@{2026-11-21T10:00Z}`, `<commit>:tx[7]`, `<commit>:op[42]`, `session/<id>`.

- **Zeit-Auflösung:** `@{ts}` bestimmt den letzten Reflog-Eintrag des Refs mit `time ≤ ts` (Binärsuche über `gen`). Die Zeiten sind
  per Konstruktion monoton, Uhrensprünge werden nicht „zurückgespielt“. Das Verfahren ist dokumentiert.
- **Verzweigen aus der Vergangenheit:** `branch_at(<commit>:tx[i])` erzeugt einen **synthetischen Commit** (Elternteil = Commit, Baum = `StateAt`),
  danach normaler Branch („Was-wäre-wenn“-Simulationen, Agenten-Replay).
- **Wiederherstellen/Revert** erzeugt immer neue Commits (INV-V6). Kein Ref-Zurücksetzen ohne Reflog-Eintrag.

### 9.4 Zeitreise und Schema

Das Schema liegt **im Baum** jedes Commits. Lesen bei `AS OF` nutzt das Schema dieses Commits. Zeilen tragen `schema_id` und werden lazy
gelesen (Abschnitt 10). Dadurch sind Spaltenänderungen zeitreisefest.

### 9.5 Zeitreise und Retention

Gekappte Historie (Shallow/Archiv) liefert `HistoryPruned { requested, oldest_available, policy }`. Es gibt kein stilles Fallback.

---

## 10. Tabellenschicht (`contextra-table`)

- **Schlüsselkodierung:** memcomparable (Tupel-Layer-Stil): Typ-Tags, Integer mit Vorzeichenflip, Floats total geordnet,
  Strings/Bytes mit Escape-Terminator, `NULL` fest einsortiert (kleinster Wert wie in SQLite). Vergleich ist **BINARY**;
  Collations kommen später als eingefrorene `collation_id` im Schema.
- **Zeilenformat:** `[schema_id][null-bitmap][Spalten nach stabiler `column_id`]`. Spalten-IDs sind unveränderlich, daher sind
  Hinzufügen/Umbenennen/Löschen billig. Alte Zeilen werden mit ihrem Schema gelesen und beim Neuschreiben migriert.
- **Typen:** `INTEGER, REAL, TEXT, BLOB, BOOL, TIMESTAMP, UUID, JSON, VECTOR(n, dtype)`. JSON wird kanonisch binär gespeichert
  (Schlüssel sortiert); optionale Pfadindizes. Große `BLOB`/`TEXT` gehen als `BlobTree` in `contextra-cas` (Spalte hält nur die ID).
- **Sekundärindizes** sind eigene Prolly-Maps `(indexierte Spalten, pk) → leer`, **im selben Baum-Update** wie die Zeile
  (atomar, gleiche Wurzelkarte). Unique-Constraints prüft die Schicht beim Schreiben **und nach Merge**.
- **Schema im Baum:** `schema/<table_id>` mit Versionsliste. Schemamerge: Spaltenmengen strukturell, Typkonflikte = Konflikt.
- **Primärschlüssel:** Standard `UUIDv7`/`ULID` (merge-freundlich). `AUTOINCREMENT` wird in v1 **nicht** unterstützt (Zähler konfligieren
  zwischen Branches). SQL-Funktionen `uuid7()`, `ulid()` sind vorhanden.
- **Zellenweiser Merge:** Ändern beide Seiten dieselbe Zeile in **verschiedenen** Spalten ⇒ sauberer Merge; gleiche Spalte mit
  unterschiedlichen Werten ⇒ Konflikt.
- **Schreibpfad:** Mutationen sortieren/bündeln, **ein** `TreeEngine.apply` je Tabelle und Transaktion (Amortisierung der Pfadkosten).

---

## 11. SQL-Zugang und Agentenschnittstellen

### 11.1 Entscheidung (V-08)

| Variante | Urteil |
|---|---|
| SQLite-VFS über Seiten | Abgelehnt: kein semantischer Diff/Merge (Abschnitt 3, Option D). |
| **SQLite Virtual Tables** (`contextra-sqlite`) | **Frontend 1.** SQLite liefert Parser, Planner, Funktionen, Tooling. Wir liefern Speicher und Versionierung. |
| DataFusion `TableProvider` | Später, für analytische Lesepfade (Arrow). |
| Eigene SQL-Engine | Nein. |

Begründung: LLMs erzeugen SQLite-SQL am zuverlässigsten. Die vtab-Schnittstelle (`xBestIndex`, `xFilter`, `xUpdate`,
`xBegin/xSync/xCommit/xRollback`) bildet Bereichs-/Punktzugriffe, Schreibpfad und Transaktionen ab.

### 11.2 Abbildung

- **SQL `BEGIN…COMMIT`** = eine **Transaktion** (ein `TxRecord`) in der Sitzung. **Versions-Commit** ist ein eigener Schritt:
  `SELECT vcs_commit('Nachricht', json_metadata)`. Konfigurierbar: `autocommit_mode = per_tx | manual` (Standard für Agenten: `manual`).
- **Hidden Columns** für Zeitreise: `SELECT * FROM notes WHERE _asof = 'main~3'`, pushdown in `xBestIndex`.
- **Systemtabellen und Funktionen** (damit der Agent Historie **mit SQL** erforscht):
  `vcs_log`, `vcs_branches`, `vcs_tags`, `vcs_diff(a, b, table)`, `vcs_blame(table, pk)`, `vcs_conflicts`, `vcs_merge(a, b)`,
  `vcs_branch(name, from)`, `vcs_revert(commit)`, `vcs_explain(sql)`.
- **Bekannte vtab-Grenzen** (dokumentiert und per Tests abgesichert): keine Trigger/FKs durch SQLite selbst auf vtabs, eingeschränkte
  Indexinformation. **Constraints erzwingt `contextra-table`**, nicht SQLite. Typaffinität und NULL-Sortierung werden gegen
  SQLite-Nativtabellen verglichen (Differentialtest, Abschnitt 14).

### 11.3 Weitere Frontends

- **Rust-API** (Referenz), **C-ABI/Python/Node** (`contextra-ffi`), **CLI `ctx`**.
- **MCP-Server** (`contextra-mcp`): Werkzeuge `sql_query`, `sql_execute`, `session_begin/commit/abort`, `branch`, `merge`, `diff`, `log`,
  `time_travel`, `revert`, `explain`. Er erzwingt Berechtigungstokens (Sitzungs-Capabilities) und Quoten.
- **Mehrprozess:** Der MCP-/Daemon-Server ist der Weg für mehrere Prozesse (V-13).

### 11.4 Agentenfreundliche Fehler und unvertrauenswürdige Inhalte

- Fehler sind **strukturiert**: stabiler `code`, `message`, `details` (z. B. `ConflictSet`), `remediation` (maschinenlesbarer nächster Schritt),
  `retryable`.
- Alles, was aus der Datenbank an ein LLM zurückgeht, ist **unvertrauenswürdig**. Antworten tragen `provenance`
  (Commit, Prinzipal, Zeit) und `untrusted: true` für freien Text (Commit-Nachrichten, Metadaten, Zellinhalte). Die Frontend-Schicht
  kennzeichnet das. **Der Speicher führt nichts aus.**

---

## 12. Abgeleitete Indizes (Volltext, Vektor, Graph, Aggregate)

### 12.1 Prinzip

Indizes, die nicht natürlich Prolly-Maps sind (ANN/HNSW, Invertierte Listen), werden **nicht** in den versionierten Baum gelegt, sondern als
`DerivedIndex` neben ihm geführt:

- Eingabe: `diff(prev_root, next_root)` (billig, weil Prolly-Diff).
- Artefakte: im `cas` (Seiteneffekt-Objekte) oder Kernel-Keyspace, **adressiert über `(index_id, root_id)`**.
- **Konsistenzvertrag:** Eine Abfrage mit `AS OF X` nutzt den Index nur, wenn `index_root == root(X)`. Andernfalls wählt der Planner
  explizit `exact_fallback` (Scan) oder liefert `stale` mit dem Abstand. Kein stilles Ergebnis aus einem falschen Stand.

### 12.2 Aktualisierung

Asynchron per `CommitListener`. **Nachholen nach Absturz:** Jeder Index speichert seinen zuletzt verarbeiteten Commit je Ref;
beim Start wird über `diff` aufgeholt (Schritt 4 des Commit-Protokolls ist deshalb unkritisch).

### 12.3 Vektorindizes

- **Segmentbasiert** (wie LSM): unveränderliche ANN-Segmente je Commit-Bereich, Abfrage über Vereinigung der Segmente, **Sichtbarkeitsprüfung
  und Re-Ranking gegen den Baum** (gelöschte/überschriebene Zeilen fallen heraus). Segmente werden im Hintergrund verschmolzen.
- Embedding-Spalten tragen `model_id` und Dimension im Schema. Modellwechsel = neue Spalte/Version, keine stille Vermischung.
- Branches teilen Segmente, solange die zugrunde liegenden Teilbäume gleich sind (gleiche Hashes).

### 12.4 Volltext/Graph

Gleiches Muster. Graphkanten sind normale Tabellen/Maps (Quelle, Ziel), Traversal-Indizes sind abgeleitet.

---

## 13. Sicherheit, Mandanten, Datenschutz, Löschung

### 13.1 Bedrohungsmodell (Auszug)

| Bedrohung | Maßnahme |
|---|---|
| Datenträger-/Backup-Diebstahl | Objekt- und Packverschlüsselung (AEAD), Schlüsselhierarchie aus Teil I (`KeyHierarchy`), verschlüsselte Kernel-Daten |
| Manipulation der Historie | Hash-Kette (Merkle), verkettetes Reflog, optionale Commit-Signaturen, `verify` |
| Dedup-/Existenz-Orakel zwischen Mandanten | Keyed `ObjectId` (V-05), Repo je Mandant (V-13) |
| Adversariales Chunking / Ressourcen-DoS | Keyed Boundary-Seed, Max-Chunkgröße, Quoten, Limits (Schlüssel/Wert/Zeilen/Commit) |
| Kompromittierter Agent | Capability-Token je Sitzung (Tabellen-/Ref-Allowlist, read-only `AS OF`), geschützte Branches, Audit |
| Unvertrauenswürdiger Inhalt → LLM | Kennzeichnung (11.4), keine Ausführung |
| Supply Chain | Allowlist je Crate (`cargo-deny`), FFI isoliert, SQLite gepinnt und reproduzierbar gebaut |

### 13.2 Löschpflicht vs. unveränderliche Historie (V-11)

Drei Mechanismen, absteigend nach Regelmäßigkeit:

1. **Crypto-Shredding je Subjekt:** Spalten/Werte mit personenbezogenen Daten werden mit einem **Subjektschlüssel** (Envelope) verschlüsselt;
   der Baum enthält Chiffretext. Löschung = Schlüssel vernichten. Hashes bleiben gültig. Aktivierung per Schema-Annotation.
2. **Repo-Löschung:** Schlüssel der Repo-Instanz vernichten (ganzer Mandant).
3. **`rewrite_history` (Admin-Operation):** schreibt betroffene Commits neu (neue IDs), setzt Refs atomar um, schreibt einen Audit-Eintrag, macht
   Signaturen ungültig, löst GC aus. Nur für Notfälle und nie auf geschützten Refs ohne Freigabe.

### 13.3 Zugriffsregeln

`AccessPolicy` wird **in der VCS-Schicht** durchgesetzt (nie im Kernel): Aktionen `ReadRef`, `Commit`, `CreateBranch`, `Merge`, `RewriteHistory`,
`ReadTable`, `WriteTable`. Embedded-Standard: erlaubt alles für lokalen Prinzipal. Server/MCP: Token mit Capabilities.
Zeilenebenen-Sicherheit ist v2 (Prädikat in der Tabellenschicht).

---

## 14. Betrieb

### 14.1 Garbage Collection (kritischster Betriebsprozess)

Erreichbarkeits-GC mit **gleichzeitigen Schreibern**:

- **Wurzeln:** alle Refs, Reflog-Fenster (`undo_window`), Sitzungs-Basen und `staged_root`, explizite Pins, Objekte in-flight.
- **Protokoll (Snapshot-at-the-beginning + Remark):**
  1. Mark ab Kernel-Snapshot `S0` (Refs, Sitzungen, Pins).
  2. Kandidaten (unmarkierte Objekte aus Packs **älter als `S0`**) werden im Index als `sweep_candidate(epoch)` markiert.
  3. **Remark:** alle Ref-/Sitzungsänderungen seit `S0` (über Kernel-Listener/Reflog) werden nachmarkiert.
  4. **Dedup-Falle:** Ein Schreiber, der ein Objekt per Dedup wiederverwendet, das `sweep_candidate` ist, **schreibt es erneut** (neues Pack) oder löscht
     die Markierung im selben Kernel-Batch wie den Ref-Update.
  5. Physisches Löschen nur für Packs, deren Lebendanteil unter Schwelle liegt (**Copy-Forward** in neue Packs), nach Grace-Periode und
     erneuter Prüfung.
- **Gate:** deterministische Simulation (zufällige Interleavings von Schreibern, GC, Abstürzen) beweist „kein erreichbares Objekt geht verloren“
  (Property-Test mit `FaultVfs`).

### 14.2 Retention und Kosten

`RetentionPolicy` je Ref: `KeepAll | KeepLast(N) | KeepDays(D)` plus `undo_window`. `shallow`-Kappung ersetzt alte Commits durch einen Snapshot-Commit.
Kalte Packs können über einen `PackBackend`-Port (z. B. Objektspeicher) ausgelagert werden. Wachstum wird als Metrik und Alarm geführt.

### 14.3 Backup, Restore, `verify`

- **Konsistente Sicherung:** Kernel-Checkpoint (Hardlink/Kopie der unveränderlichen Dateien) + Packs bis zum `DurableMark`.
  Ergebnis ist ein Verzeichnis, das ohne Replay öffnet.
- **`verify`:** (a) Hashes aller Objekte, (b) Erreichbarkeit aller Ref-Ziele, (c) Reflog-Kette, (d) Index ↔ Pack-Konsistenz, (e) Format/Features.
  Läuft online (Snapshot) und offline.
- **Restore-Test** ist Teil der CI (Sicherung → Zerstörung → Wiederherstellung → `verify` → Vergleich der Wurzeln).

### 14.4 Formatevolution

`REPO_FORMAT` (neben `STORE_FORMAT`): `format_version`, `required_features`, `optional_features`. Unbekannte **Pflicht**features ⇒ Öffnen
verweigert (`Unsupported`), unbekannte optionale werden ignoriert. Jede Schicht führt Golden-Dateien. Upgrades sind explizit (`ctx upgrade`), vorwärts-kompatible
Lesbarkeit ist Regel, rückwärts nicht garantiert.

### 14.5 Beobachtbarkeit und Explain

Metriken (Auszug): `vcs_commit_seconds`, `tree_nodes_written`, `tree_height`, `node_cache_hit_ratio`, `gc_live_ratio`, `history_bytes_per_day`,
`session_open`, `session_conflicts_total`, `index_lag_commits`. `vcs_explain` beschreibt Plan, genutzte Indizes, geschätzte Kosten.

### 14.6 Plattform

Unterstützte Dateisysteme: lokal (ext4/XFS/APFS/NTFS). **Nicht** unterstützt: NFS/SMB für Schreibzugriff (fsync-/Lock-Semantik). macOS: `sync_all` nutzt `F_FULLFSYNC`
(in `RealFs` per Test zu belegen). Windows: Dateisicherheit über `FileSecurity`-Port (Teil I).

### 14.7 Grenzen (konfigurierbar, mit Fehlercode)

Schlüssel ≤ 4 KiB, Zellwert inline ≤ 256 KiB (darüber BlobTree), Zeilen je Transaktion, Objekte je Commit, Sitzungen je Prinzipal, Refs je Repo, Baumhöhe (Alarm).

---

## 15. Leistungs- und Kostenmodell (Hypothesen)

| ID | Hypothese | Validierung |
|---|---|---|
| H-1 | Ziel-Chunk 4 KiB (1–16 KiB) balanciert Höhe, Schreibmenge und Cache. | V0-Parameterscan |
| H-2 | Commit mit Δ = 100 Zeilen in 1 Mio. Zeilen schreibt ≲ 1 MiB Knoten (Höhe ≈ 3–4, Pfadteilung beim Bündeln) und endet bei `p50 < 15 ms`, `p99 < 50 ms` (NVMe, ein `fsync` Pack + ein `fsync` Kernel-Batch). | V0 |
| H-3 | Punktlesen am Head mit warmem Cache `p99 < 200 µs`; kalt höchstens Höhe × I/O. | V0 |
| H-4 | Historienwachstum eines Agenten-Workloads ≤ 20× des logischen Änderungsvolumens (ohne Kompression). | V0 + Langläufer |
| H-5 | Diff zweier Commits kostet O(Δ·Höhe), unabhängig von der Tabellengröße. | V2 |
| H-6 | Zwei `fsync` pro Commit lassen sich mit Group-Commit (Teil I) über viele Sitzungen amortisieren. | V3 |

**Vergleichs-Benchmarks:** SQLite (WAL, `synchronous=FULL`) für Punkt/Range/Schreiben; ein Dolt-ähnlicher Workload für Diff/Merge/Branch;
ein simulierter Agenten-Workload (viele kleine Transaktionen, viele kurzlebige Branches, wenige Merges). Ziel ist **nicht**, SQLite im reinen OLTP zu schlagen,
sondern: im Agenten-Workload konkurrenzfähig bei vollständiger Versionierung.

**Abbruchkriterien V0 (Rückfall auf Option C oder Parameterüberarbeitung):** H-2 `p99 > 50 ms` **oder** H-4 `> 20×` **oder** H-3 warm `p99 > 200 µs`
bei Referenzhardware, nach Abstimmung der Parameter.

---

## 16. Phasenplan Track V

Abhängigkeiten zu Teil I: V0 darf nach P0 mit Wegwerf-Code starten. V1+ benötigt die Kernel-Vorleistungen K-1…K-10 (Teil I, Abschnitt 12).
Größen: S ≈ Tage, M ≈ 1–2 Wochen, L ≈ 3+ Wochen.

| Phase | Inhalt | Abhängig von | Tor (Auszug) |
|---|---|---|---|
| **V0** (M) | Wegwerf-Prototypen: ProllyMap in Memory + einfacher Packspeicher; Messung H-1…H-4, Agenten-Workload-Generator, Vergleich mit SQLite. Ergebnis: Parameter und Go/No-Go. | P0 | Bericht `docs/spikes/v0.md`, Abbruchkriterien ausgewertet |
| **V1** (M) | `contextra-cas`: Objekt-ID, Packformat, Index im Kernel-Keyspace, Tail-Scan-Rebuild, Kompression/Verschlüsselung per Feature, Fehlerpolitik INV-V5 | P4, K-1, K-8 | Crash-Matrix über Pack+Kernel; Index-Rebuild-Test; Fuzz Pack-Decoder |
| **V2** (L) | `contextra-vcs/tree`: ProllyMap (get/range/apply/diff/merge3), Blob-Bäume, Knoten-Cache | V1, K-3 | Gesetze: Historienunabhängigkeit, `apply(diff(a,b),a)=b`, Merge-Eigenschaften; Modelltest gegen `BTreeMap` |
| **V3** (L) | Commits, Refs, Reflog, Commit-Protokoll (7.2), Sitzungen + Overlay, Idempotenz, Hooks, AccessPolicy-Trait | V2, K-2, K-6, K-7 | Absturzmatrix je Protokollschritt; Linearisierbarkeit der Ref-Updates; Cancel-/Retry-Tests |
| **V4** (M) | Zeitreise: Journal, `StateAt`, Adressierung, `branch_at`, Zeit-Auflösung, `HistoryPruned` | V3 | Property: `StateAt(c, n) = Endzustand`, Wiederholungs-Äquivalenz; Referenzmodell |
| **V5** (L) | `contextra-table`: Schlüssel-/Zeilenkodierung, Schema im Baum, Sekundärindizes, Constraints, zellenweiser Merge, JSON/Vector/Blob-Typen | V3 | Kodierung geordnet (proptest); Merge-Matrix; Schemaevolution × Zeitreise |
| **V6** (L) | `contextra-query` + `contextra-sqlite` (vtab) + `contextra-agent` + `contextra-mcp` + `ctx` | V5 | Differentialtest vs. SQLite-Nativtabellen; Systemtabellen; MCP-Konformitätstests |
| **V7** (M) | Abgeleitete Indizes: Framework, FTS, Vektor (segmentbasiert), Konsistenzvertrag, Nachholen | V3, V5 | Absturz-/Nachhol-Tests; Recall-Messung; `stale`-Verhalten |
| **V8** (L) | Betrieb: GC (14.1) mit Simulation, Retention, Backup/Restore, `verify`, Metriken, `ctx upgrade` | V3 | GC-Simulation ohne Verlust (≥ 10⁶ Schritte); Restore-CI |
| **V9** (M) | Sicherheit: Objekt-Verschlüsselung, Subjektschlüssel, `rewrite_history`, Signaturen, Tokens/Capabilities, Audit | V3, V5 | Bedrohungsmodell-Review; Shredding-Test (nach Schlüsselvernichtung unlesbar, Hashes intakt) |
| **V10** (M) | Migration/Integration des Altsystems (Collections, Graph, Text, Session-DAG) auf Tabellen/VCS; Abgleich K-11 | V5, A-V1 | Importer erzeugt Initialcommit; Äquivalenztests gegen Altpfad |

Jede Phase endet mit grüner CI, aktualisiertem Fortschrittsprotokoll (Abschnitt 20) und Format-Goldens.

---

## 17. Teststrategie

| Netz | Inhalt |
|---|---|
| **Gesetze (proptest)** | Historienunabhängigkeit des Baums; `apply(diff(a,b), a) = b`; `diff(a,a)=∅`; `merge3(b,a,a)=a`; `merge3(b,x,b)=x`; Kommutativität sauberer Merges (bei symmetrischer Auflösung); Rang/`COUNT` stimmt mit Iteration überein |
| **Modellbasiert** | Referenzmodell (`BTreeMap` + Commitliste im Speicher) gegen die Implementierung, zufällige Operationsfolgen inkl. Branches/Merges/Zeitreise |
| **Crash-Matrix** | `FaultVfs` bricht an **jedem** I/O-Schritt von Commit, Flush, GC, Backup ab; Erwartung: Zustand = Präfix der bestätigten Commits, `verify` grün |
| **GC-Simulation** | Deterministische Interleavings (Schreiber, Dedup, GC, Abstürze); Invariante „alles Erreichbare existiert“ |
| **Differential** | Gleiche SQL-Skripte gegen SQLite-Nativtabellen und `contextra-sqlite`: Ergebnisse, Fehlerklassen, NULL-/Typregeln |
| **Fuzz** | Objekt-/Knoten-/Commit-/Journal-Decoder, Schlüssel-/Zeilenkodierung, vtab-Constraintplaner |
| **Format-Goldens** | Je Schicht Referenzdateien; ID-Stabilität der kanonischen Kodierung über Versionen/Plattformen |
| **Determinismus** | Uhr/Zufall nur über Ports; gleiche Eingaben ⇒ gleiche IDs (außer Zeit/Autor) |
| **Soak/Chaos** | Langläufer mit Agenten-Workload, Absturz-Injektion, Festplatte-voll, Speicherdruck |
| **Agenten-Tests** | Golden-Transkripte typischer Werkzeugaufrufe (Konflikt, Retry, Zeitreise, Revert), Fehlerkatalog-Stabilität |
| **Lizenz/Supply Chain** | `cargo deny` (Lizenzen, Advisories), reproduzierbarer SQLite-Build |

---

## 18. Blindfleck-Register

| # | Risiko / Lücke | Entscheidung / Maßnahme | Phase |
|---|---|---|---|
| 1 | Löschpflicht vs. unveränderliche Historie | Crypto-Shredding je Subjekt, Repo-Löschung, `rewrite_history` | V9 |
| 2 | GC vs. Zeitreise vs. Retention | Policies, `undo_window`, `HistoryPruned` | V8 |
| 3 | GC-Race mit Dedup (Objekt „wiederbelebt“ während Sweep) | SATB + Remark, Candidate-Markierung, Wiederschreiben, Simulation | V8 |
| 4 | Dedup-/Existenz-Orakel zwischen Mandanten | Keyed IDs, Repo je Mandant | V1/V9 |
| 5 | Adversariales Chunking, Ressourcen-DoS | Keyed Seed, Max-Chunk, Quoten, Limits | V2/V3 |
| 6 | Nichtdeterministische Kodierung (Float, Unicode, Mapreihenfolge) | Kanonische Kodierung + Goldens, BINARY-Collation | V2/V5 |
| 7 | Collation/Locale | v1 nur BINARY, später `collation_id` im Schema | V5 |
| 8 | Uhren, Skew, `AS OF TIMESTAMP` | Monotones Reflog, dokumentierte Auflösung | V4 |
| 9 | Schemaevolution × Zeitreise | Schema im Baum, `schema_id` je Zeile | V5 |
| 10 | Merge-Semantik für Zähler, Unique, FK | Kein AUTOINCREMENT, UUIDv7, Constraint-Verletzungsmenge, Abbruch | V5 |
| 11 | Konflikt-UX für Agenten | Strukturierter `ConflictSet`, Resolver, Fehlerkatalog | V3/V6 |
| 12 | Retries erzeugen Doppel-Commits | Idempotenzschlüssel | V3 |
| 13 | Verwaiste Sitzungen/Pins | TTL, Heartbeat, Pin-Ablauf, Quoten | V3 |
| 14 | Große Blobs, Streaming | BlobTree (FastCDC), Streaming-API, Limits, K-9 | V1/V2 |
| 15 | Viele Repos: Threads, Cache, I/O | `JobRunner`, `ResourceGovernor`, Single-Flight-Committer (K-6) | P7/V3 |
| 16 | Crash-Konsistenz Pack ↔ Kernel | Reihenfolge INV-V1, Pack als Wahrheit, Tail-Scan-Rebuild | V1 |
| 17 | fsync-Fehler | Fail-stop auch für Packs (INV-V5) | V1 |
| 18 | Backup-Konsistenz | Checkpoint + `DurableMark`, Restore-CI | V8 |
| 19 | Formatevolution | `STORE_FORMAT`/`REPO_FORMAT` mit Pflicht-/optionalen Features | P6/V3 |
| 20 | Hash-Agilität | `algo`-Byte in jeder ID | V1 |
| 21 | Beobachtbarkeit und Erklärbarkeit | Metriken, `vcs_explain`, Reflog | V6/V8 |
| 22 | Berechtigungen für Agenten | Capability-Token, geschützte Branches, `AccessPolicy` | V3/V9 |
| 23 | Prompt-Injection über gespeicherte Inhalte | Kennzeichnung `untrusted`, Provenienz, keine Ausführung | V6 |
| 24 | SQLite-Semantik-Abweichungen (Affinität, NULL, vtab-Grenzen) | Differentialtests, dokumentierte Grenzen, Constraints in Tabellenschicht | V6 |
| 25 | Vektorindex vs. Versionierung | Segmentbasiert, Sichtbarkeitsprüfung gegen Baum, Konsistenzvertrag | V7 |
| 26 | Mehrprozess-Zugriff (SQLite erlaubt ihn) | Single-Process-Instanz, Daemon/MCP als Mehrprozessweg, klar dokumentiert | V6 |
| 27 | Plattform: `fsync`, NFS, Windows-ACL | Unterstützungsmatrix, `RealFs`-Tests, `FileSecurity` | P4/V8 |
| 28 | Limits und Quoten | Konfigurierbar mit Fehlercodes (14.7) | V3 |
| 29 | Determinismus der Tests | Ports für Uhr, Zufall, I/O | alle |
| 30 | Lizenzen/Supply Chain (SQLite, zstd, ggf. DataFusion) | `cargo deny`, Allowlist je Crate | V6 |
| 31 | Migration des Altsystems | Importer, Initialcommit, Äquivalenztests, K-11 | V10 |
| 32 | Spätere Replikation/Sync | Objektmodell neutral, `have/want` möglich, Keyed-ID-Grenze dokumentiert | nach V10 |
| 33 | Unbegrenztes Historienwachstum, Kosten | Retention, Kompression, kalte Packs, Metriken/Alarme | V8 |
| 34 | LLM-Nutzbarkeit (Schemaentdeckung, Fehlertexte) | `vcs_explain`, Schema-Introspektion, Fehlerkatalog mit `remediation` | V6 |
| 35 | Korrektheit der Prolly-Implementierung (Randfälle, Höhenwechsel) | Gesetzes- und Modelltests, Fuzz, Referenzimplementierung im Test | V2 |

---

## 19. Entscheidungsregister

| ID | Entscheidung |
|---|---|
| V-01 | Persistente inhaltsadressierte Prolly-Bäume als Versionierungsgrundlage; Hybrid C nur als Rückfall (Abschnitt 3). |
| V-02 | Schichten: Kernel → cas → vcs → table → query/index → Frontends; Abhängigkeiten nur nach unten. |
| V-03 | Objekte in **Packs** (Quelle der Wahrheit), Index im Kernel (rekonstruierbar); keine großen Blobs durch den LSM. |
| V-04 | Zwei Ebenen: Transaktion und Commit; **Journal** im Commit für G3-Zeitreise ohne Baum-Commit je Transaktion. |
| V-05 | Verschlüsselte Repos nutzen **keyed ObjectIds**; `algo`-Byte für Hash-Agilität. |
| V-06 | Sitzung = Branch-Arbeitskopie mit dauerhaftem Overlay; Isolation `Snapshot` oder `Serializable` (Diff-basierte Lese-Set-Validierung). |
| V-07 | Idempotenzschlüssel für Commits; Fehlerkatalog mit `remediation` und `retryable`. |
| V-08 | SQL über SQLite-Virtual-Tables; Query-Port erlaubt weitere Frontends; keine eigene SQL-Engine; kein Seiten-VFS. |
| V-09 | Abgeleitete Indizes ausserhalb des versionierten Baums, mit Konsistenzvertrag `(index_id, root_id)`. |
| V-10 | Tabellenschicht: memcomparable Schlüssel, Spalten-IDs, `schema_id` je Zeile, kein AUTOINCREMENT in v1, zellenweiser Merge. |
| V-11 | Löschung durch Crypto-Shredding, Repo-Löschung, Admin-`rewrite_history`; Historie bleibt sonst unveränderlich (INV-V6). |
| V-12 | GC mit SATB + Remark + Candidate-Markierung; GC ist Phase-Gate mit Simulation. |
| V-13 | **Ein Repository = eine Kernel-Instanz**, Single-Process; Mehrprozess über Daemon/MCP. |
| V-14 | Zeit ist informativ; `AS OF TIMESTAMP` löst über monotones Reflog auf. |
| V-15 | Kernel bleibt domänenfrei und `forbid(unsafe_code)`; FFI nur in `contextra-sqlite` und `contextra-ffi`. |
| V-16 | Commit-Politik pro Branch `linear` (Standard) oder `merge-commits`. |
| V-17 | Konflikte in v1 als Ergebnismenge, nicht im Commit gespeichert; erstklassige Konflikte als spätere Option. |

---

## 20. Annahmen, offene Punkte, Fortschritt

| ID | Annahme / offener Punkt |
|---|---|
| AV-1 | Das Datenmodell der Downstream-Crates (`contextra-db/engine/checkpoint/agent`: Collections, Graph, Text, `__session_dag`) ist in Tabellen und Maps abbildbar. Inventur in P0/V10. |
| AV-2 | `contextra-checkpoint` und `rollback_to_tx` werden durch VCS-`reset`/Commits ersetzt (K-11, Teil I A-07). |
| AV-3 | Ein Single-Process-Modell reicht für den Embedded-Einsatz; Mehrprozess läuft über den Daemon. |
| AV-4 | Die Referenzhardware für V0 (NVMe, Kerne, RAM) wird in V0 festgelegt und dokumentiert. |
| AV-5 | SQLite (C) darf als gepinnte Abhängigkeit **nur** in `contextra-sqlite` verwendet werden (Lizenz und Build geprüft). |
| AV-6 | Teil I ist umgesetzt, bevor V1 beginnt (K-Vorleistungen enthalten). Ohne K-2/K-3 ist V3 nicht sinnvoll. |

**Grenzen dieser Analyse:** Die Konzeption stützt sich auf den Kernel-Stand `47c9865` und allgemein bekannte Verfahren (Prolly-Bäume, Packs, SATB-GC,
SQLite-vtab). Es gibt keine Prototypen. Alle Leistungsaussagen sind Hypothesen, bis V0 sie misst.

**Fortschrittsprotokoll (Implementierer pflegt):**

| Phase | Status | PR/Commit | Abweichung | Datum |
|---|---|---|---|---|
| V0 | offen | | | |
| V1 | offen | | | |
| V2 | offen | | | |
| V3 | offen | | | |
| V4 | offen | | | |
| V5 | offen | | | |
| V6 | offen | | | |
| V7 | offen | | | |
| V8 | offen | | | |
| V9 | offen | | | |
| V10 | offen | | | |

---

## Anhang A: Beispiel-Ablauf (Agent)

```
begin_session(base = main, principal = agent:planner-7, isolation = Snapshot, mode = PerTx)
  SQL: BEGIN; INSERT INTO facts(...); UPDATE tasks SET state='done' WHERE id=...; COMMIT;     -- TxRecord #1
  SQL: BEGIN; INSERT INTO facts(...); COMMIT;                                                  -- TxRecord #2
commit(message="Plan abgeschlossen", metadata={tool_call_id, model_id}, idempotency_key=K, on_conflict=Rebase)
  → Objekte schreiben + fsync → Kernel-Batch {Index, ref CAS main, Reflog, idem/K, Session=committed}
time_travel:  SELECT * FROM facts WHERE _asof='main~1';                  -- Zustand vor dem Commit
              SELECT * FROM vcs_diff('main~1','main','facts');           -- Zeilen- und Spaltendiff
              SELECT * FROM vcs_blame('facts', :pk);                     -- wer/wann/welche Zelle
what_if:      vcs_branch('experiment', from => 'main:tx[1]')             -- synthetischer Commit aus Transaktion 1
revert:       vcs_revert('<commit>')                                      -- neuer Commit, nichts wird überschrieben
```

## Anhang B: Abgrenzung zu Git, Dolt, SQLite

| | Git | Dolt | SQLite | dieses System |
|---|---|---|---|---|
| Datenmodell | Dateien/Bäume | Tabellen | Tabellen | Tabellen + Maps + Blobs |
| Versionierung | Commit-DAG | Commit-DAG (Prolly) | keine | Commit-DAG (Prolly) + Journal je Transaktion |
| Zellen-Merge | nein | ja | – | ja |
| Zeitreise | Commit | Commit | keine | Commit, Transaktion, Operation, Zelle |
| Eingebettet / Dateimodell | Verzeichnis | Server/Embedded | Einzeldatei, Mehrprozess | Verzeichnis, Single-Process (+ Daemon) |
| Agentenfunktionen | – | teilweise | – | Sitzungen, Idempotenz, Quoten, MCP, Provenienz |
| Löschung in Historie | `filter-repo` | `rebase`/GC | n/a | Crypto-Shredding, `rewrite_history` |
