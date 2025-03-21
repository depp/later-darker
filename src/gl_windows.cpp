// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
//build: windows && !compo
#include "gl.hpp"

#include "log.hpp"
#include "os_windows.hpp"

namespace demo {
namespace gl_api {

void LoadProcs() {
	const char *namePtr = gl_api::FunctionNames;
	for (int i = 0; i < gl_api::FunctionPointerCount; i++) {
		PROC proc = wglGetProcAddress(namePtr);
		if (proc == nullptr) {
			FAIL("Could not load OpenGL function.",
			     log::Attr{"function", namePtr});
		}
		FunctionPointers[i] = static_cast<void *>(proc);
		namePtr += std::strlen(namePtr) + 1;
	}
}

} // namespace gl_api
} // namespace demo
