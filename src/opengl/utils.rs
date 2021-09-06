use gl::types::*;

use super::VertexAttributeFormat;

pub fn get_format_and_type(internal_format: GLenum) -> (GLenum, GLenum) {
    match internal_format {
        gl::R8 => (gl::RED, gl::UNSIGNED_BYTE),
        gl::R8_SNORM => (gl::RED, gl::BYTE),
        gl::R8UI => (gl::RED, gl::UNSIGNED_BYTE),
        gl::R8I => (gl::RED, gl::BYTE),
        gl::R16UI => (gl::RED, gl::UNSIGNED_SHORT),
        gl::R16I => (gl::RED, gl::SHORT),
        gl::R16F => (gl::RED, gl::HALF_FLOAT),
        gl::RG8 => (gl::RG, gl::UNSIGNED_BYTE),
        gl::RG8_SNORM => (gl::RG, gl::BYTE),
        gl::RG8UI => (gl::RG, gl::UNSIGNED_BYTE),
        gl::RG8I => (gl::RG, gl::BYTE),
        gl::R32UI => (gl::RED, gl::UNSIGNED_INT),
        gl::R32I => (gl::RED, gl::INT),
        gl::R32F => (gl::RED, gl::FLOAT),
        gl::RG16UI => (gl::RG, gl::UNSIGNED_SHORT),
        gl::RG16I => (gl::RG, gl::SHORT),
        gl::RG16F => (gl::RG, gl::HALF_FLOAT),
        gl::RGBA8 => (gl::RG, gl::UNSIGNED_BYTE),
        gl::SRGB8_ALPHA8 => (gl::RGBA, gl::UNSIGNED_BYTE),
        gl::RGBA8_SNORM => (gl::RGBA, gl::BYTE),
        gl::RGBA8UI => (gl::RGBA, gl::UNSIGNED_BYTE),
        gl::RGBA8I => (gl::RGBA, gl::BYTE),
        gl::RGB10_A2 => (gl::RGBA, gl::UNSIGNED_INT_10_10_10_2),
        gl::R11F_G11F_B10F => (gl::RGB, gl::UNSIGNED_INT_10F_11F_11F_REV),
        gl::RG32UI => (gl::RG, gl::UNSIGNED_INT),
        gl::RG32I => (gl::RG, gl::INT),
        gl::RG32F => (gl::RG, gl::FLOAT),
        gl::RGBA16UI => (gl::RGBA, gl::UNSIGNED_SHORT),
        gl::RGBA16I => (gl::RGBA, gl::SHORT),
        gl::RGBA16F => (gl::RGBA, gl::HALF_FLOAT),
        gl::RGBA32UI => (gl::RGBA, gl::UNSIGNED_INT),
        gl::RGBA32I => (gl::RGBA, gl::INT),
        gl::RGBA32F => (gl::RGBA, gl::FLOAT),
        gl::DEPTH_COMPONENT32F => (gl::DEPTH_COMPONENT, gl::FLOAT),
        gl::DEPTH24_STENCIL8 => (gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8),
        gl::DEPTH32F_STENCIL8 => (gl::DEPTH_STENCIL, gl::FLOAT_32_UNSIGNED_INT_24_8_REV),
        _ => panic!("OpenGL, unsupported internal format"),
    }
}

pub fn get_vertex_attribute_size_and_type(format: VertexAttributeFormat) -> (GLint, GLenum) {
    match format {
        VertexAttributeFormat::Float => (1, gl::FLOAT),
        VertexAttributeFormat::Vec2 => (2, gl::FLOAT),
        VertexAttributeFormat::Vec3 => (3, gl::FLOAT),
        VertexAttributeFormat::Vec4 => (4, gl::FLOAT),
    }
}
