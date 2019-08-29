extern crate nalgebra_glm as glm;
use gl;
use luminance::{
    context::GraphicsContext, framebuffer::Framebuffer, state::GraphicsState,
};
use sdl2;
use std::cell::RefCell;
use std::rc::Rc;

mod camera;
mod debug;
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
}

impl SdlContext {
    fn swap_buffers(&mut self) {
        self.window.gl_swap_window();
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

    let mut event_pump = sdl.event_pump().unwrap();
    let mut back_buffer = Framebuffer::back_buffer([width, height]);

    let mut camera =
        camera::Camera::persp(width as f32 / height as f32, 0.9, 0.1, 100.0);

    let mut ocean = ocean::Ocean::new(context);

    use std::time::Instant;
    let start = Instant::now();
    let mut previous_frame_start = start;
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
                    if let Some(Escape) = scancode {
                        break 'app;
                    }
                }
                Event::Window { win_event, .. } => {
                    use sdl2::event::WindowEvent;
                    if let WindowEvent::SizeChanged(width, height) = win_event {
                        resize = Some([width, height]);
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

        let builder = context.pipeline_builder();

        let duration = current_frame_start - start;
        let f_time = duration.as_secs() as f32
            + duration.subsec_nanos() as f32 / 1_000_000_000.0;

        let ocean_frame = ocean.simulate(context, &builder, f_time);

        builder.pipeline(
            &back_buffer,
            [0.1, 0.2, 0.3, 1.0],
            |pipeline, shader_gate| {
                let view_projection = camera.projection() * camera.view();
                ocean_frame.render(
                    context,
                    &pipeline,
                    &shader_gate,
                    view_projection,
                );
            },
        );

        context.swap_buffers();
        previous_frame_start = current_frame_start;
    }
}
