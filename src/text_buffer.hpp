// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include <cstddef>
#include <cstring>
#include <string>
#include <string_view>
#include <type_traits>

namespace demo {

// Automatically growable text buffer.
class TextBuffer {
public:
	// Initialize an empty text buffer.
	TextBuffer()
		: mStart{nullptr}, mPos{nullptr}, mEnd{nullptr}, mIsDynamic{false} {}

	// Initialize the text buffer with existing, preallocated storage. This
	// storage will be used by the buffer until the buffer grows beyond that
	// size, at which point it will dynamically allocate new memory.
	template <size_t N>
	explicit TextBuffer(char (&arr)[N])
		: mStart{arr}, mPos{arr}, mEnd{arr + N}, mIsDynamic{false} {}

	~TextBuffer();

	TextBuffer(const TextBuffer &) = delete;
	TextBuffer &operator=(const TextBuffer) = delete;

	// Get the start of the buffer, where text has been written.
	char *Start() { return mStart; }
	// Get the writing pos, where new text will be written.
	char *Pos() { return mPos; }
	// Get the pointer past the end of available space.
	char *End() { return mEnd; }
	// Get the amount written.
	std::size_t Size() const { return mPos - mStart; }
	// Get the amount of space available.
	std::size_t Avail() const { return mEnd - mPos; }
	// Get the data written to the buffer.
	std::string_view Contents() const {
		return std::string_view(mStart, mPos - mStart);
	}

	// Append a single character.
	void AppendChar(char c) {
		if (mPos == mEnd) {
			Grow();
		}
		*mPos++ = c;
	}

	// Append a string.
	void Append(const char *str, size_t count);
	// Append a string.
	void Append(const std::string &value);
	// Append a string, null-terminated.
	void Append(const char *str) { Append(str, std::strlen(str)); }

	// Append a string view, or something convertible to a string view.
	template <
		typename SV,
		std::enable_if_t<
			std::conjunction_v<
				std::is_convertible<const SV &, std::basic_string_view<char>>,
				std::negation<std::is_convertible<const SV &, const char *>>>,
			bool> = true>
	void Append(const SV &value) {
		const std::string_view view = value;
		Append(view.data(), view.size());
	}

	// Append a string, enclosed in quotes, with characters escaped.
	void AppendQuoted(std::string_view str);

	// Append a string with the characters escaped as necessary.
	void AppendEscaped(std::string_view str);

	// Append a wide character string.
	void AppendWide(const std::wstring_view value);

	// Append a string, enclosed in quotes, with characters escaped.
	void AppendWideQuoted(std::wstring_view str);

	// Append a wide string with the characters escaped as necessary.
	void AppendWideEscaped(std::wstring_view value);

	// Append a number.
	void AppendNumber(long long value);
	// Append a number.
	void AppendNumber(unsigned long long value);
	// Append a number.
	void AppendNumber(float value);
	// Append a number.
	void AppendNumber(double value);

	// Append a number.
	template <
		typename Integer,
		std::enable_if_t<std::conjunction_v<
							 std::is_integral<Integer>, std::is_signed<Integer>,
							 std::negation<std::is_same<
								 long long, std::remove_cvref_t<Integer>>>>,
	                     bool> = true>
	void AppendNumber(Integer value) {
		AppendNumber(static_cast<long long>(value));
	}

	// Append a number.
	template <typename Integer,
	          std::enable_if_t<
				  std::conjunction_v<
					  std::is_integral<Integer>, std::is_unsigned<Integer>,
					  std::negation<std::is_same<
						  unsigned long long, std::remove_cvref_t<Integer>>>>,
				  bool> = true>
	void AppendNumber(Integer value) {
		AppendNumber(static_cast<unsigned long long>(value));
	}

	// Append a boolean.
	void AppendBool(bool value);

	// Append using a function. The function is called with larger and larger
	// buffer sizes until it succeeds. The function should return nullptr if it
	// fails, or a pointer past the last character written if it succeeds.
	template <std::invocable<char *, char *> Function>
	void AppendFunction(Function f) {
		if (Avail() == 0) {
			Grow();
		}
		for (;;) {
			char *pos = f(mPos, mEnd);
			if (pos != nullptr) {
				mPos = pos;
				return;
			}
			Grow();
		}
	}

	// Clear the text buffer, but do not release storage.
	void Clear() { mPos = mStart; }

	// Increase the amount of available space to write.
	void Grow();

	// Reserve space for writing the given number of characters.
	void Reserve(std::size_t size);

private:
	// Reallocate the buffer, with the given new capacity.
	void Reallocate(std::size_t newCapacity);

	char *mStart;
	char *mPos;
	char *mEnd;
	bool mIsDynamic;
};

} // namespace demo
