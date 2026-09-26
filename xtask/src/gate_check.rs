use std::process::Command;

/// Gate level for phase orchestrator checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateLevel {
    Zero,
    One,
    Two,
    Three,
    Four,
}

impl GateLevel {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "0" | "zero" => Ok(GateLevel::Zero),
            "1" | "one" => Ok(GateLevel::One),
            "2" | "two" => Ok(GateLevel::Two),
            "3" | "three" => Ok(GateLevel::Three),
            "4" | "four" => Ok(GateLevel::Four),
            _ => Err(format!(
                "Unbekanntes Gate-Level: '{}'. Gültige Werte: 0..4 (oder Zero..Four)",
                s
            )),
        }
    }

    pub fn number(&self) -> u8 {
        match self {
            GateLevel::Zero => 0,
            GateLevel::One => 1,
            GateLevel::Two => 2,
            GateLevel::Three => 3,
            GateLevel::Four => 4,
        }
    }
}

pub struct GateCheckOptions {
    pub level: GateLevel,
    pub crate_name: Option<String>, // Mandatory for Level Zero, ignored for higher levels
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateCheckReport {
    pub level: GateLevel,
    pub crate_name: Option<String>,
    pub passed_steps: Vec<String>,
    pub todo_notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateCheckError {
    MissingCrateName,
    StepFailed {
        step_name: String,
        command: String,
        exit_code: Option<i32>,
        stderr: String,
    },
    NotImplemented {
        level: GateLevel,
        message: String,
    },
    IoError(String),
}

impl std::fmt::Display for GateCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GateCheckError::MissingCrateName => write!(
                f,
                "Fehler: --crate <name> ist für Gate Level 0 erforderlich."
            ),
            GateCheckError::StepFailed {
                step_name,
                command,
                exit_code,
                stderr,
            } => write!(
                f,
                "Schritt '{}' (Befehl: '{}') fehlgeschlagen mit Exit-Code {:?}.\nStderr:\n{}",
                step_name, command, exit_code, stderr
            ),
            GateCheckError::NotImplemented { level: _, message } => write!(f, "{}", message),
            GateCheckError::IoError(msg) => write!(f, "I/O-Fehler: {}", msg),
        }
    }
}

impl std::error::Error for GateCheckError {}

/// Ring 0 and Ring 1 crates as defined in docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md §A.3 & Ring Table.
/// Source of Truth: docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md Teil A.3, capabilities.toml, and docs/ARCHITECTURE.md.
pub const RING_0_1_CRATES: &[&str] = &[
    // Ring 0: Core Domain & Storage Kernels
    "contextra-types",
    "contextra-ports",
    "contextra-vector",
    "contextra-text",
    "contextra-graph",
    "contextra-rank",
    "contextra-adapt",
    "contextra-simd",
    // Ring 1: Persistence & Cryptographic Isolation
    "contextra-store",
    "contextra-mvcc",
    "contextra-checkpoint",
    "contextra-kvcache",
    "contextra-crypto",
    "contextra-privacy",
    "contextra-sys",
    "contextra-wire",
];

pub fn run(opts: GateCheckOptions) -> Result<GateCheckReport, GateCheckError> {
    match opts.level {
        GateLevel::Zero => run_gate_zero(opts),
        GateLevel::One => run_gate_one(opts),
        GateLevel::Two | GateLevel::Three | GateLevel::Four => {
            let n = opts.level.number();
            let prev_phase = n - 1;
            let message = format!(
                "Gate {} ist spezifiziert, aber die zugehörigen Abnahmekriterien aus Phase {} sind in dieser xtask-Version noch nicht als automatisierte Prüfung hinterlegt — siehe docs/spec/CONTEXTRA_FINALE_PRODUKTSPEZIFIKATION.md Teil A.3",
                n, prev_phase
            );
            Err(GateCheckError::NotImplemented {
                level: opts.level,
                message,
            })
        }
    }
}

fn execute_cmd(
    step_name: &str,
    program: &str,
    args: &[&str],
    envs: &[(&str, &str)],
) -> Result<(), GateCheckError> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }

    let cmd_str = format!("{} {}", program, args.join(" "));
    let output = cmd.output().map_err(|e| GateCheckError::IoError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let combined = if stderr.is_empty() {
            stdout
        } else {
            format!("{}\n{}", stdout, stderr)
        };

        return Err(GateCheckError::StepFailed {
            step_name: step_name.to_string(),
            command: cmd_str,
            exit_code: output.status.code(),
            stderr: combined,
        });
    }

    Ok(())
}

fn run_gate_zero_for_crate(
    crate_name: &str,
    report: &mut GateCheckReport,
) -> Result<(), GateCheckError> {
    // 1. cargo fmt --check -p <name>
    let step1_name = format!("Gate 0: cargo fmt ({})", crate_name);
    execute_cmd(
        &step1_name,
        "cargo",
        &["fmt", "--check", "-p", crate_name],
        &[],
    )?;
    report.passed_steps.push(step1_name);

    // 2. cargo clippy -p <name> --all-targets --locked -- -D warnings
    let step2_name = format!("Gate 0: cargo clippy ({})", crate_name);
    execute_cmd(
        &step2_name,
        "cargo",
        &[
            "clippy",
            "-p",
            crate_name,
            "--all-targets",
            "--locked",
            "--",
            "-D",
            "warnings",
        ],
        &[],
    )?;
    report.passed_steps.push(step2_name);

    // 3. cargo test -p <name> --locked
    let step3_name = format!("Gate 0: cargo test ({})", crate_name);
    execute_cmd(
        &step3_name,
        "cargo",
        &["test", "-p", crate_name, "--locked"],
        &[],
    )?;
    report.passed_steps.push(step3_name);

    // Marker TODO note
    report.todo_notes.push(format!(
        "TODO (P7): Marker des Crates '{}' neu erzeugen (P7 Marker-Generator)",
        crate_name
    ));

    Ok(())
}

fn run_gate_zero(opts: GateCheckOptions) -> Result<GateCheckReport, GateCheckError> {
    let crate_name = opts
        .crate_name
        .clone()
        .ok_or(GateCheckError::MissingCrateName)?;

    let mut report = GateCheckReport {
        level: GateLevel::Zero,
        crate_name: Some(crate_name.clone()),
        passed_steps: Vec::new(),
        todo_notes: Vec::new(),
    };

    run_gate_zero_for_crate(&crate_name, &mut report)?;
    Ok(report)
}

fn run_gate_one(opts: GateCheckOptions) -> Result<GateCheckReport, GateCheckError> {
    let mut report = GateCheckReport {
        level: GateLevel::One,
        crate_name: opts.crate_name,
        passed_steps: Vec::new(),
        todo_notes: Vec::new(),
    };

    // 1. Gate 0 for ALL Ring 0/1 crates
    for krate in RING_0_1_CRATES {
        run_gate_zero_for_crate(krate, &mut report)?;
    }

    // 2. Loom tests: cargo test --workspace --features loom -- --test-threads=1 with RUSTFLAGS="--cfg loom"
    let loom_step = "Gate 1: Loom tests".to_string();
    execute_cmd(
        &loom_step,
        "cargo",
        &["test", "--workspace", "--features", "loom", "--", "--test-threads=1"],
        &[("RUSTFLAGS", "--cfg loom")],
    )?;
    report.passed_steps.push(loom_step);

    // 3. Layering tests: cargo test --manifest-path xtask/Cargo.toml --test layering
    let layering_step = "Gate 1: Layering tests".to_string();
    execute_cmd(
        &layering_step,
        "cargo",
        &["test", "--manifest-path", "xtask/Cargo.toml", "--test", "layering"],
        &[],
    )?;
    report.passed_steps.push(layering_step);

    // 4. Ring layering full check: cargo run --manifest-path xtask/Cargo.toml -- check-ring-layering-full
    let ring_full_step = "Gate 1: Ring layering full check".to_string();
    execute_cmd(
        &ring_full_step,
        "cargo",
        &["run", "--manifest-path", "xtask/Cargo.toml", "--", "check-ring-layering-full"],
        &[],
    )?;
    report.passed_steps.push(ring_full_step);

    // 5. Duplicate core primitives check: cargo run --manifest-path xtask/Cargo.toml -- check-duplicate-core-primitives
    let dup_primitives_step = "Gate 1: Duplicate core primitives check".to_string();
    execute_cmd(
        &dup_primitives_step,
        "cargo",
        &["run", "--manifest-path", "xtask/Cargo.toml", "--", "check-duplicate-core-primitives"],
        &[],
    )?;
    report.passed_steps.push(dup_primitives_step);

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_level_from_str() {
        assert_eq!(GateLevel::from_str("0").unwrap(), GateLevel::Zero);
        assert_eq!(GateLevel::from_str("zero").unwrap(), GateLevel::Zero);
        assert_eq!(GateLevel::from_str("1").unwrap(), GateLevel::One);
        assert_eq!(GateLevel::from_str("one").unwrap(), GateLevel::One);
        assert_eq!(GateLevel::from_str("2").unwrap(), GateLevel::Two);
        assert_eq!(GateLevel::from_str("two").unwrap(), GateLevel::Two);
        assert_eq!(GateLevel::from_str("3").unwrap(), GateLevel::Three);
        assert_eq!(GateLevel::from_str("three").unwrap(), GateLevel::Three);
        assert_eq!(GateLevel::from_str("4").unwrap(), GateLevel::Four);
        assert_eq!(GateLevel::from_str("four").unwrap(), GateLevel::Four);
        assert!(GateLevel::from_str("5").is_err());
    }

    #[test]
    fn test_gate_check_level_zero_missing_crate() {
        let opts = GateCheckOptions {
            level: GateLevel::Zero,
            crate_name: None,
        };
        let res = run(opts);
        assert!(matches!(res, Err(GateCheckError::MissingCrateName)));
    }

    #[test]
    fn test_gate_check_level_2_not_implemented() {
        let opts = GateCheckOptions {
            level: GateLevel::Two,
            crate_name: None,
        };
        let res = run(opts);
        match res {
            Err(GateCheckError::NotImplemented { level, message }) => {
                assert_eq!(level, GateLevel::Two);
                assert!(message.contains("Gate 2 ist spezifiziert, aber die zugehörigen Abnahmekriterien aus Phase 1 sind in dieser xtask-Version noch nicht als automatisierte Prüfung hinterlegt"));
            }
            _ => panic!("Expected NotImplemented error"),
        }
    }
}
