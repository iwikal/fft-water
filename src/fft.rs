use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::{BoundTexture, Builder},
    pixel::{Floating, RGBA32F},
    shader::program::{Program, Uniform},
    tess::{Mode, Tess, TessBuilder},
    texture::{Dim2, Flat, GenMipmaps, Texture},
};
use luminance_derive::UniformInterface;

const QUAD_VS_SRC: &str = include_str!("../shaders/quad.vert");

#[derive(UniformInterface)]
struct H0kInterface {
    input_texture:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    n: Uniform<i32>,
    scale: Uniform<i32>,
    amplitude: Uniform<f32>,
    intensity: Uniform<f32>, // wind speed
    direction: Uniform<[f32; 2]>,
    l: Uniform<f32>, // capillary supress factor
}

type H0kTexture = Texture<Flat, Dim2, RGBA32F>;

pub struct H0k {
    tess: Tess,
    input_texture: Texture<Flat, Dim2, RGBA32F>,
    shader: Program<(), (), H0kInterface>,
    pub framebuffer: Framebuffer<Flat, Dim2, RGBA32F, ()>,
    scale: i32,
    amplitude: f32,
    intensity: f32, // wind speed
    direction: glm::Vec2,
    l: f32, // capillary supress factor
}

const N: u32 = 0x100;

impl H0k {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0).expect("framebuffer creation");
        let shader = crate::shader::from_strings(
            QUAD_VS_SRC,
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

            input_texture.upload(GenMipmaps::No, &pixels);
        }

        let tess = TessBuilder::new(context)
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

        Self {
            tess,
            input_texture,
            shader,
            framebuffer,
            scale: 1000,
            amplitude: 4.0,
            intensity: 40.0, // wind speed
            direction: glm::vec2(1.0, 1.0),
            l: 0.5, // capillary supress factor
        }
    }

    pub fn render(
        &self,
        context: &mut impl GraphicsContext,
        builder: &Builder,
    ) -> &H0kTexture {
        builder.pipeline(
            &self.framebuffer,
            [1.0, 1.0, 0.0, 1.0],
            |pipeline, shader_gate| {
                let bound_noise = pipeline.bind_texture(&self.input_texture);
                shader_gate.shade(&self.shader, |render_gate, iface| {
                    iface.input_texture.update(&bound_noise);
                    iface.n.update(N as i32);
                    iface.scale.update(self.scale);
                    iface.amplitude.update(self.amplitude);
                    iface.intensity.update(self.intensity);
                    iface.direction.update(self.direction.into());
                    iface.l.update(self.l);
                    use luminance::render_state::RenderState;
                    render_gate.render(RenderState::default(), |tess_gate| {
                        tess_gate.render(context, (&self.tess).into());
                    });
                });
            },
        );
        self.framebuffer.color_slot()
    }
}

#[derive(UniformInterface)]
struct HktInterface {
    input_texture:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    n: Uniform<i32>,
    time: Uniform<f32>,
}

type HktTexture = Texture<Flat, Dim2, RGBA32F>;

pub struct Hkt {
    tess: Tess,
    shader: Program<(), (), HktInterface>,
    pub framebuffer: Framebuffer<Flat, Dim2, RGBA32F, ()>,
}

impl Hkt {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let size = [N, N];
        let framebuffer =
            Framebuffer::new(context, size, 0).expect("framebuffer creation");
        let shader = crate::shader::from_strings(
            QUAD_VS_SRC,
            include_str!("../shaders/hkt.frag"),
        );

        use luminance::texture::{MagFilter, MinFilter, Sampler};
        let mut sampler = Sampler::default();
        sampler.mag_filter = MagFilter::Nearest;
        sampler.min_filter = MinFilter::Nearest;

        let tess = TessBuilder::new(context)
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

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
    ) -> &HktTexture {
        builder.pipeline(
            &self.framebuffer,
            [0.0, 0.0, 0.0, 1.0],
            |pipeline, shader_gate| {
                let bound_noise = pipeline.bind_texture(input_texture);
                shader_gate.shade(&self.shader, |render_gate, iface| {
                    iface.input_texture.update(&bound_noise);
                    iface.n.update(N as i32);
                    iface.time.update(time);
                    use luminance::render_state::RenderState;
                    render_gate.render(RenderState::default(), |tess_gate| {
                        tess_gate.render(context, (&self.tess).into());
                    });
                });
            },
        );
        self.framebuffer.color_slot()
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

        texture.upload(GenMipmaps::No, &pixels);
    }

    texture
}

#[derive(UniformInterface)]
struct ButterflyInterface {
    twiddle_indices:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    input_texture:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
    stage: Uniform<i32>,
    direction: Uniform<i32>,
}

#[derive(UniformInterface)]
struct InversionInterface {
    input_texture:
        Uniform<&'static BoundTexture<'static, Flat, Dim2, Floating>>,
}

type FftTexture = Texture<Flat, Dim2, RGBA32F>;
pub type FftFramebuffer = Framebuffer<Flat, Dim2, RGBA32F, ()>;

pub struct Fft {
    twiddle_indices: TwiddleTexture,
    butterfly_shader: Program<(), (), ButterflyInterface>,
    inversion_shader: Program<(), (), InversionInterface>,
    pingpong_buffer: FftFramebuffer,
    tess: Tess,
}

impl Fft {
    pub fn new(context: &mut impl GraphicsContext) -> Self {
        let twiddle_indices = twiddle_indices(context);

        let butterfly_shader = crate::shader::from_strings(
            QUAD_VS_SRC,
            include_str!("../shaders/butterfly.frag"),
        );

        let inversion_shader = crate::shader::from_strings(
            QUAD_VS_SRC,
            include_str!("../shaders/inversion.frag"),
        );

        let size = [N, N];

        let pingpong_buffer =
            Framebuffer::new(context, size, 0).expect("framebuffer creation");

        let tess = TessBuilder::new(context)
            .set_mode(Mode::TriangleStrip)
            .set_vertex_nb(4)
            .build()
            .unwrap();

        Self {
            tess,
            twiddle_indices,
            butterfly_shader,
            inversion_shader,
            pingpong_buffer,
        }
    }

    pub fn render<'a>(
        &self,
        context: &mut impl GraphicsContext,
        builder: &Builder,
        input_texture: &FftTexture,
        output_buffer: &'a mut FftFramebuffer,
    ) -> &'a FftTexture {
        let Self {
            tess,
            pingpong_buffer,
            twiddle_indices,
            butterfly_shader,
            inversion_shader,
        } = self;

        let bits = (N as f32).log2() as usize;
        let buffers = [pingpong_buffer, output_buffer];
        let mut pingpong = bits % 2;
        let mut first_round = true;

        for &direction in &[0, 1] {
            for stage in 0..bits {
                let input = if first_round {
                    first_round = false;
                    input_texture
                } else {
                    buffers[pingpong].color_slot()
                };
                let output = buffers[1 - pingpong];

                builder.pipeline(
                    output,
                    [1.0, 1.0, 0.0, 1.0],
                    |pipeline, shader_gate| {
                        let bound_twiddle =
                            pipeline.bind_texture(twiddle_indices);
                        let bound_input = pipeline.bind_texture(input);
                        shader_gate.shade(
                            butterfly_shader,
                            |render_gate, iface| {
                                iface.twiddle_indices.update(&bound_twiddle);
                                iface.input_texture.update(&bound_input);
                                iface.stage.update(stage as i32);
                                iface.direction.update(direction);
                                use luminance::render_state::RenderState;
                                render_gate.render(
                                    RenderState::default(),
                                    |tess_gate| {
                                        tess_gate.render(context, tess.into());
                                    },
                                );
                            },
                        );
                    },
                );
                pingpong = 1 - pingpong;
            }
        }
        {
            let input = buffers[pingpong].color_slot();
            let output = buffers[1 - pingpong];
            builder.pipeline(
                output,
                [1.0, 1.0, 0.0, 1.0],
                |pipeline, shader_gate| {
                    let bound_input = pipeline.bind_texture(input);
                    shader_gate.shade(
                        inversion_shader,
                        |render_gate, iface| {
                            iface.input_texture.update(&bound_input);
                            use luminance::render_state::RenderState;
                            render_gate.render(
                                RenderState::default(),
                                |tess_gate| {
                                    tess_gate.render(context, tess.into());
                                },
                            );
                        },
                    );
                },
            );
        }
        output_buffer.color_slot()
    }
}
