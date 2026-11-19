# Contextra Spezifikation v3.1: Löscharchitektur (rechtsstabile Überarbeitung von v3.0)

**Status:** Entwurf zur Freigabe, Zielzustand v1.0. Ersetzt `Contextra_Spezifikation_v3.0_Loescharchitektur.md` vollständig.
**Stand der Recherche:** 26.10.2026.
**Geltungsbereich:** Löschung, Schlüsselverwaltung, Backup und Wiederherstellung, Nachweis, Fristen. Ring-Architektur, Crate-Budgets, Gate-Katalog und alle nicht genannten Teile der Gesamtspezifikation 2.2.1 bleiben gültig. Wo diese Datei etwas anderes festlegt, gilt sie (Ersetzungstabelle in §23).
**Selbständigkeit:** Diese Datei ist ohne weitere Unterlagen lesbar. Sie enthält die Änderungen an 2.2.1 als Ersetzungstabelle (§23), sodass keine weitere Datei nötig ist.

> **Rechtshinweis.** Technische Spezifikation, keine Rechtsberatung. Software kann eine Rechtswirkung nicht herbeiführen oder garantieren. Sie kann den technischen Zustand herstellen, den die vertretenen Rechtsauffassungen verlangen, und diesen Zustand belegen. Die rechtliche Bewertung trägt der Verantwortliche. Punkte mit „P-xx“ sind vor jeder Außenkommunikation fachanwaltlich zu prüfen. Alle Fundstellen tragen einen Prüfstatus (§25).

---

## 0. Änderungsübersicht gegenüber v3.0

Die Zeilen F-01 bis F-17 sind Befunde aus der Prüfung von v3.0 und der Recherche. Jede Zeile nennt den Abschnitt, in dem v3.1 sie behebt.

| ID | Befund in v3.0 | Behebung in v3.1 | Abschnitt |
|---|---|---|---|
| F-01 | Flash-Reste des gewrappten DocDek bleiben mit KEK und Passphrase entschlüsselbar. Die Klasse C stützt sich nur auf eine Dateisuche. NIST 800-88r2 zählt Wrapping-Keys zu den CE-Zielschlüsseln, ISO/IEC 27040 verlangt die Bereinigung aller Schlüsselkopien. | Epoch Seal (Re-Wrap und Vernichtung der KEK-Epoche), Custody-Stufen K1/K2/K3, Annahmecodes im Bericht, Klasse C getrennt nach C1/C2/C3 | §2, §4 |
| F-02 | Die Begründung der Saga-Reihenfolge („Vernichtung vor Bereinigung würde andere Dokumente unlesbar machen“) widerspricht der Record-Verschlüsselung. Folge: Wartezeit bis zu 30 Tage vor der Schlüsselvernichtung. | Fast-Path: Klartext bereinigen und messen, dann sofort Schlüssel vernichten. Hygiene (physische Bereinigung, Graph-Neuaufbau) danach und terminiert | §6 |
| F-03 | Postings und Graph können nicht unter einem Dokumentschlüssel liegen, weil sie viele Dokumente enthalten. | Artefaktschlüssel (ArtKey) je Index-Artefakt. Vektor-Payload bleibt unter DocDek | §4, §8 |
| F-04 | Der Hash des gewrappten Blobs taugt nicht für den Rohbyte-Scan. | Wrap-Nonce als Suchmuster („Fahndungsnadel“) | §10 |
| F-05 | Die Kanarien-Probe wurde als Beleg für die konkrete Vernichtung gelesen. | Aussagegrenze benannt. Zusätzlich Pre-Destruction-Inventar und Nadelscan | §10 |
| F-06 | Dokumente mit mehreren Betroffenen sind nicht gelöst (nur Verweis auf P-05/O-14). | Betroffenen-Einheiten, Segmentschlüssel, Mixed-Policy | §5 |
| F-07 | Restmetadaten (Länge, Header, `key_id`) verknüpfen Records nach der Vernichtung. | Padmé-Padding, verschlüsselte Record-Header, Selbsttest zu den drei EDPB-Kriterien | §4.6, §10.6 |
| F-08 | Statistik-Reste (IDF, Längen, Zähler) sind nicht geregelt. | Zuordnung zu Index-Artefakten und Neuberechnung | §8 |
| F-09 | „E: technisch vollständig“ und „nachweislich vernichtet“ überzeichnen. | Klassen L, P, C1–C3, M mit Annahmecodes und Textgenerator | §2 |
| F-10 | v3.0 führt `DeletionProof` V5 und `doctor verify-report` ein, 2.2.1 friert V4 ein (E26) und streicht den Verifier (REQ-PRF-5/6). | Ausdrückliche Ersetzung samt ADR-Liste | §21, §23 |
| F-11 | Disaster Recovery: Der `ErasedSet` des Zielsystems fehlt nach Totalverlust. Schlüsselkopien in Backups sind nicht geregelt. | Zwei-Spuren-Backup (Daten ohne Schlüssel, befristete Schlüssel-Backups) und `ErasedSetMirror` | §9 |
| F-12 | „Commitment“ für einen HMAC. EDPB Rn. 52 nennt das keyed hash, Rn. 53 meint etwas anderes. | Umbenennung in Referenzschlüssel und Referenz-Hash. Die Retention folgt dem Schlüssel-Backup-Fenster | §12 |
| F-13 | Zitat- und Statuskorrekturen: EDPB 02/2025 liegt als v2.0 final vor. Die Anonymisierungs-Leitlinien 02/2026 sind Entwurf. Pseudonymisierungs-Leitlinien 01/2025: keine Endfassung gefunden. OLG-Dresden-Datum verifiziert. Art. 41/42 eIDAS. | §1.2 | §1 |
| F-14 | Verification und Validation (NIST r2) fehlen. | Zwei getrennte Zustände, Vier-Augen-Freigabe im Regulated-Tier | §10.5, §11 |
| F-15 | Kein Zwischenzustand nach Art. 18 und keine Karenz für irrtümliche Löschung. | Frozen entspricht Einschränkung. Optionale `erasure_grace` | §6 |
| F-16 | Empfängerinformation (Art. 17 Abs. 2, Art. 19) nicht spezifiziert (P-12). | `RecipientLedger` | §13 |
| F-17 | NIST-Vorbedingung „kein früherer Klartext“ und Prozessspeicher/Dumps ungeregelt. | Härtungsanforderungen und `doctor`-Prüfungen | §14 |

---

## 1. Rechtsmodell und Grenzen der Rechtswirkung

### 1.1 Grundsatz

1. Art. 17 DSGVO verlangt Löschung ohne unangemessene Verzögerung, definiert Löschen aber nicht. Das Standard-Datenschutzmodell (Baustein 60, V1.0a) versteht Löschen als Unkenntlichmachen. Der angeordnete Informationsverlust muss tatsächlich wirken.
2. Die Aufsichten fordern Wirksamkeit und Nachweisbarkeit: EDPB 02/2025 v2.0 Rn. 50 (technische Unmöglichkeit rechtfertigt keinen Verstoß), Rn. 76 (Wirkung zählt, nicht formale Maßnahmen), CEF-Bericht 2025 (Löschung verifizieren und nachweisen können).
3. Es gibt keine bekannte höchstrichterliche Entscheidung, die Schlüsselvernichtung als Löschung anerkennt. Die Rechtslage ist offen (P-01, P-02).
4. Deshalb gilt der Konstruktionsgrundsatz **Verteidigung in der Tiefe**: Contextra stellt für jede erreichbare Datenschicht den Zustand her, den die strengste Auffassung verlangt (physische Bereinigung). Für nicht erreichbare Reste (Flash, Backups) kommt ein zweiter, unabhängiger Mechanismus (Schlüsselvernichtung mit Backup-Trennung) hinzu. Jede Aussage im Bericht nennt ihre Voraussetzungen und Grenzen.

### 1.2 Quellenlage (Belegstärke: ✔ Primärquelle gelesen, ◐ Sekundärquelle, ○ nicht geprüft)

| Quelle | Aussage (paraphrasiert) | Status | Prüfung |
|---|---|---|---|
| EDPB Guidelines 02/2025 Blockchain, **v2.0 final, 07.07.2026** | Rn. 51: Nach Schlüsselvernichtung sind verschlüsselte Daten unverständlich, solange der Algorithmus nicht gebrochen ist, die Entschlüsselung nicht fortschreitet und der Schlüssel nicht kompromittiert war. Verschlüsselte Daten bleiben personenbezogen. Rn. 52: Ein Hash mit Salt oder Schlüssel bleibt personenbezogen, nach Löschung des Schlüssels aber nicht mehr verknüpfbar, wenn dieser nicht kompromittiert war. Rn. 103–104: Zu erreichen ist „effektive Anonymisierung“ durch Löschung der Off-chain-Daten. Verschlüsselte Daten auf der Chain sind nicht ratsam. Rn. 85, 96, Empf. 13: Notfallplan für Algorithmenwechsel, periodische Neubewertung. | final, Analogie (Blockchain), keine unmittelbare Aussage zu RAG | ✔ |
| EDPB Guidelines 02/2026 Anonymisierung, **Entwurf v1.0, 07.07.2026**, Konsultation bis 30.10.2026 | Kriterien No Record Isolation, No Linkage, No Inference. Kontextueller und vereinfachter Ansatz. Verschlüsselung ist reversibel und kein Ersatz für Anonymisierung. Periodische Neubewertung. Anonymisierung ist selbst Verarbeitung. | Entwurf | ◐ (Kanzleiberichte, Suchauszug, Primär-PDF nicht geöffnet) |
| EDPB Guidelines 01/2025 Pseudonymisierung | Weiterhin „Public consultation closed“, keine Endfassung gefunden | Entwurf | ◐ |
| EDPB CEF 2025 Recht auf Löschung (10.02.2026) | 764 Verantwortliche, 32 Aufsichten, 9 Formalverfahren. Issue 6: Löschung in Backups nicht immer ratsam, dann aber Verfahren, um Löschungen nachzuverfolgen und auf wiederhergestellten Systemen umzusetzen („as much as possible“). Best Practice: Daten durch Zufallszeichen ersetzen. Empfehlung: anerkannte Standards befolgen, Löschung verifizieren und demonstrieren. Issue 7: einfache Pseudonymisierung oder Maskierung genügt nicht als Löschung. Issue 4: Einschränkung nach Art. 18, wenn die Entscheidung noch aussteht. | Bericht | ✔ |
| NIST SP 800-88 Rev. 2 (26.09.2025), FAQ vom 16.07.2026 | CE nur bei: kein früherer Klartext, mindestens 128 Bit, Entropie nicht unter Schlüssellänge, Zeroization aller Zielschlüssel (DEK, Wrapping, Master). Backup- und Escrow-Kopien brauchen eigene Regeln. Teil-CE nur bei absoluter Sicherheit, dass Daten die verschlüsselte Grenze nie verlassen haben. Verification und Validation getrennt. Zertifikat mit Methode, Technik, Validierungsentscheidung, Concurrence. | Norm/FAQ | ✔ (FAQ), ◐ (Norm) |
| ISO/IEC 27040:2024 | Alle Kopien der Zielschlüssel sind zu bereinigen, sonst gilt es nicht als CE | Norm | ◐ (SNIA) |
| BayLfD 30. TB (2020) Nr. 12.5 | Zeitversetzte Backup-Löschung zulässig bei technischer Unmöglichkeit, begründeter Löschfrequenz, Zugriff nur über Restore, Nachlöschung bei jedem Restore, Dokumentation | Aufsichtsauffassung (Bayern) | ✔ |
| SDM Baustein 60 V1.0a | Löschen als Unkenntlichmachen. Informationsverlust muss wirken | Aufsichtsauffassung | ✔ (Auszug) |
| EuGH C-413/23 P (EDPS/SRB), 04.09.2025 | Relativer Personenbezug. Für den Verantwortlichen bleiben pseudonyme Daten personenbezogen. Für Empfänger ohne Mittel kann dies anders sein | Urteil | ◐ |
| AG Lörrach, 03.03.2025, 3 C 1099/24 | Kein Anspruch der betroffenen Person auf Löschnachweis ohne Streitfall. Rechenschaftspflicht liegt beim Verantwortlichen | Instanzgericht | ◐ |
| OLG Dresden, **15.10.2024**, 4 U 940/24 | Der Verantwortliche muss vom Auftragsverarbeiter eine aussagekräftige Löschbestätigung einholen. Eine Ankündigung genügt nicht | Berufungsgericht | ◐ |
| OLG Dresden, 03.02.2026, 4 U 292/25 | Anonymisierung als „Minus“ zur Löschung, wenn die betroffene Person sie verlangt | Berufungsgericht, nur Sekundärquelle | ◐ |
| DSB Österreich, Bescheid 05.12.2018 | Löschung durch Unkenntlichmachung zulässig, völlige Irreversibilität nicht nötig | Behördenbescheid | ◐ |
| eIDAS Art. 41 Abs. 2 / Art. 42 | Vermutung der Richtigkeit von Datum und Datenintegrität bei qualifiziertem Zeitstempel (Art. 41). Anforderungen an qualifizierte Zeitstempel (Art. 42). § 371a ZPO betrifft qualifizierte Signaturen, nicht Zeitstempel unmittelbar | Recht | ◐ |
| Ghost Vectors, arXiv 2606.18497 (16.06.2026) | Soft-gelöschte HNSW-Vektoren bleiben rekonstruierbar. Backup-Snapshot liefert gleiche Qualität. SQ8 bewahrt 93 %. Ghost-Knoten verändern Treffermengen und erzeugen Timing-Kanal. Eigener Vorbehalt: Schlüssel-Reste im Dateisystem entwerten die Methode | Preprint, nicht begutachtet | ✔ |

**Eintragsregel:** Aussagen, die in dieser Spezifikation auf ◐ oder ○ beruhen, dürfen nicht in Außentexte übernommen werden, bevor der Volltext geprüft ist (P-14).

### 1.3 Rechtsstabilitätsmatrix

Contextra muss gleichzeitig unter allen plausiblen Auffassungen bestehen. Die Matrix zeigt, welcher technische Zustand welche Auffassung bedient.

| Auffassung | Maßstab | Erfüllender Zustand in Contextra | Rest |
|---|---|---|---|
| T1 Strenge Vernichtung | Alle Kopien im Einflussbereich sind physisch entfernt (SDM, EDPB Rn. 50, 63) | P plus Hygiene (physische Kompaktion, WAL-Scrub, Checkpoint-Rewrite, Index-Neuaufbau) = Klasse M | Flash-Reste, externe Backups (ausgewiesen) |
| T2 Unkenntlichmachung | Nach dem Vorgang ist niemand mit vertretbarem Aufwand zur Identifizierung in der Lage (SDM, DSB, OLG Dresden 4 U 292/25, EDPB-Entwurf 02/2026) | C plus Restmetadaten-Selbsttest (§10.6) plus Annahmen der Perspektivenmatrix | Aufwandsbewertung ist Rechtsfrage (P-01, P-02) |
| T3 Cryptographic Erase | NIST-Bedingungen, ISO/IEC 27040 (EDPB Rn. 51 analog) | C2/C3 mit CE-Traceability, C1 mit ausgewiesener Annahme A-FLASH | Hardware-Restrisiko bei K1 |
| T4 Einschränkung/Aufbewahrung | Art. 18, Art. 17 Abs. 3 (CEF Issue 4, Beispiel 1) | L, `Restricted`, Legal Hold | Dauer ist Rechtsfrage (O-12) |
| T5 Backups | Löschungen nachverfolgen und bei Restore umsetzen (CEF Issue 6, BayLfD 12.5) | Zwei-Spuren-Backup, Restore-Guard, `ErasedSetMirror` | Externe Backups, deren Rücksicherung Contextra nicht sieht |

### 1.4 Perspektivenmatrix

Der Entwurf 02/2026 prüft Anonymität aus Sicht jeder Stelle, die realistisch Zugang hat. Der Bericht trägt deshalb ein Urteil je Perspektive.

| Perspektive | Zugang | Zustand nach Klasse L | nach P | nach C | Anmerkung |
|---|---|---|---|---|---|
| Q1 Leser (MCP-Client, API, `as_of`, Export) | Lesepfade | gesperrt | gesperrt | gesperrt | Lesesperre vor jedem Rückgabepfad |
| Q2 Betreiber mit Prozess- und Dateizugriff, mit Passphrase | Engine, Dateien, Keystore | Klartext nur über Lesepfade, die gesperrt sind | keine Klartextstruktur mit Scope-Inhalt | Persistente Kopien unter vernichtetem DocDek nicht lesbar | Betreiber ist Verantwortlicher, Perspektive ist maßgeblich |
| Q3 Dateiangreifer ohne Passphrase | Datenverzeichnis | alles verschlüsselt | alles verschlüsselt | alles verschlüsselt | Schutz besteht unabhängig von der Löschung |
| Q4 Medienangreifer mit Flash-Reste-Zugriff und ohne Passphrase | Chip-off, Controller-Reste | alles verschlüsselt | alles verschlüsselt | alles verschlüsselt | |
| Q5 Medienangreifer mit Flash-Resten **und** Passphrase | wie Q4 plus Schlüssel | lesbar | lesbar (Ciphertext) | C1: möglich. C2/C3: nicht möglich | Kern von F-01. Annahme A-FLASH |
| Q6 Besitzer eines Daten-Backups ohne Schlüssel-Backup | Backup-Dateien | verschlüsselt | verschlüsselt | nicht lesbar | Zwei-Spuren-Backup |
| Q7 Besitzer eines Schlüssel-Backups älter als die Löschung | Schlüssel-Backup | lesbar bis Fensterende | lesbar bis Fensterende | unlesbar nach Fensterende (`key_backup_max_age`) | Restore-Guard greift beim Einspielen |
| Q8 Empfänger/Dritte (Exporte, LLM-Anbieter, Auftragsverarbeiter) | außerhalb von Contextra | nicht erfasst | nicht erfasst | nicht erfasst | Ausschluss (§15), Empfängerregister (§13) |

### 1.5 Was der Betreiber tun muss

Die Software ersetzt kein Löschkonzept. `doctor export-concept` erzeugt eine Vorlage mit folgenden Bausteinen (nicht rechtsverbindlich):

- Verzeichnis der Löschschichten und Löschfristen je Collection (Struktur angelehnt an die Löschklassen der DIN 66398, deren Status zu prüfen ist, F-13).
- Backup-Richtlinie: Daten-Backup, Schlüssel-Backup, `key_backup_max_age`, Ablageorte, `ErasedSetMirror`-Ziel.
- Textbaustein für Verzeichnis der Verarbeitungstätigkeiten und für die Auftragsverarbeitung (Löschbestätigung nach OLG Dresden 4 U 940/24).
- Ablauf bei Restore (Restore-Guard, Dokumentationspflicht).

### 1.6 Offene Prüfpunkte (P)

| ID | Frage | Hinweis |
|---|---|---|
| P-01 | Gilt Schlüsselvernichtung als Löschung, Anonymisierung oder Einschränkung? | Entscheidend für C. Anonymisierung als „Minus“ (OLG Dresden 4 U 292/25) nur Sekundärquelle |
| P-02 | Entfällt der Personenbezug von Chiffraten nach Schlüsselvernichtung für jede Stelle? | Relative Betrachtung (EuGH C-413/23 P). Entwurf 02/2026 prüfen |
| P-03 | Nachweis der Abwesenheit von Schlüsselmaterial in RAM, Swap, Dumps | §14, nicht vollständig möglich |
| P-04 | Wirkungsbereich: Quelle, Chunks, Vektoren, Indexe, Caches, Logs | durch §7 abgedeckt |
| P-05 | Embeddings als personenbezogen | vorsorglich ja (Ghost Vectors) |
| P-06 | Backup-Frist und Wiedereinspielsperre | bestimmt `key_backup_max_age` und `erased_set_retention` |
| P-07 | Personenbezug der Nachweis-Metadaten | Referenz-Hash bleibt personenbezogen (Rn. 52) |
| P-08 | Rolle des Anbieters | kein Datenzugriff, keine Telemetrie |
| P-09 | Aufbewahrungspflichten, `Restricted` | O-12 |
| P-10 | Produktaussagen (Wettbewerbs- und Vertragsrecht) | §2.5 |
| P-11 | „Unverzüglich“ und Fristen der Hygiene | §13 |
| P-12 | Empfängerinformation Art. 17 Abs. 2, Art. 19 | §13 |
| P-13 | Notfallplan bei Algorithmenversagen | §4.9 |
| P-14 | Volltextprüfung aller ◐-Quellen, insbesondere Entwurf 02/2026 | nach Konsultationsende erneut |
| P-15 | Checkpoint-Policy als Schicht, Haltefrist | O-15 |
| P-16 | Restinformation im Graph bis zum Neuaufbau, Frist `purge_rebuild_max_days` | §8 |
| P-17 | CRA, AI Act, NIS2-Lieferkette | offen |
| P-18 | AVV, Drittland, Beweiswert bei **K3-Verwahrern** | wieder relevant (§4.2) |
| P-19 | Frist- und Eskalationsregel bei `Partial` | §13 |
| P-20 | Weitere Betroffenenrechte (Auskunft, Berichtigung) | außerhalb |

---

## 2. Begriffe, Klassen, Claims

### 2.1 Löschwirkungsklassen

Die Klassen bauen aufeinander auf. Ein Bericht darf nur die höchste Klasse ausweisen, deren Voraussetzungen gemessen sind.

| Klasse | Name | Aussage | Voraussetzungen |
|---|---|---|---|
| **L** | Gesperrt | Der Scope ist für alle Lesepfade gesperrt. Rechtlich einer Einschränkung nach Art. 18 entsprechend. | `ErasedSet`-Eintrag durabel (fsync), Lesepfade geprüft (inkl. `as_of`, Export, Checkpoint) |
| **P** | Klartext bereinigt | In keiner Klartextstruktur, die Contextra beschreibt, ist ein Inhalt des Scopes auffindbar. | Alle Ebene-A-Schichten gemessen, Positivkontrolle bestanden |
| **C1** | Reste unlesbar, Software-Schlüssel | Alle persistenten Kopien liegen nur als Chiffrat unter vernichteten Schlüsseln. Annahme A-FLASH. | Ebene B gemessen, Nadelscan, Pre-Destruction-Inventar, Epoch Seal vollzogen, Custody K1 |
| **C2** | wie C1, Schlüssel in manipulationsresistentem Modul | Annahme A-FLASH entfällt für die Schlüsselwurzel | wie C1, Custody K2 mit bestandener Konformitätsprüfung |
| **C3** | wie C1, externer KMS mit Vernichtungsattest | Annahme A-FLASH ersetzt durch Anbieterattest (Prüfpflicht des Verantwortlichen, P-18) | wie C1, Custody K3 |
| **M** | Messvollständig | P und C sind erreicht, alle Hygienemaßnahmen sind vollzogen (physische Kompaktion, Checkpoint-Policy, Graph-Neuaufbau), die offenen Ausschlüsse sind nur die Pflicht-Ausschlüsse | P, C, Hygiene |

**Orthogonale Angaben im Bericht (immer vorhanden):**
- `legal_state ∈ {Eingeschränkt, TechnischGelöscht, BeschränktAufbewahrt}`
- `backup_window ∈ {Closed, OpenUntil(date), ExternalUnmanaged}` (§9)
- `assumptions[]` (Annahmecodes, §2.2)
- `residuals[]` (zeitlich begrenzte Reste mit Frist, z. B. `R-GRAPH until`)

### 2.2 Annahmecodes

| Code | Annahme |
|---|---|
| A-FLASH | Flash-Reste gewrappter Schlüssel und die Passphrase kommen nicht zusammen in die Hand derselben Stelle (nur bei K1) |
| A-ALGO | Der Algorithmus (AES-256-GCM-SIV, Argon2id, HKDF) ist nicht gebrochen (EDPB Rn. 51) |
| A-KEYLEAK | Der Schlüssel war vor der Vernichtung nicht kompromittiert (EDPB Rn. 51) |
| A-RAM | Kein Klartext oder Schlüssel liegt in Swap, Dumps, Hibernation (P-03) |
| A-EXTBACKUP | Backups außerhalb von Contextra sind nicht erfasst |
| A-HOSTHIST | Verläufe des Host-Systems (Prompt-Historien) sind nicht erfasst |

### 2.3 Weitere Begriffe

**Ebene A:** Klartext-Strukturen (§3). **Ebene B:** persistente Reste (§3). **DocDek:** Schlüssel je Betroffenen-Einheit (§4, §5). **ArtKey:** Schlüssel je Index-Artefakt (§4). **KEK-Epoche:** Schlüsselverschlüsselungsschlüssel mit Gültigkeitsepoche (§4). **Epoch Seal:** Re-Wrap und Vernichtung einer KEK-Epoche (§4.4). **Siegel 1–4:** gestufte Teilberichte (§6.5). **Fast-Path:** Saga-Abschnitt L → P → C (§6). **Hygiene:** Saga-Abschnitt zur physischen Bereinigung (§6).

### 2.4 Verbotene und erzeugte Aussagen

**Verboten** (Gate `deletion-honesty`): „100 % gelöscht“, „unwiederbringlich“, „DSGVO-konform“, „Art.-17-konform“, „rechtskonform gelöscht“, „garantiert gelöscht“, „kryptografisch gelöscht“ ohne Custody und Annahmen, „Löschzertifikat“ ohne Scope und Klasse.

Außentexte werden aus dem Bericht erzeugt (INV-CLAIMS-1). Beispiele:

- Klasse C1: „Der Scope ist gesperrt. Klartextstrukturen sind bereinigt. Persistente Kopien sind nur als Chiffrat vorhanden. Die zugehörigen Schlüssel sind vernichtet. Dies gilt unter den Annahmen A-FLASH, A-ALGO, A-KEYLEAK, A-RAM. Folgende Ausschlüsse bestehen: …“
- Klasse M: wie C, ergänzt um die vollzogene physische Bereinigung und die Liste der Pflicht-Ausschlüsse.

### 2.5 Textqualität

Der Generator liefert Sätze nur aus einer festen Satzbibliothek (kein Freitext). Jeder Satz ist an eine gemessene Tatsache gebunden. Ein Satz ohne gemessene Tatsache ist ein Build-Fehler.

---

## 3. Kernprinzip: zwei Ebenen, drei Spuren

**Ebene A, Klartext-Strukturen.** MemTable, Index-Arena (fp32 und SQ8), Postings im Speicher, Block-, Embedding-, RRF- und Antwort-Caches, Surrogat-Vault. Diese Strukturen brauchen Klartext für die Suche und werden physisch bereinigt (Nullen, Zeroizen, Neuschreiben). Gemessen wird per Scanner.

**Ebene B, persistente Reste.** SSTables, WAL, Blob-Segmente, Index-Artefakte, Snapshots, Checkpoints, Flash-Reste, Daten-Backups. Jeder persistierte Nutzwert liegt als AEAD-Chiffrat unter einem Schlüssel, der zu genau einer Löscheinheit gehört. Die Vernichtung dieses Schlüssels macht alle Kopien gleichzeitig unlesbar, ohne dass jede Kopie überschrieben werden muss.

**Spur Backup.** Daten-Backups enthalten keine Schlüssel. Schlüssel-Backups sind befristet (§9). Dadurch erlischt die Lesbarkeit in Backups nach einer definierten Frist ohne Eingriff in die Backups.

**Warum beides nötig ist.**
- Ohne Ebene A bleiben Klartext und die Suchstruktur (Vektoren sind invertierbar, siehe Ghost Vectors).
- Ohne Ebene B bleiben Flash-Reste und Kopien lesbar. Überschreiben ist auf SSD nicht zuverlässig (Wear-Leveling).
- Ohne Spur Backup wären Backups eine ungelöschte Kopie.

---

## 4. Schlüsselarchitektur

### 4.1 Hierarchie

```
Wurzel (Custody-Slot K1 | K2 | K3)
 └─ KEK[coll, epoche]       je Collection und Epoche, gewrappt unter der Wurzel
     ├─ DocDek[einheit]     je Betroffenen-Einheit, gewrappt unter KEK
     └─ ArtKey[artefakt]    je Index-Artefakt (Text-Segment, Graph-Snapshot), gewrappt unter KEK
K_eng                       Struktur und Integrität (keine Nutzdaten), HKDF aus Wurzel
K_ref[tenant, epoche]       Referenzschlüssel für Referenz-Hashes (§12), zufällig, gewrappt unter KEK
K_sc/*                      Zweckschlüssel der Sidecar-Ketten (HMAC), HKDF aus K_eng
```

Jeder Schlüssel hat genau einen Zweck und ein Label im Zweckregister (INV-KEYPURPOSE-1).

### 4.2 Custody-Stufen

| Stufe | Wurzel | Klasse |
|---|---|---|
| K0 | keine Verschlüsselung (nur Entwicklung, `doctor` rot) | nur L, P |
| K1 | Argon2id aus Passphrase (m ≥ 64 MiB, t ≥ 3, ≥ 250 ms), Software-Keystore | C1 |
| K2 | Siegelschlüssel in manipulationsresistentem, nicht exportierbarem Modul (z. B. TPM-2.0-Objekt im Modul-Speicher, HSM). Die Eignung je Plattform ist Spike S-K2 | C2 |
| K3 | externer KMS/HSM mit Zeitfenster-Löschung und Vernichtungsattest (z. B. AWS-KMS-Modell 7–30 Tage Wartefenster, danach unwiederbringlich) | C3 |

K2 und K3 sind in v1.0 als Port-Implementierungen **vorgesehen**, aber nicht ausgeliefert, solange ihre Konformitätssuite nicht besteht. Der Port liefert bis dahin `CapabilityUnsupported`, nie einen Stub, der Erfolg meldet (REQ-DEL-027). Die Entscheidung E27 (K2/K3 gestrichen) wird dadurch **teilweise zurückgenommen** (ADR N-E44).

**Hinweis zur Eignung:** Hardware-Effaceable-Konzepte (z. B. Apple Effaceable Storage) dienen dem Schnell-Löschen eines Volumenschlüssels. Sie sind kein Mechanismus für beliebige Einzelschlüssel von Anwendungen. Eine Plattform wird nur als K2 anerkannt, wenn die Konformitätssuite nachweist, dass Schlüsselmaterial nach Vernichtung nicht aus Resten rekonstruierbar ist.

### 4.3 Port `EffaceableKeySlot`

```rust
pub trait EffaceableKeySlot: Send + Sync {
    fn custody(&self) -> Custody;                         // K1 | K2 | K3
    fn wrap(&self, material: &Zeroizing<[u8; 32]>, ad: &[u8]) -> Result<WrappedBlob, KeyError>;
    fn unwrap(&self, blob: &WrappedBlob, ad: &[u8]) -> Result<Zeroizing<[u8; 32]>, KeyError>;
    fn rotate(&self) -> Result<SlotGeneration, KeyError>; // neue Wurzel-Generation, alte bleibt bis destroy
    fn destroy(&self, generation: SlotGeneration) -> Result<DestroyAttestation, KeyError>;
}
```

`DestroyAttestation` enthält Custody, Verfahren (Zeroization), Zeitpunkt (aus `Clock`) und einen Beleg, der von der Implementierung stammt. Nur K1 liefert einen lokalen Beleg ohne Dritt-Attest.

### 4.4 KEK-Epochen und Epoch Seal (behebt F-01)

**Problem.** Wird nur der DocDek vernichtet, liegt sein gewrappter Blob als Rest auf dem Flash. Wer KEK und Passphrase besitzt, kann ihn entschlüsseln.

**Lösung.** Die KEK wechselt epochenweise. Ein Epoch Seal:

1. erzeugt `KEK[coll, e+1]`, gewrappt unter der Wurzel,
2. wrappt alle lebenden DocDeks und ArtKeys unter `KEK[e+1]` neu (Re-Wrap, kein Datenrewrite),
3. schreibt die KeyTable atomar neu (REQ-FMT-5),
4. vernichtet `KEK[e]` und seine Wrap-Kopie,
5. bei Custody K1 mit Option `rotate_passphrase_on_seal`: wechselt die Passphrase, sodass die alte Wurzel nicht mehr ableitbar ist, wenn die alte Passphrase unbekannt ist,
6. bei K2/K3: ruft `rotate()` und `destroy()` am Slot.

Nach dem Seal sind Flash-Reste gewrappter DocDeks aus früheren Epochen nur mit `KEK[e]` entschlüsselbar. `KEK[e]` existiert nur noch als Rest, der selbst unter der Wurzel liegt. Bei K2 und K3 endet die Kette im Modul. Bei K1 bleibt die Annahme A-FLASH, die durch Passphrasen-Rotation zeitlich begrenzt wird.

**Auslöser.** Zeitgesteuert (`kek_epoch_len`, Startwert 30 Tage), auf Anforderung (`seal-now`) und automatisch, wenn seit dem letzten Seal mindestens eine Löschung stattgefunden hat und `seal_after_deletion_max` (Startwert 7 Tage) erreicht ist. Batch: Löschungen innerhalb einer Epoche teilen sich einen Seal.

**Klassenwirkung.** C1 bis C3 gelten erst nach dem ersten Seal, der die Löschung umfasst. Zwischen Vernichtung und Seal weist der Bericht die Klasse `C*` (vorläufig) mit `residuals: [R-KEKEPOCH until]` aus.

**Kosten.** Re-Wrap ist O(Anzahl Schlüssel), nicht O(Datenmenge). Die Messung gehört in `benchmarks/budgets.toml`.

### 4.5 DocDek-Regeln

1. Jede Betroffenen-Einheit (§5) erhält genau einen DocDek. Er wird bei `remember` erzeugt.
2. Jeder Record in WAL, SSTable und Blob-Segment wird mit dem DocDek seiner Einheit verschlüsselt. Ein Block mit Records mehrerer Einheiten enthält mehrere Records unter verschiedenen Schlüsseln. Die Blockverschlüsselung unter `K_eng` bleibt als äußere Schicht erhalten. Die Vernichtung eines DocDek macht deshalb **keine anderen Records unlesbar**.
3. Vektor-Payload (fp32 und SQ8) im persistenten Snapshot liegt je Slot unter dem DocDek (Siegel im Snapshot).
4. Der gewrappte DocDek liegt in der **KeyTable** (Ersetzen-Semantik), nicht in einem Append-Log. Das `KeyRegistry`-Sidecar führt nur Ereignisse ohne Schlüsselmaterial.
5. Vernichtung: Eintrag mit Nullen überschreiben, fsync, Eintrag entfernen, KeyTable atomar ersetzen. Arbeitsspeicher wird zeroized (REQ-KEY-008).
6. Der Key-Lock der Einheit wird über die gesamte Saga gehalten. Ein konkurrierendes `remember` wird abgewiesen.

### 4.6 Record-Format und Restmetadaten (behebt F-07)

- AEAD: AES-256-GCM-SIV, Nonce 96 Bit zufällig, Tag 128 Bit. AD = `scope_id ‖ doc_key_id ‖ record_version`.
- **Padding:** Klartextlänge wird nach dem Padmé-Schema aufgefüllt (Leckage der Länge O(log log M), Overhead höchstens 12 %). Mindestlänge je Record: `min_record_bytes` (Startwert 256).
- **Header:** `doc_key_id` und Sequenzbezug stehen **innerhalb** des unter `K_eng` verschlüsselten Blockbereichs, nicht im Klartext des Records.
- **Wrap der Schlüssel:** AEAD-Wrap (AES-256-GCM-SIV) mit zufälliger 96-Bit-Nonce, AD = `key_id ‖ kek_epoch`. Die Nonce dient als Suchmuster (§10.3). Alternative AES-KWP (SP 800-38F) ist per ADR wählbar, hat aber keine Nonce und braucht ein anderes Suchmuster.

### 4.7 Index-Artefakte (behebt F-03)

Postings und Graph enthalten viele Dokumente und können nicht unter einem DocDek liegen.

| Artefakt | Schlüssel | Bereinigung |
|---|---|---|
| Vektor-Payload im Snapshot | DocDek | Vernichtung des DocDek |
| Graph-Snapshot (Topologie, Slot-Nummern) | ArtKey der Snapshot-Generation | Neuaufbau, danach Vernichtung des ArtKey (§8) |
| Text-Segment (Postings, Term-Wörterbuch, Längen) | ArtKey je Segment | Segment neu schreiben mit neuem ArtKey, alten ArtKey sofort vernichten |

### 4.8 KeyTable und KeyRegistry

- KeyTable ist ein eigener Speicherbereich mit Ersetzen-Semantik. Nach jeder Vernichtung und jedem Seal wird die Segmentdatei ohne vernichtete Einträge neu geschrieben.
- `KeyRegistry` (`MFKR`) führt Ereignisse (`created`, `destroyed`, `sealed`) mit Referenz-Hash, ohne Schlüsselmaterial, ohne Wrap-Blob (REQ-DEL-N5).

### 4.9 Krypto-Agilität und Notfallplan (P-13)

Jedes persistierte kryptografische Objekt trägt Algorithmus- und Formatkennung. Ein Wechsel von AEAD, KDF, Hash oder Signatur ist ohne Datenverlust möglich (Re-Wrap bzw. Neuverschlüsselung per Kompaktion). Das Notfallverfahren bei Algorithmenversagen (EDPB Rn. 85, 96, Empf. 13) wird dokumentiert und halbjährlich per Test geübt (`doctor crypto-drill`). Die Eignung der Primitive wird periodisch neu bewertet.

---

## 5. Betroffenen-Einheiten und Mehrfachbezug (behebt F-06)

**Einheit.** Eine Betroffenen-Einheit ist die kleinste Menge von Records, die ohne Kollateralverlust für einen Betroffenen gelöscht werden kann. Standard: ein Dokument mit genau einem Betroffenen.

**Referenzen.** Jedes Dokument trägt `subjects: [subject_ref]`. `subject_ref` ist eine zufällige ID mit Mindestentropie (§12), vergeben vom Aufrufer (oder durch eine optionale Erkennung `[R]`). Ohne Angabe gilt das Dokument als Einheit des Aufrufers.

**Mehrfachbezug.** Hat ein Dokument mehrere Betroffene, wird es in Segmente zerlegt. Jedes Segment erhält die Menge seiner Betroffenen. Segmente mit gleicher Menge teilen einen DocDek.

Bei einer Löschung für Betroffenen A:

| Segmentart | Behandlung |
|---|---|
| Menge = {A} | DocDek vernichten |
| Menge enthält A und andere (Mixed) | Standard `BlockAll`: Segment bleibt gesperrt, `Partial{ReviewRequired}` mit Frist. Der Betreiber entscheidet |
| Menge enthält A nicht | unberührt |

**Mixed-Policy** (je Collection, Betreiberentscheidung):
- `BlockAll` (fail-closed, Standard),
- `RedactAndRekey`: Der Betreiber liefert den redigierten Text. Neuer DocDek, Re-Embedding, alter DocDek wird vernichtet,
- `DeleteAll`: Segment vollständig löschen (Betreiber trägt die Verantwortung für Kollateral).

Contextra entscheidet nie selbst, ob ein Mixed-Segment nach Entfernung von A noch personenbezogen ist. Das ist eine Rechtsfrage (Art. 17 Abs. 3).

**Ableitungen** (Zusammenfassungen, Chunk-Präfixe, Embedding-Caches) kaskadieren über die Lineage (INV-DERIVED-1). Ableitungen mit weiteren Quellen werden neu abgeleitet oder als `stale` markiert.

**Collection-Löschung (Akte).** Eine ganze Collection wird gelöscht, indem alle DocDeks, ArtKeys und `KEK[coll,*]` in einer Saga vernichtet werden und die Index-Dateien der Collection entfallen.

---

## 6. Löschsaga

### 6.1 Zustände

`Requested → Authorized → Frozen → LogicallyDeleted → PlaintextPurged → PlaintextMeasured → KeyDestroyed → CryptoMeasured → Hygiene → HygieneMeasured → Reported → Completed`, sowie `Partial{outstanding}`, `Restricted`, `ReviewRequired`, `Rejected`.

### 6.2 Reihenfolge und Begründung

Fast-Path (Ziel Minuten bis Stunden):

| Schritt | Aktion | Durabilität | Begründung |
|---|---|---|---|
| 1 | `Frozen`: `ErasedSet`-Eintrag, Lesesperre, `legal_state = Eingeschränkt` | fsync vor jeder Änderung | Klasse L, Wirkung nach Art. 18 |
| 2 | `LogicallyDeleted`: Tombstone im WAL-Batch, Lineage-Kaskade | WAL-Commit | crash-sicher |
| 3 | `PlaintextPurged`: Ebene A bereinigen (Arena-Slots nullen, Caches zeroizen, Postings im Speicher, Antwort-Cache). Text-Segmente neu schreiben (neuer ArtKey, alter ArtKey sofort vernichten) | Journal-Eintrag vor jedem Schritt | Klasse P, Index-Reste sofort |
| 4 | `PlaintextMeasured`: Scanner Ebene A, **Pre-Destruction-Inventar** der Chiffrat-Blöcke je `key_id` | Bericht mit Positivkontrolle | P. Das Inventar ist nur vor der Vernichtung möglich |
| 5 | `KeyDestroyed`: DocDek vernichten (nach optionaler `erasure_grace`) | Journal-Intent vor Vernichtung | alle Ebene-B-Reste sofort unlesbar |
| 6 | `CryptoMeasured`: Nadelscan, Kanarien-Probe, Chiffrat-Identität | Bericht | C* (vorläufig) |
| 7 | Epoch Seal (§4.4), soweit fällig oder `seal-now` | Journal | C1/C2/C3 |

Hygiene (terminiert, ≤ `purge_rebuild_max_days`):

| Schritt | Aktion | Begründung |
|---|---|---|
| 8 | Physische Kompaktion der betroffenen SSTables, WAL-Scrub, Blob-Rewrite, Checkpoint-Policy | Strenge Auffassung T1, Platz |
| 9 | Graph-Neuaufbau (Generationswechsel), danach ArtKey der alten Generation vernichten (§8) | schließt `R-GRAPH` |
| 10 | `HygieneMeasured`, `Reported`, Siegel 4 (Klasse M) | Abschluss |

**Warum die Vernichtung vor der physischen Bereinigung liegt.** Records sind je Einheit verschlüsselt (§4.5). Eine gemeinsame Blockstruktur macht die Vernichtung deshalb nicht kollateral. Die Reihenfolge in v3.0 (Bereinigung vor Vernichtung) verzögerte die wirksamste Maßnahme um bis zu 30 Tage. Sie hat nur ein Messargument: Das Pre-Destruction-Inventar und die entschlüsselte Sicht sind nur vorher möglich. Beides ist in Schritt 4 abgedeckt.

**Verworfene Alternative „Key-first“.** Die Vernichtung unmittelbar nach `Frozen` würde die Messung der entschlüsselten Sicht verhindern und die Rücknahme bei Irrtum ausschließen. Die Karenz (§6.3) ersetzt sie.

### 6.3 Karenz und Rücknahme

Bis Schritt 5 ist die Löschung rücknehmbar (`Unfreeze` mit Begründung im Audit). `erasure_grace` (Startwert 0, Obergrenze 72 h, Pflichtwert in Regulated ohne Standard) legt fest, wie lange nach Schritt 4 gewartet wird. Während der Karenz gilt `legal_state = Eingeschränkt`. Der Retention-Reaper nutzt die Karenz von 7 Tagen (§11.6 der Gesamtspezifikation). Das Modell folgt dem Wartefenster üblicher KMS-Dienste (AWS 7–30 Tage, Abbruch möglich), ist aber auf Stunden verkürzt, um die Pflicht zur unverzüglichen Löschung nicht zu verzögern.

### 6.4 Crash-Verhalten

- Absturz vor Schritt 5: Recovery setzt bei `PlaintextPurged` fort. Der Scope bleibt gesperrt.
- Absturz nach Schritt 5: Die Vernichtung ist vollzogen, Recovery misst nur noch. Ein Bericht trägt nie Klasse C vor abgeschlossener Messung.
- Intent ohne Ausführungsnachweis wird beim Start wiederholt. Vernichtung ist idempotent.
- Der Epoch Seal ist ein eigener durabler Ablauf mit Journal. Ein Absturz mitten im Re-Wrap hinterlässt `KEK[e]` und `KEK[e+1]` nebeneinander und wird beim Start fortgesetzt.

### 6.5 Siegel (gestufte Teilberichte)

| Siegel | Zeitpunkt | Inhalt |
|---|---|---|
| 1 | nach Schritt 1 | Klasse L, `legal_state` |
| 2 | nach Schritt 4 | Klasse P |
| 3 | nach Schritt 7 | Klasse C1/C2/C3 mit Annahmen |
| 4 | nach Schritt 10 | Klasse M |

Jedes Siegel ist ein signierter Bericht, in die AuditChain eingetragen und mit dem vorherigen verkettet. Der Betreiber kann damit nachweisen, wann welcher Zustand erreicht war.

### 6.6 Restricted und Legal Hold

Bei Aufbewahrungspflicht (Art. 17 Abs. 3) oder Legal Hold geht die Saga nach Schritt 1 in `Restricted{basis, until}`. Keine Löschoperation, kein Schlüsselvernichten. Die Lesesperre bleibt. Nach Ablauf setzt die Saga bei Schritt 2 fort. Checkpoint-Policy `ExpireAfterHold` gilt nur für Checkpoints.

---

## 7. Schichten

Die Schichten aus 2.2.1 bleiben. Jede Schicht ist einer Ebene zugeordnet. Besitzer-Crate ist der Stellung von Scanner und Purger.

| Schicht | Crate | Ebene | Maßnahme | Messung |
|---|---|---|---|---|
| S1 LSM und WAL | `store` | A (MemTable), B | Records unter DocDek. MemTable zeroizen. Hygiene: `compact_range(scope)`, WAL-Scrub | `LsmScanner` (Klartext, Chiffrat, Nadel) |
| S1b Blob-Segmente | `store` | B | Records unter DocDek. Hygiene: Segment-Rewrite | `LsmScanner` |
| S2 Vektorindex | `vector` | A, B | Arena-Slots (fp32, SQ8) nullen. Snapshot-Payload unter DocDek. Graph unter ArtKey, Neuaufbau nach Tombstone-Anteil > 0,2 oder spätestens `purge_rebuild_max_days` (Startwert 14, P-16). Slot-Nummern werden vor Neuaufbau nicht wiederverwendet (INV-HNSW-SLOT-1). Kein Rewiring (VETO-F02) | `VectorScanner` |
| S3 Textindex | `text` | A, B | Segmente neu schreiben (neuer ArtKey, alter sofort vernichtet). Terme, die nur im Scope vorkamen, aus dem Wörterbuch entfernen. Statistik neu berechnen | `TextScanner` |
| S4 Caches | `store`, `engine`, `privacy` | A | Block-, Embedding-, RRF-, Antwort-Cache und Surrogat-Vault invalidieren und zeroizen | `CacheScanner` |
| S5 Lineage und Ableitungen | `engine` | A | Kaskade (INV-DERIVED-1) | `LineageScanner` |
| S6 Checkpoints | `checkpoint` | B | Policy `Rewrite` (Standard), `ExpireNow`, `ExpireAfterHold` nur unter Hold. Records liegen unter DocDek und sind nach Vernichtung unlesbar. Bereinigt gelten sie erst nach Rewrite | `CheckpointScanner` |
| S7 Sidecars | `crypto` | – | nur Referenz-Hashes, keine Nutzinhalte, keine Schlüssel | `SidecarScanner` |
| S8 Externe Sicherungen | – | – | ausweisen (`ExternalBackups`). Daten-Backups der Spur Backup sind keine externen Backups, sofern Contextra sie erzeugt (§9) | keine Messung |

**Oberflächen-Inventar (Gate `surface-coverage`).** Jede persistente Datei- und Strukturklasse des Format-Registers (§7.11 der Gesamtspezifikation) steht in genau einer Zeile dieser Tabelle oder in §15. Ein neues Format ohne Schichtzuordnung bricht den Build (Lückenfreiheit).

---

## 8. Index-Generationen und Restinformation

### 8.1 Residual `R-GRAPH`

Im HNSW-Graph bleiben nach der Löschung Routing-Knoten ohne Nutzdaten (VETO-F02 verbietet Rewiring). Die Kanten enthalten nur interne Slot-Nummern. Die Topologie verrät die Nachbarschaftsbeziehungen zu gelöschten Vektoren. Das ist Restinformation im Sinne von No Inference. Sie bleibt als `R-GRAPH` mit Frist im Bericht, bis der Neuaufbau erfolgt ist.

### 8.2 Neuaufbau

1. Hintergrund-Neuaufbau erzeugt Generation g+1 ohne gelöschte Dokumente (deterministisch, INV-INDEX-APPLY-1).
2. Atomarer Umschalt-Commit auf g+1.
3. Vernichtung des ArtKey der Generation g.

Mehrere Löschungen teilen sich einen Neuaufbau (Batch). Auslöser: Tombstone-Anteil ≥ 0,2, Fristablauf, `seal-now`.

### 8.3 Restkanäle

- Treffermengen: Ghost-Knoten verändern in der Studie 95 % der Top-K-Mengen und erzeugen einen Timing-Unterschied von rund 4 %. Contextra sichert Seiteneffekte nur klassenweise zu (§9.4 der Gesamtspezifikation). Der Neuaufbau beseitigt sie.
- Statistik: IDF, Dokumentlängen und Zähler liegen im Text-Segment und werden mit der Segment-Neuschreibung neu berechnet.

### 8.4 Vergleich

Qdrant (Vacuum-Optimizer mit Schwellen von 20 % gelöschter Punkte und 1000 Vektoren), Milvus (Compaction plus Garbage Collection mit Stunden Verzögerung), Weaviate (Tombstones, Cleanup per Timer) und Chroma/FAISS (kein erzwungener Neuaufbau) behandeln Löschung als Wartung, nicht als Nachweis (Sekundärquellen, §19). Contextra erzwingt die Frist und misst.

---

## 9. Backup und Restore

### 9.1 Zwei-Spuren-Backup (behebt F-11)

| Spur | Inhalt | Lebensdauer | Schutz |
|---|---|---|---|
| D (Daten) | SSTables, WAL, Blob-Segmente, Index-Artefakte (Chiffrat), ohne KeyTable und ohne Keystore | beliebig (Betreiberentscheid) | ohne Schlüssel nicht lesbar |
| K (Schlüssel) | KeyTable-Snapshot (gewrappte DocDeks, ArtKeys, KEK-Epochen) | **befristet** auf `key_backup_max_age` (Startwert 30 Tage), signiertes `valid_until` | Verschlüsselung unter der Wurzel |

**Wirkung.** Nach Löschung und Ablauf aller Schlüssel-Backups, die den DocDek enthalten, existiert der Schlüssel nirgends mehr. Die Daten-Backups sind für diese Einheit unlesbar. Die Löschung wirkt in Backups spätestens nach `key_backup_max_age`, ohne Eingriff in das Backup.

**Bericht.** `backup_window = OpenUntil(deletion_at + key_backup_max_age)` für von Contextra verwaltete Backups. `ExternalUnmanaged` für alles andere.

### 9.2 Restore-Guard

Vor dem ersten Lesezugriff nach einem Restore:

1. `ErasedSet` des Zielsystems lesen (nicht aus dem Backup).
2. Für jeden Eintrag: Saga ab Schritt 3 erneut anwenden, wiederhergestellte DocDeks vernichten.
3. Messung wie in §10. Journal-Ereignis `restore_reapplied`.
4. Dokumentation (Grund, Zeitpunkt) im Audit (BayLfD 12.5).

**Restore-Horizont.** Ein Schlüssel-Backup mit `valid_until` in der Vergangenheit oder unbestimmbarem Alter wird verweigert (`RestoreBeyondHorizon`). Daten-Backups unterliegen keinem Horizont, weil sie ohne Schlüssel inert sind.

### 9.3 `ErasedSetMirror` (behebt F-11)

Nach Totalverlust fehlt der `ErasedSet` des Zielsystems. Der `ErasedSetMirror` ist ein Port, der jeden `ErasedSet`-Eintrag (Referenz-Hash, Zeitpunkt, Epoche) zusätzlich an ein vom Betreiber bestimmtes Ziel außerhalb des Datenverzeichnisses schreibt (zweiter Datenträger, WORM, Fremdsystem). Im Regulated-Tier ist er Pflicht. Ohne Mirror oder bei einem Mirror, der älter ist als das Schlüssel-Backup, ist ein Restore nur mit `--accept-unguarded` möglich. Der Bericht und `doctor` weisen `restore_unguarded` aus.

Die Alternative eines zentralen Löschregisters außerhalb von Contextra (O-35) bleibt als Betreiberentscheidung offen und kompatibel (es ist ein zulässiges Mirror-Ziel).

### 9.4 Aufbewahrungsfrist des `ErasedSet`

`erased_set_retention ≥ key_backup_max_age + Sicherheitszuschlag` (Startwert insgesamt 90 Tage, Obergrenze nach Rechtsprüfung P-06/P-07). Die Konfiguration erzwingt `key_backup_max_age ≤ erased_set_retention`. Die lange Frist von 365 Tagen aus 2.2.1 ist nicht mehr nötig, weil nur Schlüssel-Backups eine Rücksicherung gelöschter Inhalte ermöglichen.

### 9.5 Verhältnis zu Aufsichtsauffassungen

CEF Issue 6 und BayLfD 12.5 verlangen Nachverfolgung und Nachlöschung beim Restore sowie Begründung, wenn Backups nicht sofort bereinigt werden. Der Zwei-Spuren-Ansatz erfüllt dies mit einem festen, dokumentierten Zeitfenster. Ob ein Chiffrat ohne Schlüssel für den Besitzer des Daten-Backups keine personenbezogene Information mehr ist, bleibt P-02. Der Entwurf 02/2026 betont die Perspektive der Stelle mit Zugang.

---

## 10. Messung und Nachweis

### 10.1 Grundsätze

1. Messen vor Behaupten. Ein Beleg entsteht nur aus einem Scanner der besitzenden Schicht (INV-WITNESS-MINT-1).
2. Jeder Scan-Lauf hat eine Positivkontrolle. Ohne bestandene Kontrolle ist der Lauf ungültig.
3. Messungen am Speicher, nicht am Lesepfad.
4. **Verification** (technischer Lauf erfolgreich) und **Validation** (Betreiber nimmt das Ergebnis ab) sind getrennte Zustände (NIST r2, F-14).

### 10.2 Ebene A: Klartext-Scanner

1. Rohbyte-Scan aller Dateien des Datenverzeichnisses nach kanonischen Termen, Roh-IDs, Embedding-Bytes und Chunk-Prüfsummen des Scopes.
2. Dekodierte Sicht über die normalen Lesepfade mit allen aktiven Schlüsseln.
3. Positivkontrolle (AC-1): Ein eingeschleuster Kontrolldatensatz wird in beiden Sichten gefunden.

### 10.3 Ebene B: drei Nachweise

1. **Pre-Destruction-Inventar.** Vor der Vernichtung wird je `key_id` die Liste aller Chiffratblöcke (Datei, Offset, Länge) als Digest und Zähler festgehalten. Danach gilt: Jeder Block muss einem vernichteten oder lebenden Schlüssel zugeordnet sein. Blöcke ohne Zuordnung sind ein Fehler (fail-closed).
2. **Nadelscan (Schlüsselabwesenheit, F-04).** Gesucht wird nach der **Wrap-Nonce** des vernichteten Schlüssels (96 Bit, zufällig), die vor der Vernichtung im Journal liegt, in allen Dateien: KeyTable, Keystore, Sidecars, Checkpoints, Temp, `.bak`. Ergebnis: null Treffer. Ein Hash des Blobs eignet sich nicht, weil die Rohbyte-Suche nach einem Hash alle Offsets hashen müsste.
3. **Kanarien-Probe (Positivkontrolle).** Ein Kanarien-Datensatz mit eigenem DocDek wird verschlüsselt und vom Scanner entschlüsselt, dann wird sein Schlüssel über **denselben Vernichtungspfad** vernichtet und die Entschlüsselung muss scheitern. Die Probe belegt Scanner und Vernichtungsweg. Sie belegt **nicht** die Vernichtung des konkreten DocDek (F-05). Dafür stehen Nadelscan und Inventar.

### 10.4 Grenzen (immer im Bericht)

Der Scanner sieht keine Flash-Remanenz unterhalb des Dateisystems, keinen Schlüssel in RAM, Swap, Dumps (A-RAM), keine externen Backups.

### 10.5 Verification und Validation

| Zustand | Bedeutung |
|---|---|
| `Verified` | Alle Schichten gemessen, Positivkontrollen bestanden |
| `Validated` | Eine zuständige Person nimmt das Ergebnis ab (Validierungsentscheidung). Im Regulated-Tier Pflicht, mit zweiter Signatur (Concurrence) |

Die Klasse im Bericht gilt für beide Zustände. `Validated` ist ein zusätzliches Feld.

### 10.6 Restmetadaten-Selbsttest (zu F-07)

Nach der Vernichtung prüft `doctor` stichprobenartig mit einfachen Tests, ob verbleibende Chiffrat-Records einer gelöschten Einheit

- sich über Klartext-Header einer Einheit zuordnen lassen (No Linkage),
- eine einzigartige Länge haben (No Record Isolation, Prüfung der Verteilung nach Padding),
- aus Zählern, Statistik oder Zeitstempeln Inhalte ableiten lassen (No Inference).

Der Test liefert eine Warnung, kein Rechtsurteil. Er ersetzt keine Bewertung nach den Leitlinien.

### 10.7 Statistische Residualtests

Auf Restdaten, die nach der Löschung noch Chiffrat sind, laufen Entropietests. Ein unerwartet niedriger Wert zeigt einen Chiffrierfehler an.

---

## 11. Bericht, Bescheinigung, Zeitanker

### 11.1 Bericht (`DeletionProof` V5)

Felder (Mindestumfang): Klasse, Siegelnummer, `legal_state`, `backup_window`, `assumptions[]`, `residuals[]`, je Schicht (Ebene, Methode, Zahlen, Digest, Positivkontrolle, Ergebnis), Custody und Destroy-Attestation, Epoch-Seal-Referenz, Referenz-Hash des Betroffenen, Pflicht-Ausschlüsse, Verification/Validation, Zeitstempel (aus `Clock`), Signatur (Ed25519 aus dem `KmsProvider`, INV-PROOF-KEY-1), Prüfsumme über alle Felder.

**NIST-Zertifikatsfelder.** Zuordnung: Produkt und Version, Medium (Dateisystemtyp, Laufwerksklasse falls bekannt), Methode (Purge), Technik (Cryptographic Erase, Blockrewrite), Validierungsentscheidung, Concurrence, CE-Traceability (Algorithmen, Schlüsselstärken, Schlüsseltypen, Escrow-/Backup-Historie).

### 11.2 Löschbescheinigung (abstrakt, datensparsam)

Aus dem Bericht wird eine einseitige Bescheinigung erzeugt (OLG Dresden 4 U 940/24, AG Lörrach 3 C 1099/24):

- enthält: Datum, Art des Scopes (z. B. „Akte“), erreichte Klasse je Schichtgruppe, Annahmen, Ausschlüsse (zusammengefasst), Bericht-Digest, Signatur, Zeitanker-Status,
- enthält **nicht**: IDs, Dateinamen, Inhalte, Embedding-Daten.

Sie dient der Beantwortung betroffener Personen, der Auftragsverarbeitung (Löschbestätigung) und Aufsichtsanfragen. Ob und an wen sie auszuliefern ist, entscheidet der Verantwortliche. Ein Anspruch der betroffenen Person auf diesen Nachweis ist nicht gesichert (AG Lörrach).

### 11.3 Zeitanker

Der Digest des Berichts wird optional über einen qualifizierten Zeitstempel (RFC 3161, eIDAS Art. 41, 42) verankert. Der Zeitstempel belegt, dass der Bericht zu diesem Zeitpunkt existierte (Datum und Datenintegrität), nicht, dass die Löschung stattgefand.

- **Offline-Betrieb:** Die Anforderung erfolgt, sobald eine Verbindung besteht (`anchor: pending`). Verifikation ist offline möglich, wenn die Zertifikatskette und die Vertrauensliste als Schnappschuss im Bericht liegen (Langzeitvalidierungsdaten).
- Ein ausgestelltes Token ist nicht widerrufbar.
- Die Anbieterwahl ist offen (O-34).
- Der Zeitstempel überträgt nur den Digest.

### 11.4 Verifikation

`doctor verify-report` prüft Signatur, Prüfsumme, Klasse, Konsistenz der Siegelkette und Zeitanker ohne Netzwerk (REQ-DEL-N9). Ein Bericht mit Klasse M und einer Schicht ohne Positivkontrolle ist ungültig. Ein unabhängiger Referenz-Prüfer in anderer Sprache dient nur als Testorakel (INV-ORACLE-1) und gehört nicht zum Produkt.

### 11.5 Aufbewahrung des Berichts

Der Bericht enthält nur Referenz-Hashes und gilt selbst als personenbezogen (EDPB Rn. 52). Aufbewahrung nach `report_retention`, danach pseudonym-ablaufend (§12). Die Dauer ist Rechtsfrage (P-07).

---

## 12. Referenzschlüssel (behebt F-12)

**Zweck.** Berichte, `ErasedSet`, `KeyRegistry` und Journal müssen auf Betroffene verweisen, ohne sie zu identifizieren.

**Regeln.**
1. `subject_ref_hash = HMAC-SHA256(K_ref[tenant, epoche], kanonische Betroffenen-Referenz)`. Dies ist ein **Referenz-Hash mit Schlüssel** nach EDPB Rn. 52 und bleibt personenbezogen. Die Bezeichnung „Commitment“ entfällt. Ein echtes Commitment-Schema ist optional (O-36).
2. Die kanonische Referenz hat Mindestentropie (zufällige Betroffenen-ID, nicht Name oder E-Mail). Sie steht nur im Dokumentdatensatz.
3. `K_ref` ist zufällig, nicht per HKDF ableitbar, unter `KEK` gewrappt, je (Tenant, Epoche).
4. Nach `ref_epoch_len` (90 Tage) plus `erased_set_retention` wird `K_ref` vernichtet (Epoch Seal gilt). Danach sind die Hashes dieser Epoche keiner Person mehr zuordenbar, soweit `K_ref` nicht in Backups steht (Spur K ist befristet, daher unkritisch).
5. Sidecars enthalten keine Nutzinhalte und keine Schlüssel.

---

## 13. Fristen, Empfänger, Legal Hold

### 13.1 Fristen

| Frist | Quelle | Wert |
|---|---|---|
| Beantwortung | Art. 12 Abs. 3 | ein Monat, Verlängerung um zwei Monate mit Mitteilung binnen eines Monats |
| `Frozen` (Klasse L) | intern | sofort nach Autorisierung |
| Klasse P | intern, Zielgröße | ≤ 1 Stunde |
| Klasse C* | intern | unmittelbar nach P (zuzüglich `erasure_grace`) |
| Epoch Seal | intern | ≤ 7 Tage nach Vernichtung |
| Hygiene (Klasse M) | intern, P-11 | ≤ `purge_rebuild_max_days` (14 Tage) |
| Alarm bei `Partial` | intern | 24 h |
| Eskalation bei `Partial` | intern | 14 Tage |
| Meldung Fristgefährdung | intern | 25 Tage |

Die Werte sind Zielgrößen. Contextra sendet nichts nach außen. Alarme erscheinen in `doctor`, Metrik, CLI, MCP-Antwort und AuditChain.

### 13.2 `RecipientLedger` (Art. 17 Abs. 2, Art. 19, P-12)

Je Dokument führt Contextra ein Register der Ausleitungen (Export, Egress-Kanal `[R]`, Checkpoint-Export), ohne Inhalte: Empfängerkennung, Zeitpunkt, Datenklasse. Bei einer Löschung liefert `doctor` die Empfängerliste. Die Mitteilung an Empfänger erfolgt durch den Verantwortlichen. Ausleitungen außerhalb von Contextra sind Ausschluss.

### 13.3 Legal Hold

Siehe §6.6. Ein Hold wird im `RevocationLog` und Audit geführt. Er betrifft die Saga, nicht die Lesesperre.

---

## 14. Härtung (zu F-17, P-03)

NIST verlangt für CE, dass vor der Verschlüsselung kein Klartext auf das Medium gelangt ist. Contextra setzt dazu:

1. **Core-Dumps aus:** `RLIMIT_CORE = 0`, Prozessattribut gegen Dumps. Speicherbereiche für Schlüssel und Klartext-Arenen sind von Dumps ausgenommen (`MADV_DONTDUMP` bzw. Plattformäquivalent).
2. **Schlüsselspeicher gesperrt:** `mlock` für Schlüsselpuffer. Meldet die Sperrung Fehler, weist `doctor` dies aus.
3. **Swap:** `doctor` prüft, ob Swap aktiv und verschlüsselt ist. Große Arenen (Vektoren) lassen sich nicht sperren, deshalb Warnung bei unverschlüsseltem Swap.
4. **Temp-Dateien:** keine Klartext-Zwischenspeicher. Alle Zwischenergebnisse liegen im Speicher oder verschlüsselt unter ArtKey/`K_eng`.
5. **Dateisystem:** unterstützte lokale Typen (INV-PLATFORM-1). Journal, Snapshots (APFS, btrfs, zfs) und Time Machine gelten als externe Kopien und stehen im Bericht.
6. **Logs:** nur IDs und Zähler (INV-LOG-1).
7. **Hibernation:** `doctor` weist auf aktive Hibernation hin (A-RAM).

Die plattformspezifischen Aufrufe sind in Spike S-HRD zu verifizieren. Bis dahin gilt: Fehlt ein Mittel, meldet `doctor` die Lücke, und die Annahme A-RAM bleibt im Bericht.

---

## 15. Ausschlüsse (verpflichtend im Bericht)

- Externe Sicherungen außerhalb der Spur Backup (A-EXTBACKUP).
- Prozessspeicher, Swap, Dumps, Hibernation vor der Vernichtung (A-RAM).
- Hardwareinterne Reste (Wear-Leveling, Controller-Cache). Gegenmaßnahme: Verschlüsselung plus Epoch Seal, Restannahme A-FLASH bei K1.
- Trainings- und Fine-Tuning-Daten externer Modelle.
- Kopien beim Auftragsverarbeiter, Modellanbieter-Logs, Prompt-Caches, Host-Historien (A-HOSTHIST).
- Ableitungen außerhalb der Lineage (Exporte, die der Betreiber angelegt hat).
- Dateisystem-Snapshots und Journale.

Der Ausschlusstext darf nicht gekürzt werden, wenn der Bericht weitergegeben wird.

---

## 16. Nicht Gegenstand

- Maschinelles Verlernen in Modellgewichten.
- Löschung beim LLM-Anbieter.
- Löschung in Backup-Medien außerhalb der Spur Backup.
- Rechtliche Wertung der Löschwirkung.

---

## 17. Anforderungen

Klasse [K] = Kern. Die Tabelle ersetzt und ergänzt die Anforderungen REQ-DEL-N1 bis N12 aus v3.0.

| ID | Klasse | Anforderung | Mechanismus |
|---|---|---|---|
| REQ-DEL-N1 | [K] | Jeder persistierte Nutzwert MUSS als AEAD-Chiffrat unter dem Schlüssel seiner Löscheinheit (DocDek, ArtKey) vorliegen. Ausnahme K0: nur Klassen L und P | Typestate, Rohdatei-Scan |
| REQ-DEL-N2 | [K] | Die Saga MUSS §6.2 einhalten: Vernichtung erst nach gemessener Ebene A und Pre-Destruction-Inventar | Zustandsautomat, Crash-Matrix |
| REQ-DEL-N3 | [K] | Klasse C DARF nur ausgewiesen werden, wenn Nadelscan, Inventar, Kanarien-Probe und Epoch Seal bestanden sind. Vor dem Seal ist nur `C*` zulässig | Gate, Berichtstyp |
| REQ-DEL-N4 | [K] | Die KeyTable DARF nach Vernichtung keinen Eintrag mehr enthalten. Die Segmentdatei MUSS ersetzt sein | Rohdatei-Scan, Ersetzen-Test |
| REQ-DEL-N5 | [K] | Das `KeyRegistry`-Log DARF kein Schlüsselmaterial und keinen Wrap-Blob enthalten | Literal-Scan, Schema-Test |
| REQ-DEL-N6 | [K] | Jeder Scan MUSS seine Positivkontrolle bestehen | Test-Harness |
| REQ-DEL-N7 | [K] | Antwort-Caches MÜSSEN als Schicht S4 erfasst sein | Scanner, Test |
| REQ-DEL-N8 | [K] | Der Bericht MUSS Klasse, Annahmen, Residuale, `legal_state`, `backup_window` und die Pflicht-Ausschlüsse enthalten | Typestate |
| REQ-DEL-N9 | [K] | `doctor verify-report` MUSS offline Signatur, Prüfsumme, Klasse, Siegelkette und Zeitanker prüfen | CLI |
| REQ-DEL-N10 | [K] | `K_ref` MUSS nach Epoche plus Retention vernichtet werden | Journal, Backup-Test |
| REQ-DEL-N11 | [K] | Der Restore-Guard MUSS wiederhergestellte DocDeks gelöschter Einheiten vernichten | Restore-Test |
| REQ-DEL-N12 | [K] | Außentexte MÜSSEN aus dem Bericht erzeugt werden. Verbotene Begriffe führen zum Build-Fehler | Gate `deletion-honesty` |
| REQ-DEL-N13 | [K] | Die Epoch Seal MUSS KEK-Epochen rotieren, alle lebenden Schlüssel neu wrappen und die alte KEK vernichten. Sie ist crash-sicher und idempotent | Crash-Matrix, Property |
| REQ-DEL-N14 | [K] | Die Custody-Stufe MUSS im Bericht stehen. K2 und K3 DÜRFEN nur nach bestandener Konformitätssuite ausgewiesen werden, sonst `CapabilityUnsupported` | Port-Suite |
| REQ-DEL-N15 | [K] | Records MÜSSEN gepaddet sein (Padmé, Mindestlänge). `doc_key_id` DARF nicht im Klartext des Records stehen | Format-Test |
| REQ-DEL-N16 | [K] | Index-Artefakte MÜSSEN unter ArtKey liegen. Beim Neuschreiben MUSS der alte ArtKey vernichtet werden | Scanner, Test |
| REQ-DEL-N17 | [K] | Mixed-Segmente DÜRFEN nicht automatisch freigegeben oder gelöscht werden. Standard ist `BlockAll` mit `ReviewRequired` | Integration |
| REQ-DEL-N18 | [K] | Daten-Backups DÜRFEN keine KeyTable enthalten. Schlüssel-Backups MÜSSEN `valid_until` tragen und nach Ablauf verweigert werden | Backup-Test |
| REQ-DEL-N19 | [K] | Der `ErasedSetMirror` MUSS im Regulated-Tier konfiguriert sein. Restore ohne aktuellen Mirror nur mit `--accept-unguarded`, im Bericht ausgewiesen | Integration |
| REQ-DEL-N20 | [K] | Der Bericht MUSS Verification und Validation trennen. Im Regulated-Tier ist die Concurrence-Signatur Pflicht | Typestate |
| REQ-DEL-N21 | [K] | Die Löschbescheinigung MUSS aus dem Bericht erzeugt werden und DARF keine IDs und Inhalte enthalten | Schema-Test, Literal-Scan |
| REQ-DEL-N22 | [K] | Der `RecipientLedger` MUSS Ausleitungen ohne Inhalte erfassen und bei Löschung eine Empfängerliste liefern | Integration |
| REQ-DEL-N23 | [K] | `doctor` MUSS Härtungslücken (Core-Dumps, Swap, mlock, Hibernation) ausweisen | Integration |
| REQ-DEL-N24 | [K] | Das Gate `surface-coverage` MUSS jede persistente Strukturklasse einer Schicht oder einem Ausschluss zuordnen | Gate |
| REQ-DEL-N25 | [K] | Der Graph-Neuaufbau MUSS spätestens nach `purge_rebuild_max_days` laufen. Bis dahin steht `R-GRAPH` im Bericht | Integration, Chaos |
| REQ-DEL-N26 | [K] | `erasure_grace` MUSS im Regulated-Tier ohne Standardwert konfiguriert werden und darf 72 h nicht überschreiten | Konfigurationstest |
| REQ-DEL-N27 | [K] | Der Bericht MUSS Siegel 1 bis 4 verketten und in die AuditChain eintragen | Property |

Weitere Anforderungen aus 2.2.1 bleiben, soweit §23 sie nicht ersetzt.

---

## 18. Tests und Gates

**Crash-Matrix:** Abbruch an jedem Zustandsübergang der Saga und des Epoch Seal mit Neustart. Geprüft: Lesesperre, Schlüsselstatus, Berichtszustand.

**Rohdatei-Forensik:** Nach der Löschung durchsuchen Rohsicht und entschlüsselte Sicht alle Dateien des Datenverzeichnisses nach Kanarien-Klartext und Wrap-Nonce. Eine Prozessspeicherkopie dient nur als Positivkontrolle für Ebene A.

**Backup/Restore:** Backup vor der Löschung, Löschung, Restore (Daten-Backup mit altem und mit frischem Schlüssel-Backup). Geprüft: Der Scope bleibt gesperrt, keine lesbaren Reste. Ein Schlüssel-Backup nach Ablauf wird verweigert. Restore ohne Mirror wird ausgewiesen.

**Gleichzeitigkeit:** loom-Tests für `remember` und `forget` auf derselben Einheit und für den Epoch Seal.

**Mixed-Segmente:** Integrationstest für `BlockAll`, `RedactAndRekey`, `DeleteAll`.

**Propertytests:** Nach Seal und Hygiene enthält keine Datei die Nonce eines vernichteten Schlüssels. Kein Schlüssel hat mehr als einen Zweck.

**Berichts-Negativsuite:** Bit-Flip je Feld, Feldvertauschung, Downgrade, falscher Schlüssel, fehlende Ausschlüsse, Klasse ohne Annahmen, gefälschtes Siegel.

**Mutation:** Eine Konstante statt Messung lässt einen Test fehlschlagen (AC-2).

**Gate `claim-generator`:** Jeder Satz der Satzbibliothek ist an einen Messwert gebunden. Ein absichtlich überzogener Text lässt das Gate fehlschlagen.

**Last:** Messung der Leseleistung mit DocDek-Lookup, Padding-Overhead (≤ 12 %), Re-Wrap-Dauer. Ziele in `benchmarks/budgets.toml` (O-24).

**Orakel:** Ein unabhängiger Referenz-Prüfer (anderer Autor, andere Sprache) verifiziert Berichte im Test (INV-ORACLE-1).

---

## 19. Marktvergleich (Recherche)

| System | Ansatz | Stärke | Lücke | Übernahme in Contextra |
|---|---|---|---|---|
| Kafka/Conduktor | Schlüssel je Betroffenem, Löschung per Tombstone des Schlüssels, Envelope-Verschlüsselung | Unmittelbare Wirkung auch in Replikaten und Backups. Praxisverbreitet | Downstream-Kopien nicht erfasst. Schlüsselspeicher ist Single Point und Schwachstelle. Kein Nachweis | DocDek, Envelope; Erweiterung um Messung, Bericht, Backup-Trennung |
| Datomic | Excision (physisches Herausschneiden), nicht transaktional, im nächsten Indexierungslauf, unwiderruflich, Excise-Ereignis bleibt dauerhaft als Spur | Echte Entfernung inklusive Historie | Zeitverzögert. In Cloud nicht verfügbar, dort Empfehlung „Verschlüsseln und Schlüssel verwerfen“ | Hygiene-Phase terminiert. Spur der Löschung bleibt als Referenz-Hash |
| AWS KMS | Löschung eines Schlüssels mit Wartefenster 7–30 Tage, Abbruch möglich, danach unwiederbringlich | Schutz vor Irrtum | Fenster verzögert die Wirkung. Kein Bezug zu Flash-Resten oder Daten-Backups | `erasure_grace` (Stunden), K3-Port |
| Apple Effaceable Storage | Wenige Schlüssel in dedizierter, direkt löschbarer NAND-Zone, Schnell-Löschen eines Volumenschlüssels | Hardwarenahe Löschung | Nur für Volumenschlüssel, nicht für beliebige Anwendungsschlüssel | Leitidee für K2 (Wurzel in löschbarem Modul), keine Zusage je Plattform |
| Qdrant, Milvus, Weaviate | Tombstone und Wartung (Vacuum, Compaction, GC) | Leistung | Löschung ist Wartung, keine Frist, keine Messung (Sekundärquellen) | Frist und Messung, Index-Generationen |
| vector-forget (pgvector) | Erzwungenes REINDEX/VACUUM plus Verifikation, unsigniert, Alpha | Erkennt die Lücke | nur ein Backend, unsigniert, kein Rechtsmodell | Signierter, gestufter Bericht |
| Ghost Vectors (Preprint) | Epochenschlüssel für Vektoren, Verwerfen des Schlüssels, signierter Beleg | Quantifiziert das Risiko, Idee passt | Schlüssel-Reste im Dateisystem, Graph bleibt, Beleg ist Selbstauskunft | ArtKey, Epoch Seal, Graph-Neuaufbau, Annahmecodes |

**Befund:** Kein gefundenes System kombiniert (a) physische Bereinigung der Klartextstrukturen, (b) Schlüsselvernichtung mit Flash-Rest-Behandlung, (c) Backup-Trennung mit Frist, (d) gemessene, gestufte Berichte mit generiertem Außentext und Annahmen. Das ist die Lücke, die Contextra besetzt. Die Aussage beruht auf der begrenzten Recherche dieser Fassung und ist keine Marktanalyse.

---

## 20. Alleinstellungsmerkmal

**Löschwirkungs-Dossier statt Löschbehauptung.** Contextra erzeugt für jede Löschung ein Dossier, das

1. **je Perspektive** (§1.4) aussagt, wer nach der Löschung was noch kann,
2. **je Rechtsauffassung** (§1.3) den erfüllenden Zustand benennt,
3. **ehrlich** die Annahmen und Restfristen ausweist (Annahmecodes, `residuals`),
4. **gestuft** (Siegel 1–4) belegt, wann welcher Zustand erreicht war,
5. **abstrakt** als Bescheinigung für Betroffene und Auftraggeber nutzbar ist (§11.2),
6. **Backups** ohne Eingriff in die Backups in ein festes Zeitfenster überführt (§9).

Die Aussagen stammen aus Messung, nicht aus Zielzustand (INV-CLAIMS-1). Die Außentexte entstehen aus einer Satzbibliothek, deren Sätze an Messwerte gebunden sind.

---

## 21. Entscheidungsregister

| ID | Festlegung | Status |
|---|---|---|
| N-E27 | Schlüsselvernichtung ist Löschmaßnahme für persistente Reste (Klassen C1–C3), keine Maßnahme für Klartext-Strukturen. Ersetzt E27 aus 2.2.1 in diesem Punkt | ADR-pflichtig |
| N-E40 | DocDek je Betroffenen-Einheit, ArtKey je Index-Artefakt, KEK-Epochen, KeyTable mit Ersetzen-Semantik | ADR-pflichtig |
| N-E41 | Löschwirkungsklassen L, P, C1–C3, M mit Annahmecodes als einzige Außensprache | ADR-pflichtig |
| N-E42 | Saga-Reihenfolge nach §6.2 (Fast-Path vor Hygiene) | verbindlich |
| N-E43 | Zeitanker über qualifizierten Zeitstempel, optional, Anbieter offen (O-34) | offen |
| N-E44 | K2/K3 als Port-Stufen wieder vorgesehen (teilweise Rücknahme E27), ausgeliefert nur nach Konformitätssuite | ADR-pflichtig |
| N-E45 | Zwei-Spuren-Backup, `ErasedSetMirror`, `erased_set_retention` abgeleitet aus `key_backup_max_age` | ADR-pflichtig |
| N-E46 | `DeletionProof` V5 und `doctor verify-report` ersetzen E26 (V4 eingefroren, kein Verifier) | ADR-pflichtig |
| N-E47 | Padmé-Padding und verschlüsselte Record-Header | ADR-pflichtig |
| N-E48 | Mixed-Policy mit Standard `BlockAll` | verbindlich |

---

## 22. Offene Punkte

| ID | Frage | Hinweis |
|---|---|---|
| O-12 | Dauer und Zulässigkeit von `Restricted` | Rechtsfrage |
| O-15 | Standard-`checkpoint_policy` | Standard `Rewrite`, Haltefrist offen |
| O-24 | Budgets (Recovery, Re-Wrap, Padding) | Benchmarks |
| O-34 | Zeitstempel-Anbieter mit Offline-Verifikation | Auswahl |
| O-35 | Zentrales Löschregister als Mirror-Ziel | Betreiberentscheid |
| O-36 | Echtes Commitment-Schema statt Referenz-Hash | optional |
| O-37 | Spike S-K2: welche Plattformen erfüllen die Konformitätssuite für K2 | Verifikation |
| O-38 | Spike S-HRD: plattformspezifische Härtung | Verifikation |
| O-39 | Standardwerte `kek_epoch_len`, `key_backup_max_age`, `purge_rebuild_max_days` | Rechtsprüfung P-06, P-11 |
| O-40 | Kosten des Padmé-Paddings und des Re-Wrap | Benchmarks |
| O-41 | Status DIN 66398 und SDM-Version | Prüfung |

---

## 23. Auswirkung auf die Gesamtspezifikation 2.2.1

Diese Tabelle ersetzt die genannten Stellen. Alles Übrige in 2.2.1 bleibt.

| Stelle in 2.2.1 | Wirkung |
|---|---|
| §0.5, §2.3 (Nicht-Ziele „Schlüsselvernichtung als Löschmaßnahme“, „K2/K3“, „Epochen“) | ersetzt: Schlüsselvernichtung ist Löschmaßnahme für persistente Reste, K2/K3 als Port vorgesehen, KEK-Epochen und Epoch Seal sind Kern |
| §2.6 (Ehrliche Grenzen), §12.11 (Claim-Taxonomie, Außenformulierung) | ersetzt durch §2 und §15 dieser Datei |
| §3.4 `INV-SHRED-1` | bleibt entfallen. Neue Invarianten siehe unten |
| §5.2 (`KmsProvider`, `ErasureJournal` usw.) | ergänzt um `EffaceableKeySlot`, `ErasedSetMirror`, `RecipientLedger` als Sicherheitsports |
| §7.2 (Format `MFKS`, Sidecars) | ergänzt: KeyTable-Format, Schlüssel-Backup-Format, Klartext-Header entfallen |
| §7.9 (Sidecar-Liste), Regel 5 (Backup) | ersetzt: Backup nach §9 |
| §7.10 (Notlöschpfad) | bleibt. Der Notfall-Sperr-only-Pfad bleibt Klasse L |
| §9.3 (HNSW-Löschung), §10.1 (Text) | ergänzt durch §8 |
| §12.2 (Primitive, Schlüsselhierarchie, Verwahrung) | ersetzt durch §4 |
| §12.3 (Zweckregister) | ersetzt: Zwecke DocDek, ArtKey, KEK-Epoche, K_ref (statt K_commit) |
| §12.4, §12.5 (Schichten, Löschstufen) | ersetzt durch §7 und §6 |
| §12.7 (Löschbericht V4, eingefroren), REQ-PRF-5/6, E26 | ersetzt durch §11 (V5, Verifier) |
| §12.8 (Löschsaga) | ersetzt durch §6 |
| §12.9 (Restore-Guard, Horizont), REQ-RST-3, INV-RESTORE-HORIZON-1 | ersetzt durch §9 (Horizont gilt für Schlüssel-Backups) |
| §12.10 (Commitments, Pseudonym-Ablauf) | ersetzt durch §12 |
| §12.12 (AC-Kriterien) | ergänzt: AC-1 bis AC-20 gelten weiter. Neu: AC-21 (Epoch Seal), AC-22 (Backup-Trennung), AC-23 (Mixed-Segmente), AC-24 (Claim-Generator), AC-25 (Nadelscan) |
| §13.8 (Checkpoints, Backup) | ersetzt durch §9 |
| §15.5 (Startwerte Löschpfad) | ergänzt: `kek_epoch_len` 30 Tage, `key_backup_max_age` 30 Tage, `erasure_grace` 0 h (Obergrenze 72 h), `purge_rebuild_max_days` 14, `min_record_bytes` 256, `ref_epoch_len` 90 Tage, `erased_set_retention` 90 Tage |
| §17.3 (Gates) | ergänzt: `surface-coverage`, `claim-generator`, `key-remnant` |
| §19 (Katalog) | REQ-DEL-N1…N12 ersetzt, N13…N27 ergänzt. REQ-DEL-016, -033, -035 entsprechend angepasst |
| §20.1 (E26, E27) | ersetzt durch N-E27, N-E44, N-E46 |
| §25 (P-18) | P-18 ist wieder gültig (K3) |

**Neue Invarianten:**
- `INV-EPOCH-SEAL-1`: Eine Löschung erhält die Klasse C erst nach einem Epoch Seal, der ihre Schlüsselvernichtung umfasst.
- `INV-BACKUP-SPLIT-1`: Kein Daten-Backup enthält Schlüsselmaterial. Jedes Schlüssel-Backup trägt ein Ablaufdatum.
- `INV-ARTKEY-1`: Jedes Index-Artefakt hat genau einen ArtKey. Beim Neuschreiben wird der alte vernichtet.
- `INV-CLAIM-BIND-1`: Jeder Satz eines Außentextes ist an einen gemessenen Wert gebunden.
- `INV-SURFACE-1`: Jede persistente Strukturklasse ist einer Löschschicht oder einem Ausschluss zugeordnet.

---

## 24. Umsetzungsplan und Schuldenfreiheit

### 24.1 Prinzipien

1. **Eine Typfamilie je Konzept.** Löschtypen existieren nur in `crypto`/`ports`.
2. **Ports statt Verzweigungen.** K2 und K3 sind Port-Implementierungen. Bis zur Konformität liefern sie `CapabilityUnsupported`.
3. **Kein Feature ohne Test (A10).** Jedes Arbeitspaket endet mit Crash-Test, Forensik-Test und Gate.
4. **Formate vor Code.** Jedes neue Format (KeyTable, Schlüssel-Backup, V5-Bericht) bekommt Magic, Version, Golden-File und Migrationspfad.
5. **Keine Stubs im Produktionscode** (INV-WIRING-2).
6. **Kein Panic in Produktion** (P7).

### 24.2 Arbeitspakete

| WP | Inhalt | Abnahme |
|---|---|---|
| L-1 | Typen und Ports: `EffaceableKeySlot`, Berichtstypen V5, Klassen, Annahmecodes, Satzbibliothek | Typestate-Tests, Gate `claim-generator` |
| L-2 | Schlüsselhierarchie: KEK-Epochen, DocDek, ArtKey, KeyTable, `KeyRegistry` | Rohdatei-Forensik, Propertytests |
| L-3 | Record-Format: Padmé, verschlüsselte Header, Format-Registereinträge | Format-Test, Golden-Files |
| L-4 | Saga-Fast-Path (Schritte 1–7) mit Journal, Karenz, Restricted | Crash-Matrix |
| L-5 | Epoch Seal inklusive Passphrasen-Rotation (K1) | Crash-Matrix, Nadelscan-Test |
| L-6 | Index-Artefakte: ArtKey, Text-Segment-Neuschreibung, Graph-Neuaufbau, Generationswechsel | Scanner, Chaos |
| L-7 | Scanner und Messungen (Inventar, Nadel, Kanarien, Restmetadaten-Selbsttest) | Positivkontrollen |
| L-8 | Zwei-Spuren-Backup, Restore-Guard, `ErasedSetMirror` | Backup-/Restore-Tests, Disaster-Recovery-Test |
| L-9 | Bericht, Siegelkette, Bescheinigung, `doctor verify-report`, Zeitanker | Berichts-Negativsuite, Referenz-Prüfer |
| L-10 | Mixed-Policy, `RecipientLedger`, Fristen und Alarme | Integrationstests |
| L-11 | Härtung und `doctor` (Core-Dumps, Swap, mlock) | Plattformtests (Spike S-HRD) |
| L-12 | Gate `surface-coverage`, Dokumentationsgates, ADRs | Gate grün auf Merge-Ergebnis |
| L-13 (später) | K2- und K3-Implementierungen nach Konformitätssuite (Spike S-K2) | Konformitätssuite |

Reihenfolge: L-1, L-2, L-3, L-4, L-7, L-5, L-6, L-8, L-9, L-10, L-11, L-12. L-13 nach L-12 und der Entscheidung zu O-37.

### 24.3 Definition of Done je Arbeitspaket

- Build, Clippy, Tests, Miri (wo relevant) grün auf dem Merge-Ergebnis.
- Crash-Test am neuen Übergang.
- Forensik-Test (keine Klartext- oder Schlüsselreste in Dateien).
- Format-Golden-Files und Migrationstest, falls Format betroffen.
- Eintrag im Entscheidungsregister und, falls Garantie betroffen, ADR.
- Keine neue `unwrap`-/`panic`-Stelle, keine neue Abhängigkeit auf Rand-Crates.

---

## 25. Quellen und Prüfstatus

Prüfstatus: ✔ im Volltext gelesen, ◐ nur über Sekundärquelle oder Suchauszug, ○ nicht geprüft.

| Quelle | Status |
|---|---|
| EDPB Guidelines 02/2025 v2.0 (Blockchain), edpb.europa.eu | ✔ |
| EDPB CEF-Bericht 2025 zum Recht auf Löschung, edpb.europa.eu | ✔ |
| NIST SP 800-88 Rev. 2 FAQ (16.07.2026), csrc.nist.gov | ✔ |
| NIST SP 800-88 Rev. 2 (Haupttext) | ◐ |
| EDPB Guidelines 02/2026 (Anonymisierung, Entwurf) | ◐ (Hogan Lovells, Bird & Bird, Kanzleiberichte, Suchauszug des PDF) |
| EDPB Guidelines 01/2025 (Pseudonymisierung) | ◐ |
| SDM Baustein 60 V1.0a, datenschutz-mv.de | ✔ (Auszug) |
| BayLfD 30. Tätigkeitsbericht Nr. 12.5 | ✔ (Zitat über Boorberg) |
| Ghost Vectors, arXiv 2606.18497 | ✔ |
| Datomic-Dokumentation und Blog zu Excision | ✔ |
| AWS KMS Dokumentation zur Schlüssellöschung | ✔ |
| Apple Platform Security (Effaceable Storage) | ◐ (Glossar, Abschnitt zu Data Protection) |
| Conduktor-Dokumentation und Blog zu Crypto-Shredding | ◐ (Herstellerangaben) |
| Qdrant-Optimizer-Dokumentation | ✔ (Auszug) |
| Milvus, Weaviate, Qdrant Standardwerte (Blog particula.tech) | ◐ (Sekundärquelle, vor Nutzung prüfen) |
| vector-forget (GitHub) | ✔ (README) |
| PURBs/Padmé, PoPETs 2019 | ✔ (Zusammenfassung) |
| SNIA zu ISO/IEC 27040:2024 und CE | ◐ |
| EuGH C-413/23 P (04.09.2025) | ◐ |
| AG Lörrach 3 C 1099/24, OLG Dresden 4 U 940/24 | ◐ |
| OLG Dresden 4 U 292/25 (Anonymisierung als Minus) | ◐ (nur Sekundärquelle) |
| DSB Österreich, Bescheid 05.12.2018 | ◐ |
| eIDAS Art. 41, 42; BNotK Zeitstempel-PDS | ◐ |
| DIN 66398 (Status) | ○ (Datenbank-Eintrag „Withdrawn“) |
| IEEE 2883, ISO/IEC 27555 | ○ |

---

## 26. Fazit

Die rechtliche Wirkung einer Löschung kann die Software nicht garantieren. Sie kann den Zustand herstellen, den die plausiblen Auffassungen verlangen, ihn je Perspektive belegen und ehrlich benennen, was sie nicht erreicht. v3.1 schließt dazu die Lücken von v3.0: Die Schlüsselvernichtung wird durch den Epoch Seal und die Custody-Stufen belastbar. Die Saga wirkt in Minuten statt in Wochen. Backups verlieren ihre Lesbarkeit in einem festen Fenster. Index-Reste sind terminiert. Mehrfachbezug ist geregelt, und der Bericht trennt Technik, Rechtsstatus und Annahmen.

Ob ein konkreter Vorgang rechtlich als Löschung gilt, entscheidet der Verantwortliche auf Grundlage dieses Dossiers, der offenen P-Punkte und der Rechtslage. Vor jeder Außenkommunikation und vor der Nutzung der ◐- und ○-Quellen ist die fachanwaltliche Prüfung (P-14) erforderlich.
