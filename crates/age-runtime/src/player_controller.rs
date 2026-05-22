use crate::input::InputState;
use crate::physics_simple::{resolve_aabb_move, Collider};

#[derive(Debug, Clone)]
pub struct ThirdPersonController {
    pub position: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub camera_distance: f32,
    pub camera_height_offset: f32,
    pub move_speed: f32,
}

impl Default for ThirdPersonController {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            yaw: 0.0,
            pitch: -0.25,
            camera_distance: 3.5,
            camera_height_offset: 1.5,
            move_speed: 4.0,
        }
    }
}

impl ThirdPersonController {
    pub fn with_spawn(spawn: [f32; 3]) -> Self {
        Self {
            position: spawn,
            ..Default::default()
        }
    }

    pub fn update(&mut self, input: &InputState, dt: f32, colliders: &[Collider]) {
        if input.rmb_down {
            self.yaw += input.mouse_delta.0 * 0.005;
            self.pitch = (self.pitch - input.mouse_delta.1 * 0.005).clamp(-1.2, 0.3);
        }

        let forward = glam::Vec3::new(self.yaw.sin(), 0.0, self.yaw.cos());
        let right = glam::Vec3::new(forward.z, 0.0, -forward.x);
        let mut move_dir = glam::Vec3::ZERO;
        if input.forward {
            move_dir += forward;
        }
        if input.backward {
            move_dir -= forward;
        }
        if input.left {
            move_dir -= right;
        }
        if input.right {
            move_dir += right;
        }
        if move_dir.length_squared() > 0.0 {
            move_dir = move_dir.normalize();
            let delta = move_dir * self.move_speed * dt;
            resolve_aabb_move(
                &mut self.position,
                [delta.x, delta.y, delta.z],
                colliders,
            );
        }
    }

    pub fn camera_eye(&self) -> [f32; 3] {
        let target = glam::Vec3::from(self.position) + glam::Vec3::new(0.0, self.camera_height_offset, 0.0);
        let offset = glam::Vec3::new(
            self.yaw.sin() * self.pitch.cos() * self.camera_distance,
            (-self.pitch).sin() * self.camera_distance,
            self.yaw.cos() * self.pitch.cos() * self.camera_distance,
        );
        let eye = target + offset;
        [eye.x, eye.y, eye.z]
    }

    pub fn camera_target(&self) -> [f32; 3] {
        let t = glam::Vec3::from(self.position) + glam::Vec3::new(0.0, self.camera_height_offset, 0.0);
        [t.x, t.y, t.z]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_forward_on_w_key() {
        let mut controller = ThirdPersonController::default();
        let input = InputState {
            forward: true,
            ..Default::default()
        };
        controller.update(&input, 0.5, &[]);
        assert!(controller.position[2].abs() > 0.0 || controller.position[0].abs() > 0.0);
    }
}
