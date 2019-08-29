#![allow(dead_code)]
use luminance::{
    context::GraphicsContext,
    linear::M44,
    pipeline::BoundTexture,
    pixel::Floating,
    shader::program::{Program, Uniform},
    tess::{Mode, Tess, TessBuilder},
    texture::{Dim2, Flat},
};
use luminance_derive::UniformInterface;

#[derive(UniformInterface)]
pub struct DebugShaderInterface {
    input_texture:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    view_projection: Uniform<M44>,
    model: Uniform<M44>,
}

impl DebugShaderInterface {
    pub fn set_texture(&self, t: &BoundTexture<'_, Flat, Dim2, Floating>) {
        self.input_texture.update(t);
    }

    pub fn set_model(&self, m: impl Into<M44>) {
        self.model.update(m.into());
    }

    pub fn set_view_projection(&self, vp: impl Into<M44>) {
        self.view_projection.update(vp.into());
    }
}

pub struct Debugger {
    pub shader: Program<(), (), DebugShaderInterface>,
    pub tess: Tess,
}

impl Debugger {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let shader = crate::shader::from_strings(
            include_str!("../shaders/framebuffer-debug.vert"),
            include_str!("../shaders/framebuffer-debug.frag"),
        );

        let tess = TessBuilder::new(context)
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

        Self { shader, tess }
    }
}
