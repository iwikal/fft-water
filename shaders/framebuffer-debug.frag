in vec2 uv;

out vec4 frag;

uniform sampler2D input_texture;

vec4 get_pixel(sampler2D sampler, ivec2 uv) {
  vec2 size = textureSize(sampler, 0);
  return texture(sampler, (uv + 0.5) / size);
}

void main() {
  vec2 size = textureSize(input_texture, 0);
  frag = vec4(get_pixel(input_texture, ivec2(floor(uv * size))).xyz, 1.0);
}
