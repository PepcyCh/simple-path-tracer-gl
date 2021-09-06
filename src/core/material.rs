pub struct Material {
    pub albedo: [f32; 3],
    pub ior: f32,
    pub roughness: f32,
    pub metallic: f32,
    pub is_translucent: bool,
}

impl Material {
    pub fn new(
        albedo: [f32; 3],
        ior: f32,
        roughness: f32,
        metallic: f32,
        is_translucent: bool,
    ) -> Self {
        Self {
            albedo,
            ior,
            roughness: roughness * roughness,
            metallic,
            is_translucent,
        }
    }
}
