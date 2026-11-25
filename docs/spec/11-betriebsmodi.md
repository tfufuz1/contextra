---
source: CONTEXTRA_SPEC_v4_MASTER.md
chapter: "11"
---
## 11. Betriebsmodi

Contextra wird ausschließlich eingebettet betrieben: im Prozess des aufrufenden Agenten (Rust- oder Python-Bindung)
oder als lokaler MCP-Server über stdio-JSON-RPC. Es gibt keinen Server-Modus mit Netzwerk-Listener für
Multi-Tenant-Zugriff. Der Cloud-Egress-Pfad (§10.4) ist der einzige Punkt, an dem Daten das lokale System
verlassen — ausschließlich auf explizite Anforderung, nie als Hintergrundtelemetrie.

---

<a id="12-schema"></a>
