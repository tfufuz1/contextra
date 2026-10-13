import re

with open('.jules/contextra-Prompter.html', 'r', encoding='utf-8') as f:
    h = f.read()

# === Task 1 ===
old_mcp = "  { id:'contextra-mcp', layer:9, order:31, desc:'Model Context Protocol Server & Cloud Egress Gateway', loc:'7073', tests:'112+', status:'❓', risk:'proto', tier:'1' }"
new_mcp_and_others = """  { id:'contextra-durable-fs', layer:0, order:7, desc:'Durable Filesystem Operations und synchrone Disk-Flushes', loc:'790', tests:'10+', status:'❓', risk:'crash', tier:'2' },
  { id:'contextra-audit-export', layer:0, order:8, desc:'Audit Log Export und DSGVO Art. 30 Verarbeitungsverzeichnis-Export', loc:'560', tests:'10+', status:'❓', risk:'none', tier:'2' },
  { id:'contextra-avv-generator', layer:0, order:9, desc:'AVV (Auftragsverarbeitungsvertrag) Template Generator', loc:'356', tests:'5+', status:'❓', risk:'none', tier:'2' },
  { id:'contextra-license', layer:4, order:32, desc:'Lizenz- und Aktivierungsprüfung für Contextra Feature-Ringe', loc:'456', tests:'15+', status:'❓', risk:'sec', tier:'2' },
  { id:'contextra-mcp', layer:9, order:33, desc:'Model Context Protocol Server & Cloud Egress Gateway', loc:'7073', tests:'112+', status:'❓', risk:'proto', tier:'1' }"""

assert old_mcp in h, "old_mcp missing"
h = h.replace(old_mcp, new_mcp_and_others)

old_calib_comp = """  'contextra-calibration': [
    { file:'isotonic.rs', size:'M', risk:'—', desc:'(unkommentiert — Existenz per gen-prompter-data verifiziert, redaktionelle Einschätzung ausstehend)', focus:'◆ AUSSTEHEND (Session: v27, TS: 2026-03-02) — Crate liegt außerhalb der Tier-1-Priorisierung dieser Sitzung.' },
  ],"""

new_rank_and_others_comp = """  'contextra-rank': [
    { file:'calibration/isotonic.rs', size:'M', risk:'—', desc:'Isotonische Score-Kalibrierung', focus:'Monotonie-Invariante wahren.' },
    { file:'calibration/platt.rs', size:'M', risk:'—', desc:'Platt-Skalierung Kalibrierung', focus:'Numerische Stabilität bei Sigmoid-Transformation sicherstellen.' },
    { file:'fusion/rrf.rs', size:'M', risk:'fusion', desc:'Reciprocal Rank Fusion (RRF)', focus:'mRRF Signal-Gewicht-Modulation wahren.' },
    { file:'lib.rs', size:'S', risk:'—', desc:'Modul-Exports der Ranking-Engine', focus:'Re-Exports der Kalibrierungs- und Fusion-Typen.' },
  ],
  'contextra-durable-fs': [
    { file:'lib.rs', size:'M', risk:'crash', desc:'Durable Dateisystem-Operationen (fsync, fdatasync, atomares Ersetzen)', focus:'Synchrones Flushen und atomare Datei-Operationen erzwingen.' },
  ],
  'contextra-audit-export': [
    { file:'lib.rs', size:'S', risk:'none', desc:'DSGVO Art. 30 Verarbeitungsverzeichnis-Export', focus:'Integrität des Exportformats sicherstellen.' },
  ],
  'contextra-avv-generator': [
    { file:'lib.rs', size:'S', risk:'none', desc:'AVV-Template-Generator (Art. 28 DSGVO)', focus:'Korrekte Vorlagen-Generierung ohne Leaks.' },
  ],
  'contextra-license': [
    { file:'lib.rs', size:'M', risk:'sec', desc:'Lizenz- und Aktivierungsprüfung für Feature-Ringe', focus:'Constant-time Hash-Vergleich und fixint bincode-Deserialisierung.' },
  ],"""

if old_calib_comp not in h:
    # try normalized replacement
    pos = h.find("'contextra-calibration': [")
    pos_end = h.find("],", pos) + 2
    h = h[:pos] + new_rank_and_others_comp[2:] + h[pos_end:]
else:
    h = h.replace(old_calib_comp, new_rank_and_others_comp)

h = h.replace("'contextra-calibration':", "'contextra-rank':")
h = h.replace('"contextra-calibration"', '"contextra-rank"')
h = h.replace("'contextra-calibration'", "'contextra-rank'")
h = h.replace("crates/contextra-calibration/", "crates/contextra-rank/")
h = h.replace("-p contextra-calibration", "-p contextra-rank")

assert "ALLE 31 Workspace-Member" in h, "PKG comment string missing"
h = h.replace("ALLE 31 Workspace-Member", "ALLE 35 Workspace-Member")


# === Task 2 ===
old_bootstrap_part = """8. run_in_bash_session("cargo xtask claim --crate ${claimCrate} --issue \\"$TASK_ID\\"")
   — Bei Claim-Konflikt: STOP. Nicht fortfahren. Meldung ausgeben.
9. run_in_bash_session("cargo run -p xtask -- check-duplicate-intent")
   — Prüft auf bereits offene, inhaltlich überlappende PR-Intents anderer Agenten-Sessions (Gate 12).
   Bei Treffer: STOP, bevor Arbeit an potenziell redundantem Feature beginnt (siehe docs/GITHUB_ANALYSE.md
   — 3 Agenten implementierten dieselbe Kalibrierungsfunktion parallel in #1627/#1634/#1645 und brachen main).
10. run_in_bash_session("cargo xtask session-init --crate ${claimCrate} --task \\"$TASK_ID\\" --output-env")
   — Erzeugt SESSION_HASH kanonisch (statt manueller date|sha256sum-Konstruktion) und gibt die Anzahl
   aktuell offener BLOCKER-Tags für diese Crate aus. Übernimm das ausgegebene SESSION_HASH für JEDES in
   dieser Sitzung geschriebene AI-TAG/ANCHOR/REVIEW-PASS (siehe Phase 0/SESSION_BLOCK unten — dort NICHT
   erneut generieren). Bei >0 offenen BLOCKERn: lies sie, bevor du entscheidest, ob diese Aufgabe davon
   betroffen ist.
11. run_in_bash_session("cargo xtask check-veto-deadlines")
   — Reine Frühwarnung (kein exit(1) bei bloß anstehender Frist, nur bei bereits ÜBERSCHRITTENER Frist).
   Macht laufende VETO-Review-Fristen (aktuell u.a. VETO-F02/VETO-OP03) sichtbar, bevor du in einem
   Bereich arbeitest, den ein offenes VETO einschränkt — prüfe bei Treffer, ob dein Scope betroffen ist.
12. run_in_bash_session("cargo run -p xtask -- jules-preflight --fast")
   — Bei Failure: Befund lesen, beheben, dann fortfahren
13. Optional, aber empfohlen als schneller Einstieg VOR der Detail-Lektüre der Zieldateien:
   run_in_bash_session("cargo xtask context-pack --crate ${claimCrate} --fast")
   — erzeugt \`.jules/context/CONTEXT_PACK.md\` als komprimierten Orientierungs-Digest dieser Crate.
   Ersetzt NICHT die Pflichtlektüre der tatsächlichen Zieldateien (Schritt 0 des Task-Protokolls unten)
   — reine Prosa aus einem generierten Digest ist kein Ersatz für das Lesen des echten Quellcodes."""

new_bootstrap_part = """8. run_in_bash_session("cargo xtask jules start --crate ${claimCrate} --issue \\"$TASK_ID\\"")
   — Kanonischer Start der Jules-Sitzung (führt Card-Linter, Claim-Reservierung, Session-Init und Context-Pack intern aus).
9. read_file(".jules/context/CONTEXT_PACK.md") — Kanonischen Context-Pack lesen (Orientierungs-Digest).
10. run_in_bash_session("cargo xtask check-veto-deadlines")
11. run_in_bash_session("cargo run -p xtask -- jules-preflight --fast")"""

assert old_bootstrap_part in h, "old_bootstrap_part missing"
h = h.replace(old_bootstrap_part, new_bootstrap_part)

old_jules_proto_code = """function JULES_PROTO(crate) {
  return `---

## Arbeitsprotokoll (strikt einzuhalten)

Arbeite eigenständig und ohne Rückfragen durch alle Schritte. Lies jede relevante Datei **vollständig** — ein einzelner Grep-Treffer ohne Kontext der umgebenden Funktion und des gesamten Moduls führt zu Folgefehlern und Fehlentscheidungen.

**Pflicht-Vorbereitung vor der ersten Codeänderung (in dieser Reihenfolge):**
1. Lies \`crates/${crate}/src/lib.rs\` vollständig — verstehe die Modulstruktur und alle pub-Exporte
2. Lies JEDE Zieldatei vollständig, einschließlich der Test-Sektion am Dateiende — kein Schreiben ohne vollständige Lektüre
3. Lies alle Dateien, die die Zieldatei via \`use crate::\`/\`mod\` importieren oder von ihr importiert werden — Seiteneffekte über Modulgrenzen sind die häufigste Fehlerquelle
4. Führe \`cargo check ${PKG(crate)}\` aus und dokumentiere den Ist-Zustand (Fehleranzahl, Warnungen)
5. Lies alle \`${AUDIT_GLOB(crate)}\`-Dateien (gesamte Audit-Historie dieser Crate) vollständig — bekannte Befunde und Sackgassen nicht erneut betreten
6. Falls vorhanden: lies alle Dateien in \`docs/audits/round2/\` mit \`${crate}\` im Dateinamen vollständig — dort stehen die tiefsten bereits gewonnenen Erkenntnisse zu diesem Crate

**Geschwister-Konsistenz (APM-6):** Wenn du eine Funktion änderst oder prüfst, suche aktiv nach strukturell ähnlichen Funktionen im selben Modul (gleicher Name-Präfix, gleiche Signatur-Form, gleiches Implementierungsmuster). Jede Geschwister-Funktion muss explizit geprüft und bei Bedarf gleichzeitig korrigiert werden — ein Befund gilt erst als behoben, wenn alle strukturell analogen Stellen untersucht sind.

**Nach jeder Codeänderung sofort:**
\`\`\`bash
cargo check ${PKG(crate)} --all-features 2>&1 | grep "^error"
\`\`\`

**Gate-Stack — vor jedem Commit zwingend (bei Fehler: sofort stoppen):**
\`\`\`bash
cargo check  ${PKG(crate)} --all-features
cargo clippy ${PKG(crate)} -- -D warnings
cargo fmt    --check ${PKG(crate)}
command -v cargo-nextest &>/dev/null && cargo nextest run ${PKG(crate)} --all-features || cargo test ${PKG(crate)} --all-features -- --nocapture
cargo check --workspace
\`\`\`

**Zusätzliche Governance-Gates (justfile/xtask — vor PR-Erstellung, nicht optional):**
\`\`\`bash
just check-vetoes                        # VETOES.md: keine permanenten Feature-Vetos in jüngsten Commits verletzt
just debt-audit                          # scannt .unwrap()/.expect()/unsafe/std::fs in Produktionscode gegen Baseline
cargo run -p xtask -- check-duplicate-symbols  # ADR-065: verhindert nicht-überlappende Top-Level-Konstanten-Kollisionen zwischen Branches
${crate === 'contextra-store' ? 'just chaos-test                           # Fault-Injection-Suite (Pflicht bei jeder wal.rs/lsm.rs/compaction.rs-Änderung)\\n' : ''}\`\`\`
${TOOLING_BLOCK(crate)}${GITHUB_BLOCK(crate)}${LOOP_BLOCK(crate)}${COMPONENT_SCOPE(crate)}`;
}"""

new_jules_proto_code = """function JULES_PROTO(crate) {
  return `---

## Arbeitsprotokoll (Kanonischer Jules Harness Workflow — strikt einzuhalten)

Arbeite eigenständig und ohne Rückfragen durch alle Schritte. Lies jede relevante Datei **vollständig** — ein einzelner Grep-Treffer ohne Kontext der umgebenden Funktion und des gesamten Moduls führt zu Folgefehlern und Fehlentscheidungen. Bei dem ersten fehlschlagenden Schritt sofort stoppen und Befund dokumentieren.

**1. Session starten:**
\`\`\`bash
cargo xtask jules start --crate ${crate} --issue "$TASK_ID"
# (Falls eine Task-Karte existiert: cargo xtask jules start --card <karte>)
\`\`\`
*(Internalisiert Card-Linting, Claim-Reservierung, Session-Init und Context-Pack-Erzeugung)*

**2. Pflicht-Lektüre vor jeder Codeänderung (in dieser Reihenfolge):**
1. Lies \`.jules/context/CONTEXT_PACK.md\` vollständig (Orientierung und Workspace-Kontext)
2. Lies \`crates/${crate}/src/lib.rs\` vollständig — verstehe die Modulstruktur und alle pub-Exporte
3. Lies JEDE Zieldatei im Scope vollständig, einschließlich der Test-Sektion am Dateiende — kein Schreiben ohne vollständige Lektüre
4. Lies alle Dateien, die die Zieldatei via \`use crate::\`/\`mod\` importieren oder von ihr importiert werden
5. Lies alle \`${AUDIT_GLOB(crate)}\`-Dateien (Audit-Historie dieser Crate) — bekannte Befunde und Sackgassen nicht erneut betreten

**3. Anker- & Konflikt-Protokoll:**
Prüfe auf laufende Arbeit anderer Sessions oder offene Blocker (\`ANCHOR[...]\` STATUS:IN-PROGRESS, \`AI-TAG[...][BLOCKER]\`). Bei Fund: STOP, Klärung herbeiführen.

**4. Iterative Entwicklung & Status-Check:**
\`\`\`bash
cargo xtask jules check
\`\`\`
Prüft kontinuierlich Syntax, Crate-Scope, geschützte Pfade und Duplikate.

**Geschwister-Konsistenz (APM-6):** Wenn du eine Funktion änderst oder prüfst, suche aktiv nach strukturell ähnlichen Funktionen im selben Modul. Jede Geschwister-Funktion muss explizit geprüft und gleichzeitig korrigiert werden.

**5. Verifikation & Akzeptanz-Gates:**
\`\`\`bash
cargo xtask jules verify
\`\`\`
Verifiziert Akzeptanz-Tests, Test-Coverage und Architektur-Gates. (Proof-of-Work: Test-Ergebnisse und Evidenz im Report dokumentieren — Prosa-Alleinbehauptung ist ungültig).

**6. Abschluss & Submit:**
Bei Erfolg:
\`\`\`bash
cargo xtask jules submit
\`\`\`
Bei unlösbaren Blockaden oder doppelten Fehlern:
\`\`\`bash
cargo xtask jules stop --reason <grund>
\`\`\`
${TOOLING_BLOCK(crate)}${GITHUB_BLOCK(crate)}${LOOP_BLOCK(crate)}${COMPONENT_SCOPE(crate)}`;
}"""

assert old_jules_proto_code in h, "old_jules_proto_code missing"
h = h.replace(old_jules_proto_code, new_jules_proto_code)


# === Task 3 ===
old_env_code = """function ENV_REALITY_BLOCK(crate) {
  return `---

## Umgebungs-Fakten (nur das nicht Selbstverständliche — Basis-Toolchain bereits per \`cargo xtask
env-validate\` im Bootstrap geprüft, hier nur Tools, die dort NICHT als Pflicht-Tool geführt werden)

\`\`\`bash
[ -f Cargo.toml ] && [ -f AGENTS.md ] || { echo "WARNUNG: nicht im Repo-Root"; find / -maxdepth 4 -name AGENTS.md 2>/dev/null; }
for t in cargo-llvm-cov cargo-mutants maturin gh; do command -v "$t" &>/dev/null || echo "FEHLT: $t"; done
\`\`\`

Fehlende Tools bei Bedarf nachinstallieren (\`cargo install <tool> --quiet\`); bei Fehlschlag im Audit-Report als \`[ÜBERSPRUNGEN: <tool> nicht installierbar]\` vermerken, nie stillschweigend auslassen. \`nix\` existiert nicht — kein \`nix develop\` versuchen, \`just\`-Rezepte laufen über den Cargo-Fallback. Netzwerk-Egress ist auf Git-Hosts und Paket-Registries beschränkt, keine freie Web-Recherche. Diese VM ist flüchtig: alles, was nicht committed und gepusht wird, ist beim Sitzungsende verloren — auch selbst nachinstallierte Tools.
`;
}"""

new_env_code = """function ENV_REALITY_BLOCK(crate) {
  return `---

## Umgebungs-Fakten (Basis-Toolchain per \`cargo xtask env-validate\` im Bootstrap geprüft)

\`\`\`bash
[ -f Cargo.toml ] && [ -f AGENTS.md ] || { echo "WARNUNG: nicht im Repo-Root"; find / -maxdepth 4 -name AGENTS.md 2>/dev/null; }
for t in cargo-llvm-cov cargo-mutants maturin gh cargo-fuzz cargo-insta cargo-hack cargo-nextest; do command -v "$t" &>/dev/null || echo "FEHLT: $t"; done
cargo +nightly --version &>/dev/null || echo "FEHLT: cargo +nightly"
\`\`\`

Fehlende Fuzz/Bench-Tools nicht per Ad-hoc-\`cargo install\` nachinstallieren, sondern die dedizierten Setup-Skripte nutzen:
- \`bash .jules/setup/install-fuzz-toolchain.sh\` (für Fuzzing/Nightly-Toolchain)
- \`bash .jules/setup/install-bench-deps.sh\` (für Benchmark-Abhängigkeiten)

Bei Fehlschlag im Audit-Report als \`[ÜBERSPRUNGEN: <tool> nicht installierbar]\` vermerken.
`;
}"""

assert old_env_code in h, "old_env_code missing"
h = h.replace(old_env_code, new_env_code)


# === Task 4 ===
old_apm_header = "## Anti-Pattern-Matrix — Pflicht-Scan (${active.length} Kategorien)"
new_apm_header = "## Anti-Pattern-Matrix — Pflicht-Scan (${active.length} Kategorien; Quelle: \`.jules/ANTI_PATTERNS.md\`)"

assert old_apm_header in h, "old_apm_header missing"
h = h.replace(old_apm_header, new_apm_header)


# === Task 5 ===
old_ss_decl = "const SS = { crate: null, task: null, subsystem: null, subsystemTask: null, fmt: 'full', par: 'on', components: new Set(), loop: 'on', maxLoop: '5', github: 'on', ablation: 'off', delegation: 'high' };"
new_ss_decl = "const FUZZ_CRATES = ['contextra-crypto', 'contextra-db', 'contextra-mcp', 'contextra-store', 'contextra-text', 'contextra-vector', 'contextra-wire'];\nconst SS = { crate: null, task: null, subsystem: null, subsystemTask: null, fmt: 'full', par: 'on', components: new Set(), loop: 'on', maxLoop: '5', github: 'on', ablation: 'off', delegation: 'high', fuzzbench: 'on' };"

assert old_ss_decl in h, "old_ss_decl missing"
h = h.replace(old_ss_decl, new_ss_decl)

fuzz_defs = """function FUZZ_BENCH_BLOCK(crate) {
  if (SS.fuzzbench === 'off' || !FUZZ_CRATES.includes(crate)) return '';
  return `---

## Fuzz / Bench / Mutation Verification (Fuzz-Crate: ${crate})

\`\`\`bash
# 1. Nightly Fuzz-Target-Lauf (im Crate-Verzeichnis crates/${crate}):
cargo +nightly fuzz run <target> -- -max_total_time=60

# 2. Fuzz-Smoke, Benchmark-Gate & Mutation-Diff:
just fuzz-smoke ${crate}
cargo xtask bench-gate
cargo xtask mutants-diff
\`\`\`
`;
}

function selFuzzBench(val) {
  SS.fuzzbench = val;
  const onEl = document.getElementById('fb-on');
  const offEl = document.getElementById('fb-off');
  if (onEl) onEl.classList.toggle('on', val === 'on');
  if (offEl) offEl.classList.toggle('on', val === 'off');
  generate();
}

function updateFuzzBenchVisibility(crate) {
  const el = document.getElementById('fuzzbench-section');
  if (el) {
    el.style.display = (crate && FUZZ_CRATES.includes(crate)) ? 'block' : 'none';
  }
}

"""

pos_loop = h.find("function LOOP_BLOCK(crate) {")
assert pos_loop != -1, "pos_loop missing"
h = h[:pos_loop] + fuzz_defs + h[pos_loop:]

# selCrate hook
old_selcrate = "initComponents(id);\n  const invResult = document.getElementById('inv-result');"
new_selcrate = "initComponents(id);\n  updateFuzzBenchVisibility(id);\n  const invResult = document.getElementById('inv-result');"

assert old_selcrate in h, "old_selcrate missing"
h = h.replace(old_selcrate, new_selcrate)

# generate() hook
old_gen_code = """  _rawPrompt = mandatoryBootstrap + promptBody;"""
new_gen_code = """  promptBody += FUZZ_BENCH_BLOCK(effectiveCrate);
  _rawPrompt = mandatoryBootstrap + promptBody;"""

assert old_gen_code in h, "old_gen_code missing"
h = h.replace(old_gen_code, new_gen_code)

# HTML UI element
old_gh_ui = """        <div class="sl"><span class="sl-dot" style="background:var(--purple)"></span>GitHub-Kontext-Injektion (Issue/PR/Historie)</div>
        <div class="chips">
          <div class="chip on" id="gh-on" onclick="selGithub('on')">Aktiv</div>
          <div class="chip" id="gh-off" onclick="selGithub('off')">Deaktiviert</div>
        </div>"""

new_gh_ui = """        <div class="sl"><span class="sl-dot" style="background:var(--purple)"></span>GitHub-Kontext-Injektion (Issue/PR/Historie)</div>
        <div class="chips">
          <div class="chip on" id="gh-on" onclick="selGithub('on')">Aktiv</div>
          <div class="chip" id="gh-off" onclick="selGithub('off')">Deaktiviert</div>
        </div>
        <div id="fuzzbench-section" style="display:none;margin-top:10px">
          <div class="sl"><span class="sl-dot" style="background:var(--red)"></span>Fuzz / Bench / Mutation Baustein</div>
          <div class="chips">
            <div class="chip on" id="fb-on" onclick="selFuzzBench('on')">Aktiv</div>
            <div class="chip" id="fb-off" onclick="selFuzzBench('off')">Ausblenden</div>
          </div>
        </div>"""

assert old_gh_ui in h, "old_gh_ui missing"
h = h.replace(old_gh_ui, new_gh_ui)

with open('.jules/contextra-Prompter.html', 'w', encoding='utf-8') as f:
    f.write(h)

print("clean_patch.py executed successfully.")
