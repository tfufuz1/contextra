Die finale Löscharchitektur kombiniert die Fast-Path-Saga aus v3.1 mit dem zweitaktigen Ausführungsmodell der Neukonzeption, den Speicher- und Verifikationsprimitiven des Erasure Core sowie den Härtungs- und Governance-Regeln aus Rückbau/Neuaufbau und den Integrationsvorgaben.

Referenzierte Dateien sind in delete/ !! Extrahiere de relevanten Informationen

---

## 1. Extrahierte Architektur-Bausteine

### 1.1 Konsolidierungsmatrix

| Baustein | Quelle (laut Zuordnung) | Spezifizierte Rolle in der finalen Architektur |
| --- | --- | --- |
| **Schlüsselhierarchie & Epochen** | v3.1

 | Zufällige, gewrappte `DocDek` (je Betroffenen-Einheit) und `ArtKey` (je Index-Artefakt) unter epochenweisen `KEK[coll, e]`; gebündelter `Epoch Seal` mit atomarer KeyTable-Ersetzung.

 |
| **Saga-Ablauf, Siegel 1–4 & Modus-Trennung** | v3.1 & Umsetzungsprüfung

 | Fast-Path-Reihenfolge (`L` → `P` → `C*` → `C1–C3` → `M`); kryptografisch verkettete Teilberichte (Siegel 1–4); strikte Trennung zwischen reversiblem `Restricted` (Art. 18 DSGVO) und irreversiblem `Gated`-Löschpfad (Art. 17 DSGVO).

 |
| **Takt-1-Journal, `LiveMask`/`Slot` & Snapshot-Freiheit** | Neukonzeption

 | Group-Commit-Journal mit gebündeltem `fsync` in Takt 1; dichte `u32`-`Slot`-Adressierung mit lock-freier `LiveMask`-Bitmap; offene Snapshots blockieren den Löschbeginn nicht mehr.

 |
| **Zentrales `ReadGate` & Zeitreise-Vorrang** | Erasure Core

 | Eine einzige Prüfstelle in `TenantScopedStorage` vor jedem Lesezugriff (`get`, `scan`, `get_at_seq`, `scan_prefix_at`); „Erase gewinnt gegen Zeitreisen“.

 |
| **`Purge-Compaction` & `ErasureManifest**` | Erasure Core

 | Verschlüsseltes `ErasureManifest` je Dokument (listet Hilfsschlüssel); streamender SSTable-Rewrite (`create_like`) in der Hygiene-Phase für alle MVCC-Versionen, Klartext-Indexschlüssel und Termschlüssel.

 |
| **Scope-Persistenz, Intent, Fence & Koordinator** | Löschpfad-Spezifikation (ergänzt durch)

 | Typisierter `ErasureScope` mit 128-Bit-`scope_id`; persistierte Scope-Datei bzw. verschlüsselter `ScopeDescriptor` unter `K_saga`; Journal-Intent nach Regel `J1` vor jeder destruktiven Mutation; lock-freie Publikationsbarriere (`Dekker-Fence` / atomare Speicherordnung) zwischen Journal-Commit und `LiveMask`-Lesepfad; genau ein zentraler Saga-Koordinator mit typisierten Fehlern und Retry-Backoff.

 |
| **Nachweis, Kanonische Kodierung & Claim-Taxonomie** | Analyse und Zielarchitektur (ergänzt durch)

 | `ResidualReport` bzw. `DeletionProof` V5 ausschließlich aus gemessenen `LayerWitness`-Belegen; deterministische kanonische Kodierung statt mehrdeutiger Formate (wie `bincode`) über alle signierten Felder; strenge Claim-Taxonomie (`L`, `P`, `C*`, `C1–C3`, `M` + Annahmecodes); Ed25519-Signatur.

 |
| **`KeyAnchor`-Port & Custody-Profile** | Erasure Core & v3.1

 | Hardware-/OS-Abstraktion (`KeyAnchor` / `EffaceableKeySlot`) mit Profilen `K0`, `K1` (`FileOnly`), `K2` (`Hardware`/`OsKeyring`) und `K3` (externer KMS); ehrliche Deckelung auf `C1` mit Restrisiko `A-FLASH` / `KeyStoreRemanence` bei `FileOnly`.

 |
| **`Erasure Lab` (Verifikations-Suite)** | Erasure Core

 | Unabhängiger `ForensicScanner` ohne Store-Parser, verpflichtende Positivkontrolle (`AC-1`) je Lauf, `FaultVfs` (Crash- und Torn-Write-Injektion), `HeapScanner`, `ReferenceModel`, `NeighborDigest` und `LeakyPurger`-Mutant.

 |
| **Governance, Typ-Besitzer & Wiring-Pflicht** | Umsetzungsprüfung (ergänzt durch)

 | Verbindliche Architektur-Anforderungen (`R1–R6` sowie `REQ-DEL-N1..N27` und `N-01..N-29`); eindeutige Typ-Besitzer in `types`, `ports` und `crypto` ohne Dubletten; erzwungene Verdrahtung ohne Stubs (`INV-WIRING-2`) und Build-Gates (`surface-coverage`, `claim-generator`, `deletion-honesty`).

 |

---

### 1.2 Spezifikation der Kernbausteine

#### A. Schlüsselhierarchie, Padding & `Epoch Seal`

* **Mehrstufige Zufallsschlüssel:** Unter einer Custody-Wurzel (`K1`, `K2` oder `K3`) liegt pro Collection und Epoche ein zufälliger, gewrappter `KEK[coll, e]`. Darunter liegen für jede Betroffenen-Einheit ein zufälliger 32-Byte-`DocDek`, für jedes Index-Artefakt (Text-Segment, Graph-Snapshot) ein zufälliger `ArtKey` sowie für Pseudonym-Hashes ein zufälliger `K_ref[tenant, e]`. Zwischenschlüssel werden niemals per HKDF abgeleitet, damit sie durch Vernichtung unwiederbringlich entfernt werden können (HKDF wird nur für strukturelle Schlüssel wie `K_eng` aus der Wurzel genutzt).


* **AEAD & Restmetadaten-Schutz:** Nutzdaten und Schlüssel-Wraps verwenden `AES-256-GCM-SIV` mit zufälliger 96-Bit-Nonce und 128-Bit-Tag. Records werden nach dem **Padmé-Schema** (Overhead $\le 12\%$, Mindestlänge `min_record_bytes = 256` Byte) aufgefüllt; `doc_key_id` und Sequenzbezug stehen innerhalb des unter `K_eng` verschlüsselten Blockbereichs und nicht im Klartext-Header.


* **KeyTable & `Epoch Seal`:** Gewrappte Schlüssel liegen in einer `KeyTable` mit Ersetzen-Semantik (atomare Segment-Neuschreibung ohne Append-Log). Beim `Epoch Seal` (ausgelöst spätestens `seal_after_deletion_max = 7` Tage nach einer Löschung, regulär nach `kek_epoch_len = 30` Tagen oder per `seal-now`) wird `KEK[coll, e+1]` erzeugt, alle lebenden `DocDek`s und `ArtKey`s werden in $O(\text{Anzahl Schlüssel})$ neu gewrappt, die `KeyTable` wird atomar ersetzt und `KEK[e]` samt Wrap-Kopie vernichtet.



#### B. `Slot`, `LiveMask` und zentrales `ReadGate`

* **Dichter `Slot` & `LiveMask`:** Jede 128-Bit-`DocId` (berechnet als `BLAKE3-keyed(K_scope, tenant ‖ collection ‖ user_key)` mit Kollisionsprüfung beim Insert) wird pro Collection auf einen dichten `u32`-`Slot` abgebildet. Eine lock-freie Bitmap (`LiveMask`, atomare Wörter bzw. Copy-on-Write-`Arc`-Swap) bildet die unmittelbare Lesewahrheit für HNSW-Kandidatenfilterung, BM25-Posting-Iteratoren und Chunk-Abrufe. Ein `Slot` wird erst nach vollständigem Index-Rebuild wiederverwendet.


* **`ReadGate` in `TenantScopedStorage`:** `LsmStorage` ist nach außen `pub(crate)`; jeder Lesezugriff der Engine läuft zwingend über `TenantScopedStorage`. Dort wird vor `get`, `scan_prefix`, `get_at_seq` und `scan_prefix_at` geprüft, ob der Eintrag in `LiveMask` oder im durablen `ErasedSet` (`HMAC-SHA256` bzw. `BLAKE3-keyed` unter `K_ref`/`K_commit`) gesperrt ist. Treffer liefern `Erased` – auch für offene Snapshots und historische Zeitreisen.



#### C. `ErasureManifest` & `Purge-Compaction`

* **`ErasureManifest`:** Beim Schreiben eines Dokuments wird in derselben Transaktion ein unter dem `DocDek` verschlüsseltes `ErasureManifest` (`KeyKind::ErasureManifest`) abgelegt, das alle zugehörigen Hilfsschlüssel (`Rel`, `Intent`, Termschlüssel, Graph-Entität, Vektor-Slot) verzeichnet.


* **`Purge-Compaction`:** Da SSTable-Blockindizes und Bloom-Filter Nutzerschlüssel im Klartext führen können, führt die Hygiene-Phase eine streamende `Purge-Compaction` (`LsmStorage::erase_keys`) durch. Unter einer exklusiven Compaction-Barrier werden kandidierende SSTables über `create_like` (gleicher `KeyManager`, Formatversion, originale `seq` und `tx_id`) neu geschrieben, wobei **alle** MVCC-Versionen und Tombstones der Zielschlüssel verworfen werden. Der Austausch erfolgt atomar über `tmp` $\rightarrow$ `fsync` $\rightarrow$ Manifest-Commit $\rightarrow$ `fsync` $\rightarrow$ Verzeichnis-`fsync` $\rightarrow$ Reader-Swap, während alte Dateien über das `PendingDelete`-Journal erst nach Ablauf offener Zeitreisen gelöscht werden.



#### D. `KeyAnchor` und ehrliche Custody-Stufen

* Über den Port `KeyAnchor` (`EffaceableKeySlot`) abstrahiert Contextra die Wurzelverwahrung in `K0` (unverschlüsselt, nur Entwicklung, maximal Klasse `P`), `K1` / `FileOnly` (Argon2id mit $m \ge 64\text{ MiB}, t \ge 3$ und Software-Keystore $\rightarrow$ Klasse `C1` mit verpflichtender Restrisiko-Deklaration `A-FLASH` / `KeyStoreRemanence`), `K2` / `Hardware` & `OsKeyring` (manipulationsresistentes Modul $\rightarrow$ Klasse `C2`) und `K3` (externer KMS mit Vernichtungsattest $\rightarrow$ Klasse `C3`).



---

## 2. Finale Löschsaga: End-to-End-Spezifikation (Schritte 1–9)

### 2.1 Phasen-, Zustands- und Siegelübersicht

| Schritt | Phase / Takt | Saga-Zustand | Durabilitätsanker | Erreichte Klasse & Siegel |
| --- | --- | --- | --- | --- |
| **1** | Takt 1 (synchron) | `Requested` $\rightarrow$ `Authorized` $\rightarrow$ `Frozen` | Group-Commit-Journal (`fsync`), `ErasedSet` | **Klasse `L**` / **Siegel 1**

 |
| **2** | Takt 2 (Fast-Path) | `LogicallyDeleted` | WAL-Commit unter durchgehendem Key-Lock | Klasse `L` (interner Zwischenzustand)

 |
| **3** | Takt 2 (Fast-Path) | `PlaintextPurged` | Journal-Eintrag + atomarer Text-Segment-Switch | Vorbereitung Klasse `P`

 |
| **4** | Takt 2 (Fast-Path) | `PlaintextMeasured` | `LayerWitness` Ebene A + `Pre-Destruction-Inventar` | **Klasse `P**` / **Siegel 2**

 |
| **5** | Takt 2 (Fast-Path) | `KeyDestroyed` | Journal-Intent (`J1`) + atomares `KeyTable`-Replace (`fsync`) | Schlüssel vernichtet

 |
| **6** | Takt 2 (Fast-Path) | `CryptoMeasured` | `LayerWitness` Ebene B (Nadelscan + Entschlüsselungsprobe) | **Klasse `C***` (vorläufig)

 |
| **7** | Takt 2 (gebündelt) | `EpochSealed` | Journal-Eintrag + `KEK[e]`-Vernichtung | **Klasse `C1**` (`C2`/`C3`) / **Siegel 3**

 |
| **8** | Hintergrund-Hygiene | `Hygiene` $\rightarrow$ `HygieneMeasured` | Manifest-Commit, WAL-Prüfung, Graph-Gen `g+1` | **Klasse `M**` / **Siegel 4**

 |
| **9** | Abschluss | `Reported` $\rightarrow$ `Completed` | Ed25519-Signatur, `AuditChain`, `ErasedSetMirror` | Finaler Bericht & Bescheinigung

 |

---

### 2.2 Detaillierte Schritt-Spezifikation

#### Schritt 1: Takt 1 – Synchroner Einstieg, Journal-Commit & `LiveMask`-Sperre (Klasse `L`)

* **Eingang & Validierung:** Der zentrale Koordinator nimmt einen typisierten `ErasureScope` (`TenantId`, `CollectionId`, `ErasureTarget`) entgegen, prüft Mandantenbindung und Berechtigung (`Authorized`) und löst die 128-Bit-`scope_id` im Katalog auf den dichten `u32`-`Slot` auf. Existiert kein gültiger Scope, wird mit einem typisierten Fehler abgebrochen.


* **Modus-Weiche (`Restricted` vs. `Gated`):** Liegt eine gesetzliche Aufbewahrungspflicht (Art. 17 Abs. 3 DSGVO) oder ein `Legal Hold` vor, verzweigt die Saga nach dem Sperren in den reversiblen Zustand `Restricted{basis, until}` (Wirkung nach Art. 18 DSGVO: Lesesperre aktiv, keinerlei Daten- oder Schlüsselvernichtung). Für reguläre Löschaufträge (Art. 17 DSGVO) wird der irreversible `Gated`-Pfad eingeleitet.


* **Durabilität & Speicherbarriere (`Dekker-Fence`):**
1. Der `ScopeDescriptor` wird unter einem ephemeren `K_saga[erasure_id]` verschlüsselt (Lösung des Lösch-Paradoxons im Journal) und zusammen mit dem Referenz-Hash (`HMAC-SHA256(K_ref, scope)`) als `Gated`/`Frozen`-Datensatz in das `ErasureJournal` geschrieben.


2. Mehrere parallel eintreffende Löschanfragen werden im Group-Commit (Wartefenster 1–2 ms) über einen gemeinsamen `fsync`/`fdatasync` durabel gemacht. Schlägt der Flush fehl, liefert Takt 1 sofort `Err` zurück, ohne die Lesesperre zu aktivieren.


3. Nach erfolgreichem `fsync` wird der Eintrag in das `ErasedSet` aufgenommen und das Bit in der `LiveMask` gesetzt. Ein strikter Speicher-Fence (`Dekker-Fence` / `SeqCst`-Publikation) stellt sicher, dass kein Leser eine gesetzte `LiveMask` ohne durablen Journal-Zustand oder umgekehrt einen veralteten Maskenzustand nach Rückgabe von Takt 1 sieht.




* **Wirkung & Siegel 1:** Ab diesem Zeitpunkt verweigert das `ReadGate` in `TenantScopedStorage` jeden Lesezugriff auf den Scope – einschließlich laufender Snapshots, `as_of`-Zeitreisen, Checkpoints und Exporte. Der Koordinator stellt **Siegel 1** (`Klasse L`, `legal_state = Eingeschränkt`) aus, trägt es in die `AuditChain` ein und gibt das `ErasureTicket` synchron an den Aufrufer zurück (Zielwert p99 $\le 5\text{ ms}$).



#### Schritt 2: Logische Löschung & Lineage-Kaskade unter gehaltenem Key-Lock

* **Durchgehender Key-Lock:** Der Koordinator erwirbt das dokument- bzw. einheitsbezogene Lock (`kv_locks.lock_for(id)`) und hält es **ununterbrochen über die gesamte Fast-Path-Saga bis einschließlich Schritt 5**. Konkurrierende `remember`- oder Update-Operationen auf derselben Einheit werden abgewiesen.


* **`ErasureManifest` auslesen:** Vor jeder destruktiven Änderung liest der Koordinator das unter dem `DocDek` verschlüsselte `ErasureManifest` aus dem Store, um die vollständige Liste aller betroffenen Hilfsschlüssel (`__rel:`, `__tx_intent:`, Termschlüssel, Graph-Entitäts-IDs, Vektor-Slots) zu erhalten.


* **Logische Löschung & Kaskade:** Die Saga setzt im WAL-Batch den logischen Tombstone (`LogicallyDeleted`) und stößt über `INV-DERIVED-1` die Lineage-Kaskade für abgeleitete Artefakte (Zusammenfassungen, Chunk-Präfixe, Abfrage- und Embedding-Kontexte) an. Ableitungen mit weiteren, nicht gelöschten Quellen werden als `stale` markiert oder neu berechnet. Bei Mehrpersonen-Dokumenten greift die konfigurierte `Mixed-Policy` (Standard: `BlockAll` mit `Partial{ReviewRequired}`).



#### Schritt 3: Bereinigung der Ebene A & Text-Segment-Rotation

* **Vorbereitung & Journal:** Vor jeder Bereinigung wird der Übergang nach `PlaintextPurged` im Journal protokolliert. Alle Ebene-A-Purger arbeiten asynchron ohne `block_on`.


* **HNSW-Arena (Schicht S2):** Über den `Slot` aus dem Katalog werden die zugehörigen Vektor-Slots in der RAM-Arena sowohl in `fp32` als auch in quantisierter `SQ8`-Darstellung deterministisch mit Nullen überschrieben (`zeroize`). Eine Kanten-Umverdrahtung findet nicht statt (`VETO-F02`: kein Rewiring); die Slot-Nummer bleibt bis zum Hintergrund-Neuaufbau gesperrt (`INV-HNSW-SLOT-1`).


* **Caches & MemTable (Schichten S1, S4):** Alle im `Purgeable`-Register eingetragenen Caches (Block-, Embedding-, RRF-, Antwort-Cache sowie Surrogat-Vault) werden für den Scope invalidiert und im Speicher zeroized. In der aktiven MemTable werden Klartext-Einträge des Scopes entzogen bzw. geflusht.


* **Text-Segment-Neuschreibung (Schicht S3):** Da Postings und Term-Wörterbücher viele Dokumente bündeln und nicht unter einem einzelnen `DocDek` stehen, wird das betroffene Text-Segment unter Ausschluss des Scopes neu geschrieben: Terme, die ausschließlich im gelöschten Scope vorkamen, werden aus dem Wörterbuch entfernt, BM25-Statistiken (IDF, Dokumentlängen, Zähler) werden neu berechnet, das neue Segment wird unter einem frisch erzeugten `ArtKey` verschlüsselt und der alte `ArtKey` wird sofort vernichtet (`INV-ARTKEY-1`).



#### Schritt 4: Messung Ebene A, Positivkontrolle & Pre-Destruction-Inventar (Klasse `P`)

* **Unabhängige Schicht-Scanner:** Die besitzenden Schicht-Scanner (`LsmScanner`, `VectorScanner`, `TextScanner`, `CacheScanner`, `LineageScanner`) prüfen die Ebene-A-Strukturen direkt auf Speicherebene, ohne Code der Purger zu teilen (`INV-WITNESS-MINT-1`). Geprüft wird per Rohbyte-Scan (kanonische Terme, Roh-IDs, Embedding-Bytes, Chunk-Prüfsummen) sowie über die dekodierte Sicht.


* **Pflicht-Positivkontrolle (`AC-1`):** Jeder Scan-Lauf schleust ein definiertes Kontrolldatum (Canary) ein und verifiziert, dass der Scanner es findet. Schlägt die Positivkontrolle fehl, ist der gesamte Messlauf ungültig und führt zu einem typisierten Fehler mit Wiederholung.


* **Pre-Destruction-Inventar (nur Metadaten):** Unmittelbar vor der Schlüsselvernichtung erfasst der Scanner für die betroffene `key_id` alle noch existierenden Chiffrat-Blöcke auf Platte (Dateipfad, Byte-Offset, Länge) ausschließlich als kryptografischen Digest und Blockzähler – ohne Nutzdaten. Nach Schritt 5 dient dieses Inventar dem Nachweis, dass jeder verbliebene Block einem vernichteten Schlüssel zugeordnet ist.


* **Siegel 2:** Sind alle Ebene-A-Schichten ohne Restbefund gemessen und ist die Positivkontrolle bestanden, erreicht die Saga `PlaintextMeasured` (**Klasse `P**`) und stellt **Siegel 2** aus.



#### Schritt 5: Idempotente Vernichtung des `DocDek`

* **Optionale Karenz (`erasure_grace`):** Falls konfiguriert (Startwert `0 h`, Obergrenze `72 h`, im Regulated-Tier ohne Default Pflichtwert), wartet die Saga vor der Schlüsselvernichtung das Karenzfenster ab. Bis zum Ablauf kann der Verantwortliche den Vorgang per `Unfreeze` mit auditierter Begründung zurücknehmen.


* **Journal-Intent (`Regel J1`) & Atomares Replace:**
1. Zuerst wird der unwiderrufliche Vernichtungs-Intent im Journal persistent gemacht (`fsync`), inklusive der 96-Bit-Wrap-Nonce des zu vernichtenden `DocDek` für den anschließenden Nadelscan.


2. Der gewrappte `DocDek` wird im Speicher (`Zeroizing`) genullt, in der `KeyTable` mit Nullen überschrieben (`fsync`) und die `KeyTable`-Segmentdatei wird atomar ohne den vernichteten Eintrag neu geschrieben (`tmp` $\rightarrow$ `fsync` $\rightarrow$ `rename` $\rightarrow$ Verzeichnis-`fsync`).


3. Das `KeyRegistry`-Sidecar (`MFKR`) protokolliert das Ereignis `destroyed` mit Referenz-Hash, jedoch ohne Schlüsselmaterial oder Wrap-Blob (`REQ-DEL-N5`).




* **Idempotenz:** Ist der `DocDek` nach protokolliertem Intent (etwa nach einem Crash während Schritt 5) in der `KeyTable` bereits nicht mehr auffindbar, gilt die Vernichtung als vollständig vollzogen. Nach Abschluss von Schritt 5 wird der Key-Lock freigegeben.



#### Schritt 6: Messung Ebene B – Nadelscan & Entschlüsselungsprobe (Klasse `C*`)

* **Nadelscan (Schlüsselabwesenheit):** Der Ebene-B-Scanner durchsucht alle persistenten Dateien (`KeyTable`, Keystore, Sidecars, Checkpoints, temporäre Dateien, `.bak`) im Rohbyte-Modus nach der 96-Bit-Wrap-Nonce des vernichteten `DocDek`. Erwartetes Ergebnis: **0 Treffer**.


* **Entschlüsselungsprobe & Kanarien-Pfadtest:**
1. Auf Basis des Pre-Destruction-Inventars aus Schritt 4 wird eine echte `unwrap`/`open`-Probe auf mindestens einen Chiffrat-Block des gelöschten Dokuments ausgeführt; diese muss zwingend mit `KeyAbsent` fehlschlagen.


2. Zusätzlich verifiziert die Kanarien-Probe (eigener Kanarien-Datensatz mit eigenem `DocDek`, der über exakt denselben Vernichtungspfad entfernt wurde), dass Vernichtungsmechanismus und Scanner end-to-end funktionieren.




* **Zustand `CryptoMeasured` (`C*`):** Ist Ebene B sauber gemessen, erreicht die Löschung den Zustand `CryptoMeasured`. Solange der `Epoch Seal` (Schritt 7) noch aussteht, trägt der Status die Klasse **`C*` (vorläufig)** mit dem offenen Residual `R-KEKEPOCH until`.



#### Schritt 7: Gebündelter `Epoch Seal` & Aufwertung auf `C1` (`C2`/`C3`) / Siegel 3

* **Bündelung:** Löschungen innerhalb derselben Collection und Epoche teilen sich einen gemeinsamen `Epoch Seal` (spätestens nach `seal_after_deletion_max = 7` Tagen oder sofort via `seal-now`).


* **Ablauf des `Epoch Seal`:**
1. Über den `KeyAnchor`-Port (`EffaceableKeySlot`) wird `KEK[coll, e+1]` erzeugt.


2. Alle noch lebenden `DocDek`s und `ArtKey`s der Collection werden unter `KEK[coll, e+1]` neu gewrappt (kein Datenrewrite).


3. Die `KeyTable` wird atomar neu geschrieben (`fsync`).


4. `KEK[coll, e]` und ihre Wrap-Kopie werden vernichtet (bei `K1` optional mit Passphrasen-Rotation `rotate_passphrase_on_seal`; bei `K2`/`K3` über `rotate()` und `destroy()` am Hardware-/KMS-Slot).




* **Klassenaufwertung & Siegel 3:** Erst nach abgeschlossenem `Epoch Seal` (`INV-EPOCH-SEAL-1`) entfällt `R-KEKEPOCH`. Abhängig vom Custody-Profil des `KeyAnchor` erreicht der Vorgang:


* **Klasse `C1**` bei `K1` / `FileOnly` (mit ausgewiesener Annahme `A-FLASH` / `KeyStoreRemanence`),


* **Klasse `C2**` bei `K2` / `Hardware` oder `OsKeyring` (nach bestandener Konformitätssuite),


* **Klasse `C3**` bei `K3` / externem KMS (mit Anbieter-Vernichtungsattest).
Der Koordinator signiert und verkettet **Siegel 3**.





#### Schritt 8: Terminierte Hintergrund-Hygiene (Klasse `M` / Siegel 4)

Die Hygiene läuft asynchron im Hintergrund, ist jedoch durch `purge_rebuild_max_days` (Startwert `14 Tage`) hart terminiert. Sie bedient die strengste Rechtsauffassung (physische Entfernung aller erreichbaren Kopien):

1. **`Purge-Compaction` (SSTables & Blobs):** `LsmStorage::erase_keys` schreibt betroffene SSTables und Blob-Segmente streamend unter `create_like` neu, entfernt alle MVCC-Versionen, Tombstones und Klartext-Indexschlüssel des Scopes und tauscht die Dateien atomar im Manifest aus. Der Blockcache für ersetzte Dateien wird invalidiert und zeroized.


2. **WAL-Entfernung mit Pflichtprüfung:** Aktive WALs werden rotiert, Memtables geflusht und versiegelte WAL-Dateien gelöscht. Anschließend wird per `Vfs` geprüft, dass die Dateien physisch nicht mehr im Verzeichnis existieren; verbleibt eine Datei, bleibt der Zustand auf `Pending` mit Retry.


3. **Graph-Neuaufbau & Generationswechsel (Schicht S2):** Bei Überschreiten des Tombstone-Anteils ($> 0{,}2$) oder bei Fristablauf baut ein Hintergrund-Worker eine neue HNSW-Generation `g+1` ausschließlich aus lebenden Slots auf. Nach dem atomaren Umschalt-Commit auf `g+1` wird der `ArtKey` der alten Generation `g` vernichtet (schließt `R-GRAPH`) und die alten Slots werden wieder freigegeben.


4. **Checkpoint-Sweep (Schicht S6):** Lokale Checkpoints und Hardlink-Klone werden gemäß Policy (`Rewrite` als Standard) bereinigt und einzeln gescannt, da Hardlinks durch das Neuschreiben der Quelldatei nicht mitbereinigt werden.


5. **Abschlussmessung (`HygieneMeasured`) & Siegel 4:** Nach erneuter Schichtmessung und Positivkontrolle erreicht der Vorgang **Klasse `M` (Messvollständig)** und **Siegel 4** wird ausgestellt.



#### Schritt 9: Kanonischer Ed25519-Bericht, Bescheinigung & Außentext-Generierung

* **Kanonischer `DeletionProof` V5:** Aus den gesammelten `LayerWitness`-Datensätzen wird deterministisch in kanonischer Kodierung der Gesamtbericht erstellt. Pflichtfelder sind: erreichte Klasse (`L`, `P`, `C*`, `C1–C3`, `M`), verkettete Siegel 1–4, `legal_state`, `backup_window`, `assumptions[]` (`A-FLASH`, `A-ALGO`, `A-KEYLEAK`, `A-RAM`, `A-EXTBACKUP`, `A-HOSTHIST`), `residuals[]`, Schicht-Digests inklusive Positivkontrollen, `Custody` und `DestroyAttestation`, `Epoch-Seal`-Referenz, `subject_ref_hash` (ohne Klartext-IDs), Pflicht-Ausschlüsse sowie `Verification`- und `Validation`-Status (im Regulated-Tier mit Vier-Augen-Concurrence-Signatur).


* **Signatur & Zeitanker:** Alle Felder werden als kanonisches Byte-Array mit **Ed25519** signiert (`INV-PROOF-KEY-1`) und mit einer Prüfsumme versehen. Optional wird der Digest über einen qualifizierten Zeitstempel (RFC 3161 / eIDAS Art. 41, 42) samt Langzeitvalidierungsdaten verankert. Bei `Completed` wird `K_saga` vernichtet, sodass das Journal keinen Personenbezug mehr enthält.


* **Datensparsame Löschbescheinigung:** Für Betroffene, Auftraggeber oder Aufsichten wird aus dem Bericht eine einseitige Bescheinigung abgeleitet, die nur Datum, Scope-Art, erreichte Klassen, Annahmen, Ausschlüsse, Digest, Signatur und Zeitanker enthält – **niemals** IDs, Dateinamen oder Inhalte.


* **Faktengesteuerter Außentext (`INV-CLAIMS-1`):** Außentexte werden ausschließlich über eine geschlossene Satzbibliothek erzeugt, in der jeder Satz an einen gemessenen Wert des Berichts gebunden ist. Absolutbegriffe wie „100 % gelöscht“, „unwiederbringlich“ oder „DSGVO-konform“ sind durch das Build-Gate `deletion-honesty` gesperrt.



---

## 3. Querschnittsregeln für Betrieb, Recovery und Backups

* **Deterministisches Crash-Recovery (`recover()`):** Beim Start läuft `recover()` zwingend **vor** der Freigabe des Lesepfads. Das Journal wird wiedergegeben, alle `Frozen`-/`Gated`-Scopes werden sofort in `ErasedSet` und `LiveMask` geladen. Abstürze vor Schritt 5 setzen bei `PlaintextPurged` fort; Abstürze nach protokolliertem Vernichtungs-Intent in Schritt 5 prüfen die Idempotenz der Schlüsselvernichtung und setzen die Messung in Schritt 6 fort. Ein Absturz mitten im `Epoch Seal` lässt `KEK[e]` und `KEK[e+1]` koexistieren und setzt den Re-Wrap fort.


* **Zwei-Spuren-Backup & `Restore-Guard`:** Daten-Backups (Spur D) enthalten ausschließlich Chiffrate ohne `KeyTable` und ohne Keystore. Schlüssel-Backups (Spur K) tragen ein signiertes Ablaufdatum `valid_until` (`key_backup_max_age`, Startwert `30 Tage`) und werden nach Ablauf verweigert (`RestoreBeyondHorizon`). Bei jedem Restore gleicht der `Restore-Guard` vor dem ersten Lesezugriff das lokale `ErasedSet` sowie den externen `ErasedSetMirror` ab, vernichtet etwaige wiederhergestellte `DocDek`s sofort erneut und protokolliert `restore_reapplied` im Audit.


* **Empfängerregister (`RecipientLedger`):** Ausleitungen über Exporte oder Egress-Kanäle werden ohne Nutzinhalt protokolliert, sodass `doctor` bei einer Löschung die Liste der zu benachrichtigenden Empfänger (Art. 17 Abs. 2, Art. 19 DSGVO) ausgeben kann.
