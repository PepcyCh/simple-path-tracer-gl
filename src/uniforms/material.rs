#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Material {
    albedo_ior: [f32; 4],
    roughness: f32,
    metallic: f32,
    is_translucent: i32,
    _pad: f32,
}

impl Material {
    pub fn new(
        alebdo: [f32; 3],
        ior: f32,
        roughness: f32,
        metallic: f32,
        is_translucent: bool,
    ) -> Self {
        Self {
            albedo_ior: [alebdo[0], alebdo[1], alebdo[2], ior],
            roughness,
            metallic,
            is_translucent: is_translucent as _,
            _pad: 0.0,
        }
    }
}
