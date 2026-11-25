# ADR-115: Vereinfachte synchrone Löschung ohne kryptografischen Löschnachweis

* **Datum**: 2026-11-25
* **Status**: Accepted (Auftraggeber-Freigabe 2026-11-25). Umsetzung ausstehend.
* **Ersetzt**: ADR-112 (Option C wird verworfen, Option B umgesetzt). Die Invarianten aus Abschnitt 5 werden per Umsetzung entfernt.
* **Betroffene Komponenten**: `contextra-crypto`, `contextra-engine`, `contextra-db`, `contextra-graph`, `contextra-vector`, `contextra-kvcache`, `contextra-store`, `contextra-mcp`, `contextra-audit-export`, `contextra-avv-generator`
* **Spezifikation**: `docs/spec/loeschung-vereinfacht.md`

---

## 1. Kontext

Contextra ist ein Speicher- und Retrieval-System für lokale Large-Language-Modelle im Mittelstand. Die Hauptfunktion ist die hybride Suche (Volltext, Vektor, Graph) mit einer effizienten Speicher- und Vektorschicht. Die Löschung personenbezogener Daten ist eine Pflichtfunktion für den Kunden als Verantwortlichen (Art. 17 DSGVO). Contextra unterstützt den Kunden dabei als Auftragsverarbeiter (Art. 28 Abs. 3 lit. e DSGVO).

Der bisherige Stand war auf einen kryptografischen Löschnachweis ausgelegt: signierte `DeletionProof`-Objekte (v3), WAL-Delete-Receipts, Crypto-Shredding und ein Art.-30-Export der Beweise. Das ist aufwendig und schwer wartbar. Der Nachweis bezieht sich außerdem auf Zustände außerhalb des Einflussbereichs von Contextra (Backups des Kunden, Kopien in externen Modellen, Dateisystem-Reste). Er kann deshalb nicht halten, was er verspricht.

## 2. Entscheidung

1. **Löschung ist eine synchrone, transaktionale Operation** `delete_document(id)`. Sie entfernt Dokument, Vektoreinträge, Graph-Kanten und KV-Einträge in einer Transaktion. Die Operation gibt einen Status zurück (Erfolg oder Fehler mit Grund). Das ist eine Systembestätigung, kein kryptografischer Beweis.
2. **Kein kryptografischer Löschnachweis.** Es gibt keine signierten Löschbeweise, keine Löschbeweis-Schlüssel und keinen Beweis-Export.
3. **Kein Crypto-Shredding als Löschmechanismus.** Physische Bereinigung erfolgt über die bestehende Compaction. Eine Verschlüsselung at rest bleibt unabhängig davon bestehen, weil sie andere Zwecke erfüllt (Schutz der Daten auf dem Datenträger).
4. **Tombstones bleiben als interne Konsistenz-Mechanik** für Hybridsuche, HNSW-Index und MVCC-Snapshots. Sie sind kein Löschnachweis.
5. **Die asynchrone Kaskade wird durch eine synchrone Kaskade ersetzt.** Der Hyperedge-Worker für Löschungen entfällt, sofern die Kaskade in der Transaktion abgeschlossen werden kann. Ist das nicht möglich, bleibt er als Ausnahme mit dokumentierter Begründung.
6. **MCP-Tool `contextra_delete`** wird einheitlich dokumentiert und liefert nur den Status. Die bisherige Abweichung zwischen Doku („issue DeletionProof v3“) und Handler („keine Beweise“) entfällt.

## 3. Begründung

- Der Mittelstand braucht eine einfache, verlässliche Löschung. Der Nachweis ist für die Kundenpflicht nicht zwingend, die Pflicht selbst bleibt bestehen.
- Der Beweis-Pfad erzeugt Wartungslast in Kryptografie, WAL und Audit-Export, die in keinem Verhältnis zum Nutzen für die Hauptfunktion steht.
- Ein Nachweis über Zustände außerhalb des Einflussbereichs von Contextra wäre irreführend und wird deshalb nicht mehr angeboten.

## 4. Konsequenzen

**Positiv:** weniger Code in `contextra-crypto`, einfachere Transaktionen, klarere Schnittstelle, weniger Wartung.

**Negativ / Risiken:**
- Kunden können gegenüber Aufsichtsbehörden keinen kryptografischen Löschnachweis aus Contextra vorlegen. Die Kunden müssen ihre Löschprozesse dokumentieren. Contextra stellt dafür einen Status und ein Protokoll der Löschaufrufe bereit (ohne Beweischarakter).
- WAL-Einträge bleiben bis zur Kürzung nach Snapshot erhalten. Dafür ist ein Löschverhalten beim WAL-Replay festzulegen (siehe Spezifikation, Abschnitt 4).
- Rechtliche Bewertung der Löschpflicht im konkreten Kundenfall liegt beim Kunden. Dieser ADR ist keine Rechtsberatung.

## 5. Betroffene Invarianten (werden bei Umsetzung ersetzt)

Die Invarianten bleiben bis zur Umsetzung in `AGENTS.md` bestehen, sind dort aber als „superseded by ADR-115“ markiert. Sie werden entfernt, sobald die zugehörigen Tests entfernt sind. Gates werden nicht abgeschwächt.

| ID | Entscheidung |
|---|---|
| INV-DELETION-1 | entfällt |
| INV-DELETION-2 | entfällt |
| INV-DURABILITY-RING | entfällt (betrifft Beweis) |
| INV-PERF-PROFILE-1 | entfällt (betrifft Beweis) |
| INV-COLLECTION-PROFILE-1/2 | entfällt (betrifft Beweis und CryptoShred) |
| INV-VAULT-1/2/3 | bleibt (Verschlüsselung at rest) |
| INV-WAL-LEGACY-KEY-1, INV-WAL-TRUNCATION-1 | bleibt |
| INV-MCP-CLASSIFY-1 | bleibt, `contextra_delete` bleibt klassifiziert |
| Fail-Closed | bleibt |

## 6. Verworfene Alternativen

- **Option A: Status quo (Beweis und Shredding vollständig).** Verworfen wegen Wartungslast und fehlendem Nutzen für die Hauptfunktion.
- **Option C: Beibehaltung als Krypto-Scaffolding (ADR-112, Empfehlung dort).** Verworfen, weil Scaffolding ohne Aufrufer dauerhaft Last erzeugt.
- **Vollständige Entfernung jeder Löschfunktion.** Verworfen, weil der Kunde als Verantwortlicher ohne Löschfunktion die Pflichten nicht erfüllen kann.

## 7. Umsetzung

Siehe `docs/spec/loeschung-vereinfacht.md`. Die Umsetzung erfolgt in Phasen, jede Phase mit grünen Gates und eigenem Commit.
