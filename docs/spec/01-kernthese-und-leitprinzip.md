---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "01"
---
## 1. Kernthese und Leitprinzip

**Contextra ist eine souveräne, vollständig lokal betriebene Gedächtnisschicht für KI-Agenten** — eine
eingebettete, kryptographisch isolierte AI-Memory-Bibliothek in Rust mit Python- und MCP-Bindings, die
ohne Cloud-Abhängigkeit, ohne Telemetrie und ohne API-Key betrieben werden kann.

Ihr Alleinstellungsmerkmal ist die Kombination aus:

- einer **4-Signal-Retrieval-Fusion** (Vektor, Volltext, Graph, Metadaten) statt reiner Vektorsuche,
- einer **kryptographisch integritätsgesicherten Storage-Engine** (LSM-Tree, WAL mit HMAC-Kette, AES-256-GCM-SIV at rest),
- **WASM-/Sandbox-Ausführungsisolation** für Agent-Tool-Aufrufe,
- echter **Air-Gap-Inferenz** (lokales GGUF-Backend, kein Netzwerkzwang) mit verschlüsseltem, LSM-rückfallfähigem KV-Cache,
- einem **Contextual-Bandit-Router**, der Anfragen adaptiv auf Retrieval-Strategien verteilt,
- und — als jüngste, noch zu bauende Erweiterung des Datenmodells — **n-ären Hyperkanten** für Fakten, die sich
  nicht auf ein Subjekt-Prädikat-Objekt-Tripel reduzieren lassen (§6).

### Leitprinzip

**Korrektheit schlägt Performance schlägt Feature.**

Jede Optimierung, die eine Korrektheitsgarantie (Datenintegrität, Nebenläufigkeitssicherheit,
Wiederherstellbarkeit, Deadlockfreiheit) aufweicht, ist unzulässig — unabhängig vom Performancegewinn.
Jede Performance-Optimierung, die eine noch nicht spezifizierte Fähigkeit vorwegnimmt, ist nachrangig
gegenüber der Fertigstellung bereits spezifizierter Fähigkeiten. Jede Erweiterung eines bestehenden
Subsystems muss geprüft werden gegen die Invarianten, die dieses Subsystem bereits trägt — nicht nur
dagegen, *dass* eine Erweiterung grundsätzlich möglich ist, sondern *welches bestehende Invariant dadurch
unter Druck gerät* und wie es gewahrt bleibt.

---

<a id="2-vision"></a>
