use luminance::{
    context::GraphicsContext,
    linear::M44,
    pipeline::{BoundTexture, Builder, Pipeline, ShadingGate},
    pixel::RGBA32F,
    render_state::RenderState,
    shader::program::Program,
    tess::{Mode, Tess},
    texture::{Dim2, Flat},
};

uniform_interface! {
    pub struct OceanShaderInterface {
        heightmap: &'static BoundTexture<'static, Flat, Dim2, RGBA32F>,
        view_projection: M44,
        offset: [f32; 2]
    }
}

impl OceanShaderInterface {
    pub fn set_view_projection(&self, value: M44) {
        self.view_projection.update(value);
    }

    pub fn set_offset(&self, value: [f32; 2]) {
        self.offset.update(value);
    }

    pub fn set_heightmap(&self, value: &BoundTexture<Flat, Dim2, RGBA32F>) {
        self.heightmap.update(value);
    }
}

type OceanVertex = [f32; 3];
type OceanShader = Program<OceanVertex, (), OceanShaderInterface>;

use crate::fft::{Fft, FftFramebuffer, H0k, Hkt};
pub struct Ocean {
    pub h0k: H0k,
    pub hkt: Hkt,
    pub fft: Fft,
    pub heightmap_buffer: FftFramebuffer,
    shader: OceanShader,
    tess: Tess<OceanVertex>,
}

impl Ocean {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let h0k = H0k::new(context);
        {
            let builder = context.pipeline_builder();
            h0k.render(context, &builder);
        }
        let hkt = Hkt::new(context);
        let fft = Fft::new(context);
        let heightmap_buffer = FftFramebuffer::new(context, [0x100, 0x100], 0)
            .expect("framebuffer creation");
        let shader = crate::shader::from_strings(
            include_str!("../shaders/ocean.vert"),
            include_str!("../shaders/ocean.frag"),
        );
        let tess = {
            let side: usize = 0x100;
            let n_lines = side + 1;
            let vertices = {
                let mut vertices = Vec::with_capacity(n_lines * n_lines);
                for x in 0..n_lines {
                    for z in 0..n_lines {
                        let x = x as f32;
                        let z = z as f32;
                        let side = side as f32;
                        vertices.push([x / side, 0.0, z / side]);
                    }
                }
                vertices
            };

            let indices = {
                let mut indices =
                    Vec::with_capacity(side * (n_lines * 2 + 3) - 3);
                let side = side as u32;
                let n_lines = n_lines as u32;
                for x in 0..side {
                    if x != 0 {
                        indices.push(x * n_lines + side);
                        indices.push(x * n_lines);
                        indices.push(x * n_lines);
                    }
                    for y in 0..n_lines {
                        indices.push(x * n_lines + y);
                        indices.push(x * n_lines + y + n_lines);
                    }
                }
                assert_eq!(indices.len(), indices.capacity());
                indices
            };

            Tess::new(context, Mode::TriangleStrip, &vertices[..], &indices[..])
        };

        Self {
            h0k,
            hkt,
            fft,
            heightmap_buffer,
            shader,
            tess,
        }
    }

    pub fn simulate(
        &mut self,
        context: &mut impl GraphicsContext,
        builder: &Builder,
        time: f32,
    ) -> OceanFrame {
        let Self {
            h0k,
            hkt,
            fft,
            heightmap_buffer,
            ..
        } = self;
        hkt.render(context, builder, time, h0k.framebuffer.color_slot());
        fft.render(
            context,
            builder,
            hkt.framebuffer.color_slot(),
            heightmap_buffer,
        );
        OceanFrame(self)
    }
}

pub struct OceanFrame<'a>(&'a Ocean);

impl<'a> OceanFrame<'a> {
    pub fn render(
        &self,
        context: &mut impl GraphicsContext,
        pipeline: &Pipeline,
        shader_gate: &ShadingGate,
        view_projection: impl Into<M44>,
    ) {
        let Self(Ocean {
            heightmap_buffer,
            shader,
            tess,
            ..
        }) = self;

        let heightmap = pipeline.bind_texture(heightmap_buffer.color_slot());
        shader_gate.shade(shader, |render_gate, iface| {
            iface.set_view_projection(view_projection.into());
            iface.set_heightmap(&heightmap);
            render_gate.render(RenderState::default(), |tess_gate| {
                for x in -1..1 {
                    for y in -1..1 {
                        iface.set_offset([x as f32, y as f32]);
                        tess_gate.render(context, tess.into());
                    }
                }
            });
        })
    }
}
