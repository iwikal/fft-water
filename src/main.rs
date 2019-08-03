extern crate nalgebra_glm as glm;
#[macro_use]
extern crate luminance;

use gl;
use luminance::{
    context::GraphicsContext, framebuffer::Framebuffer, state::GraphicsState,
};
use sdl2;
use std::cell::RefCell;
use std::rc::Rc;
use tiny_ecs;

mod camera;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

struct SdlContext {
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

fn main() {
    let sdl = sdl2::init().expect("Could not init sdl2");
    let video_system = sdl.video().expect("Could not initialize video system");
    sdl.mouse().set_relative_mouse_mode(true);
    let window = video_system
        .window("Hello", SCREEN_WIDTH, SCREEN_HEIGHT)
        .opengl()
        .fullscreen_desktop()
        .build()
        .expect("Could not create window");
    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_system.gl_get_proc_address(s) as *const _);

    let state = GraphicsState::new().unwrap();

    let (width, height) = window.size();

    let mut graphics_context = SdlContext {
        window,
        state: Rc::new(RefCell::new(state)),
    };

    type Position = [f32; 3];
    type RGB = [f32; 3];
    type Vertex = (Position, RGB);

    use luminance::shader::program::Program;
    let (shader, warnings) =
        Program::<Vertex, (), ShaderInterface>::from_strings(
            None, SHADER_VS, None, SHADER_FS,
        )
        .unwrap();

    for warning in warnings {
        eprintln!("{:#?}", warning);
    }

    let mut event_pump = sdl.event_pump().unwrap();
    let mut back_buffer = Framebuffer::back_buffer([width, height]);

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

    use luminance::tess::{Mode, Tess};
    let triangles = Tess::new(
        &mut graphics_context,
        Mode::Triangle,
        &vertices[..],
        &indices[..],
    );

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

    use std::time::Instant;
    let mut previous_frame_start = Instant::now();
    'app: loop {
        let current_frame_start = Instant::now();
        let delta_t = current_frame_start - previous_frame_start;

        for event in event_pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => {
                    break 'app;
                }
                Event::KeyDown { keycode, .. } => {
                    use sdl2::keyboard::Keycode::*;
                    match keycode {
                        Some(Escape) => {
                            break 'app;
                        }
                        _ => {}
                    }
                }
                Event::Window { win_event, .. } => {
                    use sdl2::event::WindowEvent;
                    match win_event {
                        WindowEvent::SizeChanged(width, height) => {
                            back_buffer = Framebuffer::back_buffer([
                                width as u32,
                                height as u32,
                            ]);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        camera
            .take_input(&event_pump, delta_t.as_micros() as f32 / 1_000_000.0);

        graphics_context.pipeline_builder().pipeline(
            &back_buffer,
            [1.0, 1.0, 0.0, 1.0],
            |_, shader_gate| {
                shader_gate.shade(&shader, |render_gate, uniform_interface| {
                    let view_projection = camera.projection() * camera.view();
                    uniform_interface
                        .view_projection
                        .update(view_projection.into());

                    use luminance::render_state::RenderState;
                    render_gate.render(RenderState::default(), |tess_gate| {
                        let components =
                            entities.borrow_mut::<Primitive>().unwrap();
                        for component in components.get() {
                            let (_id, Primitive { tess }) = component;
                            tess_gate
                                .render(&mut graphics_context, tess.into());
                        }
                    })
                });
            },
        );

        graphics_context.swap_buffers();
        previous_frame_start = current_frame_start;
    }
}
