use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::{BoundTexture, Builder},
    pixel::RGBA32F,
    shader::program::Program,
    tess::{Mode, Tess},
    texture::{Dim2, Flat, Texture},
};

uniform_interface! {
    struct H0kInterface {
        input_texture: &'static BoundTexture<'static, Flat, Dim2, RGBA32F>
    }
}

type H0kTexture = Texture<Flat, Dim2, RGBA32F>;

pub struct H0k {
    tess: Tess<()>,
    input_texture: Texture<Flat, Dim2, RGBA32F>,
    shader: Program<(), (), H0kInterface>,
    pub framebuffer: Framebuffer<Flat, Dim2, H0kTexture, ()>,
}

const N: u32 = 256;

impl H0k {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0).expect("framebuffer creation");
        let shader = crate::shader::from_strings(
            include_str!("../shaders/quad.vert"),
            include_str!("../shaders/h0k.frag"),
        );

        use luminance::texture::{MagFilter, MinFilter, Sampler};
        let mut sampler = Sampler::default();
        sampler.mag_filter = MagFilter::Nearest;
        sampler.min_filter = MinFilter::Nearest;

        let input_texture = Texture::new(context, size, 0, &sampler).unwrap();
        {
            let length = N * N;
            let mut pixels = Vec::with_capacity(length as usize);
            let mut rng = rand::thread_rng();
            for _ in 0..length {
                use rand::Rng;
                pixels.push(rng.gen());
            }

            input_texture.upload(false, &pixels);
        }

        let tess = Tess::attributeless(context, Mode::TriangleStrip, 4);
        Self {
            tess,
            input_texture,
            shader,
            framebuffer,
        }
    }

    pub fn render(
        &self,
        context: &mut impl GraphicsContext,
        builder: &Builder,
    ) {
        builder.pipeline(
            &self.framebuffer,
            [1.0, 1.0, 0.0, 1.0],
            |pipeline, shader_gate| {
                let bound_noise = pipeline.bind_texture(&self.input_texture);
                shader_gate.shade(&self.shader, |render_gate, iface| {
                    iface.input_texture.update(&bound_noise);
                    use luminance::render_state::RenderState;
                    render_gate.render(RenderState::default(), |tess_gate| {
                        tess_gate.render(context, (&self.tess).into());
                    });
                });
            },
        );
    }
}

uniform_interface! {
    struct HktInterface {
        input_texture: &'static BoundTexture<'static, Flat, Dim2, RGBA32F>,
        time: f32
    }
}

type HktTexture = Texture<Flat, Dim2, RGBA32F>;

pub struct Hkt {
    tess: Tess<()>,
    shader: Program<(), (), HktInterface>,
    pub framebuffer: Framebuffer<Flat, Dim2, HktTexture, ()>,
}

impl Hkt {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0).expect("framebuffer creation");
        let shader = crate::shader::from_strings(
            include_str!("../shaders/quad.vert"),
            include_str!("../shaders/hkt.frag"),
        );

        use luminance::texture::{MagFilter, MinFilter, Sampler};
        let mut sampler = Sampler::default();
        sampler.mag_filter = MagFilter::Nearest;
        sampler.min_filter = MinFilter::Nearest;

        let tess = Tess::attributeless(context, Mode::TriangleStrip, 4);
        Self {
            tess,
            shader,
            framebuffer,
        }
    }

    pub fn render(
        &self,
        context: &mut impl GraphicsContext,
        builder: &Builder,
        time: f32,
        input_texture: &H0kTexture,
    ) {
        builder.pipeline(
            &self.framebuffer,
            [0.0, 0.0, 0.0, 1.0],
            |pipeline, shader_gate| {
                let bound_noise = pipeline.bind_texture(input_texture);
                shader_gate.shade(&self.shader, |render_gate, iface| {
                    iface.input_texture.update(&bound_noise);
                    iface.time.update(time);
                    use luminance::render_state::RenderState;
                    render_gate.render(RenderState::default(), |tess_gate| {
                        tess_gate.render(context, (&self.tess).into());
                    });
                });
            },
        );
    }
}

type TwiddleTexture = Texture<Flat, Dim2, RGBA32F>;

pub fn twiddle_indices(context: &mut impl GraphicsContext) -> TwiddleTexture {
    use luminance::texture::{MagFilter, MinFilter, Sampler};
    let mut sampler = Sampler::default();
    sampler.mag_filter = MagFilter::Nearest;
    sampler.min_filter = MinFilter::Nearest;

    let bits = (N as f32).log2() as u32;
    let width = bits;
    let height = N;
    let texture = Texture::new(context, [width, height], 0, &sampler).unwrap();
    {
        const TAU: f32 = std::f32::consts::PI * 2.0;

        let length = width * height;
        let mut pixels = Vec::with_capacity(length as usize);
        for y in 0..height {
            for x in 0..width {
                let nf = N as f32;
                let span = u32::pow(2, x);

                let index = span * 2;

                let k = (y as f32 * nf / index as f32) % nf;
                let t = TAU * k / nf;

                let top_wing = y % index < span;

                let reverse = |i: u32| i.reverse_bits().rotate_left(bits);

                let (mut z, mut w) = if top_wing {
                    (y, y + span)
                } else {
                    (y - span, y)
                };

                if x == 0 {
                    z = reverse(z);
                    w = reverse(w);
                }

                pixels.push((t.cos(), t.sin(), z as f32, w as f32));
            }
        }

        texture.upload(false, &pixels);
    }

    texture
}