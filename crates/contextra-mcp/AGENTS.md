Der Server verarbeitet untrusted stdio-Input direkt. Module: protocol, server, server_dispatch, server_tools, tools_crud, tools_erase, validation, prompt_injection (guard, policy, audit), sandbox, config, io, explain.

Harte Regeln:
- Kein HTTP-Transport und kein Axum (ADR-010). Eine Abweichung ist eine Architekturverletzung.
- Fail-closed bei Prompt-Injection-Verdacht und bei Cloud-Egress. Im Zweifel blockieren, nicht durchlassen.
- `MAX_RPC_BYTES` begrenzt jede Nachricht. Parser dürfen nie unbegrenzt puffern.
- Jede Tool-Änderung aktualisiert die Klassifikation in der Tool-Registry und die Tabelle in `lib.rs`. Tests `tool_registry_classification` und `confirm_required_semantics` sind Pflicht.
- Löschwerkzeuge (`contextra_delete`, `contextra_forget`) verlangen die dokumentierten Bestätigungen.
- Der Python-Launcher lädt eine Binärdatei nach (`_download.py`). Herkunft, Versionsbindung und Prüfsumme müssen verifiziert werden, bevor Änderungen daran gemergt werden.

Verifikation: `cargo test -p contextra-mcp --locked`, Fuzz-Targets `fuzz_jsonrpc_parsing` und `fuzz_prompt_injection_guard`, `cd python && python -m pytest tests/`.

Optimierungsfokus: Parser ohne Zwischenallokation, Guard-Regeln mit geringer Latenz, klare Fehlercodes ohne Informationslecks.
