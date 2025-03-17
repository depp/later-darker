// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include "gl.hpp"

namespace demo {
namespace gl_shader {

extern GLuint TriangleProgram;
extern GLuint CubeProgram;
extern GLint MVP;

// Compile all OpenGL shader programs.
void Init();

} // namespace gl_shader
} // namespace demo
