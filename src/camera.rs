#[derive(Debug)]
pub struct Camera {
    position: glm::Vec3,
    pitch: f32,
    yaw: f32,
    orientation: glm::Mat4,
    projection: glm::Mat4,
}

impl Camera {
    fn new(projection: glm::Mat4) -> Camera {
        Camera {
            position: glm::zero(),
            pitch: 0.,
            yaw: 0.,
            orientation: glm::identity(),
            projection,
        }
    }

    pub fn persp(width: f32, height: f32, near: f32, far: f32) -> Camera {
        let fov = 1.0; // radians
        let projection = glm::perspective_rh(fov, width / height, near, far);
        Camera::new(projection)
    }

    pub fn take_input(&mut self, pump: &sdl2::EventPump, delta_t: f32) {
        {
            let scale = 1.0 / 128.0;
            let state = pump.relative_mouse_state();
            self.yaw += (state.x() as f32) * scale;
            self.pitch += (state.y() as f32) * scale;

            let ori = glm::rotate(
                &glm::identity(),
                self.pitch,
                &glm::vec3(1., 0., 0.),
            );
            let ori = glm::rotate(&ori, self.yaw, &glm::vec3(0., 1., 0.));
            self.orientation = ori;
        }
        {
            let state = pump.keyboard_state();

            use sdl2::keyboard::*;

            let mut move_vector = glm::vec3(0., 0., 0.);

            let is_pressed = |name| {
                let key = Scancode::from_name(name).unwrap();
                state.is_scancode_pressed(key)
            };

            if is_pressed("W") {
                move_vector = move_vector + glm::vec3(0., 0., -1.);
            }
            if is_pressed("A") {
                move_vector = move_vector + glm::vec3(-1., 0., 0.);
            }
            if is_pressed("S") {
                move_vector = move_vector + glm::vec3(0., 0., 1.);
            }
            if is_pressed("D") {
                move_vector = move_vector + glm::vec3(1., 0., 0.);
            }
            if is_pressed("Space") {
                move_vector = move_vector + glm::vec3(0., 1., 0.);
            }
            if is_pressed("Left Shift") {
                move_vector = move_vector + glm::vec3(0., -1., 0.);
            }
            let length = glm::length(&move_vector);
            let length = if length > 1.0 { length } else { 1.0 };
            move_vector /= length;

            move_vector = {
                let x = move_vector.x;
                let y = move_vector.y;
                let z = move_vector.z;
                let sin = self.yaw.sin();
                let cos = self.yaw.cos();
                glm::vec3(x * cos + z * -sin, y, z * cos + x * sin)
            };

            move_vector = move_vector * (2.0 * delta_t);

            self.position = self.position + move_vector;
        }
    }

    pub fn projection(&self) -> glm::Mat4 {
        self.projection
    }

    pub fn view(&self) -> glm::Mat4 {
        let view: glm::Mat4 = glm::identity();
        let view = view * self.orientation;
        let view = glm::translate(&view, &-self.position);

        view
    }
}
