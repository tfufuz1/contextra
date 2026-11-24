# SP-CUTOVER — Migration und Ablösung des Altstands

| | |
|---|---|
| Modul | `tools/migrate/`, separate Migrationstools und Betriebsdokumentation |
| Besitzt | Import/Export, Resume/Checkpoint des Migrators, Differentialvergleich, Umstellungs-/Rollbackplan |
| Konsumiert | öffentliche Storage-Fassade, Altstand nur als Input/Adapter |
| Spec-Bezug | R8 §3, §12, §12.8, §14; E-29–E-32, E-43, E-55; Audit A-10–A-12 |
| Phase | P8+ nach G-P |

## 1 Auftrag
`contextra-storage` ersetzt `contextra-store` als dauerhaften Persistenzkern des Gesamtsystems. Es ist die gemeinsame Durable-Record-Schicht für DB-/Agentenzustand und clusterlokale Records für Dokumente, quantisierte Vektoren, BM25-Postings und Graph-Deltas. Globale ANN-Strukturen bleiben ableitbare Extensions mit Watermark/Erase-Vertrag. Der Kern übernimmt keinen Altstandcode. Kein Cutover, bis G-P, Formatfreeze, Recovery-Soak und Differentialprüfung bestanden sind. Migration produziert Bericht ohne Nutzdaten und kann anhand stabiler Import-IDs fortgesetzt werden.

Der Altstand bleibt während Implementierung und stufenweiser Migration unverändert im Workspace, aber sein Status ist **Legacy**. Er ist keine Build-/Laufzeitabhängigkeit des neuen Kerns; nur ein isolierter Migrationsadapter darf alte Bestände lesen. Verhaltensparität wird aus dokumentierten öffentlichen Verträgen und Charakterisierungsergebnissen abgeleitet; keine LSM-, WAL-, SSTable- oder Verschlüsselungscodepfade werden kopiert.

## 2 Ablösegrenze und Verbraucher

`contextra-storage` bleibt synchron, embedded und unabhängig von Tokio, `contextra-store`, `contextra-ports` und Fachschichten. Async-Aufrufer gehen ausschließlich über `contextra-storage-bridge`; Persistenzverträge oberer Crates werden gegen die neue Fassade bzw. einen eigenen fachlichen Port entkoppelt.

Betroffene Produktionsverbraucher sind anhand der Workspace-Abhängigkeiten zu migrieren, insbesondere `contextra-db`, `contextra-agent`, `contextra-engine`, `contextra-graph`, `contextra-cognition`, `contextra-checkpoint`, die Produktfassade `contextra` und `contextra-py`. Das ist eine verifizierbare Inventarliste aus den derzeitigen Cargo-Abhängigkeiten, kein abschließender Ausschluss weiterer Verbraucher. `contextra-vector` und weitere Fachcrates müssen gegen die neue Record-/Extension-API integriert werden, statt den LSM-Kern direkt zu betreiben. Tests und Beispiele sind Teil der Ablöse: kein Default-Member oder Produktionsfeature darf heimlich `contextra-store` nachziehen.

Es gibt keine dauerhafte `LsmStorage`-Kompatibilitätsfassade in `contextra-storage`. Semantische Unterschiede werden im Bridge-/Cutover-Mapping sichtbar gemacht; nicht abbildbare Operationen liefern `Unsupported` mit Migrationshinweis. Während einer Migration gibt es pro Datenbestand genau einen schreibenden Eigentümer. Dual-Write ist verboten.

## 3 Regeln
Bulk-Import nutzt `Origin::Migration`, `Relaxed` nur mit anschließendem `durable_barrier`; IdemToken macht Wiederholung sicher. Größere Nutzlasten werden auf oberer Ebene gechunked. Altstand `rollback_to_tx` wird Branch abgebildet oder als Unsupported gemeldet. Legacy Space ist Plain und darf keine personenbezogenen Daten enthalten.

## 4 Ablösefolge und Abnahme

1. **Contract inventory:** jede produktive `contextra-store`-Nutzung und jeder persistierte Namespace erhält einen Eigentümer und Ziel-Storage-Modell.
2. **Fachliche Entkopplung:** DB, Agent, Engine, Graph, Cognition, Checkpoint, Produktfassade und Python-API hängen nicht mehr an `LsmStorage`-konkreten Typen.
3. **Adapter- und Importpfad:** async-/Legacy-Aufrufe werden über Bridge oder Migrationstool abgebildet; Roundtrip und semantische Differenzen sind versioniert.
4. **Stufenmigration:** je Store/Namespace exportieren, importieren, dauerhaft barrieren, Stichproben lesen und Cutover-Eigentümer umschalten. Ein persistenter Marker verhindert parallele Writer.
5. **Entfernung:** `contextra-store` aus Produktions- und Dev-Dependencies, Workspace-Default-Members und CI-Gates entfernen, sobald alle Verbraucher migriert sind. Historischer Quellstand wird read-only unter `legacy-store-final` archiviert.
6. **Systemabnahme (G-CUTOVER):** gesamter Workspace baut ohne `contextra-store` als Dependency; End-to-End-Recovery einschließlich externem Fencing des alten Instanzbesitzers, Crash-/DiskFull-Szenarien, Extension-Watermarks, vollständige Erase-Atteste/DeletionProof-v3-Bridge, gemeinsames RSS-/Disk-/CPU-Budget, RAG-Differentialbench und 72-h-Soak bestehen. Ein Zwei-Host-Rehearsal beweist, dass nie beide Kopien derselben `InstanceId` schreibend öffnen. Jede der aktuellen sieben DeletionProof-Witness-Klassen ist einer konkreten R8-/Extension-Quelle zugeordnet oder der Proof bleibt `Incomplete`.

Rollback bedeutet Rückkehr des **Betriebseigentümers** vor dem Cutover oder Restore aus geprüftem Backup. Nach neuen Writes in `contextra-storage` werden diese nicht rückwärts in den Altstand gespiegelt; Rollback verlangt einen dokumentierten Export aus dem neuen Format oder Wiederherstellung eines konsistenten Vor-Cutover-Backups.

## 5 Rollback- und Berichtsregeln

Der Bericht enthält Vollständigkeitszählung, Schlüssel-/Clusterzuordnung, Sample-Readback und semantischen Differentialvergleich. Vor Umschaltung bleibt der Altstand authoritative; nach Umschaltung wird nur über dokumentierten Rückweg zurückgerollt, nie durch paralleles Schreiben in zwei Kerne. Geheimnisse und Klartext gehören nicht in Logs/Artefakte.

## 6 Arbeitspakete

WP-24.1 Verbraucher-/Namespace-Inventar (P8) · WP-24.2 API-Entkopplung und Bridge-Mapping (P8) · WP-24.3 resumierbarer Export/Import (P8) · WP-24.4 Differentialaudit + Rehearsal (P8) · WP-24.5 stufenweiser Single-Writer-Cutover (P8+) · WP-24.6 Dependency-/Default-Member-Bereinigung und Legacy-Archiv (P8+) · WP-24.7 Gesamtsystem-Erase-/DeletionProof-/RAG-Abnahme (P8+). Abnahme: reproduzierbarer Bericht, keine produktive `contextra-store`-Dependency, erfolgreicher Restore-/Recovery-Soak; G-CUTOVER nur mit vollständigem Proof-Mapping und Ressourcenbudget.

## R8.2-Nachtrag

**Spec-Bezug ergänzt:** R8.2 §3.5, §9.5.2, §10.4, §11.4; E-75, E-76, E-81.

Die übrigen Crates (`contextra-db`, `-agent`, `-engine`, `-graph` u. a.) bauen bisher auf `contextra-store`. Für die Umstellung sind die R8.2-Schnittstellen in die Caller-Matrix (WP-24.1) aufzunehmen:

- Neue Fehlerpfade, die Verbraucher behandeln müssen: `SnapshotTooOld` (lange Scans über `ResumeToken` neu aufsetzen statt Views offenzuhalten), `ErasePending` (Löschabsicht aufgenommen, Neustart nötig), `BackupBeyondHorizon` (Restore-Runbooks).
- Backup-Werkzeuge müssen `Storage::backup_mark()` bei jeder Kopie speichern und `backup_horizon` einhalten; sonst ist Retire nicht aktivierbar.
- `OpenProfile::Production` verlangt `Hardware`/`Kms`-Provider für `Personal`-Daten und ein vom Datenverzeichnis getrenntes Ledger-Gerät; Betriebsdokumentation und Rehearsals (WP-24.4) prüfen beides.
- Derivation-Caches (Embeddings/Token) der oberen Schicht folgen der Konvention aus SPEC §12.6 (clusterlokales Zufallssecret); der Altbestand darf nicht über Cluster hinweg dedupliziert importiert werden.

| ID | Titel | PW | Phase | Abnahme |
|---|---|---|---|---|
| WP-24.8 | R8.2-Caller-Matrix: Fehlerpfade, Backup-Marken, Production-Anforderungen, Cache-Konvention | 1.0 | WP-24.1, WP-23.5 | P8 | Matrix ohne ungeklärte Semantik, Runbook-Review |

PW = Personenwochen, grobe Schätzung ohne Zusage (wie SPEC §14).

## R9.1-Synchronisierung

Die Caller-Matrix umfasst die R9-Fehler und Rückdruckpfade (`GuardRetry`, `ClusterMetaBudget`, `ClusterRetired`, `EraseFanoutExceeded`, FD-/Scratch-Admission) und die branch-bewusste Lesesemantik. Cutover-Evidenz trennt bestätigte Messungen von H-24…H-28-Designzielen; kein Bericht darf diese Hypothesen als Produktionswerte ausgeben.
