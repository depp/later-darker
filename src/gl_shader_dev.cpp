// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl_shader.hpp"

#include "gl_shader_data.hpp"
#include "log.hpp"
#include "os_file.hpp"
#include "var.hpp"

#include <array>
#include <cstdio>
#include <cstring>
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

// Compile the shaders that have been embedded into the program.
void CompileEmbedded() {
	std::array<ShaderSource, ShaderCount> source = GetEmbeddedShaderSource();
	std::array<GLuint, ShaderCount> shaders;
	for (int i = 0; i < ShaderCount; i++) {
		GLuint shader = glCreateShader(
			i < VertexShaderCount ? GL_VERTEX_SHADER : GL_FRAGMENT_SHADER);
		if (shader == 0) {
			FAIL("Could not create shader.");
		}
		shaders[i] = shader;
		glShaderSource(shader, 1, &source[i].ptr, &source[i].size);
		glCompileShader(shader);
	}
	std::array<GLuint, ProgramCount> programs;
	for (int i = 0; i < ProgramCount; i++) {
		GLuint program = glCreateProgram();
		if (program == 0) {
			FAIL("Could not create program.");
		}
		programs[i] = program;
		const ProgramSpec &spec = ProgramSpecs[i];
		glAttachShader(program, shaders[spec.vertex]);
		glAttachShader(program, shaders[spec.fragment]);
		glLinkProgram(program);

		GLint status;
		glGetProgramiv(program, GL_LINK_STATUS, &status);
		if (!status) {
			FAIL("Shader program failed to link.");
		}

		glDetachShader(program, shaders[spec.vertex]);
		glDetachShader(program, shaders[spec.fragment]);
	}
	for (int i = 0; i < ShaderCount; i++) {
		glDeleteShader(shaders[i]);
	}
	TriangleProgram = programs[0];
	CubeProgram = programs[1];
	MVP = glGetUniformLocation(CubeProgram, "MVP");
}

// Compile shaders from the filesystem.
void CompileFiles() {
	GLuint vertex = CompileShader(GL_VERTEX_SHADER, "triangle.vert");
	GLuint fragment = CompileShader(GL_FRAGMENT_SHADER, "triangle.frag");
	GLuint program = LinkProgram(vertex, fragment);
	glDeleteShader(vertex);
	glDeleteShader(fragment);
	TriangleProgram = program;

	vertex = CompileShader(GL_VERTEX_SHADER, "cube.vert");
	fragment = CompileShader(GL_FRAGMENT_SHADER, "cube.frag");
	program = LinkProgram(vertex, fragment);
	glDeleteShader(vertex);
	glDeleteShader(fragment);
	CubeProgram = program;
}

} // namespace

GLuint TriangleProgram;
GLuint CubeProgram;
GLint MVP;

void Init() {
	os_string_view view = var::ProjectPath.get();
	if (var::ProjectPath.get().empty()) {
		CompileEmbedded();
	} else {
		CompileFiles();
	}

	MVP = glGetUniformLocation(CubeProgram, "MVP");
}

} // namespace gl_shader
} // namespace demo
