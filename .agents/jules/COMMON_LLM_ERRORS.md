# Häufige Fehler bei der Arbeit am Code und ihre Korrektur

Nicht normativ. Jede Zeile verweist auf die SPEC-Stelle, die maßgeblich ist. Bei Unsicherheit zuerst hier nachsehen, dann die SPEC.

| Fehler | Richtig (SPEC) |
|---|---|
| `std::fs` oder `tokio::fs` im Store, Vektor oder Löschkern | Nur der Port `Vfs` (D-04) |
| `unwrap()`, `expect()`, `panic!`, `let _ =` auf `Result` im Produktionscode | Typisierter Fehler, Propagation (P6) |
| Feature-Schalter für Verschlüsselung, WAL-Integrität oder Legacy-Schlüssel | Kein Schalter; Verschlüsselung ist immer aktiv (P5, D-05) |
| Passphrase mit HKDF abgeleitet | Argon2id innerhalb der Grenzen aus D-06; HKDF nur für Subschlüssel |
| `TenantPolicy`, `Option`-Mandant oder Funktion ohne `TenantContext` | `TenantContext` ist Pflicht (D-07) |
| `DocId(u64)` als Dokumentidentität | `DocRef` mit Betroffenem; Suchtreffer als `DocHandle` (D-08, D-70) |
| `Collection::delete` oder Löschaussage bei `soft_delete` | `soft_delete` ist ein Tombstone; Löschpflicht über `erase` (D-51) |
| Vektor durch Nullvektor ersetzen | Surrogat, Batch-Kantenreparatur, deterministischer Rebuild (D-77) |
| `u64::MAX` als Wert für „alles sichtbar“ | `Option` oder `current_seq` (8.7, I-12) |
| „Ebene B“ oder ADR-001 als Grundlage zitieren | ADR-001 ist ersetzt; Schlüsselvernichtung gehört zu v1 (D-01) |
| `mmap` oder `memmap2` für Lesezugriffe | Positionelles Lesen über `Vfs` (D-94) |
| WAL-Frames aus einem Segment herausschneiden | Ganze Segmente bis zum Flush-Punkt entfernen; `WalFloor` (D-73) |
| Schreibzugriff ohne Prüfung des ReadGate | Prüfung in der Commit-Phase (D-57) |
| Löschbericht ohne Positivkontrolle | `LayerWitness` nur aus `Scanner<Armed>` (D-72) |
| Löschaussagen außerhalb der Außenaussage | Nur die Außenaussage aus SPEC 6.4 (HR-5) |
| Neue Abhängigkeit ohne Matrixprüfung oder mit Netzwerk-Crate | Matrix SPEC 5.3; kein Netzwerk (HR-1) |
| `Cargo.toml` geändert, `Cargo.lock` nicht | Im selben Commit erneuern; mit `--locked` prüfen (D-93) |
| Agentenmarken, Zeitstempel oder absolute Pfade im Code | Verboten (D-91) |
| Module `misc`, `util`, `*_impl`, `mod.rs` | Namensregeln (D-90) |
| Erfundene Pfade oder Befehle (z. B. `tests/layering.rs`, `just fuzz-all`, `.jules/`) | Vor der Verwendung prüfen. Im Repository existieren sie nicht; die Ablage ist `.agents/jules/` |
| Erfundene Workflows (z. B. `mutation-testing.yml`, `post-merge-verification.yml`) | Vorhanden sind die Dateien in `.github/workflows/`; siehe Verzeichnisliste |
