use std::sync::{Arc, RwLock};

use crate::schema::ui::UiDocument;
use crate::schema::world3d::SceneDocument;

#[derive(Debug, Clone)]
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

impl Default for ToolContext {
    fn default() -> Self {
        Self::new(SceneDocument::default_empty(), UiDocument::default_empty())
    }
}
