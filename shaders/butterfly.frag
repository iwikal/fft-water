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

void horizontalButterflies() {
  vec2 xy = gl_FragCoord.xy - 0.5;

  vec4 data = get_pixel(twiddle_indices, ivec2(stage, xy.x)).rgba;
  vec2 p = get_pixel(input_texture, ivec2(data.z, xy.y)).rg;
  vec2 q = get_pixel(input_texture, ivec2(data.w, xy.y)).rg;
  vec2 w = vec2(data.x, data.y);

  //Butterfly operation
  vec2 H = p + cmul(w, q);

  frag = vec4(H, 0, 1);
}

void verticalButterflies() {
  vec2 xy = gl_FragCoord.xy - 0.5;

  vec4 data = get_pixel(twiddle_indices, ivec2(stage, xy.y)).rgba;
  vec2 p = get_pixel(input_texture, ivec2(xy.x, data.z)).rg;
  vec2 q = get_pixel(input_texture, ivec2(xy.x, data.w)).rg;
  vec2 w = vec2(data.x, data.y);

  //Butterfly operation
  vec2 H = p + cmul(w, q);

  frag = vec4(H, 0, 1);
}

void main(void) {
  if(direction == 0)
    horizontalButterflies();
  else verticalButterflies();
}
