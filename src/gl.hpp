// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

// This file provides the OpenGL API.

#if __APPLE__
// On macOS, an OpenGL loader is not necessary. We can just get the definitions
// directly from the OpenGL framework.

// OpenGL is deprecated on macOS. We don't care. This silences the warnings.
#define GL_SILENCE_DEPRECATION 1

#include <OpenGL/gl3.h> // IWYU pragma: export

#elif COMPO

struct __GLsync;

using GLenum = unsigned;
using GLuint = unsigned;
using GLint = int;
using GLsync = __GLsync *;

#include "gl_api.hpp"

#else

#include <glad/gl.h> // IWYU pragma: export

#endif
