use cgmath::{InnerSpace, Vector3};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    eye: [f32; 4],
    forward: [f32; 4],
    up: [f32; 4],
    right: [f32; 4],
    fov: f32,
    half_cot_half_fov: f32,
    _pad: [f32; 2],
}

impl Camera {
    pub fn new(eye: Vector3<f32>, forward: Vector3<f32>, up: Vector3<f32>, fov_deg: f32) -> Self {
        let forward = forward.normalize();
        let right = forward.cross(up).normalize();
        let up = right.cross(forward);
        let fov = fov_deg * std::f32::consts::PI / 180.0;

        Self {
            eye: [eye.x, eye.y, eye.z, 1.0],
            forward: [forward.x, forward.y, forward.z, 0.0],
            up: [up.x, up.y, up.z, 0.0],
            right: [right.x, right.y, right.z, 0.0],
            fov,
            half_cot_half_fov: 0.5 / (fov * 0.5).tan(),
            _pad: [0.0, 0.0],
        }
    }
}
