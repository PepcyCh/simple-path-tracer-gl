use cgmath::{Point3, Vector3};

#[derive(Copy, Clone)]
pub struct MeshVertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
}

pub struct TriangleMesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub mesh_index: u32,
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            position: cgmath::Point3::new(0.0, 0.0, 0.0),
            normal: cgmath::Vector3::unit_z(),
        }
    }
}

impl TriangleMesh {
    pub fn new(vertices: Vec<MeshVertex>, indices: Vec<u32>, mesh_index: u32) -> Self {
        Self {
            vertices,
            indices,
            mesh_index,
        }
    }

    pub fn position(&self, index: usize) -> cgmath::Point3<f32> {
        self.vertices[index].position
    }
}
