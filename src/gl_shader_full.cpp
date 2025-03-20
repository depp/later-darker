// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl_shader.hpp"

#include "gl_shader_data.hpp"
#include "log.hpp"
#include "os_file.hpp"
#include "var.hpp"

#include <array>
#include <string_view>

namespace demo {
namespace gl_shader {

namespace {

// ============================================================================
// Shaders
// ============================================================================

struct Shader {
	GLuint shader;
};

std::array<Shader, ShaderCount> Shaders;

// Compile a shader, given the source code for that shader.
void CompileShader(int shaderId, std::string_view source) {
	Shader &shader = Shaders[shaderId];
	const char *ptr[1] = {source.data()};
	const int len[1] = {static_cast<int>(source.size())};
	glShaderSource(shader.shader, 1, ptr, len);
	glCompileShader(shader.shader);
}

// Compile all shaders using the shader source code embedded in the exeuctable.
void CompileEmbedded() {
	std::array<ShaderSource, ShaderCount> sources = GetEmbeddedShaderSource();
	for (int shaderId = 0; shaderId < ShaderCount; shaderId++) {
		const ShaderSource source = sources[shaderId];
		const std::string_view sourceText{
			source.ptr, static_cast<std::size_t>(source.size)};
		CompileShader(shaderId, sourceText);
	}
}

// TODO: Don't hardcode this.
const std::string_view ShaderFilenames[ShaderCount] = {
	"triangle.vert",
	"cube.vert",
	"triangle.frag",
	"cube.frag",
};

// Compile shaders from the filesystem.
void CompileFiles() {
	std::vector<unsigned char> data;
	std::string filename;
	for (int shaderId = 0; shaderId < ShaderCount; shaderId++) {
		filename.assign("shader/");
		filename.append(ShaderFilenames[shaderId]);
		if (!ReadFile(&data, filename)) {
			FAIL("Could not read shader.", log::Attr{"filename", filename});
		}
		std::string_view sourceText{reinterpret_cast<const char *>(data.data()),
		                            data.size()};
		CompileShader(shaderId, sourceText);
	}
}

// ============================================================================
// Shader Programs
// ============================================================================

struct Program {
	GLuint program;
};

std::array<Program, ProgramCount> Programs;

// Link all shader programs.
void LinkPrograms() {
	// Link and then check status separately. This way, the driver can compile
	// shaders in parallel.
	for (int programId = 0; programId < ProgramCount; programId++) {
		Program &program = Programs[programId];
		glLinkProgram(program.program);
	}

	for (int programId = 0; programId < ProgramCount; programId++) {
		Program &program = Programs[programId];
		GLint status;
		glGetProgramiv(program.program, GL_LINK_STATUS, &status);
		if (!status) {
			FAIL("Shader program failed to link.");
		}
	}

	TriangleProgram = Programs[0].program;
	CubeProgram = Programs[1].program;
	MVP = glGetUniformLocation(CubeProgram, "MVP");
}

} // namespace

// ============================================================================
// Initialization
// ============================================================================

GLuint TriangleProgram;
GLuint CubeProgram;
GLint MVP;

void Init() {
	// Create shader objects.
	for (int shaderId = 0; shaderId < ShaderCount; shaderId++) {
		GLuint shader =
			glCreateShader(shaderId < VertexShaderCount ? GL_VERTEX_SHADER
		                                                : GL_FRAGMENT_SHADER);
		if (shader == 0) {
			FAIL("Could not create shader.");
		}
		Shaders[shaderId].shader = shader;
	}

	// Create shader program objects and attach shaders.
	for (int programId = 0; programId < ProgramCount; programId++) {
		GLuint program = glCreateProgram();
		if (program == 0) {
			FAIL("Could not create program.");
		}
		Programs[programId].program = program;
		const ProgramSpec &spec = ProgramSpecs[programId];
		glAttachShader(program, Shaders[spec.vertex].shader);
		glAttachShader(program, Shaders[spec.fragment].shader);
	}

	// Figure out where shader source code is coming from.
	os_string_view view = var::ProjectPath.get();
	if (var::ProjectPath.get().empty()) {
		// No ProjectPath, so we only have the embedded shaders code. Compile
		// and link, and then destroy the shader objects since we do not need
		// them any more.
		CompileEmbedded();
		LinkPrograms();
		for (int programId = 0; programId < ProgramCount; programId++) {
			const GLuint program = Programs[programId].program;
			const ProgramSpec &spec = ProgramSpecs[programId];
			glDetachShader(program, Shaders[spec.vertex].shader);
			glDetachShader(program, Shaders[spec.fragment].shader);
		}
		for (int shaderId = 0; shaderId < ShaderCount; shaderId++) {
			Shader &shader = Shaders[shaderId];
			glDeleteShader(shader.shader);
			shader.shader = 0;
		}
	} else {
		CompileFiles();
		LinkPrograms();
	}
}

} // namespace gl_shader
} // namespace demo
