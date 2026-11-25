# SYS-5 — Sicherheit, Compliance, Mandant, Betroffenenrechte

Ersetzt R14 §1.3, §8.11, §11.9 (Policy-Teil), §11.15. Abhängigkeiten: SYS-1 (Ports), SYS-2 (Formate), SYS-4 (Zahlen), CYC-3/CYC-4 (Mechanismen). Gate: G-E, G-R4.

**Grenze.** Dieses Dokument legt fest, *was zugesichert wird, wer es prüft und was offengelegt wird*. Mechanismen (Löschprotokoll, Schlüsselzyklus, Pseudonymisierung) stehen in den Zyklen. Ein Profil ist kein Rechtsurteil; die Suite macht keine Rechtsaussage zu Art. 17 DSGVO (NZ-14).

## 1. Schutzziele

| ID | Ziel | Träger |
|---|---|---|
| SZ-1 | Vertraulichkeit ruhender Daten gegenüber Dateileser und Datenträgerbesitzer | AEAD-Siegel, Schlüssel nur über `KeyProvider` |
| SZ-2 | Integrität und Rollback-Erkennung | Ketten-MAC, Anker, Ledger |
| SZ-3 | Nachweisbare Löschung | CYC-3, Receipt |
| SZ-4 | Mandantentrennung | mandantenreine Partitionen, `tenant_tag`, `TenantPolicy` |
| SZ-5 | Kontrolle über Abfluss an externe Modelle | `EgressPolicy` |
| SZ-6 | Keine Inhalte in Telemetrie und Fehlern | SYS-4 §7, SYS-1 §6 |

## 2. Bedrohungsmodell

| ID | Angreifer/Fehler | Gegenmaßnahme | Restrisiko |
|---|---|---|---|
| AT-1 | Leser des Datenverzeichnisses ohne Schlüssel | AEAD (SYS-2 §3) | Klartext-Metadaten (§8 Nr. 6) |
| AT-2 | Datenträgerbesitzer ohne Schlüssel | Schlüssel nur im Provider (CYC-4) | Provider selbst |
| AT-3 | Datei-/Index-Manipulation | Ketten-MAC, AAD mit Zufalls-UIDs (DEC-58), Katalogkette, schlüsselgebundene Index-MACs; keine statischen Schlüssel | Austausch ganzer, konsistenter Kopien ⇒ AT-5 |
| AT-4 | Stale oder bösartiger Index | nur versiegelte `SegmentIo`-Wege; `filter_visible` vor Ausgabe (`INV-VIS-1`); Orakel `O-Segments`/`O-Files` | |
| AT-5 | Rollback auf ältere Kopie, um Gelöschtes zurückzubringen | Ledger + monotoner Anker + Restore-Reconcile + Backup-Horizont (CYC-6 §6, §7) | Anker mit zurückgesetzt ⇒ nur kryptografischer Schutz |
| AT-6 | Löschwunsch gegen aktive Restriction | Prüfung auf Aufruferthread, Revalidierung im Sequencer, Purge, `cluster_state`-Edit (CYC-3 §7) | |
| AT-7 | Seitenkanal über globale Index-Metadaten (seltene Komposita, Filterlatenz) | mandantenreine Partitionen, Neuaufbau aus lebenden Dokumenten, keine Inhalte in Metriken (CYC-7 §5, §7) | Zeit zwischen Gate und Rebuild: nur Maske |
| AT-8 | PII-Abfluss an Cloud-LLMs; Prompt-Injection über Inhalte | `EgressPolicy` (§5), Pseudonymisierung (CYC-7 §9), Trust-Label; Injection-Filter beim Client/Plugin | Qualität des Filters |
| AT-9 | Bösartige Uploads (PDF, ZIP, Office, HTML) | Parsing außerhalb des Workspace; Contextra nimmt nur extrahierten Text an (§6) | |
| AT-10 | Mandantenübergreifender Leak durch Engine-Fehler | Kern kennt keinen Mandanten; `tenant_tag` geheim und zufällig; Differentialtests (§3 Nr. 6) | Fehler in `TenantPolicy` |
| AT-11 | Replay einer alten, legitimen `lift`-Signatur | `request_digest` bindet `restriction_id`, `cluster`, `policy_epoch` und Nonce (CYC-3 §7) | |
| AT-12 | Zeitmanipulation (Uhrsprung), um Fristen zu strecken oder zu verkürzen | `clock_floor`, `max_clock_skew`, `ClockUncertain` ⇒ fail-closed (CYC-3 §6) | Dauerhaft falsche Uhr ohne Sprung |
| AT-13 | Seitenkanal über Fehlermeldungen (existiert Cluster/Person?) | `UnknownCluster` und `Erased` unterscheiden sich nur für Aufrufer mit Cluster-Kenntnis; Engine übersetzt für Endnutzer in einheitliches „nicht gefunden“ (§3 Nr. 5) | Timing: nur grobe Angleichung |
| F-1…F-9 | Torn Write, Fsync-EIO, ENOSPC, Bit-Rot, Schlüsselverlust, Crash im Löschprotokoll, Retry-Sturm, Index-Lag | SYS-1 §6, CYC-1, CYC-6 | |

AT-11, AT-12, AT-13 sind in R14 nicht enthalten; GAP-27.

## 3. Mandant, Principal, `TenantPolicy`

Der Kern kennt keinen Mandanten (NZ-16). `engine::collection` und `engine::compliance` setzen Mandantentrennung durch. Dieser Abschnitt legt die dafür nötigen Begriffe fest, die R14 voraussetzte, aber nirgends definierte (GAP-29).

```rust
pub struct TenantId(pub u128);                 // Engine-intern, in Registry, nie im Kern
pub struct TenantTag(pub [u8; 16]);            // zufällig bei Tenant-Anlage (Entropy), geheim, im Tenant-Record (cluster-sealed System-Space der Engine)
pub struct Principal { pub subject: PrincipalId, pub tenant: TenantId, pub roles: RoleSet, pub auth_strength: AuthStrength }
pub struct TenantPolicy {
    pub profile: ComplianceProfile,
    pub egress: EgressRules, pub pii: PiiRules,                    // Klassifikationsregeln, Quarantäne-Politik
    pub pseudonym_scope: PseudonymScope, pub placeholder_retention: PlaceholderRetention,
    pub retention: RetentionRules,                                 // je DataClass: Dauer → `due`
    pub lift_policy: LiftPolicy, pub restriction_owners: Vec<PrincipalId>,
    pub quotas: TenantQuotas,                                      // Cluster, Bytes, Partitionen
}
```

Regeln:
1. Jede Engine-API nimmt einen `Principal`. Ohne gültigen `Principal` ⇒ `Unauthorized` (Engine-Fehler, nicht Kernfehler). Der Kern sieht nur `owner: [u8;16]` (Restriction) und `created_by` (Branch), beide pseudonym aus `PrincipalId`.
2. `TenantTag` entsteht einmal je Tenant aus `Entropy`, wird nie aus `TenantId` abgeleitet und nie in Logs oder Metriken geführt. Rotation ist nicht vorgesehen (die Packgruppe ist Teil des Datenlayouts); ein Tenant-Wechsel eines Clusters existiert nicht.
3. `tenant_slot` (Identity-Directory, CYC-7 §9) **ist** der `TenantTag`; eine zweite Kennung gibt es nicht (R14 ließ `tenant_slot` offen, GAP-28).
4. `canonical_person_id` = UTF-8-Bytes der durch `IdentityResolver` gelieferten kanonischen Kennung nach NFC, längenpräfixiert (`u16 be`), höchstens 256 B. Verschiedene Quellen (E-Mail, Kundennummer) werden vom `IdentityResolver` auf **eine** kanonische Kennung abgebildet; das Mapping selbst ist `IdentityDirectoryOwner`-pflichtig (`INV-DIR-1`).
5. **Fehlerübersetzung an der Mandantengrenze.** Die Engine liefert Endnutzern bei `UnknownCluster`, `Erased`, `Restricted{BLOCK_READ}` und fehlender Berechtigung dieselbe Antwort „nicht verfügbar“ (AT-13). `Restricted` wird nur Principals mit Rolle `Compliance` ausgewiesen.
6. **Differentialtests** (Pflicht vor G-R4): (a) zwei Mandanten mit identischem Korpus liefern disjunkte Partitionen und keine gemeinsamen Cache-Treffer; (b) jede Query mit Mandant A liefert nie einen Record mit `tenant_tag` B; (c) Canary-Records in B tauchen in keiner A-Antwort, keinem A-Metrikwert und keinem A-Fehlertext auf.

## 4. Compliance-Profile

Ein Profil: `ComplianceProfile::{Development, Standard, Strict}` (R14 FIX-24). `open` verweigert Widersprüche mit `ProfileViolation{rule}`; die Regel-ID ist Teil des stabilen `hint()`.

| Regel | `Development` | `Standard` | `Strict` |
|---|---|---|---|
| P-1 Schlüsselzusicherung | beliebig | `Kms` oder `Hardware` | `Hardware` |
| P-2 `KeysInBackups` | beliebig | `No` oder `Bounded` | `No` |
| P-3 Anker | `Weak` erlaubt | `Required`; `Weak` nur mit `acknowledged_weak` | `Strong` |
| P-4 Ledger-Gerät | gleich erlaubt | getrennt; `ledger_same_device_ack` erlaubt | getrennt und andere Failure Domain; kein Ack |
| P-5 `Personal` | Warnung | `Sealed` erzwungen | `Sealed` + Provider-Attest |
| P-6 Acknowledgments | alle | erlaubt, erscheinen im Receipt (Statuscap/Restexposition) | **verboten** (`ProfileViolation`) |
| P-7 `placeholder_retention` | beliebig | `Keep` oder `Scrub` | `Scrub` |
| P-8 Pflichtmessung nicht `Complete` | informativ | `Incomplete` | `Incomplete` + Alarm |
| P-9 `ryw_wait`/`IndexLag` | tolerant | Standard | strenger Alarm |
| P-10 `HostProbe` | Diagnose | `Declared` genügt | `Known`; `Unknown` ⇒ Warnung, Receipt `HostProbe=Unknown` im Disclosure |
| P-11 Retention-Verlängerung | erlaubt | erlaubt, Observer-Ereignis | erlaubt nur mit `FourEyes` |
| P-12 Inspector | frei | `InspectCap` | `InspectCap::Diagnostic`, Audit-Eintrag |

`Standard` ist der Standard bei persistentem Open. `Development` ist an Feature/`cfg` gebunden: ein Release-Build ohne Feature `development` lehnt `Development` ab (`ProfileViolation{P-0}`), damit kein Produktivsystem versehentlich schwach öffnet (GAP-30).

## 5. Egress

`EgressPolicy::check(items: &[EgressItem], target: &EgressTarget) -> Verdict { Allow | AllowPseudonymized | Deny{reason} }` ist Pflicht auf jedem Pfad zu externen Modellen (`INV-EGRESS-1`).

1. `EgressTarget` hat `locality ∈ {Local, EuCloud, ThirdCountry}` und `contract_ref` (AV-Vertrag, opak). Der Kern kennt das nicht; es ist Engine-Konfiguration.
2. Standard `DenyExternal` für `Personal`. `NonPersonal` mit Trust-Label ≥ `tool_verified` darf extern, wenn die Policy es erlaubt.
3. Externe Ziele erhalten nur Platzhalter. Rückübersetzung findet lokal nach der Modellantwort statt (CYC-7 §9).
4. Das Verdikt wird je Item ausgewertet; ein einziges `Deny` im Kontextpaket entfernt das Item (nie das ganze Paket), und die Antwort weist `egress_denied_count` aus.
5. Der Egress-Check wird **nach** `DiBud` und **vor** der Übergabe ausgeführt (CYC-2 §5, QP-8). Er ist unabhängig vom Query-Plan; ein Plugin, das einen Provider direkt ruft, verletzt `INV-EGRESS-1` und wird in der Engine-API nicht angeboten.
6. Prompt-Injection: Contextra liefert Trust-Labels und erzwingt, dass `untrusted`/`tool_unverified` nie mit `system` verschmelzen (CYC-7 §10). Der Injection-Filter liegt beim Client/Plugin (DEC-36-Linie).

## 6. Untrusted Eingaben und Extraktionsgrenze (AT-9, DEC-35, NZ-24)

1. Contextra parst keine untrusted Dateiformate. Es nimmt **extrahierten Text** entgegen. Die Extraktion läuft in einem anderen Prozess oder Repository (Sandbox); der Workspace enthält keine Parser (SYS-1 §1, Bereinigung Nr. 2).
2. Die Annahmegrenze (`engine::ingest`) prüft vor jeder Weiterverarbeitung: gültiges UTF-8 (sonst `InvalidInput`, kein Reparieren), Normalisierung NFC, Steuerzeichen außer `\n`, `\t` entfernt oder abgelehnt (Policy), Länge `≤ ingest_max_doc_bytes` (Standard 64 MiB, Engine-Parameter, nicht im Kernregister), Nullbytes abgelehnt, Verhältnis Zeichen/Bytes plausibel (ablehnen bei > 20 % Ersatzzeichen).
3. Metadaten des Extraktors (Dateiname, MIME) sind `untrusted` und gelangen nur in den Kopf mit Trust-Label `untrusted`; sie fließen nie in Schlüssel oder Pfade.
4. Der Extraktor bekommt keinen Schreibzugriff auf Contextra-Daten. Seine Ausgabe wird als `Vec<u8>` über die Engine-API übergeben, nicht über eine Datei im Datenverzeichnis.
5. `contextra-engine` hat keine Parser-Abhängigkeit (SYS-1 §8, Verbotsliste); der Test `no_untrusted_parsers` prüft `cargo tree`.

## 7. Betroffenenrechte (Mapping, keine Rechtsaussage)

| Recht | Technische Grundlage | API | Besonderheit |
|---|---|---|---|
| Auskunft (Art. 15) / Übertragbarkeit (Art. 20) | `export_cluster` je Cluster, Wurzelcluster + Nachfahren über `docs_index` | `engine::compliance::export_subject` | ohne Schlüssel und Indexstrukturen; Rechte Dritter und Übermittlung liegen oben |
| Berichtigung (Art. 16) | normaler Change mit neuem `Put`; Vorversion bleibt bis `history_floor` physisch | `engine::collection::update` | bei `Personal` nur `KeepUntilFloor` mit `history_retention`; `Logical`-Range-Aufträge weisen Disclosure aus |
| Löschung (Art. 17) | CYC-3 | `Eraser::cluster`, Wurzel + Links | Receipt, Frist, Restexposition |
| Einschränkung (Art. 18) | `Restriction{BLOCK_READ|BLOCK_WRITE|BLOCK_ERASE}` | `Clusters::restrict` | Ablauf löst nie Löschung aus |
| Widerspruch/Aufbewahrungsende | `due` + Retention | `engine::compliance` | CYC-3 §11 |
| Rechenschaft (Art. 5 Abs. 2) | Ledger, Receipt, `verify_receipt` | `Eraser::receipt` | Signatur oben (Ed25519, optional) |

Mehrbetroffene Daten: gemeinsamer Fallcluster; jede Person hat einen Wurzelcluster; Link nur, wo die Löschung der Person den Fallcluster mitziehen soll, sonst nicht kaskadierende Referenz (CYC-7 §9).

## 8. Restexposition (Disclosure-Katalog, vom Orakel geprüft)

Jeder Eintrag erscheint im Receipt als **Statuscap** (begrenzt auf `CompleteWithResidual`) oder **Disclosure** (immer aufgeführt, kein Cap). Die Unterscheidung liegt in CYC-3 §10; hier steht die Liste.

| Nr. | Bereich | Restrisiko | Art | Zuständig |
|---|---|---|---|---|
| 1 | Prozessspeicher, Swap, Core-Dateien | Klartextreste | Disclosure; `HostKnownExposed` (Cap) bei bekanntem unsicheren Swap/Core-Dump | `Zeroizing`, Cache-Drop beim Gate; `mlock`/`DONTDUMP` im Adapter; Betreiber: Swap verschlüsseln; `HostProbe` |
| 2 | FS-Journal, Flash-Wear-Leveling, Volume-Snapshots, externe Backups, Checkpoints mit `Retired` Plain-Daten | alte Blockversionen | Disclosure | Sealed: nur Ciphertext; Plain: Betreiber begrenzt Aufbewahrung |
| 3 | KMS-Karenz | Schlüssel bis Fristende wiederherstellbar | Ticket `KeyDestroying`/`Blocked` | CYC-4 |
| 4 | Schlüsselkopien | Stufe G1 hinfällig | Cap `BoundedBackupKeys{until}` | `KeysInBackups` |
| 5 | `FilePassphrase` | Vernichtung nicht beweisbar | Cap `WeakKeyAssurance` | CYC-4 §6 |
| 6 | Metadaten (Klartext) | ClusterId in Redo-Gate-Frames bis Segmentabbau, Anzahl, Größen, Zeitpunkte, Packgruppe/`tenant_tag` | Disclosure | zufällige IDs; Ledger nur epochengebundene HMAC-`tomb_id` |
| 7 | Rollback über den Anker hinaus | Ticket rückgängig | Ankerstärke im Receipt; Cap `WeakAnchor` | `Strong`-Anker |
| 8 | Ledger auf demselben Gerät | gemeinsame Failure Domain | Cap `LedgerSameDeviceAck` | `ledger_dir` |
| 9 | Obere Schichten (Prompt-Caches, Embedding-Kopien, Logs, Fine-Tuning, Modellgewichte, Provider-Logs) | nicht im Kern lösbar | Disclosure; unregistrierte externe Persistenz = Cap | nur über Links/`SegmentOwner` angebundene Daten werden mitgelöscht |
| 10 | Pseudonymisierter Restkontext im Korpus | Re-Identifizierbarkeit; bei `Keep` `PseudonymousPlaceholdersInCorpus{count}` | Disclosure | Einzelfallbewertung |
| 11 | Verzeichnis Person → Wurzelcluster; `dir_key`-Label in Backups | linkbares Pseudonym | Cap `CompleteWithResidual` bis Backup-Frist | CYC-7 §9 |
| 12 | Nur `Logical` erreicht | Inhalt auf Platte | Cap `LogicalOnly` | CYC-3 §3 |
| 13 | Historie bei `Personal` (`Logical`-Range-Aufträge, Korrekturen) | Vorversion physisch bis Boden + Compaction | Disclosure | `history_retention` |
| 14 | Retention-Verlängerung | Aufbewahrung über `due` | Disclosure (`RetentionExtension`) | Restriction |

## 9. Telemetrie und Fehlertexte

Metrikkatalog und Verbote: SYS-4 §7. Fehlertexte: SYS-1 §6 (`hint()` stabil, ohne Schlüssel, Nutzerschlüssel, Inhalte, Queries, Embeddings). `Debug` für `ClusterId`, Schlüssel, Nutzdaten, Query-Texte ist `Redacted<T>`. Observer-Ereignisse tragen nur Enum-Typen und Zähler. Tracing-Spans der Engine dürfen weder Query-Text noch `PrincipalId` im Klartext enthalten (nur ein prozessgebundenes, rotierendes Pseudonym).

## 10. Entscheidungen und geschlossene Lücken

DEC-36 (PiiClassifier offen, NER als Plugin), DEC-37 (Egress Standard `DenyExternal`), DEC-56 (`Development` nur mit Feature), DEC-63 (`TenantTag` = `tenant_slot`, kanonische Personen-ID), DEC-65 (einheitliche Mandantenfehler); GAP-27, GAP-28, GAP-29, GAP-30 (SYS-7 §3).

## 11. Abnahme

`profile_matrix` (jede Regel P-0…P-12 positiv/negativ), `tenant_differential` (§3 Nr. 6), `error_translation_uniform` (AT-13), `egress_denied_never_leaves` (Canary), `no_untrusted_parsers`, `ingest_boundary_rejects` (§6 Nr. 2), `receipt_disclosures_complete` (§8 gegen Orakel), `canonical_person_id_vectors`.

**Offene Punkte (extern):** Rechtsprüfung der Fristen und der Aussage „kryptografisch gelöscht“ im Einsatzfall; AV-Verträge je `EgressTarget`.
