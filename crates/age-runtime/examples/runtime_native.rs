use age_core::project::Project;

fn main() {
    let project_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "templates/default-project".to_string());
    let project = Project::load(&project_path).expect("load project");
    let scene_json = serde_json::to_string(&project.scene).expect("scene json");
    let game = age_runtime::GameApp::from_scene_json(&scene_json).expect("game app");
    age_runtime::game_loop::run_native(game);
}
