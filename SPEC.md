# SPEC.md — `contextra-storage` (Speicherkern), Revision R6

| | |
|---|---|
| Status | **Normativ ab Gate G-0 (§24), Revision R6.** Konsolidiert aus R5 (Abnahmekandidat), R2, R3, der R5-Variante (Freigabekandidat), den Konzeptdokumenten und einer unabhängigen Analyse. Ersetzt alle Vorfassungen vollständig (jetzt in `docs/archive/`). Änderungen gegenüber R5: Anhang E. |
| Gegenstand | Eingebetteter, synchroner, crash-sicherer Schlüssel-Wert-Kern mit atomaren Mehr-Space-Änderungen, MVCC-Sichten, löschlokaler Datenanordnung und nachweisbarer, kryptografischer plus physischer Löschung. Crates: `contextra-storage` (Kern), `contextra-storage-ports` (Ports), `contextra-crypto-ref` (Referenz-Primitive, nur Dev-Abhängigkeit). |
| Sprache | Prosa und Rustdoc deutsch, Bezeichner englisch (P-7). |
| Verifikation | Erstellt durch Lesen von `docs/`, Prüfung des Altstands mit Messwerten (Anhang B, am 22.11.2026 per `grep`/`wc` nachgeprüft) und Abgleich der Konzepte. **Nichts kompiliert, nichts im Kern gemessen.** Alle Leistungszahlen sind Hypothesen (`H-*`, §23). Formate brauchen vor **G-F** Referenzvektoren aus einem unabhängigen Referenzcodierer (Anhang A). |
| Rechtlicher Vorbehalt | Der Kern liefert technische Eigenschaften. Er trifft keine Rechtsaussage zu Art. 17 DSGVO, Fristen oder Ausnahmen (§26, O-2). „Gelöscht“ und „nicht lesbar“ sind immer an eine Stufe aus §17.4 gebunden. |

---

## 1. Geltungsbereich und Konventionen

### 1.1 Zweck

`contextra-storage` speichert Daten für Sprachmodell-Anwendungen: viele kleine atomare Schreibvorgänge, kurzlebige Sichten, Lesen von Kopfdaten ohne Körperzugriff, budgetierte Ergebnisseiten, Verlauf mit Fortschrittszeigern, flüchtige Arbeitsbereiche und Löschung als Entwurfsprinzip. Der Kern kennt weder Tokenizer, Indexstrukturen, Mandanten, Rechte noch Rechtsgrundlagen. Er liefert Mechanismus, nie Politik (§3.3). **Auf diesem Kern baut alles auf; bis G-P (§24) wird nichts anderes entwickelt.**

### 1.2 Normsprache

**MUSS / DARF NICHT**: zwingend. **SOLL / SOLL NICHT**: Regelfall, Abweichung braucht Begründung im Review. **KANN**: optional.

### 1.3 Dokumentenhierarchie

1. `docs/SPEC.md` (diese Datei): **normativ** (Formate, Protokolle, Invarianten, API-Vertrag, Phasen, Gates).
2. `AGENTS.md` (Repository-Wurzel): Arbeitskontext. Fasst zusammen, darf der Spec nie widersprechen, verweist bei Zweifeln auf sie.
3. `docs/archive/`: Herkunft. Keine Anforderungsquelle.

Ein separates Implementierungsdokument gibt es nicht. Der Ist-Stand steht in `AGENTS.md` §3, die Phasen in §24.

### 1.4 Kennungen

`Z-n` Ziel · `NZ-n` Nicht-Ziel · `P-n` Codierregel · `F-n` Fehlerklasse · `AT-n` Angreifer · `AN-nn` Annahme · `SD-nn` Entscheidung (§25) · `INV-*` Invariante (§22) · `H-n` Hypothese (§23) · `O-n` offene Entscheidung (§26) · `G-*` Gate (§24) · `ER0…ER7` Schritte der Löschung (§17.5) · `K-nn` Korrektur gegenüber R5 (Anhang E).

### 1.5 Änderungsprozess

Änderungen an §8, §11, §12, §15, §17, §18, §22 brauchen zwei Reviewer, aktualisierte Referenzvektoren (Anhang A) und bei Formatbezug eine Prüfung gegen die Freeze-Liste (§11.9). Nach **G-F** ist jede Formatänderung ein Migrationswerkzeug (SD-04), nie ein Zweitpfad im Kern. Reihenfolge immer: erst diese Datei, dann der Code.

---

## 2. Leitentscheidungen

| # | Entscheidung | Wirkung |
|---|---|---|
| L1 | **Cluster** (`ClusterId`, 16 Byte, opak) ist Löschdomäne, Schlüsseldomäne und Platzierungseinheit. | Eine Einheit für Löschung, Kryptografie und Lokalität. |
| L2 | Ordnung in geclusterten Spaces: `(cluster, user_key, tag↓)`. Blöcke sind **clusterrein**; die Daten eines Clusters liegen je Datei als **ein zusammenhängender Abschnitt**. | Löschen = Dateiabwurf (Pure) oder Abschnitt überspringen und Rest roh kopieren (Packed). |
| L3 | In versiegelten, geclusterten Spaces ist **alles** eines Clusters (Schlüssel, Kopf, Körper, Index, Filter, Marker) mit dem Clusterschlüssel versiegelt (§17.3). | Schlüsselvernichtung macht den ganzen Cluster unlesbar. |
| L4 | Dateien sind **Pure** (ein Cluster) oder **Packed** (mehrere Cluster einer Packgruppe). | Skaliert auf Millionen Cluster; Ablauf = Dateiabwurf. |
| L5 | **Ein Schreiber** (Committer) für alle stamp-relevanten Zustandsänderungen einschließlich Löschmarken (K-02). Dateien unveränderlich, `Version` per `Arc`. | Race-Freiheit durch Konstruktion (§19). |
| L6 | **Catalog ist die einzige Wahrheit über Dateien.** Lebendig ⇔ ein durabler Edit nennt die Datei. | Crash-Sicherheit, Verwaisten-Suche. |
| L7 | **Reihenfolge ist Vertrag:** Gate vor Schlüsselvernichtung vor Dateiwechsel vor Unlink. Nur Roll-forward. | §17.5. |
| L8 | **Cluster-Links:** abgeleitete Cluster an Quellcluster binden; Löschung zieht sie atomar mit, nur auf ausdrücklichen Auftrag. | §18.2. |
| L9 | **Restriction** (Sperrvermerk mit Pflichtgrund) verhindert Löschung. Kein Override im Kern. | §16.5. |
| L10 | **Stamps werden nie doppelt vergeben**, auch nicht nach Absturz. | `INV-STAMP-3`. |
| L11 | Kopf ohne Körper lesbar; **Gewichte** im Klartext und als Block-Aggregat. | Budgetseiten ohne Körperzugriff. |
| L12 | **Fünf Erweiterungsverträge** (§18). | Anbindung ohne Formatbruch. |
| L13 | **Instanzsperre** (`LOCK`), `Perm::Private`, **Rollback-Anker** (K-04). | Keine Doppelöffnung, keine Wiederkehr gelöschter Daten. |
| L14 | **Formale Modelle** (TLA+) und deterministische Simulation sind Gate-Bestandteil. | Race-Freiheit wird bewiesen oder gemessen. |
| L15 | **Flüchtige Spaces** (`Volatile`) ohne Redo (SD-26). | Scratchpads ohne fsync. |

---

## 3. Architektur

### 3.1 Crates und Schichten

```
contextra-storage-ports   Traits und Werttypen, keine Logik, keine Laufzeit-Abhängigkeiten
        ▲
contextra-storage         Kern; nur std + ports + bewilligte Abhängigkeiten (§3.5); #![forbid(unsafe_code)]
        ▲ (nur dev-dependency)
contextra-crypto-ref      Referenz für Cipher/Mac/Kdf (nur Tests)
```

Plattformnahes (Direkt-E/A, Hardware-CRC, Speicherschutz, `mmap`) ist Port-Implementierung in eigenen Crates.

**Namensraum (SD-33):** Der Neubau liegt in `crates/contextra-storage` und `crates/contextra-storage-ports` im Repository `tfufuz1/contextra`. `contextra-store` (Altstand) und `contextra-ports` (bestehend, anderes Crate) bleiben unverändert, bis der Neubau sie ablöst.

### 3.2 Modulschnitt

Abhängigkeiten nur nach unten, per Test `layering` erzwungen:

```
api        Facade: Storage, Space, View, Change, Cursor, Hold, Restriction, Consumer, Eraser, Inspector
engine     committer · memtable · version · views · registry · claims · retention · volatile
maint      flush · compaction · blob_gc · purge · pack_drop · scheduler (Governor)
erase      gate · cluster_table · links · protocol · ledger · oracle (Feature testing)
recovery   open-Sequenz, Anker-Prüfung
catalog    snapshot · edits · log · rollover · current · reservations
store      run · blob · segment · redo
keys       cluster_keys · key_cache · generations · rotation
codec      varint · tags · checksums · comparator
env        Fs · Clock · Entropy · Governor · Metrics · Anchor (MemFs, RealFs, FaultFs, Sim)
sync       Sperren mit Rang, Epoche
```

`std::fs` nur in `env/real_fs.rs`.

### 3.3 Mechanismus, nie Politik

Der Kern entscheidet nie, **ob** gelöscht werden darf oder muss, **wer** ein Recht hat, **welche** Frist gilt oder **was** ein Cluster bedeutet. Er setzt Aufträge atomar, geordnet und nachweisbar um und verweigert sie bei technischem Hindernis oder `Restriction`. Politik bleibt oben.

### 3.4 Threading

- **Ein Committer** (Rang 1): Flat-Combining, Gruppen-Fsync, einziger Ort für Stamp-Vergabe und Cluster-Zustandsübergänge.
- **Lesen:** lock-frei bis auf `Arc`-Klone.
- **Wartung:** Lanes `urgent`, `flush`, `compact`, `blobgc`, `idle` (§14.1). Flush wartet nie hinter Compaction.
- **Threads gehören dem Aufrufer** oder einem übergebenen `Spawner`.

### 3.5 Bewilligte Abhängigkeiten (SD-03)

| Zweck | Wahl | Begründung / Alternative |
|---|---|---|
| Memtable | `crossbeam-skiplist` hinter Trait `Memtable` | lock-frei ohne eigenes `unsafe`; Alternative: eigener Baum hinter demselben Trait |
| Puffer | `bytes` | Zero-Copy |
| Hash/MAC | `blake3` | in `crypto-ref` |
| AEAD | `chacha20poly1305` (XChaCha) | in `crypto-ref` |
| Zeroize | `zeroize` | Schlüsselmaterial |
| Test (dev) | `loom`, `shuttle`, `proptest`, `cargo-mutants` | §20 |

Kryptografie erreicht den Kern **nur über Ports**; `blake3` und `chacha20poly1305` stehen im Kern nur, soweit der Standard-Port sie bündelt. Jede weitere Abhängigkeit braucht Registereintrag und `cargo-deny` (`advisories`, `bans`, `sources`).

### 3.6 Lint-Pflicht

Im Kern sind per `#![deny]`/`clippy.toml` Fehler: `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`, `indexing_slicing`, sowie `let _ =` auf Rückgaben von `sync_*`, `rename`, `unlink`. Ausnahmen nur in `#[cfg(test)]`.

---

## 4. Ziele, Nicht-Ziele, Bedrohungsmodell

**Ziele**

| ID | Ziel |
|---|---|
| Z-1 | Atomarität über Spaces, auch nach Absturz. |
| Z-2 | Durability: sichtbar nur nach Durabilität (Ausnahme `Relaxed`, Opt-in). |
| Z-3 | Snapshot-Isolation ohne Lesesperren. |
| Z-4 | Löschung eines Clusters in Zeit unabhängig von der Gesamtgröße des Speichers. |
| Z-5 | Ablauf einer Packgruppe als O(Dateien)-Operation. |
| Z-6 | Nachweisbarkeit: Receipt, `verify`, Orakel, Ledger mit Kette und Anker. |
| Z-7 | Kopf ohne Körper lesen, Budgetseiten ohne Körperzugriff. |
| Z-8 | Race-Freiheit durch Konstruktion, belegt durch Modell, Simulation, Loom. |
| Z-9 | Begrenzte Ressourcen (Speicher, Deskriptoren, Wartungs-E/A). |
| Z-10 | Stabile Verträge nach oben (§18). |
| Z-11 | Flüchtige Arbeitsbereiche ohne Redo-Kosten. |
| Z-12 | Idempotente Wiederholung für Agenten (`IdemToken`, §12.5). |

**Nicht-Ziele**

| ID | Nicht-Ziel | Wohin |
|---|---|---|
| NZ-1 | Verteilung, Replikation, Konsens | obere Schicht |
| NZ-2 | Eigene Kryptografie | Ports |
| NZ-3 | Schemas, Sekundärindizes, SSI, Lesemengen | obere Schicht (aus `Change`/`Guard`/`View`) |
| NZ-4 | Merge-Operatoren (Tag `6` reserviert) | spätere Formatentscheidung |
| NZ-5 | Schutz von Prozessspeicher, Swap, Core-Dateien | best-effort (§17.9) |
| NZ-6 | Umordnung über Clustergrenzen | nie |
| NZ-7 | Kompatibilität mit dem Altformat im Kern | optionaler Importer |
| NZ-8 | Windows mit voller Durability in v1 | Port `Fs::sync_dir` |
| NZ-9 | `O_DIRECT`, `io_uring`, `mmap` im Leseweg | alternative `Fs` |
| NZ-10 | Kompression in v1 (Feld reserviert) | spätere Formatentscheidung |
| NZ-11 | Inhaltsadressierung, Dedup von Nutzdaten | obere Schicht (O-5) |
| NZ-12 | Tokenizer, Embeddings, ANN, Volltext, Graph | obere Schicht über §18 |
| NZ-13 | Prolly-Tree, Commit-DAG, Branches Stufe B | obere Schicht (Anhang D) |
| NZ-14 | Lösch-Orchestrierung, Berichte, Signaturen | obere Schicht |
| NZ-15 | Schlüsselverwaltung | `KeyProvider` |
| NZ-16 | Mandanten-, Rechte-, Lizenzprüfung | obere Schicht |
| NZ-17 | Async-Fassade | obere Schicht |
| NZ-18 | Benannte Checkpoints über Komponenten | obere Schicht |

**Angreifer**

| ID | Angreifer |
|---|---|
| AT-1 | Prozess mit Lesezugriff auf das Datenverzeichnis, ohne Schlüssel. |
| AT-2 | Besitzer des Datenträgers, ohne Schlüssel. |
| AT-3 | Dateimanipulation: Einfügen, Abschneiden, Vertauschen. |
| AT-4 | Fehlerhafte oder bösartige obere Schicht. |
| AT-5 | **Rollback:** Zurückspielen eines früheren, konsistenten Verzeichnisstands, um gelöschte Daten zurückzubringen. |

**Annahmen**

| ID | Annahme |
|---|---|
| AN-01 | `sync_data`/`sync_all`/`sync_dir` machen Änderungen durabel (POSIX). |
| AN-02 | Ein Schreibvorgang kann zerreißen; alle Rahmen sind prüfsummengeschützt. |
| AN-03 | Ein Fsync-Fehler ist **nicht** wiederholbar. |
| AN-04 | Verzeichnis-`rename` ist atomar. |
| AN-05 | Prozessspeicher ist vertrauenswürdig und wird nicht kontrolliert gelöscht (§17.9). |
| AN-06 | `KeyProvider::destroy` ist durabel bei Rückkehr. |
| AN-07 | Zufall aus `Entropy` ist kryptografisch stark. |
| AN-08 | Der `Anchor` liegt außerhalb des Datenverzeichnisses und ist für AT-3/AT-5 nicht beschreibbar. Sonst gilt §17.10. |
| AN-09 | Die Systemuhr darf rückwärts laufen; sie dient nur Fristen und Ablauf, nie Stamps. |

---

## 5. Ports (`contextra-storage-ports`, semver-stabil)

| Port | Aufgabe | Vertrag |
|---|---|---|
| `Fs` | Dateien, Verzeichnisse, `read_at`, `append`, `preallocate`, `sync_data`, `sync_all`, `sync_dir`, `rename`, `lock`/`unlock`, `list`, optional `link`, `reflink`, `punch_hole` | Fehlerklassen §6, `Perm::Private` |
| `Clock` | monotone Zeit, Wanduhr | injizierbar; nie für Stamps |
| `Entropy` | kryptografischer Zufall | injizierbar |
| `Anchor` | monotoner Rollback-Anker: `read()`, `advance(to)`, durabel bei Rückkehr | §17.10; `AnchorValue = (erase_epoch u64, ledger_head [u8;32], catalog_epoch u64)` |
| `KeyProvider` | KEK, Clusterschlüssel, `destroy`, `probe` | §17.2 |
| `Checksum` | CRC32C | Slice-by-16 Standard, Hardware austauschbar |
| `Cipher`/`Mac`/`Kdf` | AEAD, Kettenprüfung, Ableitung | §17.1 |
| `Metrics` | Zähler, Histogramme | nicht blockierend |
| `Observer` | Ereignisse (Flush, Frist, Poison, Restricted-Read, Anker) | nie unter Sperren |
| `ColdTier` (optional) | Auslagerung kalter Dateien | Garantien des Ports gelten |

---

## 6. Fehlermodell

| Klasse | Beispiele | Verhalten |
|---|---|---|
| F-1 transient | `EINTR`, Überlast | Wiederholung mit Backoff, begrenzt |
| F-2 Ressource | `ENOSPC`, Handle-Limit | Rückdruck, dann **Poison** des Schreibpfads; Lesen bleibt |
| F-3 E/A-hart | `EIO`, Fsync-Fehler | **nie wiederholen**, Poison, Wiederanlauf nur über Neustart/Recovery |
| F-4 Integrität | CRC/MAC-Fehler | kleinste isolierbare Einheit wird vergiftet (Block → Abschnitt → Datei → Space), `Err(Corrupt{..})`, Rest läuft |
| F-5 Format | `Unsupported`, unbekannter Tag | Öffnen verweigert, nie ignorieren (`INV-FMT-1`) |
| F-6 Schlüssel | Provider offline, Schlüssel fehlt | `LockedKeys` bzw. `Shredded` |
| F-7 Aufrufer | Guard verletzt, Cluster gesperrt, Limit, `Restricted` | typisierte Fehler mit `hint()` |
| F-8 Logik | Invariante verletzt (Bug) | Test: Abbruch. Produktion: Poison + Observer. **Kein `panic`** |
| F-9 Rollback | Anker höher als Catalog | Öffnen schreibend verweigert (`Err(Rollback)`), nur `ReadOnly` mit Diagnose (§15.1 R2a) |

Jeder Fehler hat `class()` und `hint()`. Fehlertexte enthalten **keine** Schlüssel, Nutzerschlüssel, Körper oder Klartextfragmente.

---

## 7. Datenmodell

### 7.1 Vokabular

Im Crate kommen weder Produkt- noch Fremdsystemnamen vor.

| Begriff | Bedeutung |
|---|---|
| **Storage** | Handle einer geöffneten Instanz |
| **Space** | Benannter, physisch getrennter LSM-Baum (`SpaceId(u16)`), dauerhaft oder `Volatile` (§7.4) |
| **Cluster** | Löschdomäne, Schlüsseldomäne und Platzierungseinheit (`ClusterId`, 16 Byte, opak). `ClusterId::NONE` = „kein Cluster“. Nur über `Clusters::register` angelegt. |
| **Packgruppe** | Cluster gleichen Mandanten-Tags, Ablaufzeitraums und Shards: `PackKey = (tenant_tag, due_bucket, shard)` (K-09). Packed-Dateien enthalten nur **eine** Packgruppe. |
| **Change / Guard / Receipt** | Atomare Menge von Schreibvorgängen / Vorbedingung / Quittung `(stamp, durable)` |
| **Stamp** | Monotone Zählung angenommener Changes (`u64` < 2^56); entspricht der Transaktionszeit |
| **View / Cursor / Page** | Unveränderliche Sicht / Streaming-Iterator / budgetierte Seite mit `ResumeToken` |
| **Hold** | Befristete **technische** Halteregel gegen Verwerfen alter Versionen |
| **Restriction** | **Rechtlicher** Sperrvermerk mit Pflichtgrund (§16.5) |
| **Consumer** | Persistenter Fortschrittszeiger, zählt wie Hold, mit Maximalrückstand |
| **Redo-Log** | Segmentiertes Wiederholungsprotokoll (nur dauerhafte Spaces) |
| **Run / Blob** | Unveränderliche sortierte Datei mit Köpfen / mit Wertkörpern |
| **Header / Body / Weights** | Erste `header_len` Wertbytes / Rest / Zählgrößen im Klartext |
| **Sealed** | Mit Einmalschlüssel aus dem Clusterschlüssel versiegelt (§17.3) |
| **Retired** | Cluster, dessen Löschung logisch erfolgt ist (Stamp `S`) |
| **Pure / Packed** | Datei mit einem / mehreren Clustern einer Packgruppe (§8) |
| **Segment** | Vom Catalog verwaltete Datei einer oberen Schicht (§18.1) |
| **Claim / Graveyard** | Reservierung von Dateien durch einen Job / Warteschlange zu löschender Dateien |
| **Catalog** | Einzige Wahrheit über lebendige Dateien und Zustand |
| **Anchor** | Monotoner Rollback-Anker außerhalb des Datenverzeichnisses (§17.10) |
| **Env** | Bündel der Ports |

### 7.2 Cluster, Schlüssel, Reihenfolge, Stamp

- **Nutzerschlüssel:** Bytes, `1 ≤ Länge ≤ max_key_bytes`. In versiegelten Clustern geschützt (L3), sonst MUSS er frei von personenbezogenen Daten sein (§17.9).
- **Vergleich** (`cmp`, **kein** `memcmp` über Konkatenation): Cluster aufsteigend, dann Nutzerschlüssel aufsteigend (bytelexikografisch), dann **Tag absteigend**. Ohne `clustered` ist der Cluster implizit `NONE` und wird nicht gespeichert.
- **Tag** `u64 = (stamp << 8) | kind`. `kind`: `Put = 1`, `Delete = 2`, `RangeDelete = 3`, `EraseRange = 4`, `PutRef = 5`, `Merge = 6` (**reserviert**, Decoder meldet `Unsupported`).
- `PutRef` kommt **nur in Runs** vor. Redo und Memtable kennen `Put`, `Delete`, `RangeDelete`, `EraseRange`.
- **Stamp:** erster Stamp 1, `0` = leer. Streng monoton über **angenommene** Changes (`INV-STAMP-1`), nie doppelt vergeben, auch nicht nach Absturz (`INV-STAMP-3`, §12.3). Lücken zulässig. Überlauf ⇒ `Exhausted` (`INV-STAMP-2`).
- **Bereichsmarker** (`RangeDelete`, `EraseRange`): `(cluster, start, end, tag)`, `end` exklusiv. Flags `UNBOUNDED_END`, `WHOLE_CLUSTER`. Ein Marker mit Stamp `S` verdeckt für jede Sicht ≥ `S` alle Versionen mit Stamp < `S` im Bereich.

### 7.3 Wert, Kopf, Körper, Gewichte

Ein Wert besteht aus `header` und `body`, getrennt durch `header_len`. **Der Kern interpretiert den Kopf nie.** Er liefert ihn ohne Körper-E/A. Jeder Eintrag trägt `weights: [u32; W]` (`W` = `weight_slots` des Space, 0…4). Was ein Gewicht bedeutet (z. B. Token), bestimmt die obere Schicht. Der Block-Index hält `weight_sum[W]` und `entry_count`; `page` überspringt Blöcke ohne Dekomprimierung.

Körper ≥ `value_separation_min` (1 KiB) gehen in Blobs (`PutRef`), kleinere bleiben inline. Der Kopf liegt **immer** inline.

### 7.4 `SpaceOptions` (unveränderlich nach dem Anlegen)

```rust
pub struct SpaceOptions {
    pub durability_class: SpaceClass,       // Durable | Volatile (SD-26)
    pub clustered: bool,                    // true: Ordnung und Abschnitte nach Cluster (L2)
    pub packing: PackPolicy,                // None | ByTenantAndExpiry { bucket: Month|Quarter|Year }; nur bei clustered
    pub pure_threshold: u64,                // Cluster ≥ Schwelle wird bei Compaction Pure (Standard 8 MiB)
    pub seal: Seal,                         // None | Cluster (nur clustered + Encrypted + Durable)
    pub weight_slots: u8,                   // 0..=4
    pub history: HistoryPolicy,             // KeepLatest | KeepUntilFloor | AppendOnly (Erase nur mit Admin)
    pub value_separation_min: u32,
    pub max_key_bytes: u32,
    pub compression: Compression,           // v1: nur None (NZ-10)
    pub durability_default: Durability,     // Durable | Relaxed; für Volatile ohne Wirkung
}
```

`integrity_mode` steht **global** in `FORMAT`. `EraseLevel::Cryptographic` ist nur in Spaces mit `Seal::Cluster` möglich, sonst `Err(Unsupported)`.

**Volatile-Spaces (SD-26):** kein Redo, keine Runs, keine Blobs; im Catalog nur der Name. Inhalt verschwindet bei Schließen und Absturz. Gleiche Sichtsemantik (Stamps, `View`, `Guard`), aber sofort sichtbar nach Einfügen. `Erase` = Speicherfreigabe mit Überschreiben (best-effort). Ein Change mit Volatile- **und** Durable-Zielen ist über Abstürze hinweg **nicht** atomar und wird nur `durable`, wenn alle Ziele Durable sind (INV-VOL-1). `apply` weist solche Mischungen nur mit ausdrücklichem Flag `allow_mixed` an.

### 7.5 Cluster-Tabelle (Catalog)

Je Cluster: `{ cluster_id, tenant_tag, pack_key, state, key_epoch, created_stamp, due_bucket, links, restrictions[] }`.

`state`: `Live → Retired(S) → [Shredded] → Purged` (nur vorwärts). `Shredded` entfällt bei Stufen ohne Schlüsselvernichtung (K-24). `Restricted` ist **kein** Zustand, sondern eine Menge `restrictions` (§16.5). Ein `Purged`-Cluster wird nie wiederverwendet (`INV-CLUSTER-1`).

### 7.6 Limits

| Limit | Standard | Obergrenze | Bei Überschreitung |
|---|---|---|---|
| `max_key_bytes` | 1 KiB | 4 KiB | `Limit` |
| `max_header_bytes` | 4 KiB | 64 KiB | `Limit` |
| `max_body_bytes` | 16 MiB | 256 MiB | `Limit` (kein Stückeln) |
| `max_change_ops` | 4096 | 65536 | `ChangeTooLarge` (INV-LIM-1) |
| `max_change_bytes` | 8 MiB | 64 MiB | `ChangeTooLarge` (INV-LIM-1) |
| `max_page_bytes` | 1 MiB | 16 MiB | `Limit` |

Ein Change über den Limits wird **vor** jeder Sperre abgewiesen (§12.1 Schritt 1).

---
## 8. Platzierung: Pure und Packed

### 8.1 Regeln

- **R-PL-1** Jeder Run, Blob und jedes Segment ist **Pure** (ein Cluster) oder **Packed** (mehrere Cluster derselben Packgruppe), nie beides (`INV-PL-1`).
- **R-PL-2** In einer Packed-Datei bildet jeder Cluster **einen zusammenhängenden Abschnitt** (`ClusterSection`), sortiert nach `ClusterId`. Blöcke sind clusterrein (`INV-PL-2`).
- **R-PL-3** Überschreiten die lebenden Bytes eines Clusters `pure_threshold`, wird er bei der nächsten Compaction **Pure**. Der umgekehrte Weg findet nicht statt.
- **R-PL-4** Compaction mischt nur innerhalb **einer Wartungsgruppe**: einer Packgruppe oder eines Pure-Clusters (`INV-PL-3`).
- **R-PL-5** `PackKey = (tenant_tag, due_bucket, shard)`. Packgruppen je Space ≤ `max_pack_groups` (4096).
- **R-PL-6** Der Wechsel in eine andere Packgruppe (`rebucket`) ist ein atomarer Umzug (§17.8).

### 8.2 Wirkung auf die Löschung

| Fall | Pure | Packed |
|---|---|---|
| Cluster löschen | Dateien **entfernen**, ein Edit | Abschnitt **überspringen**, übrige Abschnitte **roh kopieren** (kein Entschlüsseln, kein Neukomprimieren), neues Verzeichnis, ein Edit |
| Packgruppe läuft ab | alle Packed-Dateien der Gruppe entfernen, ein Edit | |
| Mandant verlässt | alle Pure-Cluster und Packgruppen des Tags entfernen | |

Die AAD der Blöcke ist **nicht** an die Dateiposition gebunden (§17.3); darum ist Rohkopieren gültig.

---

## 9. Öffentliche API

Normativ in Form und Semantik, Lebensdauern offen.

```rust
// ---- Öffnen und Struktur ---------------------------------------------------------
pub struct Storage { /* Arc<Inner> */ }
impl Storage {
    /// LOCK, Anker-Prüfung (§15.1 R2a), Recovery. Mode ReadWrite | ReadOnly(shared).
    pub fn open(cfg: Config, env: Env) -> Result<Storage, OpenError>;
    pub fn space(&self, name: &str) -> Result<Space, Error>;
    pub fn spaces_add(&self, name: &str, o: SpaceOptions) -> Result<Space, Error>;
    pub fn close(self) -> Result<CloseReport, Error>;
}
pub struct Config { pub dir: PathBuf, pub integrity: IntegrityMode, pub anchor: AnchorPolicy, pub limits: Limits }
pub enum AnchorPolicy { Required, Optional { acknowledged_residual_risk: bool } } // Standard: Required bei Encrypted/Authenticated

// ---- Schreiben: ein Change über viele Spaces, atomar ----------------------------
pub struct Change { /* Ops + Guards */ }
impl Change {
    pub fn new() -> Change;
    pub fn put(&mut self, sp: &Space, c: ClusterId, key: &[u8], v: NewValue) -> &mut Self;
    pub fn delete(&mut self, sp: &Space, c: ClusterId, key: &[u8]) -> &mut Self;
    pub fn delete_range(&mut self, sp: &Space, c: ClusterId, r: KeyRange) -> &mut Self;
    pub fn guard(&mut self, sp: &Space, c: ClusterId, key: &[u8], g: Precondition) -> &mut Self; // Absent | Version(Stamp) | Unchanged
    pub fn idempotency(&mut self, t: IdemToken) -> &mut Self;     // Wiederholung → AlreadyApplied { stamp }
}
pub struct NewValue { pub header: Bytes, pub body: Body, pub weights: [u32; 4] }
impl Storage {
    pub fn apply(&self, c: Change, d: Durability) -> Result<Receipt, ApplyError>;
    // ApplyError: GuardFailed, AlreadyApplied, ClusterRetired, Restricted, ChangeTooLarge,
    //             Backpressure(Retry), Poisoned, Indeterminate, Exhausted, Unsupported, ...
}

// ---- Lesen --------------------------------------------------------------------------
impl Storage { pub fn view(&self) -> View; pub fn view_at(&self, s: Stamp) -> Result<View, Error>; }
impl View {
    pub fn stamp(&self) -> Stamp;
    pub fn get_header(&self, sp: &Space, c: ClusterId, key: &[u8]) -> Result<Option<HeaderRef>, ReadError>;
    pub fn get(&self, sp: &Space, c: ClusterId, key: &[u8]) -> Result<Option<EntryRef>, ReadError>; // Körper lazy
    pub fn cursor(&self, sp: &Space, c: ClusterId, r: KeyRange, o: ScanOpts) -> Cursor;
    pub fn page(&self, sp: &Space, c: ClusterId, r: KeyRange, budget: Budget, resume: Option<ResumeToken>)
        -> Result<Page, ReadError>;                                  // weight_sum, überspringt Blöcke
    pub fn history(&self, sp: &Space, c: ClusterId, key: &[u8]) -> History;
}
impl EntryRef { pub fn header(&self) -> &[u8]; pub fn weights(&self) -> &[u32];
                pub fn body(&self) -> Result<Body, BodyError>; }     // Err(Shredded | Erased{..}), nie Daten, nie leer

// ---- Cluster und Verknüpfung ------------------------------------------------------------
impl Storage { pub fn clusters(&self) -> Clusters; }
impl Clusters {
    pub fn register(&self, spec: ClusterSpec) -> Result<ClusterId, Error>;
    pub fn link(&self, derived: ClusterId, sources: &[ClusterId]) -> Result<(), Error>; // ≤ max_links, azyklisch
    pub fn rebucket(&self, c: ClusterId, new_due: Due) -> Result<RebucketReport, Error>;
}

// ---- Holds, Restrictions, Consumer ---------------------------------------------------------
impl Storage {
    pub fn hold(&self, at: Stamp, owner: OwnerId, ttl: Duration) -> Result<Hold, Error>;  // ttl ≤ max_hold_ttl
    pub fn restrict(&self, c: ClusterId, r: RestrictionSpec) -> Result<RestrictionId, Error>;
    pub fn lift(&self, id: RestrictionId, by: OwnerId) -> Result<(), Error>;
    pub fn consumer(&self, id: ConsumerId, from: Stamp, max_lag: u64) -> Result<Consumer, Error>;
}
impl Consumer { pub fn next(&mut self, max: usize) -> Result<Changes, Error>; pub fn ack(&mut self, upto: Stamp) -> Result<(), Error>; }

// ---- Löschung --------------------------------------------------------------------------------
impl Storage { pub fn eraser(&self) -> Eraser; }
impl Eraser {
    pub fn cluster(&self, c: ClusterId, req: EraseReq) -> Result<Ticket, EraseError>;   // ER0–ER1 synchron, Rest asynchron
    pub fn clusters_batch(&self, v: &[ClusterId], req: EraseReq) -> Result<Ticket, EraseError>;
    pub fn pack_expired(&self, k: PackKey, req: EraseReq) -> Result<Ticket, EraseError>;
    pub fn range(&self, sp: &Space, c: ClusterId, r: KeyRange, req: EraseReq) -> Result<Ticket, EraseError>;
    pub fn status(&self, t: &Ticket) -> EraseStatus;     // Retired · KeyDestroyed · Purging · Done(ErasureReceipt)
    pub fn wait(&self, t: &Ticket, until: EraseStage, timeout: Duration) -> Result<EraseStatus, Error>;
    pub fn verify(&self, t: &Ticket) -> VerifyReport;
}
pub struct EraseReq {
    pub level: EraseLevel,              // Logical | Physical | Scrubbed | Cryptographic
    pub views: ViewPolicy,              // Wait(timeout) | Revoke
    pub purge_deadline: Duration,       // PFLICHT, kein Standardwert (SD-06)
}

// ---- Checkpoint, Betrieb, Prüfung ---------------------------------------------------------
impl Storage {
    pub fn checkpoint(&self, dst: &Path, m: CheckpointMode) -> Result<Manifest, Error>;  // Copy | Reflink | Hardlink(Opt-in)
    pub fn restore_check(&self, m: &Manifest, ledger: &LedgerRef) -> RestorePlan;
    pub fn maintenance(&self) -> Maintenance;
    pub fn inspect(&self) -> Inspector;
}
```

**Berechtigungen (K-25):** Der Kern kennt keine Benutzer. Er kennt nur zwei **nicht klonbare Fähigkeiten** (`AdminCap`, `RestrictedReadCap`), die `Storage::open` an den Eigentümer des Handles ausgibt. Wo diese Spec „Admin-Recht“ sagt, ist `AdminCap` gemeint. Sie wird nie persistiert.

**Eigenschaften**

- Kein Aufruf blockiert unbegrenzt; Wartefunktionen haben Timeouts, Rückdruck ist `Backpressure(Retry)` mit Hinweis.
- `apply` ist über `IdemToken` idempotent (§12.5).
- `View` ist `Send + Sync`, billig klonbar, hält keine Sperre.
- `body()` unterscheidet `Shredded` von `Erased`. Beides liefert nie Daten. Diesen Unterschied verbirgt die obere Schicht vor Nutzern ohne Recht.
- Bei `Indeterminate` (Fsync-Ergebnis unbekannt) ist der Change nach Neustart vorhanden oder nicht. Der Aufrufer wiederholt mit demselben `IdemToken`.

---

## 10. Dateilayout und Dateilebenszyklus

### 10.1 Layout

```
<root>/
  FORMAT              Magic, Version, Feature-Bits, integrity_mode, instance_id (§11.2)
  LOCK                exklusive (oder geteilte, read-only) Sperre über Fs::lock
  CURRENT             Zeiger auf aktive Catalog-Datei (atomar ersetzt)
  CATALOG-000007      Snapshot-Record + Edit-Records (§11.8)
  KEYS                (Authenticated/Encrypted) gesiegelte Space-Schlüssel (§17.2), atomar ersetzt
  ERASE-LEDGER/       append-only Löschprotokoll, kettenverknüpft (§17.8)
  redo/redo-0000000000000012.log     vorallokiert, am Ende gesiegelt (§11.4)
  spaces/0001/…       0000000000000a3f.run, 0000000000000071.blob
  segments/<kind:04x>/<id:016x>.seg  Dateien oberer Schichten (§18.1)
```

Der **Anker** liegt nicht im Verzeichnis, sondern hinter dem Port `Anchor` (§17.10).

### 10.2 Regeln

- **Numerische IDs** benennen alle Dateien, nie Pfad-Strings im Catalog.
- **Blob-Inkarnation:** `BlobRef = (blob_id, offset, len)`. Ein Blob wird nie verändert. Entfällt ein Cluster-Abschnitt, entsteht eine **neue Inkarnation** derselben `blob_id` (`<id>.<inc>.blob`) mit **gleichen Offsets**, entfallene Bereiche sind Löcher (Nullen, sparse bei `punch_hole`). Der Catalog tauscht atomar (`blob_replace`). Der Umzug in eine **neue** `blob_id` ist nur Blob-GC (§14.3).
- **Blob-Lebendigkeit (K-05):** Ein Blob oder eine Inkarnation wird nur entfernt, wenn **kein lebender Run** mehr einen `PutRef` darauf enthält (Abgleich der lebenden Runs). `blob_live_delta` dient nur der Auswahl von GC-Kandidaten (`INV-BLOB-1`).
- **Redo-Segmente sind kein `FileKind`**: lebendig ⇔ `segment_id ≥ redo_floor` (auch das Reservesegment, `INV-LOG-7`). `FileKind` kennt `Run`, `Blob`, `Segment(kind)`.
- Alle Dateien und Verzeichnisse entstehen mit `Perm::Private`.
- **Temporäre Dateien** `.building.<tmp_id>` (`tmp_id` aus dem im Catalog reservierten Bereich); beim Öffnen entfernt.
- **Verwaiste Dateien** (nicht im Catalog) werden nach R8 und periodisch gelöscht; sicher, weil nur ein Edit eine Datei lebendig macht (`INV-CAT-1`).
- **Ein Space = ein Verzeichnis.** Space entfernen = Edit + Dateien löschen + `remove_dir`.
- **Dateien sind unveränderlich**, sobald lebendig (Ausnahme: Redo-Segment bis zum Siegel).

### 10.3 Zustandsdiagramm einer Datei

```
(nicht vorhanden) ─create .building─► im Bau ─sync_all + rename + sync_dir─► dauerhaft, UNREFERENZIERT
      ▲                                                                          │ Edit(add) dauerhaft
      │ Verwaisten-Suche                                                         ▼
      └─ Unlink + sync_dir ◄── Graveyard ◄── obsolet ◄──────────────────── LEBENDIG
         (nach letzter Referenz)                       ▲ Edit(remove) dauerhaft
```

Jeder Wechsel zwischen „unreferenziert“ und „lebendig“ ist **ein** Catalog-Edit (P-2).

---

## 11. Formate

### 11.1 Konventionen

- Alle Zahlen **little-endian**. `varint` = LEB128 für `u64` (≤ 10 Byte), **nur kanonisch** (zehntes Byte nur `0x01`, keine überflüssigen Fortsetzungen). Sonst `Corrupt` (`INV-FMT-1`).
- Jede Dateiart hat **genau eine Version (`1`)** und Feature-Bits (SD-04). Unbekannte Version oder Pflicht-Bit ⇒ `Unsupported`.
- Dateikopf: `magic[8] ‖ file_kind u8 ‖ format_ver u16 ‖ feature_bits u32 ‖ file_id u64 ‖ instance_id[16] ‖ header_crc u32`.
- Längenfelder werden gegen Grenzen geprüft, **bevor** allokiert wird (P-4). Reservierte Felder werden mit 0 geschrieben und beim Lesen auf 0 geprüft (P-10).
- CRC32C gegen Zufallsfehler; in `Authenticated`/`Encrypted` zusätzlich Tag (§17.1).

### 11.2 `FORMAT`

`magic ‖ format_ver ‖ feature_bits ‖ integrity_mode u8 ‖ algo_ids (aead, mac, kdf) ‖ instance_id ‖ limits ‖ crc`. Unveränderlich. In `Authenticated`/`Encrypted` zusätzlich mit KEK-abgeleitetem MAC geschützt.

### 11.3 `KEYS`

Umhüllte **Space-Schlüssel** (je Space ein DEK, mit KEK umhüllt, atomar ersetzt), höchstens `max_spaces` (≤ 4096). **Clusterschlüssel stehen nicht hier** (§17.2).

### 11.4 Redo-Segment und Frame

Segment: vorallokiert (`preallocate`; ist `prealloc_zeroed` falsch, füllt der Kern mit Nullen und nutzt `sync_all`), `header ‖ Gruppe*`, am Ende **gesiegelt**. Eine **Gruppe** beginnt an einer `redo_align`-Grenze (4096) und enthält einen oder mehrere **Frames**. Torn-Tail-Regel (`INV-LOG-8`): Der erste ungültige Gruppenkopf beendet das Log nur, wenn danach im Segmentrest keine prüfbar gültige Gruppe folgt; sonst `Corrupt{RedoMidLog}`. Alles bis zur letzten `redo_mark` MUSS gültig sein.

```
Frame := head ‖ ops ‖ tail
head (Klartext, Ketten-MAC-geschützt):
      len u32 | stamp u64 | kind u8 | n_ops u16 | idem_token[16] | epoch u64 | flags u8 |
      ops_head[]: { space u16, cluster[16], op_kind u8, op_uid[16], payload_len u32 }
ops  : je Op die Nutzlast
       - Seal=Cluster: GESAMTE Nutzlast (Schlüssel, Kopf, Körper, Gewichte) mit Clusterschlüssel versiegelt (§17.3)
       - sonst: unter Redo-Schlüssel (Encrypted) bzw. Klartext (Plain/Authenticated)
tail : crc32c | chain_mac    (H_i = MAC(H_{i-1} ‖ head ‖ payload_tags))
```

`kind` des Frames: `Change`, `ClusterGate` (ER1, §17.5), `Reserve` (Hochwassermarke). `ClusterGate` trägt `(Clustermenge H, S, ticket, level)` im Klartext, damit Recovery ohne Schlüssel `Retired` ableitet (K-02). Ein Gate-Frame beginnt immer ein **neues Segment** (Segmentschnitt, `INV-LOG-4`): das vorige Segment wird vorher gesiegelt.

`op_uid` ist ein vom Aufrufer in `apply` gezogener Zufallswert (K-24): Die Versiegelung geschieht **vor** der Stamp-Vergabe und darf den Stamp daher nicht in der AAD tragen; der Stamp ist über die Ketten-MAC des Frames gebunden.

Folgen: Die Kette erkennt Abschneiden und Einfügen. Stamp, Space, Cluster und Frame-Typ stehen im Klartext; Recovery und Löschplanung brauchen **keine** Schlüssel. Nach Vernichtung des Clusterschlüssels bleibt die Nutzlast unlesbar; Replay erzeugt einen `ErasedMarker`.

### 11.5 Run

```
Run := header
       ClusterSection*          (aufsteigend nach ClusterId; Pure: genau eine)
       ClusterDirectory         (Klartext: je Cluster { cluster, section_off, section_len, n_blocks, min/max_stamp, weight_sum[W] })
       PackMeta                 (space_id, pack_key | pure_cluster, Blob-Bereich, Eintragszahlen)
       ExtensionBlock*          (§18.3)
       Footer                   (Offsets, Prüfsumme, Kette)

ClusterSection := DataBlock* ‖ SubIndexBlock ‖ FilterBlock ‖ RangeDelBlock
   DataBlock       Ziel 16 KiB, Präfixkompression mit Restart-Punkten, dann Seal (Clusterschlüssel) oder Block-AEAD (Space-Schlüssel)
   SubIndexBlock   je Datenblock: letzter Schlüssel, Offset, Länge, entries, weight_sum[W], min/max_stamp, block_uid[16]
   FilterBlock     Schlüsselfilter dieses Clusters (Ribbon oder Bloom, keyed, clustergebunden)
   RangeDelBlock   Bereichsmarker des Clusters
```

- **Seal=Cluster:** SubIndex, Filter, RangeDel und alle DataBlocks sind versiegelt. Klartext bleiben `ClusterDirectory`, Cluster-ID (opak), Zähler, Größen, Stamp-Bereiche.
- Verzeichnisse geöffneter Runs liegen im Speicher (etwa 48 Byte je Cluster und Datei, als Hypothese H-12 zu messen). Vorauswahl über Packgruppen-Filter im Catalog.
- Packed-Runs enthalten Körper nur inline bis `max_inline_bytes`; größere erzwingen Pure.

### 11.6 Blob

```
Blob := header ‖ ClusterSection* ‖ ClusterDirectory ‖ footer
ClusterSection := Record*
Record := len u32 | nonce[24] | sealed_body | crc32c
```

**AAD** des Records: `(space, cluster, blob_id, offset)`; Records bleiben bei Inkarnation mit gleichen Offsets gültig, ein GC-Umzug **versiegelt neu**. Ziel ≤ 64 MiB, ein Blob gehört zu einem Space und ist Pure oder Packed.

### 11.7 Segment (obere Schicht)

`header (kind u16, id u64, space, pack_key|cluster, cover) ‖ opake Nutzlast ‖ footer (crc, Kette)`. Vertrag §18.1.

### 11.8 Catalog

`Snapshot-Record` (Vollzustand) gefolgt von `Edit-Records`. Jeder Edit ist atomar (ein Append + `sync_data`) und trägt `seq`, `catalog_epoch`, Kette, CRC. Rollover bei `catalog_roll_bytes` (8 MiB): neue Datei mit Snapshot, `CURRENT` per atomarem `rename`.

Ops: `space_add | space_drop | cluster_register | cluster_state(c, state, stamp) | cluster_link | restriction_set | restriction_lift | pack_assign | files_add(kind, id, group|cluster, meta) | files_remove | blob_replace(blob_id, inc) | blob_live_delta | redo_floor | redo_mark | segment_add | segment_remove | hold_set | consumer_ack | erase_pending_add(ticket) | erase_done(ticket) | erased_ranges_add | key_epoch | idem_trim | reserve_stamps(hw) | reserve_ids(kind, hw) | anchor_mark(epoch)`. Reservierter Tag-Bereich `0xF000…0xFFFF`.

- **Reservierungen:** `reserve_stamps(hw)`, `reserve_ids` heben eine Hochwassermarke **vorab** an (2^20 Stamps, 2^16 IDs). Nach Absturz beginnt die Vergabe oberhalb der Marke. Nachreservieren auf einem Wartungsthread bei < 50 % Rest. Fallback im Committer: ein synchroner Edit, **einzige** Doppelhaltung von Rang 1 und Rang 3 neben dem Gate (§19.2).
- Unbekannte Tags ⇒ `Unsupported`.
- **Regel (L6):** lebendig ⇔ durabler `files_add` ohne `files_remove`.

### 11.9 Freeze-Liste (Gate G-F)

Eingefroren: Dateikopf, `FORMAT`, `KEYS`, Redo-Gruppe, Frame (mit `kind`, `epoch`, `op_uid`), Kette, ClusterSection, ClusterDirectory, SubIndex-Felder, Seal-Rahmen (Nonce, AAD-Regeln), Blob-Record und Inkarnationsregel, Catalog-Op-Tags (mit reserviertem Bereich), `anchor_mark`, `AnchorValue`, Ledger-Format, `cmp`, Tag-Kodierung. Zu jedem Format: unabhängiger Referenzcodierer (Python) und goldene Vektoren (Anhang A). Nach G-F ist jede Änderung eine Migration.

---

## 12. Schreibpfad

### 12.1 Ablauf von `apply`

```
1  Vorprüfung ohne Sperre: Größen (ChangeTooLarge), Space/Cluster bekannt,
   Idempotenz-Cache (IdemToken bekannt ⇒ AlreadyApplied)
2  Vorbereiten auf dem Aufruferthread: Ops kodieren, je Op op_uid ziehen, Nutzlast versiegeln (Seal=Cluster)
3  Stamp-Reserve prüfen; nötigenfalls vorher reserve_stamps (vor Rang 1)
4  Intake (Rang 2), Warten auf Gruppenführer
5  Gruppenführer = Committer (Rang 1) nimmt alle Wartenden in Ankunftsreihenfolge:
     a) Cluster-Zustand prüfen: Retired/Shredded/Purged ⇒ ClusterRetired; BLOCK_WRITE ⇒ Restricted
     b) Guards prüfen gegen Memtable ∪ Version ∪ Gruppen-Overlay (Schreibvorgänge früherer Changes derselben Gruppe)
        ⇒ fehlgeschlagene Changes scheiden aus
     c) Stamps vergeben (aus der reservierten Marke)
     d) Gruppe bauen: ein Frame je Change, Kette/MAC
     e) ein write + ein sync_data je Gruppe                              ◄── DURABILITÄTSPUNKT
6  Als Committer: Memtable einfügen (lock-frei, K-01), Version veröffentlichen (Rang 4), visible_stamp anheben (Release)
7  Rückkehr an alle Wartenden: Receipt { stamp, durable }
```

- **Sichtbarkeit:** nur nach Durabilität. Ausnahme `Relaxed` (Opt-in je Aufruf, bei Absturz verlierbar, für Löschnachweise unzulässig). Volatile-Ziele werden in Schritt 6 ohne 5e veröffentlicht.
- **Sync-Fehler:** `sync_data` scheitert ⇒ Poison (F-3). Alle Changes der Gruppe erhalten `Indeterminate`, alle späteren `Poisoned`. Lesen bleibt.
- **Memtable-Wechsel** führt nur der Committer an Gruppengrenzen aus.
- **Rückdruck:** Memtable-Größe, ungeflushte Memtables, L0-Dateien, Redo-Rückstand wirken stufenweise: Warten **außerhalb** jeder Sperre (vor dem Intake), dann `Backpressure(Retry)`. Warten unter Rang 1 ist verboten.
- **Redo-Lebenszyklus:** `Open → Sealed → Obsolete (< redo_floor) → Graveyard`. Ein Reservesegment liegt stets bereit (`INV-LOG-7`). Der Scheduler erzwingt Flush, wenn das älteste Segment `redo_max_age` (1 h) überschreitet.
- **ENOSPC/EIO:** Poison (F-2/F-3), Lesen bleibt.

### 12.2 Memtable

Je Space eine aktive, lock-freie Memtable in `cmp`-Ordnung (je Cluster zusammenhängend), dazu höchstens `max_immutable` (3) eingefrorene. Größen: `memtable_bytes` je Space (64 MiB) und `memtable_total_bytes` je Instanz (harte Obergrenze). Sie hält **Klartext** (AN-05). Überschriebene Knoten werden erst nach Epochen-Reklamation freigegeben (Restexposition §17.9).

### 12.3 Stamps und Reservierung

Der Committer vergibt Stamps lückenlos je Gruppe aus der Marke. Nach Absturz: `next_stamp = max(stamp_limit, letzter gültiger Redo-Stamp, max Run-Stamp) + 1`. Nicht genutzte Stamps verfallen; zulässig, weil `INV-STAMP-1` nur **angenommene** Changes betrifft. Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` (`INV-CONS-1`).

### 12.4 Flush

Je Space: Memtable einfrieren → je Cluster ein Abschnitt (Pure ab `pure_threshold`) → Run schreiben (`.building` → `sync_all` → `rename` → `sync_dir`) → **ein** Edit (`files_add`, `redo_floor`, `anchor_mark`) → neue Version. Absturz vor dem Edit: Run verwaist, Redo spielt erneut ein. `redo_floor` rückt nur auf Segmente vor, deren Frames komplett in durablen Runs stehen **und** deren Gate-Frames im Catalog als `erase_pending` stehen (`INV-LOG-5`).

### 12.5 Idempotenz

`IdemToken` liegt im Frame-Kopf und in einer begrenzten Token-Tabelle (TTL `idem_ttl`, `idem_trim` im Catalog). Die Tabelle enthält **keine Inhalte und keine Cluster-Kennungen**. Wiederholt ein Aufrufer einen durablen Change, antwortet `apply` mit `AlreadyApplied { stamp }`.

---

## 13. Lesepfad

1. `view()` lädt **zuerst** `visible_stamp` (Acquire), **danach** `Arc<Version>` (`INV-VIEW-2`): jede danach geladene Version enthält alle Einträge ≤ `visible_stamp`. Dann keine Sperre.
2. **Gate zuerst:** Clusterzustand und Restrictions vor jeder Entschlüsselung (O(1), §19.5). `Retired` oder weiter ⇒ `Erased`; `BLOCK_READ` ⇒ `Restricted`.
3. **Punktabfrage:** Memtable → immutable → je Level die Dateien der Wartungsgruppe → `ClusterDirectory` → Abschnitt → SubIndex (gecacht) → Filter → ein Datenblock. Ziel: höchstens **ein** Datenblock-Read je Treffer-Level.
4. **Cursor/Page:** k-Wege-Merge über Quellen **eines Clusters**; Kopf ohne Körper-E/A; `page` summiert `weight_sum`, überspringt Blöcke, liefert `ResumeToken` (Stamp, Cluster, Schlüssel, Rest-Budget).
5. **Körper** erst bei `body()`: Blob-Read, CRC, Entsiegeln. Schlüssel fehlt ⇒ `Shredded`.
6. **Block-Cache** (sharded, Pin-Zähler); bei Dateiverlust `invalidate_file`.
7. **Ältere Sicht:** View unter `S` in einen Bereich mit `erased_ranges` ⇒ `Err(Erased{..})` (`INV-ERA-5`).
8. **Keine clusterübergreifenden Scans** im Kern; `Inspector::clusters()` iteriert.

---

## 14. Wartung

### 14.1 Lanes und Claims

| Lane | Aufgabe | Priorität |
|---|---|---|
| `urgent` | Löschfolgearbeit, Purge mit naher Frist, Packgruppen-Abwurf | höchste, darf Compaction abbrechen |
| `flush` | Memtable → Runs | hoch, nie hinter Compaction |
| `compact` | Leveled-Compaction je Wartungsgruppe | normal |
| `blobgc` | Blob-GC | normal |
| `idle` | Scrub, Verwaisten-Suche, Schlüsselrotation, Reserven | niedrig |

**Claims:** Jede Arbeit beansprucht ihre Eingabedateien exklusiv (`registry`, Rang 6). Höhere Priorität bricht niedrigere ab (Flag, Prüfung zwischen Blöcken, Ausgabe verworfen). Der Ausgabe-Edit wird **abgelehnt**, wenn `files_remove ⊄ gehaltene Claims` (`INV-CLAIM-1`).

### 14.2 Compaction

Leveled je Wartungsgruppe, `T` = 10, L0 nach Dateizahl. Bereinigt werden Versionen unter `floor` (kleinster Hold-/View-/Consumer-Stamp) und verdeckte Versionen; Tombstones entfallen, wenn keine tiefere Ebene den Schlüssel enthält. Pure-Promotion nach R-PL-3. Ein kleiner Cluster wird nie auf zwei Dateien eines Levels geteilt; Packed-Runs enden an Clustergrenzen (`INV-PL-5`). Unveränderte Abschnitte werden **byte-identisch kopiert**. Für versiegelte Cluster braucht Compaction den Clusterschlüssel; ist der Cluster `Retired`/`Shredded`, wird sein Abschnitt **ohne Entschlüsseln verworfen**. **Retired-Filter:** Jeder Job notiert `retire_gen`; der Ausgabe-Edit verwirft die Ausgabe und plant neu, wenn sich `retire_gen` geändert hat und ein Eingabecluster inzwischen `Retired` ist.

### 14.3 Blob-GC und Graveyard

Kandidaten nach Lebendanteil (`blob_live_delta`). GC kopiert überlebende Records in einen neuen Blob (neue `blob_id`, neu versiegelt). Betroffene Runs werden **im selben Edit** auf neue Refs umgeschrieben, wenn sie zu **einem** Cluster gehören; Packed-Fälle laufen über die nächste Compaction. Der alte Blob wird erst entfernt, wenn der Abgleich nach `INV-BLOB-1` keinen lebenden Run mehr findet.

**Graveyard:** Eine Datei wird erst gelöscht, wenn (a) der entfernende Edit durabel ist **und** (b) die letzte `Arc<Version>` mit Verweis gefallen ist (`INV-GRAVE-1`). Das Unlink führt die Wartung aus, nie `Drop`.

### 14.4 Segmente

§18.1.

### 14.5 Packgruppen-Abwurf (Ablauf der Aufbewahrung)

```
PD1 ClusterGate-Frame für alle Cluster der Gruppe → Retired(S)            ◄── logischer Punkt
    (Restrictions in der Gruppe ⇒ diese Cluster ausnehmen; Rest wird neu gepackt, Fallback: Abwurf wartet)
PD2 (Cryptographic) Clusterschlüssel vernichten, Edit → Shredded
PD3 Claims auf Dateien der Gruppe, Compaction dort abbrechen
PD4 EIN Edit { files_remove[alle Dateien der Gruppe], cluster_state → Purged, erased_ranges_add, anchor_mark }   ◄── physischer Punkt
PD5 Graveyard → unlink → sync_dir → Ledger → Anker → Receipt
```

Bei sehr großen Gruppen wird das Gate in Teil-Frames zu je höchstens `max_gate_clusters` (4096) geschnitten, **alle im selben Segmentschnitt** und unter einem Ticket (`INV-ERA-1` gilt je Cluster).

### 14.6 Governor

Token-Bucket je Klasse (Wartungs-E/A, CPU, Handles). Wartung nutzt höchstens `maint_io_fraction` (50 %) der gemessenen Kapazität, `urgent` hat eine Reserve. Handles: LRU bis `max_open_files`; kalte Gruppen halten keine Handles.

---
## 15. Recovery

### 15.1 Ablauf beim Öffnen (deterministisch)

```
R1  LOCK nehmen (exklusiv; ReadOnly: geteilt). Misserfolg ⇒ Err(Locked)
R2  FORMAT lesen und prüfen (Kette, Version); Unsupported ⇒ Abbruch
R2a Anker prüfen (K-04): Anchor.read() = (E_a, H_a, C_a) gegen Catalog und Ledger.
    Catalog-Epoche C_c < C_a oder (E_c = E_a und Ledger-Kopf ≠ H_a) ⇒ Err(Rollback) (F-9), nur ReadOnly mit Diagnose.
    C_c > C_a ⇒ Anker nachziehen (Absturz zwischen Edit und Anker).
    AnchorPolicy::Required und Anker nicht erreichbar ⇒ Err(AnchorUnavailable).
R3  CURRENT → Catalog: Snapshot + Edits bis zum letzten gültigen Record, Rest verwerfen (Torn Tail)
R4  Verzeichnisse scannen: Dateien ohne Catalog-Eintrag ⇒ verwaist (Liste, noch nicht löschen)
R5  KeyProvider prüfen; KEK fehlt ⇒ Modus LockedKeys (nur Spaces ohne Seal lesbar, Schreiben verweigert)
R6  Redo ab redo_floor in zwei Durchläufen:
      Durchlauf 1: Gruppenkette prüfen, ClusterGate-Frames und Stamps sammeln ⇒ retired_set = Catalog ∪ Gate-Frames
      Durchlauf 2: Frames anwenden; Sektionen retired Cluster überspringen (nie entsiegeln), sonst entsiegeln und einspielen.
      Fehlt der Clusterschlüssel eines **nicht** retired Clusters ⇒ Corrupt{KeyMissing}.
      Torn-Tail-Regel INV-LOG-8; alles bis zur letzten redo_mark MUSS gültig sein (INV-LOG-7).
R7  Memtable aufbauen; visible_stamp = durable_stamp = letzter gültiger Stamp; next_stamp nach §12.3
R8  Offene Löschungen = erase_pending ∪ Gate-Frames ohne erase_done ∪ Cluster Retired/Shredded ≠ Purged
    ⇒ in `urgent` einplanen, ab der ersten offenen Stufe fortsetzen (destroy ist idempotent)
R9  Verwaiste Dateien und .building.* entfernen (nach R8, nie davor)
R10 Reserven (Stamp, ID, Redo-Segment) herstellen, Wartung starten, OpenReport
    (Frames, verworfene Bytes, offene Löschungen, verwaiste Dateien, Ankerstatus)
```

Ehrliche Grenze: Ein beschädigter **letzter** quittierter Gruppenabschnitt hinter der letzten `redo_mark`, dem nichts Gültiges folgt, ist von einem Torn Write nicht unterscheidbar. Die Heilung kürzt dann; das ist die einzige Stelle, an der Bitfäule `durable = true` verletzen kann. `redo_mark` (alle `redo_mark_bytes`, 8 MiB, und bei jedem Siegel) hält das Fenster klein.

### 15.2 Absturzmatrix

| Unterbrechung | Soll nach Neustart |
|---|---|
| vor Redo-`sync` | Change nicht sichtbar, `apply` kehrte nicht zurück |
| nach Redo-`sync`, vor Rückkehr | Change vorhanden (Token verhindert Doppelung) |
| mitten im Run-Bau | `.building` entfernt, Redo spielt ein |
| nach `rename`, vor Edit | Run verwaist, entfernt |
| nach Edit, vor Unlink | alte Dateien verwaist, entfernt |
| Gate-Frame durabel, Catalog-Edit fehlt | `Retired` aus Frame abgeleitet, Edit nachgeholt |
| Gate durabel, Schlüssel noch da | Löschung wird fortgesetzt (Roll-forward) |
| Schlüssel vernichtet, `Shredded`-Edit fehlt | `probe` = Absent ⇒ Edit nachholen |
| Purge-Edit durabel, Unlink offen | Verwaisten-Suche entfernt |
| Edit durabel, Anker nicht nachgezogen | Anker wird in R2a nachgezogen |
| Ledger-Rahmen halb | Kette erkennt Torn Tail, Eintrag wird aus Catalog neu erzeugt |
| Absturz nach nicht-durablem `Relaxed`-Change | Stamp wird nicht wiederverwendet |
| Volatile-Space mit Inhalt | Inhalt weg, kein Fehler |
| Checkpoint abgebrochen | `dst` unvollständig ohne Siegel, Quelle unverändert |

---

## 16. Sichten, Holds, Consumer, Restrictions

### 16.1 Sichten
`View` bei Stamp `V` ist unveränderlich (`INV-VIEW-1`). Jede View zählt in der `registry` (Untergrenze `floor`).

### 16.2 Holds
Befristet (`ttl ≤ max_hold_ttl`, 24 h), mit Eigentümer, halten Versionen ≥ Hold-Stamp. Technisch, **keine** Rechtsfunktion: Ein Erase überstimmt einen Hold ohne Fehler (Views auf retired Cluster liefern `Erased`).

### 16.3 Consumer
Persistent (`consumer_ack`), zählt wie ein Hold, mit **Maximalrückstand** `max_lag`. Überschreitet er ihn: `Lagged` (Hold fällt, `next` liefert `Err(Lagged)`, Aufholen per Vollscan oben).

### 16.4 Change-Feed
Reihenfolge nach Stamp, **mindestens einmal**. Löschungen erscheinen als `Change::Erased{cluster|range, at}`. Volatile-Spaces erscheinen nicht im Feed.

### 16.5 Restriction

```rust
pub struct RestrictionSpec {
    pub reason: u16,                   // opaker Code der oberen Schicht, PFLICHT (≠ 0)
    pub owner: OwnerId,
    pub effects: RestrictionEffects,   // BLOCK_ERASE (immer) | BLOCK_WRITE | BLOCK_READ
    pub expires: Option<WallTime>,     // Ablauf lässt nur den Vermerk fallen, keine automatische Löschung
}
```

- Solange eine `Restriction` mit `BLOCK_ERASE` auf einem Cluster der **Link-Hülle** liegt, antwortet jede Löschung mit `Err(Restricted{ids, reasons})`. **Kein Override im Kern** (SD-09). Aufheben (`lift`, Eigentümerprüfung, Ledger-Eintrag) ist Entscheidung der oberen Schicht.
- `BLOCK_READ`: Lesepfade liefern `Err(Restricted)`; zweckgebundene Lesung nur mit `RestrictedReadCap` (nicht klonbar, `Observer`-Ereignis mit Zweck-Code, nie im Cache oder Consumer-Feed).
- Restrictions sind durabel und überleben Restore (Ledger-Anhang).
- `restrict`/`lift` laufen als Control-Frame durch den Committer, in derselben Stamp-Reihenfolge wie das Gate (INV-ERA-10).

---

## 17. Verschlüsselung, Versiegelung und Löschung

### 17.1 Modi (in `FORMAT`, unveränderlich)

| Modus | Schutz | Verfahren |
|---|---|---|
| `Plain` | Zufallsfehler | CRC32C, Kettenprüfsumme im Redo (nicht kryptografisch) |
| `Authenticated` | Integrität gegen Manipulation (AT-3), keine Vertraulichkeit | keyed MAC (16 Byte) mit Zusatzdaten, Kette in Redo, Catalog, Ledger |
| `Encrypted` | Vertraulichkeit und Integrität (AT-1…AT-3) | AEAD (XChaCha20-Poly1305, 24-Byte-Zufallsnonce), Schlüssel per KDF |

Algorithmen sind Ports mit IDs im `FORMAT`; ein Wechsel ist eine Migration. Zufallsnonces (SD-14): kein Zählerzustand, sicher beim Umkopieren. Preis: 24 Byte je Block oder Record.

### 17.2 Schlüsselhierarchie

```
KEK (KeyProvider, extern; nie in Dateien, nie in Checkpoints)
 ├─ Space-DEK        zufällig, mit KEK umhüllt in KEYS
 │     └─ Block-Schlüssel (Spaces ohne Seal) = KDF(DEK, "ctxs blk v1" ‖ file_id)
 ├─ Redo-Schlüssel   = KDF(KEK, "ctxs redo v1" ‖ segment_id)
 ├─ Catalog-/FORMAT-/Ledger-Schlüssel = KDF(KEK, "ctxs cat v1" | "ctxs led v1")
 └─ Clusterschlüssel KeyProvider::cluster_key(cluster, epoch)   (NICHT in KEYS)
        └─ Siegelschlüssel = KDF(cluster_key, "ctxs seal v1" ‖ nonce)
```

```rust
pub trait KeyProvider: Send + Sync {
    fn kek(&self) -> Result<Key32, KeyError>;
    fn cluster_key(&self, c: ClusterId, epoch: u32) -> Result<Option<Key32>, KeyError>;  // None = vernichtet
    fn create_cluster_key(&self, c: ClusterId, e: &dyn Entropy) -> Result<u32, KeyError>; // liefert epoch
    fn destroy(&self, c: ClusterId) -> Result<(), KeyError>;      // durabel bei Rückkehr, idempotent (AN-06)
    fn probe(&self, c: ClusterId) -> Result<KeyState, KeyError>;  // Present | Absent
}
```

Der Kern speichert **keine** Clusterschlüssel; er hält einen begrenzten Cache (TTL, Obergrenze) mit Generationszähler (§19.5). Schlüsselmaterial liegt in `Zeroizing`-Hüllen.

**Rotation (K-13):** Eine neue `key_epoch` gilt für neue Schreibvorgänge. Eine alte Epoche wird erst vernichtet, wenn kein Block und kein Redo-Frame mehr mit ihr versiegelt ist (Flush, Compaction). Catalog-Edit `key_epoch`, Roll-forward (`INV-KEY-3`).

### 17.3 Versiegelung (Seal)

- Jeder Block, jede Redo-Op-Nutzlast und jeder Blob-Record eines `Seal::Cluster`-Space wird mit `AEAD(siegelschlüssel, nonce[24] zufällig, plaintext, aad)` versiegelt.
- **AAD:** Block `(space, cluster, block_uid)`; Redo-Op `(space, cluster, op_uid, op_index)` (K-24, **ohne** Stamp); Blob-Record `(space, cluster, blob_id, offset)`. **Nicht** an Dateiname oder Dateiposition gebunden ⇒ Rohkopieren ist gültig.
- Versiegelt wird **vor** der Stamp-Vergabe in `apply` (§12.1 Schritt 2), nicht unter dem Committer. Die Memtable hält Klartext. Der Flush versiegelt mit Block-AAD neu.
- Folge (L3): Nach Vernichtung des Clusterschlüssels ist **alles** des Clusters (Nutzerschlüssel, Kopf, Körper, Index, Filter, Marker) in Redo, Runs, Blobs, Checkpoints und Backups ohne Schlüssel unlesbar (`INV-ERA-7`). Offen bleiben Cluster-ID, Zähler, Größen, Stamp-Bereiche.

### 17.4 Stufen und ehrliche Garantien

| Stufe | Verfahren | Schützt gegen | Schützt **nicht** gegen |
|---|---|---|---|
| `Logical` | Gate (`Retired`) | Lesen über die API | Inhalt auf Platte |
| `Physical` | Gate, Purge (Datei abwerfen oder Rohkopie), Edit, Unlink, `sync_dir` | Lesen aus lebenden Dateien, Wiederauftauchen nach Absturz (mit Anker) | Journal, Flash-Reste, externe Backups, Hardlinks |
| `Scrubbed` | wie `Physical`, vor Unlink mit Nullen überschreiben + Sync | einfache Wiederherstellung klassischer Datenträger | CoW-Dateisysteme, SSDs (best-effort) |
| `Cryptographic` | Gate, **Clusterschlüssel vernichten**, dann Purge | Reste in Redo, Backups, Flash, sofern der Schlüssel nirgends sonst liegt | Schlüsselkopien; nicht verfügbar ohne Seal |

Standardpfad: `Cryptographic` + Purge in `Seal::Cluster`-Spaces. Ohne Seal höchstens `Physical`/`Scrubbed`. Das Receipt nennt Stufe, geprüfte Muster und Restexposition (§17.11), es spricht nie von „unwiderruflich“ oder „konform“.

### 17.5 Protokoll `Eraser::cluster` (ER0–ER7)

```
ER0 Vorprüfung:   Cluster existiert; Link-Hülle H = {c} ∪ transitiv gebundene abgeleitete Cluster;
                  Restriction auf H ⇒ Err(Restricted); Level vs. Modus; KeyProvider erreichbar (Crypto); bereits Retired ⇒ vorhandenes Ticket
ER1 Gate:         Committer-Op: ClusterGate-Frame { H, S, ticket, level } als erstes Frame eines neuen Segments, Gruppen-Fsync   ◄── LOGISCHER Punkt
                  Der Committer berechnet Hülle und Restrictions NEU (INV-ERA-10); wuchs die Hülle ⇒ Retry ab ER0.
                  Danach: cluster_table ← Retired{S}, gen++, Memtable-Einträge von H entfernen/überschreiben, Cache invalidieren;
                  Catalog-Edit { cluster_state(H → Retired, S), erase_pending_add } und Ledger `Retired` folgen.
                  `cluster()` kehrt mit Ok(ticket) zurück; ab hier ist nie mehr ein Fehler die Antwort.
ER2 Schlüssel:    (Cryptographic) KeyProvider::destroy(c) ∀ c ∈ H; probe == Absent; Cache leeren, gen++; Edit { cluster_state(H → Shredded) }  ◄── KRYPTO-Punkt
ER3 Plan:         betroffene Dateien über Packgruppen-Filter und ClusterDirectory; Claims (urgent); Segmente mit Überlappung (§18.1)
ER4 Barriere:     Physical/Scrubbed: Flush aller Memtables mit Frames in Segmenten < Schnitt, `redo_floor` ≥ Schnittsegment, ältere Segmente ⇒ Graveyard
                  (Kein Segment-Rewrite nötig; Fremddaten sind durch den Flush in Runs gesichert, K-03).
                  Cryptographic mit Seal: keine Barriere nötig, Redo-Reste sind unlesbar (INV-ERA-7).
ER5 Umbau:        Pure: nur entfernen. Packed: neue Datei = übrige Abschnitte roh kopiert + neues Verzeichnis (sync_all, sync_dir).
                  Blobs: Inkarnation (gleiche Offsets, Löcher) bzw. entfernen. Segmente: SegmentOwner::rewrite_without
ER6 Edit:         EIN Edit { files_add, files_remove, blob_replace, blob_live_delta, cluster_state(H → Purged),
                              erased_ranges_add(H, S), erase_done(ticket), anchor_mark }                          ◄── PHYSISCHER Punkt
ER7 Danach:       Version veröffentlichen, Cache invalidieren, alte Dateien ins Graveyard (ViewPolicy), optional Scrub, unlink, sync_dir,
                  Ledger `Done`, Anker fortschreiben, ErasureReceipt
```

- **Reihenfolge ist Vertrag (L7):** ER1 < ER2 < ER6 < Unlink. Bei `Cryptographic` läuft ER3–ER6 **nie**, solange `probe` Present meldet (`INV-ERA-3`).
- **Warum ER1 im Committer liegt (K-02):** Ein Change, der vor dem Gate geprüft wurde, darf nicht nach dem durablen Gate sichtbar werden. Das Gate steht in derselben Stamp-Reihenfolge wie alle Changes.
- **Zeit:** ER0–ER2 sind O(1). ER3–ER6 sind durch die Größe der betroffenen Dateien begrenzt (je Packed-Datei ≤ `run_target_bytes`, 64 MiB), nicht durch den Store. Der Purge wird **gebündelt**: Auslöser Frist zu 70 % verbraucht, ≥ `purge_batch_min` Cluster oder anstehende Compaction in der Datei (Mitnahme im selben Edit).
- **Frist:** `purge_deadline` ist Pflichtparameter (SD-06). Wird sie gerissen, steigt die Priorität auf `urgent`, `Observer` meldet es.
- **Views:** `ViewPolicy::Wait(timeout)` (Unlink wartet; nach Timeout bleibt der Vorgang in `Purging{AwaitingViews}`, die logische Garantie gilt bereits) oder `Revoke`.
- **Receipt (`INV-ERA-8`)** erst, wenn INV-ERA-2/6 gelten, Ledger-Eintrag durabel, Anker fortgeschrieben und alle Segmentbesitzer bestätigt haben.

### 17.6 `Eraser::range`

Marker `EraseRange` (Stamp `S`) im Committer (**logischer Punkt**), Flush-Barriere für den Cluster, betroffene Blöcke (über SubIndex) werden ohne die Versionen ≤ `S` im Bereich neu geschrieben, Blobs per Inkarnation, **ein** Edit, danach wie ER7. Roll-forward, idempotent. Reads unter `S` liefern `Erased` (`INV-ERA-5`). Stufen: `Logical | Physical | Scrubbed` (kein Schlüssel je Bereich). Append-only-Spaces nur mit `AdminCap`. Wer Löschbares ohne Wartezeit braucht, legt es in eigene Cluster (L1).

### 17.7 Recovery der Löschung

Offene Menge = `erase_pending ∪ {Gate-/EraseRange-Frames nach Anker} ∪ {Cluster Retired/Shredded} − erase_done`. R8 setzt ab der ersten offenen Stufe fort. ER3–ER6 sind idempotent (Auswahl aus dem Catalog; vollzogenes ER6 erkennbar an `erase_done`); ER7 über die Verwaisten-Suche.

### 17.8 Ledger, Backups, Rebucket

- **Ledger** (`ERASE-LEDGER/`): append-only, kettenverknüpft über den `Mac`-Port (Plain: BLAKE3 unkeyed über den Hash-Teil der Ports). Einträge `{ticket, cluster_id|pack_key, level, S, stage, ts, prev_hash}`, **ohne Inhalte und Nutzerschlüssel**; Cluster-ID-Listen in hashgebundener Anlage. Signatur und Berichte sind Sache oben; der Kern liefert kanonische Bytes. Vor jedem ER-Übergang ist der Eintrag der vorigen Stufe durabel. Reihenfolge Catalog → Ledger → Receipt.
- **Checkpoint:** (1) Flush-Barriere; (2) unter `catalog_edit` `Version` pinnen und `erase_epoch` + Ankerwert lesen; (3) Dateien kopieren (`Copy`, `Reflink`; `Hardlink` nur Opt-in mit Warnung); (4) Manifest siegeln. `KEYS` ist enthalten, **KEK und Clusterschlüssel nicht** (`INV-KEY-1`). Das Ledger liegt nie im Checkpoint.
- **Restore:** vor der ersten Freigabe zum Lesen werden alle Ledger-Einträge nach `erase_epoch` erneut angewandt (`INV-ERA-9`); Restrictions kommen aus dem Ledger-Anhang. Liegt das Manifest hinter dem Anker, ist es ein Rollback und wird verweigert. Parallele Kopien: `rekey_clone()` (neuer `instance_id`, neuer Root-Schlüssel) vor dem ersten Schreiben.
- **`rebucket(c, due')`:** neue Packgruppe, Abschnitt roh in einen Run der Zielgruppe, **ein** Edit `{files_add, files_remove|Abschnitt-Purge, pack_assign(c)}` (`INV-PL-4`).

### 17.9 Vertrag und Grenzen

- **Nutzerschlüssel** ohne Seal MÜSSEN frei von personenbezogenen Daten sein (nicht prüfbar; `Inspector::lint_keys` sucht in Tests Muster).
- **Cluster-IDs** sind opak und zufällig (oder keyed HMAC der oberen Schicht), nie aus Kennungen ableitbar (`INV-CLUSTER-2`). Sie bleiben als Pseudonym im Klartext; Anzahl, Größen, Zeitpunkte und Packgruppe bleiben als Metadaten sichtbar.
- **Nicht kontrolliert:** Prozessspeicher, Swap, Core-Dateien, Reste in noch nicht reklamierten Skiplist-Knoten, Dateisystem-Journal, Flash-Wear-Leveling, externe Backups. `ColdTier`-Dateien unterliegen dem Port.
- **Abgeleitete Daten** sind nur so sicher, wie ihre Besitzer §18 einhalten.
- Der Kern bildet **keine** Inhalts-Hashes über Klartext (`INV-PRIV-1`).

### 17.10 Rollback-Anker (K-04, SD-27)

Ohne Anker kann ein Angreifer (AT-3/AT-5) das Verzeichnis auf einen früheren konsistenten Stand zurücksetzen; gelöschte Daten kehren zurück.

- **Inhalt:** `AnchorValue = (erase_epoch, ledger_head, catalog_epoch)`.
- **Fortschreibung:** bei jedem ER-Schritt (ER1, ER2, ER6, PD-Schritte) und jedem Catalog-Rollover, **nicht** bei jedem Write.
- **Grenze:** Ein Rollback, der nur bestätigte Schreibvorgänge entfernt, wird nur bis zum letzten Ankerstand erkannt. Das ist ein Integritäts-, kein Löschrisiko.
- **Richtlinie:** `Required` ist Standard bei `Encrypted`/`Authenticated`; `Plain` darf `Optional` mit ausdrücklicher Risikoanerkennung nutzen.
- Speicherort (Hardware-Zähler, Tresor, zweiter Datenträger) ist Sache des Implementierers (O-10).

### 17.11 Restexposition (im Receipt zu nennen, im Orakel zu prüfen)

1. Prozessspeicher, Swap, Core-Dateien. 2. Dateisystem-Journal, Flash-Reste, externe Backups. 3. Metadaten: Cluster-ID, Anzahl, Größen, Zeitpunkte, Packgruppe. 4. Rollback über den Anker hinaus. 5. Kopien des Clusterschlüssels außerhalb des Providers.

---

## 18. Abgeleitete Daten und Erweiterungsverträge

### 18.1 Segment-Register

Vom Besitzer geschriebene Dateien neben den Runs, vom Catalog verwaltet, **Pure oder Packed**.

```rust
pub trait SegmentOwner: Send + Sync {
    fn kind(&self) -> SegmentKind;
    fn cover(&self, seg: &SegmentMeta) -> SegmentCover;   // Exclusive(ClusterId | PackKey) | Mixed(Liste oder Filter)
    fn rewrite_without(&self, seg: &SegmentMeta, drop: &ClusterSet, io: &mut SegmentIo)
        -> Result<RewriteOutcome, SegErr>;                // Pflichtmethode
    fn watermark(&self) -> Stamp;
}
```

`Exclusive`: der Kern wirft das Segment selbst ab. `Mixed`: der Kern ruft `rewrite_without`; **ein Erase ist erst `Done`, wenn alle überlappenden Segmente bestätigt haben** (`INV-ERA-8`). Keine Antwort in der Frist ⇒ `Purging` mit Alarm, nie überspringen. Zugriff nur über `SegmentIo` (versiegelt wie §17.3).

### 18.2 Cluster-Links

`Clusters::link(derived, sources)` bindet einen Cluster an ≤ `max_links` (8) Quellcluster, azyklisch. Löschung irgendeines Quellclusters zieht den abgeleiteten Cluster im selben Gate-Frame mit. **Nur ausdrücklich gebundene** Cluster, keine implizite Vererbung. Einträge bleiben einclustrig (SD-24); Mehrbetroffenen-Daten über gemeinsame Cluster oder Links.

### 18.3 ExtensionBlock

Opake Blöcke am Run (Tag `u16`, Version `u16`), je Clusterabschnitt oder Datei. `ExtensionBuilder` bekommt den Eintragsstrom bei Flush/Compaction; `merge` und `rewrite_without` sind Pflicht; unbekannter Tag beim Rewrite ⇒ **Fehler**. Ein Besitzer ohne `rewrite_without` darf in `clustered`-Spaces keinen Block anlegen.

### 18.4 Consumer und Change-Feed
§16.3, §16.4: einzige Schnittstelle für inkrementell aufholende Strukturen.

### 18.5 Kopf, Gewichte, Budget
`header`, `weights`, `page`, `weight_sum` sind die Schnittstelle für Rang- und Budgetlogik oben.

### 18.6 Bitemporalität als Konvention (K-14, SD-32)
Gültigkeitszeit lebt im opaken Kopf der oberen Schicht. Die Transaktionszeit ist der `Stamp` (`view_at`). „Gültig zum Zeitpunkt t“ ist Filterung oben.

### 18.7 Stabilität
Ports, Werttypen und diese Verträge folgen semver in `contextra-storage-ports`.

---

## 19. Nebenläufigkeit: Race-Freiheit durch Konstruktion

### 19.1 Prinzipien
1. Ein Schreiber (Committer) für alle stamp-relevanten Zustandsänderungen; serialisierte Catalog-Edits. 2. Unveränderlichkeit: Dateien nach `rename`, `Version` per `Arc`. 3. Besitz durch Claims. 4. Wahrheit durch durable Edits und Frames. 5. Generationszähler statt Sperren für Schlüssel-Cache und Cluster-Zustand. 6. Keine Sperre über E/A, außer Committer (Redo-`sync`) und `catalog_edit` (Catalog-`sync_data`).

### 19.2 Sperrränge (nur aufsteigend, je Thread höchstens einmal, `debug_assertions` prüft per Rangstack)

| Rang | Sperre | Über E/A gehalten | Bemerkung |
|---|---|---|---|
| 1 | `committer` | ja (Redo-`sync`) | einziger Stamp-Vergeber und Cluster-Zustandsgeber |
| 2 | `intake` | nein | |
| 3 | `catalog_edit` | ja (Catalog-`sync_data`) | |
| 4 | `state` (`RwLock<Arc<Version>>`) | nein | Veröffentlichung |
| 5 | — | — | entfällt (K-01): Memtable ist lock-frei |
| 6 | `registry` (Holds, Consumer, Claims, Restrictions) | nein | Blatt |
| 7 | `cluster_table` | nein | Blatt, Lesen aus Rang 1 zulässig |
| 8 | `maint` (je Lane) | nein | Blatt |
| 9 | `notify` | nein | Blatt |

Rang 1 und Rang 3 gleichzeitig **nur** im Reservierungs-Fallback (§11.8), in dieser Reihenfolge und kurz. Jede andere Reihenfolge ist verboten. Cache- und Governor-Sperren sind Blätter. Der Catalog-Edit zu ER1 läuft **nach** dem Gate-Fsync ohne gehaltenen Rang 1.

### 19.3 Nahtstellen

| Nahtstelle | Gefahr | Lösung | Nachweis |
|---|---|---|---|
| Lesen × Unlink | Datei weg, Leser hält Verweis | Graveyard + `Arc<Version>` | loom, Sim |
| Compaction × Erase | Compaction schreibt Gelöschtes zurück | Claim-Konflikt, `retire_gen`-Prüfung im Ausgabe-Edit | M3, Sim |
| Flush × Erase | Memtable enthält Daten des Clusters | Gate-Frame im Committer vor jeder Memtable-Einfügung; ErasedMarker; Barriere ER4 | M2, Sim |
| Schlüssel-Cache × Vernichtung | Entsiegeln mit gecachtem Schlüssel nach ER2 | `gen`-Prüfung vor **und** nach dem Entsiegeln (§19.5) | loom |
| Checkpoint × Erase | Checkpoint kopiert Dateien, die Erase ersetzt | `Arc<Version>` + `erase_epoch`, serialisiert gegen ER6 über `catalog_edit` | M4, Sim |
| Consumer × Erase | Consumer verliert Versionen | Consumer ist Hold; `Erased`-Change | M2 |
| Rebucket × Lesen | Eintrag doppelt oder nie | `cluster → Gruppe` im selben Edit | M5 |
| Zwei Erase, überlappende Hülle | Blockade | Serialisierung nach Ticket-ID in `urgent` | Sim |
| Restriction × Erase | Löschung läuft während `restrict` | beides im Committer, eine Stamp-Reihenfolge | M2 |
| Link × Erase | Hülle wächst zwischen ER0 und ER1 | Neuberechnung im Committer, Retry | M2 |
| Idempotenz × Löschung | Token verrät Daten | Tabelle ohne Inhalte, TTL | Test |
| Anker × Edit | Edit durabel, Anker alt | Nachzug in R2a, Anker monoton | M6, Absturzmatrix |

### 19.4 Claim-Protokoll

```
claim(files, job) -> Ok(guard) | Err(Held{by})       atomar unter `registry`
Priorität: urgent > flush > compact > blobgc > idle
höhere Priorität setzt job.cancel(); der Job prüft zwischen Blöcken, gibt Claims frei, verwirft die Ausgabe
Ausgabe-Edit: files_remove ⊆ gehaltene Claims, sonst abgelehnt (INV-CLAIM-1)
```

### 19.5 Cluster-Zustand und Schlüsselgenerationen

Der Zustand im Speicher ist ein `AtomicU64` (`state:8 | gen:56`). Gate, ER2 und ER6 erhöhen `gen`. Ein Leser prüft `state` am Anfang und `gen` nach dem Entsiegeln (`INV-KEY-2`). Bereits geliefertes Material im Aufruferspeicher ist Grenze (§17.9).

### 19.6 Verbotene Muster

Alles aus §3.6. Kein `std::sync::Mutex` außer über `sync::Ranked`. Keine Threads außerhalb des `Spawner`. Keine blockierende E/A unter Rang ≥ 4. Keine zweite Schreibquelle außer Committer und `catalog_edit`. Kein Rückruf (`Observer`, `SegmentOwner`) unter Sperren.

---
## 20. Verifikation und Testabdeckung

### 20.1 Ebenen

| Ebene | Werkzeug | Inhalt |
|---|---|---|
| Formal | TLA+/TLC (oder Quint) | M1 Commit/Sichtbarkeit/Recovery/Stamps · M2 Erase (ER0–ER7, Gate, Restriction, Link) · M3 Compaction × Erase × Claims · M4 Checkpoint × Erase × Restore · M5 Rebucket · M6 Anker × Edit × Rollback |
| Code-Modell | `loom` | Sperren, `Arc`-Swap, Generationszähler, Claims, Graveyard, Committer-Gruppenführung |
| Simulation | eigener Scheduler + `shuttle` | **Hauptwerkzeug gegen Races:** seeded Interleavings, `FaultFs`, `SimClock`, `SimEntropy`, reproduzierbarer Seed |
| Absturz | `FaultFs` | Abbruch an **jedem** E/A-Index, Torn Writes, verlorene Fsyncs, Rename-Reihenfolgen, Rollback-Versuche; alle INV nach Neustart |
| Eigenschaft | `proptest` | Codecs, `cmp`, Compaction äquivalent zum Referenzmodell (`BTreeMap` mit Versionen) |
| Mutation | `cargo-mutants` | Mutationsscore auf `codec`, `recovery`, `erase`, `committer` (H-16) |
| Linearisierbarkeit | Wing-Gong-Prüfer | Historien aus der Simulation gegen die sequenzielle Spezifikation |
| Löschorakel | `ErasureOracle` (`testing`) | Eindeutiges Muster in Schlüssel, Kopf, Körper (mit/ohne Seal); nach Erase **alle Bytes aller Dateien** durchsuchen (Runs, Blobs aller Inkarnationen, Redo, `.building`, Segmente, `MemFs`, Checkpoints) |
| Fuzz | `cargo-fuzz` | Dekoder aller Formate, Recovery auf beschädigten Verzeichnissen (≥ 8 Ziele) |
| Referenz | Python-Codierer | Goldene Bytes je Format |
| Statisch | clippy, `cargo-deny`, `layering`, §3.6 | Lints, Lizenzen, Advisories, Schichten |
| Dauerlauf | Soak 72 h, Crash-Loop, Platte voll | Stabilität, Speicherwachstum, Handle-Lecks |

### 20.2 Testabdeckung (K-15)

- **INV-Zitierpflicht:** Jede Invariante aus §22 wird in mindestens einem Test per ID zitiert (`// INV-…`); ein Skript prüft das. Ziel **100 %**.
- **Abdeckung** (`cargo-llvm-cov`, ohne Testcode): Ziel ≥ 90 % Zeilen, ≥ 85 % Zweige (H-15). Ziele, keine Messung; die Messung ist Gate-Bestandteil.
- **Fehlerpfade:** Jeder Pfad jeder Fehlerklasse F-1…F-9 hat einen Test, der ihn auslöst.
- **Keine Abdeckung durch Stubs allein:** Absturztests laufen auf `FaultFs`.

### 20.3 Orakelvertrag

`Plain`/`Physical`: nach Purge **0** Treffer für alle Muster. `Cryptographic`: nach ER2 sind Körper-, Kopf- **und** Schlüsselmuster ohne Schlüsselzugriff unlesbar (0 Klartexttreffer, auch im Redo); nach Purge 0 Ciphertext-Reste des Clusters in lebenden Dateien. Reste in Graveyard-Dateien vor Unlink sind zulässig und werden ausgewiesen.

### 20.4 CI

Pflicht: `check`, `fmt`, `clippy -D warnings`, `test`, `layering`, `loom`, `doc -D warnings`, `deny`, Sim- und Absturz-Kurzlauf, INV-Zitierprüfung, Abdeckung. Nachtlauf: lange Sim, Fuzz, Orakel-Vollsuche, Mutationstest. **Ohne funktionierende CI ist kein Gate gültig.**

---

## 21. Konfiguration und Limits

| Parameter | Standard | Bemerkung |
|---|---|---|
| `max_key_bytes` / `max_header_bytes` / `max_body_bytes` | 1 KiB / 4 KiB / 16 MiB | §7.6 |
| `max_change_ops` / `max_change_bytes` | 4096 / 8 MiB | INV-LIM-1 |
| `block_target` | 16 KiB | |
| `value_separation_min` / `max_inline_bytes` | 1 KiB / 1 MiB | |
| `memtable_bytes` / `max_immutable` / `memtable_total_bytes` | 64 MiB / 3 / 256 MiB | |
| `redo_segment_bytes` / `redo_align` / `redo_mark_bytes` / `redo_max_age` | 64 MiB / 4096 / 8 MiB / 1 h | |
| `run_target_bytes` | 64 MiB | Obergrenze der Rohkopie je Packed-Datei |
| `max_pack_groups` / `pure_threshold` | 4096 / 8 MiB | |
| `max_gate_clusters` | 4096 | §14.5 |
| `max_open_files` / `maint_io_fraction` | 1024 / 0,5 | |
| `purge_batch_min` | 16 | |
| `purge_deadline` | **kein Standard** | Pflichtparameter je Erase-Aufruf (SD-06) |
| Stamp- / ID-Reservierung | 2^20 / 2^16 | |
| `max_links` / `max_hold_ttl` / `idem_ttl` | 8 / 24 h / 24 h | |
| `anchor` | `Required` bei Encrypted/Authenticated | SD-27 |
| `catalog_roll_bytes` | 8 MiB | |

---

## 22. Invarianten

Normativ und vollständig für alle in der Spec zitierten Invarianten. Jeder Test zitiert seine ID.

| ID | Aussage |
|---|---|
| INV-STAMP-1 | Stamps streng monoton über angenommene Changes |
| INV-STAMP-2 | Kein Umbruch; Überlauf ⇒ `Exhausted` |
| INV-STAMP-3 | Kein Stamp wird je zweimal vergeben, auch nicht nach Absturz |
| INV-ID-1 | Dateikennungen und `tmp_id` werden nie wiederverwendet |
| INV-ATOM-1 | Ein Change ist ganz oder gar nicht sichtbar, auch nach Absturz (nicht für Volatile-Ziele) |
| INV-COMMIT-2 | Ein sichtbarer Eintrag ist durabel (außer `Relaxed`) |
| INV-VIEW-1 | Eine View bei `V` sieht stets dieselben Daten |
| INV-VIEW-2 | `visible_stamp` wird vor `Arc<Version>` geladen |
| INV-CONS-1 | Consumer und `view_durable` sehen nur Stamps ≤ `durable_stamp` |
| INV-LOCK-1 | Sperren nur aufsteigend, je Thread höchstens einmal |
| INV-LOCK-2 | Stamp-Vergabe und Cluster-Zustandsübergänge geschehen nur im Committer |
| INV-CAT-1 | Datei lebendig ⇔ durabler `files_add` ohne `files_remove` |
| INV-FMT-1 | Nur kanonische Kodierungen; unbekannte Version/Tags ⇒ Fehler |
| INV-LOG-4 | Ein Gate-Frame ist das erste Frame eines neuen Segments |
| INV-LOG-5 | `redo_floor` rückt nur über Segmente, deren Frames in durablen Runs stehen und deren Gates als `erase_pending` im Catalog stehen |
| INV-LOG-7 | Reservesegment ist stets vorallokiert; alles bis zur letzten `redo_mark` ist gültig |
| INV-LOG-8 | Ein ungültiger Gruppenkopf beendet das Log nur, wenn danach nichts Gültiges folgt |
| INV-GRAVE-1 | Keine Datei wird gelöscht, solange eine `Arc<Version>` auf sie zeigt |
| INV-CLAIM-1 | Ein Edit entfernt nur Dateien unter eigenem Claim |
| INV-BLOB-1 | Blob/Inkarnation wird nur entfernt, wenn kein lebender Run darauf verweist |
| INV-PL-1 | Jede Datei ist Pure oder Packed, nie beides |
| INV-PL-2 | Blöcke und Abschnitte sind clusterrein, ein Cluster ist je Datei ein zusammenhängender Abschnitt |
| INV-PL-3 | Compaction mischt nur innerhalb einer Wartungsgruppe |
| INV-PL-4 | `cluster → Gruppe` wechselt nur im selben Edit wie die Dateimengen |
| INV-PL-5 | Ein kleiner Cluster liegt je Level in höchstens einer Datei |
| INV-CLUSTER-1 | Ein `Purged`-Cluster wird nie wiederverwendet |
| INV-CLUSTER-2 | `ClusterId` ist nicht aus einer Kennung ableitbar |
| INV-ERA-1 | Logische Atomarität: für Views ≥ `S` ist der Cluster leer, für Views < `S` `Erased`, keine Mischsicht |
| INV-ERA-2 | Nach `Purged` enthält kein lebender Run und kein lebendes Redo-Segment eine Version des Clusters (Physical); bei Cryptographic zusätzlich INV-ERA-7 |
| INV-ERA-3 | ER1 < ER2 < ER6 < Unlink; bei `Cryptographic` kein ER3–ER6, solange `probe` Present meldet |
| INV-ERA-4 | Nur Roll-forward |
| INV-ERA-5 | Reads unterhalb `S` liefern `Erased`, nie Daten, nie „leer“ |
| INV-ERA-6 | Nach `Purged` enthält keine lebende Blob-Inkarnation einen Record des Clusters |
| INV-ERA-7 | In `Seal::Cluster`-Spaces ist nach ER2 alles des Clusters ohne Schlüssel unlesbar |
| INV-ERA-8 | `ErasureReceipt` nur bei INV-ERA-2/6, durablem Ledger, fortgeschriebenem Anker, bestätigten Segmenten |
| INV-ERA-9 | Nach Restore ist jeder Cluster aus dem Ledger wieder mindestens `Retired` |
| INV-ERA-10 | Eine `BLOCK_ERASE`-Restriction in der Link-Hülle verhindert jede Löschung; Prüfung im Committer |
| INV-ANCH-1 | Anker monoton; Catalog mit kleinerer Epoche wird nicht schreibend geöffnet |
| INV-KEY-1 | Schlüsselmaterial liegt nie unverhüllt in Dateien und nie in Checkpoints |
| INV-KEY-2 | Kein Entsiegeln liefert Klartext, wenn sich `gen` während des Aufrufs änderte |
| INV-KEY-3 | Eine alte Schlüsselepoche wird erst vernichtet, wenn nichts mehr mit ihr versiegelt ist |
| INV-PRIV-1 | Der Kern bildet keine Inhalts-Hashes über Klartext |
| INV-VOL-1 | Volatile-Spaces schreiben kein Redo; ein Change mit Volatile-Zielen ist nicht durabel über Absturz |
| INV-LIM-1 | Changes über den Limits werden vor jeder Sperre abgewiesen |

---

## 23. Leistung (Hypothesen, Messverfahren)

Alles **Entwurfsziele**. Freigabe nur mit Messbericht (Hardware, Dateisystem, Last, Methode, Streuung). Scheitert eine Hypothese, wird **sie** korrigiert, nicht das Verfahren gelockert.

| ID | Hypothese | Messung |
|---|---|---|
| H-1 | `apply`: Durchsatz ≥ 80 % von Fsync-Rate × mittlere Gruppengröße; p99 ≤ 2 × Fsync-Latenz | Mikrobench, variierende Schreiberzahl |
| H-2 | Punktabfrage warm: ≤ 1 Datenblock-Read je Treffer-Level, p99 ≤ 50 µs | Zipf-Last |
| H-3 | Kopf-Scan ohne Körper ≥ 10× Durchsatz gegenüber Scan mit Körper (≥ 4 KiB) | Scan-Bench |
| H-4 | `page` liest ≤ 1,2 × Blöcke des Minimums | Gewichtsverteilungen |
| H-5 | Schreibverstärkung Metadaten ≤ 15, Körper ≤ 2 | Compaction-Statistik |
| H-6 | ER0–ER2 je Cluster ≤ 10 ms (ohne Providerlatenz) | Löschbench |
| H-7 | Purge ∝ Größe der Packgruppe, nicht des Stores (Faktor ≤ 1,5 bei 10× Store) | Skalierungstest |
| H-8 | Pure-Cluster löschen ≤ 100 ms je 1000 Dateien (ohne Scrub) | Drop-Bench |
| H-9 | 1000 Cluster einer Gruppe ≤ 2 × Aufwand eines Clusters | Bündelungsbench |
| H-10 | Rohkopie beim Purge ≥ 80 % sequentielle Plattenbandbreite | Purge-Bench |
| H-11 | Seal ≤ 15 % CPU im Lastmix; Overhead kleiner Cluster ≤ 5 % Bytes (Cluster ≥ 1 KiB) | A/B-Bench |
| H-12 | Speicher ≤ `memtable_bytes × (1 + max_immutable)` + Cache + O(offene Gruppen); Verzeichnisse etwa 48 Byte je Cluster und Datei | RSS-Messung |
| H-13 | Recovery ∝ Redo-Rückstand + Catalog; 1 GiB Redo ≤ 10 s | Recovery-Bench |
| H-14 | Ribbon ca. 30 % kleiner als Bloom bei gleicher Fehlrate (Literaturwert, nachmessen) | Filterbench |
| H-15 | Zeilenabdeckung ≥ 90 %, Zweigabdeckung ≥ 85 % | `cargo-llvm-cov` |
| H-16 | Mutationsscore ≥ 80 % auf `codec`, `recovery`, `erase`, `committer` | `cargo-mutants` |

---

## 24. Phasen und Gates

| Phase | Inhalt | Gate |
|---|---|---|
| **P0 Fundament** | `ports`, `codec`, `env` (MemFs, RealFs, FaultFs, Sim), `sync`, Anker (Mem, Datei), CI, Lint-Gerüst, Referenzcodierer, Workspace-Einbindung | **G-0:** SPEC R6 normativ (zwei Reviewer, §26 entschieden), CI grün |
| **P1 Format** | alle Formate §11 inkl. ClusterSection, Seal, Ledger, Anker; Goldens; ≥ 8 Fuzz-Ziele | **G-F** Formatfreeze (§11.9) |
| **P2 Schreibpfad** | Committer mit echtem Redo, Reservierungen, Memtable, Recovery, Catalog, Volatile | **G-W:** Absturzmatrix grün, M1 geprüft, Linearisierbarkeit in Sim |
| **P3 Lesepfad** | Flush, Runs (Pure/Packed), Cache, Views, `page` | **G-R:** Modellgleichheit, H-2…H-4 gemessen |
| **P4 Wartung** | Compaction je Gruppe, Claims, Blob-GC, Governor, Rotation | **G-M:** M3 geprüft, Sim ohne Invariantenbruch, H-5 |
| **P5 Löschung** | Cluster, Links, Restrictions, ER0–ER7, Gate im Committer, Abwurf, Ledger, Anker, Restore, Orakel | **G-E:** M2/M4/M5/M6 geprüft, Orakel 0 Treffer, H-6…H-11 |
| **P6 Härtung** | Soak, Dauer-Fuzz, Platte voll, Fehlerinjektion, Facade, Abdeckung | **G-P:** alle INV getestet (100 %), H-15/H-16, Soak 72 h, Messbericht §23, Krypto-Review |

Keine Phase beginnt vor dem Gate der vorherigen. Der Ist-Stand steht in `AGENTS.md` §3 mit Testnachweis. Erst nach **G-P** werden aufsetzende Komponenten angebunden; bis dahin enthalten Repo-Kontext und Dokumente nur den Kern und die Verträge aus §18.

**Sofortmaßnahmen vor P1** (Befunde: Anhang B):

1. `contextra-storage` in den Workspace aufnehmen und die fehlenden Crates `contextra-storage-ports` und `contextra-crypto-ref` anlegen (heute verweist `Cargo.toml` auf nicht vorhandene Pfade).
2. Den Bestand von `crates/contextra-storage` (R2-Entwurf ohne Cluster) als Vorlage behandeln und Modul für Modul auf R6 umstellen; `redo`, `run`, `blob`, `catalog`, `engine/committer` sind formatbetroffen.
3. `recovery`, `flush`, `compaction` fertigstellen; Testbäume wieder kompilierbar machen.
4. `unwrap`/`expect`/Panikstellen aus Produktionscode entfernen (Altstand: 1260 + 16 Treffer, Anhang B).
5. `deny.toml` vervollständigen, CI einrichten (BL-9).
6. Toolchain-Pin klären (`rust-toolchain.toml` pinnt 1.89.0; die Sitzungsumgebung kann sie nicht laden, O-7).
7. Fehlende, in Dokumenten referenzierte Dateien (`ARCHITECTURE.md`, `CONTRIBUTING.md`, `tools/check_spec_sync.py`) anlegen oder die Verweise entfernen.

---

## 25. Entscheidungen

### 25.1 Register

| ID | Entscheidung | Begründung | Verworfen |
|---|---|---|---|
| SD-01 | Catalog als einzige Wahrheit, ein Edit = ein Durabilitätspunkt | Crash-Sicherheit | Zustand in Dateinamen |
| SD-02 | Clusterreine Blöcke, ein Abschnitt je Cluster und Datei | Löschen ohne Entschlüsseln und Neukomprimieren | gemischte Blöcke |
| SD-03 | Bewilligte Abhängigkeiten (§3.5) | Sicherheit ohne eigenes `unsafe` | eigener Baum im Kern |
| SD-04 | Eine Formatversion je Dateiart, Migration als Werkzeug | kein Zweitpfad | Rückwärtskompatible Leser |
| SD-05 | Seal=Cluster umfasst Schlüssel, Kopf, Körper, Index, Filter, Marker | kryptografische Löschung auch der Nutzerschlüssel | nur Körper versiegeln |
| SD-06 | `purge_deadline` Pflichtparameter ohne Standard | Fristen sind Rechts-/Betriebsentscheidung | Standard 72 h im Kern |
| SD-07 | Pure/Packed mit Packgruppe (Mandant × Zeitraum × Shard) | Skalierung, O(1)-Ablauf | Datei je Cluster |
| SD-08 | Rohkopie beim Purge (AAD ohne Dateiposition) | Bandbreite statt CPU | Neuverschlüsseln |
| SD-09 | `Restriction` im Kern, **kein Override** | Mechanismus ohne Politik | `HoldPolicy::Override` |
| SD-10 | Lanes inkl. `urgent`; Flush nie hinter Compaction | Latenz | ein Job je Instanz |
| SD-11 | Fünf Erweiterungsverträge | Anbindung ohne Formatbruch | Nachrüsten |
| SD-12 | Kein 2PC; ein Redo-Frame je Change, ein Catalog-Edit | Atomarität im Kern | verteilter Intent-Log |
| SD-13 | Hardlink-Checkpoints nur Opt-in | Purge erreicht Inodes nicht | Standard-Hardlink |
| SD-14 | XChaCha20-Poly1305, Zufallsnonce | kein Zählerzustand | AES-GCM mit Zähler |
| SD-15 | TLA+-Modelle M1–M6 als Gate | Race-Freiheit beweisen | nur Tests |
| SD-16 | Fsync-Fehler nie wiederholen | Fsync-Semantik | Retry-Schleifen |
| SD-17 | Cluster-Zustand `state|gen` als Atomic | Leser ohne Sperre | Sperre je Cluster |
| SD-18 | Opaker `header`, `weights`, `weight_sum` im Kern | Kopf ohne Körper, Budget ohne Bedeutungswissen | strukturierter Kopf |
| SD-19 | Stamp-Reservierung im Catalog | `INV-STAMP-3` | Stamp aus Redo-Ende |
| SD-20 | Redo in ausgerichteten Gruppen, vorallokiert, Reservesegment | Torn-Write-Eingrenzung, kein ENOSPC im Rollover | fortlaufender Append |
| SD-21 | Blob-Inkarnation ohne Referenz-Umschreibung | Purge ändert keine Runs fremder Cluster | Generationen im `BlobRef` |
| SD-22 | `LOCK` über `Fs::lock`, ReadOnly geteilt | Doppelöffnung verhindern | keine Sperre |
| SD-23 | Ledger getrennt von Catalog und Checkpoints | Backup-Problem | Ledger im Checkpoint |
| SD-24 | Einclustrige Einträge; Mehrbezug über gemeinsame Cluster oder Links | keine Querverweise beim Purge | Mehrfach-Scope-Einträge |
| SD-25 | `cmp` statt `memcmp` über Konkatenation | variable Schlüssellängen | Escaping |
| SD-26 | Volatile-Spaces ohne Redo (je Space) | Arbeitsgedächtnis ohne fsync | Durability je Change |
| SD-27 | Rollback-Anker als Port, Pflicht bei Encrypted/Authenticated | AT-5 | kein Anker (R5) |
| SD-28 | Gate (ER1) ist ein Committer-Frame | Race zwischen Prüfung und Sichtbarkeit | Gate nur im Catalog-Edit (R5) |
| SD-29 | Blob-Entfernung nur nach Abgleich lebender Runs | Zähler kann abweichen | Zähler als alleinige Wahrheit |
| SD-30 | Memtable lock-frei | widerspruchsfreie Sperrordnung | `RwLock` Rang 5 (R5) |
| SD-31 | Change-Limits | begrenzter Frame- und Speicherbedarf | unbegrenzt |
| SD-32 | Bitemporalität als Konvention oben | Kern ohne Zeitlogik | Gültigkeitszeit im Kern |
| SD-33 | Namensraum `contextra-storage(-ports)`, Altstand unverändert | keine Kollision mit `contextra-ports` | Umbenennung im selben Schritt |
| SD-34 | **Eine** normative Spec plus `AGENTS.md`; Vorfassungen in `docs/archive/` | vier widersprüchliche Fassungen | parallele Fassungen |
| SD-35 | Redo-AAD mit `op_uid` statt Stamp | Versiegelung vor Stamp-Vergabe, außerhalb des Committers | Versiegeln unter Rang 1 |
| SD-36 | `AdminCap`/`RestrictedReadCap` als nicht klonbare Fähigkeiten | Kern ohne Benutzerbegriff | Rechteprüfung im Kern |

### 25.2 Herkunft der Übernahmen

| Thema | Quelle | Stelle |
|---|---|---|
| Cluster, Pure/Packed, ER0–ER7 | R3/R5 | §2, §8, §17 |
| Gate im Committer, Segmentschnitt, Zwei-Durchlauf-Recovery, Guard-Overlay, `Indeterminate`, `redo_mark`, Torn-Tail-Regel, Retired-Filter | R5-Variante (Freigabekandidat) | §11.4, §12.1, §14.2, §15.1 |
| Rollback-Anker | R2 (F-10) | §17.10 |
| Volatile, Durability-Stufen, Idempotenz, Bitemporalität | Mnemos-Konzept, R2 | §7.4, §12.5, §18.6 |
| Gewichte, Konsumenten als Holds, Orakel über alle Dateien | Review `contextra-storage` vs. Mnemos | §7.3, §16.3, §20.1 |
| Limits, Lint-Pflicht, Abdeckungsziele, Fähigkeiten statt Rechte | Analyse zu R6 | §3.6, §7.6, §20.2 |

---

## 26. Offene Entscheidungen

| # | Frage | Vorschlag |
|---|---|---|
| O-1 | Wer setzt `Restriction`s, und gilt ein Ablaufdatum automatisch? | Ablauf lässt nur den Vermerk fallen; Aufheben bleibt aktiv (`lift`) |
| O-2 | Tragfähigkeit der Purge-Frist; Körper-Unlesbarkeit sofort genug; Spaces ohne Seal | Seal=Cluster für alle personenbezogenen Spaces vorschreiben (Rechts-/Datenschutzprüfung) |
| O-3 | Mehrbetroffenen-Daten: gemeinsame Cluster oder Links | Gemeinsamer Fallcluster mit Links; Löschung nur auf Auftrag oben |
| O-4 | Produktiver `KeyProvider` (HSM, Tresor, eigener Store) | Referenz in `crypto-ref`, Produktivprovider getrennt |
| O-5 | Inhalts-Hashes über Klartext | Nur über versiegelte Daten; Dedup nur in Spaces ohne Cluster |
| O-6 | Packgruppen-Zeitraum und `max_pack_groups` | Monat / 4096, aus Last ableiten |
| O-7 | Toolchain-Pin und CI-Plattform | CI mit ladbarer, geprüfter Version |
| O-8 | Altbestand (`REPORT.md`, `RunBuilder`, `legacy/`) | nach Freigabe kennzeichnen oder entfernen |
| O-10 | Anker-Implementierung in Produktion | Port-Referenz mit Datei-Anker für Tests; schwache Anker (gleicher Datenträger) als Restrisiko benennen |
| O-11 | Bitemporalitäts-Konvention (§18.6) bestätigen | erst festschreiben, wenn ein Anwendungsfall sie braucht |
| O-12 | Zweiter Reviewer für G-0 | G-0 bleibt offen, bis ein zweiter Reviewer benannt ist |

---

## Anhang A — Referenzvektoren (Pflicht vor G-F)

Je Format mindestens: leeres Objekt, ein Eintrag, Eintrag mit versiegeltem Körper, Cluster-Abschnitt mit zwei Blöcken, Packed-Run mit drei Clustern, Tombstone, Bereichsmarker, Blob mit Loch, Catalog-Edit-Kette mit Reservierung und `anchor_mark`, Ledger-Kette, Redo-Gruppe mit zwei Frames inkl. `ClusterGate`, abgeschnittener und beschädigter Frame, nicht-kanonische `varint`. Erzeugung durch `tools/reference/generator.py` (unabhängig von Rust), Vergleich im Test `golden`. **Stand: tools/reference existiert nicht; keine Vektoren vorhanden.**

## Anhang B — Gemessener Ist-Stand (22.11.2026, per `grep`/`wc`)

### B.1 Altstand `crates/contextra-store`

| Messgröße | Wert |
|---|---|
| Zeilen `src/` / `tests/` | 27 898 / 21 893 |
| Testfunktionen (`#[test]`, `#[tokio::test]`) | 580 |
| `.unwrap()` und `.expect(` in `src/` (inkl. Inline-Tests) | 1 260 |
| `panic!`/`todo!`/`unimplemented!` in `src/` | 16 |
| `unsafe` in `src/` | 0 (`#![forbid(unsafe_code)]`) |
| `let _ =` in `src/` | 75 |
| Default-Features / Toolchain-Pin | `default = []` / `1.89.0` |

Frühere Fassungen nannten andere Zahlen (u. a. 32 015 Zeilen in `src/`). Sie waren nicht belegt und sind ersetzt.

### B.2 Neubau `crates/contextra-storage`

| Befund | Folge |
|---|---|
| Nicht in `[workspace] members` | wird nicht gebaut |
| `Cargo.toml` verweist auf `../contextra-storage-ports` und `../contextra-crypto-ref`; beide Verzeichnisse fehlen | baut nicht (Sofortmaßnahme 1) |
| 24 961 Zeilen in `src/` und `tests/`; Modulstand laut `lib.rs`: `api` (nur Listener), `codec`, `keys`, `env`, `redo`, `run`, `blob`, `catalog`, `engine` ohne `recovery`/`flush`/`compaction`; `erase` nur Dokumentation | Teilbestand |
| Entwurf folgt R2 (kein Cluster, kein Gate-Frame, kein Anker; `TagKind` kennt `PutRef = 5`) | formatbetroffene Module werden umgestellt (Sofortmaßnahme 2) |
| `README.md` und `lib.rs` verweisen auf `ARCHITECTURE.md`, `IMPLEMENTATION_SPEC.md`, `CONTRIBUTING.md`; `CLAUDE.md` auf `tools/check_spec_sync.py`; keine davon existiert | Verweise bereinigt oder Dateien angelegt (Sofortmaßnahme 7) |

### B.3 Herkunft

Konzepte und Vorfassungen liegen in `docs/archive/` (`SPEC-R2.md`, `SPEC-R3.md`, `SPEC-R5-Abnahmekandidat.md`, `SPEC-Variante-R5-Freigabekandidat.md`, `REFACTORING_PLAN_v2.md`, `REVIEW_contextra-storage_vs_mnemos.md`, `MNEMOS_CONCEPT.md`, `Optimal-LLM-Datenbank.md`, `OCTA-SPEC-FINAL.md`). Szenarioklassen des Altbestands (Crash-Recovery, Chaos, Group Commit, Compaction, Format, Verschlüsselung, Löschung) dienen als Vorlage für die Netze in §20; Code wird nicht übernommen.

## Anhang C — Nicht übernommen

| Vorschlag | Grund |
|---|---|
| Prolly-Tree, Commit-DAG, Branches Stufe B | zu groß für den Kern (NZ-13) |
| Slab-Allokator, Object-Header-Layout | Blob-Trennung deckt den Bedarf |
| Content-Addressing im Kern | verletzt Löschlokalität (O-5) |
| Merge-Operator, `mmap` im Leseweg, Kompression in v1 | NZ-4, NZ-9, NZ-10 |
| MCP-Server, LLM-API im Kern | oberhalb des Kerns |
| Strukturierter Wertkopf (`class`, `lineage`) | SD-18 |
| `class`-Byte und `class_mask` der R5-Variante | nicht begründet gegenüber Filtern im opaken Kopf; vergrößert die Freeze-Liste |

## Anhang D — Leitplanken für aufsetzende Schichten

- **Token-Budget:** `weights` je Eintrag, `weight_sum` je Block, `page` mit `ResumeToken`. Die Bedeutung legt die obere Schicht fest.
- **Branches Stufe A:** Overlay über `Guard`, `Hold`, `history`. Commit-DAG erst, wenn Sync, Audit oder Replikation es verlangen.
- **Abgeleitete Indizes:** Consumer mit `max_lag`, Neuaufbau per Vollscan, Segmente mit Besitzer-Vertrag.
- **Vergessen:** Entscheidung der oberen Schicht; keine stille Entfernung ohne Löschauftrag.
- **Idempotenz:** `IdemToken` je Change; Runs und Schritte darüber.
- **Löschung abgeleiteter Daten:** Jede Struktur mit Klartextresten implementiert `SegmentOwner` oder `ExtensionBuilder`, sonst gilt sie nicht als löschbar.

## Anhang E — Änderungen gegenüber R5

| ID | R5 | R6 | Grund |
|---|---|---|---|
| K-01 | Memtable (Rang 5) vor Version (Rang 4) in §12.1 | Memtable lock-frei, Rang 5 entfällt | Verstoß gegen die eigene Sperrordnung |
| K-02 | Gate ER1 nur als Catalog-Edit | `ClusterGate`-Frame im Committer, Segmentschnitt, Recovery leitet `Retired` ab | Race: Change nach durablem Gate sichtbar |
| K-03 | ER4 ohne Unterscheidung der Stufen | Barriere für Physical/Scrubbed; keine für Cryptographic mit Seal | Fremddaten im Redo dürfen nicht verloren gehen |
| K-04 | Kein Anker | Port `Anchor`, R2a, F-9, `INV-ANCH-1` | AT-5 |
| K-05 | Blob-Entfernung nach Zähler | Abgleich lebender Runs (`INV-BLOB-1`) | Zähler kann abweichen |
| K-06 | Keine Change-Limits | §7.6, `INV-LIM-1` | begrenzter Speicher |
| K-07 | Kein flüchtiger Space | `Volatile` | Arbeitsgedächtnis |
| K-09 | `PackKey` mit/ohne Shard | überall mit Shard | Widerspruch |
| K-10 | §3.5 lock-frei, §19.2 `RwLock` | einheitlich lock-frei | Widerspruch |
| K-11 | Frame ohne `kind`/`epoch` | ergänzt | Gate-Frame und Anker darstellbar |
| K-13 | Rotation nur erwähnt | §17.2, `INV-KEY-3` | Lücke |
| K-14 | Bitemporalität offen | §18.6 | Einordnung |
| K-15 | Abdeckung unquantifiziert | §20.2, H-15, H-16 | messbar |
| K-16 | Verweis auf fehlendes `IMPLEMENTATION_SPEC.md` | Phasen in §24, Stand in `AGENTS.md` | Hierarchie widerspruchsfrei |
| K-17 | Pfade `acta/`, `legacy/…` | Namensraum nach Repository (SD-33) | Realität |
| K-18 | Ungeprüfte Altstandszahlen | Messung Anhang B | „keine erfundenen Zahlen“ |
| K-20 | „unwiderruflich“ im Löschkontext | Stufen und Restexposition | Dokumentationssprache REQ-DOC-2 |
| K-21 | Lint-Regeln nicht in der Spec | §3.6 | messbar |
| K-24 | Redo-AAD enthielt den Stamp, Versiegelung geschieht vor der Stamp-Vergabe; `Shredded` für alle Stufen | AAD mit `op_uid`; `Shredded` nur bei Schlüsselvernichtung | Widersprüche in R5 und im Zwischenentwurf |
| K-25 | „Admin-Recht“ ohne Definition, obwohl der Kern keine Rechte kennt | `AdminCap`, `RestrictedReadCap` | Widerspruch zu §3.3 |
| K-26 | Guards sahen nur Memtable und Version | Gruppen-Overlay (§12.1 5b) | Guards früherer Changes derselben Gruppe |
| K-27 | Kein Fehler bei unbekanntem Fsync-Ergebnis | `Indeterminate` | Wiederholung per `IdemToken` |

## Anhang F — Zustände

```
Cluster:    Live ──ClusterGate(S)──► Retired(S) ──ER2──► Shredded ──ER6──► Purged   (Shredded nur bei Cryptographic; nur vorwärts; Restrictions orthogonal)
Packgruppe: Aktiv ──PD1──► Abwurf ──PD4──► Entfernt
Datei:      Building ──rename+Edit──► Lebendig ──Edit──► Obsolet ──letzte Referenz──► Unlink
Erase:      Retired ─► KeyDestroyed ─► Purging ─► Done(Receipt)    (Absturz an jeder Stelle: Fortsetzen, nie Rückabwicklung)
Anker:      monoton; Catalog-Epoche ≤ Anker-Epoche beim Öffnen (sonst Rollback, F-9)
Volatile:   Leer ──Change──► Sichtbar ──Schließen/Absturz──► Verworfen
```
