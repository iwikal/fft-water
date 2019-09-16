in vec2 uv;

uniform sampler2D input_texture;
uniform int n = 512;

out vec4 frag;

void main() {
  vec2 xy = gl_FragCoord.xy - 0.5;

  float perm = mod(dot(xy, xy), 2) * -2.0 + 1.0;

  float h = texture(input_texture, uv).x;
  frag = vec4(perm * h / float(n * n));
  frag.w = 1.0;
}
