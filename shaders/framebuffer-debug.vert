out vec2 uv;

uniform mat4 view_projection;
uniform mat4 model;

void main() {
  vec4 position;
  position.x = float(gl_VertexID % 2);
  position.y = float(gl_VertexID / 2);
  position.z = 0.0;
  position.w = 1.0;
  uv = position.xy;

  gl_Position = view_projection * model * position;
}
