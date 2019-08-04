layout (location = 0) in vec3 a_position;

out float height;

uniform mat4 view_projection;
uniform vec2 offset;

void main() {
  vec3 position = vec3(a_position.x + offset.x, a_position.y, a_position.z + offset.y);
  gl_Position = view_projection * vec4(position, 1.0);
  height = a_position.y;
}
