use cgmath::{Matrix, Matrix4, SquareMatrix};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SceneObject {
    model: [[f32; 4]; 4],
    model_iv: [[f32; 4]; 4],
}

impl SceneObject {
    pub fn new(model: Matrix4<f32>) -> Self {
        let model_iv = model.transpose().invert().unwrap();
        Self {
            model: *model.as_ref(),
            model_iv: *model_iv.as_ref(),
        }
    }
}
