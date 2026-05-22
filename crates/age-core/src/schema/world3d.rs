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

impl Default for SceneNode {
    fn default() -> Self {
        Self {
            id: String::new(),
            node_type: NodeType::MeshInstance,
            transform: Transform::default(),
            tags: vec![],
            mesh: None,
            prefab: None,
            material: None,
            components: vec![],
        }
    }
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

fn default_ambient_color() -> String {
    "#404050".into()
}

fn default_ambient_intensity() -> f32 {
    0.3
}

impl Default for AmbientLight {
    fn default() -> Self {
        Self {
            color: default_ambient_color(),
            intensity: default_ambient_intensity(),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            ambient: AmbientLight::default(),
            background: default_background(),
            skybox: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SceneMetadata {
    pub name: String,
    #[serde(default = "default_units")]
    pub units: String,
}

fn default_units() -> String {
    "meters".into()
}

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
                metadata: SceneMetadata {
                    name: "untitled".into(),
                    units: "meters".into(),
                },
                environment: Environment::default(),
                nodes: vec![],
                materials: vec![],
            },
        }
    }

    pub fn new_node_id(prefix: &str) -> String {
        format!("{}_{}", prefix, &Uuid::new_v4().to_string()[..8])
    }
}
