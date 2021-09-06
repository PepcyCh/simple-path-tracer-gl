use std::rc::Rc;

use gl::types::*;
use uuid::Uuid;

macro_rules! resource_defination {
    ( $( ( $struct_name:ident, $info_name:ty ) ),+ $(,)? ) => {
        $(
            pub struct $struct_name {
                pub(crate) id: Uuid,
                pub info: $info_name,
            }

            impl $struct_name {
                pub(crate) fn new(info: $info_name) -> Self {
                    Self {
                        id: Uuid::new_v4(),
                        info,
                    }
                }
            }

            impl std::hash::Hash for $struct_name {
                fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                    self.id.hash(state);
                }
            }
        )+
    };
}

pub struct BufferInfo {
    pub size: u32,
}

pub struct OpenglBuffer {
    pub id: GLuint,
}

pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
    pub mips: u8,
    pub samples: u8,
    pub format: GLenum,
    pub ty: GLenum,
}

impl TextureInfo {
    pub fn max_mips(&self) -> u8 {
        log2_floor(self.width).max(log2_floor(self.height)) as u8
    }
}

pub struct OpenglTexture {
    pub id: GLuint,
    pub image_type: GLenum,
    pub internal_format: GLenum,
    pub pixel_format: GLenum,
    pub pixel_type: GLenum,
}

pub struct SamplerInfo {
    pub filter_min: GLenum,
    pub filter_mag: GLenum,
    pub address_u: GLenum,
    pub address_v: GLenum,
    pub address_w: GLenum,
}

pub struct OpenglSampler {
    pub id: GLuint,
}

pub struct ShaderInfo {
    pub source: String,
    pub stage: GLuint,
}

pub struct OpenglShader {
    pub id: GLuint,
    pub stage: GLuint,
}

pub struct ComputePipelineInfo {
    pub shader: Rc<Shader>,
}

pub struct OpenglComputePipeline {
    pub program: GLuint,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VertexAttributeFormat {
    Float,
    Vec2,
    Vec3,
    Vec4,
}

pub struct VertexAttribute {
    pub index: u32,
    pub binding: u32,
    pub format: VertexAttributeFormat,
    pub offset: u32,
}

pub struct VertexBinding {
    pub index: u32,
    pub stride: u32,
    pub is_per_instance: bool,
}

pub struct GraphicsShaders {
    pub vertex: Rc<Shader>,
    pub fragment: Rc<Shader>,
}

pub struct GraphicsPipelineInfo {
    pub shaders: GraphicsShaders,
    pub vertex_attributes: Vec<VertexAttribute>,
    pub vertex_bindings: Vec<VertexBinding>,
    pub primitive_topology: GLenum,
}

pub struct OpenglGraphicsPipeline {
    pub program: GLuint,
    pub vao: GLuint,
}

resource_defination! {
    (Buffer, BufferInfo),
    (Texture, TextureInfo),
    (Sampler, SamplerInfo),
    (Shader, ShaderInfo),
    (ComputePipeline, ComputePipelineInfo),
    (GraphicsPipeline, GraphicsPipelineInfo),
}

fn log2_floor(x: u32) -> u32 {
    32 - x.leading_zeros()
}
