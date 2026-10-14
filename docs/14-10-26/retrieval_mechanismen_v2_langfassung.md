# Recherchebericht v2 (Langfassung): Verifizierte Mechanismen für ein eingebettetes, deterministisches Retrieval-System

**Dieses Dokument ersetzt alle vorherigen Antworten und Dateien dieser Konversation.** Es enthält den vollständigen Bericht (Teile 1–7), das Recherche-Protokoll und die Konventionen.

**Stand:** 2026-10-14

---

## 0. Konventionen, Umfang und Abweichungen

### 0.1 Abweichung vom Auftrag (vorab)

Dieser Durchlauf liefert **3 vollständig spezifizierte Mechanismen**, nicht 8–12. Begründung: Qualität der Quellenbelege hat Vorrang vor Anzahl (Auftrag §0, §7). Felder, die ich nicht bearbeitet habe, stehen in Teil 6 als „nicht recherchiert“. Das ist etwas anderes als „keine belegbare Quelle gefunden“.

### 0.2 Belegstatus

| Kürzel | Bedeutung |
|---|---|
| **[V]** | Volltext gelesen, Aussagen am Volltext geprüft |
| **[A]** | nur Abstract (bzw. Abstract-Seite) gelesen |
| **[S]** | nur über Sekundärquelle bekannt |
| **[X]** | nicht verifiziert |
| **[Entwurf]** | eigene Umsetzung oder Herleitung, **kein** Quellenbeleg |

### 0.3 Zitat- und Zahlenregel

- Wörtlich zitiere ich höchstens **ein Mal je Quelle** und mit **weniger als 15 Wörtern**. Das ist strenger als die Auftragsregel „wörtliches Zitat je Zahl“ und hat Vorrang (Urheberrecht).
- Zahlen aus Tabellen gebe ich mit **Fundstelle** (Abschnitt/Tabelle) statt Zitat an.
- Zahlen ohne Fundstelle stehen nicht im Bericht.

### 0.4 Recherche-Protokoll (was tatsächlich geschah)

| Schritt | Aktion | Ergebnis |
|---|---|---|
| 1 | Suche: *Breaking the Sorting Barrier for Directed Single-Source Shortest Paths arXiv* | arXiv-Abstract- und PDF-Seiten, Sekundärseiten (nicht als Quelle verwendet) |
| 2 | Abruf `arxiv.org/abs/2504.17033` | Autorenliste, Datum, Abstract |
| 3 | Abruf `arxiv.org/pdf/2504.17033v2` | **Volltext** (bis Referenzen) |
| 4 | Abruf `arxiv.org/abs/2106.00170` | **fehlgeschlagen** (URL nicht in vorherigen Ergebnissen) |
| 5 | Suche: *Adaptive Conformal Inference Under Distribution Shift Gibbs Candès arXiv* | arXiv-v3-Abstract, NeurIPS-Proceedings, ar5iv, Treffer zu arXiv 2208.08401 |
| 6 | Abruf NeurIPS-Proceedings-PDF `papers.nips.cc/…0d441de7…Paper.pdf` | **Volltext** (13 Seiten inkl. Referenzen) |
| 7 | Suche: *Efficiently making (almost) any concurrency control mechanism serializable Serial Safety Net* | arXiv-PDF, ESSN-Abstract, Sekundärseiten |
| 8 | Abruf `arxiv.org/pdf/1605.04292` | **Volltext** (bis Abschnitt 9, vom Abrufer gekürzt) |

Hinweise zur Reichweite:
- Q3 wurde bis in den Abschnitt „Related Work“ gelesen. Die letzten Absätze von Abschnitt 9 und Abschnitt 10 (Fazit) fehlten im Abruf. Für die berichteten Aussagen ist das unerheblich, wird aber hier benannt.
- Q4 und Q5 wurden nur über arXiv-Abstract-Treffer in Suchergebnissen geöffnet.
- Sekundärseiten (Blogs, Scribd, alphaXiv, DEV, HuggingFace, deepai, itnext) sind **keine** Quellen des Berichts. Einzige Ausnahme: das Datum von Q3 (nur [S], so markiert).

---

## 1. Teil 1: Kurzfassung

**Anzahl vollständig spezifizierter Mechanismen: 3.** Sie ist identisch mit der Zahl der Abschnitte in Teil 2 und der Zahl der tragenden Quellen (Q1–Q3) in Teil 5.

| Rang | Mechanismus | Feld | Art des Belegs | Pflichtkriterien |
|---|---|---|---|---|
| 1 | Adaptive Conformal Inference (ACI) | H | Beweis plus kleine Empirie | 1 ✔ (ganzzahlig implementiert), 2 ✔ O(1), 3 ✔ O(1)-Zustand |
| 2 | Serial Safety Net (SSN) | F | Beweis plus Messung (ERMIA, nur im Speicher) | 1 ✔ **nur bei fester Commit-Reihenfolge**, 2 ✔ ∝ Footprint, 3 ✔ ∝ Footprint |
| 3 | BMSSP/FindPivots (Duan et al.) | A | **rein Theorie, keine Messung** | 1 ✔, 2 bedingt (Konstante C unbeziffert), 3 bedingt (nur über Lemma 3.12) |

Kernaussagen:
- **ACI:** ein einzelner Skalar-Zustand, verteilungsfreie Langzeit-Überdeckung (Proposition 4.1), kommutativer Zustand (t, Fehlerzähler) [Entwurf-Herleitung].
- **SSN:** Commit-Test `π ≤ η ⇒ abort` über min/max auf Integer-Stempeln. Die latch-freie Variante der Quelle ist **nicht** deterministisch.
- **BMSSP:** O(m·log^{2/3} n) asymptotisch. **Die Quelle zeigt keinen praktischen Vorteil.** Sie enthält keinen Experimentabschnitt.

Gesamteinordnung: ACI und SSN sind die einzigen beiden Kandidaten mit belegbarer Praxisrelevanz und sauberem Determinismus-Pfad. BMSSP ist theoretisch interessant und bleibt ohne eigene Messung hinter Dijkstra zurück.

---

## 2. Teil 2: Mechanismen (gerankt)

---

### Mechanismus 1: Adaptive Conformal Inference (ACI) — Feld H

#### 1. Name und Kurzbeschreibung
ACI führt ein einzelnes Miscoverage-Niveau α_t per Online-Update nach. So bleibt die Überdeckungsrate eines Vorhersagemengen-Verfahrens auch bei Verteilungsverschiebung langfristig auf dem Zielwert.

#### 2. Quelle
- Erstautor Isaac Gibbs; Autoren: Isaac Gibbs, Emmanuel Candès.
- Titel: *Adaptive Conformal Inference Under Distribution Shift*.
- NeurIPS 2021 (Advances in Neural Information Processing Systems 34), laut Proceedings-Seite. Die neurips.cc-Seite führt den Beitrag als „Oral“.
- arXiv 2106.00170 (Abstract-Seite v3 geöffnet; Einreichungsdatum dort nicht angezeigt).
- **Belegstatus: [V]** (Volltext der Proceedings-PDF, 13 Seiten).

#### 3. Problem
Konforme Verfahren garantieren Überdeckung nur bei Austauschbarkeit der Daten. Mit festem α fällt die Überdeckung bei Drift weg. Die Autoren zeigen das an einer Volatilitätsprognose: Das nicht adaptive Verfahren verfehlte bei Fannie Mae in der Krise 2008 die Zielüberdeckung (Abschnitt 2.2, Fig. 1, Fig. 4 im Anhang).

#### 4. Ein- und Ausgaben
- **Parameter:** Zielniveau α ∈ (0,1); Schrittweite γ > 0; Startwert α₁ ∈ [0,1] (in den Experimenten α₁ = α).
- **Pro Schritt t gegeben:**
  - Score-Funktion S_t(x, y), Beispiele der Quelle: |µ̂(X_t) − y| oder der CQR-Score max{q̂(X_t; α/2) − y, y − q̂(X_t; 1−α/2)}.
  - Quantilfunktion Q̂_t aus einer Kalibriermenge D_cal.
  - Q̂(p) := inf{ s : (1/|D_cal|)·Σ 1{S ≤ s} ≥ p } (Gl. 1).
  - Kovariaten X_t.
- **Ausgabe:** Ĉ_t(α_t) = { y : S_t(X_t, y) ≤ Q̂_t(1 − α_t) }.
- **Rückkopplung:** Nach jedem Schritt wird Y_t offenbart; daraus err_t ∈ {0,1}.
- **Zustand:** α_t ∈ [−γ, 1+γ] (Lemma 4.1).

#### 5. Algorithmus (Gl. 2 und 3, Abschnitt 2)

```
init(α, γ, α1):
    require 0 < α < 1, γ > 0, 0 <= α1 <= 1
    a ← α1                                # Quelle: α1 = α in den Experimenten

predict(x, calib):
    p ← 1 − a
    if p > 1:  return ALL                 # α_t < 0 ⇒ Q̂ = +∞ (Beweis Lemma 4.1; Unendlich-Symbol im Textextrakt
                                          #  beschädigt, aus der Beweislogik rekonstruiert)
    if p < 0:  return EMPTY               # symmetrischer Fall; Quelle: "the case ... is identical" [Entwurf-Lesart des Symbols]
    if calib leer: return ALL             # Quelle schweigt [Entwurf]
    n  ← |calib|
    kk ← max(1, ceil(p · n))              # inf{s : F(s) ≥ p} = kk-te Ordnungsstatistik [Entwurf: Herleitung aus Gl. 1]
    thr ← kk-kleinster Score in calib
    return { y : S_t(x, y) ≤ thr }

update(covered):
    err ← 0 falls covered, sonst 1
    a ← a + γ · (α − err)                 # Gl. (2)
```

- **Variante (Gl. 3):** ersetze err_t durch Σ_s w_s·err_s mit w_s = 0.95^{t−s} / Σ_{s'} 0.95^{t−s'}. Die Quelle sagt, (2) und (3) lieferten „almost identical results“; (3) liefert glattere Trajektorien (Abschnitt 2, Anhang A.3).
- **Terminierung:** keine Schleife, jeder Schritt terminiert in O(1) (Q̂-Berechnung ausgenommen). **Rückgabe:** Vorhersagemenge, Zustand α_t.
- **Fallback:** triviale Mengen ALL/EMPTY, wenn 1−α_t außerhalb [0,1].

#### 6. Invarianten und Korrektheitsargument
- **Lemma 4.1 (Abschnitt 4.1):** Fast sicher gilt α_t ∈ [−γ, 1+γ] für alle t.
  - Beweisidee: α_t < 0 ⇒ Q̂_t(1−α_t) = ∞ ⇒ err_t = 0 ⇒ α_{t+1} ≥ α_t. Ein Abstieg unter 0 ist damit durch γ begrenzt.
- **Proposition 4.1 (Abschnitt 4.1):** Für alle T gilt
  |(1/T)Σ_{t≤T} err_t − α| ≤ (max{α₁, 1−α₁} + γ) / (T·γ),
  **ohne Annahme an die Datenverteilung.** Beweis: Teleskopieren von Gl. 2 und Lemma 4.1. Die Formel ist aus dem beschädigten Extrakt und der Beweisstruktur rekonstruiert (Sichtprüfung offen). Folge: lim (1/T)Σ err_t = α fast sicher.
- **Ehrliche Einordnung der Quelle:** Die Schranke ist nahezu scharf direkt nach der Initialisierung (Diskussion nach Prop. 4.1); sie sagt nichts über Überdeckung zu einem einzelnen Zeitpunkt.
- **Theoreme 4.1 und 4.2** gelten nur im Hidden-Markov-Modell (stationäre Verteilung, spektrale Lücke). Theorem 4.2 beschränkt E[(M(α_t|A_t) − α)²]; die Schranke wird minimal bei γ = (2·E|α*_{A_{t+1}} − α*_{A_t}|)^{1/2}. Die Quelle merkt an, das sei „not directly applicable in practice“ (Abschnitt 2.1).

#### 7. Komplexität
- Update: O(1) Zeit, O(1) Zustand. Die Quelle betont, dass nur ein einzelner Parameter verfolgt wird (Einleitung).
- Q̂-Kosten: abhängig von der Kalibrierung (Sortieren O(n log n) oder Ordnungsstatistik-Struktur). Die Quelle analysiert das nicht.
- I/O: nicht zutreffend.
- Schranken sind asymptotisch in T. Verborgene Konstanten: keine bekannt.

#### 8. Determinismus
- ACI selbst enthält keinen RNG.
- Die Quelle erwähnt zusätzliche Randomisierung für **exakte** Überdeckung und lässt sie aus (Abschnitt 1.1). Ohne sie gilt unter Austauschbarkeit die Überdeckung ⌈|D_cal|(1−α)⌉/(|D_cal|+1).
- **Tie-Break bei gleichen Scores [Entwurf]:** totale Ordnung auf (Score, Einfügeseq) mit `f64::total_cmp`.
- **Reduktionsreihenfolge:** keine Gleitkomma-Reduktion im Update; Q̂ ist eine Ordnungsstatistik.
- **Hash-Behandlung:** nicht beteiligt.
- **RNG-Seed-Ableitung:** nicht zutreffend (kein RNG).
- **Herleitung [Entwurf]:** Aus Gl. 2 folgt α_t = α₁ + γ·(t·α − Σ_{s<t} err_s). Der Zustand ist damit nur (t, Fehlerzähler) und die Summe ist reihenfolgeunabhängig.

#### 9. Numerik
Die Quelle schweigt zu NaN und Überlauf. [Entwurf]:
- Nicht-finite Scores beim Ingest ablehnen; Score-Vergleiche über `total_cmp`.
- **Exakte Rekursion:** α_fix = round(α·2^F), γ = g/2^G, Zustand `a: i128` in Einheiten 2^−(F+G). Update `a += g·(α_fix − err·2^F)` ist exakt und plattformidentisch; die Telescoping-Identität aus Prop. 4.1 gilt bit-genau für die diskretisierten Parameter.
- Wegen |a| ≤ (1+γ)·Skala tritt kein Überlauf auf; Unterlauf gibt es in Ganzzahlen nicht.

#### 10. Fehler- und Randfälle
- Leere Kalibrierung: Q̂ laut Quelle undefiniert; Vorschlag: triviale Menge ALL.
- Verzögerte oder gebündelte Labels: **von der Quelle als offenes Problem genannt** (Abschnitt 7). Nicht abgedeckt.
- Duplikate: unproblematisch.
- Erschöpftes Budget: nicht zutreffend.
- Gleichzeitige Löschung kalibrierender Punkte [Entwurf]: Q̂ aus unveränderlichem Snapshot berechnen.

#### 11. Evaluation (empirisch, klein)
- Zwei Datensätze, von den Autoren selbst als Begrenzung genannt (Abschnitt 7).
- **(a) Volatilitätsprognose:** 4 von 12 untersuchten Aktien. Laut Quelle wurden die vier ausgewählt, weil das nicht adaptive Verfahren dort **klar versagte** ⇒ Selektionsbias. Setup: GARCH(1,1) auf die letzten 1250 Handelstage, α = 0.1, γ = 0.005, Messgröße lokale Überdeckung über 500 Tage (Gl. 4). Baseline: festes α_t = α. Preisdaten laut Quelle vom Wall Street Journal.
- **(b) Wahlnacht 2020:** Vorhersage der Biden-Stimmen für etwa 3000 US-Counties mit CQR, Ziel 90 %, Counties nach Zeitzone geordnet (innerhalb zufällig). Baseline: festes α_t.
- **Hardware:** nicht angegeben.
- **Zahlenergebnisse stehen nur in Abbildungen (Fig. 1–3, 9); keine zitierbare Zahl.**
- Zitat (Abstract): „robust to visible and significant distribution shifts“.
- Art: **empirisch** (Beweis separat in Punkt 6).

#### 12. Rust-Umsetzung
- Nur Std: `i128`, `Vec`, `BTreeMap<(u64 /*order-erhaltende Bits*/, u64 /*seq*/), ()>` für das Kalibrierfenster [Entwurf]. Keine externen Crates.
- Panic-Stellen: Indexzugriff auf das k-te Element (`get` statt `[]`); `ceil(p·n)` bei n = 0 (explizit abfangen); Arithmetik-Überlauf (`checked_*`); `as`-Casts.

#### 13. Testplan
- **Property-Test:** Prop. 4.1 und Zustandsgrenzen [−γ, 1+γ] für beliebige, auch adversariale err-Folgen.
- **Golden-Test:** bit-genauer Vergleich mit einer Referenzimplementierung in rationaler Arithmetik (möglich wegen dyadischer Parametrisierung).
- **Fuzzing:** Score-Folgen mit Duplikaten und Extremwerten.
- **Determinismus:** Hash der α-Trajektorie in zwei Läufen und auf x86 und ARM; Generator-Seeds aus dem Testnamen abgeleitet.

#### 14. Integrationspunkt
- Schreibpfad: Label-Rückmeldung, asynchron; wegen Kommutativität reihenfolgeunabhängig, solange t konsistent gezählt wird.
- Lesepfad: Snapshot von α_t beim Entscheidungsschritt, synchron und wartefrei.

#### 15. Risiken, offene Fragen, Gegenbelege
- Label je Schritt nötig.
- Garantie marginal, nicht bedingt.
- γ heuristisch.
- Empirie klein und selektiert; Heavy-Tail-Verhalten nicht gezeigt.
- Keine Gegenbelegsuche durchgeführt.

---

### Mechanismus 2: Serial Safety Net (SSN) — Feld F

#### 1. Name und Kurzbeschreibung
SSN ist ein Commit-Zertifizierer. Er erzwingt Serialisierbarkeit auf Read Committed oder Snapshot Isolation über zwei Zeitstempel-Wasserzeichen π und η und einen Exclusion-Window-Test.

#### 2. Quelle
- Erstautor Tianzheng Wang; Autoren: Tianzheng Wang, Ryan Johnson, Alan Fekete, Ippokratis Pandis (Titelseite der PDF).
- Titel: *Efficiently making (almost) any concurrency control mechanism serializable*.
- arXiv 1605.04292. Datum 13.05.2016 **nur laut Sekundärseite (deepai) [S]**.
- Venue: auf keiner geöffneten Seite belegt. Der Text verweist auf eine frühere Version [50].
- **Belegstatus: [V]** (Text); Datum [S].

#### 3. Problem
- 2PL und SSI verbieten viele gültige serialisierbare Schedules.
- OCC bricht bei langen, lesedominanten Transaktionen zusammen.
- Exaktes Serialization-Graph-Testing ist unpraktisch, weil jeder Commit eine Zyklensuche braucht (Abschnitt 1).
- SSN prüft nur **direkte** Abhängigkeiten und kann den Großteil der Lesevorgänge unverfolgt lassen.

#### 4. Ein- und Ausgaben
- **Vorausgesetzt:** ein zugrunde liegendes CC-Verfahren, das **Dirty Reads und Lost Writes** verbietet („certifiable scheduler“, Def. 3; mindestens so stark wie RC), plus global eindeutige Zeitstempel (u64).
- **Metadaten (Tabelle 1):**
  - Transaktion: cstamp (Initialwert 0), status, pstamp = η (0), sstamp = π (∞), reads, writes.
  - Version: cstamp, pstamp, sstamp (∞), prev.
- **Ausgabe:** Commit oder Abort je Transaktion. Garantie: nur serialisierbare Ausführungen (Theorem 6).

#### 5. Algorithmus (Algorithmus 1 und 2, Abschnitt 4.1, serielle Commit-Variante)

```
verify(t):                                   # Def. 1 + §3.1: Verletzung wenn π(T) ≤ η(T)
    if t.sstamp <= t.pstamp: abort(t); return ABORT
    return OK

ssn_read(t, v):                              # v vom zugrunde liegenden CC geliefert
    if v ∉ t.writes:
        t.pstamp ← max(t.pstamp, v.cstamp)           # w:r-Kante
        if v.sstamp == ∞: t.reads.add(v)             # noch nicht überschrieben
        else:             t.sstamp ← min(t.sstamp, v.sstamp)    # r:w-Kante
        verify(t)

ssn_write(t, v):                             # v = neue Version, v.prev vom CC gesetzt
    if v ∉ t.writes:
        t.pstamp ← max(t.pstamp, v.prev.pstamp)
        t.writes.add(v)
        t.reads.discard(v)                           # vermeidet Selbstkante / unnötigen Abort
        verify(t)

ssn_commit(t):                               # unter Latch, seriell
    t.cstamp ← next_timestamp()                      # Pre-Commit
    t.sstamp ← min(t.sstamp, t.cstamp)
    for v in t.reads:  t.sstamp ← min(t.sstamp, v.sstamp)
    for v in t.writes: t.pstamp ← max(t.pstamp, v.prev.pstamp)
    if verify(t) == ABORT:
        entferne installierte Versionen; verwirf Stempel; return ABORT
    t.status ← COMMITTED                             # Post-Commit
    for v in t.reads:  v.pstamp ← max(v.pstamp, t.cstamp)
    for v in t.writes:
        v.prev.sstamp ← t.sstamp
        v.cstamp ← t.cstamp; v.pstamp ← t.cstamp; v.sstamp ← ∞    # Initialwerte laut Tabelle 1
    return COMMIT
```

- **Terminierung:** alle Schleifen sind durch |reads| bzw. |writes| beschränkt. **Rückgabe:** COMMIT oder ABORT.
- **Bewusst nicht spezifiziert:**
  - Algorithmus 3 (latch-frei, §4.2: CAS, Spin-Waits, Thread-Bitmap für Reader): nicht deterministisch (Punkt 8).
  - Phantomschutz (§6: hierarchische Locks, Gap-Locks nach Graefe). Er ist für ein System mit Inserts und Bereichsscans erforderlich.
  - Read-Mostly-Optimierung (§5.3) mit `last_cstamp` und MSB-Lock auf sstamp.

#### 6. Invarianten und Korrektheitsargument
- **Definition 1:** Eine Kante U ← T verletzt das Exclusion-Window von T, wenn π(T) ≤ c(U) < c(T). Dabei ist π(T) der Commit-Zeitpunkt des ältesten Nachfolgers von T über eine Kette von Back-Kanten; η(T) ist der größte Commit-Zeitpunkt direkter Vorgänger über Forward-Kanten.
- **Theorem 4:** Jede nicht serialisierbare Historie eines certifiable scheduler enthält mindestens eine solche Verletzung. Beweisskizze (§3.3): Im Zyklus sei T_n als Erster committet; wähle das kleinste k mit T_k ←b* T_n; dann π(T_k) ≤ c(T_n) ≤ c(T_{k−1}) < c(T_k).
- **Theorem 6:** Mit SSN-Zertifizierung (Def. 5) sind alle Ausführungen serialisierbar.
- **Theorem 7 (Safe Retry):** Ein sofortiger Retry scheitert nicht an derselben Abhängigkeit. Annahme: T sieht keine Versionen, die vor seinem Beginn überschrieben wurden.
- SSN lässt False Positives zu (§3.4, Fig. 2d).
- Die Quelle behauptet, SSN funktioniere trotz Fehlern im zugrunde liegenden CC, „so long as the basic requirements still hold“ (§1.1). Das ist eine Autorenbehauptung.

#### 7. Komplexität
- Platz und Zeit der Basisprotokolle: linear im kombinierten Lese-/Schreib-Footprint laufender Transaktionen, plus konstanter Platz je Version (§4.1).
- Pre-Commit-Arbeit ∝ |reads| + |writes| (§5).
- Platz je Version: 16 Byte zusätzlich bei 8-Byte-Stempeln (Fußnote 5); pstamp und sstamp lassen sich in ein Maschinenwort packen (§5.1).
- I/O: im speicheroptimierten Modell außerhalb des kritischen Pfads.
- Verborgene Konstanten: Spin-Waits der latch-freien Variante hängen von der Konkurrenz ab; die Quelle gibt keine Schranke an.

#### 8. Determinismus
- π und η sind min/max über Integer-Stempel ⇒ **reihenfolgeunabhängige** Reduktionen. Keine Hashes, kein Gleitkomma.
- Die Entscheidung ist deterministisch **bei fester Commit-Reihenfolge**.
- **Nichtdeterministische Stellen laut Quelle:**
  - `next_timestamp()` per Fetch-and-Add unter echter Parallelität (§1.1).
  - Spin/CAS-Protokoll in Alg. 3.
  - Reader-Bitmap: Slot-Wiederverwendung erzeugt scheduling-abhängige False Positives (§4.2.3).
  - Manuell gesetzter Stale-Schwellwert (§5.3, Fn. 11).
- **[Entwurf] Deterministische Konfiguration:** ein serieller Commit-Sequencer in Log-Reihenfolge (Alg. 2). Kein RNG nötig.

#### 9. Numerik
- Nur u64-Zeitstempel. ∞ als `u64::MAX` [Entwurf]; Zählerüberlauf mit `checked_add` melden.
- NaN und `total_cmp`: nicht zutreffend.

#### 10. Fehler- und Randfälle
- Reine Lesetransaktionen können über Safe Snapshots (§5.2) SSN komplett umgehen.
- Wiederholte Lesezugriffe: Non-Repeatable Reads erscheinen als Zyklus T ←r:w W ←w:r T und werden von SSN erkannt (§4.1.2).
- Eigenes Lesen: ignoriert (`v ∉ t.writes`).
- Löschen: „invalid“-Wert als letzte Version, physisch im Hintergrund entfernt, sobald unerreichbar (§2). Einfügen: Update einer „invalid“-Version, `prev` existiert daher im Modell immer.
- [Entwurf] `prev = None` explizit behandeln statt `unwrap`.
- Erschöpftes Budget: von der Quelle nicht vorgesehen. [Entwurf] Read-Set-Obergrenze, bei Überschreitung Abort.
- Lange Transaktionen: Read-Mostly-Optimierung (§5.3).

#### 11. Evaluation (empirisch: Autoren-Implementierung in ERMIA plus Simulation)
- **Hardware:** Quad-Socket-Linux-Server, Intel Xeon E7-4890 v2 mit 2,8 GHz, 60 Kerne, 3 TB RAM; Logs gehen nach `/dev/null` (§8.2).
- **Benchmarks:** TPC-C, TPC-CC (hohe Konkurrenz), TPC-EH (mit langer, lesedominanter Asset-Eval-Transaktion).
- **Baselines:** SI, SSI (jeweils in ERMIA) und OCC (Silo).
- **Zitat (§8.5, TPC-CC, 60 Threads, ohne Retry):** „SSN exhibits overall ∼60% lower aborts due to serialization failures when compared to SSI“.
- **Tabelle 3 (TPC-EH, 60 Threads, Gesamt-TPS):**

  | Variante | TPS |
  |---|---|
  | SI | 10177,37 |
  | SSI | 4191,75 |
  | SI+SSN | 4450,90 |
  | SI+SSN-R (Read-Mostly-Optimierung) | 10525,52 |

- Laut §8.4 liegt SI+SSN-R gegenüber vanilla SSN rund 136 % (gesamt) und 97 % (Asset-Eval) höher.
- Der Simulator hat laut Autoren ein „low-fidelity timing model“ (§7.1).
- Einschränkung: nur im Speicher, ohne Logging gemessen; Ergebnisse stammen von den Autoren selbst.

#### 12. Rust-Umsetzung
- Nur Std: Versions-Arena `Vec<Version { cstamp: u64, ps: u64, prev: Option<u32> }>` (ps = geteiltes Feld nach §5.1), Indizes statt Zeiger, kein `unsafe`. Keine externen Crates.
- Panic-Stellen: `prev`-Dereferenzierung, Indexzugriffe (`get`), Zeitstempelüberlauf (`checked_add`). Rekursion: keine.

#### 13. Testplan
- **Property-Test nach der Methode der Quelle (§7.2):** Zufalls-Schedules kleiner Transaktionen unter RC und SI laufen lassen; danach Offline-SCC-Test auf dem Abhängigkeitsgraphen der committeten Transaktionen (muss zyklenfrei sein). Der Test ist unabhängig vom CC-Verfahren.
- **Golden-Test:** Abort-Mengen gegen den Simulator der Autoren (Fußnote 8 der Quelle; Repository nicht geöffnet).
- **Fuzzing:** doppelte Lesezugriffe, Delete/Insert, Self-Write.
- **Determinismus:** gleiche Schedule und gleiche Commit-Reihenfolge ⇒ identische Abort-Menge, in zwei Läufen und auf zwei Plattformen.

#### 14. Integrationspunkt
- Schreibpfad, synchron: Commit-Validierung.
- Lesepfad: blockiert nicht, **schreibt aber beim Commit `v.pstamp`**. Read-only-Anfragen über Safe Snapshot (§5.2) sind metadatenfrei.

#### 15. Risiken, offene Fragen, Gegenbelege
- False Positives.
- Stale-Schwellwert nicht selbstjustierend (Fn. 11).
- Latch-freie Variante nichtdeterministisch.
- Lock-basiertes SSN nennt die Quelle als zukünftige Arbeit (§4).
- Nur im Speicher gemessen.
- ESSN (Teil 3) behauptet laut Abstract eine Verallgemeinerung; ungeprüft.
- Keine Gegenbelegsuche durchgeführt.

---

### Mechanismus 3: BMSSP / FindPivots (Duan et al.) — Feld A

#### 1. Name und Kurzbeschreibung
Rekursive Teile-und-herrsche-Kombination aus Dijkstra und Bellman-Ford. Sie vermeidet die vollständige Sortierung der Frontier über „Pivots“.

#### 2. Quelle
- Erstautor Ran Duan; Autoren: Ran Duan, Jiayi Mao, Xiao Mao, Xinkai Shu, Longhui Yin.
- Titel: *Breaking the Sorting Barrier for Directed Single-Source Shortest Paths*.
- arXiv 2504.17033; eingereicht 2025-04-23, v2 2025-07-30. Venue: auf den geöffneten Seiten nicht belegt (Stand: arXiv-Preprint).
- **Belegstatus: [V]** (v2 vollständig bis zu den Referenzen).

#### 3. Problem
Dijkstra sortiert die Frontier implizit (Θ(n log n)). Das Paper behandelt gerichtete Graphen mit reellen, nichtnegativen Gewichten im Vergleichs-Additions-Modell. Zitat (§1.1): „breaks the sorting bottleneck on sparse graphs“.

#### 4. Ein- und Ausgaben
- **Eingabe:** G = (V,E), w: E → ℝ≥0, Quelle s. Alle Knoten von s erreichbar (OBdA, §2).
- **Ausgabe:** d(v) für alle v. **Keine** Knotenordnung nach Distanz.
- **Intern:** Level l ∈ [0, ⌈log n / t⌉], Menge S mit |S| ≤ 2^{lt}, Schranke B.

#### 5. Algorithmus (§2, §3.1, Algorithmen 1–3, Lemma 3.3)

**Definitionen:**
- d̂[v]: Distanzschätzung, immer ≥ d(v). „Vollständig“: d̂[v] = d(v).
- Frontier S: Knoten mit b ≤ d̂ < B.
- Pivot: Wurzel eines Baums mit ≥ k Knoten im Wald F.
- **Totale Ordnung ≺ (Annahme 2.1):** Pfade als Tupel ⟨Länge, Knotenanzahl α, v_α, …, v_1⟩ lexikographisch. In O(1) vergleichbar über PRED und α. Damit hat jeder Pfad eine eindeutige Länge und F ist ein Wald.
- **Parameter:** k := ⌊log^{1/3} n⌋, t := ⌊log^{2/3} n⌋ (Logarithmusbasis in der Quelle nicht angegeben).

```
SSSP(G, s):
    G' ← ConstDegree(G)      # §2: jeder Knoten v → Zyklus aus Nullgewicht-Kanten mit Knoten x_vw je Nachbar w;
                             #     jede Kante (u,v) → (x_uv → x_vu) mit Gewicht w_uv; Grad ≤ 2, O(m) Knoten
    n ← |V(G')|; berechne k, t; L ← ⌈log n / t⌉
    d̂[v] ← ∞ ∀v; d̂[s] ← 0; PRED[v] ← ⊥
    (B', U) ← BMSSP(L, ∞, {s})
    require B' == ∞          # Quelle: |U| ≤ |V| = o(kn) ⇒ erfolgreiche Ausführung; sonst Err
    return d̂

FindPivots(B, S):                                    # Algorithmus 1
    W ← S; Wprev ← S
    for i in 1..=k:
        Wi ← ∅
        for (u,v) ∈ E mit u ∈ Wprev:
            if d̂[u] + w(u,v) ≤ d̂[v]:                # "≤" erlaubt Wiederverwendung auf höherem Level (Remark 3.4)
                d̂[v] ← d̂[u] + w(u,v); PRED[v] ← u
                if d̂[u] + w(u,v) < B: Wi ← Wi ∪ {v}
        W ← W ∪ Wi
        if |W| > k·|S|: return (P = S, W)
        Wprev ← Wi
    F ← {(u,v) ∈ E : u,v ∈ W ∧ d̂[v] = d̂[u] + w(u,v)}   # gerichteter Wald (Ann. 2.1)
    P ← {u ∈ S : u ist Wurzel eines Baums in F mit ≥ k Knoten}
    return (P, W)

BaseCase(B, S = {x}):                                # Algorithmus 2, x vollständig
    U0 ← {x}; H ← Binärheap mit (x, d̂[x]), Schlüsselordnung ≺
    while H ≠ ∅ and |U0| < k+1:
        (u,_) ← H.extract_min(); U0 ← U0 ∪ {u}
        for (u,v) ∈ E:
            if d̂[u] + w ≤ d̂[v] and d̂[u] + w < B:
                d̂[v] ← d̂[u] + w; PRED[v] ← u
                if v ∉ H: H.insert(v) else H.decrease_key(v)
    if |U0| ≤ k: return (B, U0)
    B' ← max_{v ∈ U0} d̂[v]; return (B', {v ∈ U0 : d̂[v] < B'})

# Datenstruktur D (Lemma 3.3; Aufbau aus dem Beweis, Pseudocode von mir geformt)
D.Initialize(M, B): D0 ← []; D1 ← [Block(ub = B)]; BST über D1-Obergrenzen; Blockgröße ≤ M
D.Insert(a,b):      if a ∈ D mit Wert b': if b ≥ b' return else D.Delete(a,b')
                    Block in D1 mit kleinster Obergrenze ≥ b (per BST); anhängen; |Block| > M ⇒ Split
D.Delete(a,b):      aus Liste entfernen, O(1); wird ein D1-Block leer: Obergrenze aus BST entfernen
Split(Block):       Median per BFPRT; untere Hälfte bleibt, obere Hälfte neuer Block; BST aktualisieren
D.BatchPrepend(L):  je Schlüssel kleinsten Wert behalten; alle Werte < jedem Wert in D
                    |L| ≤ M: ein neuer Block vorn in D0
                    sonst: wiederholte Medianteilung in Blöcke ≤ ⌈M/2⌉, vorn in D0
D.Pull():           S0', S1' = Präfixblöcke von D0, D1 bis ≥ M Elemente (oder erschöpft)
                    if |S0' ∪ S1'| ≤ M: S' ← alle; x ← B
                    else: S' ← M kleinste; x ← kleinster verbleibender Wert
                    S' aus D löschen; return (x, S')       # max(S') < x ≤ min(D)

BMSSP(l, B, S):                                      # Algorithmus 3
    if l == 0: return BaseCase(B, S)
    (P, W) ← FindPivots(B, S)
    D.Initialize(M = 2^{(l-1)t}, B); für x ∈ P: D.Insert(x, d̂[x])
    i ← 0; B'_0 ← min_{x ∈ P} d̂[x]  (falls P = ∅: B'_0 ← B); U ← ∅
    while |U| < k·2^{lt} and D ≠ ∅:
        i ← i+1
        (B_i, S_i) ← D.Pull()
        (B'_i, U_i) ← BMSSP(l−1, B_i, S_i)
        U ← U ∪ U_i; K ← ∅
        for (u,v) ∈ E mit u ∈ U_i:
            if d̂[u] + w ≤ d̂[v]:
                d̂[v] ← d̂[u] + w; PRED[v] ← u
                if d̂[u] + w ∈ [B_i, B):      D.Insert(v, d̂[u] + w)
                elif d̂[u] + w ∈ [B'_i, B_i):  K ← K ∪ {(v, d̂[u] + w)}
        D.BatchPrepend(K ∪ {(x, d̂[x]) : x ∈ S_i, d̂[x] ∈ [B'_i, B_i)})
    return (min(B'_i, B), U ∪ {x ∈ W : d̂[x] < min(B'_i, B)})
```

- **Fundstellen:** Alg. 1 = FindPivots, Alg. 2 = BaseCase, Alg. 3 = BMSSP, Lemma 3.3 = Datenstruktur, §2 = Konstantgrad-Transformation und Ordnung ≺.

#### 6. Invarianten und Korrektheitsargument
- **Lemma 3.1 / 3.7:** Unter der Voraussetzung, dass jeder unvollständige Knoten mit d < B einen vollständigen Knoten aus S auf seinem kürzesten Pfad hat, gilt am Ende U = T_{<B'}(S), und U ist vollständig. Entweder B' = B (Erfolg) oder B' < B mit |U| = Θ(k·2^{lt}) (Teilausführung).
- **Lemma 3.2:** FindPivots liefert |P| ≤ |W|/k.
- **Lemma 3.9:** |U| ≤ 4k·2^{lt}; ist B' < B, dann |U| ≥ k·2^{lt}.
- **Beweistechnik:** Induktion über Level, „Sandwich“-Argument für die Propositionen (a) und (b) (§3.3).
- **Theorem 1.1:** deterministischer Algorithmus in O(m·log^{2/3} n).

#### 7. Komplexität (theoretisch)
- Zeit: O(m·log^{2/3} n) im Vergleichs-Additions-Modell.
- Lemma 3.12 gibt explizit C(k + 2t/k)(l+1)|U| + C(t + l·log k)·|N⁺|; die Konstante C ist nur als „große Konstante“ definiert und **nicht beziffert**.
- Rekursionstiefe ≤ (log n)/t = O(log^{1/3} n) (§3.2).
- Speicher und I/O: **von der Quelle nicht analysiert**.
- Verborgen: Konstantgrad-Transformation erzeugt O(m) Knoten.
- [Entwurf] Illustration: n = 10⁶, log₂ n ≈ 19,9 ergäbe k = 2, t = 7, L = 3 (Basis 2 angenommen; die Quelle nennt keine Basis).

#### 8. Determinismus
- Der Algorithmus ist laut Theorem 1.1 deterministisch.
- Tie-Break: totale Ordnung ≺ (§2). Medianauswahl: BFPRT (deterministisch, von der Quelle zitiert).
- d und PRED sind durch Korrektheit und ≺ eindeutig; Zwischenzustände hängen von der Kantenreihenfolge ab.
- [Entwurf] Kanten nach Ziel-ID ordnen, dichte `Vec`-Indizes statt Hash-Maps, kein RNG.

#### 9. Numerik
- Die Quelle nimmt ideale reelle Arithmetik an; in Gleitkomma sind Summen nicht assoziativ.
- [Entwurf]: Gewichte beim Einlesen auf u64-Festkomma abbilden; NaN und negative Werte ablehnen; Addition mit `checked_add` (Überlauf ⇒ Fehler); ∞ als `Option`/Flag statt Sentinel-Zahl. Bei Gleitkomma-Weiterverwendung Vergleiche über `total_cmp`.

#### 10. Fehler- und Randfälle
- P = ∅: B'_0 ← B (Quelle).
- Nicht erreichbare Knoten, parallele Kanten, Schleifen, kleine n (k oder t = 0): von der Quelle nicht behandelt. [Entwurf] Erreichbarkeits-Vorfilter, Untergrenze 1 für k und t.
- Nullgewichte erlaubt; Gleichstände regelt ≺.
- **Kein externes Budget, keine Anytime-Eigenschaft.** Die Teilausführungen sind interne Arbeitslast-Grenzen.
- Gleichzeitige Löschung: nicht behandelt. [Entwurf] Nur auf unveränderlichem Graph-Snapshot ausführen.

#### 11. Evaluation
- **Keine.** Das Paper hat keinen Experimentabschnitt: keine Hardware, kein Datensatz, keine Messung.
- Das Ergebnis ist ausschließlich **theoretisch**. Daraus folgt kein praktischer Vorteil gegenüber Dijkstra mit Binärheap.
- Die Baseline der Quelle ist Dijkstra mit Fibonacci-Heap, O(m + n log n) (§1).

#### 12. Rust-Umsetzung
- Nur Std: `Vec`, `BinaryHeap<Reverse<…>>`, `BTreeMap` (an Stelle des Rot-Schwarz-Baums). Decrease-Key über lazy deletion [Entwurf].
- Die Medianauswahl muss Worst-Case-linear sein. Ob `select_nth_unstable_by` das garantiert, habe ich **nicht geprüft**; Ersatz: eigenes BFPRT.
- Keine externen Crates.
- Panic-Stellen: Indexzugriffe (`get`), Überlauf (`checked_add`), Logarithmus von 0 oder 1, `unwrap` auf `pop`. Rekursion O(log^{1/3} n) ist unkritisch. `require B' == ∞` als `Result::Err`, nicht `assert!`.

#### 13. Testplan
- **Golden-Test** gegen Dijkstra (Ganzzahlgewichte, exakt gleich).
- **Property-Test:** Nachbedingungen aus Lemma 3.1 und 3.9 an jedem rekursiven Aufruf.
- **Fuzzing:** Nullgewichte, parallele Kanten, Schleifen, Gleichstände, n ∈ {1, 2, 3}.
- **Determinismus:** Hash über (d, PRED) in zwei Läufen und auf zwei Plattformen.

#### 14. Integrationspunkt
- Lesepfad, synchron, auf unveränderlichem Graph-Snapshot.
- Nur nach eigener Messung sinnvoll; ohne sie bleibt Dijkstra der Standard.

#### 15. Risiken, offene Fragen, Gegenbelege
- Kein Praxisbeleg. Konstanten, Gadget-Vergrößerung und kleine k, t sprechen laut [Entwurf]-Rechnung gegen einen schnellen Gewinn.
- **Lokalität:** Das Paper ruft BMSSP nur mit B = ∞ auf. Lemma 3.1 und 3.12 gelten allgemein und drücken die Kosten in |U| und |N⁺(U)| aus; ein Teilaufruf mit endlichem B erfüllt formal die Voraussetzungen, wird vom Paper aber nicht angewandt. Der Konstantgrad-Umbau ist global, außer man erzeugt die Gadgets lazy [Entwurf].
- Kein Budget, keine Speicheranalyse.
- Keine Gegenbelegsuche durchgeführt.

---

## 3. Teil 3: Verworfene Kandidaten

| Kandidat | Quelle (beide nur [A]) | Verfehltes Kriterium |
|---|---|---|
| Extended Serial Safety Net (ESSN) | Kitazawa, Ito, Yoshida, Shioi; arXiv 2511.22956 | **4 (Beleglage):** kein [V]. Der Abstract behauptet eine Lockerung der Exclusion-Bedingung, bewiesene Erhaltung von Multiversion-Serialisierbarkeit und strikte Verallgemeinerung von SSN; ungeprüft. |
| Online-konform mit automatisch abgestimmter Schrittweite | Gibbs, Candès; arXiv 2208.08401 | **4 (Beleglage):** nur Abstract. Ob die Abstimmung deterministisch ist (Kriterium 1), geht aus dem Abstract nicht hervor. |

**Einstufungshinweis zu Mechanismus 3:** Bei strenger Lesart („Lokalität und Budget müssen demonstriert sein“) verfehlt BMSSP die Kriterien 2 und 3 und gehörte in diese Tabelle. Ich habe es in Teil 2 gelassen, weil Lemma 3.12 eine explizite, in |U| ausgedrückte Schranke liefert. Die Einschränkungen stehen in Mechanismus 3, Punkt 15.

---

## 4. Teil 4: Baselines

| Baseline | Quelle | Vergleichsangabe |
|---|---|---|
| Dijkstra mit Fibonacci-/Relaxed-Heap | Q1 [V], §1 | O(m + n log n), theoretisch. Keine Messung. |
| Festes α_t = α (nicht adaptives Konform) | Q2 [V], §2.2 und §6 | Nur qualitativ in Abbildungen. Keine Zahl. |
| SI, SSI, OCC (Silo), vanilla SSN | Q3 [V], §8, Tabelle 3 | siehe Mechanismus 2, Punkt 11 |
| HNSW | **Keine belegbare Quelle gefunden (nicht recherchiert)** | – |
| BM25 mit Block-Max-WAND | **Keine belegbare Quelle gefunden (nicht recherchiert)** | – |
| Standard-RRF | **Keine belegbare Quelle gefunden (nicht recherchiert)** | – |

---

## 5. Teil 5: Quellenverzeichnis (nur geöffnete Quellen)

| ID | Titel | Erstautor | Datum | Venue | Belegstatus |
|---|---|---|---|---|---|
| Q1 | Breaking the Sorting Barrier for Directed Single-Source Shortest Paths | Ran Duan | 2025-04-23 (v2: 2025-07-30) | arXiv 2504.17033; Venue nicht belegt | [V] |
| Q2 | Adaptive Conformal Inference Under Distribution Shift | Isaac Gibbs | 2021 | NeurIPS 2021 (Oral laut neurips.cc); arXiv 2106.00170 | [V] |
| Q3 | Efficiently making (almost) any concurrency control mechanism serializable | Tianzheng Wang | 2016-05-13 nur laut Sekundärseite | arXiv 1605.04292; Venue nicht belegt | [V] Text, Datum [S] |
| Q4 | Extended Serial Safety Net: A Refined Serializability Criterion for Multiversion Concurrency Control | Atsushi Kitazawa | 2025-11-28 | arXiv 2511.22956 | [A] |
| Q5 | Conformal Inference for Online Prediction with Arbitrary Distribution Shifts | Isaac Gibbs | nicht auf geöffneter Seite | arXiv 2208.08401 | [A] |

**Autorenlisten (aus den geöffneten Seiten):**
- Q1: Ran Duan, Jiayi Mao, Xiao Mao, Xinkai Shu, Longhui Yin.
- Q2: Isaac Gibbs, Emmanuel Candès.
- Q3: Tianzheng Wang, Ryan Johnson, Alan Fekete, Ippokratis Pandis.
- Q4: Atsushi Kitazawa, Chihaya Ito, Yuta Yoshida, Takamitsu Shioi.
- Q5: Isaac Gibbs, Emmanuel Candès.

Q1–Q3 tragen die drei Mechanismen; Q4 und Q5 die zwei verworfenen Kandidaten.

---

## 6. Teil 6: Offene Punkte

### 6.1 Nicht recherchiert (nicht gleichbedeutend mit „keine Quelle gefunden“)

Mit je nur einem Suchlauf wurden die Felder A (nur Pfadsuche/SSSP), F und H behandelt. Das Auftragsminimum von zwei unterschiedlichen Suchanfragen je Feld habe ich **nirgends** erreicht. Gar nicht bearbeitet:

| Feld | Thema |
|---|---|
| A (Rest) | Personalized PageRank mit Push-Verfahren, budgetierte und diverse Pfadsuche |
| B | Rank-Fusion (RRF und Varianten), Anytime-Fusion, exakte Präfixe unter Lesebudget |
| C | ANN-Updates und Löschungen ohne Tombstones, lokale Graphreparatur, prädikatsbewusste Filtersuche, Low-Bit-Quantisierung |
| D | Block-Max-Verfahren, hierarchische und Out-of-Core-Indizes |
| E | Compaction, Schreibverstärkung, gelernte Policies |
| F (Rest) | SSI-Varianten über SSN hinaus, Lange-Transaktionen-Verfahren jenseits der Quelle |
| G | Merkle-Strukturen, Löschnachweise, Crypto-Shredding, Integritätsketten |
| H (Rest) | robuste Change-Point-Erkennung bei schweren Rändern |
| I | Bandits bei Nicht-Stationarität, Regelung unter Budgets |
| J | Streaming-Sketches, Hypergraph-Sparsifikation, Entitäts-Konsolidierung |
| K | Linearzeit-Automaten (PII), ReDoS-freie Verfahren |
| L | deterministische Reduktionsreihenfolge, ganzzahlige Distanzen, FMA-Plattformunabhängigkeit |
| M | Gitter- und Codebook-Quantisierung mit 1–4 Bit |

### 6.2 Weitere offene Punkte

1. Venue von Q1 und Q3 auf geöffneten Seiten nicht belegt (nicht aus dem Gedächtnis ergänzt).
2. Datum von Q3 nur über eine Sekundärseite.
3. Q2: Symbole in Lemma 4.1 und Proposition 4.1 waren im Textextrakt beschädigt und wurden aus der Beweislogik rekonstruiert (in Mechanismus 1 markiert). Sichtprüfung an der PDF steht aus.
4. Q1: Logarithmusbasis für k und t nicht angegeben; Behandlung kleiner n fehlt.
5. Q3: Algorithmus 3 (latch-frei) und Phantomschutz nicht spezifiziert; Repository des Simulators nicht geöffnet; Schluss von Abschnitt 9 und Abschnitt 10 im Abruf abgeschnitten.
6. Zitatlimit verhindert für Zahlen ein wörtliches Zitat je Wert (siehe §0.3). Zahlen aus Tabellen stehen mit Fundstelle.
7. Alle mit `[Entwurf]` markierten Aussagen (Rust-Hinweise, Herleitungen, Rechenbeispiel) sind Vorschläge ohne Quellenbeleg und wurden weder kompiliert noch getestet.
8. Keine Gegenbelegsuche für irgendeinen Mechanismus.
9. Q4 und Q5 nur auf Abstract-Ebene gelesen.
10. Mechanismus 3 erfüllt die Pflichtkriterien 2 und 3 nur bedingt (siehe Teil 3, Einstufungshinweis).

---

## 7. Teil 7: Selbstprüfung

| Frage | Antwort |
|---|---|
| Habe ich jede genannte Quelle geöffnet? | **Ja.** Q1–Q3 als Volltext, Q4 und Q5 nur über arXiv-Abstract-Treffer. |
| Stammen alle Autorennamen aus den geöffneten Seiten? | **Ja.** |
| Hat jede Zahl ein wörtliches Zitat? | **Nein.** Wegen des Zitatlimits (höchstens ein Zitat je Quelle) stehen Zahlen mit Fundstelle statt Zitat. |
| Sind alle Pseudocodes vollständig und alle Begriffe definiert? | **Nein.** Mechanismus 1 und 3 sind vollständig, enthalten aber markierte Rekonstruktionen; bei Mechanismus 2 fehlen die latch-freie Variante, der Phantomschutz und die Read-Mostly-Optimierung. |
| Stimmt die Anzahl der Mechanismen in Teil 1, 2 und 5 überein? | **Ja:** 3 / 3 / 3 tragende Quellen (Q1–Q3). Q4 und Q5 sind verworfene Kandidaten. |
| Enthält der Bericht keinen Widerspruch zwischen Zusammenfassung und Abschnitten? | **Ja**, nach meiner Durchsicht. Die Abweichung von 8–12 Mechanismen ist offen benannt. |
| Habe ich jede mögliche Panic-Stelle genannt, und ist sie mit „keine Panics“ vereinbar? | **Ja** für die genannten Stellen, jeweils mit `Result`, `checked_*` oder `get` als Gegenmaßnahme. Die Vollständigkeit der Liste habe ich nicht per Kompilat oder Test geprüft. |
| Ist für jede Evaluation klar, ob theoretisch oder empirisch? | **Ja.** Mechanismus 1: Beweis plus kleine Empirie; Mechanismus 2: Beweis plus Implementierungsmessung; Mechanismus 3: nur Theorie. |
