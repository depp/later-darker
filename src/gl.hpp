// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

// This file provides the OpenGL API.

#if __APPLE__

// ============================================================================
// macOS
// ============================================================================

// On macOS, an OpenGL loader is not necessary. We can just get the definitions
// directly from the OpenGL framework.

// OpenGL is deprecated on macOS. We don't care. This silences the warnings.
#define GL_SILENCE_DEPRECATION 1

#include <OpenGL/gl3.h> // IWYU pragma: export

#else

// ============================================================================
// Windows
// ============================================================================

#define GLAPI __stdcall
#define GLIMPORT __declspec(dllimport)

struct __GLsync;

using GLenum = unsigned;
using GLuint = unsigned;
using GLint = int;
using GLsync = __GLsync *;
using GLDEBUGPROC = void(GLAPI *)(GLenum source, GLenum type, unsigned id,
                                  GLenum severity, int length,
                                  const char *message, const void *userParam);

#if COMPO

// Generated interface file, containing minimal OpenGL API.
#include "gl_api_compo.hpp"

#else

// Generated interface file, containing full OpenGL API.
#include "gl_api_full.hpp"

#endif

#endif

// ============================================================================
// Loader
// ============================================================================

namespace demo {
namespace gl_api {

#if _WIN32

// Load OpenGL function pointers.
void LoadProcs();

#else

// Load OpenGL function pointers.
inline void LoadProcs() {}

#endif

// Check which extensions are loaded.
void LoadExtensions();

} // namespace gl_api
} // namespace demo
