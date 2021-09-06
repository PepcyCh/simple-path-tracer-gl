use std::{cell::RefCell, rc::Rc};

use crate::{
    opengl::*,
    uniforms::{SceneUniform, VariableUniform},
};

pub struct OutputConfig {
    pub file: String,
    pub width: u32,
    pub height: u32,
    pub scale: u32,
}

pub struct Renderer {
    context: RefCell<OpenglContext>,
    pub output_config: OutputConfig,
    scene_uniform: Box<SceneUniform>,
    variable_uniform: VariableUniform,
    gl_resources: Option<GlResources>,
}

struct GlResources {
    pub scene_uniform_buffer: Rc<Buffer>,
    pub variable_uniform_buffer: Rc<Buffer>,
    pub traced_img: Rc<Texture>,
    pub traced_img_sampler: Rc<Sampler>,
    pub trace_pipeline: Rc<ComputePipeline>,
    pub post_pipeline: Rc<GraphicsPipeline>,
}

impl Renderer {
    pub fn new(
        output_config: OutputConfig,
        scene_uniform: Box<SceneUniform>,
        variable_uniform: VariableUniform,
    ) -> Self {
        Self {
            context: RefCell::new(OpenglContext::new()),
            output_config,
            scene_uniform,
            variable_uniform,
            gl_resources: None,
        }
    }

    pub fn init(&mut self) {
        let info = BufferInfo {
            size: std::mem::size_of::<SceneUniform>() as u32,
        };
        let scene_uniform_buffer = self
            .context
            .borrow_mut()
            .create_buffer(info, Some(bytemuck::bytes_of(self.scene_uniform.as_ref())));

        let info = BufferInfo {
            size: std::mem::size_of::<VariableUniform>() as u32,
        };
        let variable_uniform_buffer = self
            .context
            .borrow_mut()
            .create_buffer(info, Some(bytemuck::bytes_of(&self.variable_uniform)));

        let info = TextureInfo {
            width: self.output_config.width,
            height: self.output_config.height,
            mips: 1,
            samples: 1,
            format: gl::RGBA32F,
            ty: gl::TEXTURE_2D,
        };
        let traced_img = self.context.borrow_mut().create_texture(info);

        let info = SamplerInfo {
            filter_min: gl::LINEAR,
            filter_mag: gl::LINEAR,
            address_u: gl::CLAMP_TO_EDGE,
            address_v: gl::CLAMP_TO_EDGE,
            address_w: gl::CLAMP_TO_EDGE,
        };
        let traced_img_sampler = self.context.borrow_mut().create_sampler(info);

        let info = ShaderInfo {
            source: include_str!("../../shaders/ray_tracing.comp").to_owned(),
            stage: gl::COMPUTE_SHADER,
        };
        let trace_cs = self.context.borrow_mut().create_shader(info);
        let info = ComputePipelineInfo { shader: trace_cs };
        let trace_pipeline = self.context.borrow_mut().create_compute_pipeline(info);

        let info = ShaderInfo {
            source: include_str!("../../shaders/screen.vert").to_owned(),
            stage: gl::VERTEX_SHADER,
        };
        let screen_vs = self.context.borrow_mut().create_shader(info);
        let info = ShaderInfo {
            source: include_str!("../../shaders/post.frag").to_owned(),
            stage: gl::FRAGMENT_SHADER,
        };
        let post_fs = self.context.borrow_mut().create_shader(info);
        let info = GraphicsPipelineInfo {
            shaders: GraphicsShaders {
                vertex: screen_vs,
                fragment: post_fs,
            },
            vertex_attributes: vec![],
            vertex_bindings: vec![],
            primitive_topology: gl::TRIANGLES,
        };
        let post_pipeline = self.context.borrow_mut().create_graphics_pipeline(info);

        self.gl_resources = Some(GlResources {
            scene_uniform_buffer,
            variable_uniform_buffer,
            traced_img,
            traced_img_sampler,
            trace_pipeline,
            post_pipeline,
        });
    }

    pub fn render(&self) {
        self.context
            .borrow_mut()
            .bind_compute_pipeline(&self.resource().trace_pipeline);
        self.context.borrow_mut().bind_image(
            0,
            &self.resource().traced_img,
            0,
            Some(0),
            gl::WRITE_ONLY,
        );
        self.context.borrow_mut().bind_shader_storage_buffer(
            1,
            &self.resource().scene_uniform_buffer,
            None,
        );
        self.context.borrow_mut().bind_uniform_buffer(
            2,
            &self.resource().variable_uniform_buffer,
            None,
        );
        self.context.borrow().dispatch_compute(
            (
                self.output_config.width / 8,
                self.output_config.height / 8,
                1,
            ),
            true,
        );

        self.context
            .borrow_mut()
            .bind_graphics_pipeline(&self.resource().post_pipeline);
        self.context
            .borrow_mut()
            .bind_texture(0, &self.resource().traced_img);
        self.context
            .borrow_mut()
            .bind_sampler(0, &self.resource().traced_img_sampler);
        self.context.borrow().draw(3);
    }

    fn resource(&self) -> &GlResources {
        self.gl_resources.as_ref().unwrap()
    }
}
