// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl_shader.hpp"

#include "log.hpp"
#include "os_file.hpp"

#include <cstdio>
#include <limits>
#include <string>
#include <string_view>

namespace demo {
namespace gl_shader {

namespace {

// Compile a shader from the given source code.
GLuint CompileShader(GLenum shaderType, std::string_view fileName) {
	std::vector<unsigned char> data;
	std::string filePath{"shader/"};
	filePath.append(fileName);
	if (!ReadFile(&data, filePath)) {
		FAIL("Could not read shader.", log::Attr{"file", fileName});
	}
	std::string_view source{reinterpret_cast<const char *>(data.data()),
	                        data.size()};

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
GLuint CubeProgram;
GLint MVP;

void Init() {
	GLuint vertex = CompileShader(GL_VERTEX_SHADER, "triangle.vert");
	GLuint fragment = CompileShader(GL_FRAGMENT_SHADER, "triangle.frag");
	GLuint program = LinkProgram(vertex, fragment);
	glDeleteShader(vertex);
	glDeleteShader(fragment);
	Program = program;

	vertex = CompileShader(GL_VERTEX_SHADER, "cube.vert");
	fragment = CompileShader(GL_FRAGMENT_SHADER, "cube.frag");
	program = LinkProgram(vertex, fragment);
	glDeleteShader(vertex);
	glDeleteShader(fragment);
	CubeProgram = program;
	MVP = glGetUniformLocation(program, "MVP");
}

} // namespace gl_shader
} // namespace demo
