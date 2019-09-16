out float height;
out vec3 normal;

uniform sampler2D heightmap;
uniform mat4 view_projection;
uniform vec2 offset;

const int N = 256;

vec3 position_at_coordinates(int x, int y) {
  vec2 position = vec2(x, y) + offset * N;
  vec2 uv = position / N;
  vec3 result;
  result.x = position.x;
  result.y = texture(heightmap, mod(uv, 1)).r;
  result.z = position.y;
  return result;
}

void main() {
  vec2 size = textureSize(heightmap, 0);
  int line_count = N + 1;
  int x = gl_VertexID / line_count;
  int y = gl_VertexID % line_count;
  vec3 position = position_at_coordinates(x, y);

  float height_left  = position_at_coordinates(x - 1, y).y;
  float height_right = position_at_coordinates(x + 1, y).y;
  float height_up    = position_at_coordinates(x, y - 1).y;
  float height_down  = position_at_coordinates(x, y + 1).y;

  normal.x = height_left - height_right;
  normal.y = height_down - height_up;
  normal.z = 2.0;
  normal = normalize(normal);

  height = position.y;
  gl_Position = view_projection * vec4(position, 1.0);
}
