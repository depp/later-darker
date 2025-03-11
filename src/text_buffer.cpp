// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "text_buffer.hpp"

#include "text_unicode.hpp"
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

void TextBuffer::AppendQuoted(std::string_view str) {
	AppendChar('"');
	AppendEscaped(str);
	AppendChar('"');
}

namespace {

const unsigned char Escape[128] = {
	'x', 'x', 'x', 'x', 'x',  'x', 'x', 'x', //
	'x', 't', 'n', 'x', 'x',  'r', 'x', 'x', //
	'x', 'x', 'x', 'x', 'x',  'x', 'x', 'x', //
	'x', 'x', 'x', 'x', 'x',  'x', 'x', 'x', //
	0,   0,   '"', 0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   '\\', 0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   0,   //
	0,   0,   0,   0,   0,    0,   0,   'x', //
};

const char HexDigit[16] = {'0', '1', '2', '3', '4', '5', '6', '7',
                           '8', '9', 'a', 'b', 'c', 'd', 'e', 'f'};

char *AppendHexEscape8(char *ptr, unsigned ch) {
	ptr[0] = '\\';
	ptr[1] = 'x';
	ptr[2] = HexDigit[(ch >> 4) & 15];
	ptr[3] = HexDigit[ch & 15];
	return ptr + 4;
}

char *AppendHexEscape16(char *ptr, unsigned ch) {
	ptr[0] = '\\';
	ptr[1] = 'u';
	ptr[2] = HexDigit[(ch >> 12) & 15];
	ptr[3] = HexDigit[(ch >> 8) & 15];
	ptr[4] = HexDigit[(ch >> 4) & 15];
	ptr[5] = HexDigit[ch & 15];
	return ptr + 6;
}

char *AppendHexEscape32(char *ptr, unsigned ch) {
	ptr[0] = '\\';
	ptr[1] = 'U';
	ptr[2] = '0';
	ptr[3] = '0';
	ptr[4] = HexDigit[(ch >> 20) & 15];
	ptr[5] = HexDigit[(ch >> 16) & 15];
	ptr[6] = HexDigit[(ch >> 12) & 15];
	ptr[7] = HexDigit[(ch >> 8) & 15];
	ptr[8] = HexDigit[(ch >> 4) & 15];
	ptr[9] = HexDigit[ch & 15];
	return ptr + 10;
}

} // namespace

void TextBuffer::AppendEscaped(std::string_view str) {
	constexpr std::size_t MinSpace = 10;
	static_assert(util::GrowSize(0) >= MinSpace, "Wrong growth curve.");

	const char *ptr = str.data(), *end = ptr + str.size();
	while (ptr != end) {
		if (mEnd - mPos < MinSpace) {
			Grow();
		}
		unsigned ch = static_cast<unsigned char>(*ptr);
		if (ch < 0x80) {
			ptr++;
			unsigned escape = Escape[ch];
			if (escape == 0) {
				*mPos++ = static_cast<char>(ch);
			} else if (escape == 'x') {
				mPos = AppendHexEscape8(mPos, ch);
			} else {
				mPos[0] = '\\';
				mPos[1] = static_cast<char>(escape);
				mPos += 2;
			}
		} else {
			unicode::UTF8Result result = unicode::ReadUTF8(ptr, end);
			if (result.ok) {
				ptr = result.ptr;
				if (result.codePoint < 0x10000) {
					mPos = AppendHexEscape16(mPos, result.codePoint);
				} else {
					mPos = AppendHexEscape32(mPos, result.codePoint);
				}
			} else {
				ptr++;
				mPos = AppendHexEscape8(mPos, ch);
			}
		}
	}
}

void TextBuffer::AppendWide(std::wstring_view value) {
	constexpr std::size_t MinSpace = 4;
	static_assert(util::GrowSize(0) >= MinSpace, "Wrong growth curve.");

	const wchar_t *ptr = value.data(), *end = ptr + value.size();
	while (ptr != end) {
		unsigned ch, ch2;
		if (mEnd - mPos < MinSpace) {
			Grow();
		}
		ch = static_cast<unsigned short>(*ptr++);
		if (ch < 0x80) {
			*mPos++ = static_cast<char>(ch);
		} else {
			if (unicode::IsSurrogate(ch)) {
				if (unicode::IsSurrogateHigh(ch) && ptr != end &&
				    unicode::IsSurrogateLow(ch2 = *ptr)) {
					ptr++;
					ch = unicode::DecodeSurrogatePair(ch, ch2);
				} else {
					ch = unicode::ReplacementCharacter;
				}
			}
			mPos = unicode::WriteUTF8(mPos, ch);
		}
	}
}

void TextBuffer::AppendWideQuoted(std::wstring_view str) {
	AppendChar('"');
	AppendWideEscaped(str);
	AppendChar('"');
}

void TextBuffer::AppendWideEscaped(std::wstring_view value) {
	constexpr std::size_t MinSpace = 4;
	static_assert(util::GrowSize(0) >= MinSpace, "Wrong growth curve.");

	const wchar_t *ptr = value.data(), *end = ptr + value.size();
	while (ptr != end) {
		unsigned ch, ch2, escape;
		if (mEnd - mPos < MinSpace) {
			Grow();
		}
		ch = static_cast<unsigned short>(*ptr++);
		if (ch < 0x80) {
			unsigned escape = Escape[ch];
			if (escape == 0) {
				*mPos++ = static_cast<char>(ch);
			} else if (escape == 'x') {
				mPos = AppendHexEscape8(mPos, ch);
			} else {
				mPos[0] = '\\';
				mPos[1] = static_cast<char>(escape);
				mPos += 2;
			}
		} else {
			if (unicode::IsSurrogate(ch)) {
				if (unicode::IsSurrogateHigh(ch) && ptr != end &&
				    unicode::IsSurrogateLow(ch2 = *ptr)) {
					ptr++;
					mPos = AppendHexEscape32(
						mPos, unicode::DecodeSurrogatePair(ch, ch2));
				} else {
					mPos = AppendHexEscape16(mPos, ch);
				}
			} else {
				mPos = AppendHexEscape16(mPos, ch);
			}
		}
	}
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
