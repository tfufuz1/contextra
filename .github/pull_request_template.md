<!-- Jules CI/CD Fixer aktivieren: Kommentiere "@jules fix-ci" im PR -->
<!-- Jules Gate-Prüfung: Kommentiere "@jules check-gates" -->
<!-- Jules Submit-Gate: cargo xtask jules-submit-gate --crate <CRATE> -->

## WP-Zuordnung
WP: <!-- z.B. WP 2.2 -->
Crate: <!-- primäre Crate -->
Ring: <!-- 0–4 -->

---

## Checkliste

### Pflicht (CI blockierend)
- [ ] `cargo test --workspace` grün
- [ ] `just check` grün (Format + Clippy)
- [ ] `cargo xtask jules-submit-gate --crate <CRATE>` ✅ SUBMIT GATE BESTANDEN
- [ ] Rebase auf origin/main: `git fetch origin && git rebase origin/main`

### Gates (CI blockierend — SPEC §17.3)
- [ ] `ring-layering`: keine Aufwärtskante (Ringdisziplin P1/P5)
- [ ] `no-security-stub`: kein `cfg(not(feature))` für Security-Typen (A1)
- [ ] `panic-surface`: kein `unwrap`/`expect`/`panic!` in Produktionspfaden (P7)
- [ ] `determinism`: Zeit/Zufall/IDs nur über Ports `Clock`/`Rng`/`IdGen` (P28)
- [ ] `async-io`: kein direktes I/O in Ring-0-async (P26)
- [ ] `witness-mint`: kein direktes Erstellen von `DeletionWitness` außerhalb des Purgers (INV-WITNESS-MINT-1)
- [ ] `no-literal-secrets`: kein hartkodierter Schlüssel/Secret-Literal (REQ-KEY-010)
- [ ] `test-surface`: keine `*_for_test`-Symbole in public API (INV-SCAN-1)
- [ ] `doc-stamp`: Commit-Stempel in geänderten Docs aktualisiert (INV-DOC-SINGLE-SOURCE-1)

### Veto-Check (SPEC §3.1)
- [ ] **VETO-F10:** Kein mandantenübergreifender Datenfluss
- [ ] **VETO-F02:** HNSW: kein Rewiring (nur Tombstone + Slot-Nullung + Rebuild)
- [ ] **Single-Node-Doktrin:** kein Sharding, kein P2P, kein verteilter Betrieb

### Unsafe (nur in simd / sys / wire)
- [ ] Kein neues `unsafe` ohne `// SAFETY:` Begründung (P2)
- [ ] `unsafe` nur in genehmigten Inseln: `contextra-simd`, `contextra-sys`, `contextra-wire`

---

## Beschreibung

### Was wurde geändert?


### Warum? (Invariante / Anforderung)
Invariante/REQ: <!-- z.B. INV-WIRING-2, REQ-DEL-001 -->


### Verifikation
Befehl: <!-- z.B. cargo nextest run -p contextra-store -->

---

## Jules CI/CD Integration
<!--
@jules fix-ci      → Jules analysiert und behebt CI-Fehler automatisch
@jules check-gates → Jules prüft alle Gates und meldet Ergebnis
-->

---

## Out-of-scope Findings


---

## Risiken / Rollback


---

## ADR / Spec-Sync
ADR-Nummer: <!-- falls architektonisch relevant -->
SPEC-Abschnitt: <!-- z.B. §4.2, §12.7 -->
