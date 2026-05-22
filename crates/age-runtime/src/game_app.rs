use age_core::schema::world3d::SceneDocument;
use glam::{Mat4, Vec3, Vec4};

use crate::input::InputState;
use crate::physics_simple::colliders_from_scene;
use crate::player_controller::ThirdPersonController;
use crate::scene_loader::{load_scene, LoadedScene};

#[derive(Debug)]
pub struct GameApp {
    pub scene: LoadedScene,
    pub controller: ThirdPersonController,
    pub input: InputState,
    pub colliders: Vec<crate::physics_simple::Collider>,
}

impl GameApp {
    pub fn from_scene_document(doc: &SceneDocument) -> Result<Self, crate::scene_loader::LoadError> {
        let scene = load_scene(doc)?;
        let spawn = scene.player_spawn.unwrap_or([0.0, 0.5, 0.0]);
        let colliders = colliders_from_scene(&scene);
        Ok(Self {
            scene,
            controller: ThirdPersonController::with_spawn(spawn),
            input: InputState::default(),
            colliders,
        })
    }

    pub fn from_scene_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let doc: SceneDocument = serde_json::from_str(json)?;
        Ok(Self::from_scene_document(&doc)?)
    }

    pub fn update(&mut self, dt: f32) {
        self.controller.update(&self.input, dt, &self.colliders);
        self.input.clear_frame();
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let eye = Vec3::from(self.controller.camera_eye());
        let target = Vec3::from(self.controller.camera_target());
        let up = Vec3::Y;
        Mat4::look_at_rh(eye, target, up) * Mat4::perspective_rh(60.0_f32.to_radians(), aspect, 0.1, 200.0)
    }

    pub fn model_matrix(transform: &age_core::schema::transform::Transform) -> Mat4 {
        let translation = Mat4::from_translation(Vec3::from(transform.position));
        let rot_x = Mat4::from_rotation_x(transform.rotation[0].to_radians());
        let rot_y = Mat4::from_rotation_y(transform.rotation[1].to_radians());
        let rot_z = Mat4::from_rotation_z(transform.rotation[2].to_radians());
        let scale = Mat4::from_scale(Vec3::from(transform.scale));
        translation * rot_y * rot_x * rot_z * scale
    }

    pub fn light_dir(&self) -> Vec4 {
        let light = self.scene.lights.first().cloned().unwrap_or(crate::scene_loader::RuntimeLight {
            direction: [-0.3, -1.0, -0.2],
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
        });
        Vec4::new(
            light.direction[0],
            light.direction[1],
            light.direction[2],
            light.intensity,
        )
    }

    pub fn ambient(&self) -> Vec4 {
        let (rgb, intensity) = self.scene.ambient;
        Vec4::new(rgb[0], rgb[1], rgb[2], intensity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_and_updates() {
        let json = include_str!("../../../templates/default-project/scenes/main.scene.json");
        let mut app = GameApp::from_scene_json(json).unwrap();
        app.input.forward = true;
        app.update(0.5);
    }
}
