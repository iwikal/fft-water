layout (location = 0) in vec3 co;
layout (location = 1) in vec3 color;

out vec3 v_color;

uniform mat4 view_projection;

void main() {
  gl_Position = view_projection * vec4(co, 1.);
  v_color = color;
}
