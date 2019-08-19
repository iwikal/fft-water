in float height;

out vec4 frag;

void main() {
  frag = vec4(vec3(height / 2.0 + 0.5), 1.0);
}
