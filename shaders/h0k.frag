#define TAU 6.283185307179586476925286766559

in vec2 uv;

uniform sampler2D input_texture;

out vec4 frag;

uniform int n;
uniform int scale;
uniform float amplitude;
uniform float intensity; // wind speed
uniform vec2 direction;
uniform float l; // capillary supress factor

const float g = 9.81;

// Box-Muller-Method
vec4 gaus_rnd() {
  vec4 random = clamp(texture(input_texture, uv), 0.001, 1.0);

  vec2 a = sqrt(-2.0 * log(random.rg));
  vec2 b = TAU * random.ba;

  vec4 rnd;
  rnd.x = a.x * cos(b.x);
  rnd.y = a.x * sin(b.x);
  rnd.z = a.y * cos(b.y);
  rnd.w = a.y * sin(b.y);
  return rnd;
}

float h0(vec2 k) {
  float L_ = (intensity * intensity) / g;

  float mag = max(length(k), 0.0001);
  float mag_sq = mag * mag;

  float phillips_k = amplitude / (mag_sq * mag_sq) *
    pow(dot(normalize(k), normalize(direction)), 2) *
    exp(-1.0 / (mag_sq * L_ * L_)) *
    exp(-mag_sq * l * l);

  return clamp(sqrt(phillips_k) / sqrt(2.0), -4000.0, 4000.0);
}

void main(void) {
  vec2 xy = gl_FragCoord.xy - float(n) / 2.0;
  vec2 k = TAU * xy / scale;

  vec4 gauss_random = gaus_rnd();
  frag.xy = gauss_random.xy * h0(k);
  frag.zw = gauss_random.zw * h0(-k);
}
