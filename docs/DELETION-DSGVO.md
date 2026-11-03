# Löschung personenbezogener Daten: technische Einordnung

Nicht normativ. Maßgeblich ist [`SPEC.md`](SPEC.md) Abschnitt 6 (Rechts- und Haftungsrahmen), Abschnitt 16 (Betreiber-Checkliste) und Anhang D (Restrisiken). Dieses Dokument fasst Abschnitt 6 zusammen.

> **Keine Rechtsberatung.** Dieses Dokument beschreibt die technische Wirkung von Contextra und die Grenzen seiner Aussagen. Die rechtliche Wertung trifft der Betreiber, gegebenenfalls mit Fachberatung.

## 1. Zulässige Außenaussage

Die einzige zulässige Außenaussage zum Löschen steht in [`README.md`](../README.md) und in SPEC 6.4. Andere Formulierungen sind nicht zulässig (HR-5). Die in SPEC 6.4 aufgeführten Begriffe sind verboten; der Wortlaut-Lint (T-LINT-1) prüft sie in allen Produktartefakten.

Contextra beansprucht den Begriff Anonymisierung für das Löschergebnis nicht (E-L13).

## 2. Rolle von Contextra

- Verantwortlicher im Sinne der DSGVO ist der Betreiber (Art. 5 Abs. 2, 17, 18, 24, 25, 32). Die Pflichten liegen bei ihm.
- Contextra läuft lokal und sendet keine Daten. Solange die harten Regeln HR-1 bis HR-4 gelten, ist Contextra bei bestimmungsgemäßem Betrieb weder Verantwortlicher noch Auftragsverarbeiter (E-L6). Diese Einordnung ist im Schrifttum überwiegend vertreten und einzelfallabhängig. Sie ist Frage 7 des Gates G-LEGAL.

## 3. Technische Wirkung

Ein Löschvorgang (`erase`) umfasst folgende Schritte (SPEC 11.2):

- **Gate.** Ab der persistierten Sperre liefern alle Lesepfade, auch Zeitreisen, `Erased`. Schreibzugriffe in das Ziel werden abgewiesen (T-GATE-1, T-VER-1).
- **Schlüsselvernichtung.** Der Schlüssel der höchsten Ebene des Ziels wird aus dem Tresor entfernt (Slot `T`, `C`, `S` oder `D`). Das Fehlen wird geprüft (T-VAULT-1 bis T-VAULT-4). Die Zeit bis `Gated` hängt nicht von der Zielgröße ab (P16).
- **Purge.** Die WAL wird nie umgeschrieben; es werden ganze Segmente bis zum Flush-Punkt entfernt. SSTables, Textsegmente und Vektor-Snapshots werden ohne Zielrecords neu geschrieben. Dateischlüssel alter Dateien werden genullt, bevor die Datei entfernt wird (D-73, D-85).
- **Index.** HNSW-Slots erhalten vor der Sperre ein Surrogat, im Purge-Fenster folgt die Kantenreparatur und bei Bedarf ein deterministischer Rebuild (D-77).
- **Messung.** Jede Schicht wird mit Positivkontrolle gemessen: MK1 (Struktur), MK2 (Rohscan auf Klartext-Needles), MK3 (Schlüsselabwesenheit). Im Feld misst die Bibliothek selbst; unabhängig wiederholbar ist die Messung mit `ctx-verify scan` und im Erasure Lab (D-72).
- **Bericht.** Ed25519-signiert, kanonisches JSON, ohne Inhalte und ohne Klartext-Kennungen. Er listet je Schicht `key_coverage` und die zutreffenden Restrisiken (SPEC 11.7, F.10, Anhang D).
- **Restore-Guard.** Ein Backup, das gelöschte Daten zurückbringt, wird beim Öffnen erkannt. Löschungen werden dann wiederholt (D-74, T-RESTORE-1 bis T-RESTORE-3).

Die Wirkung hat Grenzen. Wo sie greift, weist der Bericht aus. Die Grenzen sind in SPEC Anhang D beschrieben (z. B. `FlashRemanence`, `FsJournalOrCow`, `WrappedKeyRemanence`, `ExternalCopies`, `ProcessMemory`).

## 4. Anforderungen und Mechanismen (Auszug aus SPEC 6.5)

Die vollständige Matrix mit 23 Zeilen steht in SPEC 6.5. Auszug:

| Anforderung (Orientierung) | Mechanismus | Restrisiko im Bericht |
|---|---|---|
| Unverzüglich, Monatsfrist (Art. 17 Abs. 1, Art. 12 Abs. 3) | Saga mit Wiederholung, `erase_sla`, Status, Alarm | `ReplicaWindow` |
| Unkenntlichmachen (SDM M60, ErwGr 26) | Klassen K und P, Dateischlüssel-Vernichtung | `FlashRemanence`, `WrappedKeyRemanence` |
| Einfache OS-Löschbefehle genügen nicht | Rewrite, Dateischlüssel-Vernichtung, Unlink, Fehlen geprüft | `FsJournalOrCow` |
| Alle Versionen und Kopien | Purge aller Versionen, Snapshot-Widerruf | `ExternalCopies` |
| Rechenschaft (Art. 5 Abs. 2, Art. 24) | signierter Bericht, Feldscanner, `ctx-verify` | `PseudonymCommitment` |
| Aufbewahrung (Art. 17 Abs. 3), Einschränkung (Art. 18) | `RetentionAttestation`, Gate-Variante `Restricted` | Prüfung beim Betreiber |
| Embeddings vorsorglich personenbezogen | Vektoren im Umfang, verschlüsselter Snapshot, Surrogat, Kantenreparatur | `IndexTopologyInference` |
| Nach Restore erneut löschen | Restore-Guard mit Ledger | Backup-Inhalte außerhalb |
| Datensparsames Protokoll | Commitments mit Epochenrotation | `PseudonymCommitment` |
| Aufbewahrungspflichten, GoBD (§ 257 HGB, § 147 AO) | `RetentionCode`, `Restricted`; Zuordnung im DSFA-Baustein | Prüfung beim Betreiber |
| BSI CON.6, ISO/IEC 27001:2022 A.8.10 | Abbildung im DSFA-Baustein (Anhang G) | – |

## 5. Rechtsentscheidungen (SPEC 6.6)

Die Entscheidungen folgen dem strengsten Auslegungsmaßstab. Sie können nach einer juristischen Prüfung nur gelockert, nicht verschärft werden, wenn die Technik unverändert bleibt.

| ID | Frage | Entscheidung (Kurzfassung) |
|---|---|---|
| E-L1 | Genügt Umschreiben und Entfernen der Altdateien? | Pflichtmaßnahme, nie allein als ausreichend dargestellt |
| E-L2 | Ist Schlüsselvernichtung Löschung, Anonymisierung oder Einschränkung? | Contextra behauptet keine dieser Wertungen; der Bericht weist den Zustand als Tatsache aus |
| E-L3 | Sind Embeddings, Slot-Nummern, Kanten und Tags personenbezogen? | Vorsorglich ja |
| E-L4 | Sind Bericht, Journal und Ledger personenbezogen? | Pseudonym, daher begrenzt; Journal und Bericht nach Commitment-Epoche, Ledger nach `backup_retention` |
| E-L5 | Sind aggregierte Textstatistiken personenbezogen? | Sie werden nach jedem Segment-Rewrite neu berechnet |
| E-L6 | Rolle von Contextra? | Weder Verantwortlicher noch Auftragsverarbeiter, solange HR-1 bis HR-4 gelten |
| E-L7 | Aufbewahrungspflichten? | Betreiber erklärt sie per `RetentionAttestation`; Contextra führt kein Fristenwerk |
| E-L8 | Wortlaut der Checkliste und der Berichtstexte? | Festgelegt in SPEC 16 und Anhang D; kein Rechtsrat |
| E-L9 | ProdHaftG und CRA bei freier Software? | Contextra behandelt sich als unterliegend (strengste Annahme) |
| E-L10 | Wirkung von Haftungsausschlüssen? | Werden nicht als Schutz verbucht (HR-9) |
| E-L11 | Reichweite der Signatur? | Beweist Integrität und Herkunft aus der Installation, nicht die Wahrheit des Inhalts |
| E-L12 | Liste der `RetentionCode` | Geschlossen, kein Freitext (7.3) |
| E-L13 | Anonymisierung als Anspruch? | Nein |
| E-L14 | Wie wird „unverzüglich“ bestimmt? | Contextra misst Zeiten und weist Verletzungen von `erase_sla` aus; die Frist bestimmt der Betreiber |
| E-L15 | Sind entkoppelte Textpostings und Graphkanten Rückstände mit Personenbezug? | Vorsorglich ja, bis zum Purge bzw. Rebuild; im Bericht ausgewiesen |
| E-L16 | Sind Tags in Pfaden personenbezogen? | Pseudonym; Klartextnamen sind verboten (D-75) |

## 6. Offene Fragen (Gate G-LEGAL, SPEC 6.7)

Bis zur schriftlichen Klärung darf Contextra keine Löschaussage über die Außenaussage in 6.4 hinaus treffen. Offen sind:

1. Qualifikation der Schlüsselvernichtung und Haltung der zuständigen Aufsichtsbehörde.
2. Relativer Personenbezug nach EuGH C-413/23 P.
3. Wirksamkeit, Umfang und Remanenzrisiko des dateibasierten Tresors.
4. Einordnung von Embeddings, Slot-Nummern und Graphtopologie.
5. Backups und Horizont des Restore-Guards.
6. Zulässigkeit, Inhalt und Dauer von Commitments in Bericht und Ledger.
7. Rolle des Herstellers bei lokal installierter Software.
8. Aufbewahrungspflichten je Branche.
9. Freigabe der Checkliste (SPEC 16) und der Berichtstexte.
10. Prüfung jeder Werbeaussage auf UWG und BGB.
11. Anwendung der EDPB Guidelines 02/2026 auf Tags, Commitments, Ledger-Bereiche, Postings und Graphtopologie, nach Veröffentlichung der Endfassung.
12. Anwendbarkeit von § 35 BDSG auf automatisierte RAG-Verarbeitung.
13. Reichweite des neuen ProdHaftG für B2B-Kunden und die Ausnahme für freie Software.
14. Zweckbindung und zulässige Dauer des Ledgers (`backup_retention`).

## 7. Rechtsstand laut SPEC 5.1.0

Die Angaben stammen aus der SPEC und sind dort mit Quellen versehen. Sie können sich ändern und sind vor einer Verwendung erneut zu prüfen.

- CRA, Art. 14 (Meldepflichten): gelten seit dem 11.09.2026. Übrige Pflichten ab dem 11.12.2027.
- ProdHaftG in der Fassung nach RL (EU) 2024/2853: vom Bundestag am 08.10.2026 beschlossen; anwendbar auf Produkte, die ab dem 09.12.2026 in Verkehr gebracht werden.
- EDPB Guidelines 02/2026 (Anonymisierung): Entwurf. Die Konsultation endete am 30.10.2026; die Endfassung steht aus.

## 8. Betreiberpflichten

Die vollständige Checkliste steht in SPEC 16. Kern:

- Rechtsgrundlage und Aufbewahrungspflichten prüfen; `RetentionAttestation` wahrheitsgemäß setzen.
- Dokumente beim Anlegen dem Betroffenen zuordnen (`DocRef.subject`).
- Bei Betroffenen-Anfragen `Subject` löschen und den Status bis `Completed` beobachten.
- Bericht prüfen (`ctx-verify`) und aufbewahren.
- Externe Sicherungen, Kopien und Empfänger selbst bereinigen bzw. informieren (Art. 17 Abs. 2, Art. 19).
- Flash- und Dateisystem-Remanenz bewerten; Volume-Verschlüsselung mit Schlüsselvernichtung bei hohem Schutzbedarf.
- Swap verschlüsseln oder deaktivieren; Coredumps abschalten.
- Passphrase verwalten. Es gibt keinen Schlüssel-Escrow (HR-3).
- Den DSFA-Baustein (Anhang G) übernehmen und anpassen.
