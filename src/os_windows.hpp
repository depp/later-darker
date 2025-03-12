// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

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

} // namespace demo
