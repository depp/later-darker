#version 330

layout(location = 0) in vec3 Vertex;
layout(location = 1) in vec4 Color;

uniform mat4 MVP;

out vec4 vColor;

void main() {
	gl_Position = MVP * vec4(Vertex, 1.0);
	vColor = Color;
}
