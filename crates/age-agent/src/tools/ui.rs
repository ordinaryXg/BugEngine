use age_core::schema::ui::{UiDocument, UiScreen, UiWidget};
use age_core::tool::context::ToolContext;
use age_core::tool::protocol::ToolResult;
use age_core::tool::registry::ToolRegistry;
use serde_json::{json, Value};

use crate::safety::needs_confirmation;

fn call_id(args: &Value) -> String {
    args.get("_call_id")
        .and_then(|v| v.as_str())
        .unwrap_or("call")
        .to_string()
}

fn find_screen_mut<'a>(ui: &'a mut UiDocument, screen_id: &str) -> Option<&'a mut UiScreen> {
    ui.screens.iter_mut().find(|s| s.id == screen_id)
}

fn find_widget_mut<'a>(widget: &'a mut UiWidget, widget_id: &str) -> Option<&'a mut UiWidget> {
    if widget.id.as_deref() == Some(widget_id) {
        return Some(widget);
    }
    for child in &mut widget.children {
        if let Some(found) = find_widget_mut(child, widget_id) {
            return Some(found);
        }
    }
    None
}

fn remove_widget(root: &mut UiWidget, widget_id: &str) -> bool {
    if let Some(pos) = root
        .children
        .iter()
        .position(|c| c.id.as_deref() == Some(widget_id))
    {
        root.children.remove(pos);
        return true;
    }
    for child in &mut root.children {
        if remove_widget(child, widget_id) {
            return true;
        }
    }
    false
}

async fn create_screen(ctx: ToolContext, args: Value) -> ToolResult {
    let screen_id = args
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("game_hud")
        .to_string();
    let layer = args
        .get("layer")
        .and_then(|v| v.as_str())
        .unwrap_or("overlay")
        .to_string();
    let mut ui = ctx.ui.write().unwrap();
    if ui.screens.iter().any(|s| s.id == screen_id) {
        return ToolResult::failure(
            call_id(&args),
            "ALREADY_EXISTS",
            format!("screen '{screen_id}' already exists"),
        );
    }
    ui.screens.push(UiScreen {
        id: screen_id.clone(),
        layer,
        root: UiWidget {
            widget_type: "Canvas".into(),
            id: Some(UiDocument::new_widget_id("canvas")),
            layout: Some(json!({ "anchor": "full" })),
            props: None,
            bind: None,
            events: None,
            children: vec![],
        },
    });
    ToolResult::success(call_id(&args), json!({ "screen_id": screen_id }))
}

async fn delete_screen(ctx: ToolContext, args: Value) -> ToolResult {
    if let Err(e) = needs_confirmation(&args, "delete_screen") {
        return ToolResult::failure(call_id(&args), e.code(), e.to_string());
    }
    let screen_id = match args.get("screen_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return ToolResult::failure(call_id(&args), "INVALID_ARGS", "screen_id is required")
        }
    };
    let mut ui = ctx.ui.write().unwrap();
    let len_before = ui.screens.len();
    ui.screens.retain(|s| s.id != screen_id);
    if ui.screens.len() == len_before {
        return ToolResult::failure(call_id(&args), "SCREEN_NOT_FOUND", screen_id);
    }
    ToolResult::success(call_id(&args), json!({ "deleted": screen_id }))
}

async fn add_widget(ctx: ToolContext, args: Value) -> ToolResult {
    let screen_id = args
        .get("screen_id")
        .and_then(|v| v.as_str())
        .unwrap_or("game_hud")
        .to_string();
    let widget_type = args
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Panel")
        .to_string();
    let widget_id = args
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| UiDocument::new_widget_id("widget"));
    let parent_id = args.get("parent_id").and_then(|v| v.as_str());

    let widget = UiWidget {
        widget_type,
        id: Some(widget_id.clone()),
        layout: args.get("layout").cloned(),
        props: args.get("props").cloned(),
        bind: None,
        events: None,
        children: vec![],
    };

    let mut ui = ctx.ui.write().unwrap();
    let Some(screen) = find_screen_mut(&mut ui, &screen_id) else {
        return ToolResult::failure(call_id(&args), "SCREEN_NOT_FOUND", screen_id);
    };

    if let Some(parent_id) = parent_id {
        let Some(parent) = find_widget_mut(&mut screen.root, parent_id) else {
            return ToolResult::failure(call_id(&args), "WIDGET_NOT_FOUND", parent_id.to_string());
        };
        parent.children.push(widget);
    } else {
        screen.root.children.push(widget);
    }

    ToolResult::success(
        call_id(&args),
        json!({ "screen_id": screen_id, "widget_id": widget_id }),
    )
}

async fn remove_widget_tool(ctx: ToolContext, args: Value) -> ToolResult {
    let screen_id = args
        .get("screen_id")
        .and_then(|v| v.as_str())
        .unwrap_or("game_hud")
        .to_string();
    let widget_id = match args.get("widget_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return ToolResult::failure(call_id(&args), "INVALID_ARGS", "widget_id is required")
        }
    };
    let mut ui = ctx.ui.write().unwrap();
    let Some(screen) = find_screen_mut(&mut ui, &screen_id) else {
        return ToolResult::failure(call_id(&args), "SCREEN_NOT_FOUND", screen_id);
    };
    if !remove_widget(&mut screen.root, &widget_id) {
        return ToolResult::failure(call_id(&args), "WIDGET_NOT_FOUND", widget_id);
    }
    ToolResult::success(call_id(&args), json!({ "removed": widget_id }))
}

async fn set_layout(ctx: ToolContext, args: Value) -> ToolResult {
    update_widget_field(ctx, &args, |widget, args| {
        widget.layout = args.get("layout").cloned();
    })
    .await
}

async fn set_style(ctx: ToolContext, args: Value) -> ToolResult {
    update_widget_field(ctx, &args, |widget, args| {
        let mut props = widget.props.clone().unwrap_or(json!({}));
        if let Some(style) = args.get("style") {
            if let Some(obj) = props.as_object_mut() {
                if let Some(style_obj) = style.as_object() {
                    for (k, v) in style_obj {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        widget.props = Some(props);
    })
    .await
}

async fn bind_property(ctx: ToolContext, args: Value) -> ToolResult {
    update_widget_field(ctx, &args, |widget, args| {
        widget.bind = args.get("bind").cloned();
    })
    .await
}

async fn bind_event(ctx: ToolContext, args: Value) -> ToolResult {
    update_widget_field(ctx, &args, |widget, args| {
        let event = args
            .get("event")
            .and_then(|v| v.as_str())
            .unwrap_or("on_click");
        let handler = args.get("handler").cloned().unwrap_or(json!({}));
        let mut events = widget.events.clone().unwrap_or(json!({}));
        if let Some(obj) = events.as_object_mut() {
            obj.insert(event.to_string(), handler);
        }
        widget.events = Some(events);
    })
    .await
}

async fn preview_resolution(_ctx: ToolContext, args: Value) -> ToolResult {
    let width = args.get("width").and_then(|v| v.as_u64()).unwrap_or(1280);
    let height = args.get("height").and_then(|v| v.as_u64()).unwrap_or(720);
    ToolResult::success(
        call_id(&args),
        json!({ "width": width, "height": height, "preview": true }),
    )
}

async fn set_visibility(ctx: ToolContext, args: Value) -> ToolResult {
    update_widget_field(ctx, &args, |widget, args| {
        let visible = args.get("visible").and_then(|v| v.as_bool()).unwrap_or(true);
        let mut props = widget.props.clone().unwrap_or(json!({}));
        if let Some(obj) = props.as_object_mut() {
            obj.insert("visible".into(), json!(visible));
        }
        widget.props = Some(props);
    })
    .await
}

async fn set_text(ctx: ToolContext, args: Value) -> ToolResult {
    update_widget_field(ctx, &args, |widget, args| {
        let text = args
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mut props = widget.props.clone().unwrap_or(json!({}));
        if let Some(obj) = props.as_object_mut() {
            obj.insert("text".into(), json!(text));
        }
        widget.props = Some(props);
    })
    .await
}

async fn query_ui_tree(ctx: ToolContext, args: Value) -> ToolResult {
    let ui = ctx.ui.read().unwrap();
    let screen_id = args.get("screen_id").and_then(|v| v.as_str());
    let screens: Vec<Value> = ui
        .screens
        .iter()
        .filter(|s| screen_id.is_none_or(|id| id == s.id))
        .map(|s| {
            json!({
                "id": s.id,
                "layer": s.layer,
                "child_count": s.root.children.len(),
            })
        })
        .collect();
    ToolResult::success(
        call_id(&args),
        json!({ "screens": screens, "count": screens.len() }),
    )
}

async fn update_widget_field<F>(ctx: ToolContext, args: &Value, mut f: F) -> ToolResult
where
    F: FnMut(&mut UiWidget, &Value),
{
    let screen_id = args
        .get("screen_id")
        .and_then(|v| v.as_str())
        .unwrap_or("game_hud")
        .to_string();
    let widget_id = match args.get("widget_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return ToolResult::failure(call_id(args), "INVALID_ARGS", "widget_id is required")
        }
    };
    let mut ui = ctx.ui.write().unwrap();
    let Some(screen) = find_screen_mut(&mut ui, &screen_id) else {
        return ToolResult::failure(call_id(args), "SCREEN_NOT_FOUND", screen_id);
    };
    let Some(widget) = find_widget_mut(&mut screen.root, &widget_id) else {
        return ToolResult::failure(call_id(args), "WIDGET_NOT_FOUND", widget_id);
    };
    f(widget, args);
    ToolResult::success(
        call_id(args),
        json!({ "screen_id": screen_id, "widget_id": widget_id }),
    )
}

pub fn register_ui_tools(registry: &mut ToolRegistry) {
    registry.register("create_screen", |ctx, args| async move {
        create_screen(ctx, args).await
    });
    registry.register("delete_screen", |ctx, args| async move {
        delete_screen(ctx, args).await
    });
    registry.register("add_widget", |ctx, args| async move { add_widget(ctx, args).await });
    registry.register("remove_widget", |ctx, args| async move {
        remove_widget_tool(ctx, args).await
    });
    registry.register("set_layout", |ctx, args| async move { set_layout(ctx, args).await });
    registry.register("set_style", |ctx, args| async move { set_style(ctx, args).await });
    registry.register("bind_property", |ctx, args| async move {
        bind_property(ctx, args).await
    });
    registry.register("bind_event", |ctx, args| async move { bind_event(ctx, args).await });
    registry.register("preview_resolution", |ctx, args| async move {
        preview_resolution(ctx, args).await
    });
    registry.register("set_visibility", |ctx, args| async move {
        set_visibility(ctx, args).await
    });
    registry.register("set_text", |ctx, args| async move { set_text(ctx, args).await });
    registry.register("query_ui_tree", |ctx, args| async move {
        query_ui_tree(ctx, args).await
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use age_core::schema::world3d::SceneDocument;
    use age_core::tool::protocol::ToolCall;

    fn test_registry() -> ToolRegistry {
        let mut registry = ToolRegistry::default();
        register_ui_tools(&mut registry);
        registry
    }

    #[tokio::test]
    async fn create_screen_add_widget_and_bind() {
        let registry = test_registry();
        let ctx = ToolContext::new(SceneDocument::default_empty(), UiDocument::default_empty());

        registry
            .execute(
                ctx.clone(),
                ToolCall {
                    id: "1".into(),
                    tool: "create_screen".into(),
                    args: json!({ "id": "game_hud", "_call_id": "1" }),
                },
            )
            .await;

        let add = registry
            .execute(
                ctx.clone(),
                ToolCall {
                    id: "2".into(),
                    tool: "add_widget".into(),
                    args: json!({
                        "screen_id": "game_hud",
                        "type": "ProgressBar",
                        "id": "health_bar",
                        "_call_id": "2"
                    }),
                },
            )
            .await;
        assert!(add.ok);

        let bind = registry
            .execute(
                ctx,
                ToolCall {
                    id: "3".into(),
                    tool: "bind_property".into(),
                    args: json!({
                        "screen_id": "game_hud",
                        "widget_id": "health_bar",
                        "bind": { "value": "player.health", "max": "player.max_health" },
                        "_call_id": "3"
                    }),
                },
            )
            .await;
        assert!(bind.ok);
    }
}
