// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include <string>
#include <string_view>

namespace demo
{

// OS-native character type. For Windows, this is wchar_t. Otherwise it is char.
using os_char = wchar_t;
using os_string_view = std::basic_string_view<os_char>;
using os_string = std::basic_string<os_char>;

// Convert an OS-native string to a UTF-8 string.
std::string ToString(os_string_view value);

// Convert a UTF-8 string to an OS-native string.
os_string ToOSString(std::string_view value);

} // namespace demo
