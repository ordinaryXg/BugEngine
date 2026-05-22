use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub tool: String,
    pub args: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolResult {
    pub call_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<String>,
}

impl ToolResult {
    pub fn success(call_id: impl Into<String>, result: Value) -> Self {
        Self {
            call_id: call_id.into(),
            ok: true,
            result: Some(result),
            error: None,
            checkpoint_id: None,
        }
    }

    pub fn failure(
        call_id: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            call_id: call_id.into(),
            ok: false,
            result: None,
            error: Some(ToolError {
                code: code.into(),
                message: message.into(),
            }),
            checkpoint_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn tool_call_roundtrip_json() {
        let call = ToolCall {
            id: "call_001".into(),
            tool: "place_primitive".into(),
            args: json!({
                "shape": "builtin://cube",
                "transform": { "position": [2, 0.5, 1], "rotation": [0, 0, 0], "scale": [1, 1, 1] }
            }),
        };
        let json = serde_json::to_string(&call).unwrap();
        let parsed: ToolCall = serde_json::from_str(&json).unwrap();
        assert_eq!(call, parsed);
    }

    #[test]
    fn tool_result_success_and_failure() {
        let ok = ToolResult::success("c1", json!({ "node_id": "node_abc" }));
        assert!(ok.ok);
        assert_eq!(ok.result.unwrap()["node_id"], "node_abc");

        let err = ToolResult::failure("c2", "NODE_NOT_FOUND", "missing node");
        assert!(!err.ok);
        assert_eq!(err.error.unwrap().code, "NODE_NOT_FOUND");
    }
}
