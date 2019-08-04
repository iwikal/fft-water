extern crate nalgebra_glm as glm;
#[macro_use]
extern crate luminance;

use gl;
use luminance::{
    context::GraphicsContext,
    framebuffer::Framebuffer,
    pipeline::BoundTexture,
    pixel::RGBA32F,
    state::GraphicsState,
    tess::{Mode, Tess},
    texture::{Dim2, Flat, Texture},
};
use sdl2;
use std::cell::RefCell;
use std::rc::Rc;
use tiny_ecs;

mod camera;
mod ocean;

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

impl Drop for SdlContext {
    fn drop(&mut self) {
        eprintln!("Dropping OpenGL context.");
    }
}

uniform_interface! {
    struct ShaderInterface {
        view_projection: luminance::linear::M44
    }
}

const SHADER_VS: &str = include_str!("../shaders/example.vert");
const SHADER_FS: &str = include_str!("../shaders/example.frag");

fn main() {
    let sdl = sdl2::init().expect("Could not init sdl2");

    let mut graphics_context = {
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

        SdlContext {
            _gl_context,
            window,
            state: Rc::new(RefCell::new(state)),
        }
    };

    let (width, height) = graphics_context.window.size();

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

    uniform_interface! {
        struct DebugShaderInterface {
            input_texture: &'static BoundTexture<'static, Flat, Dim2, RGBA32F>
        }
    }

    let (framebuffer_debug_shader, _) = Program::<(), (), DebugShaderInterface>::from_strings(
        None,
        include_str!("../shaders/framebuffer-debug.vert"),
        None, 
        include_str!("../shaders/framebuffer-debug.frag"),
    ).unwrap();
    let quad = Tess::attributeless(&mut graphics_context, Mode::TriangleStrip, 4);

    let mut event_pump = sdl.event_pump().unwrap();
    let mut back_buffer = Framebuffer::back_buffer([width, height]);
    let mut offscreen_buffer =
        Framebuffer::<Flat, Dim2, Texture<Flat, Dim2, RGBA32F>, ()>::new(&mut graphics_context, [width, height], 0).expect("framebuffer creation");

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

        Tess::new(
            &mut graphics_context,
            Mode::Triangle,
            &vertices[..],
            &indices[..],
        )
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

    let ocean = ocean::Ocean::new(&mut graphics_context);
    entities
        .new_entity()
        .with(ocean)
        .unwrap()
        .finalise()
        .unwrap();

    use std::time::Instant;
    let mut previous_frame_start = Instant::now();
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
            offscreen_buffer = Framebuffer::new(&mut graphics_context, size, 0)
                .expect("framebuffer creation");
        }

        camera
            .take_input(&event_pump, delta_t.as_micros() as f32 / 1_000_000.0);

        use luminance::render_state::RenderState;
        let builder = graphics_context.pipeline_builder();
        builder.pipeline(
            &offscreen_buffer,
            [1.0, 1.0, 0.0, 1.0],
            |_, shader_gate| {
            },
        );

        builder.pipeline(
            &back_buffer,
            [0.0, 0.0, 0.0, 0.0],
            |pipeline, shader_gate| {
                if debug_framebuffer {
                    let bound_texture = pipeline.bind_texture(offscreen_buffer.color_slot());
                    shader_gate.shade(&framebuffer_debug_shader, |render_gate, iface| {
                        iface.input_texture.update(&bound_texture);
                        render_gate.render(RenderState::default(), |tess_gate| {
                            tess_gate.render(
                                &mut graphics_context,
                                (&quad).into(),
                            );
                        });
                    });
                } else {
                    let view_projection = camera.projection() * camera.view();
                    shader_gate.shade(
                        &triangle_shader,
                        |render_gate, uniform_interface| {
                            uniform_interface
                                .view_projection
                                .update(view_projection.into());

                            render_gate.render(
                                RenderState::default(),
                                |tess_gate| {
                                    let components =
                                        entities.borrow_mut::<Primitive>().unwrap();
                                    for component in components.get() {
                                        let (_id, Primitive { tess }) = component;
                                        tess_gate.render(
                                            &mut graphics_context,
                                            tess.into(),
                                        );
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
                                                uniform_interface.set_offset(
                                                    [x as f32, y as f32]
                                                );
                                                tess_gate.render(
                                                    &mut graphics_context,
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

        graphics_context.swap_buffers();
        previous_frame_start = current_frame_start;
    }

    std::mem::forget(graphics_context);
}
