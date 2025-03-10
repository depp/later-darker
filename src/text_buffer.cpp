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

} // namespace

void TextBuffer::AppendEscaped(std::string_view str) {
	const char *p = str.data(), *e = p + str.size();
	while (p != e) {
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
				Reserve(2);
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
			AppendHexEscape32(uch);
			continue;
		}

	hexEscape:
		AppendHexEscape8(ch);
		continue;

	unicodeEscapeShort:
		AppendHexEscape16(uch);
	}
}

void TextBuffer::AppendWide(std::wstring_view value) {
	const wchar_t *ptr = value.data(), *end = ptr + value.size();
	while (ptr != end) {
		unsigned ch = static_cast<unsigned short>(*ptr++);
		if (ch < 0x80) {
			if (mPos == mEnd) {
				Grow();
			}
			*mPos++ = static_cast<char>(ch);
		} else if (ch < 0x800) {
			Reserve(2);
			mPos[0] = static_cast<char>(0xc0u | (ch >> 6));
			mPos[1] = static_cast<char>(0x80u | (ch & 0x3fu));
			mPos += 2;
		} else if (ch < 0xd800 || 0xe000 <= ch) {
			Reserve(3);
			mPos[0] = static_cast<char>(0xe0u | (ch >> 12));
			mPos[1] = static_cast<char>(0x80u | ((ch >> 6) & 0x3fu));
			mPos[2] = static_cast<char>(0x80u | (ch & 0x3fu));
			mPos += 3;
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
			Reserve(4);
			mPos[0] = static_cast<char>(0xf0u | (ch >> 18));
			mPos[1] = static_cast<char>(0x80u | ((ch >> 12) & 0x3fu));
			mPos[2] = static_cast<char>(0x80u | ((ch >> 6) & 0x3fu));
			mPos[3] = static_cast<char>(0x80u | (ch & 0x3fu));
			mPos += 4;
			continue;

		replacement:
			Reserve(3);
			mPos[0] = static_cast<char>(0xef);
			mPos[1] = static_cast<char>(0xbf);
			mPos[2] = static_cast<char>(0xbd);
			mPos += 3;
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

void TextBuffer::AppendHexEscape8(unsigned ch) {
	Reserve(4);
	mPos[0] = '\\';
	mPos[1] = 'x';
	mPos[2] = HexDigit[ch >> 4];
	mPos[3] = HexDigit[ch & 15];
	mPos += 4;
}

void TextBuffer::AppendHexEscape16(unsigned ch) {
	Reserve(6);
	mPos[0] = '\\';
	mPos[1] = 'u';
	mPos[2] = HexDigit[ch >> 12];
	mPos[3] = HexDigit[(ch >> 8) & 15];
	mPos[4] = HexDigit[(ch >> 4) & 15];
	mPos[5] = HexDigit[ch & 15];
	mPos += 6;
}

void TextBuffer::AppendHexEscape32(unsigned ch) {
	Reserve(10);
	mPos[0] = '\\';
	mPos[1] = 'U';
	mPos[2] = '0';
	mPos[3] = '0';
	mPos[4] = HexDigit[ch >> 20];
	mPos[5] = HexDigit[(ch >> 16) & 15];
	mPos[6] = HexDigit[(ch >> 12) & 15];
	mPos[7] = HexDigit[(ch >> 8) & 15];
	mPos[8] = HexDigit[(ch >> 4) & 15];
	mPos[9] = HexDigit[ch & 15];
	mPos += 10;
}

} // namespace demo
