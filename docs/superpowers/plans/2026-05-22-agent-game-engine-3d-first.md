# AgentGameEngine 3D-First Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 构建 AI Agent 深度介入的 3D 全栈游戏引擎 MVP：Agent 通过 Tool 编辑 World3D/UI Schema，HTML 导出后在浏览器以第三人称探索可玩 Demo。

**Architecture:** Rust Cargo workspace 承载 Engine Core、Agent Orchestrator、Web Runtime（WASM/wgpu）与 Export Pipeline；Tauri 2 + React 提供桌面 IDE，WebView 嵌入同一 HTML Runtime 作预览。Schema 为唯一真相来源，所有编辑经 Tool 协议写入。

**Tech Stack:** Rust 2021, Tauri 2, React 18 + TypeScript, wgpu, wasm-bindgen, serde/json, rusqlite, vitest (IDE UI), cargo test (Rust)

**Spec:** [`docs/superpowers/specs/2026-05-22-agent-game-engine-3d-first-design.md`](../specs/2026-05-22-agent-game-engine-3d-first-design.md)

---

## Monorepo 文件结构

```
AgentGameEngine/
├── Cargo.toml                          # workspace root
├── crates/
│   ├── age-core/                       # Schema、Tool 协议、Project 模型
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── schema/
│   │       │   ├── mod.rs
│   │       │   ├── world3d.rs          # World3D Scene 类型
│   │       │   ├── ui.rs               # UI Document 类型
│   │       │   ├── transform.rs
│   │       │   └── validate.rs         # Schema 校验
│   │       ├── project/
│   │       │   ├── mod.rs
│   │       │   └── io.rs               # 读写 project.json / scenes / ui
│   │       └── tool/
│   │           ├── mod.rs
│   │           ├── protocol.rs         # ToolCall / ToolResult 类型
│   │           ├── registry.rs         # Tool 注册表
│   │           └── context.rs          # Tool 执行上下文
│   ├── age-agent/                      # Agent Orchestrator
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── orchestrator.rs
│   │       ├── planner.rs              # MVP: 规则 + LLM 接口 trait
│   │       ├── safety.rs
│   │       ├── checkpoint.rs
│   │       └── tools/                  # Dev Tool 实现
│   │           ├── mod.rs
│   │           ├── level.rs            # 18 个地编 tools
│   │           └── ui.rs               # 12 个 UI tools
│   ├── age-runtime/                    # Web Runtime (WASM)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── scene_loader.rs
│   │       ├── renderer3d.rs
│   │       ├── ui_renderer.rs
│   │       ├── input.rs
│   │       ├── physics_simple.rs
│   │       ├── player_controller.rs    # 第三人称
│   │       ├── interaction.rs
│   │       ├── game_loop.rs
│   │       └── runtime_agent.rs        # AgentBrain 状态机
│   └── age-export/                     # HTML Export Pipeline
│       └── src/
│           ├── lib.rs
│           ├── html.rs
│           └── targets.rs              # html/desktop/mobile ExportTarget
├── apps/
│   └── age-ide/                        # Tauri + React IDE
│       ├── src-tauri/
│       │   ├── Cargo.toml
│       │   └── src/
│       │       ├── main.rs
│       │       └── commands.rs         # Tauri IPC → age-agent / age-export
│       └── src/                        # React frontend
│           ├── main.tsx
│           ├── App.tsx
│           ├── components/
│           │   ├── AgentPanel.tsx
│           │   ├── Viewport.tsx
│           │   ├── ProjectTree.tsx
│           │   ├── ExportMenu.tsx
│           │   └── CheckpointList.tsx
│           └── lib/
│               └── tauri-api.ts
├── runtime-web/                        # WASM 引导页（导出产物模板）
│   ├── index.html
│   └── bootstrap.js
├── templates/
│   └── default-project/                # 新建项目模板
│       ├── project.json
│       ├── scenes/main.scene.json
│       └── ui/hud.ui.json
└── docs/superpowers/
    ├── specs/...
    └── plans/...
```

**职责边界：**
- `age-core`：纯数据 + Tool 协议，无 LLM、无渲染
- `age-agent`：依赖 age-core，实现 Tool 与 Orchestrator
- `age-runtime`：依赖 age-core 类型，WASM 渲染与玩法
- `age-export`：依赖 age-core，打包 HTML
- `age-ide`：UI 壳，通过 Tauri command 调用 Rust crate

---

## Phase P0: Agent Core + Tool Protocol（2 周）

**交付验收：** `cargo test -p age-core -p age-agent` 全绿；Orchestrator 能链式调用 mock tool 并写 trace。

---

### Task 0: 初始化 Cargo Workspace

**Files:**
- Create: `Cargo.toml`
- Create: `crates/age-core/Cargo.toml`
- Create: `crates/age-core/src/lib.rs`
- Create: `crates/age-agent/Cargo.toml`
- Create: `crates/age-agent/src/lib.rs`

- [ ] **Step 1: 创建 workspace 根 Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/age-core",
    "crates/age-agent",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
```

- [ ] **Step 2: 创建 age-core crate**

`crates/age-core/Cargo.toml`:
```toml
[package]
name = "age-core"
version.workspace = true
edition.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
uuid.workspace = true
```

`crates/age-core/src/lib.rs`:
```rust
pub mod schema;
pub mod project;
pub mod tool;
```

- [ ] **Step 3: 创建 age-agent crate**

`crates/age-agent/Cargo.toml`:
```toml
[package]
name = "age-agent"
version.workspace = true
edition.workspace = true

[dependencies]
age-core = { path = "../age-core" }
serde.workspace = true
serde_json.workspace = true
thiserror.workspace = true
uuid.workspace = true
```

- [ ] **Step 4: 验证 workspace 编译**

Run: `cargo check`
Expected: `Finished dev profile`

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/
git commit -m "chore: initialize cargo workspace with age-core and age-agent"
```

---

### Task 1: Tool 协议类型

**Files:**
- Create: `crates/age-core/src/tool/mod.rs`
- Create: `crates/age-core/src/tool/protocol.rs`
- Create: `crates/age-core/src/tool/protocol_test.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: 写失败测试**

`crates/age-core/src/tool/protocol.rs`:
```rust
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

    pub fn failure(call_id: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
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
```

- [ ] **Step 2: 创建 tool/mod.rs 导出**

`crates/age-core/src/tool/mod.rs`:
```rust
pub mod protocol;
pub mod registry;
pub mod context;

pub use protocol::{ToolCall, ToolError, ToolResult};
```

- [ ] **Step 3: 运行测试**

Run: `cargo test -p age-core tool::protocol`
Expected: 2 passed

- [ ] **Step 4: Commit**

```bash
git add crates/age-core/src/tool/
git commit -m "feat(core): add ToolCall and ToolResult protocol types"
```

---

### Task 2: Tool Registry

**Files:**
- Create: `crates/age-core/src/tool/registry.rs`
- Create: `crates/age-core/src/tool/context.rs`

- [ ] **Step 1: 写失败测试与实现**

`crates/age-core/src/tool/context.rs`:
```rust
use std::sync::{Arc, RwLock};

use crate::schema::world3d::SceneDocument;
use crate::schema::ui::UiDocument;

#[derive(Debug, Clone, Default)]
pub struct ToolContext {
    pub scene: Arc<RwLock<SceneDocument>>,
    pub ui: Arc<RwLock<UiDocument>>,
}

impl ToolContext {
    pub fn new(scene: SceneDocument, ui: UiDocument) -> Self {
        Self {
            scene: Arc::new(RwLock::new(scene)),
            ui: Arc::new(RwLock::new(ui)),
        }
    }
}
```

`crates/age-core/src/tool/registry.rs`:
```rust
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
        let handler: ToolHandler = Arc::new(move |ctx, args| {
            Box::pin(handler(ctx, args))
        });
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
        registry.register("echo", |_ctx, args| async move {
            ToolResult::success("test", args)
        });

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
```

- [ ] **Step 2: 运行测试**

Run: `cargo test -p age-core tool::registry`
Expected: 2 passed (需要先完成 Task 3 的 Schema stub，见下)

- [ ] **Step 3: Commit**

```bash
git add crates/age-core/src/tool/
git commit -m "feat(core): add ToolRegistry with async handler execution"
```

---

### Task 3: World3D + UI Schema 最小类型（P0 stub）

**Files:**
- Create: `crates/age-core/src/schema/mod.rs`
- Create: `crates/age-core/src/schema/transform.rs`
- Create: `crates/age-core/src/schema/world3d.rs`
- Create: `crates/age-core/src/schema/ui.rs`
- Create: `crates/age-core/src/schema/validate.rs`

- [ ] **Step 1: Transform 类型 + 测试**

`crates/age-core/src/schema/transform.rs`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}
```

- [ ] **Step 2: World3D SceneDocument 最小实现**

`crates/age-core/src/schema/world3d.rs`:
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::transform::Transform;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum NodeType {
    Camera3D,
    DirectionalLight,
    PointLight,
    MeshInstance,
    PrefabInstance,
    Marker3D,
    PlayerController,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneNode {
    pub id: String,
    #[serde(flatten)]
    pub node_type: NodeType,
    #[serde(default)]
    pub transform: Transform,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mesh: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prefab: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Component {
    #[serde(rename = "type")]
    pub component_type: String,
    #[serde(default)]
    pub props: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environment {
    #[serde(default)]
    pub ambient: AmbientLight,
    #[serde(default = "default_background")]
    pub background: String,
    #[serde(default)]
    pub skybox: Option<String>,
}

fn default_background() -> String {
    "#1a1a2e".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmbientLight {
    #[serde(default = "default_ambient_color")]
    pub color: String,
    #[serde(default = "default_ambient_intensity")]
    pub intensity: f32,
}

fn default_ambient_color() -> String { "#404050".into() }
fn default_ambient_intensity() -> f32 { 0.3 }

impl Default for AmbientLight {
    fn default() -> Self {
        Self { color: default_ambient_color(), intensity: default_ambient_intensity() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneMetadata {
    pub name: String,
    #[serde(default = "default_units")]
    pub units: String,
}

fn default_units() -> String { "meters".into() }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneInner {
    pub mode: String,
    pub metadata: SceneMetadata,
    #[serde(default)]
    pub environment: Environment,
    #[serde(default)]
    pub nodes: Vec<SceneNode>,
    #[serde(default)]
    pub materials: Vec<Material>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Material {
    pub id: String,
    #[serde(rename = "type")]
    pub material_type: String,
    pub props: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneDocument {
    pub version: u32,
    pub scene: SceneInner,
}

impl SceneDocument {
    pub fn default_empty() -> Self {
        Self {
            version: 1,
            scene: SceneInner {
                mode: "3d".into(),
                metadata: SceneMetadata { name: "untitled".into(), units: "meters".into() },
                environment: Environment {
                    ambient: AmbientLight::default(),
                    background: default_background(),
                    skybox: None,
                },
                nodes: vec![],
                materials: vec![],
            },
        }
    }

    pub fn new_node_id(prefix: &str) -> String {
        format!("{}_{}", prefix, &Uuid::new_v4().to_string()[..8])
    }
}
```

- [ ] **Step 3: UI Document 最小实现**

`crates/age-core/src/schema/ui.rs`:
```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiDocument {
    pub version: u32,
    #[serde(default)]
    pub screens: Vec<UiScreen>,
    #[serde(default)]
    pub theme: UiTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiTheme {
    #[serde(default = "default_primary")]
    pub primary: String,
    #[serde(default = "default_bg")]
    pub background: String,
    #[serde(default = "default_text")]
    pub text: String,
}

fn default_primary() -> String { "#4a90d9".into() }
fn default_bg() -> String { "#1a1a2e".into() }
fn default_text() -> String { "#ffffff".into() }

impl Default for UiTheme {
    fn default() -> Self {
        Self { primary: default_primary(), background: default_bg(), text: default_text() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiScreen {
    pub id: String,
    #[serde(default = "default_layer")]
    pub layer: String,
    pub root: UiWidget,
}

fn default_layer() -> String { "overlay".into() }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiWidget {
    #[serde(rename = "type")]
    pub widget_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub props: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<UiWidget>,
}

impl UiDocument {
    pub fn default_empty() -> Self {
        Self { version: 1, screens: vec![], theme: UiTheme::default() }
    }

    pub fn new_widget_id(prefix: &str) -> String {
        format!("{}_{}", prefix, &Uuid::new_v4().to_string()[..8])
    }
}
```

- [ ] **Step 4: validate.rs 占位 + 测试**

`crates/age-core/src/schema/validate.rs`:
```rust
use super::ui::UiDocument;
use super::world3d::SceneDocument;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ValidationError {
    #[error("unsupported scene mode: {0}")]
    UnsupportedMode(String),
    #[error("duplicate node id: {0}")]
    DuplicateNodeId(String),
}

pub fn validate_scene(doc: &SceneDocument) -> Result<(), ValidationError> {
    if doc.scene.mode == "2d" {
        return Err(ValidationError::UnsupportedMode("2d".into()));
    }
    let mut seen = std::collections::HashSet::new();
    for node in &doc.scene.nodes {
        if !seen.insert(&node.id) {
            return Err(ValidationError::DuplicateNodeId(node.id.clone()));
        }
    }
    Ok(())
}

pub fn validate_ui(_doc: &UiDocument) -> Result<(), ValidationError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::world3d::*;

    #[test]
    fn rejects_2d_mode() {
        let mut doc = SceneDocument::default_empty();
        doc.scene.mode = "2d".into();
        assert_eq!(
            validate_scene(&doc).unwrap_err(),
            ValidationError::UnsupportedMode("2d".into())
        );
    }
}
```

`crates/age-core/src/schema/mod.rs`:
```rust
pub mod transform;
pub mod world3d;
pub mod ui;
pub mod validate;
```

- [ ] **Step 5: 运行全部 core 测试**

Run: `cargo test -p age-core`
Expected: all passed

- [ ] **Step 6: Commit**

```bash
git add crates/age-core/src/schema/
git commit -m "feat(core): add World3D and UI schema types with validation"
```

---

### Task 4: Agent Orchestrator + Mock Tool 链式调用

**Files:**
- Create: `crates/age-agent/src/orchestrator.rs`
- Create: `crates/age-agent/src/lib.rs` (update)

- [ ] **Step 1: 写失败测试**

`crates/age-agent/src/orchestrator.rs`:
```rust
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
                    ToolCall { id: "1".into(), tool: "increment".into(), args: json!({ "n": 0 }) },
                    ToolCall { id: "2".into(), tool: "increment".into(), args: json!({ "n": 1 }) },
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
```

- [ ] **Step 2: 更新 lib.rs**

`crates/age-agent/src/lib.rs`:
```rust
pub mod orchestrator;

pub use orchestrator::{AgentTraceEntry, Orchestrator};
```

- [ ] **Step 3: 运行测试**

Run: `cargo test -p age-agent`
Expected: 1 passed

- [ ] **Step 4: Commit**

```bash
git add crates/age-agent/
git commit -m "feat(agent): add Orchestrator with trace logging and batch execution"
```

---

### Task 5: 2D Tool 占位注册

**Files:**
- Create: `crates/age-agent/src/tools/mod.rs`
- Create: `crates/age-agent/src/tools/stubs.rs`

- [ ] **Step 1: 实现 NOT_IMPLEMENTED stubs**

`crates/age-agent/src/tools/stubs.rs`:
```rust
use age_core::tool::protocol::ToolResult;
use age_core::tool::registry::ToolRegistry;

pub fn register_stub_2d_tools(registry: &mut ToolRegistry) {
    for name in ["paint_tiles", "create_tilemap_layer", "import_tileset"] {
        let tool_name = name.to_string();
        registry.register(name, move |_ctx, _args| {
            let tool_name = tool_name.clone();
            async move {
                ToolResult::failure(
                    "stub",
                    "NOT_IMPLEMENTED",
                    format!("tool '{}' is planned for phase 2", tool_name),
                )
            }
        });
    }
}
```

- [ ] **Step 2: 测试 stub 返回 NOT_IMPLEMENTED**

Run: `cargo test -p age-agent` (add test in stubs.rs if needed)
Expected: PASS

- [ ] **Step 3: Commit — P0 完成**

```bash
git add crates/age-agent/src/tools/
git commit -m "feat(agent): register 2D tool stubs returning NOT_IMPLEMENTED"
```

**P0 验收命令:** `cargo test -p age-core -p age-agent`

---

## Phase P1: World3D + UI Schema 完整读写（2 周）

**交付验收：** 项目模板可加载/保存；`place_primitive` mock 写入 scene；Schema diff 可用。

---

### Task 6: Project IO

**Files:**
- Create: `crates/age-core/src/project/mod.rs`
- Create: `crates/age-core/src/project/io.rs`
- Create: `templates/default-project/**`

- [ ] **Step 1: 实现 Project 加载/保存**

`crates/age-core/src/project/io.rs` 核心 API:
```rust
pub struct Project {
    pub root: PathBuf,
    pub manifest: ProjectManifest,
    pub scene: SceneDocument,
    pub ui: UiDocument,
}

impl Project {
    pub fn load(root: impl AsRef<Path>) -> Result<Self, ProjectError>;
    pub fn save(&self) -> Result<(), ProjectError>;
    pub fn create_new(root: impl AsRef<Path>, name: &str) -> Result<Self, ProjectError>;
}
```

- [ ] **Step 2: 写集成测试（tempdir）**

```rust
#[test]
fn create_and_load_project_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let project = Project::create_new(dir.path(), "demo_room").unwrap();
    project.save().unwrap();
    let loaded = Project::load(dir.path()).unwrap();
    assert_eq!(loaded.scene.scene.metadata.name, "demo_room");
}
```

Run: `cargo test -p age-core project`
Expected: PASS

- [ ] **Step 3: 添加 templates/default-project 模板文件**

- [ ] **Step 4: Commit**

```bash
git commit -m "feat(core): add project load/save and default template"
```

---

### Task 7: Dev Level Tools — 核心 8 个（第一批）

**Files:**
- Create: `crates/age-agent/src/tools/level.rs`
- Modify: `crates/age-agent/src/tools/mod.rs`

实现 Tool（含测试）:
1. `create_scene`
2. `set_environment`
3. `place_primitive`
4. `move_node`
5. `delete_node`
6. `place_spawn`
7. `get_scene_summary`
8. `query_nodes`

`place_primitive` 示例逻辑:
```rust
pub async fn place_primitive(ctx: ToolContext, args: Value) -> ToolResult {
    let shape = args.get("shape").and_then(|v| v.as_str()).unwrap_or("builtin://cube");
    let transform: Transform = serde_json::from_value(args.get("transform").cloned().unwrap_or_default()).unwrap_or_default();
    let node_id = SceneDocument::new_node_id("node");
    let mut scene = ctx.scene.write().unwrap();
    scene.scene.nodes.push(SceneNode {
        id: node_id.clone(),
        node_type: NodeType::MeshInstance,
        transform,
        mesh: Some(shape.into()),
        material: args.get("material").and_then(|v| v.as_str()).map(String::from),
        ..Default::default()
    });
    ToolResult::success("call", json!({ "node_id": node_id }))
}
```

- [ ] **每个 tool 一个单元测试**
- [ ] **Run:** `cargo test -p age-agent tools::level`
- [ ] **Commit:** `feat(agent): implement first batch of level editing tools`

---

### Task 8: Dev Level Tools — 剩余 10 个 + build_room

**Files:**
- Modify: `crates/age-agent/src/tools/level.rs`

实现:
- `add_light`, `add_camera`, `place_prefab`, `duplicate_node`, `scatter_props`
- `build_room`, `check_overlap`, `raycast`, `assign_material`, `import_gltf` (gltf stub 返回 NOT_IMPLEMENTED 若文件不存在)

`build_room` 逻辑: 生成 floor plane + 4 wall_segment + 可选 door gap。

- [ ] **build_room 集成测试:** 8×8 房间 → 节点数 ≥ 5
- [ ] **Commit:** `feat(agent): complete level editing tools including build_room`

---

### Task 9: Dev UI Tools — 12 个

**Files:**
- Create: `crates/age-agent/src/tools/ui.rs`

实现 spec 中全部 12 个 UI tools；`add_widget` 支持 Canvas/Panel/Label/Button/ProgressBar。

- [ ] **测试:** create_screen + add_widget(ProgressBar) + bind_property
- [ ] **Commit:** `feat(agent): implement UI editing tools`

---

### Task 10: Checkpoint Manager

**Files:**
- Create: `crates/age-agent/src/checkpoint.rs`
- Create: `crates/age-agent/src/safety.rs`

```rust
pub struct CheckpointManager {
    project_root: PathBuf,
}

impl CheckpointManager {
    pub fn create(&self, scene: &SceneDocument, ui: &UiDocument) -> Result<String, CheckpointError>;
    pub fn restore(&self, id: &str) -> Result<(SceneDocument, UiDocument), CheckpointError>;
}
```

- [ ] **测试:** create → modify → restore 恢复原始 scene
- [ ] **SafetyGuard:** `delete_node` 需 `args.confirmed == true`
- [ ] **Commit:** `feat(agent): add checkpoint manager and safety guard`

**P1 验收:** `cargo test -p age-core -p age-agent` + 手动 `Project::create_new` 后链式调用 8 个 tool 写入 scene.json

---

## Phase P2: Web 3D Runtime（3 周）

**交付验收:** `wasm-pack build` 成功；浏览器中加载 default scene，WASD 第三人称行走。

---

### Task 11: age-runtime crate 初始化 + WASM

**Files:**
- Create: `crates/age-runtime/Cargo.toml`
- Create: `crates/age-runtime/src/lib.rs`
- Modify: root `Cargo.toml` members

```toml
[package]
name = "age-runtime"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
age-core = { path = "../age-core" }
wgpu = "24"
winit = { version = "0.30", optional = true }
wasm-bindgen = "0.2"
serde_json = "1"
```

- [ ] **Step 1:** `cargo check -p age-runtime`
- [ ] **Step 2:** 添加 `#[wasm_bindgen] pub fn init_runtime(canvas_id: &str, scene_json: &str)` stub
- [ ] **Commit:** `feat(runtime): initialize age-runtime wasm crate`

---

### Task 12: SceneLoader

**Files:**
- Create: `crates/age-runtime/src/scene_loader.rs`

```rust
pub struct LoadedScene {
    pub meshes: Vec<RuntimeMesh>,
    pub lights: Vec<RuntimeLight>,
    pub camera: RuntimeCamera,
    pub player_spawn: Option<[f32; 3]>,
}

pub fn load_scene(doc: &SceneDocument) -> Result<LoadedScene, LoadError>;
```

- [ ] **测试:** 加载含 floor + DirectionalLight + Marker3D(spawn,player) 的 JSON
- [ ] **Commit:** `feat(runtime): add scene loader for builtin meshes`

---

### Task 13: 内置图元网格生成

**Files:**
- Create: `crates/age-runtime/src/mesh_builtin.rs`

为 `builtin://cube/plane/sphere/cylinder/ramp/wall_segment` 生成顶点/索引缓冲。

- [ ] **测试:** cube 36 顶点，plane 4 顶点
- [ ] **Commit:** `feat(runtime): add builtin primitive meshes`

---

### Task 14: Renderer3D (wgpu)

**Files:**
- Create: `crates/age-runtime/src/renderer3d.rs`

- Standard 材质 shader（albedo uniform）
- 1 DirectionalLight + ambient
- 深度测试 + 背面剔除

- [ ] **本地 winit 集成测试**（非 WASM）渲染 1 cube
- [ ] **Commit:** `feat(runtime): add wgpu 3D renderer with standard material`

---

### Task 15: PlayerController 第三人称

**Files:**
- Create: `crates/age-runtime/src/player_controller.rs`
- Create: `crates/age-runtime/src/input.rs`
- Create: `crates/age-runtime/src/physics_simple.rs`

规格（来自 spec §7.1）:
- 相机距离 3.5m，高度偏移 1.5m
- WASD 移动，RMB 拖拽旋转
- Capsule 碰撞

```rust
pub struct ThirdPersonController {
    pub camera_distance: f32,      // 3.5
    pub camera_height_offset: f32, // 1.5
    pub yaw: f32,
    pub pitch: f32,
}

pub fn update_player(&mut self, input: &InputState, dt: f32, colliders: &[Collider]);
```

- [ ] **测试:** 纯逻辑测试 position 随 W 键前进
- [ ] **Commit:** `feat(runtime): add third-person player controller`

---

### Task 16: GameLoop + WASM 入口

**Files:**
- Create: `crates/age-runtime/src/game_loop.rs`
- Modify: `crates/age-runtime/src/lib.rs`
- Create: `runtime-web/bootstrap.js`

```javascript
export async function startRuntime(canvas, sceneUrl) {
  const wasm = await init('./runtime.wasm');
  wasm.init_runtime(canvas.id, sceneUrl);
}
```

- [ ] **wasm-pack build --target web**
- [ ] **手动浏览器测试:** 加载 templates/default-project scene
- [ ] **Commit:** `feat(runtime): wire game loop and wasm entry point`

**P2 验收:** 浏览器中第三人称行走 + 相机旋转

---

## Phase P3: HTML Export Pipeline（1 周）

**交付验收:** CLI/库调用 `export_html(project)` → `dist/html/` 可双击运行。

---

### Task 17: age-export crate

**Files:**
- Create: `crates/age-export/Cargo.toml`
- Create: `crates/age-export/src/lib.rs`
- Create: `crates/age-export/src/html.rs`
- Create: `crates/age-export/src/targets.rs`

```rust
pub async fn export_html(project: &Project, out_dir: &Path) -> Result<PathBuf, ExportError> {
    validate_scene(&project.scene)?;
    // 1. copy/compile wasm
    // 2. copy scenes/ui/prefabs/assets
    // 3. write index.html from runtime-web/template
    // 4. write manifest.json
}
```

`manifest.json` 字段同 spec §9.2。

- [ ] **集成测试:** export → dist/html 目录结构完整
- [ ] **Commit:** `feat(export): implement HTML export pipeline`

---

### Task 18: ExportTarget 占位

**Files:**
- Modify: `crates/age-export/src/targets.rs`

```rust
pub fn all_targets() -> Vec<ExportTarget> {
    vec![
        ExportTarget { id: "html", status: Ready, ... },
        ExportTarget { id: "desktop", status: Placeholder, ... },
        ExportTarget { id: "mobile", status: Placeholder, ... },
    ]
}
```

- [ ] **测试:** desktop.build() 返回 ok=false + 指定 error message
- [ ] **Commit:** `feat(export): add desktop and mobile placeholder targets`

**P3 验收:** `cargo run -p age-export -- export ./templates/default-project -o ./dist/html` 后浏览器可玩

---

## Phase P4: Tauri IDE + Dev Agent 集成（2 周）

**交付验收:** IDE 内 Agent 输入「建 8×8 房间+HUD」→ 5 分钟内可 Play 预览。

---

### Task 19: Tauri 项目脚手架

**Files:**
- Create: `apps/age-ide/` (Tauri 2 + React + Vite)

```bash
npm create tauri-app@latest age-ide -- --template react-ts
# 移动到 apps/age-ide
```

- [ ] **Tauri Cargo.toml 依赖 age-agent, age-core, age-export**
- [ ] **Commit:** `feat(ide): scaffold Tauri 2 + React IDE`

---

### Task 20: Tauri Commands

**Files:**
- Create: `apps/age-ide/src-tauri/src/commands.rs`

Commands:
- `open_project(path)` → Project
- `agent_execute(prompt, mode)` → traces + results
- `export_html(out_dir)` → BuildResult
- `list_checkpoints()` / `restore_checkpoint(id)`
- `get_scene_json()` → 供 Viewport 刷新

- [ ] **Commit:** `feat(ide): add tauri commands for agent and export`

---

### Task 21: React IDE 布局

**Files:**
- Create: `apps/age-ide/src/App.tsx`
- Create: `apps/age-ide/src/components/AgentPanel.tsx`
- Create: `apps/age-ide/src/components/Viewport.tsx`
- Create: `apps/age-ide/src/components/ProjectTree.tsx`
- Create: `apps/age-ide/src/components/ExportMenu.tsx`
- Create: `apps/age-ide/src/components/CheckpointList.tsx`

布局同 spec §12；ExportMenu 中 Desktop/Mobile 按钮 `disabled` + tooltip。

- [ ] **Commit:** `feat(ide): implement main IDE layout components`

---

### Task 22: Viewport WebView 预览

**Files:**
- Modify: `apps/age-ide/src/components/Viewport.tsx`

- Play/Stop 切换
- WebView 加载 `runtime-web/index.html?scene=...`
- 选中节点 → 写入 Agent 上下文 state

- [ ] **Commit:** `feat(ide): embed HTML runtime in viewport webview`

---

### Task 23: LLM Planner 接口

**Files:**
- Create: `crates/age-agent/src/planner.rs`
- Create: `crates/age-agent/src/llm/mod.rs`

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, system: &str, user: &str) -> Result<String, LlmError>;
}

pub struct RuleBasedPlanner; // MVP fallback: 解析关键词触发固定 tool 链
pub struct LlmPlanner<P: LlmProvider> { ... }
```

MVP: 实现 `RuleBasedPlanner` 识别 demo 句式 → 调用 build_room 等 tool 链；`LlmPlanner` trait + mock provider 可注入。

- [ ] **测试:** demo 提示词 → 生成 ≥ 5 步 tool plan
- [ ] **Commit:** `feat(agent): add planner with rule-based demo fallback`

---

### Task 24: Agent Panel 端到端

**Files:**
- Modify: `apps/age-ide/src/components/AgentPanel.tsx`

- 用户输入 → `agent_execute` → 展示 Plan / Tool Trace
- 批次完成 → 刷新 Viewport + Checkpoint 列表

- [ ] **手动 E2E:** 输入 spec 附录 A 示例语句
- [ ] **Commit:** `feat(ide): wire agent panel end-to-end`

**P4 验收:** IDE 内完整 demo 场景生成 + HTML 导出

---

## Phase P5: Runtime Agent NPC（1 周）

**交付验收:** Play 模式下 guard NPC 巡逻；靠近按 E 触发对话。

---

### Task 25: Interaction 系统

**Files:**
- Create: `crates/age-runtime/src/interaction.rs`

- 检测玩家与 Interactable 距离 < range
- 设置 GameState `player.near_interactable`
- E 键触发 `on_interact`

- [ ] **Commit:** `feat(runtime): add interaction system`

---

### Task 26: UIRenderer + 数据绑定

**Files:**
- Create: `crates/age-runtime/src/ui_renderer.rs`

- 读取 UiDocument
- anchor 布局
- ProgressBar 绑定 `player.health`
- Label 绑定 `player.near_interactable` visible

- [ ] **Commit:** `feat(runtime): add UI overlay renderer with property binding`

---

### Task 27: AgentBrain 状态机

**Files:**
- Create: `crates/age-runtime/src/runtime_agent.rs`

状态: idle, patrol, dialogue

```rust
pub struct AgentBrain {
    pub personality: String,
    pub state: AgentState,
    pub patrol_points: Vec<[f32; 3]>,
    pub tick_interval_ms: u64, // 500-1000
}

pub fn tick(&mut self, perception: &Perception, dt: f32) -> Vec<RuntimeAction>;
```

- [ ] **测试:** patrol 两点间往返
- [ ] **Commit:** `feat(runtime): add AgentBrain state machine for NPC`

---

### Task 28: Runtime Agent Tools（Play 模式）

**Files:**
- Create: `crates/age-agent/src/tools/runtime.rs`

8 个 runtime tools；Play 模式下 Dev tools 白名单禁用。

- [ ] **对话:** `say` 输出到 UI 对话框
- [ ] **Commit:** `feat(agent): add runtime agent tools for play mode`

**P5 验收:** 第三人称走向 NPC → E 对话；宝箱可交互

---

## Phase P6: 占位与收尾（1 周）

---

### Task 29: 2D 模式 IDE 提示

**Files:**
- Modify: `apps/age-ide/src/components/ProjectTree.tsx`

当 `scene.mode === "2d"` 时显示 banner: 「2D 模块即将支持」

- [ ] **Commit:** `feat(ide): show 2D not-implemented banner`

---

### Task 30: 默认 Demo 项目 + Prefabs

**Files:**
- Create: `templates/demo-project/**`
- Prefabs: `guard.prefab.json`, `chest.prefab.json`

- [ ] **Commit:** `chore: add demo project template with guard and chest prefabs`

---

### Task 31: README + 开发文档

**Files:**
- Create: `README.md`

内容: 项目介绍、构建步骤、`cargo test`、wasm-pack、启动 IDE、HTML 导出、Demo 提示词。

- [ ] **Commit:** `docs: add README with build and demo instructions`

---

### Task 32: 全链路验收脚本

**Files:**
- Create: `scripts/verify-mvp.sh` (或 `.ps1`)

```powershell
cargo test --workspace
wasm-pack build crates/age-runtime --target web
cargo run -p age-export -- export templates/demo-project -o dist/html
# 输出验收清单
```

- [ ] **Run 全链路**
- [ ] **Commit:** `chore: add MVP verification script`

**P6 / MVP 最终验收:** spec §7.5 Demo 场景描述 → HTML 导出 → 浏览器第三人称探索 + NPC 对话 + 宝箱交互

---

## Spec Coverage 自检

| Spec 章节 | 对应 Task |
|-----------|-----------|
| §3 Agent-Tool-Schema 范式 | Task 4, 22, 24 |
| §4 World3D Schema | Task 3, 6, 7, 8 |
| §5 UI Schema | Task 3, 9, 26 |
| §6 Tool 协议 (18+12+8) | Task 1, 7, 8, 9, 28 |
| §6.6 2D 占位 | Task 5, 29 |
| §7 第三人称探索 | Task 15, 25 |
| §8 Web Runtime | Task 11-16, 25-27 |
| §9 HTML Export + 占位 | Task 17, 18, 20 |
| §10 Agent Orchestrator | Task 4, 10, 23 |
| §11 Checkpoint | Task 10 |
| §12 IDE 布局 | Task 19-22 |
| §13 实施阶段 P0-P6 | 全文 |
| 附录 A Demo 会话 | Task 24, 30, 32 |

---

## 依赖安装（首次开发环境）

```powershell
# Rust
rustup target add wasm32-unknown-unknown

# WASM
cargo install wasm-pack

# Node (IDE)
cd apps/age-ide && npm install

# 可选 LLM 本地
# ollama pull llama3.2
```

---

*Plan complete.*
