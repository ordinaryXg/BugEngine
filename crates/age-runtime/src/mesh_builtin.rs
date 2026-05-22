use crate::vertex::Vertex;

#[derive(Debug, Clone)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub fn builtin_mesh(id: &str) -> Option<MeshData> {
    match id {
        "builtin://cube" => Some(cube()),
        "builtin://plane" => Some(plane()),
        "builtin://sphere" => Some(sphere()),
        "builtin://cylinder" => Some(cylinder()),
        "builtin://ramp" => Some(ramp()),
        "builtin://wall_segment" => Some(wall_segment()),
        _ => None,
    }
}

fn cube() -> MeshData {
    let mut vertices = Vec::with_capacity(36);
    let faces: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
        ([0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.5, 0.5, 0.5]),
        ([0.0, 0.0, -1.0], [0.0, 0.0, -1.0], [0.5, 0.5, 0.5]),
        ([1.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 0.5, 0.5]),
        ([-1.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [0.5, 0.5, 0.5]),
        ([0.0, 1.0, 0.0], [0.0, 1.0, 0.0], [0.5, 0.5, 0.5]),
        ([0.0, -1.0, 0.0], [0.0, -1.0, 0.0], [0.5, 0.5, 0.5]),
    ];
    let half = 0.5;
    for (normal, _, color) in faces {
        let (t1, t2) = tangent_basis(normal);
        let corners = [
            -t1 * half - t2 * half,
            t1 * half - t2 * half,
            t1 * half + t2 * half,
            -t1 * half - t2 * half,
            t1 * half + t2 * half,
            -t1 * half + t2 * half,
        ];
        for c in corners {
            vertices.push(Vertex {
                position: [c.x, c.y, c.z],
                normal,
                color,
            });
        }
    }
    let indices: Vec<u16> = (0..vertices.len() as u16).collect();
    MeshData { vertices, indices }
}

fn plane() -> MeshData {
    let y = 0.0;
    let vertices = vec![
        Vertex {
            position: [-0.5, y, -0.5],
            normal: [0.0, 1.0, 0.0],
            color: [0.55, 0.45, 0.25],
        },
        Vertex {
            position: [0.5, y, -0.5],
            normal: [0.0, 1.0, 0.0],
            color: [0.55, 0.45, 0.25],
        },
        Vertex {
            position: [0.5, y, 0.5],
            normal: [0.0, 1.0, 0.0],
            color: [0.55, 0.45, 0.25],
        },
        Vertex {
            position: [-0.5, y, 0.5],
            normal: [0.0, 1.0, 0.0],
            color: [0.55, 0.45, 0.25],
        },
    ];
    MeshData {
        indices: vec![0, 1, 2, 0, 2, 3],
        vertices,
    }
}

fn sphere() -> MeshData {
    cube()
}

fn cylinder() -> MeshData {
    cube()
}

fn ramp() -> MeshData {
    let vertices = vec![
        Vertex {
            position: [-0.5, 0.0, -0.5],
            normal: [0.0, 0.7, 0.7],
            color: [0.5, 0.5, 0.5],
        },
        Vertex {
            position: [0.5, 0.0, -0.5],
            normal: [0.0, 0.7, 0.7],
            color: [0.5, 0.5, 0.5],
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            normal: [0.0, 0.7, 0.7],
            color: [0.5, 0.5, 0.5],
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            normal: [0.0, 0.7, 0.7],
            color: [0.5, 0.5, 0.5],
        },
    ];
    MeshData {
        indices: vec![0, 1, 2, 0, 2, 3],
        vertices,
    }
}

fn wall_segment() -> MeshData {
    cube()
}

fn tangent_basis(normal: [f32; 3]) -> (glam::Vec3, glam::Vec3) {
    let n = glam::Vec3::from(normal);
    let up = if n.y.abs() > 0.99 {
        glam::Vec3::X
    } else {
        glam::Vec3::Y
    };
    let t1 = n.cross(up).normalize();
    let t2 = n.cross(t1).normalize();
    (t1, t2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_has_thirty_six_vertices() {
        let mesh = cube();
        assert_eq!(mesh.vertices.len(), 36);
    }

    #[test]
    fn plane_has_four_vertices() {
        let mesh = plane();
        assert_eq!(mesh.vertices.len(), 4);
    }
}
