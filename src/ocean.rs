use luminance::{
    context::GraphicsContext,
    linear::M44,
    shader::program::Program,
    tess::{Mode, Tess},
};

uniform_interface! {
    pub struct ShaderInterface {
        view_projection: M44,
        offset: [f32; 2]
    }
}

impl ShaderInterface {
    pub fn set_view_projection(&self, value: M44) {
        self.view_projection.update(value);
    }
    
    pub fn set_offset(&self, value: [f32; 2]) {
        self.offset.update(value);
    }
}

type Vertex = [f32; 3];

pub struct Ocean {
    pub tess: Tess<Vertex>,
}

type OceanShader = Program<Vertex, (), ShaderInterface>;
pub fn shader() -> OceanShader {
    let (shader, warnings) = OceanShader::from_strings(
        None,
        include_str!("../shaders/ocean.vert"),
        None,
        include_str!("../shaders/ocean.frag"),
    )
    .unwrap_or_else(|error| {
        eprintln!("{}", error);
        panic!("Can't go on without this shader");
    });

    for warning in warnings {
        eprintln!("{:#?}", warning);
    }

    shader
}

impl Ocean {
    pub fn new(graphics_context: &mut impl GraphicsContext) -> Self {
        let tess = {
            let side = 64;
            let n_lines = side + 1;
            let vertices = {
                let mut vertices = Vec::with_capacity(n_lines * n_lines);
                for x in 0..n_lines {
                    for z in 0..n_lines {
                        let x = x as f32;
                        let z = z as f32;
                        let side = side as f32;
                        let phase = x * std::f32::consts::PI * 2.0 / side;
                        let y = phase.sin() / 2.0 + 0.5;
                        vertices.push([x / side, y, z / side]);
                    }
                }
                vertices
            };

            let indices = {
                let mut indices = Vec::with_capacity(side * side * 6);
                let side = side as u32;
                for x in 0..side {
                    for y in 0..side {
                        let n_lines = n_lines as u32;
                        indices.push(x + y * n_lines);
                        indices.push(x + y * n_lines + 1);
                        indices.push(x + y * n_lines + n_lines);
                        indices.push(x + y * n_lines + n_lines);
                        indices.push(x + y * n_lines + 1);
                        indices.push(x + y * n_lines + n_lines + 1);
                    }
                }
                indices
            };

            Tess::new(
                graphics_context,
                Mode::Triangle,
                &vertices[..],
                &indices[..],
            )
        };

        Self { tess }
    }
}
