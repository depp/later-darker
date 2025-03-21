// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
//build: windows && !compo
#include "wide_text_buffer.hpp"

#include "log.hpp"
#include "os_windows.hpp"
#include "util.hpp"

#include <cstdlib>
#include <limits>

namespace demo {

WideTextBuffer::~WideTextBuffer() {
	if (mIsDynamic) {
		std::free(mStart);
	}
}

void WideTextBuffer::AppendMultiByte(std::string_view data) {
	if (data.empty()) {
		return;
	}
	// Check for ASCII fast path.
	unsigned bits = 0;
	for (char c : data) {
		bits |= static_cast<unsigned char>(c);
	}
	if ((bits & 0x80) == 0) {
		// Fast path: ASCII.
		Reserve(data.size());
		mPos = std::copy(data.data(), data.data() + data.size(), mPos);
	} else {
		constexpr std::size_t max =
			static_cast<std::size_t>(std::numeric_limits<int>::max());
		CHECK(data.size() <= max);
		int nChars = static_cast<int>(data.size());
		int nWideChars =
			MultiByteToWideChar(CP_UTF8, 0, data.data(), nChars, nullptr, 0);
		Reserve(nWideChars);
		MultiByteToWideChar(CP_UTF8, 0, data.data(), nChars, mPos, nWideChars);
		mPos += nWideChars;
	}
}

void WideTextBuffer::AppendWideChar(std::wstring_view data) {
	Reserve(data.size());
	std::memcpy(mPos, data.data(), sizeof(wchar_t) * data.size());
	mPos += data.size();
}

void WideTextBuffer::Grow() {
	// FIXME: integer overflow?
	std::size_t capacity = mEnd - mStart;
	std::size_t newCapacity = util::GrowSize(capacity);
	Reallocate(newCapacity);
}

void WideTextBuffer::Reserve(std::size_t size) {
	// FIXME: integer overflow?
	std::size_t capacity = mEnd - mStart;
	std::size_t minimum = (mPos - mStart) + size;
	if (capacity < minimum) {
		Reallocate(util::GrowSizeMinimum(capacity, minimum));
	}
}

void WideTextBuffer::Reallocate(std::size_t newCapacity) {
	std::ptrdiff_t offset = mPos - mStart;
	std::size_t size = newCapacity * sizeof(wchar_t);
	wchar_t *ptr;
	if (mIsDynamic) {
		ptr = static_cast<wchar_t *>(std::realloc(mStart, size));
		if (ptr == nullptr) {
			FAIL_ALLOC(size);
		}
	} else {
		ptr = static_cast<wchar_t *>(std::malloc(size));
		if (ptr == nullptr) {
			FAIL_ALLOC(size);
		}
		if (offset > 0) {
			// Analysis false positive: Analysis concludes that there can be a
			// buffer overrun here.
#pragma warning(suppress : 6386)
			std::memcpy(ptr, mStart, offset * sizeof(wchar_t));
		}
	}
	mStart = ptr;
	mPos = ptr + offset;
	mEnd = ptr + newCapacity;
	mIsDynamic = true;
}

} // namespace demo
