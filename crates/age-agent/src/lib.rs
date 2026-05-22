pub mod checkpoint;
pub mod safety;
pub mod tools;

use age_core::tool::protocol::ToolResult;
use age_core::tool::registry::ToolRegistry;

pub mod orchestrator;

pub use checkpoint::{CheckpointError, CheckpointManager, CheckpointMeta};
pub use orchestrator::{AgentTraceEntry, Orchestrator};
pub use tools::{level, ui};

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

pub fn register_all_dev_tools(registry: &mut ToolRegistry) {
    tools::register_dev_tools(registry);
    register_stub_2d_tools(registry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use age_core::project::Project;
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

    #[tokio::test]
    async fn project_tool_chain_writes_scene() {
        let dir = tempfile::tempdir().unwrap();
        let project = Project::create_new(dir.path(), "demo_room").unwrap();
        let ctx = project.tool_context();

        let mut registry = ToolRegistry::default();
        register_all_dev_tools(&mut registry);

        let calls = vec![
            ToolCall {
                id: "1".into(),
                tool: "build_room".into(),
                args: json!({ "size": [8, 8], "_call_id": "1" }),
            },
            ToolCall {
                id: "2".into(),
                tool: "place_spawn".into(),
                args: json!({ "tag": "player", "position": [0, 0, 3.5], "_call_id": "2" }),
            },
            ToolCall {
                id: "3".into(),
                tool: "get_scene_summary".into(),
                args: json!({ "_call_id": "3" }),
            },
        ];

        let mut orchestrator = Orchestrator::new();
        let results = orchestrator.execute_batch(&registry, ctx.clone(), calls).await;
        assert!(results.iter().all(|r| r.ok));

        let node_count = ctx.scene.read().unwrap().scene.nodes.len();
        assert!(node_count >= 6);
    }
}
