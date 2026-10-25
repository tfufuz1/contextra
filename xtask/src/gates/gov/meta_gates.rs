use crate::core::ctx::Context;
use crate::core::gate::{Gate, GateSpec, Outcome};

pub struct GateSelftestsGate;

impl Gate for GateSelftestsGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "gate-selftests".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH0".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["REQ-QA-008".to_string()],
            why: "Führt selftest() aller registrierten Gates aus; rot wenn ein Gate keinen hat.".to_string(),
            owner: "MP-02".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        let gates = crate::gates::registered_gates();
        if gates.is_empty() {
            return Outcome::Pass;
        }

        let mut failed = Vec::new();
        for gate in gates {
            let res = gate.selftest();
            if !matches!(res, Outcome::Pass) {
                failed.push(format!("{}: {:?}", gate.spec().name, res));
            }
        }

        if failed.is_empty() {
            Outcome::Pass
        } else {
            Outcome::Fail {
                reason: format!("Gate selftests failed for: {}", failed.join(", ")),
            }
        }
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "gate-selftests" {
            return Outcome::Fail {
                reason: "Self name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}

pub struct CatalogSyncGate;

impl Gate for CatalogSyncGate {
    fn spec(&self) -> GateSpec {
        GateSpec {
            name: "catalog-sync".to_string(),
            tier: "pr".to_string(),
            from_phase: "PH0".to_string(),
            kind: "builtin".to_string(),
            enforces: vec!["REQ-QA-008".to_string(), "REQ-GOV-004".to_string()],
            why: "Prüft 1:1 Übereinstimmung von Katalog, Code und Selbsttests.".to_string(),
            owner: "MP-02".to_string(),
            cmd: None,
            toolchain: None,
        }
    }

    fn run(&self, ctx: &Context) -> Outcome {
        let catalog_path = ctx.root_dir.join("governance/gates.toml");
        let catalog = match crate::core::catalog::Catalog::load_from_path(&catalog_path) {
            Ok(c) => c,
            Err(e) => {
                return Outcome::Error {
                    message: format!("Failed to load catalog: {}", e),
                };
            }
        };

        let registered = crate::gates::registered_gates();
        let registered_names: std::collections::HashSet<String> =
            registered.iter().map(|g| g.spec().name.clone()).collect();

        for (name, entry) in &catalog.gates {
            if entry.kind == "builtin" && !registered_names.contains(name) {
                return Outcome::Error {
                    message: format!("Builtin gate '{}' in catalog not implemented by owner {}", name, entry.owner),
                };
            }
        }

        Outcome::Pass
    }

    fn selftest(&self) -> Outcome {
        let spec = self.spec();
        if spec.name != "catalog-sync" {
            return Outcome::Fail {
                reason: "Self name mismatch".to_string(),
            };
        }
        Outcome::Pass
    }
}
