// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "os_string.hpp"

#include "log.hpp"

#include <algorithm>
#include <cstdio>
#include <iterator>

namespace demo {

namespace {

// Replace separators in the string with the platform separator.
void NormalizeSeparators(os_char *start, os_char *end) {
	if constexpr (Separator != '/') {
		for (os_char *ptr = start; ptr != end; ptr++) {
			if (*ptr == '/') {
				*ptr = Separator;
			}
		}
	}
}

} // namespace

void AppendPath(os_string *path, std::string_view view) {
	if (path->empty()) {
		FAIL("Path is empty.");
	}
	if ((*path)[path->size() - 1] != Separator) {
		path->push_back(Separator);
	}
	std::size_t pos = path->size();
	std::copy(view.begin(), view.end(), std::back_inserter(*path));
	NormalizeSeparators(path->data() + pos, path->data() + path->size());
}

} // namespace demo

#if WIN32
#include "os_windows.hpp"

#include <limits>

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

std::string ToString(os_string_view value) {
	std::string result;
	Append(&result, value);
	return result;
}

} // namespace demo
#endif
