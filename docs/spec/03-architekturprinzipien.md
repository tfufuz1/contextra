---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "03"
---
## 3. Architekturprinzipien P1–P30

Diese Prinzipien sind normativ für jede gegenwärtige und künftige Erweiterung des Systems. **P1–P25 sind aus
der Vorfassung unverändert übernommen** und gelten fort. **P26–P30 sind ab dieser Fassung neu** (Quelle: Teil
A2, `CONTEXTRA_ZIELARCHITEKTUR_v2.md` §2). Zusätzlich ändert Teil A2 die **Anwendung** von P5 und P8–P22 auf
Crate-Ebene (siehe die Δ-Vermerke unten), ohne ihren Wortlaut zu ändern.

**P1 — Korrektheit schlägt Performance schlägt Feature.** Siehe §1.

**P2 — WAL-First-Persistenz.** Keine Zustandsänderung wird im Speicher sichtbar gemacht, bevor sie physisch
in das Write-Ahead-Log geschrieben und mit dem Datenträger synchronisiert wurde.

**P3 — Deterministische Recovery.** Der Systemzustand muss sich allein aus dem WAL rekonstruieren lassen.

**P4 — Keine stillschweigende I/O-Fehlerunterdrückung.** `fsync`-Fehler IMMER mit `?` propagieren.

**P5 — Strikte DAG-Modularität.** Abhängigkeiten im Crate-Graphen verlaufen strikt abwärts; ein Verstoß
gilt als Architekturdefekt. **Δ (verbindliche Korrektur, §A2.1):** In der Vorfassung war dieses Prinzip
dokumentiert, aber am realen Crate `contextra-db` (Layer 2) nachweislich verletzt — es hing hart von
`contextra-candle`/`contextra-ollama` (Layer 3) und optional von `contextra-embed` (Layer 3) ab, weil
`EmbeddingBackend` samt Konstruktion in `contextra-db` lag. Ab dieser Fassung wird P5 **maschinell** erzwungen
(`tests/layering.rs` gegen `cargo_metadata`, plus `deny.toml`-Bans mit `wrappers`-Liste, §4.2 unten), nicht
mehr nur dokumentiert. Reihenfolge und Richtung sind jetzt die Ring-Matrix in §4.2 (Ring-Modell), nicht mehr
die alte Layer-Tabelle.

**P6 — Feingranulare Fehlerbehandlung.** Domänenspezifische `Result<T, E>`-Enums statt generischer `panic!`-Pfade.

**P7 — Verbot von `unwrap()`/`expect()` auf toxischen Daten.** CI-gated über `.unwrap-baseline.json`.

**P8–P22 — Sovereign-Core-Grundsätze.** Umfassen u. a.: Verschlüsselung at rest als Default, HMAC-Kettenintegrität,
Zero-Trust-Sandbox, Air-Gap-fähige Inferenz, deterministische Transaktions-ID-Vergabe, Tombstone-Bit-Disziplin,
SSTable-Flush-Sichtbarkeit, Kaskaden-Invalidierung, kryptographische Löschnachweise, feingranulare Feature-Gates.

**P23 — Zeitbudgets sind orthogonal konfigurierbar.** Rechenschritt-Budget (Fuel) und Wall-Clock-Budget für
Sandbox-Ausführungen sind zwei unabhängige Achsen. Ein Tool kann rechnerisch günstig, aber durch blockierendes I/O
langsam sein, oder umgekehrt — beide Fälle müssen unabhängig begrenzbar sein. 🟢 Produktiv erfüllt (§10.2).

**P24 — Lokalität vor globaler Neuberechnung.** Jeder Algorithmus, dessen Eingabe eine anfragebestimmte Teilmenge
des Gesamtzustands ist (PPR mit wenigen Seed-Knoten, Cascade-Invalidierung ausgehend von einem Dokument), MUSS
eine zur Anfragegröße proportionale Laufzeit haben — niemals zur Größe des Gesamtzustands ($O(V+E)$ ist für
solche Anfragen unzulässig). Dieses Prinzip ist der normative Grund für den Forward-Push-PPR-Algorithmus (§7.2)
und für das harte Fan-out-Limit der Hyperkanten-Cascade-Invalidierung (§6.5, H5).

**P25 — Cache-Treffer sind lock-frei bzw. lock-günstig zu gestalten.** Ein Lesetreffer im Block-Cache soll nach
Möglichkeit keinen exklusiv sperrenden, mutierenden Zugriff erfordern, da Cache-Treffer der mit Abstand häufigste
Zugriffspfad sind und jede darin verborgene Schreibsperre unter Last zur Kontention wird (§5.4).

### Neu ab dieser Fassung: P26–P30 (Teil A2)

**P26 — Sync-Kern, async-Schale.** Ring 0 (`contextra-types` … `contextra-adapt`) enthält kein `tokio`. Kerne sind
Zustandsautomaten ohne I/O; die Engine (Ring 3) treibt Persistenz und bindet Kerne per begrenztem `ComputePool`
an. **Vorher/Jetzt:** Die Vorfassung empfahl Sync-Kerne als wünschenswert, ohne Ursache zu benennen; verifiziert
ist, dass die tatsächliche Ursache heutiger `async fn` in den Kernen zweierlei ist — native AFIT-Ports (nicht
`dyn`-kompatibel) und I/O-Aufrufe direkt in den Kernmodulen (`graph/csr.rs`, `text/inverted.rs`,
`index/diskann.rs`), nicht Rechenlogik. Die Migration trennt daher zuerst die Ports (P27), dann das I/O (§4.1
der Quelle), bevor Kerne als synchron gelten dürfen. Erzwingung: `cargo tree -e normal -p contextra-{vector,text,graph,rank,adapt}` ohne `tokio`.

**P27 — Ports sind `dyn`-kompatibel per Konstruktion.** Traits in `contextra-ports` sind synchron, wo das
Blockierverhalten ohnehin durch `mmap`/`pread` gegeben ist (`StorageRead`, `VectorIndex`, `TextIndex`,
`GraphIndex`); wo echtes asynchrones Warten nötig ist (`StorageWrite::commit`, `Embedder::embed`), liefern sie
`BoxFuture` statt natives `async fn`, weil natives AFIT nicht objektsicher ist. Grund: die Engine muss Backends
zur Laufzeit als `Arc<dyn Trait>` austauschen können (Composition Root, Ring 4 `contextra`, §4.2).

**P28 — Injizierter Nichtdeterminismus.** `Clock`, `Rng`, `IdGen` sind Ports, niemals direkte Aufrufe von
`SystemTime::now()`/`rand::thread_rng()` in Kern- oder Persistenzcode. `contextra-testkit` liefert
`ManualClock` und eine In-Memory-`StorageEngine` für deterministische Simulationstests (WAL/LSM-Crash-Injektion,
§20 Phase 3a) und `loom`-Lock-Protokoll-Tests. **Δ gegenüber Vorfassung:** `contextra-testkit` entsteht bereits
in Migrationsphase 0R, nicht erst nachträglich — Determinismus-Infrastruktur geht der Sync-Migration voraus,
nicht hinterher.

**P29 — Kein globaler veränderlicher Zustand.** `static OnceLock`/`Lazy` sind ausschließlich für unveränderliche
Konstanten zulässig (Regex, Stopwortlisten). Veränderlicher Zustand gehört immer einer Instanz. **Grund:**
verifiziert wurde ein globaler `static ORPHAN_REGISTRY: OnceLock` in `checkpoint/orphan.rs` sowie ein als
`unimplemented!()` spezifizierter globaler `static CIPHER_INSTANCE` (§9.3) — beide widersprechen der
Schlüssel- bzw. Registry-Hierarchie und werden im Zuge der Migration entfernt (§20 Phase 1c bzw. §9).

**P30 — Jeder Crate-Zuschnitt braucht ein Kriterium.** Ein Crate existiert nur, wenn er mindestens eines
erfüllt: **I** Isolation einer flüchtigen Abhängigkeit (candle, ort, wasmtime, pyo3, reqwest) · **U**
Unsafe-Insel · **C** eigener Änderungsrhythmus/Bounded Context · **S** Größe > 8.000 LOC (Compile-Parallelität)
· **D** Richtungserzwingung (z. B. Composition Root). Crates ohne erfülltes Kriterium werden zusammengelegt.
Die vollständige Kriterienzuordnung für alle 27+3 Crates steht in §4.2.

### Ergänzende Grundsätze

- **Nebenläufigkeitssicherheit vor Nebenläufigkeitsperformance:** Sperrenhierarchien werden explizit dokumentiert
  und dürfen nicht durch bloßen Analogieschluss auf neue Mutationspfade übertragen werden, ohne die
  Deadlockfreiheit für den neuen Fall erneut zu beweisen (konkretes Beispiel: §6.5, H2).

- **Geschlossene Enums bleiben geschlossen:** Wo ein Enum bewusst **nicht** `#[non_exhaustive]` deklariert ist
  (z. B. `SignalKind`), ist das eine architektonische Entscheidung. Eine neue Kategorie von Information wird
  in ein bestehendes offenes Signal integriert, statt das Enum breaking zu erweitern (§6.5, H3).

- **Kein Sicherungsnetz, keine Schema-Änderung:** Persistenzformat-Änderungen werden nur vorgenommen, wenn ein
  automatisiertes CI-Drift-Gate zwischen Schema und generiertem Code aktiv läuft (§6.5, H4).

- **Explizite Unvollständigkeit statt stiller Lücken:** Wo ein Subsystem eine neue Datenklasse strukturell nicht
  berücksichtigt, wird dies über ein sichtbares Konfigurations-/Report-Flag markiert (§6.5, H6).

---

<a id="4-architektur"></a>
