// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
//build: !compo
#pragma once

#include <string>
#include <string_view>

namespace demo {

#if _WIN32
// OS-native character type. For Windows, this is wchar_t. Otherwise it is char.
using os_char = wchar_t;
constexpr os_char Separator = L'\\';
#else
using os_char = char;
constexpr os_char Separator = '/';
#endif

using os_string_view = std::basic_string_view<os_char>;
using os_string = std::basic_string<os_char>;

#if _WIN32

// Append an OS-native string to a UTF-8 string.
void Append(std::string *dest, os_string_view value);

// Convert an OS-native string to a UTF-8 string.
std::string ToString(os_string_view value);

#else

inline std::string ToString(os_string_view value) {
	return std::string{value};
}

#endif

// Append a relative path to an existing path. The relative path must be
// non-empty and not start with a slash or dot.
void AppendPath(os_string *path, std::string_view view);

} // namespace demo
