#[derive(Debug, Clone)]
pub struct Collider {
    pub center: [f32; 3],
    pub half_extents: [f32; 3],
}

pub fn resolve_aabb_move(
    position: &mut [f32; 3],
    delta: [f32; 3],
    colliders: &[Collider],
) {
    let try_axis = |pos: &mut [f32; 3], axis: usize, delta: f32, colliders: &[Collider]| {
        if delta.abs() < f32::EPSILON {
            return;
        }
        pos[axis] += delta;
        for c in colliders {
            if overlaps(pos, c) {
                if delta > 0.0 {
                    pos[axis] = c.center[axis] - c.half_extents[axis] - 0.35;
                } else {
                    pos[axis] = c.center[axis] + c.half_extents[axis] + 0.35;
                }
            }
        }
    };

    try_axis(position, 0, delta[0], colliders);
    try_axis(position, 1, delta[1], colliders);
    try_axis(position, 2, delta[2], colliders);
}

fn overlaps(player: &[f32; 3], c: &Collider) -> bool {
    let r = 0.35;
    player[0] + r > c.center[0] - c.half_extents[0]
        && player[0] - r < c.center[0] + c.half_extents[0]
        && player[1] + 1.7 > c.center[1] - c.half_extents[1]
        && player[1] < c.center[1] + c.half_extents[1]
        && player[2] + r > c.center[2] - c.half_extents[2]
        && player[2] - r < c.center[2] + c.half_extents[2]
}

pub fn colliders_from_scene(scene: &crate::scene_loader::LoadedScene) -> Vec<Collider> {
    scene
        .meshes
        .iter()
        .map(|m| {
            let s = m.transform.scale;
            Collider {
                center: m.transform.position,
                half_extents: [s[0] / 2.0, s[1] / 2.0, s[2] / 2.0],
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_movement_into_collider() {
        let mut pos = [0.0, 0.0, 0.0];
        let colliders = vec![Collider {
            center: [1.0, 0.0, 0.0],
            half_extents: [0.5, 1.0, 0.5],
        }];
        resolve_aabb_move(&mut pos, [0.6, 0.0, 0.0], &colliders);
        assert!(pos[0] < 0.85);
    }
}
