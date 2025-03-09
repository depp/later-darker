// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "text_buffer.hpp"

#include "util.hpp"

#include <charconv>
#include <cstdlib>
#include <cstring>
#include <string>

namespace demo {

TextBuffer::~TextBuffer() {
	if (mIsDynamic) {
		std::free(mStart);
	}
}

void TextBuffer::Append(const char *str, size_t count) {
	Reserve(count);
	std::memcpy(mPos, str, count);
	mPos += count;
}

void TextBuffer::Append(const std::string &value) {
	std::string s;
	s.append(std::string_view(value));
	Append(value.data(), value.size());
}

void TextBuffer::AppendNumber(long long value) {
	AppendFunction([value](char *first, char *last) -> char * {
		std::to_chars_result result = std::to_chars(first, last, value);
		return result.ec == std::errc{} ? result.ptr : nullptr;
	});
}

void TextBuffer::AppendNumber(unsigned long long value) {
	AppendFunction([value](char *first, char *last) -> char * {
		std::to_chars_result result = std::to_chars(first, last, value);
		return result.ec == std::errc{} ? result.ptr : nullptr;
	});
}

void TextBuffer::AppendNumber(float value) {
	AppendFunction([value](char *first, char *last) -> char * {
		std::to_chars_result result =
			std::to_chars(first, last, value, std::chars_format::general);
		return result.ec == std::errc{} ? result.ptr : nullptr;
	});
}

void TextBuffer::AppendNumber(double value) {
	AppendFunction([value](char *first, char *last) -> char * {
		std::to_chars_result result =
			std::to_chars(first, last, value, std::chars_format::general);
		return result.ec == std::errc{} ? result.ptr : nullptr;
	});
}

void TextBuffer::AppendBool(bool value) {
	if (value) {
		Append("true", 4);
	} else {
		Append("false", 5);
	}
}

void TextBuffer::Grow() {
	// FIXME: integer overflow?
	std::size_t capacity = mEnd - mStart;
	std::size_t newCapacity = util::GrowSize(capacity);
	Reallocate(newCapacity);
}

void TextBuffer::Reserve(std::size_t size) {
	// FIXME: integer overflow?
	std::size_t capacity = mEnd - mStart;
	std::size_t minimum = (mPos - mStart) + size;
	if (capacity < minimum) {
		Reallocate(util::GrowSizeMinimum(capacity, minimum));
	}
}

void TextBuffer::Reallocate(std::size_t newCapacity) {
	std::ptrdiff_t offset = mPos - mStart;
	char *ptr;
	if (mIsDynamic) {
		ptr = static_cast<char *>(std::realloc(mStart, newCapacity));
		if (ptr == nullptr) {
			std::abort();
		}
	} else {
		ptr = static_cast<char *>(std::malloc(newCapacity));
		if (ptr == nullptr) {
			std::abort();
		}
		if (offset > 0) {
			std::memcpy(ptr, mStart, offset);
		}
	}
	mStart = ptr;
	mPos = ptr + offset;
	mEnd = ptr + newCapacity;
	mIsDynamic = true;
}

} // namespace demo
