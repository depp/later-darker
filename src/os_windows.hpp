// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include <cstdlib>
#include <string>

#define NOMINMAX 1
#define UNICODE 1
#define WIN32_LEAN_AND_MEAN 1
#include <Windows.h>

namespace demo {
namespace log {
class Record;
}

// A windows error code.
class WindowsError {
public:
	explicit WindowsError(DWORD errorCode);
	void AddToRecord(log::Record &record) const;

	static WindowsError GetLast();

private:
	DWORD mErrorCode;
	std::wstring mText;
};

// Object for cleaning up a Windows handle.
class HandleCloser {
public:
	explicit HandleCloser(HANDLE h) : mHandle{h} {}
	HandleCloser(const HandleCloser &) = delete;
	const HandleCloser &operator=(const HandleCloser &) = delete;
	~HandleCloser() { CloseHandle(mHandle); }

private:
	HANDLE mHandle;
};

// Pack two 32-bit values into a 64-bit value.
inline uint64_t Pack64(uint32_t hi, uint32_t lo) {
	return (static_cast<uint64_t>(hi) << 32) | lo;
}

// Dump environment variables.
void DumpEnv();

} // namespace demo
