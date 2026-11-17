# Contextra – Spezifikation v7.1 (normativ): Vom Ist-Zustand zum Soll-Zustand

16. Nov 2026 · Revision v7.1 „Zielbild souveräne Entscheidungsplattform, Stufenplan, Löschung und Zeitreise vereinbar“ · Vorgänger: v6 (archiviert unter `docs/archive/SPEC_v6.md`)

## 0 Lesehilfe

**Was sich gegenüber v6 ändert.** v6 beschrieb ein Soll mit 40+ Crates, 16 Invarianten, 38 nicht laufenden Gates und über 100 Schritten, ohne dass der Ist-Zustand gebaut oder getestet war. v7 dreht die Blickrichtung um: Jedes Kapitel nennt zuerst den **Ist-Zustand** (am Code belegt), dann den **Soll-Zustand**, dann die **Begründung** („Warum“) und den **Weg** dorthin. Alles, was für das Fundament (löschbares, versioniertes Gedächtnis) nicht nötig ist, wird **nicht gestrichen, sondern einer späteren Stufe zugeordnet** (§1.4, Anhang A). v7.1 ergänzt das Zielbild „souveräne Entscheidungsplattform“ und die Stufen, die dorthin führen.

**Belegmarken.**

| Marke | Bedeutung |
| --- | --- |
| **[V]** | Am Repository-Stand `24487fa` (Shallow-Klon, 16.11.2026) per Datei, `grep` oder `Cargo.toml`-Auswertung geprüft. **Nichts wurde gebaut oder ausgeführt.** |
| **[R]** | Aus den Reports in `reports/` oder aus `docs/Umsetzbarkeit_Loeschung_Contextra.md` übernommen. Dort teils durch Läufe belegt, hier nicht neu reproduziert. |
| **[E]** | Einschätzung, nicht belegt. |
| **[D]** | Festlegung dieser Spezifikation. |

**Normative Sprache.** MUSS, DARF NICHT, SOLL nach RFC 2119. **Jede MUSS-Aussage hat einen benannten Test.** Ein Test, der nicht existiert oder nicht läuft, macht die Aussage zum SOLL (D-04). Dieser Grundsatz ersetzt die Gate-Bürokratie aus v6: Eine Aussage ist so viel wert wie der Test, der sie belegt.

**Rangfolge.** (1) Diese Spezifikation samt Entscheidungsregister (§13). (2) Crate-lokale `SPEC.md`: nur Binnenstruktur, bei Widerspruch gilt (1). (3) Alles andere in `docs/` ist Material.

---

## 1 Produktdefinition

### 1.1 Soll: Zielbild

Contextra wird eine **souveräne, offline betreibbare Plattform für Datenintegration, Wissensmodellierung und Entscheidungsvorbereitung** für Unternehmen und Behörden. Sie arbeitet mit **lokalen LLMs**, ohne dass Daten das Haus verlassen. Sie gehört in die Funktionsklasse von Plattformen wie Palantir Foundry (Daten zusammenführen, zu Objekten und Beziehungen modellieren, Auswertungen und Was-wäre-wenn-Rechnungen daraus ableiten), ist aber bewusst **leichtgewichtig, europäisch, lokal und datenschutzgeprägt**.

Drei Eigenschaften unterscheiden sie und sind zugleich ihr technischer Kern:

1. **Versionierung und Zeitreise** über Daten und Regeln: jeder frühere Stand ist reproduzierbar abfragbar.
2. **Dauerhafte, belegbare Löschung** personenbezogener Daten, die den Versionsverlauf nicht zerstört (§9).
3. **Souveränität**: keine Telemetrie, kein Fernzugriff, keine Schlüsselhinterlegung beim Hersteller, lokale Modelle.

**Wichtig zur Ausdrucksweise (INV-7, D-21).** „DSGVO-konform“ ist eine Bewertung des Verantwortlichen und der Aufsichtsbehörden, keine Eigenschaft einer Software. Contextra **unterstützt** den datenschutzgerechten Betrieb und liefert Nachweise (Stufenmodell, Berichte). Produkttexte sagen das auch so. „Palantir-Ersatz“ ist eine interne Zielbeschreibung, keine Produktaussage.

### 1.2 Ist

- 39 Crates unter `crates/`, 453.495 Zeilen Rust (inklusive Tests) **[V]**.
- `CLAUDE.md` (Eigentümer-Notiz): Contextra wurde als langlebige, kaperungsgeschützte Komponente für Air-Gap-Roboter entwickelt; das Robotikziel wurde verworfen, die Entscheidungen blieben **[V]**.
- Robotik- und Regelungsreste im Code: `adapt/src/pid.rs`, `types/src/types/saos.rs`, `db/src/pid_latency_controller.rs`, `db/src/homeostat.rs`, `rank/src/fusion/resonance.rs`, `mvcc/src/snapshot.rs` (SAOS-Erwähnung) **[V]**.
- Die letzten Audits stufen das System als **nicht produktionsreif** ein: sieben Crates haben nicht kompilierende Test-Targets, darunter `vector`, `store`, `db`; der Löschanspruch ist für den öffentlichen API-Pfad nicht belegt; die Qualitätsmessung läuft nicht **[R]** (`reports/AUDIT_VERIFIKATION_2026-11-16.md`).

### 1.3 Begründung

**Warum Fokus statt Breite?** Ein Soloentwickler pflegt mit KI-Unterstützung ein System, dessen Fläche (453k Zeilen) größer ist als das, was er verifizieren kann. Jede Zusage in v6 (Z-E1 bis Z-E12, Z-T1 bis Z-T5, Z-S*) hing von einem Build ab, der nicht grün ist. Wer Löschnachweise verspricht, aber seine Kernpfade nicht kompiliert, hat keine Zusagen, sondern Absichten. Der schnellste Weg zu einem belastbaren Produkt ist, die Fläche auf das zu reduzieren, was Löschung und Zeitreise tragen, und dort Nachweise zu liefern.

**Warum das Robotikerbe ein RAG behindert** [E, begründet]:

| Robotik-Anforderung | Wirkung auf ein RAG |
| --- | --- |
| Bitgenaue Determinismus-Zusagen über x86 und aarch64, Rangfolgen ohne Gleitkomma (v6 Z-T1) | Dichte Vektorsuche und Score-Fusion rechnen mit Gleitkomma. Die Zusage ist ohne Verlust an Suchqualität nicht haltbar und für Nutzer ohne Wert. |
| Regelkreise (PID-Latenzregler, Homeostat, Stabilitätsüberwachung) | Lösen ein Echtzeitproblem, das ein lokales RAG nicht hat. Sie erhöhen Zustand und Fehlerfläche ohne messbaren Nutzen. |
| Ringmodell mit fünf Ebenen und 16 Entnehmbarkeits-Gates | Prozesskosten ohne Kundennutzen. Die Schichtung selbst ist gut (§4), die Gate-Zahl nicht. |
| Fail-closed, keine Telemetrie, lokale Schlüssel | **Behalten.** Das ist für ein DSGVO-nahes RAG ein Verkaufsargument, kein Ballast. |

### 1.4 Stufenplan zum Zielbild

Die Stufen bauen aufeinander auf; keine Stufe beginnt, bevor die vorherige ihre Zusagen (§10) erfüllt. **Warum diese Reihenfolge?** Alle späteren Stufen speichern, verknüpfen und zeigen personenbezogene Daten an. Ohne funktionierende Verschlüsselung, Sperre und Zeitauflösung müsste jede Stufe diese Mechanik selbst nachrüsten und würde dabei die Löschung unterlaufen (§9).

| Stufe | Inhalt | Crates | Voraussetzung |
| --- | --- | --- | --- |
| **1 Fundament** | Gedächtnis, Hybrid-Suche, Versionierung, Zeitreise, Löschkette, Messung | §4.2 „Kern“ | Phasen P0 bis P7 |
| **2 Ontologie und Integration** | Objekte und Beziehungen (Wissensgraph), Datenaufnahme aus Fachsystemen, versionierte Regeln und Fristen („Git für Regeln“, v6 §12), Herkunft (Lineage) | `graph`, `ingest` (ausgebaut), neues `rules` | Stufe 1 grün; §15.2 (Objekt-Gate) |
| **3 Entscheidungshilfe** | Lokale LLM-Anbindung mit Zugriffsprüfung, Agenten mit Budgets, Was-wäre-wenn-Simulation (v6 §13), Modell-Routing | `agent`, `cognition`, `router`, `adapt` (ohne Regelkreise), `infer-*`, `kvcache` | Stufe 2; §15.3 |
| **4 Betrieb in Behörden und Unternehmen** | Rollen und Zweckbindung, Mandanten, Prüfpfad, Verzeichnisse und Berichte, Zertifizierung, Lizenzierung | `license`, `audit-export`, `avv-generator`, Rollen-/Zweckmodul | Stufe 3; §15.1, §15.4 |

### 1.5 Nicht-Ziele (dauerhaft)

- **Keine automatisierte Entscheidung über natürliche Personen.** Contextra liefert Entscheidungs**vorbereitung** mit Quellen und Begründung; die Entscheidung trifft ein Mensch (D-17, Art. 22 DSGVO).
- **Kein Training und kein Fine-Tuning mit personenbezogenen Daten** (D-18, §15.3).
- **Kein Dokumentenmanagement für Originale**, keine Archivierung als Selbstzweck.

---

## 2 Ist-Befunde (Grundlage aller Entscheidungen)

Jeder Befund trägt eine ID und wird im Plan (§11) aufgelöst.

| ID | Befund | Beleg |
| --- | --- | --- |
| B-01 | Im Repository fehlt die Wurzel-`Cargo.toml`. `git ls-files` listet 51 `Cargo.toml`, keine im Wurzelverzeichnis. v6 zitiert aber Zeilen daraus (`default-members`, `exclude`). Ein frischer Klon baut den Workspace nicht. | **[V]** |
| B-02 | `contextra-erasure` (32.361 Zeilen) hat **keinen Verbraucher**: kein anderes Crate hängt davon ab. Es ist nicht in `default-members`, die Proof-Tests haben 78 Kompilierfehler (veraltete API). | **[V]** (Cargo-Auswertung), **[R]** (Fehlerzahl) |
| B-03 | `ReadGate` (die Lesesperre) wird nur in `erasure`, `ports` und `types` referenziert, **nie in `store`, `db` oder `engine`**. Kein produktiver Lesepfad prüft die Sperre. | **[V]** (`grep -rln ReadGate crates`) |
| B-04 | `delete_op` setzt Tombstones für `user_key` und `doc_key`, markiert Text-/Graph-/Index-Löschung und speichert alte Werte in der Transaktion. Kein Schlüssel wird widerrufen. | **[V]** `db/src/collection/crud/delete.rs` |
| B-05 | T-DEL-1 (Canary-Bytes nach Löschen) schlägt reproduzierbar fehl: Die alte SST mit dem Canary bleibt liegen. Die Fassade `contextra` bietet keinen Compact-/Purge-Pfad. Der Test „nach Kompaktion“ ruft keine Kompaktion auf. | **[R]** `AUDIT_VERIFIKATION` §4 |
| B-06 | `ContextraConfig` setzt `encryption_passphrase: None` als Default. `deletion_proof_active` wird in `lifecycle.rs` (Zeilen 26, 42, 60) auf `false` gesetzt. Profil `EnterpriseRegulated` deklariert `CryptoShred`, erzwingt aber keine Verschlüsselung. | **[V]** (Config), **[R]** (Profil) |
| B-07 | `KeyRegistry::new()` erzeugt den Master-KEK zufällig im RAM; nach Neustart ist jeder Chiffretext unlesbar. `revoke_record(&str)` wirkt auf keinen Schlüssel. Das Revocation-Log ist nur in Tests verdrahtet. | **[R]** `Umsetzbarkeit_Loeschung` N1 bis N3 (per Experiment belegt) |
| B-08 | Zeitreise (`as_of_timestamp`) ist ein **Metadatenfilter nach der Suche**: `builder_exec.rs:175` ruft `temporal_filter::apply_temporal_validity_filter_at`. Gelesen werden JSON-Felder (`tx_valid_from` usw., auch als String), Kandidaten ohne Metadaten gelten als gültig (Fail-Open). Es wird **kein älterer MVCC-Stand gelesen**. | **[V]** |
| B-09 | `mvcc` führt nur die Sequenzachse (`SnapshotRegistry`, Pins, `min_for_compaction`). Die Gültigkeitszeit liegt untypisiert in `db`. Das Kopfkommentar von `temporal_filter.rs` ordnet das Modul fälschlich `contextra-engine` zu. | **[V]** |
| B-10 | Kompaktion hält Tombstones zurück, solange Snapshots sie brauchen (`INV-COMPACTION-ADAPTIVE-1`, `min_active_seqno`). `pin()` ist dauerhaft bis `unpin()`. | **[V]** |
| B-11 | `checkpoint` klont SSTables per Hardlink und hält dafür einen Pin; bei Fehlschlag Byte-Kopie. Zweck: Agent-Forking. Die Dateien enthalten nach B-04/B-06 Klartext. | **[V]** `hardlink_cloner.rs` |
| B-12 | Zwei Tresor-Implementierungen: `erasure/src/vault/` und `contextra-vault` (in `exclude` laut v6). | **[R]** v6 F-19, Verzeichnisse **[V]** |
| B-13 | `DocId::from_key` ist ein 64-Bit-Präfix von BLAKE3; kein Schutz vor Kollisionen. Bei Kollision teilen zwei Dokumente `record_id` und damit Schlüsselmaterial. | **[R]** N7; `ids/src/lib.rs` **[V]** vorhanden |
| B-14 | `contextra-bench` baut nicht (`datasets` verlangt fehlende `locomo.rs`, `long_mem_eval.rs`). Es gibt **keine reproduzierbare Messung der Suchqualität**. | **[R]** |
| B-15 | Lebenszyklus: `LsmStorage::drop` wartet nicht auf die Compaction-Task, die den `DirLock` hält; ein Wiederöffnen scheitert (`persistence_test`). | **[R]**, Ursache plausibel, nicht im Code nachgeprüft |
| B-16 | Der in `reports/FIX_REPORT` genannte Branch `fix/test-targets` ist im Klon und Remote nicht auffindbar. | **[R]** |
| B-17 | CI: `|| true` hinter `cargo check --workspace` und der Layering-Prüfung; 38 Gates in `not_run_baseline`. | **[R]** (v6 F-01, §7.3) |

---

## 3 Leitprinzipien

| ID | Prinzip | Warum |
| --- | --- | --- |
| P1 | **Ein Build, der grün ist, kommt vor jeder neuen Zusage.** | B-01, B-02, B-17: Ohne Build ist jede Aussage unbelegt. |
| P2 | **Verschlüsseln ist der Standard, Klartext die begründete Ausnahme.** | Der Zustand „Klartext in SST und WAL“ (B-04/B-06) ist die Ursache für B-05. Wer von Anfang an verschlüsselt, muss nicht nachträglich Klartext jagen. |
| P3 | **Löschen vernichtet Schlüssel und sperrt Lesezugriffe. Es hängt nicht an Kompaktion.** | Kompaktion hängt an Snapshots (B-10). Eine Löschung, die auf Garbage Collection wartet, kann nie schneller sein als der älteste Pin. |
| P4 | **Zeitreise liest Geschichte, sie rekonstruiert sie nicht aus Metadaten.** | B-08: Ein Metadatenfilter kann nur ausblenden, was in den Treffern steht; er kann keine Fassung von gestern liefern. |
| P5 | **Genau ein Ort für jede Kreuzungsstelle.** Löschsperre, Zeitauflösung und Schlüsselzugriff haben je einen Einstiegspunkt. | B-03: Eine Sperre, die in jedem Pfad einzeln aufgerufen werden müsste, wird in einem Pfad vergessen. |
| P6 | **Weniger Crates, jedes mit Verbraucher und Test.** | B-02: 32k Zeilen ohne Verbraucher sind keine Funktion, sondern Last. |
| P7 | **Fehler sind ehrlich.** Keine Stufe „gelöscht“ ohne gemessene Stufe; unimplementiert heißt `NotImplemented`. | B-06: Ein Flag, das immer `false` ist, und ein Profil, das Schlüsselvernichtung deklariert, aber nicht erzwingt, sind falsche Compliance-Signale. |
| P8 | **Spezifikation folgt dem Code, nicht umgekehrt.** Neuer Spec-Text braucht einen Test, der ihn belegt. | v6 beschrieb ein Soll, das keiner bauen konnte; die Lücke wuchs mit jeder Revision. |

---

## 4 Scope: Crates in Stufe 1 und zurückgestellte Crates

### 4.1 Soll-Aufteilung

Drei Schichten statt fünf Ringe. Abhängigkeiten zeigen nur nach unten.

| Schicht | Crates (v1) | Aufgabe |
| --- | --- | --- |
| **Fundament** | `ids`, `env`, `sys`, `durable-fs`, `crypto`, `privacy`, `types`, `ports` | Typen, Uhr, Zufall, Dateisystem, Kryptografie, Erkennung personenbezogener Muster |
| **Speicher & Index** | `mvcc`, `store`, `text`, `vector`, `rank`, `erasure` | Versionen, LSM, Volltext, Vektoren, Fusion, Löschkette |
| **Fassade & Schnittstelle** | `db`, `contextra`, `mcp`, ein Embedder (`infer-ollama` **oder** `infer-onnx`), `testkit` | Zusammenspiel, öffentliche API, Werkzeugschnittstelle |

### 4.2 Entscheidung je Crate

„Verbraucher“ = Crates, die laut Cargo-Auswertung als normale Abhängigkeit darauf zeigen **[V]**. Zeilenzahlen enthalten Tests und sind Richtwerte.

| Crate | Zeilen | Verbraucher | Entscheidung | Begründung |
| --- | ---: | --- | --- | --- |
| `ids`, `env` | 1.020 / 1.341 | 8 / 10 | **Kern** | Klein, breit genutzt, Grundlage für alles. |
| `sys`, `durable-fs` | 2.988 / 3.491 | 3 / 0 | **Kern** | Dateisystem-Haltbarkeit trägt die Crash-Invarianz (§8.7). `durable-fs` hat heute keinen Verbraucher: wird in P3 angebunden oder fliegt raus. |
| `mvcc` | 9.017 | 3 | **Kern** | Besitzer der Zeit- und Sequenzachse (§7). |
| `store` | 59.788 | 1 (`db`) | **Kern, Fokus** | Größtes Crate; trägt Persistenz. Test-Targets müssen kompilieren (P2). |
| `crypto` | 11.462 | 4 | **Kern** | Verschlüsselung, KDF, WAL-HMAC. Die parallele KV-Variante (`kv_cipher`) bleibt vorerst; Bereinigung in P5. |
| `text`, `vector`, `rank` | 18.176 / 34.192 / 10.810 | 2 / 1 / 4 | **Kern** | Suche. `rank` wird auf RRF und Kalibrierung reduziert; `resonance` und `dibud` werden ausgeschlossen (§4.4). |
| `privacy` | 9.334 | 1 (`mcp`) | **Kern** | Erkennung personenbezogener Muster vor Cloud-Aufrufen und als Warnung im Schreibpfad. |
| `erasure` | 32.361 | **0** | **Kern, wird verdrahtet** | Herzstück der Löschung, heute abgekoppelt (B-02). Wird in P3/P4 an `db` angeschlossen. |
| `types`, `ports` | 9.574 / 7.208 | 27 / 21 | **Kern, gedeckelt** | Hoher Fan-in. Keine neuen Verbraucher aus Fundament und Speicher (§4.3). |
| `db`, `contextra` | 35.023 / 3.809 | 3 / 2 | **Kern** | Fassade; hier sitzt der Lese-Einstiegspunkt (§8.4). |
| `mcp` | 8.050 | 0 | **Kern (Anwendung)** | Einzige Schnittstelle für Fremdwerkzeuge in v1. |
| `infer-ollama` | 7.370 | 1 | **Kern (Embedder)** | Ein Embedder genügt. Wahl: Ollama, weil lokal und ohne ONNX-Laufzeit; `infer-onnx` (1.176 Zeilen) ist die Alternative. [E] |
| `testkit` | 7.389 | 0 | **Kern (Test)** | Hostet die Tests der Bausteine. |
| `contextra-vault` | 816 | 1 (`db`) | **Zusammenführen** | Zwei Tresore (B-12). Eine Implementierung bleibt (D-08). |
| `graph` | 34.597 | 2 | **Zurückgestellt (Stufe 2), Kernbaustein der Ontologie** | Wird in Stufe 2 zur Ontologie (§15.2). Heute zweitgrößter Block; `db` und `cognition` hängen daran. Die Kante wird hinter ein Feature gelegt (P1.2), damit Stufe 1 baut. |
| `cognition`, `agent`, `engine` | 11.979 / 11.217 / 9.335 | 1 / 0 / 0 | **Zurückgestellt (Stufe 3)** | Agentik und Konsolidierung sind Teil der Entscheidungshilfe, brauchen aber das Fundament. `engine` hat keinen Verbraucher und wird bei Stufe 3 neu bewertet. |
| `router`, `adapt` | 11.342 / 8.272 | 2 / 2 | **Zurückgestellt (Stufe 3), ohne Regelkreise** | Modellauswahl für lokale LLMs ist Zielfunktion; PID, Homeostat und Stabilitätswächter sind Robotik-Erbe und werden ausgeschlossen (§4.4). |
| `checkpoint`, `kvcache`, `infer-candle` | 11.186 / 11.614 / 8.910 | 2 / 0 / 2 | **Zurückgestellt (Stufe 3)** | Hardlink-Klone enthalten Klartext (B-11); KV-Caches sind abgeleitete Personendaten (§15.3). Rückkehr erst mit Verschlüsselung und Löschanbindung. |
| `ingest` | 30.737 | 0 | **Fokus-Prüfung (P1)** | Größter Block ohne Verbraucher. v1 braucht Text, Markdown und PDF; alles Weitere pausieren oder hinter Features legen. |
| `simd` | 6.461 | 1 | **Zurückgestellt (Leistungsstufe)** | `unsafe`-Insel; skalare Distanzen genügen für Stufe 1. Kommt zurück, wenn Messungen (P6.6) den Bedarf belegen. [E] |
| `license`, `audit-export`, `avv-generator` | zusammen ~7.400 | 0 | **Zurückgestellt (Stufe 4)** | Betriebs- und Nachweisfunktionen für Behörden und Unternehmen. |
| `sandbox`, `prompt-guard`, `infer-onnx` | zusammen ~7.500 | 1–2 | **Zurückgestellt (Stufe 3)** | Werkzeugausführung und Eingabeschutz gehören zur LLM-Anbindung. `prompt-guard` ist klein und kann früher vorgezogen werden. |
| `py` | 3.555 | 0 | **Zurückgestellt (Stufe 4)** | Anbindung für Fachanwender-Skripte. |

**Zurückstellen heißt nicht löschen.** Verfahren (D-09): (1) Git-Tag `pause/v1/<crate>`; (2) Verzeichnis nach `archive/<crate>/` verschieben **oder** unter `crates/` belassen und aus `[workspace] members` nehmen; (3) Eintrag in Anhang A. Nichts wird gelöscht, bevor der Build ohne das Crate grün ist.

### 4.3 Regeln (reduziert)

| Regel | Inhalt | Prüfung |
| --- | --- | --- |
| R1 | Abhängigkeiten zeigen nur nach unten (Fundament ← Speicher & Index ← Fassade). | `cargo-deny` plus ein Test, der `cargo metadata` auswertet |
| R2 | Kein neuer Verbraucher von `types` und `ports` im Fundament und in Speicher & Index. | Ratchet-Zahl in `governance/ratchet.toml`, darf nur sinken |
| R3 | Jedes Crate hat einen eigenen Fehlertyp (`thiserror`, `#[non_exhaustive]`). | Bereits Praxis in den meisten Crates |
| R4 | Kein `unwrap`, `expect`, `panic!` in neuem Nicht-Test-Code. | Clippy-Lint |
| R5 | Neue Features sind additiv und nie Teil von `default`, wenn sie `exp-*`, `workspace` oder Tests betreffen. | `feature_audit.py` bleibt, läuft blockierend |

**Warum reduziert?** v6 hatte 14 Regeln, 16 Invarianten und eine Entnehmbarkeitsprüfung je Crate. Das ist ein Produkt für sich. Behalten wird, was den Build, die Schichtung und die Sicherheitsaussagen schützt (R1, R2, R4); der Rest war Absicherung für eine Baukasten-Auslieferung, die nicht Ziel von v1 ist (D-10).

### 4.4 Robotik-Reste: ausschließen statt mitschleppen

**Soll (D-20):** Robotik- und Regelungscode ist **nicht Teil des Standard-Builds** und nicht Teil des Zielprodukts. Betroffen sind nach heutigem Befund `adapt/src/pid.rs`, `db/src/pid_latency_controller.rs`, `db/src/homeostat.rs`, `rank/src/fusion/resonance.rs`, `types/src/types/saos.rs` und die SAOS-Bezüge in `mvcc/src/snapshot.rs` **[V]**.

Verfahren in drei Schritten, jeder einzeln revertierbar:
1. **Deaktivieren:** Hinter ein Feature `legacy-control` legen, nie in `default`, nie in `workspace`.
2. **Messen:** Ein Test baut den Standard-Build und schlägt fehl, wenn die genannten Symbole darin vorkommen.
3. **Entfernen:** Nach zwei Releases ohne Nutzung per Git-Tag archivieren und löschen.

**Warum nicht sofort löschen?** Einzelne Teile (z. B. Bandit-Auswahl in `adapt` für Modellrouting) können in Stufe 3 sinnvoll sein. Entscheidend ist, dass Regelkreise nicht still das Latenz- oder Suchverhalten verändern: Ein lernender Regler, der Suchtiefe anpasst, macht Ergebnisse von Anfragen abhängig vom Systemzustand, und das verträgt sich nicht mit nachvollziehbaren Entscheidungsgrundlagen (§7.3).

---

## 5 Build- und Prozessgrundlage

### 5.1 Ist → Soll

| Ist | Soll | Warum |
| --- | --- | --- |
| Wurzel-`Cargo.toml` fehlt im Repository (B-01) | Wird committet; `members` nur die Crates aus §4.2 (Anhang B) | Ohne sie ist ein Klon nicht baubar; jede Spec-Aussage ist dann nicht reproduzierbar. |
| `cargo check --workspace --all-targets` mit `|| true` (B-17) | Blockierend, ohne `continue-on-error` | Ein Gate, das nie fehlschlägt, ist kein Gate. |
| 38 Gates in `not_run_baseline` | Sieben laufende Gates (§12); der Rest entfällt oder wird der Stufe zugeordnet, in der er gebraucht wird | Nicht laufende Gates erzeugen die Illusion von Kontrolle. |
| Fix-Branch `fix/test-targets` unauffindbar (B-16) | Entweder auffindbar machen und mergen oder die Änderungen neu erzeugen | Der Reports-Stand muss mit HEAD übereinstimmen. |
| Test-Targets brauchen `--features workspace`, Justfile ruft ohne Features (Fix-Report §4) | `[[test]]`-Blöcke mit `required-features`; CI ruft beide Läufe | Entscheidung im Fix-Report offen gelassen; hier festgelegt (D-11). |

### 5.2 Prozessregel für KI-gestützte Entwicklung

- Kein Spec-Text ohne Test, der ihn belegt (P8).
- Jeder Pull Request nennt, welche Zusage aus §10 er berührt und welcher Test sich ändert.
- Berichte (`reports/`) tragen HEAD-Hash und Datum und werden bei jedem Release neu erzeugt. **Warum:** Mehrere Reports widersprechen sich oder beziehen sich auf Stände, die nicht im Repository liegen (B-16).

---

## 6 Datenmodell

### 6.1 Dokument, Version, Klasse

| Begriff | Definition [D] | Warum |
| --- | --- | --- |
| **Dokument** | Stabile Identität innerhalb einer Collection (`user_key`). | Das ist heute schon der Schlüssel (`delete.rs`). |
| **Version** | Unveränderlicher Inhaltsstand eines Dokuments mit Commit-Sequenz. Eine Änderung erzeugt eine neue Version mit **neuem DEK**. | Nur so lässt sich eine Version löschen, ohne die anderen zu berühren (Löschkonzept AP5). |
| **Klasse** | Je Collection: `Personal` (Standard) oder `NonPersonal`. `Personal` erzwingt Verschlüsselung und Gate. | Verschlüsseln als Standard (P2). Die vierteilige Klassenlehre aus v6 (K/P/D/E) entfällt in v1; Ableitungen und Nachweise sind weiterhin eigene Objektarten (§8). |

**Warum Standard `Personal`?** Fehlklassifikation in Richtung „unkritisch“ ist der teure Fehler. Der Preis der sicheren Seite ist Rechenzeit für AEAD, nicht Datenverlust. `NonPersonal` ist eine bewusste Entscheidung des Betreibers und wird im Profil protokolliert.

**Referenzen auf Personen.** Wo ein Dokument auf eine Person verweist, steht dort eine zufällige Referenz (`PersonalRef`, 128 Bit), keine aus dem Inhalt abgeleitete Kennung (aus v6 §11.2 übernommen; Umsetzung in P4). **Warum:** Aus Inhalt abgeleitete Pseudonyme sind nach Schlüsselvernichtung weiter verknüpfbar.

### 6.2 Dokument-IDs

Heute 64-Bit-Präfix von BLAKE3 (B-13). Soll:

1. Beim Einfügen prüft `db`, ob eine `DocId` bereits mit einem **anderen** `user_key` belegt ist; dann `Err(IdCollision)`. Der Originalschlüssel steht schon in `user_key` **[V]**, die Prüfung ist ein zusätzlicher Lesezugriff.
2. Kryptografische Bindung (AAD) nimmt `(tenant, collection, user_key, version)`, **nicht** die 64-Bit-`DocId`.
3. `docid-128` bleibt Feature für Anwendungs-Crates; Umstellung auf 128 Bit ist eine Formatentscheidung (frühestens Stufe 2).

**Warum nicht gleich 128 Bit?** Es ändert persistierte Formate (INV-1 aus v6) und ist für Kollisionsschutz nicht nötig, solange Kollisionen erkannt statt ignoriert werden. Die Geburtstagsschranke von etwa 5·10⁹ Dokumenten **[R]** liegt weit über dem Einsatzbereich eines lokalen Gedächtnisses.

---

## 7 Zeitreise

### 7.1 Ist

Siehe B-08, B-09, B-10. Zusammengefasst: Es gibt (a) MVCC-Snapshots für Transaktionen und (b) einen Nachfilter auf Metadatenfeldern. Dazwischen gibt es keine Brücke. Eine Anfrage „Stand vom 1. März“ liefert die **heutigen** Treffer, soweit ihre Metadaten das Datum zulassen.

### 7.2 Soll: zwei Zeitachsen, ein Typ, ein Lesepfad

**Transaktionszeit (`tx`).** Commit-Sequenz aus `mvcc` (`seq_no`, dicht, monoton) mit Zeitstempel aus `env::Clock`. Nutzer können `as_of(Commit)` oder `as_of(Zeitpunkt)` angeben; ein Zeitpunkt wird über eine kleine Tabelle `tx_time → seq_no` aufgelöst.

**Gültigkeitszeit (`valid`).** Fachliche Geltung als `[valid_from, valid_until)` in Kalendertagen (`i32`, Tage seit 1970-01-01); fehlend = ausdrücklicher Zustand `Unbounded`, nie `0`. Typ `ValidityWindow` liegt in **`mvcc`** (Besitzer der Zeit), `db` nutzt ihn.

**Warum nicht Millisekunden?** Fachliche Geltung (Fristen, Gesetzesstände) ist tagesgenau; Millisekunden erzeugen Scheingenauigkeit und Zeitzonenfehler. Transaktionszeit bleibt Millisekunden, weil sie technisch ist.

**Lesepfad.** `as_of` löst zu `seq_no` auf und liest über `get_at_seq` (existiert, `store/lsm/engine/facade.rs` **[V]**) den Stand. **Nicht** über den Metadatenfilter. Der Filter für `valid_at` wird ein typisierter Vergleich auf `ValidityWindow`-Feldern, **nicht** auf JSON-Strings.

**Fail-Open entfällt.** Kandidaten ohne Gültigkeitsfeld gelten als `Unbounded` **und** werden so ausgewiesen (`validity = unbounded`). Fehlerhafte Felder (nicht parsebar) lassen die Anfrage scheitern, statt den Kandidaten stillschweigend durchzulassen. **Warum:** Ein Filter, der bei Fehlern durchlässt, liefert bei Datenfehlern gerade die Treffer, die er ausschließen sollte.

### 7.3 Determinismus (herabgestuft)

Gleicher Indexstand, gleiche Parameter, gleiche Seeds ergeben **gleiche Rangfolge**; Gleichstand nach `DocId` aufsteigend. Keine Zusage bitgleicher Scores über Architekturen hinweg (D-06). **Warum:** Reproduzierbarkeit im Test ist wertvoll (Regressionen, Evaluation); bitgleiche Gleitkommaergebnisse auf mehreren CPU-Architekturen sind es für ein RAG nicht.

### 7.4 Stufe 2: Regeln versionieren („Git für Regeln“)

Verzweigen, Tags, Merge, Vier-Augen-Freigabe, Regeldokumente als Commit-Graph und Simulation (v6 Kapitel 12 und 13) sind **Teil des Zielbilds** (Fristen, Gesetzesstände, Schwellenwerte und Prompts als versionierte Regeln). Sie kommen in Stufe 2 (P8) und 3 (P9). **Warum nicht früher?** Sie speichern und lesen Inhalte über denselben Lesepfad und dieselbe Zeitachse wie Dokumente. Werden sie vor dem `Reader` (P5.1) und `ValidityWindow` (P5.2) gebaut, entsteht eine zweite Zeitlogik, genau die Dopplung, die B-09 beschreibt. Der Entwurf in `docs/archive/SPEC_v6.md` (§12.3 bis §12.6) bleibt als Grundlage gültig.

---

## 8 Löschung

### 8.1 Ist

Siehe B-02 bis B-07. Löschen = Tombstone (B-04). Klartext liegt in WAL und SST (B-05/B-06). Der Schlüsselteil existiert in `erasure`, ist aber nicht verdrahtet und nicht neustartfest (B-07). Kein Lesepfad prüft eine Sperre (B-03).

### 8.2 Soll: Schlüsselhierarchie

```
Master-Schlüssel (aus Passphrase, Argon2id, Salt im Container)
  └─ SubjectKey(s)           je Betroffenem
       └─ DocumentKey(v)     je Version
  DEK(v) = HKDF(DocumentKey(v) ‖ SubjectKey(s1) … SubjectKey(sn), salt = Versionsreferenz, info = "contextra/dek/v1" ‖ n)
```

(Aus v6 §11.3 übernommen.) Inhalte einer Version liegen unter **AEAD mit DEK(v)** und **AAD = (tenant, collection, user_key, version)**.

**Warum diese Form?**
- *Pro Version ein Schlüssel:* Eine Version lässt sich löschen, ohne andere zu verändern. Mehrere Betroffene je Version fließen in den DEK ein, damit die Vernichtung **eines** SubjectKeys die Version unlesbar macht.
- *AAD:* Ohne Bindung kann ein Chiffretext in ein anderes Dokument kopiert werden (Löschkonzept N8).
- *Master aus Passphrase (D-02):* Der Master-KEK heute ist zufällig und flüchtig (B-07). Passphrase + Argon2id ist offline verfügbar und passt zu lokalem Betrieb. OS-Keyring/TPM bleibt optionales Plugin, ein externer KMS widerspricht dem lokalen Betrieb.
- *Warum nicht eine Schlüsselhierarchie je Collection (wie heute `group_id`)?* Ein Collection-Drop vernichtet dann alles, und eine neue Collection mit gleicher ID bleibt unbeschreibbar (Löschkonzept N11). Betroffenen- und Versionsschlüssel treffen den DSGVO-Fall genauer. Der Collection-Drop wird ein Sonderfall mit eigener Epoche.

### 8.3 Soll: Persistenter Keystore

`erasure::vault` wird zum **einzigen** Tresor (D-08). Er hält Master-Wrapping, SubjectKeys, DocumentKeys und Grabsteine. **MUSS:** atomares Schreiben mit `fsync` über `durable-fs`; versionierter Container; beim Start fail-closed bei Beschädigung; kein Standard- oder Ersatzschlüssel; Zufallsfehler erzeugen weder Umschlag noch Ersatz (Z-L5).

**Warum atomar und mit Grabstein?** Eine Löschung muss einen Neustart überleben, und ein fehlender Schlüssel ohne Grabstein ist ein Integritätsfehler, kein Löscherfolg.

### 8.4 Soll: Lese-Einstiegspunkt (Gate)

Ein einziger interner Typ `Reader` in `db` ist der **einzige** Weg zu Dokumentinhalten (`get`, Trefferhydrierung, `as_of`, Export, Vorschau). Jede Leseoperation:

1. löst Dokument und Version auf,
2. fragt `ReadGate::check` (O(1), synchron, ohne I/O),
3. entschlüsselt nur bei `Open`.

Ein Architekturtest (`tests/read_chokepoint.rs`) lässt `db` scheitern, wenn Module außerhalb von `Reader` direkt auf Storage-Lesefunktionen zugreifen.

**Warum in `db` und nicht in `store`?** `store` kennt weder Betroffene noch Dokumentversionen; es kennt Schlüssel und Bytes. `db` kennt beides. Ein Gate im Store wäre Verteidigung in der Tiefe, aber nicht der Ort der Entscheidung. **Warum ein Typ statt einer Pflicht für jeden Pfad?** Siehe P5 und B-03.

**Gate wirkt im aktuellen Stand, nicht im Snapshot-Stand.** Ein Snapshot, der vor der Löschung geöffnet wurde, sieht nach der Löschung `Erased` (D-05). **Warum:** Löschung muss rückwirkend über alle Zeitstände gelten. Würde das Gate den Snapshot-Stand benutzen, könnte ein offener Pin die Löschung beliebig lange umgehen.

**Fehlerbild.** Berechtigte erhalten `Erased`; Unberechtigte dasselbe Bild wie bei einem nicht vorhandenen Ziel (Z-E8 aus v6). Intern unterscheidet `erasure` `Erased`, `NotFound` und `IntegrityError`; der Fehlerbaum von `erasure` (`Refused` für alles) wird aufgetrennt (Löschkonzept N9).

### 8.5 Soll: Löschablauf (zwei Schritte, zwei Durabilitätspunkte)

| Schritt | Inhalt | Durabilitätspunkt | Wirkung |
| --- | --- | --- | --- |
| **L1 Sperre** | Ziel (Betroffener, Dokument oder Version) ins Ledger; Gate sofort wirksam; Trockenlauf über alle Versionen | Ledger-`fsync` | Stufe **Gesperrt**. Ab hier liefert kein Lesepfad Klartext. |
| **L2 Schlüsselvernichtung** | Grabstein je Schlüssel im Tresor; Direktlesen zur Verifikation | Tresor-`fsync` | Chiffretexte im Store, in Indexsegmenten und Sicherungen sind unlesbar. |
| **L3 Bereinigung** | Entfernen abgeleiteter Strukturen (Index, Caches, Temp-Dateien) und, bei Bedarf, Neuschreiben betroffener Segmente | `fsync` je Segment | Stufe **Bereinigt**, wenn gemessen. |
| **L4 Messung und Bericht** | Scanner mit Positivkontrolle (Canary); signierter Bericht | Ledger-`fsync` | Stufe **Vollständig gemessen**. |

Gegenüber v6 (L0 bis L6, vier Stufen, Sicherungsepochen, Wurzelschlüssel-Rotation je Generation) ist der Ablauf **auf vier Schritte verkürzt**; Rotation und Sicherungsfenster (v6 L4) werden in Stufe 4 behandelt (§8.8, §15.4). **Warum:** Die ersten beiden Schritte liefern bereits die Kernaussage „kein Klartext mehr lesbar“. Die späteren Schritte erhöhen die Belastbarkeit, aber ihr Aufwand hängt an Fragen (Sicherungsmodell, Registersenke), die erst nach P4 sinnvoll beantwortet werden.

**Reihenfolge-Variante „Schlüssel zuerst“** (v6 §11.4): bleibt für besondere Datenkategorien, in v1 aber nicht implementiert. Die Bindungsregel gilt: Ein Schlüssel wird nie vernichtet, solange eine lesbar zu bleibende Version ihn bindet (§8.6).

### 8.6 Mehrbetroffene Dokumente

Aus v6 §11.4.1 unverändert übernommen, weil rechtlich und fachlich tragend: Trockenlauf prüft Mitbetroffene; Modi `DeleteAll` (nur ohne Konflikt), `RedactSplit`, `RestrictShared`; kein automatisches `DeleteAll` nach Timeout; höchstens 64 Betroffene je Version. **Warum:** Dokumente mit mehreren Betroffenen (Verträge, Briefwechsel) sind der Normalfall in der Praxis; eine Löschung für Betroffene A darf Daten von B nicht zerstören.

### 8.7 Crash-Invarianz

Ein Absturz an jedem Durabilitätspunkt führt nach Neustart zur Vollendung der Löschung oder zu `Pending` bei **aktiver** Sperre. Physische Tatsachen (Schlüssel vernichtet) haben Vorrang vor dem Journalstand. Die Prüfung ist eine **Crash-Matrix** (§12), die an jeder Schreibstelle von L1 und L2 abbricht und neu öffnet.

**Warum Vorrang der physischen Tatsachen?** Wenn Journal und Tresor sich widersprechen, ist der Tresor die Wahrheit über Lesbarkeit.

### 8.8 Nicht in v1: Rotation, Sicherungsepochen, Restore-Schutz

Diese Punkte aus v6 §11.3 und §11.4.5 bleiben als **bedingte Zusagen** bestehen (nur mit unabhängigem Löschregister). In v1 gilt stattdessen: **Das Datenverzeichnis wird nicht ohne Keystore gesichert, und ein Wiederherstellen eines alten Standes wendet das Ledger neu an.** Gelingt das nicht (Ledger fehlt oder hat Lücken), startet der Knoten in `Quarantine`. **Warum:** Das schützt die häufigste Fehlbedienung (alten Stand zurückspielen) mit kleinem Aufwand; die vollständige Sicherungsepochenlehre braucht ein Betriebsmodell, das noch nicht existiert.

### 8.9 Indizes: der schwierigste Teil

**Ist:** Volltext-Postings enthalten Token im Klartext, Vektorindizes enthalten Embeddings. Beide sind abgeleitet personenbezogen (Embeddings sind teilweise invertierbar). Schlüsselvernichtung allein macht sie nicht unlesbar (Löschkonzept §4.3).

**Soll v1 [D, D-07]:**
- Jeder Indexeintrag trägt `(doc_id, version)` (Lineage).
- Das Gate filtert Treffer **vor** der Ausgabe (Sperre sofort).
- Physisches Entfernen: Postings und Vektoren des Ziels werden beim **nächsten Segment-Merge oder Neuaufbau** entfernt; ein Auslöser („Bereinigung jetzt“) ist öffentlich über die Fassade erreichbar (das fehlt heute, B-05).
- Stufe **Bereinigt** wird erst gemeldet, wenn der Scanner (§8.10) den Canary im Index nicht mehr findet.

**Alternativen und Verwerfung.** (A) Index nur über verschlüsselte Dokumente und Entschlüsselung je Kandidat: korrekt, aber jede Suche zahlt die Entschlüsselung. (B) Index-Einträge je Dokument verschlüsseln: korrekt, ändert aber das Indexformat und blockiert Skip-Strukturen. Beide kommen frühestens in Stufe 2, weil sie das Persistenzformat ändern. (C) Physisches Neuschreiben ist für v1 der pragmatische Weg; er kostet Schreibverstärkung, ist aber unabhängig von Formatänderungen.

**Offen [E]:** Wie schnell „Bereinigt“ auf großen Beständen erreichbar ist, ist unbekannt. Die Messung gehört in P6.

### 8.10 Messung und Positivkontrolle

Ein Scanner durchsucht **alle** Dateien des Datenverzeichnisses (WAL, SST, Indexsegmente, Temp, Journal, Logs) nach Canary-Kennungen. **Positivkontrolle:** Vor der Sperre wird ein synthetischer Betroffener eingefügt und gefunden; ein Scanner, der ihn nicht findet, liefert `NotImplemented`, nie „bereinigt“ (Z-L6). **Warum:** Ein Scanner ohne Positivkontrolle meldet bei kaputter Implementierung „nichts gefunden“, was als Löschbeweis missverstanden wird. Die Tests T-DEL-1 bis T-DEL-3 sind heute genau so aufgebaut, dass sie ihre eigene Vorbedingung nicht erfüllen (B-05).

### 8.11 Aussagen nach außen

Die Wörter „DSGVO-konform“, „revisionssicher“, „garantiert gelöscht“, „100 % gelöscht“ sind in Code, Doku und README verboten. Zulässig sind Stufenbezeichnungen mit Messergebnis. Rechtlich offen bleibt, ob Schlüsselvernichtung als Löschung im Sinne von Art. 17 DSGVO gilt; Contextra stützt sich deshalb nie allein darauf, sondern weist die Stufen aus **[⚠ Rechtsfrage, keine Rechtsberatung]**. Aufbewahrungspflichten führen zu Einschränkung (Sperre), nicht zu Löschung (Art. 17 Abs. 3, Art. 18 DSGVO).

---

## 9 Vereinbarkeit von Löschung und Zeitreise

Dies ist das Kapitel, das beide Anforderungen auflöst. Grundgedanke: **Geschichte bleibt als Struktur erhalten, Inhalt ist nur mit Schlüssel lesbar, und die Löschung bewegt keinen Zeiger der Geschichte.**

### 9.1 Atomar entkoppelt: was das heißt

- **Entkoppelt:** Der Commit-Pfad (MVCC-Sequenz, Commit-Kette) und der Löschpfad (Ledger, Tresor) haben getrennte Durabilitätspunkte und getrennte Schreiber. Eine Löschung erzeugt **keinen** neuen MVCC-Commit und ändert **keine** Sequenznummer (Z-V1).
- **Atomar:** Der Commit-Punkt einer Löschung ist **ein** Ledger-`fsync` (L1). Davor ist nichts gelöscht, danach ist alles gesperrt. Es gibt keinen Zwischenzustand, in dem ein Lesepfad Klartext liefert.
- **Warum zwei Pfade?** Würde die Löschung als MVCC-Commit laufen, verlangte sie eine neue Sequenznummer, und die Zeitreise könnte „den Stand vor der Löschung“ lesen, also genau das, was gelöscht werden soll. Würde der Commit an die Löschung gekoppelt, hinge die Haltbarkeit der Geschichte an der Garbage Collection.

### 9.2 Konfliktmatrix

| ID | Ist-Konflikt | Beleg | Soll-Auflösung | Warum | Test |
| --- | --- | --- | --- | --- | --- |
| V-01 | Tombstone lässt Klartext in SST und WAL zurück | B-04, B-05 | Payload nur als Chiffretext (P2); Löschung vernichtet Schlüssel | Löschung hängt nicht mehr an Kompaktion (P3) | T-DEL-1 mit Compaction und Positivkontrolle |
| V-02 | `as_of` ist Metadatenfilter und liest keine Historie | B-08 | `as_of` löst zu `seq_no` und liest per `get_at_seq` | P4 | `tt_asof_returns_historic_state` |
| V-03 | Snapshot-Pin verhindert GC alter Werte | B-10 | Pins verzögern nur den **Platz**, nicht die Löschwirkung; Pin-Dauer begrenzt (`reap_expired_pins`) | Chiffretext ohne Schlüssel ist ohne Wert; GC wird zur reinen Speicherfrage | `tt_pin_does_not_block_erasure` |
| V-04 | Checkpoint-Hardlinks klonen Klartext | B-11 | Pausiert; bei Reaktivierung nur Chiffretext, keine Schlüssel im Klon | Ein Klon mit Schlüsseln stellt gelöschte Daten wieder her | `restore_reapplies_ledger` (Stufe 3) |
| V-05 | Indexe tragen Ableitungen alter Versionen | §8.9 | Lineage `(doc_id, version)`; Gate-Filter; Neuaufbau | Sperre sofort, Bereinigung messbar | `idx_canary_removed_after_purge` |
| V-06 | `as_of` auf gelöschte Version: stiller Rückfall auf ältere Version? | Neu | **`as_of` überspringt nie.** Eine gelöschte Version liefert `Erased`. | Ein Rückfall würde einen veralteten Stand als „den Stand zum Zeitpunkt“ ausgeben und den Eindruck erwecken, es gäbe nichts zu löschen | `tt_asof_on_erased_returns_erased` |
| V-07 | Nebenläufigkeit: Löschung während offenem Snapshot | B-10 | Gate wirkt im **aktuellen** Stand (§8.4) | Offene Pins dürfen keine Löschlücke schaffen | `tt_open_snapshot_sees_erased` (loom) |
| V-08 | Eine Version bleibt, andere wird gelöscht | Neu | Bindungsregel (§8.5); `RedactSplit` legt vor dem Vernichten eine neue Version ohne Betroffenen an | Mehrbetroffene Dokumente | `erase_redactsplit_keeps_others` |
| V-09 | Löschung ändert Commit-Hashes oder IDs | v6 INV-14 | Verboten (Z-V1) | Prüfbarkeit der Historie (Commit-Kette bleibt gültig) | `erase_leaves_commit_chain_intact` |

### 9.3 Was nach einer Löschung bei Zeitreise sichtbar bleibt

Sichtbar: Existenz einer Version, Sequenznummer, Zeitpunkt, Inhaltsgröße gerundet (Z-E2 verlangt keine exakten Längen), Klasse und `Erased`-Status.
Nicht sichtbar: Inhalt, daraus abgeleitete Treffer, Embeddings, Texte.

**Warum nicht auch die Existenz verbergen?** Aufbewahrungs- und Nachweispflichten verlangen oft, dass ein Löschereignis dokumentiert ist, und die Commit-Kette bleibt nur prüfbar, wenn Lücken sichtbar bleiben.

---

## 10 Zusagen (testbar)

Jede Zusage hat einen Testnamen; Status **offen** bis der Test läuft und grün ist.

| ID | Zusage | Test | Status |
| --- | --- | --- | --- |
| Z-L1 | Nach Rückkehr von L1 liefert kein Lesepfad Klartext des Ziels, auch nicht `as_of`, Export, Vorschau, Sitzung. | `erase_no_read_path_leaks` (je Pfad) | offen (B-03) |
| Z-L2 | Persistierte Daten der Klasse `Personal` liegen unter AEAD; Canary-Scan über **alle** Dateien findet nichts nach L3. | `tdel1_canary_scan_all_files` | offen (B-05) |
| Z-L3 | Der Keystore übersteht einen Neustart; ein Neustart ändert nie die Lesbarkeit. | `keystore_survives_restart` | offen (B-07) |
| Z-L4 | Crash-Invarianz an jedem Durabilitätspunkt von L1 und L2. | `crash_matrix_erasure` | offen |
| Z-L5 | Fail-closed: Fehler im Schlüssel-, Zufalls- oder Siegelpfad erzeugen keine Ersatzwerte. | `fault_injection_keys` | offen |
| Z-L6 | Nachweise nur mit Positivkontrolle; unimplementierte Messung liefert `NotImplemented`. | `scanner_mutated_must_fail` | offen |
| Z-L7 | Mehrbetroffene: kein automatisches `DeleteAll`. | `erase_multi_subject_modes` | offen |
| Z-L8 | Profile, die `CryptoShred` deklarieren, scheitern beim Öffnen ohne aktive Verschlüsselung. | `profile_cryptoshred_requires_encryption` | offen (B-06), sofort umsetzbar |
| Z-V1 | Löschung ändert keine Sequenznummern, IDs, Commit-Hashes. | `erase_leaves_commit_chain_intact` | offen |
| Z-V2 | `as_of(S)` liefert den Stand zu `seq_no(S)`; spätere Commits ändern frühere Antworten nicht (außer durch Löschung). | `tt_history_stable` (Eigenschaftstest) | offen |
| Z-V3 | `as_of` auf gelöschte Version liefert `Erased`, nie einen Rückfall. | `tt_asof_on_erased_returns_erased` | offen |
| Z-V4 | Offene Snapshots und Pins verhindern die Löschwirkung nicht. | `tt_pin_does_not_block_erasure`, `tt_open_snapshot_sees_erased` | offen |
| Z-V5 | Gleicher Indexstand und gleiche Parameter ergeben gleiche Rangfolge. | `rank_deterministic_same_state` | offen (D-06) |
| Z-Q1 | Retrieval-Qualität (Recall@10, nDCG@10, MRR) ist reproduzierbar messbar und fällt nicht unter die Baseline. | `bench_retrieval_quality` | offen (B-14) |

---

## 11 Umsetzungsplan: vom Ist zum Soll

Aufwand: S = Tage, M = 1 bis 2 Wochen, L = mehrere Wochen (Schätzungen [E] für einen Entwickler). Reihenfolge ist verbindlich: Jede Phase endet mit grünem `cargo check --workspace --all-targets --locked`.

### Phase P0: Boden (S)

| AP | Inhalt | Abnahme | Warum |
| --- | --- | --- | --- |
| P0.1 | Wurzel-`Cargo.toml` committen; `members` nach Anhang B | Frischer Klon baut (`cargo check --locked`) | B-01 |
| P0.2 | CI-Bypässe (`\|\| true`, `continue-on-error`) entfernen | Fehlerhafter Check bricht CI | B-17 |
| P0.3 | Fix-Branch auffinden oder neu erzeugen; Reports mit HEAD-Hash versehen | Report-Stand = Repository-Stand | B-16 |
| P0.4 | `deletion_proof_active`-Flag und `EnterpriseRegulated`-Profil entschärfen: Flag entfernen oder mit realer Bedeutung füllen; Profil scheitert ohne Verschlüsselung (Z-L8) | `profile_cryptoshred_requires_encryption` grün | B-06, P7; sofortige Rücknahme falscher Signale |

### Phase P1: Fokus (M)

| AP | Inhalt | Abnahme | Warum |
| --- | --- | --- | --- |
| P1.1 | Crates nach §4.2 pausieren (Tag, aus `members`) | Build ohne pausierte Crates grün | P6 |
| P1.2 | `db`→`graph`/`cognition`-Kanten hinter Features legen | `cargo check -p contextra-db --no-default-features` | Abkoppeln ohne Funktionsverlust |
| P1.3 | `ingest` auf Text, Markdown, PDF reduzieren; Rest hinter Features oder pausiert | `ingest` hat Verbraucher oder fliegt raus | B-02-Logik |
| P1.4 | Robotik-Reste benennen und entkoppeln (`pid_latency_controller`, `homeostat`, `resonance`, `saos`) | Nicht im Default-Build | §1.3 |

### Phase P2: Tests wieder aussagekräftig (M bis L)

| AP | Inhalt | Abnahme | Warum |
| --- | --- | --- | --- |
| P2.1 | Test-Targets in `vector`, `store`, `db`, `text`, `checkpoint`(falls aktiv) kompilierbar machen; `required-features` pro Target (D-11) | `cargo test --workspace --locked` startet | Ohne ausführbare Tests sind Zusagen unbelegbar |
| P2.2 | `DirLock`-Lebenszyklus: `LsmStorage::drop` wartet auf Compaction-Task (B-15) | `persistence_test` grün | Wiederöffnen ist Voraussetzung für jeden Crash-Test |
| P2.3 | `bench` bereinigen: Feature `datasets` entfernen oder Datensatz liefern; kleiner Gold-Satz (200 Frage–Dokument-Paare) im Repository | `cargo bench`/`just eval` liefert Recall@10, nDCG@10, MRR | B-14, Z-Q1 |

### Phase P3: Verdrahten und Entscheiden (M)

| AP | Inhalt | Abnahme | Warum |
| --- | --- | --- | --- |
| P3.1 | ADR zu §13 (Master-Key, Verschlüsselungsort, Index, Gruppenmodell) als `docs/decisions/ADR-0001..0004` | ADRs liegen vor, widerspruchsfrei zu §8 | Blockierende Entscheidungen vorab |
| P3.2 | `erasure` kompilierbar: doppelte Traits (`FenceGate`, `WalSink`) zusammenführen, Proof-Tests auf aktuelle API | `cargo check -p contextra-erasure --all-targets` | B-02 |
| P3.3 | `erasure` in `default-members` und als Abhängigkeit von `db` | `db` kompiliert mit `erasure` | B-02, B-03 |
| P3.4 | Einen Tresor auswählen (`erasure::vault` oder `contextra-vault`), den anderen pausieren | Genau eine Implementierung | B-12 |
| P3.5 | `durable-fs` an Tresor und Ledger anbinden | Ledger und Tresor schreiben über `durable-fs` | Haltbarkeit |

### Phase P4: Schlüssel und Schreibpfad (L)

| AP | Inhalt | Abnahme | Warum |
| --- | --- | --- | --- |
| P4.1 | Persistenter Keystore (§8.3): Passphrase→Argon2id→Master; Container versioniert; fail-closed | `keystore_survives_restart`, `fault_injection_keys` | B-07 |
| P4.2 | `revoke_record(&str)` entfernen, `revoke_record_num` konsolidieren; Revocation-Log verdrahten; `Erased`-Fehlerkategorie einführen | Log wird beim Öffnen geladen | N2, N3, N9 |
| P4.3 | Schreibpfad: Payload vor WAL/SST verschlüsseln (AEAD, AAD nach §8.2); Klartext-Verbot in Klasse `Personal` erzwingen | `tdel1_canary_scan_all_files` vorbereitend (Canary nur als Chiffretext) | B-04, P2 |
| P4.4 | Update erzeugt neue Version mit neuem DEK; alter DEK wird nicht automatisch gelöscht, sondern über Ledger | `version_new_dek` | Löschkonzept AP5 |
| P4.5 | `DocId`-Kollisionsprüfung (§6.2) | `docid_collision_rejected` | B-13 |
| P4.6 | `PersonalRef` in `ids` (128 Bit, zufällig, Golden-Vektoren) | Golden-Test | §6.1 |

### Phase P5: Lesepfad und Zeitreise (L)

| AP | Inhalt | Abnahme | Warum |
| --- | --- | --- | --- |
| P5.1 | `Reader` als einziger Lese-Einstieg in `db` (§8.4), **objektgebunden** (§15.2) und mit `AccessContext`-Parameter (§15.1); Gate-Prüfung vor Entschlüsselung | `read_chokepoint`, `erase_no_read_path_leaks` | B-03, P5 |
| P5.2 | `ValidityWindow` nach `mvcc` verlegen; `temporal_filter` auf typisierte Felder; Fail-Open entfernen | Golden-Test für Intervallgrenzen | B-08, B-09 |
| P5.3 | `as_of` über `seq_no` und `get_at_seq` (§7.2); Tabelle `tx_time → seq_no` | `tt_asof_returns_historic_state`, `tt_history_stable` | B-08 |
| P5.4 | `as_of` auf gelöschte Version liefert `Erased` (V-06) | `tt_asof_on_erased_returns_erased` | Z-V3 |
| P5.5 | Pin-Dauer begrenzen; Pins ohne Einfluss auf Löschwirkung testen | `tt_pin_does_not_block_erasure`, `tt_open_snapshot_sees_erased` (loom) | V-03, V-07 |

### Phase P6: Löschkette vollständig (L)

| AP | Inhalt | Abnahme | Warum |
| --- | --- | --- | --- |
| P6.1 | Löschablauf L1 bis L4 (§8.5) über `db::delete_op`; Fassadenfunktion `purge_now` | `erase_full_chain`, `tdel1_canary_scan_all_files` | B-04, B-05 |
| P6.2 | Index-Lineage und Bereinigung (§8.9) | `idx_canary_removed_after_purge` | V-05 |
| P6.3 | Scanner mit Positivkontrolle, signierter Bericht, Stufen im Bericht | `scanner_mutated_must_fail` | P7, Z-L6 |
| P6.4 | Mehrbetroffene Modi (§8.6) | `erase_multi_subject_modes`, `erase_redactsplit_keeps_others` | Z-L7 |
| P6.5 | Crash-Matrix (§8.7) | `crash_matrix_erasure` grün | Z-L4 |
| P6.6 | Messung: Zeit bis „Bereinigt“ auf 10k, 100k Dokumente; Messbericht mit HEAD-Hash | Bericht unter `reports/` | §8.9 offen |

### Phase P7: Release v1 (S)

Aufgaben: Wortfilter für Compliance-Aussagen, README an Stufenmodell anpassen, Messbericht, alle Zusagen aus §10 grün oder ausdrücklich als `offen` im Release-Text.

### Phase P8 bis P11: Stufen 2 bis 4 (Rahmen, noch ohne Arbeitspakete)

| Phase | Stufe | Einstiegsbedingung | Erste Arbeitspakete (Rahmen) |
| --- | --- | --- | --- |
| P8 | 2 Ontologie und Integration | Alle Zusagen §10 grün; Gate und Lineage objektgebunden (§15.2) | `graph` an `Reader` und Lineage anbinden; `rules` aus v6 §12 (Commit-Graph, Takedown) neu bewerten; Integrationsschnittstellen für Fachsysteme definieren |
| P9 | 3 Entscheidungshilfe | P8 abgeschlossen | LLM-Anbindung mit Zugriffsprüfung vor dem Modell; `kvcache` verschlüsseln und löschbar machen; Agenten mit Budgets; Simulation (v6 §13) |
| P10 | 4 Betrieb | P9 abgeschlossen | Rollen- und Zweckmodell (§15.1); Aufbewahrungsprüfung (§15.4); Prüfpfad und Berichte |
| P11 | Zertifizierung | Pilotbetrieb | Nach Zielzertifikat (§15.5) |

Zusagen für diese Phasen entstehen **erst mit ihren Tests** (P8). v6 §12 und §13 bleiben bis dahin als Entwurfsgrundlage unter `docs/archive/SPEC_v6.md` erhalten.

### Reihenfolge-Begründung

P0 vor allem, weil ohne Build nichts prüfbar ist. P1 vor P2, weil weniger Fläche weniger Testarbeit bedeutet. P2 vor P3, weil `erasure` nur in einem grünen Workspace getestet werden kann. P4 vor P5, weil der Lesepfad Chiffretext voraussetzt. P5 vor P6, weil die Löschkette den `Reader` braucht. P3.1 (ADR) liegt vor P4, weil Master-Key und Verschlüsselungsort sonst im Schreibpfad nachträglich geändert werden müssten.

---

## 12 Gates (sieben laufende, blockierend)

| Gate | Prüft | Ersetzt in v6 |
| --- | --- | --- |
| G1 Build | `cargo check --workspace --all-targets --locked` | G-L3, INV-4 |
| G2 Test | `cargo test --workspace --locked` (inklusive `required-features`-Läufe) | Diverse |
| G3 Schichtung | `cargo metadata`-Test nach R1/R2 plus `cargo-deny` | G-L1, G-L2, G-L4 |
| G4 Löschung | `tdel1_*`, `erase_*`, `crash_matrix_erasure` | G-ER1 bis G-ER3 |
| G5 Zeitreise | `tt_*` | G-TT1 |
| G6 Wortfilter | Verbotene Compliance-Aussagen in Code und Doku | G-DOC1, INV-7 |
| G7 Qualität | `bench_retrieval_quality` gegen Baseline | neu (B-14) |

Alle Gates laufen ohne `|| true`. **Warum sieben?** Jedes Gate hat einen Test, der in CI läuft. Gates ohne laufenden Test entfallen (D-04). Das Ratchet-Prinzip (Werte dürfen nur besser werden) bleibt für `types_fan_in`, `ports_fan_in` und die Zahl der Unsafe-Blöcke außerhalb von `simd`, `sys`, `wire`.

---

## 13 Entscheidungsregister

| ID | Entscheidung | Alternativen | Begründung | Folge |
| --- | --- | --- | --- | --- |
| D-01 | Stufe 1 = lokales Hybrid-RAG mit löschbarem, versioniertem Gedächtnis als Fundament der Zielplattform | Baukasten sofort; alles gleichzeitig | Querschnittsthemen werden einmal gelöst, bevor Stufen 2 bis 4 darauf bauen (§1.4) | Anhang A |
| D-02 | Master-Schlüssel aus Passphrase (Argon2id); Keyring/TPM optional; kein KMS | Keyring/TPM Standard; KMS | Offline, plattformunabhängig, zum lokalen Betrieb passend | Verlust der Passphrase = Totalverlust, wird in der Doku benannt |
| D-03 | Envelope-Verschlüsselung auf Dokumentebene in `db`, AAD `(tenant, collection, user_key, version)` | Verschlüsselung im Store-Blockpfad | Store kennt keine Betroffenen und Versionen | Store hält Chiffretext, Gate in `db` |
| D-04 | Aussagen ohne laufenden Test sind SOLL | Aussagen ohne Test als MUSS | P8 | Zusagen in §10 starten als „offen“ |
| D-05 | Das Gate wirkt im aktuellen Stand, nicht im Snapshot-Stand | Gate im Snapshot-Stand | Löschung muss rückwirkend gelten | Offene Pins können Löschung nicht umgehen |
| D-06 | Determinismus: gleiche Rangfolge bei gleichem Indexstand; keine bitgleichen Scores über Architekturen | Bitgenauigkeit über x86/aarch64 | Gleitkomma ist für Suche nötig; Bitgleichheit ohne Nutzerwert | Z-T1 aus v6 entfällt |
| D-07 | Index v1: Lineage, Gate-Filter, Neuaufbau (Option C); Optionen A und B frühestens Stufe 2 | A oder B in v1 | A und B ändern das Format und kosten Suche | Zeit bis „Bereinigt“ wird gemessen (P6.6) |
| D-08 | Ein Tresor: `erasure::vault` (Vorschlag), `contextra-vault` wird pausiert, falls kleiner und redundant | `contextra-vault` als Besitzer | `erasure::vault` ist schon eng mit Gate, Ledger und Journal verzahnt [E, vor P3.4 zu prüfen] | Entscheidung in P3.4 mit Begründung |
| D-09 | Pausieren per Tag, `members`-Ausschluss, Verzeichnis nach `archive/` | Löschen | Umkehrbar und nachvollziehbar | Eintrag in Anhang A |
| D-10 | Regelwerk auf R1–R5 reduziert; Entnehmbarkeits-Score und Baukasten-Artefakte entfallen in v1 | Beibehalten | Kein Auslieferungsziel „Baukasten“ in v1 | v6 §2, §5.4, §8 entfallen |
| D-11 | Test-Targets mit `required-features`; CI ruft Default-Lauf und `--features workspace`-Lauf | Features im Justfile global | Entscheidung, die der Fix-Report offen ließ | P2.1 |
| D-12 | Zeitreise Stufe 1: `as_of` über MVCC-Sequenz plus typisierte Gültigkeitszeit in `mvcc`; Branching und „Git für Regeln“ in Stufe 2 (`rules`) | Commit-Graph sofort | Das Regelwerk ist Teil der Zielplattform, aber nur auf einem funktionierenden Fundament baubar | P8 |
| D-13 | Zeitreise überspringt nie gelöschte Versionen | Rückfall auf ältere Version | Verhindert veraltete Antworten | `Erased` im Ergebnis |
| D-14 | Standard-Klasse einer Collection ist `Personal` | `NonPersonal` als Standard | Fehlklassifikation „unkritisch“ ist teurer | Betreiber muss `NonPersonal` bewusst setzen |
| D-16 | Zielbild: souveräne, offline betreibbare Entscheidungsplattform für Unternehmen und Behörden mit lokalen LLMs | Reines RAG; Cloud-Plattform | Marktlücke Souveränität; passt zu Löschung × Zeitreise als Alleinstellung | Stufenplan §1.4 |
| D-17 | Entscheidungsvorbereitung, keine automatisierte Entscheidung über Personen | Vollautomatische Entscheidungen | Art. 22 DSGVO, Haftung, Akzeptanz | Pflichtfelder: Quellen, Version, Unsicherheit |
| D-18 | Kein Training oder Fine-Tuning mit personenbezogenen Daten | Fine-Tuning auf Kundendaten | Gewichte lassen sich nicht löschen | Wissenszufuhr nur per Retrieval |
| D-19 | Reader, Gate und Lineage sind objektgebunden (Dokument, Knoten, Kante, Ableitung) | Nur dokumentgebunden | Verhindert zweite Lesepfade in Stufe 2 | `AccessContext` im Gate ab P5.1 |
| D-20 | Robotik- und Regelkreiscode wird per Feature `legacy-control` ausgeschlossen, später entfernt | Mitführen; sofort löschen | Zustandsabhängiges Verhalten stört Nachvollziehbarkeit; einzelne Teile können in Stufe 3 nützlich sein | §4.4 |
| D-21 | Produktaussagen: „unterstützt datenschutzgerechten Betrieb“, nie „DSGVO-konform“ | „DSGVO-konform“ | Bewertung liegt beim Verantwortlichen; Wettbewerbs- und Haftungsrisiko | G6 Wortfilter |
| D-15 | Embedder: `infer-ollama`; `infer-onnx` als Alternative | `infer-candle`, beide | Ein Embedder genügt; Ollama ist lokal und ohne ONNX-Laufzeit [E] | Abgleich mit Qualitätsmessung P2.3 |

---

## 14 Risiken und offene Fragen

| Risiko | Auswirkung | Gegenmaßnahme |
| --- | --- | --- |
| Zeit bis „Bereinigt“ auf großen Beständen unbekannt (§8.9) | Löschfrist (Art. 12 DSGVO: grundsätzlich ein Monat) könnte gerissen werden | Messung P6.6; früh messen, nicht erst vor dem Release |
| Verlust der Passphrase = Totalverlust | Datenverlust | Betreiber-Dokumentation; optional zweiter Wrapper (Wiederherstellungsschlüssel, lokal) |
| Embedding-Inversion: Vektoren bleiben bis zur Bereinigung abgeleitet personenbezogen | Restrisiko zwischen L2 und L3 | Gate-Filter sofort; Bereinigung zeitnah; Stufenausweis ehrlich |
| `store` (60k Zeilen) hat nicht kompilierende Tests; unbekannte Defekte im Kernpfad | Verzögerung P2 | Früh P2.1, mit Triage statt Vollreparatur: erst kompilieren, dann Rote analysieren |
| Rechtliche Einordnung der Schlüsselvernichtung offen | Zusage könnte zu weit gehen | Stufenmodell; Verbot der Pauschalaussagen (§8.11); Beratung durch Fachjurist |
| `erasure`-Crate hat 32k Zeilen, vermutlich mehr als v1 braucht | Wartungslast, Fehleroberfläche | In P3.2 bewerten, welche Module (`restore`, `scope`, `supersede`, `measure`, `report`) v1 braucht; Rest hinter Feature oder pausieren |
| Zielbild ist größer als ein Einzelentwickler leisten kann (§15.6) | Stillstand in Stufe 1 | Stufen einzeln einsetzbar; Pilotanwender; harte Einstiegsbedingungen |
| Rechtliche Anforderungen von Behörden (KI-Verordnung, Verfassungsrecht, Archivrecht) verschieben Anforderungen | Nacharbeit | Juristische Klärung vor P10 [⚠] |
| Eigentümer pflegt allein; Bus-Faktor 1 | Wissensverlust | ADRs und Testnamen als Gedächtnis; Berichte mit HEAD-Hash |

---

## 15 Anforderungen aus dem Zielbild (Unternehmen und Behörden)

Dieses Kapitel hält fest, was die Stufen 2 bis 4 zusätzlich verlangen und **was das Fundament (Stufe 1) heute schon dafür vorbereiten muss**. Rechtliche Aussagen sind [⚠] und vor Einsatz durch eine Fachjuristin oder einen Fachjuristen zu prüfen.

### 15.1 Zugriff, Zweckbindung, Mandanten (Stufe 4, Vorbereitung in Stufe 1)

**Ist:** Es gibt Mandantentrennung (`TenantPolicy`, `TenantId`) **[V]**. Ein Rollen-, Attribut- oder Zweckbindungsmodell habe ich im Code nicht gefunden (`grep` nach `rbac`, `abac`, `Role`, `Permission`, `purpose` ergab keine Domänentypen) **[V]**.

**Soll:** Zugriffsentscheidungen berücksichtigen **Person, Rolle, Zweck und Objekt**. Zweckbindung (Art. 5 Abs. 1 lit. b DSGVO) ist in Behörden und Unternehmen der Regelfall, nicht die Ausnahme. Die Entscheidung fällt **vor Suche und Statistik** (Z-E8), damit Rangfolge, Trefferzahl und Fehlerbild nicht über unzugängliche Objekte Auskunft geben.

**Warum jetzt festhalten?** Das Gate aus §8.4 ist der natürliche Ort. Wenn es heute nur „gesperrt: ja/nein“ kennt, muss es später mit Rolle und Zweck neu gebaut werden. Deshalb gilt für Stufe 1: `ReadGate::check` bekommt einen **Kontext-Parameter** (`AccessContext`, zunächst leer). Das kostet nichts und hält den Weg offen.

### 15.2 Objekte, Beziehungen, Herkunft (Stufe 2, Vorbereitung in Stufe 1)

**Soll:** Ontologie-Objekte (Personen, Organisationen, Verträge, Vorgänge) und Beziehungen sind Daten der Klasse `Personal`, sobald sie Personenbezug haben. Sie unterliegen **denselben** Mechanismen wie Dokumente: Verschlüsselung, Version, Gate, Löschung, Lineage.

**Entscheidung für Stufe 1 (D-19):** `Reader`, Gate und Lineage werden **nicht dokumentgebunden**, sondern **objektgebunden** entworfen: Ein Dokument ist ein Objekt unter mehreren. **Warum:** Wird das Gate zuerst nur für Dokumente gebaut, entstehen später für Graphknoten und -kanten eigene Lesepfade, und genau das widerlegt P5 und B-03.

**Herkunft (Lineage):** Jede Ableitung (Index, Graphkante, Embedding, Zusammenfassung, Modellantwort) trägt `(quell_objekt, quell_version)`. Ohne Lineage ist eine Löschung nicht vollständig messbar und eine Entscheidungshilfe nicht begründbar.

**Extraktion aus Text** (Entitäten und Beziehungen per Regel oder Modell) erzeugt **neue personenbezogene Daten**; sie sind Ableitungen und müssen mit der Quelle gelöscht werden (V-05).

### 15.3 Lokale LLMs (Stufe 3)

Lokale Modelle lösen das Abflussproblem (kein Fremdanbieter), schaffen aber neue Speicherstellen:

| Speicherstelle | Personenbezug | Soll |
| --- | --- | --- |
| Prompt und Antwort (Sitzungsverlauf) | ja | Ableitung mit Lineage; Löschung folgt der Quelle |
| KV-Cache (`kvcache`) | ja (enthält Kontext) | verschlüsselt und löschbar; Cache-Segmente tragen Lineage |
| Embeddings | ja (teilweise invertierbar) | siehe §8.9 |
| **Modellgewichte (Fine-Tuning)** | **nicht löschbar** | **verboten für personenbezogene Daten (D-18)**: Man kann aus Gewichten keinen Betroffenen entfernen. Wissen wird per Retrieval zugeführt, nicht eintrainiert. |
| Retrieval-Kontext | ja | Gate **vor** der Zusammenstellung; Quellen werden in der Antwort mit Version angegeben |

**Zugriffsprüfung vor dem Modell.** Das Modell sieht nur, was die anfragende Person sehen darf. Eine nachträgliche Filterung der Antwort genügt nicht, weil das Modell Inhalte paraphrasiert.

**Prompt-Injection.** Dokumentinhalte sind nicht vertrauenswürdig und dürfen keine Werkzeugaufrufe auslösen, die über die Rechte der anfragenden Person hinausgehen (`prompt-guard`, `sandbox`).

**Entscheidungshilfe, nicht Entscheidung (D-17).** Ergebnisse tragen Quellen, Version, Unsicherheit und ausdrücklich den Hinweis „Vorschlag“. Eine automatisierte Entscheidung mit rechtlicher Wirkung gegenüber Personen ist ausgeschlossen (Art. 22 DSGVO). [⚠] Behörden unterliegen zusätzlich der KI-Verordnung; Hochrisiko-Einordnungen (u. a. bei Verwaltungsentscheidungen über Leistungen) sind vor Einsatz zu klären.

### 15.4 Aufbewahrung, Archivierung und Löschung (Stufe 4, Vorbereitung in Stufe 1)

**Konflikt:** Behörden und Unternehmen unterliegen **Aufbewahrungs- und Archivierungspflichten**. Eine Löschbitte nach Art. 17 kollidiert damit regelmäßig; Art. 17 Abs. 3 und Art. 18 DSGVO führen dann zu **Einschränkung (Sperre)**, nicht zu Löschung. [⚠]

**Soll:** Jede Löschanfrage durchläuft zuerst die **Aufbewahrungsprüfung** (Hold, Frist, Archivpflicht) und endet, wenn etwas dagegen spricht, in `Restricted` mit Grund und Frist. Die Fristen sind **Konfiguration des Verantwortlichen**, nicht Programmlogik. Nach Fristablauf löst ein Prüfauftrag, kein Automatismus, die Löschung aus (Z-L7).

**Warum so?** Ein System, das auf Zuruf löscht, verletzt Aufbewahrungspflichten; ein System, das nie löscht, verletzt Art. 17. Das Stufenmodell (§8.5) trägt beides, weil Sperre (L1) und Vernichtung (L2) getrennte Schritte sind.

### 15.5 Zertifizierung und Betrieb (Stufe 4)

- **Offline-Betrieb:** Updates als signierte Pakete; keine Rückmeldung an den Hersteller (Z-E12 aus v6).
- **Prüfpfad:** Manipulationssicherer Ledger (vorhanden in `erasure`); Export über `audit-export`.
- **Nachweise für Verantwortliche:** Verzeichnis von Verarbeitungstätigkeiten, Auftragsverarbeitungsvertrag (`avv-generator`) und Löschbericht sind **Hilfen**, keine Rechtsberatung.
- **Zertifizierung** (z. B. BSI-Grundschutz, ISO/IEC 27001, Common Criteria für Behörden) [⚠] ist eine Organisationsfrage und setzt reproduzierbare Builds, Abhängigkeitsprüfung und Dokumentation voraus. Das stützt die Linie „ein Build, der grün ist“ (P1) bereits heute.
- **Sicherheitsbehörden:** Der Einsatz automatisierter Datenanalyse bei Polizei und Nachrichtendiensten unterliegt engen verfassungsrechtlichen Grenzen [⚠, vor Planung juristisch klären]. Das Zielbild beginnt deshalb bei **Verwaltung und Unternehmen**.

### 15.6 Lieferbarkeit des Zielbilds

Ein Plattformumfang in der Funktionsklasse von Foundry ist ein Mehrjahresprojekt. Für einen Einzelentwickler ist er nur tragbar, wenn (a) jede Stufe **einzeln einsetzbar** ist, (b) ein **Pilotanwender** je Stufe die Anforderungen schärft, und (c) das Fundament die teuren Querschnittsthemen (Verschlüsselung, Sperre, Zeit, Lineage) **einmal** löst. Die Gewichtung der Entwicklungszeit: Stufe 1 trägt den Großteil des Risikos; Stufen 2 bis 4 nutzen sie.

---

## Anhang A Disposition v6 → v7 (Nachvollziehbarkeit)

| v6-Element | v7 | Begründung |
| --- | --- | --- |
| Kapitel 2 bis 5 (Ebenen E0 bis E3, Kern-Closure, Brücken, Bausteinvertrag) | §4 (drei Schichten, R1 bis R5) | Baukasten-Auslieferung ist kein v1-Ziel (D-10). Brücken (`compat/`) bleiben als Technik bestehen, werden aber nicht mehr als Zusage geführt. |
| Kapitel 6 (Golden-Korpus, ID-Breite) | Zusage nur dort, wo Formate geändert werden (P4, P5) | Formatänderungen erzeugen Golden-Dateien; ohne Formatänderung braucht es keinen Gesamtkorpus. |
| Kapitel 7 (Gates) | §12 (sieben Gates) | Laufende Gates statt Baseline. |
| Kapitel 8 (Score M, Entnehmbarkeit) | entfällt | Messgröße für Baukasten. |
| Kapitel 9 (Entkopplungsplan S-xx) | §11 (Phasen P0 bis P7) | Alter Plan misst Entkopplung; neuer Plan misst Löschung und Zeitreise. |
| Kapitel 11 (Löschkette L0 bis L6, Stufen) | §8 (L1 bis L4) | Verkürzt auf das, was die Kernaussage trägt; Rotation und Sicherungsepochen → Stufe 4. |
| Kapitel 12 (Git für Regeln, `contextra-rules`) | §7.4; nicht in v1 | Eigenes Produkt (D-12). Das Crate `rules` wird nicht angelegt. |
| Kapitel 13 (Simulation) | nicht in v1 | Setzt Kapitel 12 voraus. |
| Kapitel 14 (Prüfer `contextra-verify`) | nicht in v1 | Setzt signierte Berichte voraus (P6.3); danach prüfbar. |
| Kapitel 15 (S-100 bis S-111) | in §11 aufgegangen | Überschneidung. |
| INV-1 bis INV-16 | R1 bis R5, §10 | Formatgleichheit gilt nur noch bei Formatänderung; Ratchet bleibt. |
| Crates `graph`, `cognition`, `agent`, `engine`, `router`, `adapt`, `checkpoint`, `kvcache`, `infer-candle`, `simd`, `license`, `audit-export`, `avv-generator`, `sandbox`, `py`, `prompt-guard`, `infer-onnx` | pausiert (§4.2) | Siehe §4.2 je Crate. |

**Rückkehrkriterium (Zuordnung zu Stufen 2 bis 4):** Ein zurückgestelltes Crate kehrt zurück, wenn (a) Stufe 1 mit allen Zusagen aus §10 grün ist, (b) es einen Verbraucher in der Ziel-Stufe hat, und (c) es im Build mit grünen Tests steht.

## Anhang B Vorschlag für `members` der Wurzel-`Cargo.toml`

```toml
[workspace]
resolver = "2"
members = [
  # Fundament
  "crates/contextra-ids", "crates/contextra-env", "crates/contextra-sys",
  "crates/contextra-durable-fs", "crates/contextra-crypto", "crates/contextra-privacy",
  "crates/contextra-types", "crates/contextra-ports",
  # Speicher & Index
  "crates/contextra-mvcc", "crates/contextra-store", "crates/contextra-text",
  "crates/contextra-vector", "crates/contextra-rank", "crates/contextra-erasure",
  # Fassade & Schnittstelle
  "crates/contextra-db", "crates/contextra", "crates/contextra-mcp",
  "crates/contextra-infer-ollama", "crates/contextra-testkit",
]
# Tabelle B-12: genau einer der beiden Tresore (D-08) kommt hinzu, der andere bleibt draußen.
```

**Achtung [E]:** `db` hängt heute an `graph`, `cognition`, `checkpoint`, `adapt`, `sandbox`, `infer-onnx` und `vault` (Cargo-Auswertung). P1.2 muss diese Kanten hinter Features legen, **bevor** die Liste oben baut. Die Liste ist das Ziel, nicht ein sofort baubarer Stand.

## Anhang C Messmethodik dieser Revision

1. `git clone` (Shallow), HEAD `24487fa`.
2. `git ls-files` für Wurzeldateien und `Cargo.toml`-Zahl.
3. Python-Auswertung aller `crates/*/Cargo.toml` für Verbraucher je Crate (normale und Build-Abhängigkeiten, auch optionale; dev-Kanten nicht gezählt).
4. `find crates -name '*.rs' | xargs wc -l` für Zeilen (inklusive Tests).
5. `grep` auf `ReadGate`, `as_of_timestamp`, `min_for_compaction`, `deletion_proof_active`, `encryption_passphrase`.
6. Lesen der Dateien `delete.rs`, `temporal_filter.rs`, `snapshot.rs`, `tombstone.rs`, `hardlink_cloner.rs`, der Reports und des Löschkonzepts.

**Nicht getan:** Kein Build, kein Test, keine Toolchain-Installation. Befunde mit **[R]** stammen aus Reports und sind nicht reproduziert; sie sind in P2 und P3 zuerst zu verifizieren.
