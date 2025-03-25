// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl.hpp"

#include "os_windows.hpp"

namespace demo {
namespace gl_api {

void LoadProcs() {
	const char *namePtr = FunctionNames;
	for (int i = 0; i < FunctionPointerCount; i++) {
		PROC proc = wglGetProcAddress(namePtr);
		FunctionPointers[i] = static_cast<void *>(proc);
		namePtr += std::strlen(namePtr) + 1;
	}
}

} // namespace gl_api
} // namespace demo
