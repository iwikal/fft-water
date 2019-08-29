use luminance::shader::program::Program;

pub fn from_strings<S, Out, Uni>(vert: &str, frag: &str) -> Program<S, Out, Uni>
where
    S: luminance::vertex::Semantics,
    Uni: luminance::shader::program::UniformInterface,
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
