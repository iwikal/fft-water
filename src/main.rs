extern crate nalgebra_glm as glm;
#[macro_use]
extern crate luminance;

use gl;
use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    state::GraphicsState,
    tess::{Mode, Tess},
};
use sdl2;
use std::cell::RefCell;
use std::rc::Rc;
use tiny_ecs;

mod camera;
mod fft;
mod ocean;
mod shader;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

struct SdlContext {
    _gl_context: sdl2::video::GLContext,
    window: sdl2::video::Window,
    state: Rc<RefCell<GraphicsState>>,
}

unsafe impl GraphicsContext for SdlContext {
    fn state(&self) -> &Rc<RefCell<GraphicsState>> {
        &self.state
    }

    fn swap_buffers(&mut self) {
        self.window.gl_swap_window();
    }
}

uniform_interface! {
    struct ShaderInterface {
        view_projection: luminance::linear::M44
    }
}

const SHADER_VS: &str = include_str!("../shaders/example.vert");
const SHADER_FS: &str = include_str!("../shaders/example.frag");

mod debug_shader {
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
}

fn main() {
    let sdl = sdl2::init().expect("Could not init sdl2");

    let context = {
        let video_system =
            sdl.video().expect("Could not initialize video system");
        sdl.mouse().set_relative_mouse_mode(true);

        let window = video_system
            .window("Hello", SCREEN_WIDTH, SCREEN_HEIGHT)
            .opengl()
            .fullscreen_desktop()
            .build()
            .expect("Could not create window");
        let _gl_context = window
            .gl_create_context()
            .expect("Could not create OpenGL context");
        gl::load_with(|s| video_system.gl_get_proc_address(s) as *const _);

        let state = GraphicsState::new()
            .expect("Only one graphics state per thread allowed");

        &mut SdlContext {
            _gl_context,
            window,
            state: Rc::new(RefCell::new(state)),
        }
    };

    let (width, height) = context.window.size();

    type Position = [f32; 3];
    type RGB = [f32; 3];
    type Vertex = (Position, RGB);

    use luminance::shader::program::Program;
    let (triangle_shader, warnings) =
        Program::<Vertex, (), ShaderInterface>::from_strings(
            None, SHADER_VS, None, SHADER_FS,
        )
        .unwrap();

    for warning in warnings {
        eprintln!("{:#?}", warning);
    }

    let ocean_shader = ocean::shader();

    let h0k = fft::H0k::new(context);
    let h0k_texture = {
        let builder = context.pipeline_builder();
        h0k.render(context, &builder)
    };

    let hkt = fft::Hkt::new(context);

    let mut event_pump = sdl.event_pump().unwrap();
    let mut back_buffer = Framebuffer::back_buffer([width, height]);

    let triangles = {
        let vertices = [
            ([0.5, -0.5, -1.0], [0.0, 1.0, 0.0]),
            ([0.0, 0.5, -1.0], [0.0, 0.0, 1.0]),
            ([-0.5, -0.5, -1.0], [1.0, 0.0, 0.0]),
            ([0.5, 0.5, -1.0], [0.0, 1.0, 0.0]),
            ([0.0, -0.5, -1.0], [0.0, 0.0, 1.0]),
            ([-0.5, 0.5, -1.0], [1.0, 0.0, 0.0]),
        ];

        let indices = [
            0, 1, 2, // first triangle
            3, 4, 5, // second triangle
        ];

        Tess::new(context, Mode::Triangle, &vertices[..], &indices[..])
    };

    let mut camera =
        camera::Camera::persp(width as f32 / height as f32, 0.9, 0.1, 100.0);

    let mut entities = tiny_ecs::Entities::new(Some(64), Some(24));

    struct Primitive {
        tess: Tess<Vertex>,
    }

    entities
        .new_entity()
        .with(Primitive { tess: triangles })
        .unwrap()
        .finalise()
        .unwrap();

    let ocean = ocean::Ocean::new(context);
    entities
        .new_entity()
        .with(ocean)
        .unwrap()
        .finalise()
        .unwrap();

    let fft = fft::Fft::new(context);

    let twids = fft::twiddle_indices(context);

    let debugger = debug_shader::Debugger::new(context);

    use std::time::Instant;
    let start = Instant::now();
    let mut previous_frame_start = start;
    let mut debug_framebuffer = false;
    'app: loop {
        let current_frame_start = Instant::now();
        let delta_t = current_frame_start - previous_frame_start;

        let mut resize = None;
        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => {
                    break 'app;
                }
                Event::KeyDown { scancode, .. } => {
                    use sdl2::keyboard::Scancode::*;
                    match scancode {
                        Some(Escape) => {
                            break 'app;
                        }
                        Some(F) => {
                            debug_framebuffer = !debug_framebuffer;
                        }
                        _ => {}
                    }
                }
                Event::Window { win_event, .. } => {
                    use sdl2::event::WindowEvent;
                    match win_event {
                        WindowEvent::SizeChanged(width, height) => {
                            resize = Some([width, height]);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if let Some([width, height]) = resize {
            let size = [width as u32, height as u32];
            back_buffer = Framebuffer::back_buffer(size);
        }

        camera
            .take_input(&event_pump, delta_t.as_micros() as f32 / 1_000_000.0);

        use luminance::render_state::RenderState;
        let builder = context.pipeline_builder();

        let duration = current_frame_start - start;
        let f_time = duration.as_secs() as f32
            + duration.subsec_nanos() as f32 / 1_000_000_000.0;
        let hkt_texture = hkt.render(context, &builder, f_time, h0k_texture);

        let heightmap = fft.render(context, &builder, hkt_texture);

        builder.pipeline(
            &back_buffer,
            [0.1, 0.2, 0.3, 1.0],
            |pipeline, shader_gate| {
                let view_projection = camera.projection() * camera.view();
                if debug_framebuffer {
                    shader_gate.shade(
                        &debugger.shader,
                        |render_gate, iface| {
                            let textures =
                                [heightmap, &twids, hkt_texture, h0k_texture];
                            for (i, t) in textures.iter().enumerate() {
                                iface.set_texture(&pipeline.bind_texture(t));
                                iface.set_view_projection(view_projection);
                                let pos = glm::vec3(-1.0, 0.0, 0.0) * i as f32;
                                let pos = pos - glm::vec3(0.5, 0.5, 0.5);
                                let model_mat =
                                    glm::translate(&glm::one(), &pos);
                                iface.set_model(model_mat);
                                render_gate.render(
                                    RenderState::default(),
                                    |tess_gate| {
                                        tess_gate.render(
                                            context,
                                            (&debugger.tess).into(),
                                        );
                                    },
                                );
                            }
                        },
                    );
                } else {
                    shader_gate.shade(
                        &triangle_shader,
                        |render_gate, uniform_interface| {
                            uniform_interface
                                .view_projection
                                .update(view_projection.into());

                            render_gate.render(
                                RenderState::default(),
                                |tess_gate| {
                                    let components = entities
                                        .borrow_mut::<Primitive>()
                                        .unwrap();
                                    for component in components.get() {
                                        let (_id, Primitive { tess }) =
                                            component;
                                        tess_gate.render(context, tess.into());
                                    }
                                },
                            )
                        },
                    );
                    shader_gate.shade(
                        &ocean_shader,
                        |render_gate, uniform_interface| {
                            uniform_interface
                                .set_view_projection(view_projection.into());
                            let heightmap = pipeline.bind_texture(heightmap);
                            uniform_interface.set_heightmap(&heightmap);
                            render_gate.render(
                                RenderState::default(),
                                |tess_gate| {
                                    use ocean::Ocean;
                                    let components =
                                        entities.borrow_mut::<Ocean>().unwrap();
                                    for component in components.get() {
                                        let (_id, Ocean { tess }) = component;
                                        let side = 8;
                                        for x in 0..side {
                                            for y in 0..side {
                                                uniform_interface.set_offset([
                                                    x as f32, y as f32,
                                                ]);
                                                tess_gate.render(
                                                    context,
                                                    tess.into(),
                                                );
                                            }
                                        }
                                    }
                                },
                            )
                        },
                    );
                }
            },
        );

        context.swap_buffers();
        previous_frame_start = current_frame_start;
    }
}
