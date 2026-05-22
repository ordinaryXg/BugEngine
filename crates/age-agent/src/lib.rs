pub mod tools;

use age_core::tool::protocol::ToolResult;
use age_core::tool::registry::ToolRegistry;

pub mod orchestrator;

pub use orchestrator::{AgentTraceEntry, Orchestrator};

pub fn register_stub_2d_tools(registry: &mut ToolRegistry) {
    for name in ["paint_tiles", "create_tilemap_layer", "import_tileset"] {
        let tool_name = name.to_string();
        registry.register(name, move |_ctx, _args| {
            let tool_name = tool_name.clone();
            async move {
                ToolResult::failure(
                    "stub",
                    "NOT_IMPLEMENTED",
                    format!("tool '{tool_name}' is planned for phase 2"),
                )
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use age_core::tool::context::ToolContext;
    use age_core::tool::protocol::ToolCall;
    use serde_json::json;

    #[tokio::test]
    async fn stub_2d_tools_return_not_implemented() {
        let mut registry = ToolRegistry::default();
        register_stub_2d_tools(&mut registry);
        let ctx = ToolContext::default();
        let result = registry
            .execute(
                ctx,
                ToolCall {
                    id: "1".into(),
                    tool: "paint_tiles".into(),
                    args: json!({}),
                },
            )
            .await;
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "NOT_IMPLEMENTED");
    }
}
