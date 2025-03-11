// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "text_unicode.hpp"

namespace demo {
namespace unicode {

UTF8Result ReadUTF8(const char *ptr, const char *end) {
	unsigned uc, ucMin, b;

	if (ptr == end) {
		goto invalid;
	}
	uc = static_cast<unsigned char>(*ptr++);

	if (uc < 0x80) {
		// 1-byte sequence.
		goto valid;
	}

	if (uc < 0xc0u) {
		// Continuation character.
		goto invalid;
	} else if (uc < 0xe0u) {
		// 2-byte sequence.
		uc &= 0x1fu;
		ucMin = 0x80;
		goto seq2;
	} else if (uc < 0xf0u) {
		// 3-byte sequence.
		uc &= 0x0fu;
		ucMin = 0x800;
		goto seq3;
	} else if (uc < 0xf8u) {
		// 4-byte sequence.
		uc &= 0x07u;
		ucMin = 0x10000;
		goto seq4;
	} else {
		goto invalid;
	}

seq4:
	if (ptr == end ||
	    ((b = static_cast<unsigned char>(*ptr)) & 0xc0u) != 0x80u) {
		goto invalid;
	}
	ptr++;
	uc = (uc << 6) | (b & 0x3fu);
seq3:
	if (ptr == end ||
	    ((b = static_cast<unsigned char>(*ptr)) & 0xc0u) != 0x80u) {
		goto invalid;
	}
	ptr++;
	uc = (uc << 6) | (b & 0x3fu);
seq2:
	if (ptr == end ||
	    ((b = static_cast<unsigned char>(*ptr)) & 0xc0u) != 0x80u) {
		goto invalid;
	}
	ptr++;
	uc = (uc << 6) | (b & 0x3fu);

	if (uc < ucMin || IsSurrogate(uc)) {
		goto invalid;
	}
valid:
	return UTF8Result{ptr, static_cast<char32_t>(uc), true};

invalid:
	return UTF8Result{ptr, ReplacementCharacter, false};
}

char *WriteUTF8(char *ptr, char32_t ch) {
	if (ch < 0x80u) {
		ptr[0] = static_cast<char>(ch);
		return ptr + 1;
	}
	if (ch < 0x800u) {
		ptr[0] = static_cast<char>((ch >> 6) | 0xc0u);
		ptr[1] = static_cast<char>((ch & 0x3fu) | 0x80u);
		return ptr + 2;
	}
	if (ch < 0x10000u) {
		ptr[0] = static_cast<char>((ch >> 12) | 0xe0u);
		ptr[1] = static_cast<char>(((ch >> 6) & 0x3fu) | 0x80u);
		ptr[2] = static_cast<char>((ch & 0x3fu) | 0x80u);
		return ptr + 3;
	}
	ptr[0] = static_cast<char>((ch >> 18) | 0xe0u);
	ptr[1] = static_cast<char>(((ch >> 12) & 0x3fu) | 0x80u);
	ptr[2] = static_cast<char>(((ch >> 6) & 0x3fu) | 0x80u);
	ptr[3] = static_cast<char>((ch & 0x3fu) | 0x80u);
	return ptr + 4;
}

} // namespace unicode
} // namespace demo
