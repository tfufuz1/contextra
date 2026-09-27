use std::fs;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    Ok(String),
    Missing,
    WrongVersion { found: String, required: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCheck {
    pub name: String,
    pub command: String,
    pub version_arg: String,
    pub required: bool,
    pub status: ToolStatus,
}

fn read_required_rustc_version() -> String {
    let root = crate::find_root_dir();
    let toolchain_path = root.join("rust-toolchain.toml");
    if toolchain_path.exists() {
        if let Ok(content) = fs::read_to_string(&toolchain_path) {
            if let Ok(value) = toml::from_str::<toml::Value>(&content) {
                if let Some(channel) = value
                    .get("toolchain")
                    .and_then(|t| t.get("channel"))
                    .and_then(|c| c.as_str())
                {
                    return channel.trim().to_string();
                }
            }
        }
    }
    "1.89.0".to_string()
}

fn parse_semver(s: &str) -> Option<(u32, u32, u32)> {
    // Finds first sequence like "1.89.0" or "1.89.0-nightly"
    for word in s.split_whitespace() {
        let clean = word.split('-').next().unwrap_or(word);
        let parts: Vec<&str> = clean.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse::<u32>().ok();
            let minor = parts[1].parse::<u32>().ok();
            let patch = if parts.len() >= 3 {
                parts[2].parse::<u32>().ok().unwrap_or(0)
            } else {
                0
            };
            if let (Some(maj), Some(min)) = (major, minor) {
                return Some((maj, min, patch));
            }
        }
    }
    None
}

pub fn run_env_validate() -> Vec<ToolCheck> {
    let required_rustc = read_required_rustc_version();

    let tools = [
        ("rustc", "rustc", "--version", true),
        ("cargo clippy", "cargo", "clippy --version", true),
        ("rustfmt", "rustfmt", "--version", true),
        ("flatc", "flatc", "--version", true),
        ("sg", "sg", "--version", false),
        ("cargo nextest", "cargo", "nextest --version", false),
        ("cargo audit", "cargo", "audit --version", false),
        ("cargo deny", "cargo", "deny --version", false),
        ("python3", "python3", "--version", false),
    ];

    let mut checks = Vec::new();

    for (name, command, version_arg, required) in tools {
        let args: Vec<&str> = version_arg.split_whitespace().collect();
        let output = Command::new(command).args(&args).output();

        let status = match output {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let first_line = stdout.lines().next().unwrap_or("").trim().to_string();

                if name == "rustc" {
                    if let (Some(found_ver), Some(req_ver)) =
                        (parse_semver(&first_line), parse_semver(&required_rustc))
                    {
                        if found_ver < req_ver {
                            ToolStatus::WrongVersion {
                                found: first_line.clone(),
                                required: required_rustc.clone(),
                            }
                        } else {
                            ToolStatus::Ok(first_line)
                        }
                    } else {
                        ToolStatus::Ok(first_line)
                    }
                } else {
                    ToolStatus::Ok(first_line)
                }
            }
            _ => ToolStatus::Missing,
        };

        checks.push(ToolCheck {
            name: name.to_string(),
            command: command.to_string(),
            version_arg: version_arg.to_string(),
            required,
            status,
        });
    }

    checks
}
