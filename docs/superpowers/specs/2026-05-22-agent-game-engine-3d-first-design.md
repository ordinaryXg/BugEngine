# AgentGameEngine — 3D-First 全栈设计规格

**日期:** 2026-05-22  
**状态:** Approved — 2026-05-22  
**版本:** 0.1

---

## 1. 概述

AgentGameEngine 是一款 **AI Agent 深度介入** 的桌面游戏引擎。人类通过自然语言描述意图，Agent 通过结构化 Tool 操作场景与 UI Schema；人类在视口中预览与验收，而非以拖拽式编辑器作为主编辑路径。

### 1.1 产品定位

| 维度 | 决策 |
|------|------|
| 类型 | 全栈引擎（创作 Dev Agent + 运行时 Runtime Agent） |
| 维度 | **3D 优先**；2D Tilemap 延后，Schema 占位 |
| 首发导出 | **HTML/WASM** 完整实现 |
| 占位导出 | Desktop、Mobile（接口 + UI 占位，不实现打包） |
| MVP 玩法 | **第三人称探索**：行走、靠近 NPC 对话、简单交互（E 键） |
| 编辑范式 | UI、地编等原人工编辑模式 → **Agent Tool 执行** |

### 1.2 MVP 要证明的命题

> 用户用自然语言描述 → Agent 搭建 3D 场景 + UI HUD → HTML 打包 → 浏览器可玩（第三人称探索 + 1 个 Runtime Agent NPC）

### 1.3 非目标（MVP 不做）

- 2D Tilemap / Sprite 渲染与地编 Tool 实现
- Desktop / Mobile 实际打包
- PBR、骨骼动画、粒子、地形雕刻、NavMesh 烘焙
- 顶点级网格编辑、UV 编辑
- 可视化拖拽式关卡/UI 编辑器（主路径）
- 多 Agent 协作、每帧 LLM 决策

---

## 2. 系统架构

### 2.1 高层架构

```
┌─────────────────────────────────────────────────────────────┐
│                    Desktop IDE (Tauri 2)                     │
│  ┌─────────────┐  ┌──────────────────┐  ┌─────────────────┐ │
│  │ Agent Panel │  │ Viewport Preview │  │ Project /       │ │
│  │             │  │ (WebView 嵌入     │  │ Checkpoints     │ │
│  │             │  │  HTML Runtime)   │  │                 │ │
│  └──────┬──────┘  └────────▲─────────┘  └────────┬────────┘ │
│         │                  │                      │          │
│  ┌──────▼──────────────────┴──────────────────────▼──────┐ │
│  │              Agent Orchestrator (Rust)                   │ │
│  │  Planner → Tool Router → Memory → Safety → Checkpoint  │ │
│  └──────┬──────────────────────────────────────────────────┘ │
│         │ Tool Calls (JSON-RPC)                                │
│  ┌──────▼──────────────────────────────────────────────────┐ │
│  │              Engine Core (Rust)                          │ │
│  │  World3D Schema │ UI Schema │ Prefabs │ Event Bus        │ │
│  └──────┬──────────────────────────────────────────────────┘ │
└─────────┼─────────────────────────────────────────────────────┘
          │ Schema JSON
          ▼
┌─────────────────────────────────────────────────────────────┐
│              Web Runtime (Rust → WASM)                       │
│  wgpu (WebGPU / WebGL2) │ 3D Renderer │ UI Overlay │ Input  │
└─────────────────────────────────────────────────────────────┘
          │
          ▼
┌─────────────────────────────────────────────────────────────┐
│              Export Pipeline                                 │
│  HTML ✅  │  Desktop ⏳  │  Mobile ⏳                        │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心数据流

```
Human Intent
  → Agent Orchestrator (plan + tool selection)
  → Structured Tool Call
  → Schema Patch (World3D / UI)
  → Checkpoint (optional batch)
  → Viewport Refresh / HTML Export
```

**原则:** Schema 是唯一真相来源（Single Source of Truth）。Agent 不直接修改 WASM 或二进制；所有编辑可 diff、可回滚、可审计。

### 2.3 技术栈

| 层 | 技术 | 理由 |
|----|------|------|
| IDE 壳 | Tauri 2 | 轻量、Rust 后端与 Core 统一 |
| IDE UI | React + TypeScript | 生态成熟 |
| Engine Core | Rust | WASM 导出、性能、类型安全 |
| Web 渲染 | wgpu | WebGPU 优先，WebGL2 fallback |
| Agent LLM | 可插拔（Ollama 本地 + 云端 API） | 本地优先保护隐私 |
| 脚本 VM | Rhai（MVP 可选） | Rust 嵌入友好 |
| 项目存储 | 文件系统 + SQLite（元数据/记忆） | 简单可靠 |

---

## 3. 编辑范式：Agent-Tool-Schema

### 3.1 视口角色

视口是 **验收台**，不是主编辑器：

- **可看:** 3D 场景实时预览
- **可选:** 点击选中实体/UI 节点，注入 Agent 上下文
- **不可:** 拖拽移动、笔刷地编、UI 控件拖拽（MVP 主路径禁止）

Gizmo 在 MVP 中只读或仅用于选中反馈；位置修正是通过自然语言 + Agent Tool 完成。

### 3.2 Agent 模式

| 模式 | 行为 |
|------|------|
| **Ask** | 只分析、建议，不调用写入 Tool |
| **Agent** | 规划并执行 Tool，默认每批次创建 checkpoint |
| **Play** | Dev Agent 只读；Runtime Agent 通过 AgentBrain 驱动 NPC |

### 3.3 安全与验收

- 每次 Agent 批次编辑 = 一个 **checkpoint**，支持整批撤销
- 破坏性操作（`delete_node`、`delete_screen`、清空层）默认需用户确认
- Runtime Agent 禁止调用 Dev 地编/UI Tool
- 所有 Tool 调用记录到 **Agent Trace** 面板（plan → tool → result）

---

## 4. World3D Schema

### 4.1 文件布局

```
project/
├── project.json              # 项目元数据
├── scenes/
│   └── main.scene.json       # 主 3D 场景
├── ui/
│   └── hud.ui.json           # UI 文档
├── prefabs/
│   ├── guard.prefab.json
│   └── chest.prefab.json
├── materials/
│   └── *.mat.json
└── assets/
    └── models/               # glTF（可选）
```

### 4.2 Scene 根结构

```json
{
  "version": 1,
  "scene": {
    "mode": "3d",
    "metadata": {
      "name": "demo_room",
      "units": "meters"
    },
    "environment": {
      "ambient": { "color": "#404050", "intensity": 0.3 },
      "background": "#1a1a2e",
      "skybox": null
    },
    "nodes": [],
    "materials": []
  }
}
```

**`scene.mode` 有效值（MVP）:**

| 值 | MVP 行为 |
|----|----------|
| `"3d"` | 完整支持 |
| `"mixed"` | 3D 世界 + UI overlay |
| `"2d"` | 返回 `NotImplemented`，IDE 提示「2D 即将支持」 |

### 4.3 节点类型（MVP）

| 类型 | 说明 |
|------|------|
| `Camera3D` | 透视/正交相机 |
| `DirectionalLight` | 平行光（太阳） |
| `PointLight` | 点光源 |
| `MeshInstance` | 内置图元或 mesh 引用 |
| `PrefabInstance` | 预制体实例 |
| `Marker3D` | 出生点、路径点、交互点 |
| `PlayerController` | 第三人称玩家控制（MVP 内置） |

### 4.4 组件（附加在节点上）

| 组件 | 说明 |
|------|------|
| `Collider` | AABB 或 Capsule |
| `AgentBrain` | Runtime Agent 行为 |
| `Interactable` | 可交互（E 键），触发对话/拾取 |
| `Health` | 血量，绑定 HUD |

### 4.5 Transform

```json
{
  "position": [0.0, 0.5, 0.0],
  "rotation": [0.0, 45.0, 0.0],
  "scale": [1.0, 1.0, 1.0]
}
```

旋转单位：度（degrees），顺序 Y-up、YXZ。

### 4.6 内置图元（builtin mesh）

不依赖外部资源即可地编：

| ID | 形状 |
|----|------|
| `builtin://cube` | 1×1×1 立方体 |
| `builtin://plane` | 水平面 |
| `builtin://sphere` | 球体 |
| `builtin://cylinder` | 圆柱 |
| `builtin://ramp` | 斜面 |
| `builtin://wall_segment` | 墙段（2×2×0.2） |

### 4.7 Material（MVP 简化 Standard）

```json
{
  "id": "mat://floor_wood",
  "type": "Standard",
  "props": {
    "albedo": "#8B6914",
    "roughness": 0.8,
    "metallic": 0.0
  }
}
```

### 4.8 Prefab

```json
{
  "id": "prefabs/guard",
  "root": {
    "type": "MeshInstance",
    "mesh": "builtin://capsule_humanoid",
    "material": "mat://guard_blue",
    "components": [
      { "type": "Collider", "props": { "shape": "capsule", "radius": 0.4, "height": 1.8 } },
      { "type": "AgentBrain", "props": { "personality": "friendly_guard" } },
      { "type": "Interactable", "props": { "prompt": "与守卫交谈", "range": 2.0 } }
    ]
  }
}
```

### 4.9 2D 占位（Phase 2）

Schema 类型定义预留，无 runtime：

- `Node2D`, `Sprite2D`, `TilemapLayer`
- Tool registry 预留 `paint_tiles` 等，返回 `{ "error": "NOT_IMPLEMENTED", "phase": 2 }`

---

## 5. UI Schema

UI 独立于 3D 场景，渲染为 3D 之上的 overlay（`mixed` 模式）。

### 5.1 文件结构

```json
{
  "version": 1,
  "screens": [
    {
      "id": "game_hud",
      "layer": "overlay",
      "root": { "type": "Canvas", "layout": { "anchor": "full" }, "children": [] }
    }
  ],
  "theme": {
    "primary": "#4a90d9",
    "background": "#1a1a2e",
    "text": "#ffffff"
  }
}
```

### 5.2 Widget 类型（MVP）

| 类型 | 说明 |
|------|------|
| `Canvas` | 根容器 |
| `Panel` | 面板容器 |
| `Label` | 文本 |
| `Button` | 按钮 |
| `ProgressBar` | 血条等 |
| `Image` | 图片（可选） |

### 5.3 Layout

MVP 支持 **anchor + margin** 布局：

```json
{
  "anchor": "top_left",
  "margin": [16, 16],
  "size": [200, 24]
}
```

Anchor 枚举：`full`, `top_left`, `top_center`, `top_right`, `bottom_left`, `bottom_center`, `bottom_right`, `center`.

### 5.4 数据绑定

```json
{
  "bind": {
    "value": "player.health",
    "max": "player.max_health",
    "visible": "player.near_interactable"
  }
}
```

绑定路径指向运行时 **Game State** 对象树。

### 5.5 事件绑定

```json
{
  "events": {
    "on_click": { "action": "script", "handler": "ui.on_pause_click" }
  }
}
```

---

## 6. Agent Tool 协议

### 6.1 Tool Call 格式

```json
{
  "id": "call_001",
  "tool": "place_primitive",
  "args": {
    "shape": "builtin://cube",
    "transform": { "position": [2, 0.5, 1], "rotation": [0, 0, 0], "scale": [1, 1, 1] },
    "material": "mat://stone"
  }
}
```

### 6.2 Tool Result 格式

```json
{
  "call_id": "call_001",
  "ok": true,
  "result": { "node_id": "node_abc123" },
  "checkpoint_id": "cp_0042"
}
```

错误：

```json
{
  "call_id": "call_001",
  "ok": false,
  "error": { "code": "NODE_NOT_FOUND", "message": "..." }
}
```

### 6.3 Dev Agent — 3D 地编 Tools（18 个）

#### 场景结构

| Tool | 参数 | 说明 |
|------|------|------|
| `create_scene` | `name`, `mode?` | 新建场景 |
| `set_environment` | `ambient?`, `background?` | 环境设置 |
| `add_light` | `type`, `transform`, `props?` | 添加光源 |
| `add_camera` | `id?`, `transform`, `props?` | 添加相机 |

#### 关卡布局

| Tool | 参数 | 说明 |
|------|------|------|
| `place_primitive` | `shape`, `transform`, `material?`, `tags?` | 放置内置图元 |
| `place_prefab` | `prefab_id`, `transform`, `overrides?`, `tags?` | 放置预制体 |
| `move_node` | `node_id`, `transform` | 移动/旋转/缩放 |
| `duplicate_node` | `node_id`, `offset?` | 复制节点 |
| `delete_node` | `node_id` | 删除（需确认） |
| `scatter_props` | `prefab_id`, `region`, `count`, `rules?` | 区域内随机放置 |

#### 关卡模板

| Tool | 参数 | 说明 |
|------|------|------|
| `build_room` | `size`, `wall_height?`, `door_positions?`, `floor_material?` | 生成矩形房间 |
| `place_spawn` | `tag`, `position`, `rotation?` | 放置出生点 Marker |

#### 查询与验证

| Tool | 参数 | 说明 |
|------|------|------|
| `query_nodes` | `filter` | 按 type/tag/region 查询 |
| `get_scene_summary` | — | 场景统计与边界 |
| `check_overlap` | `node_id?` | 碰撞重叠检测 |
| `raycast` | `from`, `direction`, `max_distance?` | 射线检测 |

#### 资源

| Tool | 参数 | 说明 |
|------|------|------|
| `assign_material` | `node_id`, `material_id` | 指定材质 |
| `import_gltf` | `path`, `as_prefab?` | 导入 glTF（MVP 基础） |

### 6.4 Dev Agent — UI Tools（12 个）

| Tool | 参数 | 说明 |
|------|------|------|
| `create_screen` | `id`, `layer?` | 新建 UI 屏幕 |
| `delete_screen` | `screen_id` | 删除屏幕 |
| `add_widget` | `screen_id`, `parent_id?`, `type`, `id?`, `props?` | 添加控件 |
| `remove_widget` | `screen_id`, `widget_id` | 移除控件 |
| `set_layout` | `screen_id`, `widget_id`, `layout` | 设置布局 |
| `set_style` | `screen_id`, `widget_id`, `style` | 设置样式 |
| `bind_property` | `screen_id`, `widget_id`, `bind` | 数据绑定 |
| `bind_event` | `screen_id`, `widget_id`, `event`, `handler` | 事件绑定 |
| `preview_resolution` | `width`, `height` | 预览分辨率 |
| `set_visibility` | `screen_id`, `widget_id`, `visible` | 显隐 |
| `set_text` | `screen_id`, `widget_id`, `text` | 文本内容 |
| `query_ui_tree` | `screen_id?` | 查询 UI 树 |

### 6.5 Runtime Agent Tools（Play 模式，8 个）

| Tool | 说明 |
|------|------|
| `navigate_to` | 移动到目标点 |
| `look_at` | 朝向目标 |
| `say` | 对话输出 |
| `set_goal` | 设置行为目标（patrol / chase / idle） |
| `play_animation` | 播放动画（MVP: idle / walk） |
| `use_interaction` | 触发交互 |
| `get_perception` | 获取周围感知摘要 |
| `remember` | 写入运行时记忆 |

### 6.6 预留 2D Tools（占位）

| Tool | MVP 行为 |
|------|----------|
| `paint_tiles` | `NOT_IMPLEMENTED` |
| `create_tilemap_layer` | `NOT_IMPLEMENTED` |
| `import_tileset` | `NOT_IMPLEMENTED` |

---

## 7. 第三人称探索 — MVP 玩法规格

### 7.1 相机与控制

| 项 | 规格 |
|----|------|
| 相机模式 | 第三人称跟随（PlayerController 绑定） |
| 距离 | 默认 3.5m，可配置 |
| 高度偏移 | 1.5m |
| 碰撞 | 相机与场景 AABB 简单碰撞（防穿墙） |
| 移动 | WASD / 虚拟摇杆（HTML 触屏占位） |
| 旋转 | 鼠标右键拖拽 / 双指（触屏占位） |
| 交互键 | E / 触屏按钮 |

### 7.2 玩家实体

```json
{
  "id": "player",
  "type": "PlayerController",
  "transform": { "position": [0, 0, 0] },
  "components": [
    { "type": "Collider", "props": { "shape": "capsule", "radius": 0.35, "height": 1.7 } },
    { "type": "Health", "props": { "current": 100, "max": 100 } }
  ]
}
```

出生点：场景加载时将 Player 置于 `tags` 含 `spawn` + `player` 的 Marker3D。

### 7.3 交互系统

- 实体挂载 `Interactable` 组件
- 玩家进入 `range` 内 → Game State `player.near_interactable = true` → HUD 显示提示
- 按 E → 触发 `on_interact` 事件 → 对话 / 开宝箱等

### 7.4 Runtime Agent NPC（Demo）

默认 Demo：**friendly_guard**

| 状态 | 行为 |
|------|------|
| `idle` | 站立，面向玩家 |
| `patrol` | 在 2 个 Marker 路径点间走动 |
| `dialogue` | 玩家交互后 `say` 输出对话 |
| `chase`（可选） | 玩家攻击后追击（MVP 可简化） |

AgentBrain 每 **500ms–1s** 做一次高层决策（非每帧 LLM），中间用状态机插值。

### 7.5 默认 Demo 场景描述

Agent 应能响应：

> 「做一个 8×8 的房间，留一扇门，中间放一张桌子，角落放一个守卫 NPC，另一个角落放宝箱，玩家在门口出生，加血条和交互提示 HUD。」

验收：HTML 导出后，第三人称行走 → 靠近 NPC 按 E 对话 → 靠近宝箱按 E 打开。

---

## 8. Web Runtime

### 8.1 模块

| 模块 | 职责 |
|------|------|
| `SceneLoader` | 解析 World3D + UI Schema |
| `Renderer3D` | wgpu 网格、光照、相机 |
| `UIRenderer` | overlay UI 绘制 |
| `InputSystem` | 键鼠、触屏占位 |
| `PhysicsSimple` | AABB/Capsule 静态碰撞 + 玩家移动 |
| `GameLoop` | 固定 tick + 可变渲染 |
| `RuntimeAgentHost` | AgentBrain tick |

### 8.2 渲染要求（MVP）

- 1 个 DirectionalLight + ambient
- Standard 材质（albedo + roughness，无 normal map）
- 深度测试、背面剔除
- UI 在 3D 之后渲染

### 8.3 输入映射

| 输入 | 动作 |
|------|------|
| W/A/S/D | 移动 |
| Mouse RMB + drag | 相机旋转 |
| E | 交互 |
| Escape | 暂停菜单（可选） |

---

## 9. Export Pipeline

### 9.1 ExportTarget 接口

```typescript
interface ExportTarget {
  id: 'html' | 'desktop' | 'mobile';
  status: 'ready' | 'placeholder';
  label: string;
  build(project: Project, options?: BuildOptions): Promise<BuildResult>;
}

interface BuildResult {
  ok: boolean;
  artifactPath?: string;
  error?: string;
}
```

### 9.2 HTML Export（完整实现）

**输入:** 项目目录  
**输出:**

```
dist/html/
├── index.html
├── runtime.wasm
├── runtime.js
├── scenes/main.scene.json
├── ui/hud.ui.json
├── prefabs/
├── materials/
├── assets/
└── manifest.json
```

**manifest.json:**

```json
{
  "engineVersion": "0.1.0",
  "exportTarget": "html",
  "scene": "scenes/main.scene.json",
  "ui": ["ui/hud.ui.json"],
  "builtAt": "2026-05-22T12:00:00Z"
}
```

**构建步骤:**

1. 校验 Schema
2. 复制/编译 Runtime WASM
3. 打包资源
4. 生成 `index.html` 引导页
5. 写入 manifest

### 9.3 Desktop / Mobile（占位）

```typescript
const desktopTarget: ExportTarget = {
  id: 'desktop',
  status: 'placeholder',
  label: 'Desktop（即将支持）',
  build: async () => ({
    ok: false,
    error: 'Desktop 导出即将支持，请先使用 HTML 导出',
  }),
};
```

IDE 导出面板：HTML 可点击；Desktop/Mobile 灰色禁用 + tooltip。

---

## 10. Agent Orchestrator

### 10.1 组件

| 组件 | 职责 |
|------|------|
| `Planner` | 将用户意图拆为多步计划 |
| `ToolRouter` | 选择并调用 Tool |
| `ProjectMemory` | 项目级长期记忆（SQLite + 向量） |
| `RuntimeMemory` | Play 模式 NPC 记忆 |
| `SafetyGuard` | 权限、确认、Tool 白名单 |
| `CheckpointManager` | 批次快照与撤销 |

### 10.2 上下文注入

Agent 每次请求携带：

- 当前 scene summary（`get_scene_summary` 缓存）
- 选中节点 ID（若有）
- 最近 N 条 Agent Trace
- 项目约定（命名、单位）

Play 模式额外携带：NPC perception summary。

### 10.3 LLM 策略

| 场景 | 模型策略 |
|------|----------|
| Dev 地编/UI | 云端或本地大模型，完整 Tool 集 |
| Runtime NPC 对话 | 小模型 / 本地，限制 token |
| Runtime 行为决策 | 规则 + 状态机为主，LLM 辅助 `set_goal` |

---

## 11. Checkpoint 协议

### 11.1 创建时机

- Agent 批次开始执行写入 Tool 前，自动创建 checkpoint
- 用户手动「保存检查点」

### 11.2 存储

```
.checkpoints/
├── cp_0001/
│   ├── meta.json
│   ├── scenes/main.scene.json
│   └── ui/hud.ui.json
└── cp_0002/
    └── ...
```

### 11.3 恢复

`restore_checkpoint(checkpoint_id)` → 覆盖当前 Schema → 刷新 Viewport。

---

## 12. IDE 布局

```
┌────────────────────────────────────────────────────────────┐
│ Menu: File | Edit | Play | Export(HTML✅ Desktop⏳ Mobile⏳) │
├──────────────────┬─────────────────────────────────────────┤
│ Project Tree     │  Viewport (WebView — HTML Runtime)      │
│ ├── scenes/      │  ┌─────────────────────────────────┐   │
│ ├── ui/          │  │  3D Preview + 选中高亮           │   │
│ ├── prefabs/     │  └─────────────────────────────────┘   │
│ Checkpoints      │                                         │
├──────────────────┤  Selection: node_id → Agent 上下文       │
│ Agent Panel      │                                         │
│ > 用户输入       │                                         │
│ [Plan]           │                                         │
│ [Tool calls]     │                                         │
│ [Trace log]      │                                         │
└──────────────────┴─────────────────────────────────────────┘
```

---

## 13. 实施阶段

| 阶段 | 周期 | 交付 | 验收标准 |
|------|------|------|----------|
| **P0** Agent Core + Tool Protocol | 2 周 | Orchestrator、Tool 注册、Mock 执行 | Agent 链式调用 mock tools 成功 |
| **P1** World3D + UI Schema | 2 周 | Schema 读写、校验、patch | JSON 可序列化、可 diff |
| **P2** Web 3D Runtime | 3 周 | 图元、光、相机、第三人称控制 | WebView 中可行走 |
| **P3** HTML Export | 1 周 | 一键导出 + IDE 预览 | 双击 index.html 可运行 |
| **P4** Dev Agent 3D 地编 + UI | 2 周 | 18+12 Tools 实装 | 自然语言 → 5 分钟内可玩 Demo |
| **P5** Runtime Agent NPC | 1 周 | AgentBrain + 8 Runtime Tools | NPC 巡逻 + 对话 |
| **P6** 占位 | 1 周 | 2D/Desktop/Mobile 占位 UI | 禁用态正确、提示清晰 |

**总计:** 约 12 周

---

## 14. 风险与缓解

| 风险 | 缓解 |
|------|------|
| LLM 幻觉导致错误场景 | Tool-only 写入 + checkpoint + 破坏性操作确认 |
| WebGPU 兼容性 | WebGL2 fallback |
| 3D Runtime 工作量大 | 内置图元优先，glTF 次之 |
| Agent 延迟 | Runtime 500ms–1s 决策间隔，非每帧 LLM |
|  scope 膨胀 | 严格 Non-Goals；2D 仅 Schema 占位 |

---

## 15. ADR 摘要

| ID | 决策 | 理由 |
|----|------|------|
| ADR-001 | 3D 优先，2D 延后 | 降低 MVP 复杂度，保留 Schema 扩展 |
| ADR-002 | HTML 首发导出 | 易预览、易分享、IDE WebView 同源 |
| ADR-003 | Agent-Tool-Schema 编辑 | 产品差异化，可审计可回滚 |
| ADR-004 | 第三人称探索为默认 Demo | 展示 3D + 交互 + Runtime Agent |
| ADR-005 | Tauri + Rust Core + WASM | 统一语言、本地优先 |
| ADR-006 | wgpu 渲染 | 现代 Web 图形 API |

---

## 16. 开放问题（Phase 2+）

- glTF 动画支持优先级
- 网络多人是否纳入路线图
- Desktop 打包是否基于 Tauri 内嵌 Runtime 或独立二进制
- 可视化「人类微调 → 语义指令」是否作为辅助路径

---

## 附录 A：示例 Agent 会话

**用户:** 做一个 8×8 的房间，留一扇门，中间放桌子，角落放守卫，玩家在门口出生，加血条和交互提示。

**Agent Plan:**
1. `build_room(size=[8,8], door_positions=[{wall:"south", offset:0}])`
2. `place_primitive(shape="builtin://cube", ...)` → table
3. `place_prefab(prefab_id="prefabs/guard", ...)`
4. `place_spawn(tag="player", position=[0, 0, 3.5])`
5. `create_screen(id="game_hud")` + `add_widget(ProgressBar)` + `add_widget(Label)`
6. `bind_property(health_bar, { value: "player.health" })`
7. `get_scene_summary()` + `check_overlap()`

**结果:** checkpoint `cp_0007`，Viewport 刷新，用户 Play 测试。

---

*文档结束 — 请审阅后反馈修改意见，批准后将进入实现计划阶段。*
