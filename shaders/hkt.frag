#define TAU 6.283185307179586476925286766559

in vec2 uv;

uniform sampler2D input_texture;

out vec4 frag;

uniform	int N = 512;
uniform int L = 1000;
uniform float time;

const float g = 9.81;

// Complex multiplication
vec2 cmul(vec2 c0, vec2 c1) {
  vec2 c;
  c.x = c0.x * c1.x - c0.y * c1.y;
  c.y = c0.x * c1.y + c0.y * c1.x;
  return c;
}

void main(void) {
  vec2 x = (uv - 0.5) * float(N);
  vec2 k = TAU * x / L;

  float magnitude = max(length(k), 0.00001);

  float w = sqrt(g * magnitude);

  vec4 h0k = texture(input_texture, uv);
  vec2 fou_amp = h0k.rg;
  vec2 fou_amp_conj = vec2(h0k.b, -h0k.a);

  float cosinus = cos(w * time);
  float sinus   = sin(w * time);

  // euler formula
  vec2 exp_iwt = vec2(cosinus, sinus);
  vec2 exp_iwt_inv = vec2(cosinus, -sinus);

  // dy
  vec2 h_k_t_dy = cmul(fou_amp, exp_iwt) + cmul(fou_amp_conj, exp_iwt_inv);

  // dx
  vec2 dx = vec2(0.0, -k.x / magnitude);
  vec2 h_k_t_dx = cmul(dx, h_k_t_dy);

  // dz
  vec2 dy = vec2(0.0, -k.y / magnitude);
  vec2 h_k_t_dz = cmul(dy, h_k_t_dy);

  // tilde_hkt_dy = vec4(h_k_t_dy.xy, 0.0, 1.0);
  // tilde_hkt_dx = vec4(h_k_t_dx.xy, 0.0, 1.0);
  // tilde_hkt_dz = vec4(h_k_t_dz.xy, 0.0, 1.0);
  frag.xy = h_k_t_dy;
  frag.zw = vec2(0.0, 1.0);
}
