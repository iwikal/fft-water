in vec2 uv;

uniform sampler2D twiddle_indices;
uniform sampler2D input_texture;

out vec4 frag;

uniform int stage;
uniform int direction;

// Complex multiplication
vec2 cmul(vec2 c0, vec2 c1) {
  vec2 c;
  c.x = c0.x * c1.x - c0.y * c1.y;
  c.y = c0.x * c1.y + c0.y * c1.x;
  return c;
}

vec4 get_pixel(sampler2D sampler, ivec2 uv) {
  vec2 size = textureSize(sampler, 0);
  return texture(sampler, (uv + 0.5) / size);
}

vec2 get_input_pixel(ivec2 uv) {
  if (direction != 0) uv = uv.yx; // Flip coordinates
  return get_pixel(input_texture, uv).rg;
}

void main() {
  vec2 frag_coord = gl_FragCoord.xy - 0.5;
  if (direction != 0) frag_coord = frag_coord.yx; // Flip coordinates

  vec4 twiddle = get_pixel(twiddle_indices, ivec2(stage, frag_coord.x)).rgba;
  vec2 w = vec2(twiddle.x, twiddle.y);
  vec2 p = get_input_pixel(ivec2(twiddle.z, frag_coord.y));
  vec2 q = get_input_pixel(ivec2(twiddle.w, frag_coord.y));

  //Butterfly operation
  vec2 H = p + cmul(w, q);

  frag = vec4(H, 0, 1);
}
