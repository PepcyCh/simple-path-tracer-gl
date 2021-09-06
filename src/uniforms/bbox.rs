use cgmath::Point3;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Bbox {
    p_min: [f32; 4],
    p_max: [f32; 4],
}

impl Bbox {
    pub fn new(p_min: Point3<f32>, p_max: Point3<f32>) -> Self {
        Self {
            p_min: [p_min.x, p_min.y, p_min.z, 1.0],
            p_max: [p_max.x, p_max.y, p_max.z, 1.0],
        }
    }
}
