use luminance::shader::program::Program;

pub fn from_strings<A, B, C>(vert: &str, frag: &str) -> Program<A, B, C>
where
    A: luminance::vertex::Vertex,
    C: luminance::shader::program::UniformInterface,
{
    let (shader, warnings) = Program::from_strings(None, vert, None, frag)
        .unwrap_or_else(|error| {
            eprintln!("{}", error);
            panic!();
        });

    for warning in warnings {
        eprintln!("{}", warning);
    }

    shader
}
