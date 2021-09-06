mod bbox;
mod bvhnode;
mod camera;
mod light;
mod material;
mod object;
mod triangle;

pub use bbox::*;
pub use bvhnode::*;
pub use camera::*;
pub use light::*;
pub use material::*;
pub use object::*;
pub use triangle::*;

/*
const BVH_NODES_COUNT: usize = 16;
const VERTICES_COUNT: usize = 32;
const TRIANGLES_COUNT: usize = 16;
const OBJECTS_COUNT: usize = 2;
const MATERIALS_COUNT: usize = 2;
const LIGHTS_COUNT: usize = 2;
*/

const BVH_NODES_COUNT: usize = 131072;
const VERTICES_COUNT: usize = 131072;
const TRIANGLES_COUNT: usize = 131072;
const OBJECTS_COUNT: usize = 1024;
const MATERIALS_COUNT: usize = 1024;
const LIGHTS_COUNT: usize = 1024;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct SceneUniform {
    pub bvh_nodes: [BvhNode; BVH_NODES_COUNT],
    pub vertices: [Vertex; VERTICES_COUNT],
    pub triangles: [Triangle; TRIANGLES_COUNT],
    pub objects: [SceneObject; OBJECTS_COUNT],
    pub materials: [Material; MATERIALS_COUNT],
    pub lights: [Light; LIGHTS_COUNT],
    pub lights_count: u32,
    pub max_depth: u32,
    _pad: [f32; 2],
}

unsafe impl bytemuck::Zeroable for SceneUniform {}

unsafe impl bytemuck::Pod for SceneUniform {}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VariableUniform {
    pub camera: Camera,
    pub curr_light_index: u32,
    _pad: [f32; 3],
}
