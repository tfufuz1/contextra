# SP-VERIFY — Test-Orakel, Crash-Sweeps, Referenzformate, formale Modelle

| | |
|---|---|
| Modul | `testing/`, `tools/reference/`, `formal/`, Storage CI-Skripte |
| Besitzt | unabhängige Modell-/Dateiprüfer, FaultFs-Sweeps, golden vectors, Loom/Sim-Modelle, Gates |
| Konsumiert | alle SPs als Testsubjekte; darf Produktionslogik nicht implementieren |
| Spec-Bezug | R8 §13–15, §17; INV-* |
| Phase | P0–P8; Gate-Besitzer übergreifend |

## 1 Auftrag
Jede normative Invariante wird durch mindestens einen test benannt und unabhängig geprüft. Referenzencoder ist nicht Rust-Implementierung; kanonische Bytes werden ab G-F eingefroren. Crash-Sweep wird beim Hinzufügen eines Schreibpfads erweitert.

## 2 Verträge
`FaultFs` exponiert deterministische Crashindizes je IoCtx; `Inspector` liest nur Testzustand; `Oracle` ist unabhängiges Referenzmodell. Generierte Testdaten enthalten keine persistenten Klartext-Hashes. `formal/` dokumentiert M1–M6 Zustände, Aktionen, Fairnessannahmen und Zuordnung zu INV-IDs. Golden Vectors prüfen kanonische `SpaceId u64`-Bytes, dauerhafte ID-Reservierung, domain-separierte Cluster-/Space-Tomb-HMACs, Request-ID/Digest, Ledger-Statussemantik und Erase-Reihenfolge. Restore-Tests prüfen dieselbe `InstanceId`, externe Fencing-Voraussetzung, Quarantäne vor Gate-Reconcile sowie fehlende Ledger-/Scope-Key-/Anchor-Fälle. Extension-Tests erzwingen Gate-Filter vor Scoring/Traversal und unmittelbar vor Rückgabe.

## 3 Gate-Matrix
G-0 API/Ports, forbidden APIs, layering; WS: durchgehender Crash-Sweep; G-F: Referenzvektoren und Formatfreeze nach P2; G-W/G-R/G-M/G-E/G-V/G-P gemäß INDEX. Keine Gate-Abnahme allein durch Kompilierung.

## 4 Abnahmebedingungen
Decoder: Fuzz + Rundreise + Golden; Persistenz: FaultFs Sweep; Concurrency: Loom/Sim; Invarianten: ID-Referenzen im Test; Performance: Hypothese mit reproduzierbarem Messbericht. Receipt-Oracle akzeptiert `Complete` nur bei allen erforderlichen Attesten; `CompleteWithResidual` nur bei explizit erlaubter, benannter Restexposition; fehlender erforderlicher Owner-/Scope-Beleg ist `Incomplete`; retrybare Blockade ist `Blocked`; Integritäts-/Protokollfehler ist `Failed`. Keine CI-Regel darf schwächer als normatives Gate sein.

## 5 Arbeitspakete
WP-15.1 Forbidden API/layering (P0) · WP-15.2 FaultFs/Crash harness (P0) · WP-15.3 Modellorakel + Invariant-Mapping (P1) · WP-15.4 Referenzgenerator/goldens (vor G-F) · WP-15.5 TLA+ M1–M6 (laufend) · WP-15.6 Performance/coverage/mutation gates (P8). Abnahme: Gate-Evidenz archiviert und reproduzierbar.

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §13.2.1, §15, §17 (H-4, H-23); neue IDs `INV-ERA-13`, `INV-LED-2`, `INV-KEY-5`, `INV-VIEW-3`, `INV-PRUNE-1`, `INV-VER-1`.

- **Invarianten-Mapping:** die sechs neuen IDs werden in `check_invariant_refs.sh` erst grün, wenn die Besitzer-SPs behaviorale Tests zitieren (WP-13.7, WP-20.4, WP-04.7, WP-11.6, WP-13.8). Lexikalische Platzhalter-Tests zählen nicht als Abnahme (TRACE-Befund zu `invariant_refs.rs`).
- **Orakel:** Messarten je Orakel (SPEC §13.2.1); CI-Prüfung „kein `Complete` bei Pflichtmessung ≠ `Complete`“.
- **Modelltest:** Poisoned-Erase-Modell (Intent → Neustart → Gate), Tomb-Epochen-Modell (Retire/Restore-Floor), Lease-Modell (Ablauf ⇒ `SnapshotTooOld`).
- **H-23:** Restore-Matching-Benchmark (16 Mio. Cluster × 1/4 Epochen).

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-15.10 | R8.2-Testlücken: Orakel-Messarten, Modelle (Poisoned/Tomb/Lease), Pruner-Property, H-23 | 2.0 | WP-15.3, WP-13.7, WP-20.4, WP-11.6 | P6 | neue INV-IDs von Verhaltenstests zitiert, H-23-Bericht |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

Die Referenzmodelle und Simulation decken `RACE-1` bis `RACE-22` ab, insbesondere Intake/Leader-Wechsel, Pipeline-Credits, Guard-Fenster, Link-, Gate- und Space-Drop-Races. G-F enthält den Branch-Filter-/`BranchTableBlock`-Spike sowie Admission-/MiniSection-Vektoren. H-24 bis H-28 bleiben Hypothesen: sie dürfen nur mit reproduzierbarer Evidenz als erfüllt markiert werden.
