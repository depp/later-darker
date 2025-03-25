// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "gl.hpp"

#include <string_view>
#include <unordered_map>

namespace demo {
namespace gl_api {

void LoadExtensions() {
	if constexpr (ExtensionCount > 0) {
		// Create a map from extension names to indexes.
		const char *namePtr = ExtensionNames;
		std::unordered_map<std::string_view, int> extensions;
		extensions.reserve(ExtensionCount);
		for (int i = 0; i < ExtensionCount; i++) {
			std::size_t length = std::strlen(namePtr);
			extensions.insert({std::string_view(namePtr, length), i});
			namePtr += length + 1;
		}

		// Look up the extensions present, and map them to indexes.
		int extensionCount = 0;
		glGetIntegerv(GL_NUM_EXTENSIONS, &extensionCount);
		for (int i = 0; i < extensionCount; i++) {
			const char *const ptr =
				reinterpret_cast<const char *>(glGetStringi(GL_EXTENSIONS, i));
			const std::string_view name{ptr, std::strlen(ptr)};
			const auto value = extensions.find(name);
			if (value != extensions.end()) {
				ExtensionAvailable[value->second] = true;
			}
		}
	}
}

} // namespace gl_api
} // namespace demo
