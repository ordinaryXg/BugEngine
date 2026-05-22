use std::path::Path;

use age_core::schema::transform::Transform;
use age_core::schema::world3d::{NodeType, SceneDocument, SceneNode};
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

fn parse_transform(args: &Value) -> Transform {
    args.get("transform")
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

fn find_node_index(nodes: &[SceneNode], node_id: &str) -> Option<usize> {
    nodes.iter().position(|n| n.id == node_id)
}

async fn create_scene(ctx: ToolContext, args: Value) -> ToolResult {
    let name = args
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("untitled");
    let mode = args
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("3d")
        .to_string();

    let mut scene = ctx.scene.write().unwrap();
    scene.scene.metadata.name = name.to_string();
    scene.scene.mode = mode;
    scene.scene.nodes.clear();
    scene.scene.materials.clear();

    ToolResult::success(call_id(&args), json!({ "name": name }))
}

async fn set_environment(ctx: ToolContext, args: Value) -> ToolResult {
    let mut scene = ctx.scene.write().unwrap();
    if let Some(ambient) = args.get("ambient") {
        if let Some(color) = ambient.get("color").and_then(|v| v.as_str()) {
            scene.scene.environment.ambient.color = color.to_string();
        }
        if let Some(intensity) = ambient.get("intensity").and_then(|v| v.as_f64()) {
            scene.scene.environment.ambient.intensity = intensity as f32;
        }
    }
    if let Some(background) = args.get("background").and_then(|v| v.as_str()) {
        scene.scene.environment.background = background.to_string();
    }
    ToolResult::success(call_id(&args), json!({ "updated": true }))
}

async fn add_light(ctx: ToolContext, args: Value) -> ToolResult {
    let light_type = args.get("type").and_then(|v| v.as_str()).unwrap_or("DirectionalLight");
    let node_type = match light_type {
        "PointLight" => NodeType::PointLight,
        _ => NodeType::DirectionalLight,
    };
    let node_id = SceneDocument::new_node_id("light");
    let transform = parse_transform(&args);
    ctx.scene.write().unwrap().scene.nodes.push(SceneNode {
        id: node_id.clone(),
        node_type,
        transform,
        ..Default::default()
    });
    ToolResult::success(call_id(&args), json!({ "node_id": node_id }))
}

async fn add_camera(ctx: ToolContext, args: Value) -> ToolResult {
    let node_id = args
        .get("id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| SceneDocument::new_node_id("camera"));
    let transform = parse_transform(&args);
    ctx.scene.write().unwrap().scene.nodes.push(SceneNode {
        id: node_id.clone(),
        node_type: NodeType::Camera3D,
        transform,
        ..Default::default()
    });
    ToolResult::success(call_id(&args), json!({ "node_id": node_id }))
}

async fn place_primitive(ctx: ToolContext, args: Value) -> ToolResult {
    let shape = args
        .get("shape")
        .and_then(|v| v.as_str())
        .unwrap_or("builtin://cube");
    let transform = parse_transform(&args);
    let tags: Vec<String> = args
        .get("tags")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let node_id = SceneDocument::new_node_id("node");
    ctx.scene.write().unwrap().scene.nodes.push(SceneNode {
        id: node_id.clone(),
        node_type: NodeType::MeshInstance,
        transform,
        tags,
        mesh: Some(shape.to_string()),
        material: args
            .get("material")
            .and_then(|v| v.as_str())
            .map(String::from),
        ..Default::default()
    });
    ToolResult::success(call_id(&args), json!({ "node_id": node_id }))
}

async fn place_prefab(ctx: ToolContext, args: Value) -> ToolResult {
    let prefab_id = match args.get("prefab_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return ToolResult::failure(
                call_id(&args),
                "INVALID_ARGS",
                "prefab_id is required",
            )
        }
    };
    let transform = parse_transform(&args);
    let tags: Vec<String> = args
        .get("tags")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    let node_id = SceneDocument::new_node_id("prefab");

    let mut components = vec![];
    if let Some(root) = ctx.project_root.as_ref().and_then(|root| {
        load_prefab_root(root, &prefab_id).ok()
    }) {
        components = root.components;
    }

    ctx.scene.write().unwrap().scene.nodes.push(SceneNode {
        id: node_id.clone(),
        node_type: NodeType::PrefabInstance,
        transform,
        tags,
        prefab: Some(prefab_id),
        components,
        ..Default::default()
    });
    ToolResult::success(call_id(&args), json!({ "node_id": node_id }))
}

fn load_prefab_root(
    project_root: &Path,
    prefab_id: &str,
) -> Result<SceneNode, std::io::Error> {
    let path = project_root.join(format!("{prefab_id}.prefab.json"));
    let content = std::fs::read_to_string(path)?;
    let doc: Value = serde_json::from_str(&content).unwrap_or(json!({}));
    Ok(SceneNode {
        id: String::new(),
        node_type: NodeType::MeshInstance,
        mesh: doc
            .pointer("/root/mesh")
            .and_then(|v| v.as_str())
            .map(String::from),
        material: doc
            .pointer("/root/material")
            .and_then(|v| v.as_str())
            .map(String::from),
        components: doc
            .pointer("/root/components")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default(),
        ..Default::default()
    })
}

async fn move_node(ctx: ToolContext, args: Value) -> ToolResult {
    let node_id = match args.get("node_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return ToolResult::failure(call_id(&args), "INVALID_ARGS", "node_id is required")
        }
    };
    let transform = parse_transform(&args);
    let mut scene = ctx.scene.write().unwrap();
    let Some(index) = find_node_index(&scene.scene.nodes, &node_id) else {
        return ToolResult::failure(call_id(&args), "NODE_NOT_FOUND", node_id);
    };
    scene.scene.nodes[index].transform = transform;
    ToolResult::success(call_id(&args), json!({ "node_id": node_id }))
}

async fn duplicate_node(ctx: ToolContext, args: Value) -> ToolResult {
    let node_id = match args.get("node_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return ToolResult::failure(call_id(&args), "INVALID_ARGS", "node_id is required")
        }
    };
    let offset: [f32; 3] = args
        .get("offset")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or([1.0, 0.0, 0.0]);

    let mut scene = ctx.scene.write().unwrap();
    let Some(index) = find_node_index(&scene.scene.nodes, &node_id) else {
        return ToolResult::failure(call_id(&args), "NODE_NOT_FOUND", node_id);
    };
    let mut copy = scene.scene.nodes[index].clone();
    copy.id = SceneDocument::new_node_id("node");
    copy.transform.position[0] += offset[0];
    copy.transform.position[1] += offset[1];
    copy.transform.position[2] += offset[2];
    let new_id = copy.id.clone();
    scene.scene.nodes.push(copy);
    ToolResult::success(call_id(&args), json!({ "node_id": new_id }))
}

async fn delete_node(ctx: ToolContext, args: Value) -> ToolResult {
    if let Err(e) = needs_confirmation(&args, "delete_node") {
        return ToolResult::failure(call_id(&args), e.code(), e.to_string());
    }
    let node_id = match args.get("node_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return ToolResult::failure(call_id(&args), "INVALID_ARGS", "node_id is required")
        }
    };
    let mut scene = ctx.scene.write().unwrap();
    let Some(index) = find_node_index(&scene.scene.nodes, &node_id) else {
        return ToolResult::failure(call_id(&args), "NODE_NOT_FOUND", node_id);
    };
    scene.scene.nodes.remove(index);
    ToolResult::success(call_id(&args), json!({ "deleted": node_id }))
}

async fn scatter_props(ctx: ToolContext, args: Value) -> ToolResult {
    let prefab_id = args
        .get("prefab_id")
        .and_then(|v| v.as_str())
        .unwrap_or("prefabs/prop");
    let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
    let region = args.get("region").cloned().unwrap_or(json!({
        "min": [-2.0, 0.0, -2.0],
        "max": [2.0, 0.0, 2.0]
    }));
    let min: [f32; 3] = region
        .get("min")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or([-2.0, 0.0, -2.0]);
    let max: [f32; 3] = region
        .get("max")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or([2.0, 0.0, 2.0]);

    let mut placed = vec![];
    for i in 0..count {
        let t = (i as f32 + 1.0) / (count as f32 + 1.0);
        let x = min[0] + (max[0] - min[0]) * t;
        let z = min[2] + (max[2] - min[2]) * t;
        let node_id = SceneDocument::new_node_id("scatter");
        ctx.scene.write().unwrap().scene.nodes.push(SceneNode {
            id: node_id.clone(),
            node_type: NodeType::PrefabInstance,
            transform: Transform {
                position: [x, min[1], z],
                ..Default::default()
            },
            prefab: Some(prefab_id.to_string()),
            ..Default::default()
        });
        placed.push(node_id);
    }
    ToolResult::success(call_id(&args), json!({ "node_ids": placed }))
}

async fn build_room(ctx: ToolContext, args: Value) -> ToolResult {
    let size: [f32; 2] = args
        .get("size")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or([8.0, 8.0]);
    let wall_height = args
        .get("wall_height")
        .and_then(|v| v.as_f64())
        .unwrap_or(2.0) as f32;
    let floor_material = args
        .get("floor_material")
        .and_then(|v| v.as_str())
        .map(String::from);

    let w = size[0];
    let d = size[1];
    let half_w = w / 2.0;
    let half_d = d / 2.0;

    let mut nodes = vec![
        SceneNode {
            id: SceneDocument::new_node_id("floor"),
            node_type: NodeType::MeshInstance,
            transform: Transform {
                position: [0.0, 0.0, 0.0],
                scale: [w, 1.0, d],
                ..Default::default()
            },
            mesh: Some("builtin://plane".into()),
            material: floor_material,
            tags: vec!["floor".into()],
            ..Default::default()
        },
        SceneNode {
            id: SceneDocument::new_node_id("wall"),
            node_type: NodeType::MeshInstance,
            transform: Transform {
                position: [0.0, wall_height / 2.0, -half_d],
                scale: [w, wall_height, 0.2],
                ..Default::default()
            },
            mesh: Some("builtin://wall_segment".into()),
            tags: vec!["wall".into(), "north".into()],
            ..Default::default()
        },
        SceneNode {
            id: SceneDocument::new_node_id("wall"),
            node_type: NodeType::MeshInstance,
            transform: Transform {
                position: [0.0, wall_height / 2.0, half_d],
                scale: [w, wall_height, 0.2],
                ..Default::default()
            },
            mesh: Some("builtin://wall_segment".into()),
            tags: vec!["wall".into(), "south".into()],
            ..Default::default()
        },
        SceneNode {
            id: SceneDocument::new_node_id("wall"),
            node_type: NodeType::MeshInstance,
            transform: Transform {
                position: [-half_w, wall_height / 2.0, 0.0],
                scale: [0.2, wall_height, d],
                ..Default::default()
            },
            mesh: Some("builtin://wall_segment".into()),
            tags: vec!["wall".into(), "west".into()],
            ..Default::default()
        },
        SceneNode {
            id: SceneDocument::new_node_id("wall"),
            node_type: NodeType::MeshInstance,
            transform: Transform {
                position: [half_w, wall_height / 2.0, 0.0],
                scale: [0.2, wall_height, d],
                ..Default::default()
            },
            mesh: Some("builtin://wall_segment".into()),
            tags: vec!["wall".into(), "east".into()],
            ..Default::default()
        },
    ];

    if let Some(doors) = args.get("door_positions").and_then(|v| v.as_array()) {
        for door in doors {
            let wall = door.get("wall").and_then(|v| v.as_str()).unwrap_or("south");
            nodes.retain(|n| !n.tags.iter().any(|t| t == wall));
        }
    }

    let count = nodes.len();
    ctx.scene.write().unwrap().scene.nodes.extend(nodes);
    ToolResult::success(call_id(&args), json!({ "nodes_created": count, "size": size }))
}

async fn place_spawn(ctx: ToolContext, args: Value) -> ToolResult {
    let tag = args
        .get("tag")
        .and_then(|v| v.as_str())
        .unwrap_or("player")
        .to_string();
    let position: [f32; 3] = args
        .get("position")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or([0.0, 0.0, 0.0]);
    let rotation: [f32; 3] = args
        .get("rotation")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or([0.0, 0.0, 0.0]);
    let node_id = SceneDocument::new_node_id("spawn");
    ctx.scene.write().unwrap().scene.nodes.push(SceneNode {
        id: node_id.clone(),
        node_type: NodeType::Marker3D,
        transform: Transform {
            position,
            rotation,
            ..Default::default()
        },
        tags: vec!["spawn".into(), tag],
        ..Default::default()
    });
    ToolResult::success(call_id(&args), json!({ "node_id": node_id }))
}

async fn query_nodes(ctx: ToolContext, args: Value) -> ToolResult {
    let filter = args.get("filter").cloned().unwrap_or(json!({}));
    let node_type = filter.get("type").and_then(|v| v.as_str());
    let tags: Vec<String> = filter
        .get("tags")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let scene = ctx.scene.read().unwrap();
    let matched: Vec<&SceneNode> = scene
        .scene
        .nodes
        .iter()
        .filter(|n| {
            let type_ok = node_type.is_none_or(|t| {
                serde_json::to_string(&n.node_type)
                    .map(|s| s.contains(t))
                    .unwrap_or(false)
            });
            let tags_ok = tags.is_empty() || tags.iter().all(|t| n.tags.contains(t));
            type_ok && tags_ok
        })
        .collect();

    let ids: Vec<&str> = matched.iter().map(|n| n.id.as_str()).collect();
    ToolResult::success(call_id(&args), json!({ "node_ids": ids, "count": ids.len() }))
}

async fn get_scene_summary(ctx: ToolContext, args: Value) -> ToolResult {
    let scene = ctx.scene.read().unwrap();
    let node_count = scene.scene.nodes.len();
    let lights = scene
        .scene
        .nodes
        .iter()
        .filter(|n| {
            matches!(
                n.node_type,
                NodeType::DirectionalLight | NodeType::PointLight
            )
        })
        .count();
    ToolResult::success(
        call_id(&args),
        json!({
            "name": scene.scene.metadata.name,
            "mode": scene.scene.mode,
            "node_count": node_count,
            "light_count": lights,
            "material_count": scene.scene.materials.len(),
        }),
    )
}

async fn check_overlap(ctx: ToolContext, args: Value) -> ToolResult {
    let scene = ctx.scene.read().unwrap();
    let target = args.get("node_id").and_then(|v| v.as_str());
    let mut overlaps = vec![];
    for (i, a) in scene.scene.nodes.iter().enumerate() {
        if target.is_some_and(|id| id == a.id) {
            continue;
        }
        for (j, b) in scene.scene.nodes.iter().enumerate() {
            if i >= j {
                continue;
            }
            if target.is_some_and(|id| id != a.id && id != b.id) {
                continue;
            }
            if aabb_overlap(a, b) {
                overlaps.push(json!({ "a": a.id, "b": b.id }));
            }
        }
    }
    ToolResult::success(
        call_id(&args),
        json!({ "overlaps": overlaps, "count": overlaps.len() }),
    )
}

fn node_bounds(node: &SceneNode) -> ([f32; 3], [f32; 3]) {
    let p = node.transform.position;
    let s = node.transform.scale;
    let half = [s[0] / 2.0, s[1] / 2.0, s[2] / 2.0];
    (
        [p[0] - half[0], p[1] - half[1], p[2] - half[2]],
        [p[0] + half[0], p[1] + half[1], p[2] + half[2]],
    )
}

fn aabb_overlap(a: &SceneNode, b: &SceneNode) -> bool {
    let (a_min, a_max) = node_bounds(a);
    let (b_min, b_max) = node_bounds(b);
    a_min[0] <= b_max[0]
        && a_max[0] >= b_min[0]
        && a_min[1] <= b_max[1]
        && a_max[1] >= b_min[1]
        && a_min[2] <= b_max[2]
        && a_max[2] >= b_min[2]
}

async fn raycast(ctx: ToolContext, args: Value) -> ToolResult {
    let from: [f32; 3] = args
        .get("from")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or([0.0, 1.0, 0.0]);
    let direction: [f32; 3] = args
        .get("direction")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or([0.0, 0.0, -1.0]);
    let max_distance = args
        .get("max_distance")
        .and_then(|v| v.as_f64())
        .unwrap_or(100.0) as f32;

    let scene = ctx.scene.read().unwrap();
    let mut hit: Option<String> = None;
    let mut closest = max_distance;
    for node in &scene.scene.nodes {
        let (min, max) = node_bounds(node);
        if ray_hits_aabb(from, direction, min, max, closest) {
            let dist = (min[0] - from[0]).abs() + (min[1] - from[1]).abs() + (min[2] - from[2]).abs();
            if dist < closest {
                closest = dist;
                hit = Some(node.id.clone());
            }
        }
    }
    ToolResult::success(call_id(&args), json!({ "node_id": hit, "distance": closest }))
}

fn ray_hits_aabb(origin: [f32; 3], dir: [f32; 3], min: [f32; 3], max: [f32; 3], max_dist: f32) -> bool {
    for i in 0..3 {
        if dir[i].abs() < f32::EPSILON {
            if origin[i] < min[i] || origin[i] > max[i] {
                return false;
            }
        }
    }
    let end = [
        origin[0] + dir[0] * max_dist,
        origin[1] + dir[1] * max_dist,
        origin[2] + dir[2] * max_dist,
    ];
    end[0] >= min[0] && end[0] <= max[0] && end[1] >= min[1] && end[1] <= max[1]
}

async fn assign_material(ctx: ToolContext, args: Value) -> ToolResult {
    let node_id = match args.get("node_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return ToolResult::failure(call_id(&args), "INVALID_ARGS", "node_id is required")
        }
    };
    let material_id = match args.get("material_id").and_then(|v| v.as_str()) {
        Some(id) => id.to_string(),
        None => {
            return ToolResult::failure(call_id(&args), "INVALID_ARGS", "material_id is required")
        }
    };
    let mut scene = ctx.scene.write().unwrap();
    let Some(index) = find_node_index(&scene.scene.nodes, &node_id) else {
        return ToolResult::failure(call_id(&args), "NODE_NOT_FOUND", node_id);
    };
    scene.scene.nodes[index].material = Some(material_id.clone());
    ToolResult::success(call_id(&args), json!({ "node_id": node_id, "material_id": material_id }))
}

async fn import_gltf(ctx: ToolContext, args: Value) -> ToolResult {
    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
    let full_path = ctx
        .project_root
        .as_ref()
        .map(|root| root.join(path))
        .filter(|p| p.exists());

    if full_path.is_none() {
        return ToolResult::failure(
            call_id(&args),
            "NOT_IMPLEMENTED",
            format!("gltf import not available for path '{path}'"),
        );
    }

    let node_id = SceneDocument::new_node_id("gltf");
    ctx.scene.write().unwrap().scene.nodes.push(SceneNode {
        id: node_id.clone(),
        node_type: NodeType::MeshInstance,
        mesh: Some(format!("asset://{path}")),
        ..Default::default()
    });
    ToolResult::success(call_id(&args), json!({ "node_id": node_id, "path": path }))
}

pub fn register_level_tools(registry: &mut ToolRegistry) {
    registry.register("create_scene", |ctx, args| async move { create_scene(ctx, args).await });
    registry.register("set_environment", |ctx, args| async move {
        set_environment(ctx, args).await
    });
    registry.register("add_light", |ctx, args| async move { add_light(ctx, args).await });
    registry.register("add_camera", |ctx, args| async move { add_camera(ctx, args).await });
    registry.register("place_primitive", |ctx, args| async move {
        place_primitive(ctx, args).await
    });
    registry.register("place_prefab", |ctx, args| async move { place_prefab(ctx, args).await });
    registry.register("move_node", |ctx, args| async move { move_node(ctx, args).await });
    registry.register("duplicate_node", |ctx, args| async move {
        duplicate_node(ctx, args).await
    });
    registry.register("delete_node", |ctx, args| async move { delete_node(ctx, args).await });
    registry.register("scatter_props", |ctx, args| async move {
        scatter_props(ctx, args).await
    });
    registry.register("build_room", |ctx, args| async move { build_room(ctx, args).await });
    registry.register("place_spawn", |ctx, args| async move { place_spawn(ctx, args).await });
    registry.register("query_nodes", |ctx, args| async move { query_nodes(ctx, args).await });
    registry.register("get_scene_summary", |ctx, args| async move {
        get_scene_summary(ctx, args).await
    });
    registry.register("check_overlap", |ctx, args| async move {
        check_overlap(ctx, args).await
    });
    registry.register("raycast", |ctx, args| async move { raycast(ctx, args).await });
    registry.register("assign_material", |ctx, args| async move {
        assign_material(ctx, args).await
    });
    registry.register("import_gltf", |ctx, args| async move { import_gltf(ctx, args).await });
}

#[cfg(test)]
mod tests {
    use super::*;
    use age_core::schema::ui::UiDocument;
    use age_core::schema::world3d::SceneDocument;
    use age_core::tool::protocol::ToolCall;

    fn test_registry() -> ToolRegistry {
        let mut registry = ToolRegistry::default();
        register_level_tools(&mut registry);
        registry
    }

    #[tokio::test]
    async fn place_primitive_adds_mesh_node() {
        let registry = test_registry();
        let ctx = ToolContext::default();
        let result = registry
            .execute(
                ctx,
                ToolCall {
                    id: "1".into(),
                    tool: "place_primitive".into(),
                    args: json!({ "shape": "builtin://cube", "_call_id": "1" }),
                },
            )
            .await;
        assert!(result.ok);
    }

    #[tokio::test]
    async fn build_room_creates_at_least_five_nodes() {
        let registry = test_registry();
        let ctx = ToolContext::new(SceneDocument::default_empty(), UiDocument::default_empty());
        let result = registry
            .execute(
                ctx.clone(),
                ToolCall {
                    id: "1".into(),
                    tool: "build_room".into(),
                    args: json!({ "size": [8, 8], "_call_id": "1" }),
                },
            )
            .await;
        assert!(result.ok);
        assert!(ctx.scene.read().unwrap().scene.nodes.len() >= 5);
    }

    #[tokio::test]
    async fn delete_node_requires_confirmation() {
        let registry = test_registry();
        let ctx = ToolContext::new(SceneDocument::default_empty(), UiDocument::default_empty());
        registry
            .execute(
                ctx.clone(),
                ToolCall {
                    id: "1".into(),
                    tool: "place_primitive".into(),
                    args: json!({ "_call_id": "1" }),
                },
            )
            .await;
        let node_id = ctx.scene.read().unwrap().scene.nodes[0].id.clone();
        let result = registry
            .execute(
                ctx,
                ToolCall {
                    id: "2".into(),
                    tool: "delete_node".into(),
                    args: json!({ "node_id": node_id, "_call_id": "2" }),
                },
            )
            .await;
        assert!(!result.ok);
        assert_eq!(result.error.unwrap().code, "NEEDS_CONFIRMATION");
    }

    #[tokio::test]
    async fn get_scene_summary_returns_counts() {
        let registry = test_registry();
        let ctx = ToolContext::default();
        let result = registry
            .execute(
                ctx,
                ToolCall {
                    id: "1".into(),
                    tool: "get_scene_summary".into(),
                    args: json!({ "_call_id": "1" }),
                },
            )
            .await;
        assert!(result.ok);
        assert!(result.result.unwrap().get("node_count").is_some());
    }
}
