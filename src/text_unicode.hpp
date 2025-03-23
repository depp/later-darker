// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

namespace demo {
namespace unicode {

constexpr char32_t ReplacementCharacter = 0xFFFD;

// Return true if the character is a surrogate character.
inline bool IsSurrogate(char32_t ch) {
	return 0xd800u <= ch && ch < 0xe000u;
}

// Return true if the character is a high surrogate character.
inline bool IsSurrogateHigh(char32_t ch) {
	return 0xd800u <= ch && ch < 0xde00u;
}

// Return true if the character is a low surrogate character.
inline bool IsSurrogateLow(char32_t ch) {
	return 0xdc00u <= ch && ch < 0xe000u;
}

// Decode a surrogate pair as a single character.
inline char32_t DecodeSurrogatePair(char16_t high, char16_t low) {
	constexpr unsigned off = (0xd800u << 10) + 0xdc00u - 0x10000u;
	return (static_cast<char32_t>(high) << 10) + low - off;
}

// Result from reading UTF-8 text.
struct UTF8Result {
	const char *ptr;
	char32_t codePoint;
	bool ok;
};

// Read a UTF-8 character from a text buffer.
UTF8Result ReadUTF8(const char *ptr, const char *end);

// Write a UTF-8 character to a text buffer. Return the pointer after the
// written character. At least 4 bytes must be available in the buffer.
char *WriteUTF8(char *ptr, char32_t ch);

} // namespace unicode
} // namespace demo