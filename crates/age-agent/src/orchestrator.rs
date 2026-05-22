use age_core::tool::context::ToolContext;
use age_core::tool::protocol::{ToolCall, ToolResult};
use age_core::tool::registry::ToolRegistry;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentTraceEntry {
    pub kind: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Default)]
pub struct Orchestrator {
    pub traces: Vec<AgentTraceEntry>,
}

impl Orchestrator {
    pub fn new() -> Self {
        Self { traces: vec![] }
    }

    pub async fn execute_batch(
        &mut self,
        registry: &ToolRegistry,
        ctx: ToolContext,
        calls: Vec<ToolCall>,
    ) -> Vec<ToolResult> {
        self.traces.push(AgentTraceEntry {
            kind: "batch_start".into(),
            payload: serde_json::json!({ "count": calls.len() }),
        });

        let mut results = Vec::with_capacity(calls.len());
        for call in calls {
            self.traces.push(AgentTraceEntry {
                kind: "tool_call".into(),
                payload: serde_json::to_value(&call).unwrap(),
            });
            let result = registry.execute(ctx.clone(), call).await;
            self.traces.push(AgentTraceEntry {
                kind: "tool_result".into(),
                payload: serde_json::to_value(&result).unwrap(),
            });
            results.push(result);
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use age_core::schema::ui::UiDocument;
    use age_core::schema::world3d::SceneDocument;
    use serde_json::json;

    #[tokio::test]
    async fn orchestrator_chains_two_mock_tools() {
        let mut registry = ToolRegistry::default();
        registry.register("increment", |_ctx, args| async move {
            let n = args.get("n").and_then(|v| v.as_i64()).unwrap_or(0);
            ToolResult::success("test", json!({ "n": n + 1 }))
        });

        let ctx = ToolContext::new(SceneDocument::default_empty(), UiDocument::default_empty());
        let mut orchestrator = Orchestrator::new();
        let results = orchestrator
            .execute_batch(
                &registry,
                ctx,
                vec![
                    ToolCall {
                        id: "1".into(),
                        tool: "increment".into(),
                        args: json!({ "n": 0 }),
                    },
                    ToolCall {
                        id: "2".into(),
                        tool: "increment".into(),
                        args: json!({ "n": 1 }),
                    },
                ],
            )
            .await;

        assert_eq!(results.len(), 2);
        assert!(results[0].ok);
        assert!(results[1].ok);
        assert_eq!(results[1].result.as_ref().unwrap()["n"], 2);
        assert!(orchestrator.traces.iter().any(|t| t.kind == "tool_call"));
    }
}
