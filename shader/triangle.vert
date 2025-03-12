#version 330
layout(location = 0) in vec2 Vertex;
void main() {
	gl_Position = vec4(Vertex, 0.0, 1.0);
}
