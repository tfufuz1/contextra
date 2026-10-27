use super::common::not_implemented;

pub fn run_tags_command(cmd: &str, _args: &[String]) -> Result<(), String> {
    match cmd {
        "validate-tags"
        | "check-stale-tags"
        | "tag-health"
        | "context-tags"
        | "check-review-coverage" => not_implemented(cmd),
        _ => Err(format!("Unknown command in tags cluster: {}", cmd)),
    }
}
