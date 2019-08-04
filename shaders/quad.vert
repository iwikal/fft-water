out vec2 uv;

void main() {
  gl_Position.x = float(gl_VertexID % 2);
  gl_Position.y = float(gl_VertexID / 2);
  gl_Position.z = 0.0;
  gl_Position.w = 1.0;
  uv = gl_Position.xy;
  gl_Position.xyz *= 2.0;
  gl_Position.xyz -= 1.0;
}
