# Contextra: Strategische Architektur- und Risikoberatung

|  |  |
| --- | --- |
| Repository | `tfufuz1/contextra`, Commit `2850fa5f3` |
| Stand der Prüfung | 24.11.2026 |
| Methode | Statische Analyse: Klon, Lesen der Spezifikationen, gezielte Prüfung der Aussagen am Quellcode mit `grep`, `find`, `wc` und Auswertung der `Cargo.toml` |
| Status | **Nicht gebaut, nicht getestet.** Rust ist im Analyseumfeld nicht installiert. Der Phasenstatus belegt Tests nur mit Rust 1.85.1 und `--ignore-rust-version`, nicht auf dem Pin 1.89.0. |
| Geltungsbereich | Speicherkern, Löschkette, Schlüsselmodell, Crate-Topologie, Migration, Produktversprechen (README) |
| Rechtlicher Hinweis | Technische Bewertung. Aussagen zu Art. 17 DSGVO, Fristen, Aufbewahrungspflichten und Rechenschaftspflichten gehören in eine externe rechtliche Prüfung, die die Spezifikation in R10 §18 ohnehin verlangt. |

---

## 1. Kernbotschaft

Die Architekturrichtung ist richtig: ein neuer Speicherkern mit klaren Ringen, ein einziger Löschorchestrator, ein AEAD-Standard und die Trennung von Mechanismus und Compliance. Die Probleme liegen nicht primär in der Struktur, sondern in drei Lücken:

1. **Das Produktversprechen liegt über dem, was die eigene Spezifikation zulässt.** Die README verspricht, dass Löschung „sofort und unwiederbringlich“ wirkt, auch in Sicherungen und Suchindizes. Die normative Kern-Spezifikation deckelt jeden Löschbeleg mit dem Standard-Schlüsselanbieter `keys-file` auf `CompleteWithResidual{WeakKeyAssurance}`. `Complete` ist dann ausgeschlossen.
2. **Der Löschpfad ist im Produkt nicht verdrahtet.** Es gibt mindestens vier parallele Löschpfade. Die Engine, die laut Spezifikation den Löschaufruf und die Zuordnung Person zu Cluster übernehmen soll, enthält keinerlei Löschlogik. Kein anderes Crate hängt von `contextra-erasure` ab.
3. **Der Zeitplan steht gegen die Architektur.** Der neue Kern hat null Konsumenten, die Phasen P6 bis P8 sind offen, und das Altsystem `contextra-store` mit 214 Verweisen bleibt produktiver Pfad. Die Übergangsausnahmen in `deny.toml` laufen am **30.04.2027** aus, also in rund fünf Monaten.

Hinzu kommt, dass die R11-Befundbasis an mehreren Stellen nicht mit dem Repository übereinstimmt. Entscheidungen, die auf diesen Befunden aufbauen, sollten neu bewertet werden.

**Empfehlung in einem Satz:** Das Produktversprechen auf die tatsächlich erreichbare Stufe zurückführen, Hardware- oder KMS-gebundene Schlüssel als Voraussetzung für Personendaten festlegen, den Löschpfad auf genau einen Orchestrator reduzieren und den Standardpfad für Personendaten sperren, bis die Schlüsselzusicherung vorhanden ist.

### Befundübersicht

| # | Befund | Schwere |
| --- | --- | --- |
| B-1 | Löschversprechen (README) übersteigt die Spezifikation (SPEC.md) | Kritisch |
| B-2 | Vier parallele Löschpfade, Engine ohne Löschlogik, Erasure-Crate ohne Konsumenten | Kritisch |
| B-3 | Zeitplan: Kern unverbraucht, Altsystem produktiv, Ausnahmen laufen am 30.04.2027 aus | Kritisch |
| B-4 | Standard-Schlüsselanbieter kann kein `Complete` tragen; kein Hardware- oder KMS-Adapter vorhanden | Kritisch |
| B-5 | Hash-Verkettung des Audit-Logs kollidiert mit der Löschung pro Cluster (zu klären) | Hoch |
| B-6 | Fehler in der R11-Befundbasis (vier belegte Abweichungen) | Hoch |
| B-7 | Lücken: Identity-Mapping, Multi-Subject-Dokumente, Derivate, Backups, Schlüssel-Lebenszyklus | Hoch |
| B-8 | Rust: Kopierbare Schlüsselzugriffe, Feature-Unification, Dev-Dependency-Leck, zwei Provider-Typen | Hoch |
| B-9 | Nebenläufigkeit Gate gegen Commit ist ungeklärt; formale Modelle fehlen | Hoch |
| B-10 | Windows-Durability ausdrücklich nicht versprochen, Zielgruppe Mittelstand | Mittel |

Schwere: **Kritisch** = blockiert eine Produktzusage oder die Compliance-Aussage. **Hoch** = Architektur- oder Risikolücke mit hohen Folgekosten. **Mittel** = muss vor Produktivbetrieb geklärt sein.

---

## 2. Prüfumfang

**Gelesen vollständig:** `CLAUDE.md`, `docs/SPEC-OEKOSYSTEM-R11.md` (528 Zeilen), `README.md` (Abschnitte 1, 2, 4 und 5), `Cargo.toml`, `deny.toml`, `rust-toolchain.toml`, `crates/contextra-storage/crates/contextra-storage-ports/src/keys.rs`, `crates/contextra-storage/crates/contextra-storage/src/erase/protocol.rs` (Kopf), `…/erase/receipt.rs` (Statuslogik).

**Gezielt gelesen:** `crates/contextra-storage/SPEC.md` (über `grep` auf Schlüsselbegriffe), `crates/contextra-storage/docs/PHASE-STATUS.md`, `docs/storage_migration_concepts/SPEC-ECOSYSTEM-R10.md` (über `grep`), `tests/layering.rs` und `tests/unsafe_islands.rs` (Kopf).

**Nur per `grep`/`find` erfasst:** `contextra-engine`, `contextra-db`, `contextra-mcp`, `contextra-store`, `contextra-vector`, `contextra-text`, `contextra-graph`, `contextra-kvcache`, `contextra-cognition`, `contextra-erasure`.

**Nicht geprüft:** Build und Tests, Laufzeitverhalten, Benchmark-Ergebnisse, `contextra-privacy`, `contextra-ingest` im Detail, die Logik der Owner-Crates.

**Hinweis zur Repo-Datei `CLAUDE.md`:** Die Datei enthält Anweisungen, unter anderem zu Commits, Pull-Requests und Pushes auf `main`, sowie zur Nutzung von Subagenten. Für diese Beratung liegt kein Änderungsauftrag vor. Deshalb wurde nichts committet, gepusht oder verändert. Ob und wie solche Repo-Anweisungen gelten, sollte das Team festlegen.

---

## 3. Korrekturen zur R11-Befundbasis

Die R11 ist ein sorgfältig gebautes Dokument, enthält aber nach der Prüfung am Code mehrere falsche Aussagen. Einige davon begründen Entscheidungen direkt.

| R11-ID | Behauptung der R11 | Befund am Code | Konsequenz |
| --- | --- | --- | --- |
| F-01 | Kern ist kein Workspace-Mitglied; `version.workspace = true` löst nicht auf | **Falsch.** Alle sieben Speicher-Crates stehen in `members` (`Cargo.toml`, Zeilen 2 bis 8). Das Workspace-Paket ist definiert (`[workspace.package]`, Zeile 91). | M0 als „Workspace-Reparatur“ entfällt in dieser Form. Der Befund sollte gegen den Stand geprüft werden, auf dem die R11 geschrieben wurde. |
| F-10 / R-10 | `CLAUDE.md` ist leer (0 Byte) | **Falsch.** 2.896 Byte, mit Projektvorgaben. | R-10 und die Sofortmaßnahme 2 sind gegenstandslos. Der Inhalt von `CLAUDE.md` muss gegen die R11 abgeglichen werden. |
| F-09 | `PHASE-STATUS.md` liegt unter `docs/` | **Falscher Pfad.** Die Datei liegt unter `crates/contextra-storage/docs/PHASE-STATUS.md`. | Kleinerer Fehler, aber er zeigt, dass die Befundbasis nicht vollständig gegengeprüft wurde. |
| Zählung | „36 Einträge, kein Storage-Eintrag“ in `members` | **Ungenau.** `members` hat 43 Einträge, davon 7 Speicher-Crates und 36 übrige. | Die 36 gelten für die übrigen Crates, nicht für das Gesamt-Workspace. |
| F-08 | `contextra_erasure` hat 44 Verweise über `crates/` | **Irreführend.** Alle 44 Dateien liegen im Erasure-Crate selbst. Kein anderes Crate hat eine Cargo-Abhängigkeit auf `contextra-erasure`. | Die Löschschicht ist nicht verdrahtet. Das ist strenger als die R11 formuliert (siehe B-2). |
| §3.2 | Kern darf nicht von `blake3` abhängen | **Verletzt.** Der Kern hat eine Dev-Dependency auf `contextra-crypto-ref`, und das zieht `blake3` transitiv herein. AG-1 gilt laut R11 ausdrücklich auch für Dev-Dependencies. | Eigene Regelverletzung der Spezifikation (siehe Abschnitt 6.4). |
| F-02, F-03, F-07, F-11 | `thiserror 1`, 11 `Clock`-Definitionen, zwei AEAD-Stacks, Ausnahmen bis 2027-04-30 | **Bestätigt.** | Diese Befunde tragen die Architekturentscheidungen. |

**Schlussfolgerung zur Befundbasis:** Die strukturellen Befunde zu Dubletten und Abhängigkeiten stimmen. Die Befunde zum Workspace-Status und zur Dokumentation stimmen nicht. Vor Abnahme der R11 sollte die gesamte Befundtabelle mit einem Skript neu erzeugt werden. Die Spezifikation verlangt das für AG-2 ohnehin.

---

## 4. Widersprüche zwischen Versprechen, Spezifikation und Code

Das ist der wichtigste Abschnitt für die Produktstrategie. Die README ist das Dokument, das Kunden lesen. Die SPEC.md und die R11 sind das Dokument, nach dem gebaut wird. Beide widersprechen sich.

| Thema | README (Versprechen) | Normative Spezifikation | Code-Stand | Bewertung |
| --- | --- | --- | --- | --- |
| **Vollständigkeit der Löschung** | „Wird der Schlüssel vernichtet, sind alle zugehörigen Daten sofort und unwiederbringlich unlesbar: in der Datenbank, in den Suchindizes und in den Sicherungen.“ | Löschbeleg mit `keys-file` nur `CompleteWithResidual{WeakKeyAssurance}`. Das Receipt darf nie „unwiderruflich“ oder „konform“ sagen (SPEC.md, Abschnitt 9). Für Backups gilt: Bei `Plain` fehlt der Schutz (Abschnitt 10). | Umgesetzt in `erase/receipt.rs` (`WeakKeyAssurance`). | **Kritisch.** Die Marketing-Aussage ist mit dem Standardpfad nicht einlösbar. |
| **Suchindizes ohne Neuaufbau** | „Ein langwieriges Neuordnen ist dafür nicht nötig.“ | „Externe globale Rebuildable-Indizes haben eigene Frist und Kosten.“ „Ein globaler HNSW-/CSR-Neubau ist kein O(Cluster)-Erase-Versprechen.“ | Nicht implementiert. | **Kritisch.** Der Architekturvorschlag (Abschnitt 7.4) löst den Widerspruch. |
| **Löschnachweis extern prüfbar** | „Jede Löschung lässt sich extern überprüfbar belegen.“ | Signaturschlüssel in Produktion nur Hardware-gestützt. Datei-Seed nur für Entwicklung (O-5). | Signaturpfad (`ed25519_proof.rs`) vorhanden. Hardware-Anbindung fehlt. | **Hoch.** Ohne Hardware-Signer ist die Aussage für Produktion nicht belegt. |
| **Geheimnisse nach Gebrauch überschrieben** | „Schlüssel und Passwörter … nach Gebrauch zuverlässig überschrieben.“ | NZ-5: Schutz von Prozessspeicher, Swap und Core-Dateien ist **ausdrücklich nicht Ziel**. | `Zeroizing` im Port, `MemPin` als best effort. | **Hoch.** Das Versprechen ist zu weit. Richtig wäre „beim Verlassen des Speichers überschrieben, sofern der Prozessspeicher nicht ausgelagert wird“. |
| **Externe Modelle** | „Das externe Modell sieht die echten Daten nie.“ | Obere Schichten wie Prompt-Caches, Embedding-Kopien, Logs und Provider-Logs sind „nicht im Kern lösbar“ (SPEC.md, Abschnitt 13). | Nicht implementiert. | **Hoch.** Die Löschgrenze an der Systemgrenze muss im Produkt benannt werden. |
| **Keine verlorenen Daten** | „Keine verlorenen Daten.“ | Der Ledger-Schlüssel „muss für Restore verfügbar bleiben“ und liegt außerhalb der Datenbackups. Schlüsselverlust führt zu `KeyLost`. Schlüssel-Lebenszyklus ist offen (O-5, O-6). | Ereignistyp `KeyLost` vorhanden. | **Hoch.** Schlüsselverlust und Löschpflicht stehen im Zielkonflikt, siehe 5.1. |
| **Ein Löschorchestrator** | Nicht im Produkt beschrieben. | E-06 und L-1: genau ein Orchestrator, der Kern-`Eraser`. | Vier Pfade (siehe 5.2). | **Kritisch.** Architekturvorgabe ist nicht umgesetzt. |
| **Produktiver Pfad** | Nicht beschrieben. | Kein Dual-Write. Migration per Export und Import. | `contextra-store` (LSM, 214 Verweise) ist produktiv. P8 nicht begonnen. | **Kritisch.** Das Produkt läuft auf dem Pfad, der laut CLAUDE.md an der Löschung scheiterte. |
| **Verbot von Testcode im Produktbinary** | Nicht beschrieben. | `open_test` darf nie in Produktionsbinaries auftauchen, gesichert über `deny.toml` (AG-6). | `deny.toml` prüft Abhängigkeiten, nicht aktivierte Features. | **Mittel.** Sicherung greift nur bei Abhängigkeiten, nicht bei Features (siehe 6.3). |
| **Relevanzwerte kalibriert** | „‚80 % relevant‘ entspricht tatsächlich ungefähr 80 %.“ | Nicht Teil der Kern-Spec. | Benchmarks vorhanden (`benchmarks/results/`), Kalibrierung nicht geprüft. | **Mittel.** Ohne Kalibrierungsbericht (ECE) als Marketingaussage belegen oder streichen. |

---

## 5. Blinde Flecken

### 5.1 Schlüssel-Lebenszyklus als ungelöster Zielkonflikt (kritisch)

Crypto-Shredding funktioniert nur, wenn der Schlüssel nur an einer Stelle existiert und sich zuverlässig vernichten lässt. Gleichzeitig verlangt der Betrieb Sicherungen, Wiederherstellung und Schlüsselerhalt. Diese beiden Forderungen stehen im Widerspruch, und die Spezifikation löst ihn nur über Flags (`KeysInBackups`, `KeyAssurance`). Ein Betriebsprozess fehlt:

- Erzeugung und Verwahrung des Root-Schlüssels (Key Ceremony),
- Escrow für Notfälle mit begrenzter Lebensdauer (`Bounded{until}`),
- Rotation von KEK und DEK,
- Verhalten bei Verlust des Schlüsselspeichers, mit klarer Aussage, welche Daten dann verloren sind.

Ohne diesen Prozess ist jede Löschaussage theoretisch, denn der Betreiber entscheidet im Notfall, ob Schlüssel bewahrt oder vernichtet werden.

### 5.2 Vier Löschpfade statt eines Orchestrators (kritisch)

Die Spezifikation fordert genau einen Orchestrator. Der Code enthält mindestens vier:

| Pfad | Ort | Umfang | Status |
| --- | --- | --- | --- |
| Altsystem | `contextra-store` (LSM) | 10 Dateien mit Löschbezug | Eingefroren laut R11, aber produktiv |
| Erasure-Orchestrator | `contextra-erasure` (`orchestrator.rs`, `saga.rs`, `gate.rs`, `ledger.rs`, `purge.rs` u. a.) | 128 Dateien, keine Konsumenten | Parallel zum Kern |
| Kern-Protokoll | `contextra-storage/src/erase/protocol.rs` (`EraseCoordinator`) | Zustandsmaschine mit Tickets | Ohne Konsumenten außerhalb des Moduls |
| Owner-Deletes | `contextra-db/src/transaction/db_transaction.rs` (`stage_text_delete`, `stage_graph_edge_delete` u. a.) | Löschung je Index | Verteilt, ohne gemeinsamen Gate-Schritt |

Die Engine enthält keinen einzigen Löschbezug, obwohl sie laut R11 Identity-Owner und Löschaufrufer sein soll. Damit existiert die Kette aus R11 §6 heute nur auf dem Papier.

**Konsequenz:** Ein Löschauftrag kann in einem Pfad vollständig ausgeführt werden und in einem anderen unvollständig bleiben, ohne dass das System es bemerkt. Genau diese Art von Inkonsistenz ist der Kern des Scheiterns, das CLAUDE.md beschreibt.

### 5.3 Zuordnung Person zu Cluster ist nicht implementiert (kritisch)

Die Löschung „nach Person“ setzt voraus, dass jede Person auf die Menge ihrer Cluster abgebildet wird. Die R11 legt diese Abbildung in den Identity-Owner der Engine, und die Engine ist leer. Ohne diese Abbildung gibt es keine Löschung auf Anfrage einer betroffenen Person.

### 5.4 Multi-Subject-Dokumente (hoch)

Ein Vertrag nennt zwei Personen, eine E-Mail hat drei Empfänger, ein Gesprächsprotokoll enthält Aussagen mehrerer Beteiligter. In welchem Cluster liegt das Dokument? Die Spezifikation sagt nur, dass Verknüpfungen (`link`) „nur bei gewollter Kaskade“ wirken (K-25). Es fehlt die Regel für den Normalfall:

- Entweder das Dokument gehört zu einer Löschdomäne, dann bleibt es bei Löschung der anderen Person erhalten. Das ist vermutlich rechtlich nicht vertretbar, wenn die Person nicht mehr betroffen sein soll.
- Oder es wird pro Beteiligtem dupliziert oder geschwärzt. Das erhöht Speicher und Komplexität.
- Oder es wird als gemeinsame Löschdomäne behandelt, die erst gelöscht wird, wenn alle Beteiligten es verlangt haben.

Diese Entscheidung ist fachlich und rechtlich zu treffen, nicht nur technisch.

### 5.5 Derivate und Verdichtungen (hoch)

Die README verspricht, dass Gespräche verdichtet werden und „Rohdaten archiviert“ werden. Verdichtete Zusammenfassungen, Embeddings, Wissenseinheiten, Kanten im Wissensgraphen und Caches sind abgeleitete Daten aus personenbezogenen Quellen. Die Spezifikation behandelt sie mit `Rebuildable` und `ClusterSections`, aber die Kaskade vom Quelldatum zum Derivat ist nicht definiert:

- Wird ein Derivat automatisch gelöscht, wenn seine Quelle gelöscht wird?
- Gilt das auch für Zusammenfassungen, die aus mehreren Quellen entstanden sind?
- Wie wird ein Derivat erkannt, das nach einer Löschung neu entsteht, etwa durch erneutes Einlesen aus einer Kopie?

Ohne Antworten entstehen Derivate, die ein Löschbeleg nicht erfasst.

### 5.6 Hash-Verkettung gegen Löschung (hoch, zu klären)

Die R11 (§8) legt Audit-Einträge in den Cluster der betroffenen Person, mit Hash-Verkettung über Kern-Stamps. Wird ein Cluster kryptografisch vernichtet, verschwinden auch seine Audit-Einträge. Wenn die Kette über Cluster hinweg verläuft, bricht sie für alle anderen Cluster. Das System würde dann seine eigene Integrität verlieren.

**Lösungsskizze:** Eine Kette je Löschdomäne. Darüber eine Merkle-Struktur über die Kopfwerte der Ketten. Ein gelöschter Cluster hinterlässt nur seinen Kopfwert als Hash, ohne Personenbezug. Dann bleibt die Gesamtintegrität erhalten, und die Löschung bleibt nachweisbar.

### 5.7 Pflicht zur Rechenschaft gegen Löschung (hoch, rechtlich zu klären)

Protokolle, die Löschungen belegen, müssen selbst aufbewahrt werden, sonst lässt sich die Löschung nicht nachweisen. Gleichzeitig sollen Löschungen vollständig sein. Die Spezifikation trennt Erasure-Belege (Tomb-IDs als HMAC, ohne Klartext) sauber von Nutzdaten. Das ist eine gute Lösung. Ob Belege ohne Personenbezug überhaupt Personenbezug haben, ist eine rechtliche Frage und gehört in die externe Prüfung.

### 5.8 Backup-Horizont ist Betreiberpflicht ohne Werkzeug (hoch)

Der Parameter `backup_horizon` ist Pflicht, sobald Retire aktiv ist. Die Spezifikation erzwingt aber nicht, dass Backups mit abgelaufenem Horizont tatsächlich vernichtet werden. Sie verhindert nur, dass ein Restore sie wieder einspielt (`BackupBeyondHorizon`). Der Betreiber muss also selbst Backups löschen, und dafür fehlt Werkzeug. Für Mittelständler mit täglichen Sicherungen auf Band oder Cloud-Speicher ist das der realistische Normalfall.

### 5.9 Ingest, Logs und Telemetrie (mittel)

Beim Einlesen entstehen Zwischenstände: Warteschlangen, temporäre Dateien, Dokumenttext im Speicher. Außerdem nutzt das Projekt das Crate `tracing`. Spans und Log-Zeilen können Abfragetext oder Nutzdaten enthalten. Die Spezifikation erwähnt Logs nur als externe Kopien. Es fehlt eine Regel, dass Logs keine Nutzdaten enthalten dürfen, und ein Test, der das prüft. Die `canary_no_leak`-Tests im Kern decken nur den Speicherkern ab.

### 5.10 Windows (mittel, strategisch)

Die Spezifikation sagt ausdrücklich, dass Windows-Durability kein implizites Versprechen ist und dass Produktionsfreigabe bestandene Fs-Konformanztests auf Windows erfordert (NZ-8). Gleichzeitig ist die Zielgruppe der Mittelstand, in dem Windows-Server verbreitet sind. Diese Spannung muss entschieden werden: entweder Linux-first für Personendaten oder Windows als eigener Meilenstein mit Fsync-Konformanz.

### 5.11 Externe Prüfung und Zeitbudget (mittel)

R10 §18 verlangt externe Prüfung vor jeder Freigabe für Personendaten. Die R11 nennt keinen Zeitpunkt, kein Budget und keinen Prüfer. Dieser Engpass steht am Ende des Pfades und ist deshalb der wahrscheinlichste Grund für Verzögerungen.

### 5.12 Nicht gedeckte Grenzen des Löschversprechens (hoch)

Die Spezifikation listet die Restrisiken in §9 und Anhang (`Declared`-Ausweisung): Dateisystem-Journale, Flash-Wear-Leveling, Volume-Snapshots, externe Backups, Hardlinks, Provider-Logs. Das Produkt darf diese Grenzen nicht nur im Receipt nennen, sondern muss sie dem Kunden vor Vertragsabschluss klar benennen.

---

## 6. Rust-spezifische Befunde

### 6.1 Schlüsselmaterial kann unbemerkt kopiert werden (hoch)

Die Ports-Struktur ist sauber aufgebaut: `Key32` verwendet `Zeroizing`, das `Debug`-Format ist redigiert, und `Clone` ist nicht implementiert. Zwei Stellen unterlaufen das aber:

- `Key32::from_array(raw: [u8; 32])` nimmt das Array per Wert. Die Quelle bleibt als Kopie auf dem Stack liegen und wird nicht gelöscht.
- `Key32::expose(&self) -> &[u8; 32]` gibt eine Referenz auf ein Array zurück. `[u8; 32]` implementiert `Copy`. Ein Aufrufer kann mit `let k = *key.expose();` eine ungeschützte Kopie erzeugen, die `Zeroizing` nie löscht.

`Zeroizing` schützt nur die eine Speicherstelle, die es besitzt. Der Compiler kann zusätzlich Zwischenkopien in Registern und auf dem Stack erzeugen. Die Maßnahme ist daher Schadensbegrenzung, kein Nachweis. Trotzdem ist die API der richtige Ort, Kopien zu erschweren.

**Empfehlung:** Den Zugriff auf einen Gültigkeitsbereich begrenzen und die Quelle beim Übernehmen löschen (Vorschlag in 7.6).

### 6.2 `panic = "abort"` umgeht Drop-basiertes Löschen (hoch, zu prüfen)

Das Profil `release-abort` setzt `panic = "abort"`. Dann laufen keine Destruktoren, also auch kein `Zeroize`. `Zeroizing` ist daher nur dort wirksam, wo `unwind` aktiv ist. Zu prüfen ist:

- Wird `release-abort` für Prozesse eingesetzt, die Schlüssel halten?
- Gibt es einen Pfad, der Schlüssel vor einem Abbruch explizit überschreibt?

Die Antwort sollte im Betriebskonzept festgehalten werden. Für schlüsselhaltende Prozesse empfiehlt sich `unwind` oder ein explizites Löschen in einer Abbruchbehandlung, soweit das Betriebssystem es zulässt.

### 6.3 Feature-Unification kann das Testfeature in Produktbinaries bringen (mittel)

Die Spezifikation verlangt, dass `open_test` nie in Produktionsbinaries erscheint. Die Prüfung läuft über `deny.toml`, und die prüft Abhängigkeiten, nicht aktivierte Features. Mit Resolver 2 werden Features über alle Pakete eines Build-Aufrufs vereinheitlicht. Dev-Dependencies aktivieren ihre Features nur, wenn Tests gebaut werden. Ein Aufruf wie `cargo test --workspace` oder `--all-targets` kann deshalb das Feature `testing` des Kerns auch für andere Ziele aktivieren, die im selben Aufruf gebaut werden.

**Empfehlung:** Ein CI-Job prüft den Feature-Baum des Produktbinaries getrennt, ohne Dev-Dependencies:

```bash
# Feature-Baum des Produktbinaries ohne Dev-Dependencies; darf kein "testing" enthalten
cargo tree -p contextra --no-dev-dependencies -e features -i contextra-storage-adapters \
  | grep -q 'feature "testing"' && { echo "testing-Feature im Produktbaum"; exit 1; }
```

Die Syntax ist vor dem Einsatz mit der konkreten Cargo-Version zu prüfen.

### 6.4 Dev-Dependency verletzt die Ringregel des Kerns (mittel)

Der Kern (`contextra-storage`) hat in `[dev-dependencies]` die Abhängigkeit `contextra-crypto-ref` mit Feature `testing`. Dieses Crate zieht `blake3`, `chacha20poly1305`, `hkdf` und `sha2` herein. Die R11 verbietet dem Kern genau diese Abhängigkeiten, und AG-1 erfasst ausdrücklich Dev-Dependencies. Die Verletzung lässt sich mit einem einfachen Test finden.

**Empfehlung:** Kern-Tests, die Krypto brauchen, in `contextra-storage-testkit` verlagern. Dort ist die Abhängigkeit laut Spezifikation erlaubt. Den Layering-Test auf Dev-Dependencies erweitern.

### 6.5 Zwei Repräsentationen derselben Schlüsselzusicherung (hoch)

Das Konzept „Sicherungen enthalten Schlüssel“ existiert zweimal, mit unterschiedlichen Typen:

- Im Port (`contextra-storage-ports/src/keys.rs`): `KeyProviderInfo { destroy, keys_in_backups: bool }`.
- Im Receipt (`erase/receipt.rs`, Zeile 41): `keys_in_backups: Option<u64>` mit Ablaufzeit. Das Feld `weak_key_assurance: bool` existiert nur dort.

Das verletzt A-1 der R11 (ein Begriff, eine Definition). Die Spezifikation verlangt außerdem einen typisierten Wert `KeyAssurance` (Hardware, KMS, FilePassphrase), der im Code nicht existiert. Stattdessen entscheidet ein Bool, ob ein Receipt gedeckelt wird. Dieser Bool wird vom Adapter gesetzt. Die Vertrauensgrenze für Löschbelege liegt also in einer Eingabe, die nicht typgeprüft ist.

**Empfehlung:** Einen einzigen Typ `KeyAssurance` als Enum mit Konstruktoren definieren, der nur von Adaptern mit bestandenem Konformanztest erzeugt werden kann (Vorschlag in 7.5).

### 6.6 Kanonische Kodierung mit `unwrap_or(0)` (niedrig bis mittel)

Die Receipts sind signiert, deshalb muss ihre kanonische Kodierung eindeutig sein. `keys_in_backups.unwrap_or(0)` kodiert `None` und `Some(0)` gleich. Bei heutiger Semantik ist das harmlos, weil beide keine Sicherung mit Schlüssel bedeuten. Bei einer späteren Änderung der Semantik wäre es ein Fehler, der Belege verfälscht.

**Empfehlung:** Ein Tag-Byte je optionalem Feld. Versionierung der kanonischen Form im Receipt festhalten. Golden-Vektoren für beide Fälle aufnehmen.

### 6.7 Nebenläufigkeit: Gate gegen laufenden Commit (hoch)

Die Löschung setzt ein Gate. Danach dürfen keine Commits mehr auf die betroffenen Cluster schreiben. Laut PHASE-STATUS fehlen die „Guard-Fenster“ und die „Admission/Reservierungen“. Solange das offen ist, kann ein Commit, der vor dem Gate beginnt und nach dem Gate endet, Daten in einen Cluster schreiben, dessen Schlüssel gleich vernichtet wird. Dann entstehen Daten, die nicht im Löschbeleg erscheinen.

Die Spezifikation beschreibt die Reihenfolge (Ledger-Intent, Gate, Anchor, dann KeyDestroy), aber nicht, wie Commits, die vor dem Gate begonnen haben, behandelt werden.

**Empfehlung:** Epochen-Fencing (Vorschlag in 7.7). Die Eigenschaft „kein Commit nach Gate“ als Invariante in einem Modell formulieren und prüfen.

### 6.8 Formale Modelle fehlen (hoch)

Das Verzeichnis `formal/` enthält laut Befund nur eine README. Die Löschprotokolle ER0 bis ER7 mit Crash-Idempotenz sind zu komplex, um sie nur mit Tests abzudecken. Die Spezifikation fordert Crash-Sweeps, die stichprobenartig prüfen. Ein Modell kann dagegen alle Reihenfolgen durchspielen.

**Empfehlung:** Ein Modell in Stateright (Rust-Model-Checker) oder TLA+. Invarianten: kein `Complete` ohne Vernichtungsnachweis; kein Commit nach Gate; Restore kann Tombstones nicht zurücknehmen; Wiederholung ist idempotent.

### 6.9 Unsafe-Inseln sind im Ist-Zustand nicht eingehalten (mittel)

Die R11 nennt drei Unsafe-Inseln (`simd`, `sys`, `wire`). Der Test `tests/unsafe_islands.rs` erlaubt aber zusätzlich `contextra-vector`, `contextra-store`, `contextra-db` und `contextra-crypto`, mit Verweis auf geplante Verlagerung in Phase 1c. Das ist ein offener Migrationsposten, sollte aber in der Zielarchitektur als solcher sichtbar sein.

### 6.10 Toolchain-Pin ist nicht belegt (hoch)

Der Workspace fordert Rust 1.89 (`rust-version`, `rust-toolchain.toml`). Die vorliegenden Testergebnisse stammen von 1.85.1 mit `--ignore-rust-version`. Die R11 sagt das selbst (F-12, R-6). Solange kein CI-Lauf auf dem Pin existiert, sind alle Aussagen über Build- und Testverhalten Hypothesen.

### 6.11 Commit-Historie (niedrig)

Die letzten Commits heißen „Shell-Commit“. Für ein sicherheitsrelevantes Projekt mit Löschnachweisen ist eine nachvollziehbare Historie wichtig, idealerweise mit signierten Commits und Bezug zu Spezifikations-IDs.

---

## 7. Lösungskonzepte

### 7.1 Ein Löschversprechen in drei Stufen

Die Lösung für B-1 ist keine bessere Formulierung, sondern eine Produktlogik, die das Versprechen an die Schlüsselzusicherung koppelt. Vorschlag:

| Stufe | Geltungsbereich | Voraussetzung | Zulässige Aussage (Beispiel) |
| --- | --- | --- | --- |
| **A: Kryptografisch vollzogen** | Persistenz im Kern, Personendaten, versiegelte Cluster | Hardware- oder KMS-Schlüssel, Backup-Schlüssel ohne Export oder begrenzt | „Mit der Vernichtung des Schlüssels sind die Daten im Speicher nicht mehr lesbar. Der Nachweis ist signiert.“ |
| **B: Vollzogen mit benannten Restrisiken** | Wie Stufe A, aber Standardschlüssel oder Backup-Schlüssel mit Ablaufdatum | Explizite Bestätigung im Betrieb, Ausweisung im Receipt | „Die Daten sind unlesbar, außer in Sicherungen bis zum Ablauf des Backup-Horizonts am \[Datum\]. Die Restrisiken sind im Nachweis aufgeführt.“ |
| **C: Außerhalb der Systemgrenze** | Externe Modelle, Provider-Logs, Exporte, Kopien beim Kunden | Vertragliche Regelung, Auftragsverarbeitung | „Diese Kopien liegen bei Dritten. Die Löschung dort ist vertraglich geregelt und wird nicht vom System bestätigt.“ |

Die README sollte dieses Modell abbilden. Die aktuelle Aussage „Löschen heißt wirklich löschen“ gilt nur für Stufe A.

### 7.2 Schlüsselmodell mit Hardware-Bindung

**Zielbild:** Eine Schlüsselhierarchie mit folgenden Ebenen:

- Ein Root-Schlüssel (KEK) je Instanz, verwahrt in einem TPM 2.0 oder einem HSM. Er wird nie exportiert.
- Ein Schlüssel je Löschdomäne (DEK), verschlüsselt unter dem KEK. Die Löschung einer Domäne vernichtet nur deren DEK.
- Optional ein Escrow-Schlüssel mit `Bounded{until}`, der nach Ablauf selbst vernichtet wird.

**Für den Mittelstand** ist das TPM 2.0 der naheliegende Start, weil es in vielen Servern und Arbeitsplatzrechnern vorhanden ist. Ein USB-HSM mit nicht exportierbaren Schlüsseln ist die zweite Option. Ein EU-KMS ist der Pfad für Cloud-Betrieb. Die Spezifikation sieht alle drei als Adapter vor, nennt aber keinen als Priorität.

**Vorbehalt:** Ob ein TPM oder HSM einen belastbaren Vernichtungsnachweis liefert, ist gerätespezifisch. Das muss pro Gerätemodell geprüft und dokumentiert werden, sonst bleibt `Hardware` ein Versprechen des Herstellers.

### 7.3 Ein Löschpfad, kein Parallelstapel

Die Spezifikation hat die richtige Zielgestalt, nur fehlt der Übergang:

1. Den Kern-`Eraser` mit ER0 bis ER7 als einzige Instanz festlegen, die Gate, KeyDestroy und Purge ausführt.
2. `contextra-erasure` auf Belegerstellung reduzieren: Attestsammlung, Signatur, `DeletionProofV15`. Kein eigenes Gate, kein eigenes Ledger, keine eigene Saga.
3. `contextra-engine` als einzige Stelle, die Person auf Cluster abbildet und den Löschauftrag stellt.
4. Die Owner-Deletes in `contextra-db` in Owner-Hooks (`on_gate`, `verify_absent`) umwandeln, damit kein Owner eigenständig löscht.
5. `contextra-store` bleibt eingefroren. Löschanfragen für Altdaten werden über den Export und Import gemäß O-4 erledigt, nicht durch Weiterentwicklung des Altpfads.

Der Übergang ist ein Strangler-Muster: Der neue Pfad wird parallel an Testfällen geprüft, bis der alte Pfad keine Aufrufe mehr hat. Dabei gilt, dass kein Löschauftrag im Altpfad angenommen wird, sobald der neue Pfad für denselben Mandanten aktiv ist.

### 7.4 Suchindizes nach Löschdomäne

Das ist die Lösung für den Widerspruch zu „ohne langwieriges Neuordnen“.

| Option | Löschwirkung | Aufwand | Geeignet für |
| --- | --- | --- | --- |
| Globaler Index mit Rebuild | Verzögert, abhängig von der Indexgröße | Hoch | Nicht personenbezogene Daten |
| Clusterlokale Mikroindizes | Sofort mit der Schlüsselvernichtung | Mittel, bei vielen kleinen Clustern höher | Personendaten, Fälle, Personen |
| Globaler Index mit Tombstone-Filter | Logisch sofort, physisch verzögert | Mittel | Übergang und große Bestände |

**Empfohlene Kombination:** Personendaten in clusterlokalen Mikroindizes. Für große Mandanten zusätzlich ein Tombstone-Filter, der bei jeder Suche gelöschte IDs ausblendet. Damit wirkt die Löschung für Abfragen sofort. Der physische Neuaufbau läuft im Hintergrund mit einer ausgewiesenen Frist, die als Restrisiko im Receipt steht.

Der Tombstone-Filter ist nicht nur ein Komfortmerkmal. Er ist die einzige Möglichkeit, einen Index zu bereinigen, ohne ihn sofort neu zu bauen. Er muss selbst geprüft werden: Wenn der Filter die Löschliste nicht rechtzeitig kennt, erscheinen gelöschte Treffer.

### 7.5 `KeyAssurance` als Typ statt Bool

```rust
/// Herkunft der Schlüsselzusicherung. Ersetzt die Bool-Flags in Port und Receipt.
#[non_exhaustive]
pub enum KeyAssurance {
    /// Nicht exportierbarer Schlüssel in TPM oder HSM, mit gerätespezifischem Vernichtungsnachweis.
    Hardware(HardwareAttestation),
    /// Remote-KMS mit Destroy-API.
    Kms(KmsAttestation),
    /// Datei-Passphrase. Vernichtung auf Journal-, CoW- und Flash-Dateisystemen nicht beweisbar.
    FilePassphrase,
}

pub enum KeysInBackups {
    No,
    Bounded { until_utc_ms: i64 },
    Yes,
}

/// Berechnet die Restrisiken aus der Schlüsselzusicherung. `Yes` ist für kryptografische Löschung nicht zulässig.
pub fn key_residual(
    assurance: &KeyAssurance,
    backups: &KeysInBackups,
) -> Result<Option<Residual>, Unsupported> {
    if let KeysInBackups::Yes = backups {
        return Err(Unsupported::UnboundedBackupKeys);
    }
    Ok(match (assurance, backups) {
        (KeyAssurance::FilePassphrase, _) => Some(Residual::WeakKeyAssurance),
        (_, KeysInBackups::Bounded { until_utc_ms }) => {
            Some(Residual::BoundedBackupKeys { until_utc_ms: *until_utc_ms })
        }
        _ => None,
    })
}
```

Die Funktion ist rein und lässt sich mit Property-Tests gegen die Spezifikationsregeln prüfen. Der Ablauf von `Bounded` muss beim Erzeugen des Receipts neu berechnet werden, wie die Spezifikation es verlangt. Das Receipt ist danach nur mit einem `KeyAssurance`-Wert erzeugbar, und `Complete` lässt sich nur aus `Hardware` oder `Kms` ableiten.

### 7.6 `Key32` ohne ungeschützte Kopien

```rust
use zeroize::{Zeroize, Zeroizing};

pub struct Key32(Zeroizing<[u8; 32]>);

impl Key32 {
    /// Übernimmt den Schlüssel aus einem Puffer und überschreibt die Quelle.
    /// Best effort: Zwischenkopien durch den Compiler sind nicht ausgeschlossen.
    pub fn take_from(src: &mut [u8; 32]) -> Self {
        let key = Key32(Zeroizing::new(*src));
        src.zeroize();
        key
    }

    /// Kein Referenz-Export. Der Aufrufer erhält den Schlüssel nur innerhalb des Gültigkeitsbereichs.
    pub fn with_bytes<R>(&self, f: impl FnOnce(&[u8; 32]) -> R) -> R {
        f(&self.0)
    }
}
```

Die Änderung verhindert `let k = *key.expose();` als unbedachte Kopie. Ein Clippy-Lint oder ein Architekturtest kann zusätzlich prüfen, dass kein Aufruf außerhalb des Adapters `expose` benutzt.

### 7.7 Epochen-Fencing für Gate und Commit

Die Idee: Jeder Commit trägt die Löschepoche, unter der er begonnen hat. Das Gate erhöht die Epoche unter exklusivem Zugriff, bevor der Schlüssel vernichtet wird. Commits mit alter Epoche werden vor dem Schreiben abgelehnt oder neu validiert.

```text
Commit:  epoch_start = epoch.load()
         ... Arbeit ...
         unter Commit-Lock: epoch.load() == epoch_start ? schreiben : neu prüfen
Gate:    unter exklusivem Lock: epoch += 1; Cluster sperren
         danach erst KeyDestroy
```

Die Invariante „kein Schreiben nach Gate auf einen gesperrten Cluster“ wird im Stateright-Modell (6.8) geprüft. Die Implementierung braucht dafür keine Architekturänderung im Speicherkern.

### 7.8 Hash-Verkettung je Löschdomäne

Eine Kette je Löschdomäne, darüber eine Merkle-Struktur über die Kettenköpfe. Ein gelöschter Cluster bleibt als Kopfhash erhalten, der keinen Personenbezug trägt. Die Gesamtintegrität bleibt prüfbar, und die Löschung ist nachweisbar. Diese Struktur ist unabhängig von der Schlüsselfrage und sollte früh festgelegt werden, weil sie das Ledger-Format betrifft.

### 7.9 Ring-Regel durchsetzen

Die Dev-Dependency auf `contextra-crypto-ref` wird entfernt, die betroffenen Tests ziehen in `contextra-storage-testkit`. Der Layering-Test prüft zusätzlich Dev-Dependencies gegen die Ringmatrix. Das ist ein Kleinstschritt mit sofortigem Nutzen.

---

## 8. Entscheidungen, die jetzt getroffen werden müssen

### 8.1 Stellungnahme zu den offenen Entscheidungen O-1 bis O-7

| ID | R11-Empfehlung | Stellungnahme |
| --- | --- | --- |
| O-1 | Testkit nicht zusammenführen | **Zustimmung.** |
| O-2 | `contextra-sys` als einzige FFI-Insel | **Zustimmung**, mit der Ergänzung, dass `vector`, `store`, `db` und `crypto` bis zur Verlagerung auf der Ausnahmeliste bleiben (6.9). |
| O-3 | SSI nicht im Speicher | **Zustimmung.** |
| O-4 | Re-Verschlüsselung beim Import | **Zustimmung.** Der Export muss bereits gelöschte Daten ausschließen. |
| O-5 | Hardware-Signer in Produktion | **Zustimmung**, aber mit Vorlauf: Ohne Hardware gibt es keine belastbaren Löschbelege für Personendaten. |
| O-6 | KMS-Adapter erst bei feststehendem Anbieter | **Zustimmung für KMS.** Für TPM oder HSM sollte die Entscheidung früher fallen, weil sie den Standardpfad bestimmt (7.2). |
| O-7 | Domänenbegriffe unverändert, Mapping in der Engine | **Zustimmung.** |

### 8.2 Zusätzliche Entscheidungen

| ID | Frage | Empfehlung | Begründung |
| --- | --- | --- | --- |
| N-1 | Welches Löschversprechen gilt im README? | Stufenmodell aus 7.1 übernehmen, README sofort überarbeiten | B-1. Ein falsches Versprechen ist das größte rechtliche und reputative Risiko. |
| N-2 | Dürfen Personendaten im Standardpfad liegen? | Nein, nur im Produktionsprofil mit Hardware- oder KMS-Schlüssel. Standardpfad nur für Nicht-Personendaten oder mit ausdrücklicher Bestätigung. | `open_default` verweigert `FilePassphrase` mit Personendaten nur bei fehlender Bestätigung. Diese Grenze muss im Betrieb sichtbar sein. |
| N-3 | Welcher Hardware-Pfad zuerst? | TPM 2.0 als erster Adapter, USB-HSM als zweiter, EU-KMS für Cloud | Passt zur Zielgruppe Mittelstand und zur Lieferbarkeit. |
| N-4 | Übergangsausnahmen in `deny.toml` (30.04.2027) | Nicht verlängern. Stattdessen Abhängigkeiten von `tokio` und Altcrates konkret abbauen. | Eine Verlängerung verschiebt die Nachweispflicht, ohne sie zu lösen. |
| N-5 | Regel für Multi-Subject-Dokumente | Fachlich und rechtlich festlegen, bevor der Identity-Owner entsteht | 5.4. Die technische Lösung hängt davon ab. |
| N-6 | Standardwert für `backup_horizon` und Verantwortung dafür | Pflichtfeld mit Obergrenze im Produktionsprofil, Betreiber bestätigt den Wert schriftlich | 5.8. |
| N-7 | Audit-Pflicht gegen Löschung | Externe rechtliche Prüfung beauftragen | 5.7. Hinweis: keine Rechtsberatung durch diese Analyse. |
| N-8 | Windows in der ersten Freigabe? | Erste Freigabe Linux-only für Personendaten, Windows als eigener Meilenstein | 5.10. |
| N-9 | Scope der ersten Personendaten-Freigabe | Eingeschränkt: ein Mandant, ein Datentyp, klar umrissene Quellen, Pflichtprüfung nach R10 §18 | Verkleinert das Restrisiko und das Prüfbudget. |

---

## 9. Roadmap

Die Zeitangaben sind grobe Richtwerte für ein kleines Kernteam und keine Zusage. Die Reihenfolge ist verbindlich.

### Phase 0: Sofortmaßnahmen (ca. 2 bis 4 Wochen)

- README-Löschaussagen auf das Stufenmodell umstellen (N-1). Keine neuen Löschaussagen an Kunden, bis das geschehen ist.
- R11-Befundbasis gegen den Stand korrigieren (Abschnitt 3). Befundtabelle per Skript neu erzeugen.
- Einfrieren: keine neuen Löschfunktionen in `contextra-store`, `contextra-erasure` und den Owner-Deletes, bis der Pfad vereinheitlicht ist.
- CI-Job auf Pin 1.89.0: Build, Tests, `cargo tree --no-dev-dependencies -e features` (6.3), Layering mit Dev-Dependencies (6.4).
- Entscheidungen N-1, N-2, N-4 treffen.

### Phase 1: Vereinheitlichung (ca. 2 bis 3 Monate)

- `KeyAssurance` als Typ einführen und Port und Receipt darauf umstellen (7.5, 6.5).
- `Key32`-API umbauen (7.6).
- Einen Löschpfad festlegen: Kern-`Eraser`, Erasure als Belegschicht, Engine als Auftraggeber (7.3).
- Identity-Owner in der Engine umsetzen, mit Mapping Person zu Cluster und Regel aus N-5.
- Epochen-Fencing implementieren und im Modell prüfen (7.7, 6.7, 6.8).
- End-to-End-Test: Person anlegen, Daten einlesen, Löschung auslösen, anschließend prüfen, dass keine Klartext-Canaries in Speicher, Index, Log und Sicherungssimulation zu finden sind. Dieser Test ist das wichtigste Abnahmekriterium und fehlt heute.

### Phase 2: Hardware und Derivate (ca. 3 bis 6 Monate)

- TPM-2.0-Adapter mit Vernichtungsnachweis pro Gerätemodell (7.2).
- Clusterlokale Mikroindizes für Personendaten und Tombstone-Filter (7.4).
- Hash-Verkettung je Löschdomäne mit Merkle-Struktur (7.8).
- Backup-Horizont als Werkzeug: Erkennung und Vernichtung abgelaufener Sicherungen.
- Externe Prüfung nach R10 §18 beauftragen und einplanen.

### Phase 3: Migration und Breite (danach)

- USB-HSM und EU-KMS als weitere Adapter.
- Windows-Fsync-Konformanz, falls Windows in der Zielgruppe bleibt (N-8).
- Pilotmandant auf dem neuen Kern, Differenztest gegen den Altstand (M5 der R11).
- G-CUTOVER erst, wenn Phase 1 und 2 abgenommen sind.

---

## 10. Gesamtfazit

Contextra hat eine solide architektonische Grundidee und eine Spezifikation, die viele Fehlerklassen bereits sauber benennt. Die Stärke liegt in der Ehrlichkeit der Receipts: Der Kern weigert sich, „unwiderruflich“ zu sagen, wenn die Schlüsselzusicherung schwach ist. Das ist genau die Haltung, die ein Produkt in diesem Markt braucht.

Die Schwäche liegt in der Diskrepanz zwischen dieser Haltung und dem Produkttext. Solange die README mehr verspricht als die Spezifikation, entsteht ein Haftungsrisiko, das technisch nicht zu lösen ist. Die zweite Schwäche ist die fehlende Verdrahtung: Ein Kern ohne Konsumenten, eine Engine ohne Löschlogik und ein Erasure-Crate ohne Aufrufer bedeuten, dass das Löschversprechen heute durch keinen einzelnen Pfad vollständig eingelöst wird.

Der realistische Weg ist, den Umfang einzugrenzen. Personendaten nur mit Hardware-gebundenen Schlüsseln, einen Löschpfad, eine Löschdomäne als Modell und ein Produktversprechen, das die Stufen A, B und C klar trennt. Das ist weniger, als die README heute sagt, aber es ist belegbar. Und belegbare Löschung ist im Mittelstand der Maßstab, an dem sich das Produkt messen lassen muss.

---

## Anhang A: Verifikationsprotokoll

| Prüfung | Befehl (Kurzform) | Ergebnis |
| --- | --- | --- |
| Speicher-Crates im Workspace | `sed` auf `[workspace] members` | 43 Einträge, 7 Speicher-Crates |
| CLAUDE.md-Größe | `wc -c CLAUDE.md` | 2.896 Byte |
| Clock-Definitionen | `grep -rln 'pub trait Clock'` | 11 Dateien (bestätigt) |
| Verbraucher des Kerns | `grep -rl 'contextra_storage::\|contextra-storage'` außerhalb des Kerns | 0 (bestätigt) |
| Verbraucher des Erasure-Crates | `grep` auf `contextra_erasure` und `contextra-erasure` in `Cargo.toml` | 0 außerhalb des Crates |
| Löschbezug Engine | `grep -rli 'erase\|delete_subject\|forget\|shred\|lösch'` | 0 Dateien |
| Löschbezug Store | gleiche Suche in `contextra-store/src` | 10 Dateien |
| Kern-Dev-Dependency | `Cargo.toml` des Kerns | `contextra-crypto-ref` mit Feature `testing` |
| Schlüsseltyp | `keys.rs` | `Key32` mit `Zeroizing`, `from_array` nach Wert, `expose` nach Referenz |
| Provider-Zusicherung | `grep keys_in_backups` | Port: `bool`; Receipt: `Option<u64>`; `weak_key_assurance` nur im Receipt |
| Statuslogik | `receipt.rs`, Zeilen 100 bis 135 | `WeakKeyAssurance` und `BoundedBackupKeys` korrekt verdrahtet |
| Formale Modelle | `ls formal/` | nur `README.md` |
| Panic-Profil | `grep '^\[profile' Cargo.toml` | `release` mit `unwind`, `release-abort` mit `abort` |
| Ablauf Ausnahmen | `grep 'E29\|2027' deny.toml` | Ablauf 2027-04-30 (bestätigt) |
| Phasenstatus | `PHASE-STATUS.md` | P0 bis P1 in Arbeit, P2 bis P6 teilweise, P7 und P8 nicht begonnen |

## Anhang B: Grenzen dieser Analyse

- Es wurde nichts gebaut und nichts getestet. Aussagen zu Cargo-Verhalten (Feature-Unification) beruhen auf dem dokumentierten Verhalten von Resolver 2 und sind vor dem Einsatz im Projekt zu prüfen.
- `contextra-engine`, `contextra-db`, `contextra-ingest` und die Owner-Crates wurden nur per Suche erfasst. Eine Modulanalyse steht aus, wie die R11 selbst einräumt.
- Benchmark-Aussagen der README (Speicherbedarf, Kalibrierung, Durchsatz) wurden nicht reproduziert.
- `contextra-privacy` (DLP) wurde nicht untersucht.
- Rechtliche Fragen zu DSGVO-Löschung, Rechenschaft und Aufbewahrung sind nicht bewertet.