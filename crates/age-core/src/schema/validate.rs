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
    use crate::schema::world3d::SceneDocument;

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
