use super::*;
/// Helper to look up a key or dot-notation path in [`AgentContext`].
fn get_context_value<'a>(
    key: &str,
    ctx: &'a AgentContext,
) -> Option<std::borrow::Cow<'a, serde_json::Value>> {
    let key = key.trim();
    if key.is_empty() {
        return None;
    }

    // 1. Direct lookup in memory
    if let Some(v) = ctx.memory.get(key) {
        return Some(std::borrow::Cow::Borrowed(v));
    }

    // 2. Dot-notation path in memory (e.g. "output.status")
    if key.contains('.') {
        let parts: Vec<&str> = key.split('.').collect();
        if let Some(mut current) = ctx.memory.get(parts[0]) {
            let mut found = true;
            for part in &parts[1..] {
                if let serde_json::Value::Object(map) = current {
                    if let Some(next_val) = map.get(*part) {
                        current = next_val;
                    } else {
                        found = false;
                        break;
                    }
                } else {
                    found = false;
                    break;
                }
            }
            if found {
                return Some(std::borrow::Cow::Borrowed(current));
            }
        }
    }

    // 3. Built-in context properties
    match key {
        "task_id" => Some(std::borrow::Cow::Owned(serde_json::Value::String(
            ctx.task_id.clone(),
        ))),
        "current_node" => Some(std::borrow::Cow::Owned(serde_json::Value::String(
            ctx.current_node.clone(),
        ))),
        "step_count" => Some(std::borrow::Cow::Owned(serde_json::Value::Number(
            ctx.step_count.into(),
        ))),
        "status" => Some(std::borrow::Cow::Owned(serde_json::Value::String(format!(
            "{:?}",
            ctx.status
        )))),
        _ => None,
    }
}

/// Helper to check if a [`serde_json::Value`] matches a raw string representation value.
fn value_matches(val: &serde_json::Value, raw_val_str: &str) -> bool {
    let target = raw_val_str.trim().trim_matches('"').trim_matches('\'');
    match val {
        serde_json::Value::String(s) => s == target || s == raw_val_str.trim(),
        serde_json::Value::Bool(b) => {
            b.to_string() == target || b.to_string() == raw_val_str.trim()
        }
        serde_json::Value::Number(n) => {
            n.to_string() == target || n.to_string() == raw_val_str.trim()
        }
        serde_json::Value::Null => target == "null" || target == "Null" || target.is_empty(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(raw_val_str.trim()) {
                val == &parsed
            } else {
                false
            }
        }
    }
}

/// Evaluates a declarative condition expression against the provided [`AgentContext`].
///
/// # Supported Grammar:
/// - `<key> exists`: Returns `true` if `<key>` is present in context memory or context properties and is not `null`.
/// - `<key> == <value>`: Returns `true` if the value at `<key>` matches `<value>`.
/// - `<key> != <value>`: Returns `true` if the value at `<key>` does not match `<value>` (or if `<key>` does not exist).
///
/// Key resolution supports direct keys in `ctx.memory` (e.g. `"result"`), nested dot-notation paths
/// (e.g. `"output.status"`), and built-in context fields (`"task_id"`, `"current_node"`, `"step_count"`, `"status"`).
///
/// # Error Handling:
/// Expression syntax errors or invalid formats do NOT panic; they emit a [`tracing::warn!`] log and evaluate to `false`.
pub fn evaluate_condition_expr(expr: &str, ctx: &AgentContext) -> bool {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        tracing::warn!("Empty condition expression evaluated as false");
        return false;
    }

    if let Some(key_part) = trimmed.strip_suffix(" exists") {
        let key = key_part.trim();
        if key.is_empty() {
            tracing::warn!(
                "Condition expression missing key before 'exists': '{}'",
                expr
            );
            return false;
        }
        if let Some(v) = get_context_value(key, ctx) {
            return !v.is_null();
        }
        return false;
    }

    if let Some((key_part, val_part)) = trimmed.split_once("!=") {
        let key = key_part.trim();
        let val = val_part.trim();
        if key.is_empty() {
            tracing::warn!("Condition expression missing key before '!=': '{}'", expr);
            return false;
        }
        if let Some(v) = get_context_value(key, ctx) {
            return !value_matches(&v, val);
        }
        // Missing key does not match value, so != holds true
        return true;
    }

    if let Some((key_part, val_part)) = trimmed.split_once("==") {
        let key = key_part.trim();
        let val = val_part.trim();
        if key.is_empty() {
            tracing::warn!("Condition expression missing key before '==': '{}'", expr);
            return false;
        }
        if let Some(v) = get_context_value(key, ctx) {
            return value_matches(&v, val);
        }
        return false;
    }

    tracing::warn!("Unparseable condition expression: '{}'", expr);
    false
}
