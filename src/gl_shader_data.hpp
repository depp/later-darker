// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once
#include <array>

namespace demo {
namespace gl_shader {

// FIXME: These are hard-coded. They should be generated.

constexpr int ShaderCount = 4;
constexpr int VertexShaderCount = 2;
constexpr int ProgramCount = 2;

// The source code for a shader.
struct ShaderSource {
	const char *ptr;
	int size;
};

// Get the source code for shaders embedded in the program.
std::array<ShaderSource, ShaderCount> GetEmbeddedShaderSource();

// Specification for a shader program.
struct ProgramSpec {
	int vertex;   // Index into shader array.
	int fragment; // Index into shader array.
};

// Specifications for all programs.
extern const std::array<ProgramSpec, ProgramCount> ProgramSpecs;

} // namespace gl_shader
} // namespace demo
