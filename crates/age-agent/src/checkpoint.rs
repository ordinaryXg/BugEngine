use std::path::{Path, PathBuf};

use age_core::schema::ui::UiDocument;
use age_core::schema::world3d::SceneDocument;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CheckpointError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("checkpoint not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMeta {
    pub id: String,
    pub created_at: String,
}

pub struct CheckpointManager {
    project_root: PathBuf,
}

impl CheckpointManager {
    pub fn new(project_root: impl AsRef<Path>) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
        }
    }

    fn checkpoints_dir(&self) -> PathBuf {
        self.project_root.join(".checkpoints")
    }

    pub fn create(
        &self,
        scene: &SceneDocument,
        ui: &UiDocument,
    ) -> Result<String, CheckpointError> {
        std::fs::create_dir_all(self.checkpoints_dir())?;
        let id = self.next_checkpoint_id()?;
        let dir = self.checkpoints_dir().join(&id);
        std::fs::create_dir_all(&dir)?;

        let meta = CheckpointMeta {
            id: id.clone(),
            created_at: Utc::now().to_rfc3339(),
        };
        std::fs::write(
            dir.join("meta.json"),
            serde_json::to_string_pretty(&meta)?,
        )?;
        std::fs::create_dir_all(dir.join("scenes"))?;
        std::fs::write(
            dir.join("scenes/main.scene.json"),
            serde_json::to_string_pretty(scene)?,
        )?;
        std::fs::create_dir_all(dir.join("ui"))?;
        std::fs::write(
            dir.join("ui/hud.ui.json"),
            serde_json::to_string_pretty(ui)?,
        )?;
        Ok(id)
    }

    pub fn restore(&self, id: &str) -> Result<(SceneDocument, UiDocument), CheckpointError> {
        let dir = self.checkpoints_dir().join(id);
        if !dir.exists() {
            return Err(CheckpointError::NotFound(id.to_string()));
        }
        let scene: SceneDocument = serde_json::from_str(&std::fs::read_to_string(
            dir.join("scenes/main.scene.json"),
        )?)?;
        let ui: UiDocument = serde_json::from_str(&std::fs::read_to_string(
            dir.join("ui/hud.ui.json"),
        )?)?;
        Ok((scene, ui))
    }

    pub fn list(&self) -> Result<Vec<CheckpointMeta>, CheckpointError> {
        let dir = self.checkpoints_dir();
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut items = vec![];
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let meta_path = entry.path().join("meta.json");
                if meta_path.exists() {
                    let meta: CheckpointMeta =
                        serde_json::from_str(&std::fs::read_to_string(meta_path)?)?;
                    items.push(meta);
                }
            }
        }
        items.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(items)
    }

    fn next_checkpoint_id(&self) -> Result<String, CheckpointError> {
        let existing = self.list()?;
        let next = existing.len() + 1;
        Ok(format!("cp_{next:04}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use age_core::schema::world3d::SceneDocument;

    #[test]
    fn create_modify_restore_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let manager = CheckpointManager::new(dir.path());

        let scene = SceneDocument::default_empty();
        let ui = UiDocument::default_empty();
        let id = manager.create(&scene, &ui).unwrap();

        let mut modified = scene.clone();
        modified.scene.metadata.name = "modified".into();
        manager.create(&modified, &ui).unwrap();

        let (restored_scene, _) = manager.restore(&id).unwrap();
        assert_eq!(restored_scene.scene.metadata.name, "untitled");
    }
}
