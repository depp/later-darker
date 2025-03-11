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

void TextBuffer::AppendQuoted(std::string_view str) {
	AppendChar('"');
	AppendEscaped(str);
	AppendChar('"');
}

namespace {

constexpr unsigned ReplacementChar = 0xfffd;

// Append a two-byte character.
char *Append2(char *ptr, unsigned ch) {
	ptr[0] = static_cast<char>(0xc0u | (ch >> 6));
	ptr[1] = static_cast<char>(0x80u | (ch & 0x3fu));
	return ptr + 2;
}

// Append a three-byte character.
char *Append3(char *ptr, unsigned ch) {
	ptr[0] = static_cast<char>(0xe0u | (ch >> 12));
	ptr[1] = static_cast<char>(0x80u | ((ch >> 6) & 0x3fu));
	ptr[2] = static_cast<char>(0x80u | (ch & 0x3fu));
	return ptr + 3;
}

// Append a four-byte character.
char *Append4(char *ptr, unsigned ch) {
	ptr[0] = static_cast<char>(0xf0u | (ch >> 18));
	ptr[1] = static_cast<char>(0x80u | ((ch >> 12) & 0x3fu));
	ptr[2] = static_cast<char>(0x80u | ((ch >> 6) & 0x3fu));
	ptr[3] = static_cast<char>(0x80u | (ch & 0x3fu));
	return ptr + 4;
}

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
	ptr[2] = HexDigit[ch >> 4];
	ptr[3] = HexDigit[ch & 15];
	return ptr + 4;
}

char *AppendHexEscape16(char *ptr, unsigned ch) {
	ptr[0] = '\\';
	ptr[1] = 'u';
	ptr[2] = HexDigit[ch >> 12];
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
	ptr[4] = HexDigit[ch >> 20];
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

	const char *p = str.data(), *e = p + str.size();
	while (p != e) {
		if (mEnd - mPos < MinSpace) {
			Grow();
		}
		unsigned ch = static_cast<unsigned char>(*p++);
		unsigned ch1, ch2, ch3, uch, escape;

		// ASCII characters.
		if (ch < 128) {
			escape = Escape[ch];
			if (escape == 0) {
				AppendChar(static_cast<char>(ch));
			} else if (escape == 'x') {
				goto hexEscape;
			} else {
				mPos[0] = '\\';
				mPos[1] = static_cast<char>(escape);
				mPos += 2;
			}
			continue;
		}

		// Unicode characters.
		if ((ch & 0xe0) == 0xc0) {
			if (e - p < 1) {
				goto hexEscape;
			}
			ch1 = static_cast<unsigned char>(p[0]);
			if ((ch1 & 0xc0) != 0x80) {
				goto hexEscape;
			}
			constexpr unsigned off = (0xc0u << 6) + 0x80u;
			uch = (ch << 6) + ch1 - off;
			if (uch < 0x80) {
				goto hexEscape;
			}
			p += 1;
			goto unicodeEscapeShort;
		}
		if ((ch & 0xf0) == 0xe0) {
			if (e - p < 2) {
				goto hexEscape;
			}
			ch1 = static_cast<unsigned char>(p[0]);
			ch2 = static_cast<unsigned char>(p[1]);
			if ((ch1 & 0xc0) != 0x80 || (ch2 & 0xc0) != 0x80) {
				goto hexEscape;
			}
			constexpr unsigned off = (0xe0u << 12) + (0x80u << 6) + 0x80u;
			uch = (ch << 12) + (ch1 << 6) + ch2 - off;
			if (uch < 0x800 || (0xd800 <= uch && uch < 0xe000)) {
				goto hexEscape;
			}
			p += 2;
			goto unicodeEscapeShort;
		}
		if ((ch & 0xf8) == 0xf0) {
			if (e - p < 3) {
				goto hexEscape;
			}
			ch1 = static_cast<unsigned char>(p[0]);
			ch2 = static_cast<unsigned char>(p[1]);
			ch3 = static_cast<unsigned char>(p[2]);
			constexpr unsigned off =
				(0xf0u << 18) + (0x80u << 12) + (0x80u << 6) + 0x80u;
			uch = (ch << 18) + (ch1 << 12) + (ch2 << 6) + ch3 - off;
			if (uch < 0x10000 || 0x110000 <= uch) {
				goto hexEscape;
			}
			p += 3;
			mPos = AppendHexEscape32(mPos, uch);
			continue;
		}

	hexEscape:
		mPos = AppendHexEscape8(mPos, ch);
		continue;

	unicodeEscapeShort:
		mPos = AppendHexEscape16(mPos, uch);
	}
}

void TextBuffer::AppendWide(std::wstring_view value) {
	constexpr std::size_t MinSpace = 4;
	static_assert(util::GrowSize(0) >= MinSpace, "Wrong growth curve.");

	const wchar_t *ptr = value.data(), *end = ptr + value.size();
	while (ptr != end) {
		if (mEnd - mPos < MinSpace) {
			Grow();
		}
		unsigned ch = static_cast<unsigned short>(*ptr++);
		if (ch < 0x80) {
			*mPos++ = static_cast<char>(ch);
		} else if (ch < 0x800) {
			mPos = Append2(mPos, ch);
		} else if (ch < 0xd800 || 0xe000 <= ch) {
			mPos = Append3(mPos, ch);
		} else {
			constexpr unsigned off = (0xd800u << 10) + 0xdc00u - 0x10000u;
			unsigned ch2;
			if (0xdc00 <= ch || ptr == end) {
				goto replacement;
			}
			ch2 = static_cast<unsigned short>(*ptr);
			if (ch2 < 0xdc00 || 0xe000 <= ch2) {
				goto replacement;
			}
			ptr++;
			ch = (ch << 10) + ch2 - off;
			mPos = Append4(mPos, ch);
			continue;

		replacement:
			mPos = Append2(mPos, ReplacementChar);
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
