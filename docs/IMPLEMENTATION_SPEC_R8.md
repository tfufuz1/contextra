# IMPLEMENTATION_SPEC R8 — `contextra-storage` (Umsetzungsplan und Arbeitspakete)

| | |
|---|---|
| Status | Entwurf zur Freigabe durch den Maintainer. Nicht normativ; normativ bleibt `SPEC-R8` (`docs/SPEC-R8.md`). Dieses Dokument trifft die Entscheidungen, die die Spec offen lässt (Abschnitt 3), und schneidet die Umsetzung in disjunkte Komponenten und Arbeitspakete. |
| Basis | `SPEC-R8.md` (identisch mit `docs/SPEC-R8.md` im Repository `tfufuz1/contextra`, Stand `main` bei Erstellung) |
| Verifikation | Repository geklont und am Code geprüft (Abschnitt 1). **Nicht kompiliert:** in der Arbeitsumgebung ist keine Rust-Toolchain verfügbar. Alle Aussagen zu Build und Tests sind Annahmen, bis CI läuft. |
| Sprache | Prosa deutsch, Bezeichner englisch |

---

## 0. Zusammenfassung

Die Spec R8 ist konsistent und vollständig genug, um sie umzusetzen, aber sie endet an mehreren Stellen dort, wo der Betrieb beginnt. Die größten Lücken sind nicht technischer Art im Kern selbst, sondern betreffen die Umgebung: die Ablösung der neun Konsumenten von `contextra-store`, die Herkunft des Schlüsselmaterials für den Referenz-Provider, die Abhängigkeitsfreigaben im Kern, die Toolchain und die Dokumentation, die `AGENTS.md` auf veraltete Gate- und Regelnamen verweisen lässt.

Die Entscheidung zur Umsetzung lautet **Neuentwicklung statt Refaktorisierung**. Der bestehende Crate-Stand `crates/contextra-storage` (rund 28.000 Zeilen) folgt dem früheren Entwurf R2 ohne Cluster, Gate-Frame, Anker und Ledger. Er wird verworfen, im Git-Tag `storage-r2-archive` gesichert und nicht als Code übernommen. Das entspricht der Sofortmaßnahme (6) der Spec.

Der Plan zerlegt den Kern in 17 Domänen mit disjunkter Dateizuständigkeit (Abschnitt 5). Jede Domäne besitzt eine feste Menge von Pfaden. Änderungen an gemeinsamen Schnittstellen laufen über Änderungsanträge, nicht über direkte Eingriffe. Die Arbeitspakete (Abschnitt 7) folgen den Phasen P0 bis P8 der Spec und haben jeweils eine Abnahme, die sich auf Invarianten (`INV-*`) und Hypothesen (`H-*`) bezieht.

---

## 1. Befund am Repository

| Beobachtung | Beleg | Konsequenz |
|---|---|---|
| `docs/SPEC-R8.md` ist identisch mit dem Upload | `diff` ohne Ausgabe | Die Spec liegt bereits im Repo. |
| `AGENTS.md` verweist auf `docs/SPEC.md`, `docs/IMPLEMENTATION_SPEC.md`, `CLAUDE.md`, `tests/invariants.rs` und `tools/doc_lint.py`. Keine dieser Dateien existiert. | Dateiprüfung | Tote Verweise. Der Arbeitskontext ist derzeit nicht nutzbar. → L-09 |
| `AGENTS.md` nutzt Gates G-1 bis G-6, Ränge 1 bis 9, Regeln D-xx und Abschnitte §22/§26. R8 nutzt G-0 bis G-P, Ränge 1 bis 8 und SD-xx. | Textvergleich | Zwei Regelwerke widersprechen sich. → L-09 |
| `AGENTS.md` verbietet neue Crates (D-18). R8 führt `contextra-storage-ports` und `contextra-crypto-ref` ein. | Textvergleich | Konflikt bei der Port-Ablage. → L-10 |
| `AGENTS.md` verbietet Krypto-Crates im Kern (D-05). R8 SD-03 erlaubt `chacha20poly1305`, `blake3`, `zeroize` im Kern. | Textvergleich | Konflikt bei der Abhängigkeitsfreigabe. → L-04 |
| `rust-toolchain.toml` pinnt `channel = "stable"`. R8 Sofortmaßnahme (4) verlangt 1.89.0. | Dateiinhalt | Nicht reproduzierbare Builds. → L-07 |
| `contextra-store` hat neun Konsumenten: `contextra-agent`, `-checkpoint`, `-cognition`, `-db`, `-engine`, `-graph`, `-py`, `contextra` und `benchmarks/contextra-bench`. | Grep auf `Cargo.toml` | Die Ablösung betrifft neun Crates. R8 beschreibt sie nicht. → L-01, Abschnitt 8 |
| Externe Konsumenten nutzen die Fassade (`LsmStorage`, `LsmConfig`, `DurabilityMode`, `TenantScopedStorage`, `MergeOperator`, `SystemPressure`). Einzelne Interna werden außerhalb referenziert: `sstable` und `compaction` je in einer Datei, `scanner` in einer Datei. `wal`, `manifest` und `memtable` werden außerhalb nicht genutzt. | Grep auf `contextra_store::` mit Ausschluss des Crates selbst | Mapping der Fassade nötig (Abschnitt 8). Die wenigen Interna-Zugriffe sind Umbaupunkte, kein Kernproblem. |
| `contextra-store` enthält Testhooks (`FAIL_APPEND_FOR_TX`, `DELAY_APPEND_FOR_TX`) und einen Altformat-Fallback (`legacy_fallback`). Außerhalb des Crates werden sie nicht referenziert. | Grep (10 bzw. 6 Dateien, alle intern) | Fehlerinjektion muss über `FaultFs` laufen (K-11). Der Altformat-Fallback ist nach NZ-7 verboten und wird nicht übernommen. |
| `contextra-ports::persistence` definiert bereits `Fs`, `File`, `Clock`, `Entropy`, `Metrics`, `Hooks`, `Cipher`, `Mac`, `Kdf`, `KeyProvider`, `PressureSource`, `CompactionFilter`. Die Formen weichen von R8 §3.2 ab. | Trait-Liste | Doppelte Port-Definition. Der Port-Katalog ist ein Entscheidungspunkt. → L-10, L-11 |
| `contextra-storage` (Bestand) exportiert `api` nur als Listener, `erase` nur als Dokumentation und hat keine Facade. Der Bestand hängt noch an `contextra-ports::persistence`. | `src/lib.rs`, Modulstruktur | Bestätigt die Verwerfung. |
| `contextra-crypto` enthält laut Commit-Historie (WP-2.4) XChaCha20-Poly1305, Blake3-MAC, HKDF und Blake3-Hasher. Der Modul-Kopf nennt noch AES-256-GCM-SIV und HMAC-SHA256. | Commits `7c53cb0`, `ad56c15`; Modul-Kopf | Die Krypto-Crate ist in Bewegung. Eignung muss geprüft werden. → L-04 |
| `contextra-durable-fs` implementiert Fsync-Poisoning (`INV-FSYNC-1`), atomares Schreiben und Scrubbing. Kein anderes Crate hängt davon ab. | `lib.rs`, `Cargo.toml`-Grep | Ein nicht genutzter Baustein, den R8 nicht erwähnt. → L-12 |
| `tests/layering.rs` erlaubt `contextra-graph → contextra-store` als Dev-Abhängigkeit bis Phase 1a. | `LAYER_ALLOWLIST` | Die Allowlist muss bei der Ablösung angepasst werden. |
| `.cargo/mutants.toml` listet `contextra-store` und `contextra-crypto` als Mutationsziele. `contextra-storage` fehlt. | Datei | Anpassung in WP-0.3. → L-14 |
| `tools/reference/generator.py` existiert. Inhalt und Lauffähigkeit sind nicht geprüft. | Dateiprüfung | Kandidat für den Referenzcodierer (Anhang A der Spec). |
| `evidence/` enthält Nachweise aus Vorläufen (u. a. `g-1-nachweis`, `wp-2.3-contextra`). Die Inhalte habe ich nicht gelesen. | Verzeichnisliste | Vor Gate G-0 prüfen, ob Nachweise veraltet sind. |

---

## 2. Ersetzungsstrategie

**Entscheidung S-1: Neuentwicklung von `contextra-storage` nach R8, kein Refaktor des Bestands.**

Begründung: Der Bestand folgt Entwurf R2. Ihm fehlen die Grundbegriffe, auf denen R8 aufbaut: Cluster als Löschdomäne, Gate-Frame im Redo, Anker, Ledger, Branches. Die Abhängigkeiten im Bestand (`blake3`, `aes-gcm-siv` im Feature `testing`) widersprechen R8. Ein Refaktor würde jedes Modul anfassen (Spec-Sofortmaßnahme 2 nennt `redo`, `run`, `blob`, `catalog`, `committer` als formatbetroffen), und das Ergebnis wäre schwerer zu prüfen als ein Neubau mit Referenzcodierer.

**Entscheidung S-2: Kein Code wird übernommen.** Die Spec verlangt das in Sofortmaßnahme (6). Die Primitive CRC-32C, XXH64 und Bloom-Positionen gelten laut `AGENTS.md` als geprüft. Ich übernehme sie nicht, sondern implementiere sie neu und prüfe sie gegen den Referenzcodierer (WP-0.8).

**Entscheidung S-3: Sicherung.** Vor dem Entfernen des Bestands wird ein annotierter Tag `storage-r2-archive` auf den aktuellen Stand gesetzt und nach `origin` gepusht. `contextra-store` bleibt unverändert bis zur Ablösung (SD-46, NZ-19).

**Entscheidung S-4: Ablösung der Konsumenten erfolgt gestaffelt und ohne Doppelschreiben.** Jeder Konsument wird einzeln nach G-P auf `contextra-storage` umgestellt. Während des Parallelbetriebs gibt es keine Synchronisation zwischen den beiden Stores. Ein Konsument wechselt als Ganzes. Das Mapping steht in Abschnitt 8.

**Entscheidung S-5: Datenmigration ist nicht Teil von v1.** NZ-7 schließt Altformat-Kompatibilität aus. Ob es produktive Bestandsdaten in `contextra-store` gibt, ist offen (E-7). Wenn ja, bekommt der Konsument einen eigenen Export über die Konsumentenschicht, kein Kern-Importer.

---

## 3. Lückenregister und Entscheidungen

Jede Lücke hat eine Entscheidung. Der Status zeigt, ob der Plan sie allein schließen kann (**entschieden**), ob der Maintainer zustimmen muss (**Freigabe**) oder ob sie eine Beschaffung oder ein externes Ereignis braucht (**extern**).

| ID | Lücke | Entscheidung | Status |
|---|---|---|---|
| **L-01** | Die Ablösung der neun Konsumenten ist nicht beschrieben. | Gestaffelte Umstellung nach G-P, ohne Doppelschreiben (S-4). Jeder Konsument hat ein Arbeitspaket M-n (Abschnitt 7.4). | entschieden |
| **L-02** | Die API-Abbildung von `contextra-store` auf den Kern fehlt. | Mapping-Tabelle in Abschnitt 8. Interna werden nicht öffentlich. Die wenigen externen Interna-Zugriffe (`sstable`, `compaction`, `scanner`) werden in M-1 bis M-3 durch die Fassade ersetzt. | entschieden |
| **L-03** | `MergeOperator` aus `contextra-store` hat kein Gegenstück (NZ-4). | Konsumenten lösen Merges oben über Read-Modify-Write mit `Guard::Version` oder über die typisierten Merge-Strategien (§8.5). Für `contextra-engine` ist das die relevante Stelle, weil dort `MergeOperator` aktiv genutzt wird. | entschieden, Auswirkung auf M-engine prüfen |
| **L-04** | **Krypto-Abhängigkeiten widersprechen sich.** SD-03 erlaubt `chacha20poly1305`, `blake3` und `zeroize` im Kern. AGENTS D-05 verbietet Krypto-Crates im Kern. §3.2 verlangt Krypto nur über Ports. | Der Kern bleibt krypto-frei. Er verwendet nur `Cipher`, `Mac`, `Kdf` über Port-Traits. `zeroize` ist die einzige Ausnahme, weil sie keine Kryptografie enthält. Produktive Implementierungen liegen in `contextra-crypto`, Test-Referenzen im Feature `testing`. `chacha20poly1305` und `blake3` im Kern entfallen. | **Freigabe** (E-2) |
| **L-05** | **Herkunft des KEK für `keys-file` ist nicht spezifiziert.** Der Kern kennt nur `kek(KekId)`. Eine Passwortableitung bräuchte eine Passwort-KDF (Argon2id oder scrypt), die R8 nicht nennt. | `keys-file` bekommt den KEK aus einer 32-Byte-Schlüsseldatei oder aus einem Adapter für den OS-Schlüsselspeicher. Eine Passwortableitung ist nicht in v1. Falls sie gewünscht wird, kommt eine Argon2id-Implementierung im Adapter hinzu, mit Freigabe nach D-05. | **Freigabe** (E-3) |
| **L-06** | Das Nonce-Budget ist nur in `AGENTS.md` (Rotationsgrenze 2^32 je Schlüssel) beschrieben. R8 nennt es nicht. | Übernehmen: `SealBudget` zählt Seals je Schlüssel-Epoche. Bei Erreichen der Grenze wird eine neue Epoche verlangt (`INV-KEY-3`). Der Zähler ist persistiert, damit ein Neustart ihn nicht zurücksetzt. | entschieden |
| **L-07** | Die Toolchain ist nicht gepinnt (`stable` vs. 1.89.0). | `rust-toolchain.toml` auf `1.89.0` pinnen. `rust-version = "1.89"` bleibt. | entschieden |
| **L-08** | Die Spec verlangt zwei Reviewer für G-0. `AGENTS.md` sagt: keine Pflicht zu zwei Reviewern, Freigaben erteilt der Maintainer (O-1, D-17). | Gate G-0 wird vom Maintainer freigegeben. Die externe Krypto-Prüfung bleibt in P8 als Pflicht. | **Freigabe** (E-1) |
| **L-09** | `AGENTS.md` widerspricht R8 in Gates, Rängen, Regel-IDs und Abschnittsnummern. Verweise sind tot. | In WP-0.1 wird `AGENTS.md` auf R8 umgeschrieben. Tote Verweise werden entfernt. R8 ist die einzige normative Quelle. | entschieden |
| **L-10** | **Ablage der Ports.** R8 sieht `contextra-storage-ports` vor. `AGENTS.md` sieht `contextra-ports::persistence` vor (D-02, D-18). | Neues Crate `contextra-storage-ports`. Begründung: `contextra-ports` ist eine Abhängigkeit vieler Crates. Eine Änderung dort betrifft zahlreiche Dateien und die Isolation des Speicherkerns (Ihr Auftrag) wäre nicht gegeben. `AGENTS.md` D-18 wird damit ersetzt. | **Freigabe** (E-4) |
| **L-11** | Die Traits in `contextra-ports::persistence` passen nicht zu R8 (z. B. `KeyProvider` mit anderer Signatur, fehlendes `Anchor`, `Authority`, `Observer`). | Die R8-Traits werden in `contextra-storage-ports` neu definiert. `contextra-ports::persistence` bleibt unverändert für bestehende Konsumenten. Eine Angleichung ist nach G-P optional. | entschieden |
| **L-12** | `contextra-durable-fs` ist ungenutzt und fehlt in R8. | Adapter `RealFs` wird als Modul `storage_port` in `contextra-durable-fs` implementiert. Der Kern hängt nie davon ab, nur `contextra-storage` in Dev-Tests und die Konsumentenschicht verbinden beides. Vor WP-0.7 prüfe ich, ob `INV-FSYNC-1` deckungsgleich mit F-3 ist. | entschieden, Prüfung in WP-0.7 |
| **L-13** | Die Messung der Hypothesen H-1 bis H-22 hat keine Heimat. | Benchmarks in `benchmarks/contextra-bench/benches/storage_*.rs`. Messbericht nach Schablone (WP-8.6). Keine neue Crate. | entschieden |
| **L-14** | `.cargo/mutants.toml` und `tests/layering.rs` referenzieren `contextra-store`. | WP-0.3 aktualisiert beide. Die Allowlist-Einträge für `contextra-store` bleiben bis zur Ablösung. | entschieden |
| **L-15** | Plattformen sind nicht festgelegt. R8 nennt Linux und Windows (NZ-8), aber macOS fehlt. Auf macOS liefert `fsync` keine Gerätepersistenz. | Zielplattform v1: Linux. macOS ist unterstützt, sobald `RealFs` `F_FULLFSYNC` nutzt (Adapter). Windows ohne volle Durability (NZ-8). | entschieden |
| **L-16** | `SipHash-1-3` für den Guard-Fingerabdruck ist im Stand der Technik nicht als keyed Standard in `std` verfügbar. `DefaultHasher` ist nicht stabil. | Fingerabdruck als keyed `Mac` aus dem Port, auf 64 Bit gekürzt. Er lebt nur im Speicher und wird nie persistiert, also genügt die Stabilität über Prozesse nicht. Die Kosten fallen auf dem Aufruferthread an (§6.3). | entschieden |
| **L-17** | Vergleich von Tags (MAC, AEAD) ohne Seitenkanäle ist nicht festgelegt. | Vergleich nur über den Port-Vertrag. Der Kern implementiert keinen eigenen Tag-Vergleich. Die Konformanzsuite prüft Konstantzeit-Vergleich nicht direkt, sondern Krypto-Review in P8 (Risiko). | entschieden |
| **L-18** | Ein Strong-Anker (TPM, Transparency-Log, Object-Lock) ist nicht beschafft. Ohne ihn gilt Rollback-Schutz nur gegen Dateiänderungen. | v1 nutzt `FileAnchor` (`Weak`), mit `acknowledged_weak` im Receipt. Der Produktivbetrieb braucht einen Strong-Anker (externe Vorbedingung). P1 bis P5 brauchen keinen. | **extern** (E-6) |
| **L-19** | Ein async Konsument (tokio) kann den synchronen Kern nicht direkt aufrufen. | Async-Fassade (NZ-17) liegt im Konsumenten, zum Beispiel in `contextra-engine`, über `spawn_blocking`. Der Kern bleibt synchron. | entschieden |
| **L-20** | Die Lösch-Kosten der Plain-Barriere bei häufigen Löschungen (Risiko 3) sind nicht gemessen. | Metrik `erase_barrier_count` in P6 und Bündelung nach §9.5. Messung in G-E. | entschieden |
| **L-21** | Bloom oder Ribbon für den Filter ist offen (O-13). | Bloom für v1. Ribbon wird nicht vor G-F eingeführt. Die Bloom-Positionen gehören in den Referenzcodierer. | entschieden |
| **L-22** | Der Ledger-Speicherort ist nicht getestet. Ein gemeinsames Volume mit dem Datenverzeichnis ist im Standard gesetzt. | Standard bleibt `<root>/ERASE-LEDGER`. Produktivempfehlung getrenntes Volume. Ein Test prüft, dass der Ledger-Pfad konfigurierbar ist (WP-6.6). | entschieden |

---

## 4. Crates und Abhängigkeiten

| Crate | Rolle | Darf abhängen von | Status |
|---|---|---|---|
| `contextra-storage-ports` | Traits und Werttypen (Fs, Clock, Entropy, Anchor, KeyProvider, Cipher, Mac, Kdf, Checksum, Metrics, Observer, Authority, Pressure); Fehlerkatalog | `std` | **neu**, Freigabe E-4 |
| `contextra-storage` | Kern, `#![forbid(unsafe_code)]`. Module nach §3.1. Feature `testing`: MemFs, FaultFs, Sim, Konformanzsuiten, Orakel | `contextra-storage-ports`, `zeroize`, `crossbeam-skiplist` (hinter Trait), optional `bytes`. Dev: `loom`, `shuttle`, `proptest` | Neuentwicklung |
| `contextra-crypto` (bestehend) | Produktive `Cipher`/`Mac`/`Kdf`-Implementierungen über den Port | `contextra-storage-ports` | Eignung prüfen (WP-0.5) |
| `contextra-keys-file` | Referenz-`KeyProvider` für Einzelhost. Liegt in `crates/contextra-storage/src/keys/file/` als Feature, nicht als eigenes Crate | `contextra-storage-ports` | **Entscheidung**: Modul statt Crate, um die Crate-Zahl klein zu halten (E-5) |
| `contextra-keys-cohort` | optionale Cohort-Strategie für Millionen Cluster | wie `keys-file` | optional, nach G-E, kann entfallen |
| `contextra-durable-fs` (bestehend) | Modul `storage_port` implementiert `Fs` mit `F_FULLFSYNC`/`fsync` | `contextra-storage-ports` | L-12 |
| `contextra-anchor-strong` | TPM, Transparency-Log, Object-Lock | — | **nicht in v1** (E-6) |

**Abhängigkeitsfreigaben (SD-03 bereinigt):** `crossbeam-skiplist`, `bytes`, `zeroize` im Kern. `loom`, `shuttle`, `proptest`, `cargo-mutants` nur als Dev-Abhängigkeiten. `blake3`, `chacha20poly1305` entfallen aus dem Kern (L-04). `siphasher` wird nicht eingeführt (L-16). Jede Aufnahme erfordert einen Eintrag in `deny.toml`.

---

## 5. Komponentenzerlegung (disjunkt)

### 5.1 Regel

Jeder Pfad im Repository gehört genau einer Domäne. Eine Domäne ändert nur die eigenen Pfade. Änderungen an fremden Pfaden laufen über einen Änderungsantrag (IF-Change-Request, Abschnitt 6.2). Gemeinsame Dateien (`lib.rs`, `error.rs`) haben einen Eigentümer; Modulregistrierungen erfolgen per Antrag.

Die Modulstruktur folgt §3.1 der Spec, mit einer Ergänzung: Das Modul `read` trennt Lesepfad und Cache aus `engine`. Das Modul `checkpoint` trennt Checkpoint und Restore aus `recovery`. Das Modul `ext` bündelt die fünf Erweiterungsverträge. Ohne diese Trennung läge der Lesepfad in `engine`, und Recovery-Code wäre zwischen Lese- und Schreibpfad verteilt.

### 5.2 Domänen und Pfadbesitz

| Domäne | Inhalt | Pfade (Besitz) | Phasen |
|---|---|---|---|
| **D0 Ports & Env** | Port-Traits, Werttypen, Fehlerkatalog, Env-Adapter | `crates/contextra-storage-ports/**`, `crates/contextra-storage/src/env/**`, `crates/contextra-durable-fs/src/storage_port/**` | P0 |
| **D1 Codec & Seal** | varint, Tags, CRC32C, Comparator `cmp`, Seal-Rahmen, Header, Limits | `crates/contextra-storage/src/codec/**` | P0, P1, G-F |
| **D2 Sync** | Ranked Locks, Epochen, `Arc`-Publikation, Generationszähler | `crates/contextra-storage/src/sync/**` | P0 |
| **D3 Redo** | Frames, Gruppen, Kettenmac, Segmente, Torn-Tail, Reservesegment | `crates/contextra-storage/src/redo/**` | P1, P2 |
| **D4 Catalog** | Edit-Log, Gruppen-Commit, `CURRENT`, Rollover, Reservierungen | `crates/contextra-storage/src/catalog/**` | P1, P2, P3 |
| **D5 Run & Blob** | Run-Format, DataBlock, SubIndex, Filter, ClusterDirectory, PackMeta, Footer; Blob-Records, Inkarnation | `crates/contextra-storage/src/run/**`, `crates/contextra-storage/src/blob/**` | P2, P4 |
| **D6 Engine (Schreibpfad)** | Committer, Memtable, Stamps, Guards, Idempotenz, Volatile, `DiskFull`, Limits | `crates/contextra-storage/src/engine/{committer,memtable,stamps,guard,idem,volatile,disk,limits}.rs` | P1, P3 |
| **D7 Read** | Views, Version-Publikation, Punktabfrage, Cursor, Page, Block-Cache, `filter_visible` | `crates/contextra-storage/src/read/**`, `crates/contextra-storage/src/engine/version.rs` | P1 (Minimal), P4 |
| **D8 Maintenance** | Flush, Compaction, Claims, Blob-GC, Scheduler/Lanes, Governor, Consumer-Feed-Aufbau | `crates/contextra-storage/src/maint/**` | P2, P5 |
| **D9 Erase** | Gate, Protokoll ER0–ER7, Purge, Links, Restrictions, Ledger-Schreiber, Receipt, `verify` | `crates/contextra-storage/src/erase/**` | P1 (Minimal), P2, P6 |
| **D10 Keys** | `KeyProvider`-Nutzung, Schlüsselcache, Epochen, `keys/file/`, `keys/cohort/` | `crates/contextra-storage/src/keys/**` | P1, P6 |
| **D11 Recovery & Anker & Checkpoint** | Öffnen R1–R10, Anker-Adapter, Checkpoint, Restore | `crates/contextra-storage/src/recovery/**`, `crates/contextra-storage/src/checkpoint/**`, `crates/contextra-storage/src/anchor/**` | P1 (Minimal), P2, P6 |
| **D12 Branches** | `SYS_BRANCH`, Fork, Drop, TTL, Branch-Lesen, Diff, Merge | `crates/contextra-storage/src/branches/**` | P7 |
| **D13 Erweiterungsverträge** | `SegmentOwner`, `ExtensionBlock`, Consumer-Schnittstelle, Watermarks | `crates/contextra-storage/src/ext/**` | P5, P6 |
| **D14 Fassade & API** | `Storage`, `View`, `Eraser`, `Inspector`, Re-Exporte, Fehler-Hints | `crates/contextra-storage/src/api/**`, `crates/contextra-storage/src/lib.rs` | P1 bis P8 |
| **D15 Verifikation** | Orakel, Konformanzsuiten, Testhooks für Simulation, Referenzcodierer, Benchmarks, Tests | `crates/contextra-storage/src/testing/**`, `crates/contextra-storage/tests/**`, `tests/storage/**`, `tools/reference/**`, `tools/storage/**`, `benchmarks/contextra-bench/benches/storage_*.rs`, `formal/**` (TLA+) | alle |
| **D16 Migration** | Konsumenten-Umstellung, außerhalb des Kerns | `crates/contextra-agent/**`, `crates/contextra-checkpoint/**`, `crates/contextra-cognition/**`, `crates/contextra-db/**`, `crates/contextra-engine/**`, `crates/contextra-graph/**`, `crates/contextra-py/**`, `crates/contextra/**`, `benchmarks/contextra-bench/**` (ohne `storage_*`) | nach G-P |

Hinweis zu D0 und `contextra-durable-fs`: Das Modul `storage_port` gehört zu D0, aber liegt in einem fremden Crate. Änderungen an anderen Modulen in `contextra-durable-fs` gehören nicht zu D0.

### 5.3 Abhängigkeitsgraph (nur nach unten)

```
D0 Ports & Env ─┬─► D1 Codec & Seal ─┬─► D3 Redo ──┐
                │                    ├─► D4 Catalog ┤
                │                    └─► D5 Run/Blob┤
                ├─► D2 Sync ──────────────────────────┤
                │                                     ▼
                │                    D6 Engine (Schreibpfad) ──► D7 Read
                │                                     │               │
                │                                     ▼               ▼
                ├─► D10 Keys ──────────────► D8 Maintenance ◄──── D13 Ext
                │                                     │
                │                                     ▼
                │              D9 Erase ◄─── D11 Recovery/Anker/Checkpoint
                │                    │
                │                    ▼
                │              D12 Branches
                └────────────────────────────────────────────────► D14 Fassade ─► D15 Verifikation
                                                                                      │
                                                                             D16 Migration (nach G-P)
```

Zyklen sind verboten. Der Test `layering` prüft das in der Quelltextstruktur (Ring-Regel aus `tests/layering.rs`). Die Domänen D6 und D7 teilen sich `engine/version.rs` nicht: D6 erzeugt Versionen, D7 liest sie. Der Eigentümer ist D7.

### 5.4 Konfliktregeln

Zwei Domänen dürfen nicht gleichzeitig an derselben Datei arbeiten. Wer ein Interface ändert, legt einen Änderungsantrag an, den die betroffenen Domänen gegenzeichnen. Interface-Änderungen nach G-F sind Migrationen (Spec §5.9) und brauchen Maintainer-Freigabe.

---

## 6. Schnittstellen

### 6.1 Verträge

| ID | Vertrag | Besitzer | Verwender | Stabil ab |
|---|---|---|---|---|
| **IF-1** | `Fs`, `File` (Port) | D0 | D3, D4, D5, D9, D11 | P0 |
| **IF-2** | `Anchor` (Port), `AnchorValue` | D0 (Port), D11 (Adapter) | D4, D9, D11 | P1 (Mem), P6 (File) |
| **IF-3** | `KeyProvider` (Port) | D0 | D10, D6, D9 | P0 |
| **IF-4** | `Cipher`, `Mac`, `Kdf`, `Checksum` (Ports) | D0 | D1, D3, D5, D10 | P0 |
| **IF-5** | `SegmentOwner` | D13 | D8, D9 | P5 |
| **IF-6** | `ExtensionBlock` | D13 | D5, D8 | P5 |
| **IF-7** | Fassade `Storage`, `View`, `Eraser` | D14 | Konsumenten | P1 (Minimal) |
| **IF-8** | `Error`, `hint()`, `Receipt` | D0 (Typ), D14 (Fassade) | alle | P0 |

### 6.2 Änderungsantrag

Ein Änderungsantrag nennt Interface-ID, Vorher-Nachher, betroffene Domänen, Auswirkung auf Formate (ja/nein) und ob G-F bereits geschlossen ist. Ohne Gegenzeichnung wird er nicht gemerged. Nach G-F gilt: Formatänderung = Migration.

### 6.3 Auszug der Signaturen

```rust
// IF-7: Fassade (P1-Minimum, Rest gemäß SPEC §4 bis §12)
pub struct Storage { /* intern */ }

impl Storage {
    pub fn open(env: Env, cfg: OpenConfig) -> Result<Storage, Error>;
    pub fn register_cluster(&self, spec: ClusterSpec) -> Result<ClusterId, Error>;
    pub fn apply(&self, change: Change) -> Result<Receipt, Error>;   // Receipt { stamp, durable }
    pub fn view(&self) -> View;
    pub fn eraser(&self) -> Eraser;
}

// IF-2: Anker
pub trait Anchor: Send + Sync {
    fn read(&self) -> Result<AnchorValue, AnchorError>;
    fn advance(&self, expect: AnchorValue, next: AnchorValue) -> Result<(), AnchorError>;
    fn strength(&self) -> AnchorStrength; // Strong | Weak
}

// IF-3: Schlüsselprovider (SPEC §9.4 unverändert)
pub trait KeyProvider: Send + Sync {
    fn info(&self) -> KeyProviderInfo;
    fn kek(&self, id: KekId) -> Result<Key32, KeyError>;
    fn create_cluster_key(&self, c: ClusterId, e: &dyn Entropy) -> Result<u32, KeyError>;
    fn cluster_key(&self, c: ClusterId, epoch: u32) -> Result<Option<Key32>, KeyError>;
    fn destroy_many(&self, cs: &[ClusterId]) -> Result<DestroyReceipt, KeyError>;
    fn probe(&self, c: ClusterId) -> Result<KeyState, KeyError>;
    fn list(&self) -> Result<Vec<ClusterId>, KeyError>;
}
```

Die Guard-Fingerabdrücke (L-16) werden über `Mac` berechnet: `fp = truncate64(Mac(prozessschlüssel, space ‖ cluster ‖ branch ‖ key))`.

---

## 7. Arbeitspakete

### 7.1 Konvention

Jedes Paket hat: ID, Ziel, Domäne, Abhängigkeiten, Umfang, Abnahme. Die Abnahme ist maschinell prüfbar: Tests mit `// INV-…`-Zitat, Messung mit `H-…`, Crash-Sweep oder Modell. Ein Paket gilt erst als abgeschlossen, wenn die CI grün ist (Spec §13.3). Keine Phase beginnt vor dem Gate der vorherigen.

Die Aufwände folgen den Schätzungen der Spec (3 bis 4 Entwickler und 1 Testingenieur). Sie sind Schätzungen, keine Zusage.

### 7.2 Phase P0 — Fundament (Gate G-0, ca. 3 Wochen)

| ID | Ziel | Domäne | Abhängig von | Umfang | Abnahme |
|---|---|---|---|---|---|
| WP-0.1 | `AGENTS.md` auf R8 umschreiben, tote Verweise entfernen, Regelnamen angleichen (L-09) | D15 | — | `AGENTS.md`, `docs/` | Keine Verweise ins Leere (Skript prüft Pfade); Gate-Namen identisch mit R8 |
| WP-0.2 | Altstand sichern und entfernen (S-1 bis S-3) | D15 | — | Tag `storage-r2-archive`, Bestand unter `crates/contextra-storage/src/**` entfernen | Tag existiert auf `origin`; Workspace baut mit leerem Kern-Skelett |
| WP-0.3 | Toolchain-Pin 1.89.0, `deny.toml`, `.cargo/mutants.toml`, `tests/layering.rs` anpassen (L-07, L-14) | D15 | WP-0.2 | `rust-toolchain.toml`, `.cargo/mutants.toml`, `deny.toml`, `tests/layering.rs` | CI läuft mit Pin; `cargo deny` grün; Mutants-Ziel enthält `contextra-storage` |
| WP-0.4 | `contextra-storage-ports` anlegen: Traits, Werttypen, Fehlerkatalog (L-10, L-11) | D0 | WP-0.2 | `crates/contextra-storage-ports/**` | Doc-Tests; Typen nach F-1 bis F-9 vorhanden; Kein `unsafe` (`INV-FMT`, Lint) |
| WP-0.5 | Kern-Skelett: Modulstruktur §3.1 plus `read`, `checkpoint`, `ext`, `branches`; `#![forbid(unsafe_code)]`; `disallowed_methods` für `std::fs`, `SystemTime`, `rand` außerhalb `env` | D14 | WP-0.4 | `crates/contextra-storage/src/lib.rs`, `clippy.toml` | `layering`-Test grün; Lint verbietet verbotene Aufrufe außerhalb `env` |
| WP-0.6 | Env-Adapter: `MemFs`, `FaultFs` (Crash an jedem E/A-Index, Torn Write, verlorener Fsync, ENOSPC beim Vorallokieren, Byte-Zähler), `SimClock`, `SimEntropy` | D0 | WP-0.4 | `crates/contextra-storage/src/env/**`, Feature `testing` | Crash-Sweep-Runner läuft auf MemFs; Byte-Zähler liefert reproduzierbare Werte |
| WP-0.7 | `RealFs` als Modul `storage_port` in `contextra-durable-fs`; Prüfung der Deckungsgleichheit `INV-FSYNC-1` ↔ F-3 (L-12, L-15) | D0 | WP-0.4 | `crates/contextra-durable-fs/src/storage_port/**` | Fsync-Fehler führt zu `Poisoned` ohne Wiederholung; macOS nutzt `F_FULLFSYNC` |
| WP-0.8 | Codec-Primitive neu: varint (kanonisch), CRC32C, Comparator `cmp`; Referenzcodierer `tools/reference/generator.py` prüfen und ergänzen | D1, D15 | WP-0.4 | `codec/**`, `tools/reference/**` | Goldens für Primitive gegen Referenz identisch; nicht-kanonisches varint wird abgewiesen (`INV-FMT-1`) |
| WP-0.9 | Testharness: Invarianten-Zitatprüfung (`tests/invariants.rs` neu, liest die Tabelle §15) | D15 | WP-0.5 | `crates/contextra-storage/tests/invariants.rs` | Jede `INV-*` aus §15 ist in mindestens einem Test zitiert; sonst CI rot |
| WP-0.10 | `InMemoryKeyProvider` und `KeyProviderConformance` (Idempotenz, alle Epochen, `probe`, Crash) | D10, D15 | WP-0.4 | `keys/mem.rs`, `testing/conformance/**` | Suite grün gegen Referenz; Provider-Test als Gate-Bestandteil (`INV-KEY-4`) |
| WP-0.11 | Orakel-Skelett `ErasureOracle` mit Canaries in Schlüssel, Kopf, Körper, Gewichten, Idem-Token, Branch-Namen | D15 | WP-0.6 | `testing/oracle/**` | Orakel findet eingeschleuste Canaries in Test-Dateien; Gegenprobe ohne Canary liefert 0 |
| WP-0.12 | CI einrichten: `check`, `fmt`, `clippy -D warnings`, `test`, `layering`, `loom`, `doc`, `deny`, Sim- und Absturz-Kurzlauf (Spec §13.3) | D15 | WP-0.3 | `.github/workflows/**` | Pipeline grün auf `main` |

**Gate G-0:** Spec freigegeben (E-1), CI grün, Skelett kompiliert, Referenzcodierer für Primitive steht.

### 7.3 Phase P1 — Walking Skeleton (ca. 4 Wochen, provisorische Formate)

| ID | Ziel | Domäne | Abhängig von | Abnahme |
|---|---|---|---|---|
| WP-1.1 | Seal-Rahmen (`SealFrame`, AAD für Redo-Op, Plain-CRC) | D1 | WP-0.8, WP-0.4 | Roundtrip; Rohkopie bei Abschnitt-Umzug gültig; `INV-SEAL-1` |
| WP-1.2 | Redo: Frame, Gruppe, Kettenmac, Vorallokierung, Reservesegment, Torn-Tail-Regel | D3 | WP-1.1, WP-0.6 | `INV-LOG-7`, `INV-LOG-8`; Absturz zwischen Frames (FaultFs) |
| WP-1.3 | Catalog-Minimal: `space_add`, `cluster_register`, `cluster_state`, `anchor_mark`-Platzhalter; Gruppen-Commit, `CURRENT` | D4 | WP-1.2 | `INV-CAT-1`, `INV-CAT-2`; mitten in Gruppe gilt gültiger Präfix |
| WP-1.4 | Committer und Memtable (lock-frei über Trait, `crossbeam-skiplist`), Stamps, `apply` ohne Guards | D6 | WP-1.2, WP-1.3, WP-0.5 | `INV-STAMP-1..3`, `INV-COMMIT-2`, `INV-ATOM-1` (Ein-Space); Linearisierbarkeit auf Einzelschreiber |
| WP-1.5 | Views und Publikation: `visible_stamp` vor `Arc<Version>` | D7, D2 | WP-1.4 | `INV-VIEW-1`, `INV-VIEW-2`; `loom` für Publikation |
| WP-1.6 | Cluster-Register, Gate (ER1), `destroy_many` (ER2), Ledger ohne Anker | D9, D10 | WP-1.4, WP-0.10 | `INV-ERA-1`, `INV-ERA-3`, `INV-ERA-7`; Gate ist Committer-Frame (SD-28) |
| WP-1.7 | Recovery aus Redo: zwei Durchläufe, `next_stamp` nach §6.2 | D11 | WP-1.2, WP-1.4 | `INV-STAMP-3`; Crash-Sweep über alle Fs-Operationen |
| WP-1.8 | Walking-Skeleton-Test: O-Files findet nach `Done` nichts, obwohl Daten nur in Redo und Memtable lagen | D15 | WP-1.6, WP-0.11 | Orakel-Ergebnis 0 Treffer (SPEC §13.2, Phase P1) |

**Gate P1:** Crash-Sweep grün, O-Files nach `Done` leer, Formate provisorisch markiert.

### 7.4 Phase P2 — Runs und Catalog (ca. 5 Wochen)

| ID | Ziel | Domäne | Abhängig von | Abnahme |
|---|---|---|---|---|
| WP-2.1 | Run-Format: `ClusterSection`, `DataBlock`, `SubIndexBlock`, `FilterBlock` (Bloom, L-21), `ClusterDirectory`, `PackMeta`, `Footer` | D5 | WP-1.1 | `INV-PL-2` (abschnittsrelative Offsets); Codec-Proptests |
| WP-2.2 | Flush: Staging-Datei plus Pure-Dateien je Cluster, Retired-Cluster übersprungen, ein Edit | D8 | WP-2.1, WP-1.3 | `INV-PL-1`, `INV-LOG-5`; Retired wird nicht entsiegelt |
| WP-2.3 | Catalog vollständig: alle Ops aus §5.8, Rollover, Reservierung (Vorabanhebung folgt P3) | D4 | WP-1.3 | Unbekannte Tags → `Unsupported` (`INV-FMT-1`); Rollover-Test |
| WP-2.4 | Purge (einziger Dateilöschalgorithmus, Rohkopie), Plain-Barriere (ER4), ER5/ER6 für Plain | D9 | WP-2.2, WP-2.3 | `INV-ERA-2`, `INV-ERA-6`, `INV-LOG-9`; O-Files für Runs grün |
| WP-2.5 | Vollständige Recovery R1 bis R10 (Verwaiste, `.building`, Erase-Wiederaufnahme) | D11 | WP-2.4 | `INV-ID-1`; Absturzmatrix §10.2 (Zeilen für Run, Edit, Unlink) |
| WP-2.6 | Absturzmatrix §10.2 vollständig als Tests | D15 | WP-2.5 | Jede Zeile ein Test; Modelltest M1 und M2 (ohne Branches) geprüft |

**Gate P2 und G-F (Formatfreeze):** Referenzcodierer deckt alle Formate (Freeze-Liste §5.9) ab, goldene Vektoren aus Anhang A, Rohkopie-Test, mindestens 8 Fuzz-Ziele.

| ID | Ziel | Domäne | Abnahme |
|---|---|---|---|
| WP-F.1 | Referenzcodierer vollständig (Python, unabhängig von Rust) | D15 | Anhang A: alle Vektoren erzeugt |
| WP-F.2 | Goldene Vektoren und Vergleich im Test `golden` | D15 | Bytegleich mit Rust-Codec |
| WP-F.3 | Rohkopie-Test (Abschnitt aus Lauf A in Lauf B, Entsiegeln gelingt) | D15 | `INV-SEAL-1` in Praxis |
| WP-F.4 | 8 Fuzz-Ziele: Decoder aller Formate, Recovery auf beschädigten Verzeichnissen | D15 | Bit-Flip-Test: nie stille Falschdaten |
| WP-F.5 | Freeze-Review durch den Maintainer (E-1) | D14 | Freigabe dokumentiert |

### 7.5 Phase P3 — Schreibpfad produktiv (ca. 4 Wochen)

| ID | Ziel | Domäne | Abnahme |
|---|---|---|---|
| WP-3.1 | Gruppen-Commit mit Flat-Combining und einem Sync; Intake (Rang 2) | D6 | `INV-LOCK-1`; keine E/A unter Rang 1 außer Redo-Sync |
| WP-3.2 | Guards zweistufig: Konfliktfenster (1 Mio.), Langsampfad, Fingerabdruck über `Mac` (L-16) | D6 | `INV-GUARD-1`; H-18 (Schnellpfad ≥ 99 %) |
| WP-3.3 | Reservierung `reserve_stamps` (2^20) und `reserve_ids` (2^16), Fallback als einzige Doppelhaltung | D4, D6 | `INV-STAMP-2` (Überlauf `Exhausted`); Fallback-Test |
| WP-3.4 | Volatile-Spaces (kein Redo, Erase = Freigabe mit Überschreiben) | D6 | `INV-VOL-1`; Mischchange nur mit `allow_mixed` |
| WP-3.5 | `DiskFull`-Zustand, Reserve, Hysterese (2× Reserve), `Poisoned` nur bei EIO | D6 | `INV-DISK-1`; Gate und Purge laufen bei voller Platte |
| WP-3.6 | Idempotenz über `IdemToken` mit TTL und `idem_trim` | D6, D4 | `AlreadyApplied`; Tabelle ohne Inhalte (`INV-LOG`) |
| WP-3.7 | Limits vor jeder Sperre, Backpressure außerhalb Sperren | D6 | `INV-LIM-1`; `urgent` umgeht Rückdruck |
| WP-3.8 | Linearisierbarkeit (Wing-Gong) und Historien aus Sim; H-1 | D15, D6 | G-W: Absturzmatrix, Linearisierbarkeit, H-1, H-18 |

**Gate G-W** entspricht Spec-Phase P3.

### 7.6 Phase P4 — Lesepfad (ca. 4 Wochen)

| ID | Ziel | Domäne | Abnahme |
|---|---|---|---|
| WP-4.1 | Blob-Store: Records, Inkarnation, `punch_hole` wenn verfügbar | D5 | `INV-BLOB-1`, `INV-ERA-6` |
| WP-4.2 | Block-Cache mit `(cluster, gen)`, `Zeroizing`, Check-after-decrypt | D7, D10 | `INV-KEY-2`; loom für R4 |
| WP-4.3 | Views mit `ViewPolicy::Revoke`, Graveyard | D7 | `INV-GRAVE-1`, `INV-GRAVE-2` |
| WP-4.4 | `page` mit Gewichten, Cursor, `ResumeToken` (AEAD unter Prozessschlüssel) | D7 | Paging-Properties (Budget, lückenlos, Rückwärts-Gleichheit); `oversize_single` |
| WP-4.5 | `filter_visible` und Extension-Antwortpfad | D7, D13 | `INV-VIS-1`; O-Gate |
| WP-4.6 | Gate-Check zuerst, Blockskipping, Punktabfragen | D7 | H-2, H-3, H-4 |

**Gate G-R** entspricht Spec-Phase P4.

### 7.7 Phase P5 — Wartung (ca. 5 Wochen)

| ID | Ziel | Domäne | Abnahme |
|---|---|---|---|
| WP-5.1 | Compaction je Wartungsgruppe, Claims, `retire_gen`-Prüfung im Edit | D8 | `INV-CLAIM-1`, `INV-PL-3`, `INV-PL-4`; M3 |
| WP-5.2 | Blob-GC: Umzug in neue `blob_id`, Entfernen nur ohne lebende Referenz | D8, D5 | `INV-BLOB-1` |
| WP-5.3 | Scheduler mit Lanes `urgent > flush > compact > blobgc > idle`, Governor | D8 | `urgent` nie blockiert; `INV-LOCK-2` |
| WP-5.4 | Consumer-Feed aus Redo mit `replay_floor` und `retain_floor`; `Lagged` | D13, D8 | `INV-CONS-1`; `Redacted`/`Erased` im Feed |
| WP-5.5 | `SegmentOwner`-Register (ohne `on_gate`/`verify_absent`-Implementierung, folgt in P6) und `ExtensionBlock` | D13 | Vertrag geprüft gegen Testbesitzer |
| WP-5.6 | Schlüsselepochen-Rotation und `SealBudget` (L-06) | D10 | `INV-KEY-3`; Zähler überlebt Neustart |

**Gate G-M** entspricht Spec-Phase P5.

### 7.8 Phase P6 — Löschung vollständig (ca. 6 Wochen, größter Block)

| ID | Ziel | Domäne | Abnahme |
|---|---|---|---|
| WP-6.1 | Links (`link(derived, sources ≤ 8)`), Link-Hülle im Committer neu berechnet, Retry | D9 | `INV-ERA-12`; R10 |
| WP-6.2 | Restrictions mit `LiftPolicy` (`OwnerOrAdmin`, `FourEyes` über `Authority`); Durchsetzung dreischichtig | D9 | `INV-ERA-10`; R9; AT-6 |
| WP-6.3 | Pack-Ablauf (`pack_expired`), Packgruppen-Abwurf, Mandantenabgang | D9 | Abwurf ist O(Dateien); H-7, H-8, H-9 |
| WP-6.4 | Space-Drop (Gate `Space`, Barriere, ein Edit, DEK entfernen) | D9 | R14; `SpaceId` nie wiederverwendet (`INV-ID-2`) |
| WP-6.5 | `Eraser::range` mit `EraseRange`-Marker und Lokalität | D9 | `INV-ERA-5`, `INV-ERA-11` (Byte-Zähler ×1, ×10, ×100) |
| WP-6.6 | Ledger vollständig (`tomb_id`, Kinds, Kette, `ledger_dir`); Anker (`FileAnchor`, Weak), `AnchorPolicy` | D11, D9 | `INV-LED-1`, `INV-ANCH-1`; R12 (Anker nie vor Edit); L-22 |
| WP-6.7 | Checkpoint (Flush-Barriere, Manifest, `KEYS` enthalten, KEK nicht) und Restore (`RestoreQuarantine`, `tomb_id`-Abgleich) | D11 | `INV-KEY-1`, `INV-ERA-9`; O-Restore |
| WP-6.8 | `keys/file`-Modul (Referenz-KeyProvider) mit KEK aus Datei oder OS-Schlüsselspeicher (L-05) | D10 | `KeyProviderConformance` grün; `INV-KEY-3`, `INV-KEY-4` |
| WP-6.9 | `keys/cohort` (optional) | D10 | Konformanz; kann nach G-E entfallen |
| WP-6.10 | `SegmentOwner` vollständig: `on_gate`, `rewrite_without`, `verify_absent`, `watermark`; `SegmentIo` versiegelt | D13 | `INV-ERA-8` (alle Besitzer bestätigt); Timeout-Alarm, nie überspringen |
| WP-6.11 | Receipt mit kanonischen Bytes, `verify_receipt` offline, `Eraser::verify` strukturell | D9 | Kanonische Bytes stabil; Verdict-Fälle für Kette, Anker, Reihenfolge, Zeit |
| WP-6.12 | Orakel vollständig (O-Files, O-Device, O-Gate, O-Ledger, O-Segments, O-Restore, O-Survivors) | D15 | 0 Treffer nach `Done`; O-Survivors bytegleich |

**Gate G-E** entspricht Spec-Phase P6: M2, M4, M5, M6 geprüft, Orakel 0 Treffer, INV-ERA-11, H-6 bis H-11, H-17, Konformanz grün.

### 7.9 Phase P7 — Versionierung (ca. 5 Wochen)

| ID | Ziel | Domäne | Abnahme |
|---|---|---|---|
| WP-7.1 | Feature-Bit `BRANCHES` in `FORMAT`; `SYS_BRANCH` als reservierter Space | D12, D1 | Format-Fehler bei `branch ≠ 0` ohne Bit |
| WP-7.2 | Fork (O(1), ein Control-Frame), Drop, TTL, Branch-Limits | D12 | `INV-BR-4`; H-20 |
| WP-7.3 | Branch-Lesen mit Stamp-Kappen (k-Wege-Merge) | D12 | `INV-BR-1`; H-21; Modellgleichheit |
| WP-7.4 | Diff über Clusterschlüssel und Branch-Einträge | D12 | Diff ∝ Einträge |
| WP-7.5 | Merge: Berechnung, Ergebnis `Merged` oder `Conflicts`, Anwendung als ein Frame | D12 | `INV-BR-2`; R15, R17; H-22 |
| WP-7.6 | `Eraser::branch` und Fork-Floor in `history_floor` | D12, D9 | `INV-BR-3`; R16; M3 mit Branches |
| WP-7.7 | Branch-Canaries im Orakel | D15 | Orakel mit Branch-Canaries 0 Treffer |

**Gate G-V** entspricht Spec-Phase P7.

### 7.10 Phase P8 — Härtung (ca. 5 Wochen)

| ID | Ziel | Domäne | Abnahme |
|---|---|---|---|
| WP-8.1 | Soak 72 h, Crash-Loop, Platte voll | D15 | Keine Invariantenverletzung |
| WP-8.2 | Dauer-Fuzz, Mutation (`cargo-mutants`) auf `codec`, `recovery`, `erase`, `committer`, `branches` | D15 | H-16 ≥ 80 %; H-15 Zeilen ≥ 90 %, Zweige ≥ 85 % |
| WP-8.3 | Fehlerinjektion und Fassade vollständig | D14 | Alle `Error::hint()` belegt |
| WP-8.4 | Externes Krypto-Review (E-8) | D0, D1, D10 | Bericht liegt vor, Befunde geschlossen |
| WP-8.5 | Bedrohungsmodell-Review | D15 | Restexposition §9.9 bestätigt |
| WP-8.6 | Messbericht nach §17: Hardware, Dateisystem, Last, Methode, Streuung | D15 | Jede H-Hypothese bestätigt oder korrigiert; keine Verfahrensänderung |

**Gate G-P** entspricht Spec-Phase P8: alle INV getestet (100 %), Messbericht vorliegend, keine offenen Korruptionsbefunde.

### 7.11 Migration (nach G-P)

| ID | Ziel | Domäne | Abnahme |
|---|---|---|---|
| M-1 | `contextra-engine` auf `contextra-storage` umstellen (größter Konsument, `MergeOperator`, `TenantScopedStorage`) | D16 | Tests des Konsumenten grün; Layering-Allowlist-Eintrag entfernt |
| M-2 | `contextra-db`, `contextra-graph`, `contextra-checkpoint` | D16 | Gleiche Abnahme; Dev-Abhängigkeit aus Allowlist entfernt |
| M-3 | `contextra-agent`, `contextra-cognition`, `contextra-py`, `contextra` (Umbrella) | D16 | Gleiche Abnahme. `contextra-py` hat eine Cargo-Abhängigkeit, aber keine direkte Nutzung im Quelltext (0 Treffer). Zuerst prüfen, ob die Abhängigkeit entfallen kann. |
| M-4 | `benchmarks/contextra-bench` auf neue Benchmarks umstellen | D16 | Alte Benchmarks laufen auf neuer API; Vergleich mit `contextra-store` dokumentiert |
| M-5 | `contextra-store` aus Workspace entfernen, Tag `legacy-store-final` setzen | D16 | Kein Konsument mehr; Workspace baut |

---

## 8. Mapping `contextra-store` → `contextra-storage`

| Altes API-Element (Konsumenten) | Nutzung | Ziel in R8 | Anmerkung |
|---|---|---|---|
| `LsmStorage` | Hauptobjekt | `Storage` (IF-7) | Umbenennung und neue Öffnungslogik |
| `LsmConfig`, `DurabilityMode` | Konfiguration | `OpenConfig`, `Durability { Durable, Relaxed }` (K-19) | `Relaxed` nie für Löschung |
| `KvDeleteMode` | Löschmodus | `Storage::apply` mit `Delete` oder `RangeDelete` | Löschung über `Eraser` für Lösch-Semantik |
| `MergeOperator` | Merge-Logik | nicht im Kern (NZ-4) | Konsument: Read-Modify-Write mit `Guard::Version`, oder typisierte Strategie oben (L-03) |
| `TenantScopedStorage` | Mandantenpräfix | Cluster je Person bzw. Fall (§12.7) | Mandantenkennung wird Cluster-Zuordnung, nicht Schlüsselpräfix |
| `WalObserver`, `CommittedBatch`, `WriteOrigin` | Beobachtung (außerhalb des Crates ungenutzt) | Change-Feed (§12.3, Consumer) | `Redacted` und `Erased` sind neue Ereignisse; kein Migrationsaufwand außerhalb |
| `SystemPressure`, `PressureLevel` | Druck | `Pressure`-Port, `DiskFull` | Governor liegt im Kern |
| `CompactionConfig` | Wartung | Konfiguration §16 | Werte neu messen |
| `scanner::lsm_scanners`, `lsm_purgers` | Scans, Löschläufe (1 externe Datei) | `Inspector::clusters()`, `Eraser` | Kein clusterübergreifender Scan im Kern (§7.9) |
| `wal::FAIL_APPEND_FOR_TX`, `DELAY_APPEND_FOR_TX`, `DELAY_APPEND_MS` | Testhooks, nur intern | `FaultFs` (Feature `testing`) | Testhooks gehören nicht in die öffentliche API |
| `wal::legacy_fallback` | Altformat, nur intern | entfällt (NZ-7) | Keine Kompatibilität |
| `wal::{Wal, WalEntry, WalOp, WalConfig, WalVersion}` | intern, außerhalb ungenutzt | nicht öffentlich | kein Migrationsaufwand außerhalb |
| `sstable::{SstableBuilder, SstableReader, BlockCache}` | intern, eine externe Datei | nicht öffentlich | Umbau auf Fassade in M-n |
| `manifest::{Manifest, ManifestEntry}` | intern, außerhalb ungenutzt | nicht öffentlich | kein Migrationsaufwand außerhalb |
| `compaction::{CompactionConfig, CompactionEngine}` | intern, eine externe Datei | `maint` (intern) | Umbau auf Fassade in M-n |
| `memtable::MemTable` | intern, außerhalb ungenutzt | `engine::memtable` (intern) | kein Migrationsaufwand außerhalb |

Konsequenz für die Umstellung: Die eigentliche Arbeit liegt in der Fassade-Nutzung von `contextra-engine` (144 Treffer, größter Konsument, M-1). Die übrigen Konsumenten haben zwischen 2 und 25 Treffer. Wenige Konsumenten nutzen Interna (`sstable`, `compaction`, `scanner`, je eine Datei); deren Umbau steht in M-1 bis M-3. Die Migrationsarbeit wächst mit der Zahl der Fassade-Aufrufe, nicht mit der Zahl der Crates.

---

## 9. Gates, Reihenfolge und kritischer Pfad

Die Reihenfolge ist G-0, P1, P2, G-F, P3 (G-W), P4 (G-R), P5 (G-M), P6 (G-E), P7 (G-V), P8 (G-P), M-1 bis M-5.

Der kritische Pfad läuft über den Redo, den Catalog und den Committer bis zur Löschung. P6 ist der größte Block und hängt an allen vorherigen Phasen. Die Spec schätzt die Summe auf 41 Wochen mit 3 bis 4 Entwicklern. Diese Schätzung halte ich für optimistisch, weil sie weder Konsumentenmigration noch Messung oder externes Krypto-Review enthält. Realistisch sind mehr als 41 Wochen, die Spanne habe ich nicht belastbar bestimmt.

Parallelisierbar sind P4 und P5 nach G-W (Lese- und Wartungspfad teilen nur `Version`). P7 kann nach G-E beginnen, nicht früher, weil Branch-Lesen den Lesepfad braucht.

---

## 10. Risiken und Restannahmen

| ID | Risiko | Gegenmaßnahme |
|---|---|---|
| R-1 | Die Konsumenten-Migration ist größer als der Kern, insbesondere `contextra-engine` mit `MergeOperator` und `TenantScopedStorage` | Frühe Prüfung in WP-0.1: Konsumentenmatrix gegen den Code. Umbau-Aufwand schätzen, bevor G-0 fällt. |
| R-2 | Kryptografische Abhängigkeiten sind in `contextra-crypto` im Umbruch (AES-GCM-SIV und XChaCha nebeneinander) | `Cipher`-Port hält den Kern unabhängig. WP-0.5 prüft, welche Implementierung produktiv wird. |
| R-3 | Der Referenzcodierer (`tools/reference/generator.py`) ist ungeprüft | WP-0.8 prüft ihn, bevor er Goldens liefert. Ein Fehler im Referenzcodierer verfälscht alle Formate. |
| R-4 | Die Spec nennt Restrisiken, die der Kern nicht lösen kann (Prozessspeicher, Swap, Flash, Backups) | Jedes Receipt nennt sie (§9.9). Kundenkommunikation ist eine Vorbedingung, kein Kernthema. |
| R-5 | Zeitschätzung ist optimistisch (siehe Abschnitt 9) | Gates sind strikt. Kein Gate wird übersprungen, auch nicht zeitbedingt. |
| R-6 | `contextra-durable-fs` ist ein Baustein ohne Konsumenten; sein Fsync-Verhalten ist für den Kern ungeprüft | WP-0.7 prüft Deckungsgleichheit, bevor `RealFs` produktiv wird. |
| R-7 | `AGENTS.md`-Regeln und R8 konkurrieren, bis WP-0.1 erledigt ist | WP-0.1 ist vor allen Code-Paketen einzuplanen. |

**Nicht verifiziert (ehrlich benannt):**

- Der Build ist nicht geprüft. In der Arbeitsumgebung ist keine Rust-Toolchain installiert. Die Aussagen zu Crate-Abhängigkeiten, Lint-Konfiguration und Layering beruhen auf dem Lesen der Dateien, nicht auf einem `cargo`-Lauf.
- Der Inhalt von `evidence/`, `tools/reference/generator.py` und dem Bestand `crates/contextra-storage` wurde nur oberflächlich gelesen, nicht vollständig geprüft.
- Die Messwerte in R7-Anhang B (Datum 22.11.2026) habe ich nicht gesehen. Die Spec sagt selbst, dass sie nicht neu erhoben wurden.
- Die Ablösung einzelner Konsumenten habe ich nicht im Detail geprüft. Das Mapping in Abschnitt 8 stützt sich auf die Grep-Treffer, nicht auf Aufrufstellen.

---

## 11. Entscheidungen des Maintainers

Die folgenden Punkte kann dieses Dokument nicht selbst entscheiden. Zu jedem steht eine Empfehlung.

| ID | Frage | Empfehlung | Warum der Maintainer |
|---|---|---|---|
| **E-1** | Gate G-0: Freigabe durch den Maintainer allein, oder zusätzlich ein zweiter Reviewer (SPEC) | Maintainer-Freigabe, plus externe Krypto-Prüfung in WP-8.4 | Widerspruch zwischen SPEC und `AGENTS.md` (L-08) |
| **E-2** | Krypto-Abhängigkeiten im Kern: SD-03 oder D-05 | D-05: Kern krypto-frei, nur Port-Traits (L-04) | Abhängigkeitsfreigabe |
| **E-3** | Passwortableitung für `keys-file` (Argon2id) ja oder nein | Nein in v1. KEK aus Datei oder OS-Schlüsselspeicher (L-05) | Neue Abhängigkeit und Betriebsmodell |
| **E-4** | Neues Crate `contextra-storage-ports` oder Ablage in `contextra-ports::persistence` | Neues Crate (L-10). `AGENTS.md` D-18 wird aufgehoben. | Ersetzt eine bestehende Regel |
| **E-5** | `keys-file` als Modul oder als eigenes Crate | Modul im Kern-Crate, Feature `keys-file` | Hält die Crate-Zahl klein |
| **E-6** | Strong-Anker: Beschaffung für den Produktivbetrieb | Vor G-E klären, nicht vor P5 nötig (L-18) | Externe Beschaffung |
| **E-7** | Gibt es produktive Bestandsdaten in `contextra-store`? | Bitte bestätigen. Davon hängt ab, ob eine Exportroute nötig ist (S-5). | Ohne Antwort ist die Migrationsplanung unvollständig |
| **E-8** | Externes Krypto-Review: wer, wann, mit welchem Umfang | Vor G-P beauftragen; Umfang: Seal, Ketten-MAC, Nonce-Budget, Schlüsselvernichtung | Externe Vorbedingung |

---

## Anhang: Gesamtzahl der Arbeitspakete

| Phase | Anzahl |
|---|---|
| P0 | 12 |
| P1 | 8 |
| P2 | 6 (plus 5 Freeze-Pakete WP-F.1 bis F.5) |
| P3 | 8 |
| P4 | 6 |
| P5 | 6 |
| P6 | 12 |
| P7 | 7 |
| P8 | 6 |
| Migration | 5 |
| **Gesamt** | **81** |
