out float height;

uniform sampler2D heightmap;
uniform mat4 view_projection;
uniform vec2 offset;

const int N = 256;

void main() {
  vec2 size = textureSize(heightmap, 0);
  int line_count = N + 1;
  int x = gl_VertexID / line_count;
  int y = gl_VertexID % line_count;
  vec2 position = vec2(x, y) + offset * N;
  vec2 uv = position / N;
  height = texture(heightmap, mod(uv, 1)).r;
  gl_Position = view_projection * vec4(position.x, height, position.y, 1.0);
}
