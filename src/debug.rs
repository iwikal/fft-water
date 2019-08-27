#![allow(dead_code)]
use luminance::{
    context::GraphicsContext,
    linear::M44,
    pipeline::BoundTexture,
    pixel::RGBA32F,
    shader::program::Program,
    tess::{Mode, Tess},
    texture::{Dim2, Flat},
};

uniform_interface! {
    pub struct DebugShaderInterface {
        input_texture: &'static BoundTexture<'static, Flat, Dim2, RGBA32F>,
        view_projection: M44,
        model: M44
    }
}

impl DebugShaderInterface {
    pub fn set_texture(&self, t: &BoundTexture<'_, Flat, Dim2, RGBA32F>) {
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
    pub tess: Tess<()>,
}

impl Debugger {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let shader = crate::shader::from_strings(
            include_str!("../shaders/framebuffer-debug.vert"),
            include_str!("../shaders/framebuffer-debug.frag"),
        );
        let tess = Tess::attributeless(context, Mode::TriangleStrip, 4);

        Self { shader, tess }
    }
}
