---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "00d"
---
## Teil A3 — Aktueller Umsetzungsstand & priorisierte Restarbeit (Fassung 3, Quelle der Wahrheit)

> **Geltung:** Dieser Teil führt jeden in Teil A2/§17/§18/§20 als offen (🔴), unverifiziert (🔍) oder als
> Bridge/Stub markierten Punkt gegen den zum Prüfzeitpunkt tatsächlichen Repository-Zustand nach
> (`github.com/tfufuz1/memfuse`, HEAD `d37a70b6`, 2.436 Commits Historie) und übernimmt die Priorisierung der
> externen strategischen Tiefenberatung. Geprüft wurde durch Live-Klon und direkte Verifikation
> (Datei-Existenz, `wc -l`, `grep` auf Funktions-/Typnamen, `git log`-Commit-Nachrichten und Cargo.toml-Inhalte)
> — **nicht** aus Sekundärquellen übernommen, sofern nicht ausdrücklich als „laut Commit-Historie, nicht im
> Detail nachvollzogen" gekennzeichnet. Reifegrad-Marker in §5–§20 sind bei Widerspruch **nachrangig** zu den
> Aussagen dieses Teils (siehe Leitentscheidung 5).

### A3.1 Was seit Fassung 2.1 tatsächlich fertiggestellt wurde (verifiziert 🟢)

| Bereich | Referenz in dieser Spec | Status Fassung 2.1 | Status Fassung 3 (verifiziert) |
|---|---|---|---|
| Produkt-Fassade `remember/recall/forget/relate` | §2, §A2 | 🔴 nicht vorhanden, 50-Zeilen-Skelett | 🟢 `crates/contextra/src/agent_memory.rs` implementiert und über `lib.rs` re-exportiert; `AgentMemory` mit allen vier Methoden |
| `contextra-rank`-Crate (vormals `memfuse-rank`) | §4, §A2.1 | 🔴 existiert nicht | 🟢 eigenes Crate, ~2.012 LOC, im Workspace registriert |
| `contextra-db`-Strangler-Shell | §A2.1 (D#-Tabelle) | 🔴 16.221 LOC, kein reines Re-Export | 🟢 auf 1.849 LOC reduziert (`lib.rs` allein 882 Zeilen), expliziter Commit „reduce contextra-db to pure re-export strangler shell" |
| `contextra-core`-Deprecation | §A2.1 | 🔴 kein `#[deprecated]` | 🟢 20 `#[deprecated]`-Attribute, Crate ist reine Deprecation-Shell (136 Zeilen) |
| MCP-Tool `contextra_forget` (vormals `memfuse_forget`) | §11, §16 (Abnahmekriterien MCP) | 🔴 fehlte | 🟢 implementiert, inkl. Pflichtparameter `confirm: true` als bewusste Sicherheitshürde |
| PyPI-Publish-Workflow-Risiko | §10 (Sicherheitsmodell, Supply Chain) | 🔴 aktiver Tag-Trigger auf `v*.*.*`, höchstes Einzelrisiko laut Vorberichten | 🟢 Trigger entfernt; nur noch `workflow_dispatch` mit Pflichtfeld `confirm_package_name`, geprüft gegen den im Paket deklarierten Namen |
| `default-members` (Workspace) | §0 (Meta) | 🔴 `contextra-infer-onnx` fälschlich enthalten, `contextra-sys`/`contextra-privacy` fehlten | 🟢 bereinigt: `infer-onnx` entfernt, `sys`/`privacy` ergänzt |
| Pflicht-Integrationstests AK-4, AK-14, AK-15 | §15 (Test-Spezifikation) | 🔴 drei von vier AK-Tests fehlten | 🟢 `signal_kind_no_new_variant.rs`, `kv_locks_stable_shard.rs`, `ips_requires_propensity.rs` vorhanden |
| `xtask check-module-reachability` | §15, §A.3 (Gate 0) | 🔴 fehlte | 🟢 implementiert (409+ Zeilen), inkl. Alias `check-orphan-modules` |
| `capabilities.toml` | §14 (Feature-Flag-Politik) | 🔴 fehlte | 🟢 vorhanden im Repo-Root |
| ADRs N01–N10 | §20.3 | 🔴 0 von 10 als Dokument | 🟢 alle 10 unter `docs/decisions/`, plus fünf weitere (insgesamt 15 ADR-Dateien) |
| N-äre Hyperkanten: `HyperEdgeView`, `ArcSlice<T>` (§6.6 H2ff.) | §6, Anhang B §B.6.1.1/H2 | 🔴 spezifiziert, nicht gebaut | 🟢 beide implementiert (`arc_slice.rs`, `hyperedge.rs`), 9 dedizierte Testdateien (Persistenz, Compact-Race, Cascade-Fanout/-Recovery, Memory-Budget, Payload-Sharing, Queue-Persistenz) — Commit „implement n-ary hyperedges, ArcSlice, and Convention K star weight" deutet auf einen gebündelten Umsetzungsschritt für §6.6 H2–H6 hin |
| KV-Cache §9.2 (Prefix-Radix-Baum, RAII-Guards, Tier-2-AEAD, `KvState` Stufe B) | §9.2 | 🔴 nur Platzhalter-String + Zähler (korrigierter 🟢-Widerruf aus Fassung 2) | 🟢 laut Commit-Historie umgesetzt (`a16fd650`, `d37a70b6`); `radix.rs`, `kv_state.rs` (Kommentar referenziert explizit „Spec §9.2 Stufe B"), AEAD-Bezug in `segment.rs` verifiziert vorhanden — **Prefill-Skip-Wirksamkeit selbst nicht nachgemessen, nur Struktur-Existenz verifiziert** |
| `contextra-vector` `forbid`/`allow(unsafe_code)`-Widerspruch (§A2, D#-Tabelle) | §A2.1 | 🔴 E0453, nicht kompilierbar laut Zielarchitektur-Prüfung | 🟡 Datei-Header zeigt keinen offensichtlichen Widerspruch mehr — **Detailprüfung der tatsächlichen `unsafe`-Migration nach `contextra-sys` steht noch aus, daher 🟡 statt 🟢** |
| Layering-Test-Schärfe (§4, P5) | §4, §15 | 🔍 Modus unklar | 🟢 Test läuft im „0 unallowlisted violations"-Modus mit dokumentierten Allowlist-Warnungen, schärfer als ein reiner Warnmodus |
| `results/`-Verzeichnis (~25 MB Altlast) | — | 🔴 im Repo | 🟢 nicht mehr auffindbar |

**Einordnung:** Von den in Fassung 2.1 als offen markierten Punkten sind die überwiegende Mehrheit der
strukturellen Refactorings (Strangler-Shell, Deprecation, Crate-Zerlegung) und mehrere Ring-0/Ring-1-nahe
Features (Hyperkanten-Kernstruktur, KV-Cache Stufe B/C) inzwischen umgesetzt — das bestätigt, dass der in §A.2
verlangte Ground-Truth-Zyklus tatsächlich funktioniert, wenn er durchlaufen wird.

### A3.2 Was weiterhin offen ist (verifiziert, unverändert seit Fassung 2.1)

| # | Punkt | Referenz | Status |
|---|---|---|---|
| 1 | **Namensentscheidung final vollzogen im Code** | Kopf, §2 | Diese Fassung 3 löst diesen Punkt im **Dokument** auf (Contextra). Der mechanische Rename im **Repository selbst** (Cargo.toml-Namen, Crate-Verzeichnisse, README, CI-Workflows, PyPI-Namensprüfung im Publish-Gate) ist als eigener P0-Arbeitsschritt separat durchzuführen — siehe §A3.3, Punkt 1 |
| 2 | **P26-Verstoß:** `tokio` als reguläre Dependency in `contextra-vector`, `contextra-text`, `contextra-graph` (Ring 0) | §3 (P26), §4 | 🔴 unverändert Verstoß — weder entkoppelt noch die Spezifikation revidiert |
| 3 | **`wal_backpressure.rs`** (AK-13) | §15 | 🔴 einziger der ursprünglich vier kritischen Pflichttests, der weiterhin nicht auffindbar ist |
| 4 | **Benchmark-Belastbarkeit** (§17, Optimierungs-Roadmap) | §17, Anhang B §B.7 | 🟡 verbessert gegenüber Fassung 2.1 (Messgrenzen jetzt selbst dokumentiert, u. a. „keine verifizierten Cross-System-Vergleiche", einzelne Zahlen selbst als „nicht reproduziert" gekennzeichnet), aber weiterhin **kein** methodisch belastbarer Lauf mit realistischer Stichprobe gegen einen Standard wie BEIR/LongMemEval |
| 5 | **God-Files** (§3, Wartbarkeitsprinzip) | §3 | `contextra-vector/src/hnsw/mod.rs` (3.382 Zeilen), `diskann.rs` (3.369), `contextra-engine/src/collection/tests.rs` (3.718), `crud.rs` (1.656), `search.rs` (1.346), `lib.rs` (1.310) — mehrere Dateien deutlich über der 1.000-Zeilen-Zielmarke |
| 6 | **`xtask`-Umfang** | §A.3, §15 | 16.564 LOC — deutlich über dem in Vorberichten genannten Zielwert „< 3.000 LOC" |
| 7 | **Phantom-Commit-Schutz** | neu, siehe Beratung Abschnitt 2.1 | 🔴 kein CI-Gate gefunden, das Commit-Message-Umfang gegen `git diff --stat` prüft — trotz eines dokumentierten, belegten Falls eines leeren Commits mit erfundener fünf Punkte umfassender Message |
| 8 | Sechs Product-Owner-Entscheidungen (§0.4 oben: Ring-Konsolidierung, Nonce-Strategie, Clique-Konvention u. a.) | §0.4 | ⚖️ unverändert offen, siehe Liste oben in Teil A2 |

### A3.3 Priorisierte Restarbeit — konsolidiert aus Teil A3.2 und externer Beratung

Diese Reihenfolge ersetzt für den aktuellen Stand die frühere P0-Liste in §17/§18 dort, wo sie sich
überschneiden; §17/§18 bleiben für alle übrigen (Performance-/Architektur-)Punkte gültig.

**P0 — vor jeder weiteren Feature-Arbeit (Stunden bis 1 Tag):**
1. Namens-Rename mechanisch im gesamten Repository durchziehen (Cargo.toml-Paketnamen `contextra*`, Crate-Verzeichnisse, README, GitHub-Org/Repo-Name, PyPI/crates.io/npm-Reservierung, Namensprüfung im `publish-pypi.yml`-Gate anpassen) — Skript, keine Handarbeit; danach keine weitere Revision.
2. Phantom-Commit-Gate einbauen (CI-Check `git diff --stat` gegen Commit-Message-Länge/Schlüsselwörter; < 1 Stunde Aufwand laut Beratung).

**P1 — vor öffentlichem v0.1-Launch (1–3 Wochen):**
3. Governance-Redundanz abbauen (Status-Dateien von Git-Historie entkoppeln, `.jules/claims.json`-Claim-Infrastruktur für Solo-Betrieb zurückbauen, doppelte Crate-Inventartabelle konsolidieren).
4. `wal_backpressure.rs` nachziehen (letzter fehlender Pflichttest).
5. Einen methodisch sauberen, kleinen Benchmark fahren (realistische Stichprobe, mindestens ein etablierter Referenzpunkt wie BEIR/LongMemEval-Subset), Ergebnis so veröffentlichen wie es ausfällt.
6. README/Installationsanweisungen final an „Contextra" anpassen, inklusive funktionierendem Installationspfad für den MCP-Server.

**P2 — nach erstem echten Nutzerfeedback, nicht vorher:**
7. `.unwrap()`-Reduktion (~1.047 Aufrufe außerhalb Tests) gegen die Zero-Panic-Doktrin — entweder konsequent fortsetzen oder Zielwert in §20.3/ADRs ehrlich revidieren.
8. P26-Verstoß entscheiden: echte `tokio`-Entkopplung in Ring 0 (Wochenaufwand) oder Prinzip formal revidieren (Stunde) — die billige Option zuerst ziehen, damit Spezifikation und Code nicht länger widersprüchlich sind.
9. God-Files zerlegen (§A3.2 Nr. 5) — Hygiene, nicht blockierend.
10. Die sechs Product-Owner-Entscheidungen aus §0.4 (DocId-Migration, 2PC-Härtung vs. WAL-als-Wahrheit, Ring-Konsolidierung, Nonce-Strategie, Clique-Konvention) bewusst zurückstellen, bis reale Lastmuster vorliegen — jede vorzeitige Festlegung ist Spekulation ohne Messgrundlage.

**Bewusst nicht tun (Scope-Einfrierung, siehe Beratung Abschnitt 4.3 und 9):**
- Keine neuen Crates/Subsysteme vor v0.1.
- Kein Enterprise-Ausbau (Multi-Tenancy-Admin, SLA, Compliance-Zertifizierung) vor einem konkreten Interessenten — die Architektur (Ring-Modell, MVCC, Verschlüsselung, Audit-Trail) trägt ein Enterprise-Produkt bereits strukturell, siehe §10.
- Kein Managed-Cloud-Angebot vor belegtem Self-Hosting-Interesse — widerspricht sonst dem Kern-Differenzierungsmerkmal Air-Gapped-Betrieb (§2).
- Keine Benchmark-Zahlen veröffentlichen, die methodisch nicht standhalten (§A3.2 Nr. 4).
- Keine weitere Tiefenanalyse-Runde vor Abarbeitung der P0-Liste — dieses Dokument ist ab jetzt die einzige fortzuschreibende Quelle der Wahrheit, nicht ein weiterer Bericht daneben.

### A3.4 Positionierung (aktualisiert, ersetzt Teilaussagen aus §2)

Für §2 (Produktvision) gilt ergänzend die aus der externen Beratung übernommene, geschärfte Positionierung:
*„Die eingebettete, air-gapped-fähige Gedächtnisschicht für Rust- und Lokal-KI-Entwickler, die Datenhoheit
brauchen — nicht noch eine Cloud-Memory-API."* Nicht als „Cognitive OS" oder generisches Agenten-Framework
positionieren (das ist Letta/MemGPT-Territorium); nicht auf Benchmark-Leaderboards gegen Hindsight/Zep
antreten, solange kein methodisch sauberer Wert vorliegt (§A3.2 Nr. 4); nicht gegen die Netzwerkeffekt-Breite
von Mem0 antreten. Strukturell differenzierbar und einzigartig im Vergleich zu Mem0, Zep/Graphiti, Letta,
Cognee, LangMem, Hindsight sowie zu embedded Rust-Bausteinen wie LanceDB/Qdrant embedded/tantivy: die
Kombination aus (a) echtem Embedded/Air-Gapped-Betrieb ohne externe Datenbank-Abhängigkeit, (b)
architektonisch verankerter DLP/Egress-Kontrolle (`egress_guard.rs`, `prompt_injection.rs`, fail-closed) statt
nachgerüsteter Compliance, (c) Rust-nativer Performance ohne Python-GC-Pausen, und (d) einer bereits
vorhandenen, mit Graphiti/Zep konzeptionell konkurrenzfähigen generativen Synthese-Pipeline mit
Grounding-Validierung (§6, Konsolidierung) — die bislang unveröffentlicht und unbenannt ist.

---

<a id="a4-review"></a>
