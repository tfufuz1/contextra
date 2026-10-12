import re

with open('.jules/contextra-Prompter.html', 'r', encoding='utf-8') as f:
    h = f.read()

# --- 1. DEFAULT_CRATES ---
old_mcp = "  { id:'contextra-mcp', layer:9, order:31, desc:'Model Context Protocol Server & Cloud Egress Gateway', loc:'7073', tests:'112+', status:'❓', risk:'proto', tier:'1' }"
new_mcp_and_others = """  { id:'contextra-durable-fs', layer:0, order:7, desc:'Durable Filesystem Operations und synchrone Disk-Flushes', loc:'790', tests:'10+', status:'❓', risk:'crash', tier:'2' },
  { id:'contextra-audit-export', layer:0, order:8, desc:'Audit Log Export und DSGVO Art. 30 Verarbeitungsverzeichnis-Export', loc:'560', tests:'10+', status:'❓', risk:'none', tier:'2' },
  { id:'contextra-avv-generator', layer:0, order:9, desc:'AVV (Auftragsverarbeitungsvertrag) Template Generator', loc:'356', tests:'5+', status:'❓', risk:'none', tier:'2' },
  { id:'contextra-license', layer:4, order:32, desc:'Lizenz- und Aktivierungsprüfung für Contextra Feature-Ringe', loc:'456', tests:'15+', status:'❓', risk:'sec', tier:'2' },
  { id:'contextra-mcp', layer:9, order:33, desc:'Model Context Protocol Server & Cloud Egress Gateway', loc:'7073', tests:'112+', status:'❓', risk:'proto', tier:'1' }"""

if old_mcp in h:
    h = h.replace(old_mcp, new_mcp_and_others)

# --- 2. DEFAULT_COMPONENTS & CRATE_PROFILES ---
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

if old_calib_comp in h:
    h = h.replace(old_calib_comp, new_rank_and_others_comp)

# Rename contextra-calibration in CRATE_PROFILES and elsewhere
h = h.replace("'contextra-calibration':", "'contextra-rank':")
h = h.replace('"contextra-calibration"', '"contextra-rank"')
h = h.replace("'contextra-calibration'", "'contextra-rank'")
h = h.replace("crates/contextra-calibration/", "crates/contextra-rank/")
h = h.replace("-p contextra-calibration", "-p contextra-rank")

# PKG comment
h = h.replace("ALLE 31 Workspace-Member", "ALLE 35 Workspace-Member")

# --- 3. buildMandatoryBootstrap & JULES_PROTO ---
old_bootstrap_block = h[h.find("function buildMandatoryBootstrap"):h.find("function JULES_PROTO")]

new_bootstrap_block = """function buildMandatoryBootstrap(claimCrate, taskId) {
  return `---MANDATORY BOOTSTRAP (nicht überspringen, nicht kürzen)---
${WORST_CASE_DOCTRINE_BLOCK()}
[SCHRITT 0 — Inventar-Realitätsabgleich, IMMER ZUERST, in dieser Reihenfolge]
1. run_in_bash_session("git status; git log -n 5 --oneline; git remote -v")
   — Working-Tree sauber? Aktueller Branch? Ist origin/main erreichbar?
2. run_in_bash_session("cargo xtask env-validate")
   — Harter Toolchain-Gate, ALS ERSTES ausführbares Kommando dieser Sitzung, noch vor jeder Lektüre.
3. read_file("AGENTS.md") — vollständig lesen
4. read_file(".jules/SESSION_BOOTSTRAP.md") — maschinenausführbare Bootstrap-Checkliste
5. read_file(".jules/JULES_CONTEXT.md") — permanenter Ambient-Context-Snapshot
6. run_in_bash_session("head -80 WORKING_STATE.md") — Crate-Stand prüfen
7. run_in_bash_session("export TASK_ID='${taskId}'")
8. run_in_bash_session("cargo xtask jules start --crate ${claimCrate} --issue \\"$TASK_ID\\"")
   — Kanonischer Start der Jules-Sitzung (führt Card-Linter, Claim-Reservierung, Session-Init und Context-Pack intern aus).
9. read_file(".jules/context/CONTEXT_PACK.md") — Kanonischen Context-Pack lesen (Orientierungs-Digest).
10. run_in_bash_session("cargo xtask check-veto-deadlines")
11. run_in_bash_session("cargo run -p xtask -- jules-preflight --fast")
---END MANDATORY BOOTSTRAP---

`;
}

"""

h = h[:h.find("function buildMandatoryBootstrap")] + new_bootstrap_block + h[h.find("function JULES_PROTO"):]

# Replace JULES_PROTO
proto_start = h.find("function JULES_PROTO(crate) {")
proto_end = h.find("function INVENTORY_CHECK_BLOCK", proto_start)

new_jules_proto = """function JULES_PROTO(crate) {
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
}

"""

h = h[:proto_start] + new_jules_proto + h[proto_end:]

# --- 4. ENV_REALITY_BLOCK ---
env_start = h.find("function ENV_REALITY_BLOCK(crate) {")
env_end = h.find("\n}\n", env_start) + 3

new_env = """function ENV_REALITY_BLOCK(crate) {
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
}
"""

h = h[:env_start] + new_env + h[env_end:]

# --- 5. APM_BLOCK ---
apm_start = h.find("function APM_BLOCK(selectedAPMs) {")
apm_end = h.find("\n}\n", apm_start) + 3

new_apm = """function APM_BLOCK(selectedAPMs) {
  const active = APM.filter(a => selectedAPMs.has(a.id));
  if (!active.length) return '';
  return `---

## Anti-Pattern-Matrix — Pflicht-Scan (${active.length} Kategorien; Quelle: \`.jules/ANTI_PATTERNS.md\`)

${active.map(a => `### ${a.short}
${a.desc}`).join('\\n\\n')}
`;
}
"""

h = h[:apm_start] + new_apm + h[apm_end:]

# --- 6. FUZZ_BENCH_BLOCK, selFuzzBench, updateFuzzBenchVisibility, SS declaration ---
old_ss_decl = "const SS = { crate: null, task: null, subsystem: null, subsystemTask: null, fmt: 'full', par: 'on', components: new Set(), loop: 'on', maxLoop: '5', github: 'on', ablation: 'off', delegation: 'high' };"
new_ss_decl = "const FUZZ_CRATES = ['contextra-crypto', 'contextra-db', 'contextra-mcp', 'contextra-store', 'contextra-text', 'contextra-vector', 'contextra-wire'];\nconst SS = { crate: null, task: null, subsystem: null, subsystemTask: null, fmt: 'full', par: 'on', components: new Set(), loop: 'on', maxLoop: '5', github: 'on', ablation: 'off', delegation: 'high', fuzzbench: 'on' };"

h = h.replace(old_ss_decl, new_ss_decl)

fuzz_defs = """
function FUZZ_BENCH_BLOCK(crate) {
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
h = h[:pos_loop] + fuzz_defs + h[pos_loop:]

# Hook up updateFuzzBenchVisibility in selCrate
old_selcrate_code = "document.getElementById('s-layer').textContent = `Layer ${c.layer}`;\n  initComponents(id);"
new_selcrate_code = "document.getElementById('s-layer').textContent = `Layer ${c.layer}`;\n  initComponents(id);\n  updateFuzzBenchVisibility(id);"

h = h.replace(old_selcrate_code, new_selcrate_code)

# Hook up FUZZ_BENCH_BLOCK in generate()
pos_gen = h.find("function generate() {")
pos_gen_end = h.find("\n}\n", pos_gen)
gen_code = h[pos_gen:pos_gen_end]

old_gen_assign = "_rawPrompt = mandatoryBootstrap + promptBody;"
new_gen_assign = "promptBody += FUZZ_BENCH_BLOCK(effectiveCrate);\n  _rawPrompt = mandatoryBootstrap + promptBody;"

gen_code_updated = gen_code.replace(old_gen_assign, new_gen_assign)
h = h[:pos_gen] + gen_code_updated + h[pos_gen_end:]

# Add UI controls element in HTML
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

if old_gh_ui in h:
    h = h.replace(old_gh_ui, new_gh_ui)

with open('.jules/contextra-Prompter.html', 'w', encoding='utf-8') as f:
    f.write(h)

print("apply_all_fixes.py completed.")
