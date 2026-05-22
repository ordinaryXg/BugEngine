use std::path::{Path, PathBuf};

use age_core::project::Project;
use serde::{Deserialize, Serialize};

use crate::html::{export_html, ExportError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportTargetId {
    Html,
    Desktop,
    Mobile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExportStatus {
    Ready,
    Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub ok: bool,
    pub artifact_path: Option<PathBuf>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportTarget {
    pub id: ExportTargetId,
    pub status: ExportStatus,
    pub label: String,
}

impl ExportTarget {
    pub fn build(
        &self,
        project: &Project,
        out_dir: impl AsRef<Path>,
    ) -> Result<BuildResult, ExportError> {
        match (self.id, self.status) {
            (ExportTargetId::Html, ExportStatus::Ready) => {
                let path = export_html(project, out_dir)?;
                Ok(BuildResult {
                    ok: true,
                    artifact_path: Some(path),
                    error: None,
                })
            }
            (ExportTargetId::Desktop, ExportStatus::Placeholder) => Ok(BuildResult {
                ok: false,
                artifact_path: None,
                error: Some("Desktop 导出即将支持，请先使用 HTML 导出".into()),
            }),
            (ExportTargetId::Mobile, ExportStatus::Placeholder) => Ok(BuildResult {
                ok: false,
                artifact_path: None,
                error: Some("Mobile 导出即将支持，请先使用 HTML 导出".into()),
            }),
            _ => Ok(BuildResult {
                ok: false,
                artifact_path: None,
                error: Some(format!("export target {:?} is not available", self.id)),
            }),
        }
    }
}

pub fn all_targets() -> Vec<ExportTarget> {
    vec![
        ExportTarget {
            id: ExportTargetId::Html,
            status: ExportStatus::Ready,
            label: "HTML".into(),
        },
        ExportTarget {
            id: ExportTargetId::Desktop,
            status: ExportStatus::Placeholder,
            label: "Desktop（即将支持）".into(),
        },
        ExportTarget {
            id: ExportTargetId::Mobile,
            status: ExportStatus::Placeholder,
            label: "Mobile（即将支持）".into(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use age_core::project::Project;

    #[test]
    fn desktop_placeholder_returns_error_message() {
        let target = all_targets()
            .into_iter()
            .find(|t| t.id == ExportTargetId::Desktop)
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let project = Project::create_new(dir.path(), "test").unwrap();
        let result = target.build(&project, "dist/desktop").unwrap();
        assert!(!result.ok);
        assert_eq!(
            result.error.as_deref(),
            Some("Desktop 导出即将支持，请先使用 HTML 导出")
        );
    }

    #[test]
    fn mobile_placeholder_returns_error_message() {
        let target = all_targets()
            .into_iter()
            .find(|t| t.id == ExportTargetId::Mobile)
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let project = Project::create_new(dir.path(), "test").unwrap();
        let result = target.build(&project, "dist/mobile").unwrap();
        assert!(!result.ok);
        assert_eq!(
            result.error.as_deref(),
            Some("Mobile 导出即将支持，请先使用 HTML 导出")
        );
    }
}
