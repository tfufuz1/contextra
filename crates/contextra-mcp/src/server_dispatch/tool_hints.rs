use crate::sandbox::ToolCategory;
use serde_json::Value;

pub fn annotate_tool_schema(mut schema: Value, category: ToolCategory) -> Value {
    if let Some(obj) = schema.as_object_mut() {
        let (read_only, destructive) = match category {
            ToolCategory::DatabaseRead => (true, false),
            ToolCategory::DatabaseWrite => (false, true),
            ToolCategory::CloudEgress | ToolCategory::CodeExecution => (false, false),
        };

        obj.insert("readOnlyHint".into(), Value::Bool(read_only));
        obj.insert("destructiveHint".into(), Value::Bool(destructive));
    }
    schema
}
