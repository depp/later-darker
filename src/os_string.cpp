// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once
#include "os_string.hpp"

#include <cstdio>
#include <limits>

#define NOMINMAX 1
#define UNICODE 1
#define WIN32_LEAN_AND_MEAN 1
#include <Windows.h>

namespace demo {

std::string ToString(os_string_view value) {
	std::string result;
	if (!value.empty()) {
		if (value.size() > std::numeric_limits<int>::max()) {
			std::abort();
		}
		int nWideChars = static_cast<int>(value.size());
		int nChars = WideCharToMultiByte(CP_UTF8, 0, value.data(), nWideChars,
		                                 nullptr, 0, nullptr, nullptr);
		result.resize(nChars);
		WideCharToMultiByte(CP_UTF8, 0, value.data(), nWideChars, result.data(),
		                    nChars, nullptr, nullptr);
	}
	return result;
}

os_string ToOSString(std::string_view value) {
	os_string result;
	if (!value.empty()) {
		if (value.size() > std::numeric_limits<int>::max()) {
			std::abort();
		}
		int nChars = static_cast<int>(value.size());
		int nWideChars =
			MultiByteToWideChar(CP_UTF8, 0, value.data(), nChars, nullptr, 0);
		result.resize(nWideChars);
		MultiByteToWideChar(CP_UTF8, 0, value.data(), nChars, result.data(),
		                    nWideChars);
	}
	return result;
}

} // namespace demo
