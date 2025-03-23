// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include <string_view>

namespace demo {

// Automatically growable text buffer containing wide chars. This has reduced
// functionality compared to a normal text buffer. The intention is that strings
// will be constructed in UTF-8, and converted into wide strings at the last
// moment.
class WideTextBuffer {
public:
	// Initialize an empty text buffer.
	WideTextBuffer()
		: mStart{nullptr}, mPos{nullptr}, mEnd{nullptr}, mIsDynamic{0} {}

	// Initialize the text buffer with existing, preallocated storage. This
	// storage will be used by the buffer until the buffer grows beyond that
	// size, at which point it will dynamically allocate new memory.
	template <size_t N>
	explicit WideTextBuffer(wchar_t (&arr)[N])
		: mStart{arr}, mPos{arr}, mEnd{arr + N}, mIsDynamic{false} {}

	~WideTextBuffer();

	WideTextBuffer(const WideTextBuffer &) = delete;
	WideTextBuffer &operator=(const WideTextBuffer) = delete;

	// Get the start of the buffer, where text has been written.
	wchar_t *Start() { return mStart; }
	// Get the writing pos, where new text will be written.
	wchar_t *Pos() { return mPos; }
	// Get the pointer past the end of available space.
	wchar_t *End() { return mEnd; }
	// Get the amount written.
	std::size_t Size() const { return mPos - mStart; }
	// Get the amount of space available.
	std::size_t Avail() const { return mEnd - mPos; }
	// Get the data written to the buffer.
	std::wstring_view Contents() const {
		return std::wstring_view(mStart, mPos - mStart);
	}

	// Append a multi-byte string.
	void AppendMultiByte(std::string_view data);

	// Append a wide character string.
	void AppendWideChar(std::wstring_view data);

	// Clear the text buffer, but do not release storage.
	void Clear() { mPos = mStart; }

	// Increase the amount of available space to write.
	void Grow();

	// Reserve space for writing the given number of characters.
	void Reserve(std::size_t size);

private:
	// Reallocate the buffer, with the given new capacity.
	void Reallocate(std::size_t newCapacity);

	wchar_t *mStart;
	wchar_t *mPos;
	wchar_t *mEnd;
	bool mIsDynamic;
};

} // namespace demo
