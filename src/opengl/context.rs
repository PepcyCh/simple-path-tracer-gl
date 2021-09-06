use std::{collections::HashMap, ops::Range, rc::Rc};

use gl::types::*;
use uuid::Uuid;

use crate::opengl::*;

pub struct OpenglContext {
    buffer_map: HashMap<Uuid, OpenglBuffer>,
    texture_map: HashMap<Uuid, OpenglTexture>,
    sampler_map: HashMap<Uuid, OpenglSampler>,
    shaders_map: HashMap<Uuid, OpenglShader>,
    graphics_pipeline_map: HashMap<Uuid, OpenglGraphicsPipeline>,
    compute_pipeline_map: HashMap<Uuid, OpenglComputePipeline>,
    state: OpenglState,
}

#[derive(Default)]
struct OpenglState {
    compute_pipeline: Option<Rc<ComputePipeline>>,
    graphics_pipeline: Option<Rc<GraphicsPipeline>>,
    index_buffer_type: GLenum,
}

impl OpenglContext {
    pub fn new() -> Self {
        Self {
            buffer_map: HashMap::new(),
            texture_map: HashMap::new(),
            sampler_map: HashMap::new(),
            shaders_map: HashMap::new(),
            graphics_pipeline_map: HashMap::new(),
            compute_pipeline_map: HashMap::new(),
            state: OpenglState::default(),
        }
    }
}

impl OpenglContext {
    pub fn create_buffer(&mut self, info: BufferInfo, data: Option<&[u8]>) -> Rc<Buffer> {
        let mut id: GLuint = 0;
        unsafe {
            gl::CreateBuffers(1, &mut id as *mut _);
            if let Some(data) = data {
                gl::NamedBufferStorage(id, info.size as _, data.as_ptr() as *const _, 0);
            } else {
                gl::NamedBufferStorage(id, info.size as _, std::ptr::null(), 0);
            }
        }

        let buffer = Rc::new(Buffer::new(info));
        let opengl_buffer = OpenglBuffer { id };

        self.buffer_map.insert(buffer.id, opengl_buffer);

        buffer
    }

    pub fn create_texture(&mut self, mut info: TextureInfo) -> Rc<Texture> {
        if info.mips == 0 {
            info.mips = info.max_mips();
        }

        let mut id: GLuint = 0;
        let internal_format = info.format;
        let (pixel_format, pixel_type) = utils::get_format_and_type(internal_format);
        let image_type = info.ty;
        unsafe {
            gl::CreateTextures(image_type, 1, &mut id as *mut _);
            if info.samples == 1 {
                gl::TextureStorage2D(
                    id,
                    info.mips as _,
                    internal_format,
                    info.width as _,
                    info.height as _,
                );
            } else {
                info.mips = 1;
                gl::TextureStorage2DMultisample(
                    id,
                    info.samples as _,
                    internal_format,
                    info.width as _,
                    info.height as _,
                    gl::TRUE as _,
                )
            }
        }

        let texture = Rc::new(Texture::new(info));
        let opengl_texture = OpenglTexture {
            id,
            image_type,
            internal_format,
            pixel_format,
            pixel_type,
        };

        self.texture_map.insert(texture.id, opengl_texture);

        texture
    }

    pub fn create_sampler(&mut self, info: SamplerInfo) -> Rc<Sampler> {
        let mut id: GLuint = 0;
        unsafe {
            gl::CreateSamplers(1, &mut id as *mut _);

            gl::SamplerParameteri(id, gl::TEXTURE_MAG_FILTER, info.filter_mag as _);
            gl::SamplerParameteri(id, gl::TEXTURE_MIN_FILTER, info.filter_min as _);

            gl::SamplerParameteri(id, gl::TEXTURE_WRAP_S, info.address_u as _);
            gl::SamplerParameteri(id, gl::TEXTURE_WRAP_T, info.address_v as _);
            gl::SamplerParameteri(id, gl::TEXTURE_WRAP_R, info.address_w as _);
        }

        let sampler = Rc::new(Sampler::new(info));
        let opengl_sampler = OpenglSampler { id };

        self.sampler_map.insert(sampler.id, opengl_sampler);

        sampler
    }

    pub fn create_shader(&mut self, info: ShaderInfo) -> Rc<Shader> {
        let id: GLuint;
        let stage = info.stage;
        unsafe {
            let source =
                std::ffi::CString::new(info.source.clone()).expect("invalid shader source");
            id = gl::CreateShaderProgramv(info.stage, 1, &source.as_ptr() as *const _);
        }

        let shader = Rc::new(Shader::new(info));
        let opengl_shader = OpenglShader { id, stage };

        self.shaders_map.insert(shader.id, opengl_shader);

        shader
    }

    pub fn create_graphics_pipeline(&mut self, info: GraphicsPipelineInfo) -> Rc<GraphicsPipeline> {
        let gl_vs = self.shaders_map.get(&info.shaders.vertex.id).unwrap();
        let gl_fs = self.shaders_map.get(&info.shaders.fragment.id).unwrap();

        let mut program: GLuint = 0;
        let mut vao: GLuint = 0;
        unsafe {
            gl::CreateProgramPipelines(1, &mut program as *mut _);
            gl::UseProgramStages(program, gl::VERTEX_SHADER_BIT, gl_vs.id);
            gl::UseProgramStages(program, gl::FRAGMENT_SHADER_BIT, gl_fs.id);

            gl::CreateVertexArrays(1, &mut vao as *mut _);
            for attrib in &info.vertex_attributes {
                gl::EnableVertexArrayAttrib(vao, attrib.index);
                let (size, ty) = utils::get_vertex_attribute_size_and_type(attrib.format);
                gl::VertexArrayAttribFormat(vao, attrib.index, size, ty, gl::FALSE, attrib.offset);
                gl::VertexArrayAttribBinding(vao, attrib.index, attrib.binding);
            }
        }

        let graphics_pipeline = Rc::new(GraphicsPipeline::new(info));
        let opengl_graphics_pipeline = OpenglGraphicsPipeline { program, vao };

        self.graphics_pipeline_map
            .insert(graphics_pipeline.id, opengl_graphics_pipeline);

        graphics_pipeline
    }

    pub fn create_compute_pipeline(&mut self, info: ComputePipelineInfo) -> Rc<ComputePipeline> {
        let gl_cs = self.shaders_map.get(&info.shader.id).unwrap();

        let mut program: GLuint = 0;
        unsafe {
            gl::CreateProgramPipelines(1, &mut program as *mut _);
            gl::UseProgramStages(program, gl::COMPUTE_SHADER_BIT, gl_cs.id);
        }

        let compute_pipeline = Rc::new(ComputePipeline::new(info));
        let opengl_compute_pipeline = OpenglComputePipeline { program };

        self.compute_pipeline_map
            .insert(compute_pipeline.id, opengl_compute_pipeline);

        compute_pipeline
    }

    pub fn bind_graphics_pipeline(&mut self, pipeline: &Rc<GraphicsPipeline>) {
        let gl_pipeline = self.graphics_pipeline_map.get(&pipeline.id).unwrap();

        self.state.graphics_pipeline = Some(pipeline.clone());

        unsafe {
            gl::BindProgramPipeline(gl_pipeline.program);
            gl::BindVertexArray(gl_pipeline.vao);
        }
    }

    pub fn bind_compute_pipeline(&mut self, pipeline: &Rc<ComputePipeline>) {
        let gl_pipeline = self.compute_pipeline_map.get(&pipeline.id).unwrap();

        self.state.compute_pipeline = Some(pipeline.clone());

        unsafe {
            gl::BindProgramPipeline(gl_pipeline.program);
        }
    }

    pub fn bind_vertex_buffer(&mut self, unit: u32, buf: &Rc<Buffer>) {
        let gl_buf = self.buffer_map.get(&buf.id).unwrap();
        let pipeline = self.state.graphics_pipeline.as_ref().unwrap();
        let gl_pipeline = self.graphics_pipeline_map.get(&pipeline.id).unwrap();
        let binding = pipeline
            .info
            .vertex_bindings
            .iter()
            .find(|binding| binding.index == unit)
            .unwrap();

        unsafe {
            gl::VertexArrayVertexBuffer(gl_pipeline.vao, unit, gl_buf.id, 0, binding.stride as _);
        }
    }

    pub fn bind_index_buffer(&mut self, buf: &Rc<Buffer>, ty: GLenum) {
        let gl_buf = self.buffer_map.get(&buf.id).unwrap();
        let pipeline = self.state.graphics_pipeline.as_ref().unwrap();
        let gl_pipeline = self.graphics_pipeline_map.get(&pipeline.id).unwrap();

        unsafe {
            gl::VertexArrayElementBuffer(gl_pipeline.vao, gl_buf.id);
        }

        self.state.index_buffer_type = ty;
    }

    pub fn bind_image(
        &mut self,
        unit: u32,
        tex: &Rc<Texture>,
        mip: u32,
        layer: Option<u32>,
        access: GLenum,
    ) {
        let gl_tex = self.texture_map.get(&tex.id).unwrap();

        unsafe {
            if let Some(layer) = layer {
                gl::BindImageTexture(
                    unit,
                    gl_tex.id,
                    mip as _,
                    gl::FALSE,
                    layer as _,
                    access,
                    gl_tex.internal_format,
                );
            } else {
                gl::BindImageTexture(
                    unit,
                    gl_tex.id,
                    mip as _,
                    gl::TRUE,
                    0,
                    access,
                    gl_tex.internal_format,
                );
            }
        }
    }

    pub fn bind_texture(&mut self, unit: u32, tex: &Rc<Texture>) {
        let gl_tex = self.texture_map.get(&tex.id).unwrap();

        unsafe {
            gl::BindTextureUnit(unit, gl_tex.id);
        }
    }

    pub fn bind_sampler(&mut self, unit: u32, tex: &Rc<Sampler>) {
        let gl_samp = self.sampler_map.get(&tex.id).unwrap();

        unsafe {
            gl::BindSampler(unit, gl_samp.id);
        }
    }

    pub fn bind_uniform_buffer(&mut self, unit: u32, buf: &Rc<Buffer>, range: Option<Range<u32>>) {
        let gl_buf = self.buffer_map.get(&buf.id).unwrap();

        unsafe {
            if let Some(range) = range {
                gl::BindBufferRange(
                    gl::UNIFORM_BUFFER,
                    unit,
                    gl_buf.id,
                    range.start as _,
                    range.end as _,
                );
            } else {
                gl::BindBufferRange(gl::UNIFORM_BUFFER, unit, gl_buf.id, 0, buf.info.size as _);
            }
        }
    }

    pub fn bind_shader_storage_buffer(
        &mut self,
        unit: u32,
        buf: &Rc<Buffer>,
        range: Option<Range<u32>>,
    ) {
        let gl_buf = self.buffer_map.get(&buf.id).unwrap();

        unsafe {
            if let Some(range) = range {
                gl::BindBufferRange(
                    gl::SHADER_STORAGE_BUFFER,
                    unit,
                    gl_buf.id,
                    range.start as _,
                    range.end as _,
                );
            } else {
                gl::BindBufferRange(
                    gl::SHADER_STORAGE_BUFFER,
                    unit,
                    gl_buf.id,
                    0,
                    buf.info.size as _,
                );
            }
        }
    }

    pub fn draw(&self, num_vertices: u32) {
        let pipeline = self.state.graphics_pipeline.as_ref().unwrap();

        unsafe {
            gl::DrawArrays(pipeline.info.primitive_topology, 0, num_vertices as _);
        }
    }

    pub fn draw_indexed(&self, num_indices: u32) {
        let pipeline = self.state.graphics_pipeline.as_ref().unwrap();

        unsafe {
            gl::DrawElements(
                pipeline.info.primitive_topology,
                num_indices as _,
                self.state.index_buffer_type,
                std::ptr::null(),
            );
        }
    }

    pub fn dispatch_compute(&self, num_groups: (u32, u32, u32), wait: bool) {
        assert!(self.state.compute_pipeline.is_some());

        unsafe {
            gl::DispatchCompute(num_groups.0, num_groups.1, num_groups.2);

            if wait {
                gl::MemoryBarrier(gl::SHADER_IMAGE_ACCESS_BARRIER_BIT);
            }
        }
    }
}
