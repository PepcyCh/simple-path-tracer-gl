use cgmath::{Point3, Vector3};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 4],
    normal: [f32; 4],
}

impl Vertex {
    pub fn new(position: Point3<f32>, normal: Vector3<f32>) -> Self {
        Self {
            position: [position.x, position.y, position.z, 1.0],
            normal: [normal.x, normal.y, normal.z, 0.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    indices: [u32; 4],
    material_index: u32,
    object_index: u32,
    _pad: [f32; 2],
}

impl Triangle {
    pub fn new(indices: [usize; 3], material_index: u32, object_index: u32) -> Self {
        Self {
            indices: [indices[0] as u32, indices[1] as u32, indices[2] as u32, 0],
            material_index,
            object_index,
            _pad: [0.0, 0.0],
        }
    }
}
