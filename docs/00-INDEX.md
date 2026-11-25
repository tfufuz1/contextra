# CONTEXTRA-SPEC R16 — Index der zerlegten Spezifikation

| | |
|---|---|
| Status | R16, normativ nach Freigabe als Gate G-0. Ersetzt `SPEC.md` R14 vollständig (R14 wandert nach `docs/archive/SPEC-R14.md`). |
| Zuschnitt | **Nicht pro Crate, sondern pro Zyklus und für das Gesamtsystem.** 8 Systemspezifikationen (`SYS-*`) und 8 Zyklenspezifikationen (`CYC-*`). |
| Prüfgrundlage | Statische Prüfung des Repositories (HEAD `060452f`, nicht kompiliert) gegen R14. Der Speicherkern `contextra-storage` wird neu entwickelt: vorläufige Implementierungslücken sind **nicht** Gegenstand; Gegenstand ist die finale Planung. |
| Sprache | Prosa deutsch, Bezeichner englisch. MUSS / DARF NICHT / SOLL / KANN im Sinn von RFC 2119. |

## 1. Lesereihenfolge und Zuständigkeit

| Datei | Gegenstand | Ersetzt R14 | Hauptgate |
|---|---|---|---|
| `SYS-1-ARCHITEKTUR.md` | Topologie, Ebenen, Ports/Features, Threading, Sperrränge, Fehlermodell, Abhängigkeiten, Toolchain | §1.4, §2, §10.1 | G-0 |
| `SYS-2-DATENMODELL-FORMATE.md` | Datenmodell, Kombinationsmatrix, **alle Byteformate**, Freeze, Migration | §3, §4 | G-F |
| `SYS-3-KERNFASSADE.md` | Öffentliche, synchrone Kern-API (Signaturen, Semantik) | verstreut | G-W, G-R |
| `SYS-4-KONFIGURATION-RESSOURCEN.md` | **Einziges Zahlenregister**, Speicherbudget, Admission, Validierung, Metrikkatalog | §3.4, §3.7, §15 | G-0 |
| `SYS-5-SICHERHEIT-COMPLIANCE.md` | Bedrohungsmodell, Profile, Egress, Mandant/Principal/`TenantPolicy`, Betroffenenrechte, Restexposition | §1.3, §8.11, §11.9, §11.15 | G-E, G-R4 |
| `SYS-6-VERIFIKATION-GATES.md` | Phasen, Gates, CI, Fs-Konformanz, Formalmodelle, Simulation, Orakel, Messprotokoll, H-Zuordnung | §12, §13, §16 | alle |
| `SYS-7-REGISTER.md` | Invarianten, Entscheidungen (`DEC-41…`), Lückenregister R14→R16, Glossar | §14, §17, §18, Anh. A–D | G-0 |
| `CYC-1-COMMIT.md` | Schreibzyklus: `apply`, Sequencer, Syncer, Guards, Idempotenz, Backpressure | §5 | G-W |
| `CYC-2-LESEN-ABFRAGE.md` | Lesezyklus und Abfragezyklus: View, Resolve, Page, Body, Query-Plan, Fusion, Budget | §6, §11.10–11.11 | G-R, G-R3 |
| `CYC-3-LOESCHUNG.md` | Löschzyklus: Ticket, ER0–ER7, Fristen, Restriction/Link, Receipt, Retention | §8 | G-E |
| `CYC-4-SCHLUESSEL.md` | Schlüsselzyklus: Hierarchie, Provider, Epochen, Tomb/Partition/Entry-Schlüssel | §8.4 | G-E |
| `CYC-5-WARTUNG.md` | Wartungszyklus: Flush, Compaction, Blob-GC, Purge-Ausführung, Scrub, Governor, Catalog-Rollover | §4.8, §5.4, §8.6 | G-M |
| `CYC-6-INSTANZ-LEBENSZYKLUS.md` | Open, Recovery, Close, Checkpoint, Restore, Backup-Horizont, DiskFull, Poison | §5.6, §9 | G-E |
| `CYC-7-INGEST-PROJEKTION.md` | Publish-Record, Feed, Partitionen, Generationen, Modi A/B/C, Pseudonymisierung, Analyzer | §11.1–11.9, §11.12–11.13 | G-R1…G-R4 |
| `CYC-8-VERSIONIERUNG.md` | Branches, Diff, Merge, Historie | §7, §11.14 | G-V |

**Rangfolge bei Widerspruch:** `SYS-7` Invarianten > `CYC-3`, `CYC-6` > übrige `CYC-*` > `SYS-*` > `docs/archive/*`. Innerhalb gleichen Ranges gilt die speziellere Datei. Ein Zahlenwert gilt ausschließlich, wie er in `SYS-4` steht; andere Dateien zitieren ihn nur per Name.

## 2. Aufbau jeder Zyklenspezifikation

1 Zweck und Grenzen · 2 Akteure und Zustandsautomat · 3 Ablauf (nummeriert, mit Vor-/Nachbedingung) · 4 Fehler- und Absturzverhalten (je Schritt) · 5 Parameter (Verweis `SYS-4`) · 6 Entscheidungen und geschlossene Lücken · 7 Abnahme (benannte Tests, Gate). Jede Datei endet mit „Offene Punkte“. Dort stehen ausschließlich **externe Vorbedingungen** (Recht, Hardware, Betrieb), nie Spezifikationslücken.

## 3. Änderungsregeln für die Suite

1. Jede Regel steht genau einmal. Verweise statt Kopien.
2. IDs (`INV-*`, `DEC-*`, `GAP-*`, `H-*`, `RACE-*`, `FIX-*`) sind stabil und werden nie wiederverwendet.
3. Änderungen an Formaten (`SYS-2` §4–§7) nach Gate G-F sind Migrationen (`SYS-2` §9).
4. Pro Crate gilt weiter: `API.md`, `PLAN.md`, `SPEC.md` synchron halten. Crate-`SPEC.md` enthält nur Modulinvarianten und verweist hierher; es wiederholt keine Systemregel.
5. CI-Test `spec_refs` (SYS-6 §3) bricht, wenn eine zitierte ID im Register fehlt oder eine Datei auf eine gelöschte Datei verweist.

## 4. Was die Suite gegenüber R14 *nicht* ändert

Mission und Ziele (M-1…M-9, Z-1…Z-22), Nicht-Ziele NZ-1…NZ-24, die 14 Crates, die Löschreihenfolge G1<G2<G3<G4<G5<KeyDestroyIntent<`destroy_many`<Purge<Unlink<`UnlinkAck`<`Done`, Cluster=Löschdomäne, Ein-Wahrheits-Commit, kein 2PC, XChaCha20-Poly1305/HMAC-SHA256/HKDF-SHA256/CRC32C, synchroner Kern. Änderungen sind im Lückenregister (`SYS-7` §3, `GAP-01…GAP-58`) einzeln mit Begründung aufgeführt.

## 5. Offene Punkte (extern)

Rechtsprüfung der Fristen und der Aussage „kryptografisch gelöscht“ im Einsatzfall · zweiter Reviewer für G-0 · `Strong`-Anker und HSM/KMS-Provider für den Produktivbetrieb · Windows-Freigabe nach Fs-Konformanzlauf. Das sind Vorbedingungen, keine Lücken der Suite.
