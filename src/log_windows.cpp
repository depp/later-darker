// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log_internal.hpp"

#include "log.hpp"
#include "os_windows.hpp"
#include "var.hpp"

// Note:
// https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences
//
// We use standard terminal sequences for colors.

namespace demo {
namespace log {

namespace {

HANDLE ConsoleHandle;

} // namespace

bool WindowsWriter::Init() {
	if (!var::AllocConsole) {
		return false;
	}
	BOOL ok = AllocConsole();
	if (!ok) {
		FAIL("Failed to create console.", WindowsError::GetLast());
	}
	HANDLE console = CreateFileW(L"CONOUT$", GENERIC_WRITE, FILE_SHARE_WRITE,
	                             nullptr, OPEN_EXISTING, 0, nullptr);
	if (console == INVALID_HANDLE_VALUE) {
		FAIL("Failed to open console.", WindowsError::GetLast());
	}
	ok = SetConsoleMode(console, ENABLE_PROCESSED_OUTPUT |
	                                 ENABLE_WRAP_AT_EOL_OUTPUT |
	                                 ENABLE_VIRTUAL_TERMINAL_PROCESSING);
	if (!ok) {
		FAIL("Failed to set console mode.", WindowsError::GetLast());
	}
	ConsoleHandle = console;
	return true;
}

void WindowsWriter::Log(const Record &record) {
	if (ConsoleHandle == nullptr) {
		return;
	}

	mBuffer.Clear();
	WriteLine(mBuffer, record, true);

	mWideBuffer.Clear();
	mWideBuffer.AppendMultiByte(mBuffer.Contents());
	// FIXME: overflow check?
	DWORD size = static_cast<DWORD>(mWideBuffer.Size());
	DWORD written;
	WriteConsoleW(ConsoleHandle, mWideBuffer.Start(), size, &written, nullptr);
}

[[noreturn]]
void WindowsWriter::Fail(const Record &record) {
	mBuffer.Clear();
	WriteBlock(mBuffer, record);
	mBuffer.AppendChar('\0');

	mWideBuffer.Clear();
	mWideBuffer.AppendMultiByte(buffer.buffer.Contents());
	MessageBoxW(nullptr, mWideBuffer.Start(), nullptr, MB_ICONSTOP);
	ExitError();
}

} // namespace log
} // namespace demo
