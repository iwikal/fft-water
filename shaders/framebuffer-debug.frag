in vec2 uv;

out vec4 frag;

uniform sampler2D input_texture;

void main() {
  frag = vec4(texture(input_texture, uv).xyz, 1.0);
}
