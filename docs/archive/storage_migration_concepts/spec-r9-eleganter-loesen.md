# contextra-storage R9.0: Wie löst man das eleganter?

**Lesestand:** Gelesen habe ich das Konfliktregister, §2–§6, §9.1–§9.2, §9.6–§9.8, §18 und Anhang B–D. §7, §8, §9.3–§9.5 (ER0–ER7) und §10–§17 kenne ich nur über Verweise; Vorschläge, die dort hineinreichen, sind markiert. Leistungsaussagen sind Hypothesen, wie die Spec sie selbst führt (H-24…H-28). Nichts davon ist gemessen.

## Befund

Die Leitidee ist stark und sollte bleiben: Der Cluster ist Löschdomäne, Schlüsseldomäne, Versionsdomäne und Platzierungseinheit zugleich (L1). Unelegant ist, dass die vierte Rolle, die **Platzierung**, den größten Teil der Komplexität erzeugt: Pure/Packed/Staging, Packgruppen, MiniSection, Spill-Policy, Demotion, Admission, FD-Cache (R-PL-1…8, §4.3.1, K-35…K-37). Der R9-Audit hat dafür fast durchgehend Gegenmechanismen ergänzt (R9-10 bis R9-17), statt die Ursache zu verkleinern. Dazu tragen Commit-Pipeline, Idempotenz und Ledger mehr Mechanik, als ihre Aufgabe verlangt.

## Überblick

| # | Hebel | Entfällt in v1 | Preis |
|---|---|---|---|
| 1 | Platzierungspolitik aus dem Kern nehmen; Löschung = Kryptolöschung + Compaction | Pure/Packed/Staging, Packgruppen, Spill, Demotion, `ClusterLocal`-Admission, Datei-Governor | Purge-Frist = Compaction-Zyklus; Z-5 entfällt; Plain-Cluster nur logisch löschbar |
| 2 | Ein Sequencer-Thread statt Rang-1-Sperre mit Leader-Protokoll | Intake (Rang 2), `try_lock`-Leader, Lost-Wakeup-Fall, R9-05 | eine Thread-Übergabe je Commit |
| 3 | Idempotenz-Record im Heimatcluster statt `SYS_IDEM` | Systemspace, System-DEK, TTL-Index, Trim-Marke, Pseudonym-Digest | reservierter Präfix, Ausnahme in Diff/Merge |
| 4 | Branches in v1 nur Tiefe 1 | Chain-Resolve-Allgemeinheit, `deep_ack`, H-21 für Tiefe 2–8 | kein Branch vom Branch |
| 5 | Ledger-Kennungen zurückbauen (**Option**, rechtlich zu prüfen) | `tomb_key`, Epochen-Rotation, Retire, Backup-Horizont | rohe ClusterId im Ledger |
| 6 | Norm von Geschichte und Politik trennen | Rückblicke im Fließtext, Schwellenwerte in der Norm | Umbauaufwand |

## 1. Platzierung entkoppeln (größter Hebel)

`cluster` steht in `cmp` an erster Stelle (§4.2). Ein nach `(cluster, branch, key)` sortierter LSM erzeugt zusammenhängende Cluster-Abschnitte von selbst; die Blöcke bleiben clusterrein und einzeln versiegelt (§5.2). Compaction kann Abschnitte unbeteiligter Cluster ohne Entschlüsseln kopieren und die Abschnitte gelöschter Cluster auslassen. Der Flush überspringt Retired-Cluster heute schon (§6.4).

Was darüber liegt (Dateiklassen, Packgruppen, Spill, Demotion), bedient nach meiner Lesart vor allem Z-4 (Purge ∝ betroffene Dateien) und Z-5 (Packgruppen-Ablauf in O(Dateien)). Die Spec nennt für Massen personenbezogener Cluster ohnehin „PackedBounded + Crypto-Erase“ als vorgesehenen Weg (§4.3.1, Regel 2), und `KeysDestroyed` wirkt sofort, die Byte-Hygiene darf bis zur Frist warten (K-36). Für Sealed-Cluster ist der Purge damit Garbage Collection und kein eigenes Protokoll.

**Vorschlag:**

- Ein normaler LSM je Space. Cluster sind Schlüssel- und Abschnittsgrenze, keine Dateigrenze.
- Je Datei zählt ein RAM-Zähler die Bytes gelöschter Cluster. Die Compaction-Priorität folgt dem Alter des ältesten toten Bytes gegen die Frist.
- Nachweis über den bestehenden Teil (a) von `Eraser::verify` (§9.7): keine lebende Datei trägt einen Eintrag für H.
- Für harte physische Zeitziele großer Einheiten: ein eigener Space und `space_drop` mit `Physical` (§9.8) statt `ClusterLocal`. Vorher prüfen, ob die Space-Anzahl das trägt.
- `ClusterLocal` und Packgruppen bleiben als spätere, optionale Profile denkbar. Ob sie sich rein additiv per Feature-Bit nachrüsten lassen, ist vor G-F zu klären; abschnittsrelative Offsets (R-PL-2) sprechen dafür.

**Preis:** Die Frist hängt am Compaction-Zyklus statt an „betroffene Dateien“. Wer unverschlüsselte Daten physisch löschen will, kann das nicht mehr zusagen (Plain nur `Logical`); wer physisch löschen muss, siegelt. **Prüfstelle:** ER0–ER7 habe ich nicht im Detail gelesen. Dort entscheidet sich, wie viel sich wirklich einsparen lässt.

## 2. Ein Sequencer-Thread statt Sperre und Leader-Protokoll

Der Sequencer ohne E/A ist richtig (R9-01). Seine Mechanik ist aufwendig: Anhängen an die Intake unter Rang 2, `try_lock` auf Rang 1, Leader leert die Intake, erneute Prüfung nach Freigabe gegen verlorene Wakeups (§6.1, Schritt 5), dazu der Notpool gegen Doppelhaltung (R9-05) und Race-Nahtstellen R18…R20.

**Vorschlag:** Aufrufer bereiten vor (Schritte 1–2 unverändert) und legen die Gruppe in eine beschränkte MPSC-Queue; deren Kapazität ersetzt den Credit-Semaphor. Ein Sequencer-Thread besitzt Konfliktfenster und Stamp-Zähler, ohne Sperre, und reicht per SPSC an den Syncer weiter. Die Spec hat ohnehin einen Syncer-Thread und einen `Spawner` (§3.4).

**Begründung:** Der Durchsatz ist Gruppengröße geteilt durch Sync-Latenz (§6.1, Eigenschaft 1). Flat-Combining spart eine Thread-Übergabe, die nach dieser Formel nicht der Engpass ist. Das ist eine Hypothese und gehört in den Spike (siehe unten).

**Preis:** eine Übergabe mehr je Commit (Median etwas höher, die Tail-Latenz bleibt unberührt). Der Sequencer-Thread ist ein Stall-Punkt, genau wie heute die Sperre.

## 3. Idempotenz im Heimatcluster

`SYS_IDEM` (§6.5) braucht einen instanzweiten Systemspace mit eigenem DEK, HMAC-Schlüsseln, TTL-Index, Trim-Marke, Kapazitätslimits und fail-closed-Verhalten. Außerdem muss die Schlüsselepoche bis zum Ablauf aller Tokens verfügbar bleiben, und das Ganze ist G-F-Formatbestandteil.

**Vorschlag:** Der Token-Record ist ein reservierter Schlüssel im „Heimatcluster“ des Changes (vom Aufrufer benannt, sonst die kleinste Ziel-ClusterId; bei nicht geclusterten Spaces der implizite Cluster `NONE`), im selben Redo-Frame und mit derselben `Absent`-Guard wie heute (K-38). Nur der Ort ändert sich.

**Gewinn:** Löschkorrektheit ohne Zusatzregel, denn der Record stirbt mit dem Cluster, und „nach Cluster-Retire ⇒ `Erased`“ folgt aus dem Zustand. Er ist unter dem Clusterschlüssel versiegelt, also ohne System-DEK und ohne pseudonyme Digests. Die TTL wird zu einer normalen Retention auf einem Präfix.

**Preis:** Diff und Merge müssen den reservierten Präfix ausnehmen, und die Records zählen zur Clustergröße.

## 4. Branches: Tiefe 1 in v1

Die Spec führt Default-Tiefe 4 und harte Tiefe 8 mit `deep_ack`, branch-bewusstem Filter, `BranchTableBlock`, Chain-Resolve mit Early-Exit und H-21 je Tiefe (§4.6, K-34). Sie rechnet selbst damit, die Obergrenze zu senken, falls Tiefe 8 das Ziel verfehlt, und führt das als Risiko 2 (§18).

**Vorschlag:** Forks nur vom Hauptzweig. Lesen auf einem Branch = Branch-Schicht plus `main@fork_stamp`; Merge nur Child→main (die Spec sieht ohnehin direkte Child→Parent-Merges vor). Tiefe > 1 kommt später über das vorhandene Feature-Bit `BRANCHES` (§5.3), sofern das Format das additiv erlaubt. §8 habe ich nicht gelesen, das wäre dort zu prüfen.

**Preis:** kein Branch vom Branch. Experimente auf Experimenten müssen sequenziell gemergt werden.

## 5. Ledger: weniger Kennungen (Option, keine Empfehlung ohne Rechtsprüfung)

`tomb_id` als HMAC mit `tomb_epoch`, Rotation, Retire und Backup-Horizont (K-03, K-31, §9.7, §10.4) existiert, weil `tomb_id` ein linkbares Pseudonym bleibt. Gleichzeitig ist die ClusterId reiner 128-Bit-Zufall (K-02), und §12.7 vernichtet den Directory-Eintrag Person→Cluster mit der Löschung. Nach der Löschkette ist eine rohe ClusterId dann nur noch ein Zufallswert ohne Personenbezug, sofern kein weiteres Verzeichnis die Zuordnung hält.

**Option:** ClusterId direkt im Ledger. Dann entfallen `tomb_key`, Epochen-Rotation, Retire, Horizont und die HMAC-Ableitung im Restore-Matcher.

**Risiko:** Wer Ledger und alte Kataloge besitzt, kann ohne Schlüssel verknüpfen, *welche* Cluster gelöscht wurden, nicht *wessen*. Ob das als personenbezogen gilt, ist eine Rechtsfrage, zu der die Spec ausdrücklich keine Aussage trifft. Mit dem Datenschutzbeauftragten klären, bevor G-F den Format-Slot einfriert. Der Slot selbst ist so oder so billig.

## 6. Norm von Geschichte und Politik trennen

- **Rückblicke im Fließtext.** Normative Absätze erklären, was R8.2 falsch machte (§6.3, §9.6, §9.7: „R8.2 verlangte …“). Das doppelt Anhang B und das K-Register. Die Norm sollte nur sagen, was gilt; Geschichte gehört in `DECISIONS.md`.
- **Politik in der Norm.** L19 und §3.3 versprechen „Mechanismus, nie Politik“, doch die Spec enthält Schwellenwerte (Spill 8 MiB / ×4 / 6 h, Governor ab 80 %, Quoten 1024 / 262 144). Diese gehören in ein Konfigurationsprofil.
- **Schnitt.** SPEC-CORE (Daten, Redo, MVCC, Siegel, Gate), SPEC-ERASE (Stufen, Ledger, Receipt, Orakel), SPEC-EXT (§12). Die Placement-Spec entsteht nur, falls Vorschlag 1 nur teilweise greift. Jede Datei bekommt ein eigenes Gate; das senkt das Risiko 9 (Zeitplan 1,5–2× zu knapp).

## Was ich behalten würde

- Die Cluster-Invariante (L1) und die Reihenfolge Gate < Schlüssel < Purge (L7).
- Die eine Siegelform ohne Datei- und Positionsbezug (K-27): Sie macht die Rohkopie gültig und trägt Vorschlag 1.
- Redo plus Catalog statt CAS-Superblock (K-10) und den Catalog als einzige Wahrheit (L6).
- Die zweistufigen Guards (§6.3): Stufe A beim Aufrufer, Stufe B nur Delta im Sequencer.
- Anker und Rollback-Schutz (AT-5).

## Nächster Schritt: dünner Spike vor weiteren Revisionen

Die Spec ist nach neun Revisionen auf Papier konsolidiert, aber „keine Leistungszahl ist als gemessen anzusehen“. Ein vertikaler Schnitt aus Redo, Memtable, Siegel, Gate und Gruppen-Fsync, ohne jede Platzierungspolitik, würde drei Fragen beantworten:

1. Ist der Commit-Pfad wirklich fsync-gebunden (Vorschlag 2)?
2. Wie lange braucht Compaction als Purge für N tote Cluster (Vorschlag 1)?
3. Wie teuer ist Branch-Read bei Tiefe 1 gegenüber Tiefe 4 (Vorschlag 4)?

Danach lässt sich entscheiden, welche Mechanik der Formatfreeze G-F wirklich braucht.
