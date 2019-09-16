in float height;
in vec3 normal;

out vec4 frag;

const vec3 light_dir = vec3(1.0, 0.25, 0.0);

void main() {
  vec3 world_normal = normal.xzy;
  vec3 dir = normalize(light_dir);
  frag = vec4(vec3(max(0.0, dot(dir, world_normal))), 1.0);
}
