// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log.hpp"

#include "var.hpp"

#include <cstdio>
#include <limits>
#include <string>

// Note:
// https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences

#define NOMINMAX 1
#define UNICODE 1
#define WIN32_LEAN_AND_MEAN 1
#include <Windows.h>

namespace demo {

namespace {

HANDLE ConsoleHandle;

struct LogLevelInfo {
	std::string_view color;
	std::string_view name;
};

const LogLevelInfo LogLevels[] = {
	{"\x1b[36m", "DEBUG"},
	{"", "INFO"},
	{"\x1b[33m", "WARN"},
	{"\x1b[31m", "ERROR"},
};

const LogLevelInfo &GetLogLevelInfo(LogLevel level) {
	int index = static_cast<int>(level);
	if (index < 0 || std::size(LogLevels) <= index) {
		std::abort();
	}
	return LogLevels[index];
}

} // namespace

void LogInit() {
	if (!var::AllocConsole) {
		return;
	}
	BOOL ok = AllocConsole();
	if (!ok) {
		std::abort();
	}
	HANDLE console = CreateFileW(L"CONOUT$", GENERIC_WRITE, FILE_SHARE_WRITE,
	                             nullptr, OPEN_EXISTING, 0, nullptr);
	if (console == INVALID_HANDLE_VALUE) {
		DWORD err = GetLastError();
		std::abort();
	}
	ok = SetConsoleMode(
		console, ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
	if (!ok) {
		std::abort();
	}
	ConsoleHandle = console;
}

void LogImpl(LogLevel level, std::string_view message) {
	if (ConsoleHandle == nullptr) {
		return;
	}
	const LogLevelInfo &levelInfo = GetLogLevelInfo(level);
	std::string entry;
	entry.append(levelInfo.color);
	entry.append(levelInfo.name);
	if (!levelInfo.color.empty()) {
		entry.append("\x1b[0m");
	}
	entry.append(": ");
	entry.append(message);
	entry.push_back('\n');
	if (entry.size() > std::numeric_limits<DWORD>::max()) {
		std::abort();
	}
	DWORD count = static_cast<DWORD>(entry.size());
	DWORD written;
	WriteFile(ConsoleHandle, entry.data(), count, &written, nullptr);
}

} // namespace demo
