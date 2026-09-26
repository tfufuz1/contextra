---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "02"
---
## 2. Produktvision, Alleinstellungsmerkmale und Nicht-Ziele

### 2.1 Was Contextra ist

Eine eingebettete (embedded) Gedächtnisschicht, kein Cloud-Service. Contextra läuft im Prozess des
aufrufenden Agenten oder als lokaler MCP-Server — es gibt keine serverseitige Multi-Tenant-Instanz
und keine Datenübertragung an Dritte, sofern nicht explizit über das Cloud-Egress-Gateway (§10.4)
angefordert.

### 2.2 Distributionswege

| Kanal | Paket | Zielgruppe |
|---|---|---|
| MCP-Server (primär) | `uvx contextra-mcp --db-path ... --allow-write` | Claude Desktop, Cursor, beliebige MCP-Clients |
| Python-Bibliothek | `pip install contextra` | In-Process-Einbettung in Python-Agenten |
| Rust-Crate | `cargo add contextra-db` | Native Rust-Anwendungen |

Eine Desktop-Shell (`contextra-tauri`) existierte als Prototyp, ist aber zugunsten der PyPI-Bibliothek <!-- crate-ref-ignore -->
und des MCP-Servers als primäre Vertriebswege eingestellt (deprecated, ADR-077).

### 2.3 Alleinstellungsmerkmale und ihr Reifegrad

1. **4-Signal-Hybridsuche** (🟢) — Vektorsuche (HNSW), Volltextsuche (BM25/BM25F), Wissensgraph-Traversierung
   (CSR + Forward-Push-Personalized-PageRank), Metadaten-Filter; fusioniert über Reciprocal Rank Fusion
   (RRF, Default 🟢) oder score-normalisierte Fusion (Opt-in 🟡, mit hartem RRF-Fallback bei Signaldegradation).

2. **Kalibriertes Retrieval mit Lyapunov-Drift-Erkennung** (🟢) — Score-Schwellenwerte werden nicht statisch,
   sondern über ein laufend kalibriertes Modell mit gedeckelter Drift-Eskalation bestimmt.

3. **MCP-native Zero-Trust-Sandbox** (🟢) — Tool-Ausführung mit getrennt konfigurierbarem Fuel- (Rechenschritt-)
   und Wall-Clock-Budget (Default 5 s), orthogonal zueinander konfigurierbar.

4. **Kryptographische DSGVO-Art.-17-Löschung** (🟢) — Löschvorgänge erzeugen einen verifizierbaren `DeletionProof`
   über eine race-freie HMAC-Kette.

5. **Session-DAG** (🟢) — Konversationsverzweigung als persistenter, azyklischer Graph.

6. **Air-Gap-KV-Cache-Bridge mit LSM-Fallback-Spill** (🟢) — der KV-Cache liegt primär verschlüsselt im RAM; bei
   Speicherdruck greift kontrolliertes Auslagern auf die SSD statt verlustbehafteten Verwerfens.

7. **Cloud-Egress Privacy Gateway** (🟢, weitgehend auditiert) — mehrschichtiger DLP-Pfad mit Surrogat-Tokenisierung,
   Bulk-Exfiltration-Erkennung und Rehydration der Cloud-Antwort (§10.4).

8. **Contextual-Bandit-Routing** (🟢 Grundfunktion / 🟡 mathematisch korrekte Variante) — LinUCB-basiertes Routing;
   siehe §8 für die Unterscheidung zwischen Produktions-Default und Ridge-korrekter Opt-in-Variante.

9. **Gestufte Vektorindex-Architektur** (🟢 HNSW / 🟢 DiskANN als Tier) — HNSW als Standard, DiskANN für
   RAM-sprengende Korpora.

10. **Deutsche Morphologie inkl. BM25F** (🟢) — Kompositazerlegung im Volltextindex plus feldgewichtete Bewertung.

11. **Zero-Copy-Storage-Pfad** (🟢) — seit der Grundarchitektur produktiv.

12. **Key-granulare Schreibnebenläufigkeit** (🟢) — `kv_locks` statt collection-weitem Mutex.

13. **N-äre Hyperkanten** (🔴, vollständig spezifiziert, siehe §6) — Fakten mit mehr als zwei Beteiligten als
    erstklassige, atomar invalidierbare Struktur statt Zerlegung in mehrere, im Zusammenhang verlorene Binärkanten.

### 2.4 Nicht-Ziele

Contextra ist explizit **kein** Cloud-SaaS-Produkt, **kein** Multi-Tenant-Enterprise-System, **kein** Framework für
LLM-Training, **keine** primär GUI-getriebene Desktop-Anwendung und **kein** Cluster-/Replikations-System. Ein
`contextra-cluster`-Veto besteht bewusst: verteilter Konsensbetrieb ist kein Ziel der aktuellen Produktphase. <!-- crate-ref-ignore -->
Passives WAL-Shipping für Backup-Zwecke ist als Fernziel vorgesehen (Roadmap-Stufe 4, §18), aber nicht Bestandteil
des Kernprodukts.

---

<a id="3-prinzipien"></a>
