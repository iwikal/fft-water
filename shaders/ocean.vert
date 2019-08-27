layout (location = 0) in vec3 a_position;

out float height;

uniform sampler2D heightmap;
uniform mat4 view_projection;
uniform vec2 offset;

void main() {
  vec2 size = textureSize(heightmap, 0);
  vec3 position = vec3(a_position.x + offset.x, 0.0, a_position.z + offset.y);
  vec2 uv = vec2(a_position.x, a_position.z);
  height = texture(heightmap, mod(uv, 1)).r;
  position *= 256;
  position.y = height;
  gl_Position = view_projection * vec4(position, 1.0);
}
