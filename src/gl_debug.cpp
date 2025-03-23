// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl_debug.hpp"

#include "gl.hpp" // IWYU pragma: keep
#include "log.hpp"

#if GL_KHR_debug

#include <string_view>

namespace demo {
namespace gl_debug {

namespace {

void GLAPIENTRY DebugCallback(GLenum source, GLenum type, GLenum id,
                              GLenum severity, GLsizei length,
                              const GLchar *message, const void *userParam) {
	(void)source;
	(void)type;
	(void)id;
	(void)userParam;

	std::string_view messageText = length >= 0
	                                   ? std::string_view{message}
	                                   : std::string_view(message, length);

	log::Level level;
	switch (severity) {
	default:
	case GL_DEBUG_SEVERITY_HIGH:
		level = log::Level::Error;
		break;
	case GL_DEBUG_SEVERITY_MEDIUM:
		level = log::Level::Warn;
		break;
	case GL_DEBUG_SEVERITY_LOW:
		level = log::Level::Info;
		break;
	case GL_DEBUG_SEVERITY_NOTIFICATION:
		level = log::Level::Debug;
		break;
	}

	log::Record{level, log::Location::Zero, "OpenGL",
	            log::Attr("message", messageText)}
		.Log();
}

} // namespace

void Init() {
	if (!GLAD_GL_KHR_debug) {
		return;
	}

	LOG(Info, "Using KHR_debug.");
	glDebugMessageCallback(DebugCallback, nullptr);
	glDebugMessageControl(GL_DONT_CARE, GL_DONT_CARE, GL_DONT_CARE, 0, nullptr,
	                      GL_TRUE);
	glEnable(GL_DEBUG_OUTPUT);
}

} // namespace gl_debug
} // namespace demo

#else

namespace demo {
namespace gl_debug {

void Init() {
	LOG(Debug, "KHR_debug not available.");
}

} // namespace gl_debug
} // namespace demo

#endif
