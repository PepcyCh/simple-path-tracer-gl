#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    pos_or_dir: [f32; 4],
    strength: [f32; 4],
}

impl Light {
    pub fn point(position: [f32; 3], strength: [f32; 3]) -> Self {
        Self {
            pos_or_dir: [position[0], position[1], position[2], 1.0],
            strength: [strength[0], strength[1], strength[2], 1.0],
        }
    }

    pub fn directional(direction: [f32; 3], strength: [f32; 3]) -> Self {
        let norm = (direction[0] * direction[0]
            + direction[1] * direction[1]
            + direction[2] * direction[2])
            .sqrt();
        Self {
            pos_or_dir: [
                direction[0] / norm,
                direction[1] / norm,
                direction[2] / norm,
                0.0,
            ],
            strength: [strength[0], strength[1], strength[2], 1.0],
        }
    }
}
