//! # Schichtarchitektur-Test
//!
//! Erzwingt die Schichtregeln aus SPEC §3.3 und die Präzisierungen aus IMPL §5.1:
//!
//! ```text
//! api     ──► engine, erase, catalog, keys, codec      (Typquellen)
//! erase   ──► engine (nach oben); keys, codec, env     (nach unten)
//! engine  ──► redo, run, blob, catalog
//! redo, run, blob, catalog ──► keys, codec, env
//! keys    ──► codec, env
//! codec, env ──► keine internen Module
//! testing ──► alles; außerhalb von testing nutzt niemand testing
//! ```
//!
//! Erfasst werden alle Referenzen `crate::<modul>` und `super::<…>` auf ein Top-Level-Modul:
//! `use`-Zeilen, gruppierte und mehrzeilige Importe sowie Inline-Pfade im Code.
//! Zeilenkommentare werden ignoriert. Neue Top-Level-Module müssen in `REGELN` eingeordnet
//! werden, sonst schlägt der Test fehl.

use std::fs;
use std::path::{Path, PathBuf};

/// Erlaubte Zugriffe je Schicht. Das eigene Modul ist immer erlaubt.
/// `sync` ist die Fassade über `std::sync` (SPEC §3.2) und nur dort freigegeben, wo Sperren nötig sind.
/// `erase` darf laut SPEC §3.3 nur `engine` nutzen; die Kanten nach unten (`keys`, `codec`, `env`)
/// sind eine Auslegung, die im PR zur Bestätigung steht.
const REGELN: &[(&str, &[&str])] = &[
    (
        "api",
        &["engine", "erase", "catalog", "keys", "codec", "sync"],
    ),
    ("erase", &["engine", "keys", "codec", "env", "sync"]),
    (
        "engine",
        &[
            "redo", "run", "blob", "catalog", "keys", "codec", "env", "sync",
        ],
    ),
    ("redo", &["keys", "codec", "env", "sync"]),
    ("run", &["keys", "codec", "env", "sync"]),
    ("blob", &["keys", "codec", "env", "sync"]),
    ("catalog", &["keys", "codec", "env", "sync"]),
    ("keys", &["codec", "env", "sync"]),
    ("codec", &[]),
    ("env", &[]),
];

const TESTING: &str = "testing";
const SYNC: &str = "sync";

/// Dateien direkt unter `src/`, die keiner Schicht angehören.
const WURZEL_DATEIEN: &[&str] = &["lib.rs", "sync.rs"];

/// Bekannte Verletzungen: (Datei relativ zu `src/`, Zielmodul, Grund mit Spec-Verweis).
/// Leer, weil der Stand des Repos alle Regeln erfüllt. Ein Eintrag, der keine Verletzung
/// mehr trifft, lässt den Test fehlschlagen, damit die Liste nicht veraltet.
const AUSNAHMEN: &[(&str, &str, &str)] = &[];

#[derive(Debug, Clone, PartialEq, Eq)]
struct Verstoss {
    datei: String,
    /// 0, wenn der Verstoß nicht einer Zeile zuzuordnen ist (z. B. nicht eingeordnetes Modul).
    zeile: usize,
    nach: String,
    grund: String,
}

impl Verstoss {
    fn anzeige(&self) -> String {
        if self.zeile == 0 {
            format!("{}: {}", self.datei, self.grund)
        } else {
            format!("{}:{}: {}", self.datei, self.zeile, self.grund)
        }
    }
}

fn ist_ident_zeichen(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Erster Modulname eines Pfads. `self`, `super`, `crate` und Platzhalter zählen nicht.
fn ident_am_anfang(s: &str) -> Option<String> {
    let ident: String = s.chars().take_while(|c| ist_ident_zeichen(*c)).collect();
    if ident.is_empty() || matches!(ident.as_str(), "self" | "super" | "crate") {
        None
    } else {
        Some(ident)
    }
}

/// Inhalt einer Klammergruppe. `s` beginnt direkt nach der öffnenden `{`.
fn gruppen_inhalt(s: &str) -> Option<&str> {
    let mut tiefe = 1usize;
    for (i, c) in s.char_indices() {
        match c {
            '{' => tiefe += 1,
            '}' => {
                tiefe -= 1;
                if tiefe == 0 {
                    return s.get(..i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Teilt den Inhalt einer Gruppe an Kommas der obersten Ebene.
fn oberste_elemente(inhalt: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut tiefe = 0usize;
    let mut anfang = 0usize;
    for (i, c) in inhalt.char_indices() {
        match c {
            '{' => tiefe += 1,
            '}' => tiefe = tiefe.saturating_sub(1),
            ',' if tiefe == 0 => {
                out.push(inhalt.get(anfang..i).unwrap_or(""));
                anfang = i + 1;
            }
            _ => {}
        }
    }
    out.push(inhalt.get(anfang..).unwrap_or(""));
    out
}

/// Top-Level-Module, auf die ein Pfad ab `rest` zugreift: ein Ident oder die Elemente einer Gruppe.
fn ziele_ab(rest: &str) -> Vec<String> {
    if let Some(gruppe) = rest.strip_prefix('{') {
        return gruppen_inhalt(gruppe)
            .map(|inhalt| {
                oberste_elemente(inhalt)
                    .into_iter()
                    .filter_map(|e| ident_am_anfang(e.trim_start()))
                    .collect()
            })
            .unwrap_or_default();
    }
    ident_am_anfang(rest).into_iter().collect()
}

/// Entfernt Zeilenkommentare, die Zeilenzahl bleibt erhalten.
fn ohne_zeilenkommentare(inhalt: &str) -> String {
    inhalt
        .lines()
        .map(|zeile| zeile.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Alle Referenzen auf Top-Level-Module als (Zeile, Modulname).
/// `modul_pfad` ist der Modulpfad der Datei und löst `super::` auf.
fn referenzen(code: &str, modul_pfad: &[String]) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut zeile = 1usize;
    for (idx, c) in code.char_indices() {
        if c == '\n' {
            zeile += 1;
            continue;
        }
        // Nur Pfadanfänge: davor darf weder ein Identifierzeichen noch `:` stehen.
        let vor = code.get(..idx).and_then(|p| p.chars().next_back());
        if vor.is_some_and(|v| ist_ident_zeichen(v) || v == ':') {
            continue;
        }
        let Some(rest) = code.get(idx..) else {
            continue;
        };
        if let Some(pfad) = rest.strip_prefix("crate::") {
            for modul in ziele_ab(pfad) {
                out.push((zeile, modul));
            }
        } else if rest.starts_with("super::") {
            let mut k = 0usize;
            let mut pfad = rest;
            while let Some(r) = pfad.strip_prefix("super::") {
                k += 1;
                pfad = r;
            }
            // Aus Modul `a::b` zeigt `super` auf `a` (gleiche Schicht). Erst wenn `super`
            // die Wurzel erreicht, zählt das Ziel als Top-Level-Modul.
            if modul_pfad.len() <= k {
                for modul in ziele_ab(pfad) {
                    out.push((zeile, modul));
                }
            }
        }
    }
    out
}

/// Modulpfad einer Datei relativ zu `src/`, z. B. `engine/compaction/exec.rs` -> `engine/compaction/exec`.
fn modul_pfad(rel: &str) -> Vec<String> {
    if rel == "lib.rs" {
        return Vec::new();
    }
    let ohne_endung = rel.strip_suffix(".rs").unwrap_or(rel);
    let mut teile: Vec<String> = ohne_endung.split('/').map(str::to_owned).collect();
    if teile.last().is_some_and(|t| t.as_str() == "mod") {
        teile.pop();
    }
    teile
}

fn ist_bekanntes_modul(name: &str) -> bool {
    name == TESTING || name == SYNC || REGELN.iter().any(|r| r.0 == name)
}

/// Prüft das Verbot, testing außerhalb von testing zu nutzen, und die Kanten aus `REGELN`.
fn verletzung(von: &str, nach: &str) -> Option<String> {
    if nach == TESTING {
        return (von != TESTING)
            .then(|| "Produktionscode darf nicht von 'testing' abhängen (SPEC §3.3)".to_owned());
    }
    if von == TESTING {
        return None;
    }
    // Unbekannte Herkunft (Wurzeldateien, `sync`) ist nicht eingeschränkt.
    let erlaubt = REGELN.iter().find(|r| r.0 == von)?.1;
    if erlaubt.contains(&nach) {
        None
    } else {
        Some(format!(
            "Schichtverletzung: '{von}' darf nicht von '{nach}' abhängen (SPEC §3.3)"
        ))
    }
}

fn pruefe_datei(rel: &str, inhalt: &str) -> Vec<Verstoss> {
    let pfad = modul_pfad(rel);
    let von = pfad.first().cloned().unwrap_or_default();
    let code = ohne_zeilenkommentare(inhalt);
    referenzen(&code, &pfad)
        .into_iter()
        .filter(|(_, ziel)| ist_bekanntes_modul(ziel) && ziel.as_str() != von.as_str())
        .filter_map(|(zeile, ziel)| {
            verletzung(&von, &ziel).map(|grund| Verstoss {
                datei: rel.to_owned(),
                zeile,
                nach: ziel,
                grund,
            })
        })
        .collect()
}

fn sammle_rs(dir: &Path, basis: &Path, out: &mut Vec<(String, PathBuf)>) {
    let Ok(eintraege) = fs::read_dir(dir) else {
        return;
    };
    for eintrag in eintraege.flatten() {
        let pfad = eintrag.path();
        if pfad.is_dir() {
            sammle_rs(&pfad, basis, out);
        } else if pfad.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(rel) = pfad.strip_prefix(basis) {
                let rel = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                out.push((rel, pfad.clone()));
            }
        }
    }
}

/// Top-Level-Verzeichnisse und Wurzeldateien, die in `REGELN` oder `WURZEL_DATEIEN` fehlen.
fn nicht_eingeordnet(src: &Path) -> Vec<Verstoss> {
    let Ok(eintraege) = fs::read_dir(src) else {
        return Vec::new();
    };
    eintraege
        .flatten()
        .filter_map(|e| {
            let pfad = e.path();
            let name = e.file_name().to_string_lossy().into_owned();
            let grund = if pfad.is_dir() {
                let bekannt = name == TESTING || REGELN.iter().any(|r| r.0 == name.as_str());
                (!bekannt).then(|| {
                    format!("Modul '{name}' ist in den Schichtregeln nicht eingeordnet (SPEC §3.3)")
                })
            } else if pfad.extension().and_then(|x| x.to_str()) == Some("rs") {
                let bekannt = WURZEL_DATEIEN.contains(&name.as_str());
                (!bekannt)
                    .then(|| format!("Wurzeldatei '{name}' ist nicht eingeordnet (SPEC §3.2)"))
            } else {
                None
            };
            grund.map(|grund| Verstoss {
                datei: name,
                zeile: 0,
                nach: String::new(),
                grund,
            })
        })
        .collect()
}

#[test]
fn schichtregeln_gelten_fuer_den_gesamten_quellcode() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut verstoesse = nicht_eingeordnet(&src);

    let mut dateien = Vec::new();
    sammle_rs(&src, &src, &mut dateien);
    dateien.sort();
    for (rel, pfad) in &dateien {
        match fs::read_to_string(pfad) {
            Ok(inhalt) => verstoesse.extend(pruefe_datei(rel, &inhalt)),
            Err(e) => verstoesse.push(Verstoss {
                datei: rel.clone(),
                zeile: 0,
                nach: String::new(),
                grund: format!("nicht lesbar: {e}"),
            }),
        }
    }

    let mut fehler: Vec<String> = Vec::new();
    for (datei, ziel, grund) in AUSNAHMEN {
        if !verstoesse
            .iter()
            .any(|v| v.datei == *datei && v.nach == *ziel)
        {
            fehler.push(format!("Ausnahme veraltet: {datei} -> {ziel} ({grund})"));
        }
    }
    fehler.extend(
        verstoesse
            .iter()
            .filter(|v| {
                !AUSNAHMEN
                    .iter()
                    .any(|(d, z, _)| *d == v.datei && *z == v.nach)
            })
            .map(Verstoss::anzeige),
    );
    assert!(
        fehler.is_empty(),
        "Schichtverletzungen ({}):\n{}",
        fehler.len(),
        fehler.join("\n")
    );
}

// Synthetische Fälle: prüfen die Regeln, ohne den Quellbaum zu verändern.

#[test]
fn codec_ist_senke_ohne_interne_module() {
    let v = pruefe_datei("codec/varint.rs", "use crate::sync::Mutex;\n");
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].nach, "sync");
}

#[test]
fn env_ist_senke_ohne_interne_module() {
    let v = pruefe_datei("env/clock.rs", "use crate::sync::S;\n");
    assert_eq!(v.len(), 1);
}

#[test]
fn inline_pfad_wird_erfasst() {
    let inhalt = "fn f() {\n    let x = crate::engine::Flush::new();\n}\n";
    let v = pruefe_datei("redo/frame.rs", inhalt);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].zeile, 2);
    assert_eq!(v[0].nach, "engine");
}

#[test]
fn gruppierter_mehrzeiliger_import_wird_erfasst() {
    let inhalt = "use crate::{\n    keys::A,\n    blob::{B, C},\n    codec::D,\n};\n";
    let v = pruefe_datei("redo/frame.rs", inhalt);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].nach, "blob");
}

#[test]
fn kommentare_werden_ignoriert() {
    let v = pruefe_datei("redo/frame.rs", "// use crate::engine::X;\n");
    assert!(v.is_empty());
}

#[test]
fn wurzelitem_ist_kein_modul() {
    let v = pruefe_datei("codec/varint.rs", "use crate::Error;\n");
    assert!(v.is_empty());
}

#[test]
fn erase_darf_nur_engine_nach_oben() {
    assert_eq!(
        pruefe_datei("erase/plan.rs", "use crate::blob::B;\n").len(),
        1
    );
    assert!(pruefe_datei("erase/plan.rs", "use crate::engine::E;\n").is_empty());
}

#[test]
fn api_nutzt_nur_erlaubte_quellen() {
    assert_eq!(
        pruefe_datei("api/listener.rs", "use crate::redo::R;\n").len(),
        1
    );
    assert!(pruefe_datei("api/listener.rs", "use crate::catalog::C;\n").is_empty());
}

#[test]
fn engine_darf_nicht_nach_oben_zu_api() {
    assert_eq!(
        pruefe_datei("engine/flush.rs", "use crate::api::Storage;\n").len(),
        1
    );
}

#[test]
fn testing_nur_von_testing_genutzt() {
    assert_eq!(
        pruefe_datei("engine/flush.rs", "use crate::testing::Fake;\n").len(),
        1
    );
    assert!(pruefe_datei("testing/mod.rs", "use crate::testing::x;\n").is_empty());
    assert_eq!(
        pruefe_datei("lib.rs", "pub use crate::testing::Fake;\n").len(),
        1
    );
}

#[test]
fn super_aus_modulwurzel_wird_aufgeloest() {
    // In run/mod.rs zeigt `super` auf die Krate-Wurzel, also auf das Modul `redo`.
    assert_eq!(pruefe_datei("run/mod.rs", "use super::redo::R;\n").len(), 1);
    // In run/block.rs bleibt `super` innerhalb von `run`.
    assert!(pruefe_datei("run/block.rs", "use super::error::E;\n").is_empty());
    // Zwei Ebenen nach oben erreichen die Wurzel: `blob` ist für `run` verboten.
    assert_eq!(
        pruefe_datei("run/data.rs", "use super::super::blob::B;\n").len(),
        1
    );
}

#[test]
fn modulpfad_und_mod_rs() {
    assert_eq!(modul_pfad("engine/mod.rs"), vec!["engine".to_owned()]);
    assert_eq!(
        modul_pfad("engine/compaction/exec.rs"),
        vec![
            "engine".to_owned(),
            "compaction".to_owned(),
            "exec".to_owned()
        ]
    );
    assert!(modul_pfad("lib.rs").is_empty());
}
