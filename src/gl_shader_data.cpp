// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl_shader_data.hpp"

namespace demo {
namespace gl_shader {

extern const char ShaderText[];

std::array<ShaderSource, ShaderCount> GetEmbeddedShaderSource() {
	std::array<ShaderSource, ShaderCount> shaders;
	const char *ptr = ShaderText;
	for (auto &shader : shaders) {
		std::size_t length = std::strlen(ptr);
		shader.ptr = ptr;
		shader.size = static_cast<int>(length);
		ptr += length + 1;
	}
	return shaders;
}

extern const std::array<ProgramSpec, ProgramCount> ProgramSpecs = {{
	{0, 2},
	{1, 3},
}};

} // namespace gl_shader
} // namespace demo
