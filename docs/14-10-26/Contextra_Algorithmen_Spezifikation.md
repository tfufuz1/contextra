# Contextra: Algorithmen-Spezifikation und Umbauplan

**Stand:** HEAD `500e9246` (Merge PR #4785, 14.10.2026 13:04). Die Berichte beziehen sich auf `16764583` und ältere Stände. Dieser Stand wurde nicht vollständig neu geprüft.

**Status dieses Dokuments:** Teilweise verifiziert. Was nicht ausgeführt werden konnte, steht in Abschnitt 7.

---

## 0. Kurzurteil

1. **Der Build ist rot.** `cargo +stable check -p contextra-engine` bricht mit `E0004` ab: `PurgeBlocked::UnknownFreeSpace` fehlt im Match in `crates/contextra-engine/src/deletion/purge_driver.rs:172`. Der Match hat auch keinen Wildcard-Arm. Das ist der Befund N-N aus der Gesamtanalyse, am neuen HEAD unverändert. Blocker Nummer eins.
2. **Die Toolchain ist nicht reproduzierbar.** `rust-toolchain.toml` pinnt 1.89.0. Der Download scheitert, weil der Agent-Proxy `static.rust-lang.org` mit 403 abweist. Das ist eine Policy-Entscheidung und wurde nicht umgangen. Geprüft wurde nur mit der installierten `stable`-Toolchain, als Diagnose.
3. **Die Befunde der Gesamtanalyse sind am neuen Code größtenteils noch da.** Verifiziert durch Lesen des Quelltexts: `find_k_paths` hat außerhalb von `budgeted_k_path_hub_entity.rs` keinen Aufrufer. `downcast_ref::<LsmStorage>` steht an den fünf dokumentierten Stellen. `issue_sealed` ist nur definiert, es gibt keinen Aufrufer im Produktionscode. `ErasureMode` kommt im Repository nicht vor.
4. **Die Forschungsberichte widersprechen sich.** Siehe Abschnitt 2. Eine Algorithmenauswahl darf nicht auf einem einzelnen Bericht beruhen.
5. **Empfehlung:** Zuerst den Build reparieren (Phase 0a), dann den Löschpfad, dann Algorithmen. Der Crate-Split gehört ans Ende.

---

## 1. Verifizierter Ist-Stand (HEAD `500e9246`)

| Befund | Prüfmethode | Ergebnis | Evidenz |
|---|---|---|---|
| N-N: Match-Lücke `UnknownFreeSpace` | `cargo +stable check -p contextra-engine` | **bestätigt** | E0004, `purge_driver.rs:172` |
| N-O: Fassaden-Fehler unter Feature `router` | nicht geprüft (Build nach N-N nicht möglich) | offen | Gesamtanalyse, `builder.rs:675-676` |
| N-P: Lockfile | nicht geprüft (`--locked` scheitert am Toolchain-Download, nicht am Lockfile) | offen | Gesamtanalyse |
| N-B: `Collection::delete()` scheitert bei Tenant-Wrapper | nur Lesen | nicht neu verifiziert | `collection/mod.rs:405,528,532`, `crud/delete.rs:79` |
| K4: fünf Downcasts auf `LsmStorage` | Grep | **bestätigt** | `collection/mod.rs`, `crud/delete.rs`, `db_transaction_commit.rs:35` |
| E6: `issue_sealed` ohne Aufrufer | Grep | **bestätigt** | `deletion_witness.rs:162` |
| `find_k_paths` ohne Produktionsaufrufer | Grep | **bestätigt** | nur Tests in `budgeted_k_path_hub_entity.rs` |
| `ErasureMode` existiert nicht | Grep | **bestätigt** | keine Treffer |

**Nicht verifiziert:** Die Testergebnisse aus Fassung 1 (`zz_audit_h3_l6`, `l6`, `l8`, `na_resume`) konnten nicht wiederholt werden, weil `contextra-store` nicht getestet wurde. Ich habe nur die Engine kompiliert. Die Aussagen dazu bleiben im Status „übernommen, nicht reproduziert“.

---

## 2. Quellenlage: Widersprüche zwischen den Berichten

Diese Tabelle ist wichtig, bevor irgendein Algorithmus umgesetzt wird.

| Thema | Deep Research v2 (überarbeitet) | Langfassung (retrieval_mechanismen_v2) | Bewertung |
|---|---|---|---|
| Anzahl belegter Mechanismen | 12 Mechanismen, nur B.1, D.1, F.1 mit Quelle belegt | 3 vollständig spezifiziert: ACI, SSN, BMSSP | Die Berichte sind inkompatibel. Die Langfassung hat Volltext gelesen (Q1–Q3), die v2 nur Abstracts. |
| Fusion (DiBud, B.1) | Empfohlen, Quelle 2609.15143 [A] | nicht bearbeitet (Feld B „nicht recherchiert“) | Nicht verifiziert. Keine Volltextlektüre in irgendeinem Bericht. |
| ESSN (F.1, 2511.22956) | Empfohlen, Prüfung nötig | Verworfen: nur Abstract | Nur Abstract. Nicht implementieren, bevor der Volltext gelesen ist. |
| SSN (2016) | nicht enthalten | Mechanismus 2, Volltext gelesen | Die Langfassung hat den einzigen Volltext. Datum laut Sekundärquelle, nicht belegt. |
| ACI (Gibbs/Candès) | nicht enthalten | Mechanismus 1, Volltext der NeurIPS-PDF | Der einzige belegte Volltext für Konformität mit Drift. Einige Formeln waren beschädigt und sind rekonstruiert. |
| SSSP (Duan) | Zurückgestellt | BMSSP, Volltext, rein theoretisch | Keine Messung, keine Konstanten. Nicht für den Hot Path. |
| Conformal (G.1, Jun/Ohn) | Empfohlen, Prüfung nötig | nicht enthalten | Unklar. |
| Verschiebung (G.2) | Negatives Ergebnis | nicht enthalten | Warnung für alle Konfidenzkomponenten. |

**Konsequenz:** Die einzigen Algorithmen mit Volltextbeleg sind ACI und SSN aus der Langfassung. DiBud ist der aussichtsreichste Kandidat laut v2, hat aber nur Abstract-Beleg. Ich spezifiziere deshalb ACI und SSN vollständig und DiBud mit Kennzeichnung „Eigenableitung, ungeprüft“.

**Wichtig zu den Quellen selbst:** Die Langfassung verweist auf Q1–Q3 als Volltext. Ich habe diese PDFs in dieser Sitzung nicht erneut abgerufen. Ich übernehme die Aussagen aus dem Bericht, nicht aus den Quellen.

---

## 3. Blocker und Behebungsreihenfolge

### Phase 0a: Build grün

- **Q-01a:** Match-Arm für `PurgeBlocked::UnknownFreeSpace` in `purge_driver.rs` ergänzen. Sinnvoller Text: „Freier Platz unbekannt, Probe fehlt.“ und Retry-Hinweis „Produktive Free-Space-Probe konfigurieren“. Dazu die Probe-Semantik aus W16 festlegen: `None` bedeutet „unbekannt“, nicht „0 Byte“.
- **Q-01b:** Fassaden-Wiring unter Feature `router` reparieren (`builder.rs:675-676`). Erst nach N-N prüfbar.
- **Q-01c:** `Cargo.lock` mit `cargo update -p contextra --precise`-freien Mitteln regenerieren und committen. Prüfung mit `--locked`.
- **Abnahme:** `cargo check --locked -p <jedes Default-Mitglied>` und `cargo check --locked -p contextra --features router` grün, mit der im `rust-toolchain.toml` gepinnten Version. Diese Prüfung ist in dieser Sitzung nicht möglich.

### Phase 0b: Löschpfad funktionsfähig

- N-B: `Collection::delete()` darf nicht an einem Downcast scheitern. Interim: Die Capability-Prüfung nur bei Assurance ≥ S1 ausführen. Dafür Test über die Fassade mit einem Tenant-Wrapper schreiben (Tenant-Matrix).

### Phase 0c bis 0e: Siehe Gesamtanalyse Abschnitt 10.1

Nicht wiederholt. Die dortigen Befunde L1–L15, N-A bis N-R bleiben Grundlage, sind aber nur übernommen, nicht reproduziert.

---

## 4. Algorithmen-Spezifikation

Gemeinsam für alle Algorithmen (Systemvorgaben aus v2, Abschnitt 3):

- **V1** Bit-Reproduzierbarkeit bei gleicher Eingabe und gleichem Seed.
- **V2** Totale Ordnung für Tie-Breaks (Dokument-ID als `u64`).
- **V3** Keine Hash-Reihenfolge als Ausgabevertrag. `BTreeMap` oder sortierte `Vec`.
- **V4** Gleitkomma: `total_cmp`, kein `partial_cmp().unwrap()`. Wo möglich Festkomma-Ganzzahlen.
- **V5** Keine Panics im Produktionspfad. Überläufe sind `Result`-Fehler.
- **V7** Budgets vor Ausführung.

### 4.1 Adaptive Conformal Inference (ACI)

**Quelle:** Gibbs, Candès, „Adaptive Conformal Inference Under Distribution Shift“, NeurIPS 2021, arXiv 2106.00170. Volltext gelesen (Langfassung, Mechanismus 1). Einige Symbole im Textextrakt waren beschädigt und sind rekonstruiert, markiert mit [R].

**Problem:** Konforme Vorhersagemengen garantieren Überdeckung nur bei Austauschbarkeit. Bei Drift fällt die Überdeckung mit festem α weg.

**Ein- und Ausgabe:**
- Parameter: Zielniveau α ∈ (0,1), Schrittweite γ > 0, Startwert α₁ ∈ [0,1].
- Pro Schritt: Score-Funktion S_t, Kalibriermenge D_cal, Kovariate X_t. Ausgabe: Vorhersagemenge Ĉ_t.
- Rückkopplung: err_t ∈ {0,1} nach Offenlegung von Y_t.

**Algorithmus (Gl. 2 der Quelle):**

```
init(α, γ, α1):
    require 0 < α < 1, γ > 0, 0 <= α1 <= 1
    a ← α1

predict(x, calib):
    p ← 1 − a
    if p > 1:  return ALL          # α_t < 0  => Quantil = +∞   [R]
    if p < 0:  return EMPTY        # [R] symmetrischer Fall
    if calib leer: return ALL      # Quelle schweigt, konservativ
    n  ← |calib|
    kk ← max(1, ceil(p · n))       # Ordnungsstatistik, [R]
    thr ← kk-kleinster Score in calib
    return { y : S_t(x, y) ≤ thr }

update(covered):
    err ← 0 falls covered, sonst 1
    a ← a + γ · (α − err)          # Gl. 2
```

**Invarianten:**
- **Lemma 4.1 (Quelle):** Fast sicher α_t ∈ [−γ, 1+γ].
- **Proposition 4.1 (Quelle):** Für alle T gilt |(1/T)·Σ err_t − α| ≤ (max{α₁, 1−α₁} + γ)/(T·γ), ohne Verteilungsannahme. Die genaue Formel ist rekonstruiert [R], vor Umsetzung an der PDF prüfen.
- Die Schranke sagt nichts über die Überdeckung zu einem einzelnen Zeitpunkt.

**Komplexität:** Update O(1) Zeit und Zustand. Predict: Ordnungsstatistik über die Kalibriermenge, O(log n) mit geeigneter Struktur, sonst O(n).

**Determinismus:**
- Kein RNG in ACI selbst.
- Zustand ist (t, Fehlerzähler). Die Summe ist reihenfolgeunabhängig (Herleitung aus Gl. 2: α_t = α₁ + γ·(t·α − Σ err_s)).
- Tie-Break bei gleichen Scores: totale Ordnung auf (Score, Einfügeseq) mit `total_cmp`.

**Numerik (Eigenableitung, [E]):**
- Dyadische Festkomma-Darstellung: α_fix = round(α·2^F), γ = g/2^G, Zustand `a: i128` in Einheiten 2^−(F+G). Update `a += g·(α_fix − err·2^F)` ist exakt. Damit gilt die Telescoping-Identität bit-genau.
- Nicht-finite Scores beim Einlesen ablehnen.

**Fehlerfälle:**
- Leere Kalibriermenge: ALL (konservativ, [E]).
- Verzögerte oder gebündelte Labels: von der Quelle als offenes Problem genannt. Nicht abgedeckt.
- Gleichzeitige Löschung kalibrierender Punkte: Ordnungsstatistik aus einem unveränderlichen Snapshot.

**Evaluation (Quelle):** Zwei kleine Experimente (Volatilitätsprognose mit 4 selektierten von 12 Aktien, Wahlnacht 2020). Ergebnisse nur in Abbildungen, keine zitierbare Zahl. Selektionsbias laut Quelle selbst. **Keine Messung auf eigener Hardware.**

**Rust-Umsetzung:** Nur `std`. `i128`, `Vec`, `BTreeMap<(u64, u64), ()>` für das Kalibrierfenster. Indexzugriff über `get`.

**Testplan:**
1. Property-Test: Zustandsgrenzen [−γ, 1+γ] für beliebige, auch adversariale err-Folgen.
2. Property-Test: Teleskopschranke aus Prop. 4.1 nachrechnen.
3. Golden-Test: Vergleich mit rationaler Referenzimplementierung (durch Dyadik möglich).
4. Determinismus: Hash der α-Trajektorie in zwei Läufen.

**Integrationspunkt:** Schreibpfad (Label-Rückmeldung) asynchron. Lesepfad: Snapshot von α_t, synchron.

**Risiken:** Label pro Schritt nötig. Garantie marginal. γ ist heuristisch. Empirie klein und selektiert.

**Einsatzempfehlung:** Geeignet für Konfidenzkomponenten mit Labels. Nicht geeignet, wenn Labels verzögert kommen.

---

### 4.2 Serial Safety Net (SSN), serieller Commit-Test

**Quelle:** Wang, Johnson, Fekete, Pandis, „Efficiently making (almost) any concurrency control mechanism serializable“, arXiv 1605.04292. Volltext gelesen (Langfassung, Mechanismus 2). Datum (13.05.2016) nur laut Sekundärseite. Venue nicht belegt.

**Problem:** Viele serialisierbare Schedules werden von 2PL und SSI abgelehnt. Exaktes Serialization-Graph-Testing braucht pro Commit eine Zyklensuche.

**Ein- und Ausgabe:**
- Voraussetzung: ein zugrunde liegendes CC-Verfahren, das Dirty Reads und Lost Writes verbietet, plus eindeutige u64-Zeitstempel.
- Metadaten: Transaktion (cstamp, status, pstamp = η, sstamp = π, reads, writes), Version (cstamp, pstamp, sstamp, prev).
- Ausgabe: COMMIT oder ABORT.

**Algorithmus (seriell, Algorithmen 1 und 2 der Quelle):**

```
verify(t):                                        # Def. 1: Verletzung wenn π(T) ≤ η(T)
    if t.sstamp <= t.pstamp: return ABORT
    return OK

ssn_read(t, v):
    if v ∉ t.writes:
        t.pstamp ← max(t.pstamp, v.cstamp)                 # w:r-Kante
        if v.sstamp == ∞: t.reads.add(v)
        else:             t.sstamp ← min(t.sstamp, v.sstamp)  # r:w-Kante
        verify(t)

ssn_write(t, v):
    if v ∉ t.writes:
        t.pstamp ← max(t.pstamp, v.prev.pstamp)
        t.writes.add(v)
        t.reads.discard(v)
        verify(t)

ssn_commit(t):                                    # seriell, unter Latch
    t.cstamp ← next_timestamp()
    t.sstamp ← min(t.sstamp, t.cstamp)
    for v in t.reads:  t.sstamp ← min(t.sstamp, v.sstamp)
    for v in t.writes: t.pstamp ← max(t.pstamp, v.prev.pstamp)
    if verify(t) == ABORT: verwerfe Installation; return ABORT
    t.status ← COMMITTED
    for v in t.reads:  v.pstamp ← max(v.pstamp, t.cstamp)
    for v in t.writes:
        v.prev.sstamp ← t.sstamp
        v.cstamp ← t.cstamp; v.pstamp ← t.cstamp; v.sstamp ← ∞
    return COMMIT
```

**Invarianten:**
- Theorem 4 (Quelle): Jede nicht serialisierbare Historie enthält mindestens eine Verletzung des Exclusion-Windows.
- Theorem 6 (Quelle): Mit SSN-Zertifizierung sind alle Ausführungen serialisierbar.
- False Positives sind möglich (Quelle, §3.4).

**Komplexität:** Commit-Arbeit ∝ |reads| + |writes|. Platz je Version konstant (16 Byte Zusatz bei 8-Byte-Stempeln).

**Determinismus:**
- π und η sind min/max über Integer-Stempel, damit reihenfolgeunabhängig.
- Die Entscheidung ist deterministisch **bei fester Commit-Reihenfolge**.
- Nicht deterministisch laut Quelle: `next_timestamp()` unter echter Parallelität, latch-freie Variante (Alg. 3), Reader-Bitmap, manueller Stale-Schwellwert. **Diese Varianten nicht implementieren.** Ein serieller Commit-Sequencer im Log erfüllt V1.

**Fehlerfälle:**
- `prev = None`: explizit behandeln, nicht `unwrap` ([E]).
- Read-Set-Obergrenze: Quelle sieht keine vor. Bei Überschreitung ABORT ([E]).
- Lange lesedominante Transaktionen: Read-Mostly-Optimierung (§5.3). Nicht spezifiziert, weil der Algorithmus Metadaten schreibt.

**Evaluation (Quelle, Autorenmessung):** ERMIA-Implementierung im Speicher, 60 Threads, nur Speicher, ohne Logging. Tabelle 3: TPC-EH SI 10177,37 TPS, SSI 4191,75 TPS, SI+SSN 4450,90 TPS, SI+SSN-R 10525,52 TPS. **Nicht unabhängig reproduziert.** Die Quelle nennt ein „low-fidelity timing model“ für den Simulator.

**Rust-Umsetzung:** Versions-Arena `Vec<Version>` mit Indizes statt Zeigern. `checked_add` für Zeitstempel. Keine `unsafe`.

**Testplan:**
1. Property-Test: Zufalls-Schedules unter RC und SI, danach Offline-SCC-Test auf dem Abhängigkeitsgraphen der committeten Transaktionen. Muss zyklenfrei sein.
2. Write-Skew, Delete/Insert, Self-Write als Regressionsfälle.
3. Determinismus: gleicher Schedule und gleiche Commit-Reihenfolge ergeben gleiche Abort-Menge.

**Bezug zum Bestand:** Im Repository existiert `validate_ssi_gate` (`db_transaction_commit.rs:15`). SSN wäre ein Ersatz oder eine Ergänzung. **Erst entscheiden, nachdem der bestehende Validator gegen die Testfälle aus 1 bis 3 gelaufen ist.**

---

### 4.3 Budgetierte exakte RRF-Präfixe (DiBud), nur Eigenableitung

**Quelle:** Zhang, „Top-K Is Not a Budget for Hybrid Retrieval“, arXiv 2609.15143, eingereicht 14.09.2026. Nur Abstract gelesen. Die Zahlen aus dem Abstract (95,05–97,68 % des nDCG@20 bei 65,92–99,53 % weniger Zugriffen) stammen aus dem Bericht v2, nicht aus eigener Prüfung.

**Status:** Die Zertifizierungsbedingung unten ist **Eigenableitung [E]** aus v2, Abschnitt 4.1. Sie ist nicht gegen die Quelle geprüft.

**Ein- und Ausgabe:** Zwei sortierte Listen (dicht D, spärlich S), Zugriffsbudget B, RRF-Konstante c. Ausgabe: zertifiziertes Präfix und Flag „Budget reichte“.

**Schranken (E):**
- L(d) = Σ über gelesene Kanäle von 1/(c + r_ch(d)).
- U_ch = 1/(c + n_ch), falls Kanal nicht erschöpft, sonst 0.
- U(d) = L(d) + Σ über ungelesene Kanäle von U_ch.
- U_ungesehen = Σ über alle Kanäle von U_ch.

**Zertifizierung (E):** d wird als nächstes Präfix-Element ausgegeben, wenn für alle noch nicht ausgegebenen e ≠ d gilt: L(d) > U(e), oder L(d) = U(e) und ID(d) < ID(e); und L(d) ≥ U_ungesehen (gleiche Tie-Break-Regel).

**Schleife (E):** Kanal mit dem größeren U_ch lesen (Gleichstand: D vor S). Abbruch bei erschöpftem Budget oder wenn nichts zertifizierbar ist.

**Komplexität:** Zeit O(B log B), Speicher O(B). **Nicht aus der Quelle belegt.**

**Numerik:** Summen 1/(c+r) sind rational. Für zwei Kanäle genügt `u128` für Zähler und Nenner. Für mehr Kanäle ist die Nennergröße zu prüfen.

**Kritischer Punkt:** Ein starker Ausreißer in einer Liste hält U_ch hoch, bis das Budget erschöpft ist. Die Quelle sagt dazu nichts. **Testfall mit synthetischem Ausreißer ist Pflicht, bevor DiBud in den Abfragepfad kommt.**

**Testplan:** Golden-Test gegen Brute-Force-RRF auf kleinen Listen. Property-Test: die Ausgabe ist Präfix des exakten Rankings bei beliebigem Budget. Ausreißertest.

---

### 4.4 Entfällt für den Hot Path: BMSSP / FindPivots (Duan et al.)

**Quelle:** Duan et al., arXiv 2504.17033. Volltext gelesen (Langfassung, Mechanismus 3). Algorithmus ist vollständig dort beschrieben.

**Entscheidung: nicht implementieren.** Gründe:
- Keine Messung. Die Quelle hat keinen Experimentabschnitt.
- Die Lemmata drücken die Kosten in |U| und |N⁺(U)| aus. Konstanten sind nicht beziffert.
- Die Parameter k = ⌊log^{1/3} n⌋ und t = ⌊log^{2/3} n⌋ sind für kleine n degeneriert, und die Logarithmusbasis ist nicht angegeben.
- Der Konstantgrad-Umbau ist global.
- Die Quelle selbst vergleicht nur asymptotisch gegen Dijkstra mit Fibonacci-Heap.

**Wenn überhaupt:** nur Offline-Diffusion, nach eigener Messung gegen einen Binär-Heap-Dijkstra auf Zielgraphen. Der Bericht v2 nennt zusätzlich einen Hinweis auf eine Gegenstudie (arXiv 2511.03007). Die ist nicht gelesen.

---

## 5. Architektur: Umbau

Die Zielarchitektur aus der Gesamtanalyse (Abschnitt 8) bleibt die Grundlage. Folgende Punkte sind **nicht** geprüft und stehen unter Vorbehalt:

- **Stufen Off, Assured, Attested:** Konzept übernommen. Im Repository gibt es kein `ErasureMode`. Umsetzung in Phase 1.
- **Ports in `contextra-ports`:** Die Gesamtanalyse nennt 0 Erasure-Symbole. Bestätigt.
- **Fence Option B:** Die Gesamtanalyse hält sie für entscheidbar. Ich habe den Leader-Pfad in `lsm/ops/write.rs` nicht neu gelesen. **Vor Umsetzung lesen.**
- **Crate-Split zuletzt:** Begründung bleibt. Der Split verschiebt Code, der heute einen Build-Fehler hat.

**Reihenfolge (übernommen aus Gesamtanalyse Abschnitt 10):**
1. Build grün (0a)
2. N-B (0b)
3. Gate und Orakel (0c)
4. Purge-Korrektheit (0d)
5. Löschpfad ehrlich (0e)
6. Ports, Off als Nullpfad, Fence B
7. Assured
8. Crate-Split
9. Attested-Härtung
10. Fassade

---

## 6. Empfehlungen in Prioritätsreihenfolge

1. **N-N sofort beheben.** Eine Zeile im Match, mit Test, dass die Engine kompiliert.
2. **Toolchain-Frage klären.** Der Pin 1.89.0 lässt sich in dieser Umgebung nicht laden. Entweder eine Spiegelquelle erlauben (Entscheidung der Betreiber) oder die CI-Umgebung prüfen lassen.
3. **Quellenlage vereinheitlichen.** Die drei Berichte widersprechen sich. Eine einzige Liste mit Belegstatus pro Algorithmus führen.
4. **ACI und SSN vor Ort prüfen,** nicht nur aus den Berichten. Die PDFs sind offen.
5. **DiBud nur mit Ausreißertest.**
6. **Löschpfad nicht als „funktionsfähig“ bezeichnen,** bis N-B, L1 und L4 per Test belegt sind.

---

## 7. Grenzen dieses Dokuments

- **Nicht ausgeführt:** Der vollständige Workspace-Build, alle Tests, `deletion-guarantee-gate`, Benchmarks, Loom, Crash-Matrix. Nur `contextra-engine` wurde mit `stable` geprüft.
- **Toolchain:** Pin 1.89.0 nicht verfügbar (Proxy-Policy). Die Diagnose lief mit `stable`, die Ergebnisse können bei 1.89.0 abweichen.
- **Nicht gelesen:** WAL-Append und Recovery, HNSW-Zeroisierung, KV-Shredding, MCP, Python-Bindings, `license`, Verifier-Details.
- **Nicht neu verifiziert:** Alle Testergebnisse aus Fassung 1 der Gesamtanalyse. Alle Zahlen aus Forschungsberichten.
- **Keine Rechtsberatung:** Die Aussagen zu DSGVO, ZPO § 371a und Crypto-Erase sind nicht geprüft.
- **Commit-Attribution:** Dieses Dokument wurde nicht committet. Falls es committet wird, gilt die vorgegebene Attribution.
