// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once
#include "os_string.hpp"

#include "log.hpp"

#include <cstdio>
#include <limits>

#define NOMINMAX 1
#define UNICODE 1
#define WIN32_LEAN_AND_MEAN 1
#include <Windows.h>

namespace demo {

void Append(std::string *dest, os_string_view value) {
	if (value.empty()) {
		return;
	}
	// FIXME: This isn't recursive, right?
	CHECK(value.size() <= std::numeric_limits<int>::max());
	int nWideChars = static_cast<int>(value.size());
	int nChars = WideCharToMultiByte(CP_UTF8, 0, value.data(), nWideChars,
	                                 nullptr, 0, nullptr, nullptr);
	size_t offset = dest->size();
	dest->resize(offset + nChars);
	WideCharToMultiByte(CP_UTF8, 0, value.data(), nWideChars,
	                    dest->data() + offset, nChars, nullptr, nullptr);
}

void Append(os_string *dest, std::string_view value) {
	if (value.empty()) {
		return;
	}
	// FIXME: Recursive?
	CHECK(value.size() <= std::numeric_limits<int>::max());
	int nChars = static_cast<int>(value.size());
	int nWideChars =
		MultiByteToWideChar(CP_UTF8, 0, value.data(), nChars, nullptr, 0);
	size_t offset = dest->size();
	dest->resize(offset + nWideChars);
	MultiByteToWideChar(CP_UTF8, 0, value.data(), nChars, dest->data() + offset,
	                    nWideChars);
}

std::string ToString(os_string_view value) {
	std::string result;
	Append(&result, value);
	return result;
}

os_string ToOSString(std::string_view value) {
	os_string result;
	Append(&result, value);
	return result;
}

} // namespace demo
