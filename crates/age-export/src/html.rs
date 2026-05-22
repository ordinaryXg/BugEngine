use std::fs;
use std::path::{Path, PathBuf};

use age_core::project::Project;
use age_core::schema::validate::{validate_scene, validate_ui};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExportError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("validation error: {0}")]
    Validation(#[from] age_core::schema::validate::ValidationError),
    #[error("project error: {0}")]
    Project(#[from] age_core::project::ProjectError),
    #[error("wasm build failed: {0}")]
    WasmBuild(String),
    #[error("missing wasm package at {0}; run wasm-pack build in crates/age-runtime")]
    MissingWasmPkg(PathBuf),
    #[error("missing runtime template: {0}")]
    MissingTemplate(PathBuf),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExportManifest {
    pub engine_version: String,
    pub export_target: String,
    pub scene: String,
    pub ui: Vec<String>,
    pub built_at: String,
}

pub fn export_html(project: &Project, out_dir: impl AsRef<Path>) -> Result<PathBuf, ExportError> {
    let out_dir = out_dir.as_ref().to_path_buf();
    validate_scene(&project.scene)?;
    validate_ui(&project.ui)?;

    if out_dir.exists() {
        fs::remove_dir_all(&out_dir)?;
    }
    fs::create_dir_all(&out_dir)?;

    let workspace_root = find_workspace_root()?;
    ensure_wasm_pkg(&workspace_root)?;

    copy_runtime_web(&workspace_root, &out_dir)?;
    copy_project_assets(project, &out_dir)?;
    write_manifest(project, &out_dir)?;

    Ok(out_dir)
}

fn find_workspace_root() -> Result<PathBuf, ExportError> {
    let mut dir = std::env::current_dir()?;
    for _ in 0..8 {
        if dir.join("Cargo.toml").exists() && dir.join("runtime-web").exists() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(ExportError::MissingTemplate(PathBuf::from("runtime-web")))
}

fn ensure_wasm_pkg(workspace_root: &Path) -> Result<(), ExportError> {
    let pkg_dir = workspace_root.join("runtime-web/pkg");
    let wasm_file = pkg_dir.join("age_runtime_bg.wasm");
    if wasm_file.exists() {
        return Ok(());
    }

    let status = std::process::Command::new("wasm-pack")
        .args([
            "build",
            "--target",
            "web",
            "--out-dir",
            "../../runtime-web/pkg",
        ])
        .current_dir(workspace_root.join("crates/age-runtime"))
        .status()?;

    if !status.success() {
        return Err(ExportError::WasmBuild(
            "wasm-pack build failed; install wasm-pack and wasm32 target".into(),
        ));
    }
    Ok(())
}

fn copy_runtime_web(workspace_root: &Path, out_dir: &Path) -> Result<(), ExportError> {
    let runtime_web = workspace_root.join("runtime-web");
    copy_file_if_exists(&runtime_web.join("index.html"), &out_dir.join("index.html"))?;
    copy_file_if_exists(&runtime_web.join("bootstrap.js"), &out_dir.join("bootstrap.js"))?;
    copy_dir_all(&runtime_web.join("pkg"), &out_dir.join("pkg"))?;
    Ok(())
}

fn copy_project_assets(project: &Project, out_dir: &Path) -> Result<(), ExportError> {
    let scene_rel = &project.manifest.main_scene;
    let scene_src = project.root.join(scene_rel);
    let scene_dst = out_dir.join(scene_rel);
    if scene_src.exists() {
        if let Some(parent) = scene_dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&scene_src, &scene_dst)?;
    }

    for ui_rel in &project.manifest.main_ui {
        let ui_src = project.root.join(ui_rel);
        let ui_dst = out_dir.join(ui_rel);
        if ui_src.exists() {
            if let Some(parent) = ui_dst.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&ui_src, &ui_dst)?;
        }
    }

    for subdir in ["prefabs", "materials", "assets"] {
        let src = project.root.join(subdir);
        if src.exists() {
            copy_dir_all(&src, &out_dir.join(subdir))?;
        }
    }

    Ok(())
}

fn write_manifest(project: &Project, out_dir: &Path) -> Result<(), ExportError> {
    let manifest = ExportManifest {
        engine_version: project.manifest.engine_version.clone(),
        export_target: "html".into(),
        scene: project.manifest.main_scene.clone(),
        ui: project.manifest.main_ui.clone(),
        built_at: Utc::now().to_rfc3339(),
    };
    fs::write(
        out_dir.join("manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    Ok(())
}

fn copy_file_if_exists(src: &Path, dst: &Path) -> Result<(), ExportError> {
    if src.exists() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
    } else {
        return Err(ExportError::MissingTemplate(src.to_path_buf()));
    }
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), ExportError> {
    if !src.exists() {
        return Err(ExportError::MissingWasmPkg(src.to_path_buf()));
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use age_core::project::Project;

    #[test]
    fn export_default_project_produces_html_bundle() {
        let workspace = find_workspace_root().expect("workspace root");
        let project = Project::load(workspace.join("templates/default-project")).unwrap();
        let out = tempfile::tempdir().unwrap();
        let result = export_html(&project, out.path()).unwrap();

        assert!(result.join("index.html").exists());
        assert!(result.join("bootstrap.js").exists());
        assert!(result.join("manifest.json").exists());
        assert!(result.join("pkg/age_runtime_bg.wasm").exists());
        assert!(result.join("scenes/main.scene.json").exists());
        assert!(result.join("ui/hud.ui.json").exists());

        let manifest: ExportManifest =
            serde_json::from_str(&fs::read_to_string(result.join("manifest.json")).unwrap()).unwrap();
        assert_eq!(manifest.export_target, "html");
    }
}
