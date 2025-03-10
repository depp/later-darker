// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl_shader.hpp"

#include "log.hpp"

#include <cstdio>
#include <limits>
#include <string_view>

namespace demo {
namespace gl_shader {

namespace {

constexpr std::string_view Vertex{
	"#version 330\n"
	"layout(location = 0) in vec2 Vertex;\n"
	"void main() {\n"
	"    gl_Position = vec4(Vertex, 0.0, 1.0);\n"
	"}\n"};

constexpr std::string_view Fragment{
	"#version 330\n"
	"out vec4 FragColor;\n"
	"void main() {\n"
	"    FragColor = vec4(0.5, 0.5, 0.5, 1.0);\n"
	"}\n"};

// Compile a shader from the given source code.
GLuint CompileShader(GLenum shaderType, std::string_view source) {
	GLuint shader = glCreateShader(shaderType);
	if (shader == 0) {
		FAIL("Could not create shader.");
	}

	CHECK(source.size() <= std::numeric_limits<GLint>::max());
	const char *srcText[1] = {source.data()};
	const GLint srcLen[1] = {static_cast<GLint>(source.size())};
	glShaderSource(shader, 1, srcText, srcLen);
	glCompileShader(shader);
	GLint status;
	glGetShaderiv(shader, GL_COMPILE_STATUS, &status);
	if (!status) {
		FAIL("Shader failed to compile.");
	}
	return shader;
}

GLuint LinkProgram(GLuint vertex, GLuint fragment) {
	GLuint program = glCreateProgram();
	if (program == 0) {
		FAIL("Could not create shader program.");
	}

	glAttachShader(program, vertex);
	glAttachShader(program, fragment);
	glLinkProgram(program);
	glDetachShader(program, vertex);
	glDetachShader(program, fragment);
	GLint status;
	glGetProgramiv(program, GL_LINK_STATUS, &status);
	if (!status) {
		FAIL("Shader program failed to link.");
	}
	return program;
}

} // namespace

GLuint Program;

void Init() {
	GLuint vertex = CompileShader(GL_VERTEX_SHADER, Vertex);
	GLuint fragment = CompileShader(GL_FRAGMENT_SHADER, Fragment);
	GLuint program = LinkProgram(vertex, fragment);
	glDeleteShader(vertex);
	glDeleteShader(fragment);
	Program = program;
}

} // namespace gl_shader
} // namespace demo
