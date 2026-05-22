use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::schema::ui::UiDocument;
use crate::schema::validate::{validate_scene, validate_ui};
use crate::schema::world3d::SceneDocument;

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(#[from] crate::schema::validate::ValidationError),
    #[error("missing project manifest at {0}")]
    MissingManifest(PathBuf),
    #[error("missing file: {0}")]
    MissingFile(PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectManifest {
    pub name: String,
    pub version: String,
    pub engine_version: String,
    pub main_scene: String,
    pub main_ui: Vec<String>,
}

impl Default for ProjectManifest {
    fn default() -> Self {
        Self {
            name: "untitled".into(),
            version: "0.1.0".into(),
            engine_version: "0.1.0".into(),
            main_scene: "scenes/main.scene.json".into(),
            main_ui: vec!["ui/hud.ui.json".into()],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
    pub manifest: ProjectManifest,
    pub scene: SceneDocument,
    pub ui: UiDocument,
}

impl Project {
    pub fn create_new(root: impl AsRef<Path>, name: &str) -> Result<Self, ProjectError> {
        let root = root.as_ref().to_path_buf();
        std::fs::create_dir_all(root.join("scenes"))?;
        std::fs::create_dir_all(root.join("ui"))?;
        std::fs::create_dir_all(root.join("prefabs"))?;
        std::fs::create_dir_all(root.join("materials"))?;
        std::fs::create_dir_all(root.join("assets/models"))?;

        let mut scene = SceneDocument::default_empty();
        scene.scene.metadata.name = name.to_string();

        let ui = UiDocument::default_empty();
        let manifest = ProjectManifest {
            name: name.to_string(),
            ..ProjectManifest::default()
        };

        let project = Self {
            root,
            manifest,
            scene,
            ui,
        };
        project.save()?;
        Ok(project)
    }

    pub fn load(root: impl AsRef<Path>) -> Result<Self, ProjectError> {
        let root = root.as_ref().to_path_buf();
        let manifest_path = root.join("project.json");
        if !manifest_path.exists() {
            return Err(ProjectError::MissingManifest(manifest_path));
        }

        let manifest: ProjectManifest = serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?;
        let scene_path = root.join(&manifest.main_scene);
        if !scene_path.exists() {
            return Err(ProjectError::MissingFile(scene_path));
        }
        let scene: SceneDocument = serde_json::from_str(&std::fs::read_to_string(&scene_path)?)?;

        let ui_path = root.join(
            manifest
                .main_ui
                .first()
                .cloned()
                .unwrap_or_else(|| "ui/hud.ui.json".into()),
        );
        let ui = if ui_path.exists() {
            serde_json::from_str(&std::fs::read_to_string(&ui_path)?)?
        } else {
            UiDocument::default_empty()
        };

        validate_scene(&scene)?;
        validate_ui(&ui)?;

        Ok(Self {
            root,
            manifest,
            scene,
            ui,
        })
    }

    pub fn save(&self) -> Result<(), ProjectError> {
        validate_scene(&self.scene)?;
        validate_ui(&self.ui)?;

        std::fs::create_dir_all(self.root.join("scenes"))?;
        std::fs::create_dir_all(self.root.join("ui"))?;

        let manifest_path = self.root.join("project.json");
        std::fs::write(
            &manifest_path,
            serde_json::to_string_pretty(&self.manifest)?,
        )?;

        let scene_path = self.root.join(&self.manifest.main_scene);
        if let Some(parent) = scene_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            &scene_path,
            serde_json::to_string_pretty(&self.scene)?,
        )?;

        if let Some(ui_rel) = self.manifest.main_ui.first() {
            let ui_path = self.root.join(ui_rel);
            if let Some(parent) = ui_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(ui_path, serde_json::to_string_pretty(&self.ui)?)?;
        }

        Ok(())
    }

    pub fn tool_context(&self) -> crate::tool::context::ToolContext {
        crate::tool::context::ToolContext::with_project_root(
            self.scene.clone(),
            self.ui.clone(),
            self.root.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_load_project_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let project = Project::create_new(dir.path(), "demo_room").unwrap();
        project.save().unwrap();
        let loaded = Project::load(dir.path()).unwrap();
        assert_eq!(loaded.scene.scene.metadata.name, "demo_room");
        assert_eq!(loaded.manifest.name, "demo_room");
    }
}
