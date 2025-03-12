// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "os_windows.hpp"

#include "log.hpp"
#include "util.hpp"

namespace demo {

namespace {

std::wstring GetErrorText(DWORD errorCode) {
	std::wstring text;
	constexpr DWORD flags =
		FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS;
	constexpr DWORD langId = MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT);
	text.resize(127);
	DWORD result = FormatMessageW(
		flags, nullptr, errorCode, MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
		text.data(), static_cast<DWORD>(text.size()), nullptr);
	if (result != 0) {
		text.resize(result);
	} else if (GetLastError() == ERROR_MORE_DATA) {
		LPWSTR textPtr;
		result = FormatMessageW(FORMAT_MESSAGE_ALLOCATE_BUFFER | flags, nullptr,
		                        errorCode, langId,
		                        reinterpret_cast<LPWSTR>(&textPtr), 0, nullptr);
		if (result != 0) {
			// FIXME: free even on crash.
			text.assign(textPtr, result);
			LocalFree(textPtr);
		}
	}
	if (text.size() >= 2 && text.compare(text.size() - 2, 2, L"\r\n") == 0) {
		text.resize(text.size() - 2);
	}
	return text;
}

} // namespace

WindowsError::WindowsError(DWORD errorCode)
	: mErrorCode{errorCode}, mText{GetErrorText(errorCode)} {}

void WindowsError::AddToRecord(log::Record &record) const {
	if (mErrorCode != 0) {
		record.Add("error", mErrorCode);
		if (!mText.empty()) {
			record.Add("description", mText);
		}
	}
}

WindowsError WindowsError::GetLast() {
	return WindowsError(GetLastError());
}

} // namespace demo
