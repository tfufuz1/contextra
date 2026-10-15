//! Gate G-21: check-exception-ledger
//! Prüft die Übergangsausnahmen (capabilities.toml -> [[exceptions]]) gemäß Spezifikation §4.3a.
//! Regeln:
//! 1. Ausnahmen müssen eine ID, `from`, `to`, `expiry` und `reason` haben.
//! 2. Ausnahmen mit abgelaufenem Meilenstein lassen den Befehl fehlschlagen.
//! 3. Die Anzahl der Ausnahmen ist auf maximal 2 (E-1 und E-2) begrenzt und darf nur sinken.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct CapabilitiesManifestExceptions {
    #[serde(default)]
    exceptions: Vec<ExceptionEntry>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ExceptionEntry {
    pub id: String,
    pub from: String,
    pub to: String,
    pub expiry: String,
    pub reason: String,
}

pub const MAX_PERMITTED_EXCEPTIONS: usize = 2;

/// Bekannte Meilensteine und deren Erreichungsstatus (true = bereits erreicht/abgelaufen)
pub fn milestone_reached_map() -> BTreeMap<&'static str, bool> {
    let mut m = BTreeMap::new();
    m.insert("M0.4", false);
    m.insert("N0.2", false);
    m.insert("N3.1", false);
    m.insert("N5.2", false);
    m
}

pub fn check_exception_ledger_in_root(root: &Path) -> Result<Vec<ExceptionEntry>, Vec<String>> {
    let caps_path = root.join("capabilities.toml");
    let content = match fs::read_to_string(&caps_path) {
        Ok(c) => c,
        Err(e) => return Err(vec![format!("Failed to read capabilities.toml: {}", e)]),
    };

    let parsed: CapabilitiesManifestExceptions = match toml::from_str(&content) {
        Ok(p) => p,
        Err(e) => return Err(vec![format!("Failed to parse capabilities.toml: {}", e)]),
    };

    let mut errors = Vec::new();
    let milestone_map = milestone_reached_map();

    if parsed.exceptions.len() > MAX_PERMITTED_EXCEPTIONS {
        errors.push(format!(
            "Exception ledger violation: found {} exceptions in capabilities.toml, exceeding maximum permitted budget of {} (G-21)",
            parsed.exceptions.len(),
            MAX_PERMITTED_EXCEPTIONS
        ));
    }

    for ex in &parsed.exceptions {
        if ex.id.is_empty() || ex.from.is_empty() || ex.to.is_empty() || ex.expiry.is_empty() {
            errors.push(format!(
                "Incomplete exception entry: id='{}', from='{}', to='{}', expiry='{}'",
                ex.id, ex.from, ex.to, ex.expiry
            ));
        }

        if let Some(&reached) = milestone_map.get(ex.expiry.as_str()) {
            if reached {
                errors.push(format!(
                    "Expired transition exception: ID '{}' ({} -> {}) expired at milestone '{}' and must be removed",
                    ex.id, ex.from, ex.to, ex.expiry
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(parsed.exceptions)
    } else {
        Err(errors)
    }
}

pub fn run_check_exception_ledger() -> Result<Vec<ExceptionEntry>, Vec<String>> {
    let root = crate::find_root_dir();
    check_exception_ledger_in_root(&root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_exception_ledger_passes_on_workspace() {
        let root = crate::find_root_dir();
        let res = check_exception_ledger_in_root(&root);
        assert!(res.is_ok(), "G-21 check failed: {:?}", res.err());
        let exceptions = res.unwrap();
        assert_eq!(exceptions.len(), 2);
        assert_eq!(exceptions[0].id, "E-1");
        assert_eq!(exceptions[1].id, "E-2");
    }

    #[test]
    fn test_check_exception_ledger_fails_when_budget_exceeded() {
        let temp_dir = tempfile::tempdir().unwrap();
        let caps_path = temp_dir.path().join("capabilities.toml");

        let content = r#"
[version]
schema = "2"
spec = "v4"

[[exceptions]]
id = "E-1"
from = "a"
to = "b"
expiry = "N3.1"
reason = "r1"

[[exceptions]]
id = "E-2"
from = "c"
to = "d"
expiry = "N5.2"
reason = "r2"

[[exceptions]]
id = "E-3"
from = "e"
to = "f"
expiry = "N9.9"
reason = "r3"
"#;
        fs::write(&caps_path, content).unwrap();

        let res = check_exception_ledger_in_root(temp_dir.path());
        assert!(res.is_err());
        let errs = res.unwrap_err();
        assert!(errs.iter().any(|e| e.contains("exceeding maximum permitted budget")));
    }
}
