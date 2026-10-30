ARBEITSREGELN (gelten für diesen Auftrag, gehen vor PREAMBLE.md und AGENTS.md §2a)

1. `cargo xtask` existiert nicht mehr. Nutze nur: cargo fmt, cargo check, cargo clippy,
   cargo test, cargo deny. Toolchain ist 1.89.0 (rust-toolchain.toml). Prüfe `rustc --version`
   und nenne die Version im PR. Weicht sie ab, melde das und rate nicht.
2. Das Repo ist reduziert: nur 14 Crates unter crates/ existieren. Lege KEINE entfernten Crates
   neu an (graph, cognition, adapt, kvcache, router, agent, infer-*, sandbox, py, license, core,
   ingest, mcp, wire, checkpoint, privacy, audit-export, avv-generator). Wenn Code sie braucht,
   entferne die Verwendung, ersetze sie nicht.
3. Verifiziere jedes Symbol vor der Nutzung: `grep -rn "pub fn NAME\|struct NAME\|enum NAME" crates/`.
   Erfinde keine APIs. Lege keinen Typ an, der schon existiert.
4. Ändere NUR Dateien im SCOPE. Alles andere: unter „Out-of-scope Findings" im PR melden.
5. Ändere niemals Lints, `#![allow(...)]`, clippy.toml, Baselines, deny.toml, rust-toolchain.toml,
   Cargo.toml-Lint-Tabellen, .github/** oder governance/**, um Rot zu beseitigen. Behebe die Ursache.
   Kein neues `#[allow]`, kein `unwrap`/`expect`/`panic!`/`todo!` in Produktionscode.
6. Test zuerst (rot), dann Fix (grün), wo ein Fehlverhalten behoben wird.
7. Lösche keinen Test ohne ihn im PR-Text aufzulisten (Datei, Name, Grund). Tests für entfernte
   Features dürfen entfallen; Tests für behaltene Features werden angepasst, nicht gelöscht.
8. Ein Auftrag, ein PR. Schlägt dieselbe Abnahme nach 3 Versuchen weiter fehl: STOPP, kein
   zweiter PR, kein Umweg über abgeschwächte Prüfungen. Berichte den rohen Fehler.
9. „Fertig" heißt: alle ABNAHME-Befehle sind grün. Hänge ihre letzten 15 Ausgabezeilen an den PR.
   Behaupte nichts, was du nicht ausgeführt hast. Nenne ausdrücklich, was du NICHT prüfen konntest
   (zum Beispiel fehlende CPU-Features).
10. PR-Text: Ziel, geänderte Dateien, Abnahme-Ausgabe, gelöschte Tests, Out-of-scope Findings,
    Unsicherheiten. Vor dem PR: `git fetch origin && git rebase origin/main`, Abnahme wiederholen.
