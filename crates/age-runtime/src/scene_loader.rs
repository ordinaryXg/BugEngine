use age_core::schema::transform::Transform;
use age_core::schema::world3d::{NodeType, SceneDocument, SceneNode};
use thiserror::Error;

use crate::mesh_builtin::{builtin_mesh, MeshData};

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("unsupported mesh: {0}")]
    UnsupportedMesh(String),
    #[error("scene has no camera")]
    MissingCamera,
}

#[derive(Debug, Clone)]
pub struct RuntimeMesh {
    pub node_id: String,
    pub mesh: MeshData,
    pub transform: Transform,
    pub color: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct RuntimeLight {
    pub direction: [f32; 3],
    pub color: [f32; 3],
    pub intensity: f32,
}

#[derive(Debug, Clone)]
pub struct RuntimeCamera {
    pub transform: Transform,
    pub fov_y: f32,
}

#[derive(Debug, Clone)]
pub struct LoadedScene {
    pub meshes: Vec<RuntimeMesh>,
    pub lights: Vec<RuntimeLight>,
    pub camera: RuntimeCamera,
    pub player_spawn: Option<[f32; 3]>,
    pub ambient: ([f32; 3], f32),
}

pub fn load_scene(doc: &SceneDocument) -> Result<LoadedScene, LoadError> {
    let mut meshes = vec![];
    let mut lights = vec![];
    let mut camera = None;
    let mut player_spawn = None;

    for node in &doc.scene.nodes {
        match &node.node_type {
            NodeType::MeshInstance => {
                let mesh_id = node
                    .mesh
                    .as_deref()
                    .unwrap_or("builtin://cube");
                let base = builtin_mesh(mesh_id)
                    .ok_or_else(|| LoadError::UnsupportedMesh(mesh_id.to_string()))?;
                let color = albedo_from_material(node);
                meshes.push(RuntimeMesh {
                    node_id: node.id.clone(),
                    mesh: tint_mesh(base, color),
                    transform: node.transform.clone(),
                    color,
                });
            }
            NodeType::PrefabInstance => {
                let base = builtin_mesh("builtin://cube").unwrap();
                meshes.push(RuntimeMesh {
                    node_id: node.id.clone(),
                    mesh: tint_mesh(base, [0.4, 0.6, 0.9]),
                    transform: node.transform.clone(),
                    color: [0.4, 0.6, 0.9],
                });
            }
            NodeType::DirectionalLight => {
                let dir = rotation_to_forward(&node.transform);
                lights.push(RuntimeLight {
                    direction: dir,
                    color: [1.0, 0.96, 0.9],
                    intensity: 1.0,
                });
            }
            NodeType::Camera3D => {
                camera = Some(RuntimeCamera {
                    transform: node.transform.clone(),
                    fov_y: 60.0_f32.to_radians(),
                });
            }
            NodeType::Marker3D => {
                if node.tags.iter().any(|t| t == "spawn") {
                    player_spawn = Some(node.transform.position);
                }
            }
            _ => {}
        }
    }

    let camera = camera.unwrap_or(RuntimeCamera {
        transform: Transform {
            position: [0.0, 2.0, 5.0],
            rotation: [-15.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        },
        fov_y: 60.0_f32.to_radians(),
    });

    if lights.is_empty() {
        lights.push(RuntimeLight {
            direction: [-0.3, -1.0, -0.2],
            color: [1.0, 0.96, 0.9],
            intensity: 1.0,
        });
    }

    let ambient_color = hex_to_rgb(&doc.scene.environment.ambient.color);
    let ambient_intensity = doc.scene.environment.ambient.intensity;

    Ok(LoadedScene {
        meshes,
        lights,
        camera,
        player_spawn,
        ambient: (ambient_color, ambient_intensity),
    })
}

fn albedo_from_material(node: &SceneNode) -> [f32; 3] {
    if node.material.is_some() {
        [0.55, 0.45, 0.25]
    } else {
        [0.5, 0.5, 0.5]
    }
}

fn tint_mesh(mut mesh: MeshData, color: [f32; 3]) -> MeshData {
    for v in &mut mesh.vertices {
        v.color = color;
    }
    mesh
}

fn rotation_to_forward(transform: &Transform) -> [f32; 3] {
    let pitch = transform.rotation[0].to_radians();
    let yaw = transform.rotation[1].to_radians();
    let x = yaw.sin() * pitch.cos();
    let y = -pitch.sin();
    let z = yaw.cos() * pitch.cos();
    let v = glam::Vec3::new(x, y, z).normalize();
    [v.x, v.y, v.z]
}

fn hex_to_rgb(hex: &str) -> [f32; 3] {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return [0.25, 0.25, 0.31];
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(64) as f32 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(64) as f32 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(80) as f32 / 255.0;
    [r, g, b]
}

#[cfg(test)]
mod tests {
    use super::*;
    use age_core::schema::world3d::{NodeType, SceneNode};

    #[test]
    fn loads_floor_light_spawn_and_camera() {
        let mut doc = SceneDocument::default_empty();
        doc.scene.nodes = vec![
            SceneNode {
                id: "floor".into(),
                node_type: NodeType::MeshInstance,
                mesh: Some("builtin://plane".into()),
                transform: Transform {
                    scale: [10.0, 1.0, 10.0],
                    ..Default::default()
                },
                ..Default::default()
            },
            SceneNode {
                id: "sun".into(),
                node_type: NodeType::DirectionalLight,
                ..Default::default()
            },
            SceneNode {
                id: "cam".into(),
                node_type: NodeType::Camera3D,
                ..Default::default()
            },
            SceneNode {
                id: "spawn".into(),
                node_type: NodeType::Marker3D,
                transform: Transform {
                    position: [1.0, 0.0, 2.0],
                    ..Default::default()
                },
                tags: vec!["spawn".into(), "player".into()],
                ..Default::default()
            },
        ];

        let loaded = load_scene(&doc).unwrap();
        assert_eq!(loaded.meshes.len(), 1);
        assert_eq!(loaded.lights.len(), 1);
        assert_eq!(loaded.player_spawn, Some([1.0, 0.0, 2.0]));
    }
}
