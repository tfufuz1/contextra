Module: orchestration, transaction, collection, backends und adapters, background_workers (Decay, Expiry, Orphan), decay_controller (adaptiv), chunker, extraction, fusion, filter, temporal_filter, erasure, export, import, metrics_names, compute pool.

Harte Regeln:
- Fail-closed bei jedem Fehler. Kein stiller Teilerfolg bei Mehrindex-Schreibvorgängen. Entweder alle Indizes committen oder der Vorgang schlägt sauber fehl und wird kompensiert.
- Cancellation-Safety: Jeder `.await`-Punkt in einem Schreibpfad muss so gestaltet sein, dass ein Abbruch keine halb geschriebenen Zustände hinterlässt.
- `await_holding_lock` und `await_holding_refcell_ref` sind deny. Sperren vor `.await` freigeben.
- Hintergrund-Worker sind abbrechbar und zählen Fehler als Metriken, sie verschlucken keine.
- Feature-Flags (`reranking`, `background-maintenance`, `entity-extraction`, `edge-reinforcement-learning`, `auto-extraction-opt-out`) müssen je einzeln und kombiniert testbar sein.

Verifikation: `cargo test -p contextra-engine --locked`, `cargo test -p contextra-engine --all-features`, Integrationstests über `contextra` und `contextra-db`.

Optimierungsfokus: Kosten je Schreibpfad über alle Indizes, Batchbildung für Hintergrund-Maintenance, Compute-Pool-Auslastung unter Last.
