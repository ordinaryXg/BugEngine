use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::schema::ui::UiDocument;
use crate::schema::world3d::SceneDocument;

#[derive(Debug, Clone)]
pub struct ToolContext {
    pub scene: Arc<RwLock<SceneDocument>>,
    pub ui: Arc<RwLock<UiDocument>>,
    pub project_root: Option<PathBuf>,
}

impl ToolContext {
    pub fn new(scene: SceneDocument, ui: UiDocument) -> Self {
        Self {
            scene: Arc::new(RwLock::new(scene)),
            ui: Arc::new(RwLock::new(ui)),
            project_root: None,
        }
    }

    pub fn with_project_root(scene: SceneDocument, ui: UiDocument, project_root: PathBuf) -> Self {
        Self {
            scene: Arc::new(RwLock::new(scene)),
            ui: Arc::new(RwLock::new(ui)),
            project_root: Some(project_root),
        }
    }
}

impl Default for ToolContext {
    fn default() -> Self {
        Self::new(SceneDocument::default_empty(), UiDocument::default_empty())
    }
}
