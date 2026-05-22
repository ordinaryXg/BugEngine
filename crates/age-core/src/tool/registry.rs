use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde_json::Value;

use super::context::ToolContext;
use super::protocol::{ToolCall, ToolResult};

pub type ToolHandler = Arc<
    dyn Fn(ToolContext, Value) -> Pin<Box<dyn Future<Output = ToolResult> + Send>> + Send + Sync,
>;

#[derive(Default)]
pub struct ToolRegistry {
    handlers: HashMap<String, ToolHandler>,
}

impl ToolRegistry {
    pub fn register<F, Fut>(&mut self, name: impl Into<String>, handler: F)
    where
        F: Fn(ToolContext, Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ToolResult> + Send + 'static,
    {
        let name = name.into();
        let handler: ToolHandler = Arc::new(move |ctx, args| Box::pin(handler(ctx, args)));
        self.handlers.insert(name, handler);
    }

    pub fn has(&self, name: &str) -> bool {
        self.handlers.contains_key(name)
    }

    pub async fn execute(&self, ctx: ToolContext, call: ToolCall) -> ToolResult {
        let Some(handler) = self.handlers.get(&call.tool) else {
            return ToolResult::failure(
                call.id,
                "UNKNOWN_TOOL",
                format!("tool '{}' is not registered", call.tool),
            );
        };
        handler(ctx, call.args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::ui::UiDocument;
    use crate::schema::world3d::SceneDocument;
    use serde_json::json;

    #[tokio::test]
    async fn registry_executes_registered_tool() {
        let mut registry = ToolRegistry::default();
        registry.register("echo", |_ctx, args| async move { ToolResult::success("test", args) });

        let ctx = ToolContext::new(SceneDocument::default_empty(), UiDocument::default_empty());
        let result = registry
            .execute(
                ctx,
                ToolCall {
                    id: "test".into(),
                    tool: "echo".into(),
                    args: json!({ "hello": "world" }),
                },
            )
            .await;

        assert!(result.ok);
        assert_eq!(result.result.unwrap()["hello"], "world");
    }

    #[tokio::test]
    async fn registry_returns_error_for_unknown_tool() {
        let registry = ToolRegistry::default();
        let ctx = ToolContext::new(SceneDocument::default_empty(), UiDocument::default_empty());
        let result = registry
            .execute(
                ctx,
                ToolCall {
                    id: "x".into(),
                    tool: "missing".into(),
                    args: json!({}),
                },
            )
            .await;
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "UNKNOWN_TOOL");
    }
}
