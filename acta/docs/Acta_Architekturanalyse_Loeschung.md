# Acta – Architekturanalyse: Löschbarkeit als Konstruktionsprinzip

*Principal-Review · Repository `tfufuz1/acta`, HEAD `bbfad85` (ein Squash-Commit, 2026-11-14) · Quelle: `docs/` (SPEC 2.0-E, Crypto-Shredding-Synthese, 5 DSGVO-Analysen, Interface-Spezifikation) und Quelltext-Stichproben · nichts kompiliert, nichts installiert*

**Beweismarken:** **[V]** am Quelltext des HEAD geprüft (Pfad genannt) · **[D]** aus Dokument übernommen · **[R]** Empfehlung dieser Analyse · **[A]** Annahme, vor Umsetzung prüfen.

---

## 0. Ergebnis in zehn Zeilen

1. Die Mission (jede Personendatenspur atomar entfernbar) ist in `docs/SPEC.md` architektonisch weitgehend richtig gelöst. **Der Code erfüllt sie heute nicht**: Die Löschkette existiert nur als isolierter Kern ohne Verbindung zum Speicher.
2. **Neu gegenüber der SPEC [V]:** `VaultSealer::resolve_dek` fällt bei *jedem* Ableitungsfehler (auch bei „Slot gelöscht“) auf `blake3::keyed_hash(k_unit, key)` zurück; `VaultSealer::new()` setzt `k_unit = [0x55; 32]` ohne Tresor. Damit ist Crypto-Shredding im vorhandenen Sealer **wirkungslos und fehlerhaft maskiert** (siehe V-01). Das ist wichtiger als B-02/B-10 der SPEC.
3. **Neu [V]:** Tresor-Slot-Payloads T/C/S/D liegen im Klartext in der Datei (nur Slot-MAC). Epochen-Maskierung (SPEC §5.4) ist nicht implementiert. Jede Kopie des Tresors ist bis dahin eine vollwertige Schlüsselkopie (V-04).
4. **Konstruktionsfehler in der SPEC:** Schritt E1 (Nadelmenge aus Klartext erzeugen) liegt *vor* dem Gate und ist O(Dokumente). Das bricht die Zusage „O(1) bis `Gated`“ (F-1).
5. **Größter Hebel für „maximal einfache Löschung“:** Das Schlüsselfeld `cluster` aus KeyV2 ist in der SPEC nirgends definiert. Setzt man `cluster = subject_tag` und führt dichte Dokument-Ordinale mit Roaring-Bitmaps als einheitliche Lineage ein, wird jede Schicht (Gate, Postings, HNSW, Graph, SST) mit *einer* Mengenoperation adressiert (F-3, F-4).
6. D-02 (Schlüsselvernichtung = Commit-Punkt) und die Synthese (Klartext-Purge *vor* Schlüsselvernichtung) widersprechen sich. Auflösung: Invariante „Verschlüsselung je Record, nie je Block“ plus Pflicht-Ack von `on_gate` (RAM genullt) als Vorbedingung von E3 (F-2).
7. Segment-`ArtKey` je Ableitungssegment (SPEC §6.12) erzeugt bei Sofort-Vernichtung einen Verfügbarkeitseinbruch: ein einzelner Betroffener macht bis zu 50.000 fremde Vektoren unlesbar. Besser: *Rebuild-dann-Vernichten* mit maskierter Zwischenphase (F-5).
8. Mindestens vier SPEC-interne Inkonsistenzen und fehlende Kapitel (§13, §14, Anhang A) erfüllen das eigene Gate G-DOC nicht (Abschnitt 6).
9. Empfohlene Reihenfolge: **Walking Skeleton zuerst** (Ingest → Siegeln → Store → ein Index → Erase → Messung → Kanarien-Test), dann Breite. Die SPEC schiebt den ersten Ende-zu-Ende-Beweis auf Phase 6.
10. Wording: „DSGVO-konform“ (Produktbeschreibung) ist laut Synthese §12 und SPEC §1.3 als Aussage unzulässig. Zulässig: gemessene Klassen mit deklarierten Grenzen.

---

## 1. Methodik und Beweisstand

| Gegenstand | Umfang |
| --- | --- |
| Gelesen | `docs/SPEC.md` (vollständig, §0–§12), `AGENTS.md`, Synthese (§0–§6, §10–§13, §15), Überschriftenstruktur aller weiteren DSGVO-Dokumente, Interface-Spezifikation (Struktur, 910 KB, automatisch generiert, als Index genutzt) |
| Am Code geprüft | `acta-erasure` (vault, saga, gate, revocation_log), `acta-crypto` (aead, dek), `acta-store` (tenant_codec, retention, history, fence, util, manifest-Treffer), `acta-mvcc` (revoke_covering), `acta-checkpoint` (hardlink_cloner), `acta-vector`/`-graph` (Purger-Köpfe), Workspace-`Cargo.toml` |
| Nicht geprüft | Laufverhalten, Tests, WAL-Replay, SST-Reader, Compaction im Detail, Python/MCP-Schicht, Vektor-/Text-Persistenzformate im Detail |
| Doku-Duplikat | `docs/Crypto-Shredding_RAG_Rechts-und-Architektursynthese.md` ist byte-identisch zur Kopie in `docs/dsgvo/` (`diff` ohne Ausgabe) [V] |
| Baseline-Drift | SPEC bezieht sich auf HEAD `ea1de2c`; geklont ist `bbfad85`. Die Befunde B-01, B-02, B-03, B-04, B-07, B-10 wurden am neuen HEAD **bestätigt**; B-08 ist **teilweise behoben** (V-07) |
| Rechtliche Aussagen | Nur Übernahme aus den `docs/`-Dokumenten. Keine Rechtsberatung. |

---

## 2. Löscharten in `docs/` im Vergleich (Auftrag aus AGENTS.md)

Bewertungsmaßstab: SDM Baustein 60 (Unkenntlichmachen, Informationsgehalt nicht oder nur unwahrscheinlich reproduzierbar, keine Kopie mehr), Nachweispflicht Art. 5 Abs. 2, Zeitreise-Verträglichkeit (AGENTS.md Ziel 2).

| Art | Wirkung | Zeit bis wirksam | Klasse (Synthese) | Stärke | Schwäche | Rolle in Acta |
| --- | --- | --- | --- | --- | --- | --- |
| MVCC-Tombstone (`TOMBSTONE_BIT`, Soft Delete) | Version unsichtbar, Bytes bleiben (Ghost Vector bei HNSW) | O(1) | L | Zeitreise-konform | genügt SDM nicht; **keine Löschung im Sinne der Spec** (SPEC §6.1) | nur MVCC-Sichtbarkeit |
| Logisches Gate (`ErasedSet`, ReadGate) | alle Lesepfade gesperrt | O(1) | L | sofort, auch gegen `as_of` | Bytes unverändert | E2, **immer zuerst** |
| Schlüsselvernichtung je Einheit (Crypto-Shred) | Chiffretext ohne Schlüssel | O(1) je Slot | C | wirkt auf SSD, WAL, Backups, SST gleichzeitig | rechtlich offen; Restrisiko Algorithmenbruch (EDPB 02/2025 Rn. 51); wirkt nicht auf Klartext im RAM | E3, Commit-Punkt (D-02) |
| Pauschales Scope-Key-Shredding (Mandant/Collection) | Kollateralschaden oder unterbliebene Löschung | O(1) | C mit Kollateral | einfach | Granularität falsch (Synthese 3.5 Nr. 1) | nur Ziele Collection/Tenant |
| Physisches Überschreiben | Bytes ersetzt | O(Datenmenge) | P | SDM-genannte Methode | unzuverlässig auf Flash/CoW/Snapshots (D-15) | Hygiene, kein Beweis |
| Klartext-Purge Ebene A (Nullung, Segment-Rewrite) | RAM-Strukturen bereinigt | O(Treffer) | P | einzige Wirkung auf RAM-Indizes | braucht Lineage | `on_gate` + Purge-Fenster |
| Gefilterter SST-/WAL-Rewrite | Chiffretext physisch entfernt | O(betroffene Dateien) | P | Klasse P belegbar | I/O-Last, Crash-Fenster (C7, C8) | Hygiene, nachgelagert |
| Anonymisierung („Minus zur Löschung“) | Personenbezug entfernt | variabel | – | von Instanzgerichten anerkannt (◐) | EDPB skeptisch; Praxis oft nur Pseudonymisierung | Ebene N (Chronik), nicht für P |
| Einschränkung (`Restricted`, Art. 18) | Sperre, keine Vernichtung | O(1) | L | Aufbewahrungspflicht-konform | darf nicht „gelöscht“ heißen (Synthese 2.5) | eigener Zustand |
| Supersede/Replace (Art. 16) | alte Version weg, Schlüssel bleibt | O(Versionen) | P, nicht C | Berichtigung | Klasse K nicht erreicht (`SupersededWithSameKey`) | ehrlich berichtet |
| Expired (Sweeper) | zeitgesteuert | je Einheit | wie Subject/Doc | Art. 5 Abs. 1 lit. e | braucht `HorizonTable` | Sweeper |
| Takedown in N | Payload-Key vernichten, Hash-Kette bleibt | O(1) | C | einzige N-Ausnahme | Vier-Augen nötig | §8.5 |
| Restore-Replay (`replay_erasures`) | Gelöschtes nach Rücksicherung erneut löschen | O(Ledger) | G, K, P | SDM M60.S10 | braucht Ledger außerhalb des Datenverzeichnisses | RestoreGuard |

**Befund zur Taxonomie:** Die Synthese nutzt Klassen **L/P/C/E**, die SPEC nutzt **G/K/P/D/C**. Beide bezeichnen verschiedene Schnitte (Synthese: nach Ebene A/B; SPEC: nach Zone). Es braucht *ein* Berichtsvokabular. [R] SPEC-Klassen G, K, P, D, C beibehalten (feiner) und in §7.8 eine Abbildungstabelle L→G, C→K+C, P→P+D, E→„alle erreicht“ ergänzen, sonst sind Berichte gegenüber der Rechtsgrundlage nicht lesbar.

---

## 3. Verifizierte Befunde am HEAD `bbfad85`

Schwere: **K** kritisch, **H** hoch, **M** mittel. „SPEC“ = Referenz auf Befund-ID in `docs/SPEC.md` §2.2.

| ID | Schw. | Befund | Beleg [V] |
| --- | --- | --- | --- |
| **V-01** | K | **Shredding im Sealer wirkungslos und fehlermaskiert.** `resolve_dek` baut `DocumentHierarchyTags{tenant_tag = coll_tag = doc_tag, subject_tag = None}` (Hierarchie kollabiert). Schlägt `derive_dek_from_vault` fehl, auch bei „Slot missing (erased)“, liefert der `else`-Zweig `blake3::keyed_hash(&self.k_unit, key)`. `VaultSealer::new()` hat `vault: None` und `k_unit = [0x55u8; 32]`; dort ist **jeder** DEK aus einer Konstante und dem Schlüssel reproduzierbar. Nach Slot-Vernichtung liefert `open` keinen Fehler `Erased`, sondern (ohne Tag) **stillen Datenmüll**. | `acta-erasure/src/vault/sealer.rs` Z. 98–105, 249–285 |
| **V-02** | K | Siegel = byteweises XOR mit 32-Byte-Strom aus `blake3::keyed_hash(dek, "ctx/envelope/stream")`; keine Nonce, kein Tag, keine AAD; `SealCtx` ignoriert (`_ctx`). Gleiche DEK ⇒ gleicher Strom für alle Records (Zwei-Klartext-Angriff trivial). | `vault/sealer.rs` Z. 299–345 (SPEC B-02 bestätigt) |
| **V-03** | K | Löschkern isoliert: `acta-erasure` ist weder in `members` noch in `default-members` des Root-`Cargo.toml`; einziger Implementierer von `ErasureLayer` ist `MockLayer` in einem Test; Features-Default enthält weiterhin `async` (tokio). | `Cargo.toml`, `types_local.rs:1045`, `tests/saga_core_orchestration.rs:245`, `acta-erasure/Cargo.toml` (SPEC B-01, B-05, B-15 bestätigt) |
| **V-04** | K | **Tresor-Salts im Klartext, Rotation fehlt.** `derive_dek_from_vault` nutzt `sd.payload` der Slots T/C/S/D direkt als Salt; Maskierung unter `K_enc` existiert nur für F-Slots (`mask_file_key`, XOR mit HKDF-Strom). `mask_epoch_key` ist definiert und re-exportiert, **ohne Aufrufer**. Folge: Eine alte Tresor-Kopie plus der langlebige `K_unit` leitet jede „vernichtete“ DEK erneut ab (Backup-Wiederbelebung). SPEC §5.4 setzt genau das als gelöst voraus. | `vault/sealer.rs` Z. 204–233, `vault/keyring.rs` Z. 33–55, `vault/slot.rs:208`, Grep `mask_epoch_key` |
| **V-05** | H | Fehlerpfade im Schlüsselpfad (S7 verletzt): `seal_record` endet auf `unwrap_or_default()`; `derive_dek` verwirft HKDF-Fehler (`let _ =`) und füllt fehlenden S-Salt mit `[0u8; 32]`; `ErasureConfig::default().k_idx = [0u8; 32]`; `mask_file_key` fällt bei HKDF-Fehler auf BLAKE3 zurück; `VaultKeyring::new_in_memory` nutzt `k_enc = [0x42; 32]`; `seal_record_with_nonce` nimmt 12-Byte-Nonce vom Aufrufer. | `acta-crypto/src/aead.rs:24-71`, `dek.rs:68-89`, `acta-erasure/src/saga/core.rs:72`, `vault/keyring.rs:40-45,78` (SPEC B-10 bestätigt, erweitert) |
| **V-06** | H | Store verschlüsselt keine Werte: `RecordSealer`/`RecordAuthenticator` sind Traits mit Dummy-Implementierungen nur in Tests. Schlüssel sind ASCII `t:{tenant}:{collection}:chunk:{doc}` und `…:graph:{id}`. | `acta-store/src/util.rs:47-135`, `tenant_codec.rs:37-100` (SPEC B-03, B-04 bestätigt) |
| **V-07** | M | **Retention teilweise umgesetzt:** `history.rs` (`Retention`, `RetentionPolicy`, `ReadOutcome`, `check_horizon`) existiert und `lsm/ops/compaction.rs` ruft `retain_versions_with_policy`; `ManifestEntry::SeqHwm` existiert. Aber `PartitionId = u32` mit `DEFAULT_PARTITION`, Kommentar „solange der Store nur eine physische Partition kennt“: **keine Klassentrennung N/P/D**, kein `ReadGate` in `may_drop_version`. Alte `retain_key_versions` bleibt als toter Code. | `acta-store/src/history.rs`, `lsm/ops/compaction.rs:11,154`, `manifest/entry.rs:47` (SPEC B-08, B-12 teilweise behoben) |
| **V-08** | H | `FlushPurgeFence::enter_purge` hat außerhalb der Definition keinen Aufrufer. | `acta-store/src/lsm/fence.rs:59`, Grep (SPEC B-07 bestätigt) |
| **V-09** | H | Lineage-Konzepte fehlen vollständig: 0 Treffer für `source_tags`, `ArtKey`, `lineage`, `tag_set`, `CopyRegistry` in `crates/**/*.rs`. Die Zusagen S10 (Lineage) und S9 (Kopien) sind **nicht anbaubar**, bevor das Datenmodell sie führt. | Grep über `crates/` |
| **V-10** | H | Checkpoints: `CheckpointHardlinkCloner` verlinkt SSTs oder fällt auf `fs::copy` zurück; keine Registrierung. | `acta-checkpoint/src/hardlink_cloner.rs:194-250` (SPEC B-11 bestätigt) |
| **V-11** | N (positiv) | **Code ist besser als SPEC §9.5:** `tombstone_image` nullt `tag` und `payload` (alle Bytes 0 außer `status = 0x02`, MAC gültig). Die SPEC verlangt „`tag` bleibt (Sondierung)“. Ein bleibendes Tag wäre ein dauerhaftes Pseudonym des Gelöschten (Linkability). Lineare Sondierung braucht nur `status`. **SPEC korrigieren, nicht den Code.** | `vault/slot.rs:294-306`, `vault/durable.rs:475-530` |
| **V-12** | M | `FileVault::destroy` schreibt Meta und ruft `sync_data` **je Slot**. Für `Expired`-Sweeps (viele D-Slots) fehlt ein Batch (`destroy_many` mit einem Sync je Shard). | `vault/durable.rs:480-530` |

---

## 4. Architekturanalyse: Was am Zielbild trägt, was nicht

### 4.1 Tragende Entscheidungen der SPEC (beibehalten)

- **D-02** (Commit-Punkt = Schlüsselvernichtung) und **E2 vor E3** (Gate dauerhaft, bevor der Schlüssel verschwindet): richtig; macht die Zeit bis „gelöscht“ unabhängig von der Datenmenge.
- **DEK-Ableitung über Vorfahren-Salts** (T, C, S, D): Eine Vernichtung wirkt „nach oben ableitbar“ (S weg ⇒ alle DEKs des Betroffenen weg). Kein gewrappter DEK je Record nötig. Das ist der eleganteste Teil des Entwurfs.
- **Trennung N/P/D** mit Brücke nur über zufällige Handles und Commitments; **Erase gewinnt über `as_of`**.
- **Typestate-Messung** (`Scanner<Unarmed>` → Positivkontrolle → `Armed` → `LayerWitness` nur bei `matches == 0`) und **ein** Löschpfad (`ErasureCore::execute`, D-06).
- **Sync-Kern, Adapter für async** (D-09), **Brückencrate** `acta-erasure-adapters` (D-08).

### 4.2 Fehler und Lücken (Findings F-1 bis F-8)

#### F-1 (H): E1 bricht die O(1)-Zusage

SPEC §7.3 E1 erzeugt die **Nadelmenge** (drei Klartextfenster je Dokument, Terme, Entitäten) *vor* dem Gate, „solange der Klartext noch lesbar ist“. Für einen Betroffenen mit 10⁵ Dokumenten bedeutet das 10⁵ Entschlüsselungen vor E2. `erase_return_timeout = 500 ms` liefert dann `Pending` **ohne Gate**, und S1 („ab Rückkehr kein Lesepfad mehr“) gilt nicht.

**[R]** Nadeln in drei Klassen aufteilen:

| Klasse | Quelle | Kosten | Zeitpunkt |
| --- | --- | --- | --- |
| Chiffretext-Reste (`doc_tag`, Umschlag-Header, Nonces) | Lineage/Index, kein Entschlüsseln | O(1) je Dokument, RAM | E1 |
| Index-Terme und Entitäten | Posting-Listen/Graph via Ordinal-Bitmap (F-3) | O(Treffer) | E1, nach E2 zulässig (liest nur Index) |
| Klartextfenster | Entschlüsseln | **gedeckelt**: stratifizierte Stichprobe, höchstens `NEEDLE_MAX = 4096` Fenster | E2a, vor E3, **nach** dem Gate (Leser sind bereits gesperrt, der Kern selbst darf lesen) |

Reihenfolge neu: **E2 Gate → E2a Nadel-Stichprobe → E2b `on_gate`-Ack aller Schichten → E3 Vernichtung.** Der Bericht weist den Stichprobenumfang aus (Messung bleibt Stichprobe, nie „vollständig“).

#### F-2 (H): Widerspruch D-02 gegen Synthese §5.5

Die Synthese verlangt `PlaintextPurged → PlaintextMeasured → KeyDestroyed`, weil sonst „Blöcke mit Daten mehrerer Dokumente nicht mehr geordnet rekonstruierbar“ seien. D-02 vernichtet den Schlüssel sofort nach dem Gate.

**Auflösung [R]:** Das Argument der Synthese gilt nur bei **Block-Verschlüsselung** (mehrere Dokumente unter einem Schlüssel). Acta verschlüsselt **je Record unter Dokument-DEK** (Umschlag v3). Damit ist ein Rewrite ein Filter über *Chiffretext*; Schlüsselvernichtung davor ist unschädlich. Zwei Konsequenzen als **harte Invarianten** in die SPEC:

- **I-17 (neu):** Kein persistierter Container (SST-Block, WAL-Frame, Segment) enthält Chiffretext zweier Verschlüsselungseinheiten unter *einem* Schlüssel. Test: Property auf dem Siegelpfad plus Typzwang (`SealedBytes` trägt `doc_tag`).
- **E3-Vorbedingung:** Ebene A (RAM-Klartext der Zielmenge) ist genullt, **bestätigt durch `on_gate`-Ack jeder registrierten Schicht**. Ohne Ack kein E3 (`Pending(LayerNotAcked)`). Sonst entsteht ein Fenster „Schlüssel weg, Klartext-Vektor im RAM lesbar“, das die Synthese zu Recht fürchtet.

#### F-3 (H): Lineage als dichte Ordinale und Roaring-Bitmaps statt Tag-Listen

SPEC §9.3 führt je SST `tag_set` (bis 4.096 16-Byte-Tags oder XOR-Filter) und je Segment `source_tags`. Das hat drei Schwächen: Listen sind pro Schicht unterschiedlich, XOR-Filter haben Falsch-Positive, Listen skalieren nicht.

**[R] Einheitliches Lineage-Modell:**

- Jedes Dokument bekommt beim Ingest ein **dichtes `doc_ord: u32`** je Mandant (monoton, **nie wiederverwendet** bis zum Segment-Rebuild; analog zu `SeqHwm`, I-6).
- Jede abgeleitete Struktur (SST-Footer, Posting-Segment, HNSW-Knoten, Graph-Kante, Cache-Eintrag) führt `source_set: RoaringBitmap` bzw. je Element `doc_ord`.
- `ErasedOrds: RoaringBitmap` ersetzt Bereichs-Gate: `ReadGate::check` = `contains(ord)` in O(1), mergeable, und vektorisierbar für Kandidatenmasken (HNSW-Bitset, Posting-Filter, Graph-Traversal).
- Purge-Entscheidung je Datei/Segment = `source_set ∩ target != ∅`: exakt, ohne Falsch-Positive, ein Befehl.
- `roaring = "0.10"` ist bereits Workspace-Abhängigkeit [V `Cargo.toml`]. `doc_ord` ist rein intern und **nie** Teil von N oder Berichten; die Zuordnung `ord ↔ doc_tag` liegt in P unter dem Dokument-Slot und verschwindet mit ihm. Ordinale sind kein Schlüsselbestandteil, damit bleibt KeyV2 stabil.

Zusage S10 wird dadurch zum **Typzwang**: `put_derived(...)` verlangt ein `Lineage`-Token (nicht leer), sonst Kompilierfehler (siehe 5, P3).

#### F-4 (H): `cluster` in KeyV2 undefiniert – Hebel für Rewrite-Kosten

§9.1 listet `cluster 16` im 65-Byte-Präfix, definiert es aber nirgends. Der Wert entscheidet, ob Löschung **Datei-Drop statt Rewrite** wird.

**[R]** `cluster = subject_tag` des Hauptbetroffenen (Fallback: `doc_tag`). Folgen: Alle Records eines Betroffenen liegen im Schlüsselraum zusammenhängend je (kind, tenant, collection). Compaction/Level-Platzierung hält sie nah beieinander; ein gefilterter Rewrite berührt wenige SSTs, und liegt ein Betroffener allein in einer Datei, entfällt der Rewrite zugunsten von Drop (Manifest-Batch `Remove`, `scrub`, `unlink`). Das Präfix-Layout bleibt 65 Byte; nur die Belegung wird festgelegt. Mehrbetroffene Dokumente bekommen den Hauptbetroffenen, die übrigen Salts stehen im Umschlag-Header.

**Zusätzlich [R]:** Die Reihenfolge `[kind][tenant][collection][cluster][doc]` zwingt einen Betroffenen mit Daten in `k` Collections und `m` Kinds zu `k·m` Bereichen. Das Gate läuft deshalb über `ErasedOrds` (F-3), **nicht** über Schlüsselbereiche; die Schlüsselordnung dient nur der Lokalität. Das löst das Lookup-Problem, ohne die Sortierung zu opfern.

#### F-5 (M): `ArtKey` je Segment – Verfügbarkeit

SPEC §6.12/§6.13: Wird *eine* Quelle vernichtet, wird der `ArtKey` des Segments (bis 50.000 Vektoren) vernichtet, das Segment sofort unlesbar, danach Rebuild. Folge: Ein Subject mit Dokumenten in 20 Segmenten legt 20 × 50.000 fremde Vektoren bis zum Rebuild still; Suchqualität bricht ein, und der Rebuild entschlüsselt fremde Embeddings neu.

**[R] Zweiphasig, Verfügbarkeit gewahrt:**

1. E2: Gate und RAM-Nullung wie bisher, Bitmaskierung (`ErasedOrds`) macht das Segment für Anfragen *sofort korrekt* (Treffer der Zielmenge ausgefiltert).
2. Rebuild-Auftrag je betroffenem Segment (Batch `batch_window`), neues Segment ohne Zielmenge unter neuem `ArtKey_{g+1}`.
3. **Nach Aktivierung des neuen Segments** `ArtKey_g` vernichten. Berichtsfeld `derived_exposure_window` (Zeit zwischen E3 und `ArtKey_g`-Vernichtung); Klasse D gilt erst danach als erreicht.

Die rechtliche Spannung (Embedding der Zielmenge bleibt unter `ArtKey_g` entschlüsselbar, solange das Fenster offen ist) wird **deklariert und gemessen**, nicht verschwiegen. Alternative bei hohem Schutzbedarf: Konfiguration `derived_mode = DestroyFirst` (SPEC-Verhalten). Variante ohne Persistenz: D wird nicht persistiert, nur beim Start aus P neu gebaut (Kaltstartkosten, dafür keine Spur).

#### F-6 (H): Gate-Lebensdauer und Lösch-Paradoxon sind unvollständig

Synthese §5.7 und §6.4: Ein Nachweis mit verknüpfbarer Lösch-ID ist pseudonym, solange der Verknüpfungsschlüssel lebt (AG Lörrach, ⚠ umstritten). SPEC deckt `K_commit[epoch]` ab; offen bleibt:

- **`K_scope`** (erzeugt `doc_tag`, `tenant_tag`, … per BLAKE3-keyed, SPEC §9.1) hat keine Epoche und keine Vernichtung. Solange `K_scope` lebt, sind alle in GateLog/Ledger gespeicherten Tags für den Betreiber auf Kennungen rückführbar. [R] `K_scope` epochenweise führen, Vernichtung nach `gate_compaction_delay` (Default 90 Tage), danach sind GateLog-Tags nicht mehr zu Personen auflösbar. Das passt zu §7.9 (Gate-Kompaktion).
- **`revocation_log.rs`** speichert „widerrufene Schlüsselgruppen“ append-only, Ed25519-signiert. Synthese 3.5 Nr. 2: Ein append-only Schlüsselprotokoll darf **keine Schlüsselkopie** sein. [R] Test `revocation_log_has_no_key_bytes` (Bytes-Grep über das Log mit bekanntem DEK) in die Abnahme; sonst Format ändern. Inhalt unverifiziert [A].
- **Zeitstempel/Größen/Anzahl** im Bericht erlauben Re-Identifikation bei kleinen Mandanten. [R] Bucketing wie im N-Validator (k-Schwelle) auch für Berichtsfelder.

#### F-7 (M): Restore-Horizont fehlt als Verweigerung

Synthese §5.8: Backups älter als `erased_set_retention` werden verweigert (`RestoreBeyondHorizon`). SPEC ersetzt das durch 72-Stunden-Schlüsselbackups und Live-Replik (§7.9), kennt aber kein **Ablehnen** eines alten Restore-Kandidaten am Eingang. [R] `RestoreGuard` prüft `backup.created_at + T_ROT_BACKUP`; ältere Tresor-Sicherungen werden nicht angenommen (Fehler `RestoreBeyondHorizon`). Wirkt nur, wenn V-04 (Epochen-Maskierung) geschlossen ist.

#### F-8 (M): Zeitreise-Schnittstelle gegen Retention

`RetentionPolicy::may_drop_version(key, seq, newer_exists)` (SPEC §8.2) bekommt keinen Zugriff auf `ErasedOrds`. [R] Signatur `may_drop_version(&self, v: VersionRef<'_>, gate: &dyn ErasedView)`, wobei `ErasedView` die einzige Löschfähigkeit ist, die der Store sieht (read-only, O(1)). Der Store bleibt frei von Löschwissen (D-13), Compaction wirft Chiffretext gelöschter Ziele spätestens beim nächsten Merge aus P, auch ohne Purge-Lauf.

---

## 5. Konstruktionsprinzipien („Löschbar von Geburt an“)

Jedes Prinzip ist als Rust-Typ oder Lint durchsetzbar, nicht als Konvention.

| ID | Prinzip | Umsetzung in Rust | Verhindert |
| --- | --- | --- | --- |
| **P1** | **Keine Personendaten ohne Klasse.** Klasse ist Pflichtparameter, keine Vorgabe. | `enum DataClass`, `Ingest::put(class: DataClass, …)`; Fehler `ClassViolation` bei `Unclassified` ohne Quarantäne | stille Defaults, PII in N |
| **P2** | **Chiffretext vor erstem Byte auf Platte.** Für P ist `put` nur mit `Sealed<P>` möglich. | Newtype `Sealed<P>(Envelope)`, Konstruktor nur in `acta-crypto`; Store-API `put_p(Sealed<P>)` | Klartext im WAL, B-03 |
| **P3** | **Keine Ableitung ohne Herkunft.** | `Derived<T>` trägt `Lineage { source: RoaringBitmap }` (nicht leer); `put_derived(Derived<T>)` | Index ohne Löschpfad, S10 |
| **P4** | **Lesen nur durch das Gate.** | `ReadGate::admit(key) -> Result<Admitted<'g>, Erased>`; `decrypt` und Index-Ausgabe nehmen `Admitted<'g>`; `LsmStorage` bleibt `pub(crate)` | Leseweg am Gate vorbei |
| **P5** | **Fehler im Schlüsselpfad sind nie Werte.** | `#![deny(clippy::let_underscore_must_use, clippy::unwrap_or_default)]` im Kryptopfad, `SecretKey` ohne `Default`, keine `[0u8; N]`-Konstanten als Schlüssel (`clippy.toml disallowed_methods`) | V-01, V-05 |
| **P6** | **Gesperrte Operationen per Lint, nicht per grep.** | `clippy.toml`: `disallowed_methods = ["std::fs::remove_file", "std::time::SystemTime::now", "rand::thread_rng", "std::fs::hard_link", "std::fs::copy"]` mit Ausnahmeliste für `acta-durable-fs`/`acta-sys` | G-FS-1, G3, B-11 |
| **P7** | **Eine Schicht, ein Eintrag, ein Test.** | `const LAYERS: &[LayerDesc]` und exhaustive `KeyKind`-Matches; Test `every_layer_has_purger_and_witness` | I-9, vergessene Schicht |
| **P8** | **Physische Lokalität = Löschgranularität.** | `cluster = subject_tag` (F-4), SST-Footer mit `source_set` | Rewrite-Kosten |
| **P9** | **Einheit der Verschlüsselung = Einheit der Löschung.** | I-17 (F-2): ein Umschlag je Record, nie je Block | Blockschlüssel, Kollateral |
| **P10** | **Der Nachweis ist ein Typ.** | `LayerWitness` nur bei `matches == 0`; `ErasureReport::build` nimmt `NonEmpty<LayerWitness>` | S8 |

**Ersatz von G-CRY-1 (grep-basiert):** Grep-Gates sind umgehbar (Makros, Aliase). `disallowed_methods`/`disallowed_types` plus `cargo deny` (gesperrte Crates, Lizenz, Versionen) sind strikter und laufen in der CI mit `-D warnings`.

---

## 6. Inkonsistenzen in `docs/SPEC.md` (Gate G-DOC)

| Nr. | Stelle | Befund |
| --- | --- | --- |
| 1 | Kopf §0.2, §1.2 (S5), §11.6 (G5) | Verweisen auf **§13** (Recht), **§14** (Betreiberrisiko), **Anhang A/B** (Testvektoren). Die Datei endet bei §12; die Kapitel fehlen. |
| 2 | §1.2 S4 | Verweist auf „Durabilitätspunkte (§7.6)“; die Matrix steht in §7.4. |
| 3 | §6.19 vs. §4.3 | §4.3 verweist auf „Budget §6.20“; das Budget steht in §6.19. |
| 4 | §7.5 | Überschrift „Reihenfolge a bis g“, Tabelle hat Phasen a bis **h**. Der Journal-Code und `PurgePriority` müssen übereinstimmen. |
| 5 | §9.5 | „`tag` bleibt (Sondierung)“ widerspricht dem Code (V-11), der ein Pseudonym entfernt. |
| 6 | §5.3 | Slot-Typ U (K_unit) ist [A]; V-04 zeigt zusätzlich, dass der Slot-Payload T/C/S/D unmaskiert ist. §5.3 behauptet „maskiert/verschlüsselt unter K_enc“. |
| 7 | §12 Phase 0 | Tabellenzelle bricht an `\|\| true` ab (Markdown-Pipe); P0-1 bis P0-7 sind im Rendering unleserlich. |
| 8 | `docs/` | Dateiduplikat der Synthese; zwei überlappende DSGVO-Analysen („…in RAG-Systemen.md“ ist Teilmenge von „…Löschen personenbezogener Daten.md“, Datei 2 enthält zusätzlich eine zweite Fassung ab Zeile 225). [R] Eine kanonische Fassung, Rest nach `docs/archiv/`. |

---

## 7. Priorisierter Umbau (Refaktorisierung)

Leitidee: **Walking Skeleton vor Breite.** Der Löschkern (≈ 22.900 Zeilen) und der Store (≈ 32.800) sind getrennt gereift. Das größte Risiko ist die nie getestete Verbindung. Deshalb ein dünner vertikaler Schnitt **vor** der Detailarbeit der Phasen 2 bis 5.

| Stufe | Inhalt | Befunde | Abnahme (bestandene Messung, nicht Behauptung) |
| --- | --- | --- | --- |
| **R0 Stop-the-line** (Tage) | `acta-erasure` in `members` und `default-members`; XOR-`VaultSealer` und Fallback in `resolve_dek` **entfernen** (fail-closed: `Erased` oder Fehler); `[0x55;32]`, `[0x42;32]`, `k_idx`-Default entfernen; `unwrap_or_default`/`let _ =` im Kryptopfad beseitigen; `clippy.toml` (P6) | V-01, V-02, V-03, V-05 | Test: gelöschter D-Slot ⇒ `open` liefert `Erased`, nie Bytes; Test: kein Konstanten-Schlüssel (`no_const_keys`) |
| **R1 Krypto-Kern** | Umschlag v3 (XChaCha20-Poly1305, 192-Bit-Zufallsnonce aus `Rng`-Port, AAD = Kontext); **eine** Hierarchie (D-05), `crypto_local` löschen, `derive_dek` mit `n` Betroffenen und echtem Fehlerpfad; Tresor-Payload T/C/S/D unter Epochenschlüssel maskieren, **Epochenrotation implementieren** (`mask_epoch_key`, `E_e`-Vernichtung); `destroy_many` | V-02, V-04, V-05, V-12, B-06 | Testvektoren; Rotation mit Crash C11; Bytes-Grep: alte Tresor-Kopie + `K_unit` leitet keine vernichtete DEK mehr ab |
| **R2 Walking Skeleton** | Minimaler Pfad: Ingest (Klasse deklariert, `Sealed<P>`) → ein WAL/Memtable → **ein** Index (`acta-text`) mit `Lineage` → `ErasureCore::execute` synchron → `on_gate`-Ack → E3 → Messung (MK2 mit Positivkontrolle) → Bericht → Kanarien-Test im Boot-Selbsttest | B-01, B-14, F-1, F-2 | `T-DEL-1` (Lesepfade nach Gate), Kanarien Ende-zu-Ende, Crash-Matrix C1–C6 mit `FaultVfs` |
| **R3 Datenmodell** | `doc_ord` + Roaring-Lineage (F-3); KeyV2 mit `cluster = subject_tag` (F-4); Partitionen N/P/D im Store (`DEFAULT_PARTITION` ablösen); Wert-Siegel vor dem WAL; ASCII-Codec per `ReSeal` migrieren; `ReadGate` in `may_drop_version` (F-8) | V-06, V-07, B-04, B-12 | I-5, I-6, I-7, `no_ascii_keys`, `no_v2_envelope_in_P` |
| **R4 Store-Fähigkeiten** | `FlushPurgeFence` asymmetrisch und `enter_purge` verdrahtet; `rewrite_filtered` mit Datei-Drop-Pfad; `compaction_barrier`; WAL-Frame v2; Scrub einbinden | V-08, B-09, B-19 | Loom-Modell Flush/Rollover/Purge; Crash C7/C8 |
| **R5 Schichten** | `VectorLayer`, `TextLayer`, `GraphLayer`, `CacheLayer`, `CheckpointLayer` + `CopyRegistry` (nur N/D-Metadaten hardlinken); Rebuild-dann-Vernichten (F-5) | V-09, V-10 | T-VEC, T-TEXT, T-GRAPH, T-REST, `hnsw_no_residue_after_erase` |
| **R6 Chronik, Engine, Node** | wie SPEC Phase 3/4; `Wiring` mit Pflichtfeldern | B-14 | G1, G2, G-ERA-1, G-READ-1 |
| **R7 Nachweis, Härtung** | `acta-verify` ohne Acta-Code; Notary; HSM-Signer (A1); Messwerte statt Zielwerte; Krypto-Audit (G4); Rechtsfreigabe (G5) | D-12 | unabhängige Prüfung besteht |

**Abhängigkeitslogik:** R0 sofort, weil V-01 jeden späteren Test verfälscht (ein „bestandener“ Löschtest über den Fallback wäre ein Falsch-Positiv). R1 vor R3, weil Partitionen und KeyV2 das neue Siegel brauchen. R2 *vor* R3 bis R5, damit jede Breitenarbeit gegen einen laufenden Kanarien-Test entsteht.

---

## 8. Leistungs- und Kostenmodell (Zielwerte, ungemessen)

| Operation | Soll | Treiber |
| --- | --- | --- |
| Zeit bis `Gated` | O(1): 1 Gate-Append + `gate_epoch`-Increment; Rückkehr < 500 ms | Gruppen-fsync, keine Entschlüsselung (F-1) |
| Schlüsselvernichtung Subject | 1 Slot-Schreibvorgang (64 B) + 1 `sync_data` + 1 Rücklesen | `destroy`; Batch via `destroy_many` (V-12) |
| Gate-Prüfung beim Lesen | O(1) (`RoaringBitmap::contains`), kein I/O | F-3 |
| Segment-Zuordnung beim Purge | O(Segmente) Bitmap-Schnitt, exakt | F-3 |
| Physischer Purge SST | O(betroffene Dateien · Dateigröße); mit `cluster = subject` typisch wenige | F-4 |
| Ableitungs-Rebuild | O(Segment) je betroffenem Segment, **ausgelagert**, ohne Verfügbarkeitsverlust | F-5 |
| Boot-Kanarientest | < 2 s | SPEC-Ziel |

Die Kernaussage: Nur Zeit bis `Gated` und Schlüsselvernichtung liegen im Anfragepfad. Alles andere ist batchbar (`batch_window`) und darf Datenmenge-abhängig sein, weil es nach D-02 nie Voraussetzung für „gelöscht“ ist.

---

## 9. Rechtliche und kommunikative Grenzen (aus `docs/dsgvo/`)

- Keine Behörde und kein Gericht hat Crypto-Shredding als Löschung nach Art. 17 anerkannt oder abgelehnt (Synthese §3.6, §11 Nr. 5). Acta liefert die *technischen* Bedingungen aus Synthese §3.5 (Granularität, alle Kopien, kein Klartext-Parallelbestand, Metadaten, Stand der Technik, KDF-Stärke, echte Vernichtung, Nachweis). Die Qualifikation bleibt Sache von DSB/Fachanwalt (Prüfpunkte §13 der Synthese).
- Embeddings sind **vorsorglich personenbezogen** (Synthese §4.1, SPEC §6.12). Konsequenz F-5.
- `§ 35 BDSG` trägt für automatisierte Vektor-DBs und Backups nicht (Synthese §4.4). Keine Designannahme darf darauf aufbauen.
- **Produktaussage:** „DSGVO-konform“ ist unzulässig (SPEC §1.3, Synthese §12, UWG-Risiko). Belastbar: Klassen G/K/P/D/C je Löschung mit `reached`/`not_reached`, deklarierte Annahmen `A-FLASH`, `A-RAM`, `A-ANCHOR`, `A-CRYPTO`, Zonen Z4/Z5 nur als „benachrichtigt“.

---

## 10. Offene Punkte und Annahmen

| ID | Punkt | Wirkung |
| --- | --- | --- |
| A-1 | `revocation_log` enthält keine Schlüsselbytes (nicht gelesen) | F-6 |
| A-2 | Schreibweg der Slot-Payloads T/C/S/D unmaskiert (nur Leseweg `derive_dek_from_vault` gelesen) | V-04 |
| A-3 | `HistoryFloor` im Manifest (Treffer in `manifest/*`, nicht im Detail geprüft) | V-07 |
| A-4 | `art_key` in zwei `acta-store`-Dateien nicht geprüft (vermutlich Kompaktionsbereich, nicht Schlüssel) | V-09 |
| A-5 | Aufwände nicht gemessen; kein Build, keine Tests ausgeführt | alle Stufen |
| A-6 | Interface-Spezifikation (910 KB, generiert) nur strukturell genutzt | Vollständigkeit der Pub-API-Prüfung |
| A-7 | Rechtsbezüge ausschließlich aus `docs/`; Primärquellen nicht verifiziert (Synthese §14) | Abschnitt 9 |
