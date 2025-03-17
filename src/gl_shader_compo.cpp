// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl_shader.hpp"

#include "gl_shader_data.hpp"
#include "log.hpp"

#include <array>

namespace demo {
namespace gl_shader {

GLuint Program;
GLuint CubeProgram;
GLint MVP;

// Compile the shaders that have been embedded into the program.
void Init() {
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
	}
	Program = programs[0];
	CubeProgram = programs[1];
	MVP = glGetUniformLocation(CubeProgram, "MVP");
}

} // namespace gl_shader
} // namespace demo
