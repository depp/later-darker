// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log.hpp"

#include "var.hpp"

#include <limits>
#include <string>

// Note:
// https://learn.microsoft.com/en-us/windows/console/console-virtual-terminal-sequences

#define NOMINMAX 1
#define UNICODE 1
#define WIN32_LEAN_AND_MEAN 1
#include <Windows.h>

namespace demo {
namespace log {

namespace {

HANDLE ConsoleHandle;

struct LevelInfo {
	std::wstring_view color;
	std::wstring_view name;
};

const LevelInfo Levels[] = {
	{L"\x1b[36m", L"DEBUG"},
	{L"", L"INFO"},
	{L"\x1b[33m", L"WARN"},
	{L"\x1b[31m", L"ERROR"},
};

const LevelInfo &GetLevelInfo(Level level) {
	int index = static_cast<int>(level);
	if (index < 0 || std::size(Levels) <= index) {
		std::abort();
	}
	return Levels[index];
}

} // namespace

void Init() {
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
		std::abort();
	}
	ok = SetConsoleMode(
		console, ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
	if (!ok) {
		std::abort();
	}
	ConsoleHandle = console;
}

void Log(Level level, std::string_view file, int line,
         std::string_view function, std::string_view message) {
	if (ConsoleHandle == nullptr) {
		return;
	}
	const LevelInfo &levelInfo = GetLevelInfo(level);
	std::wstring entry;
	entry.append(levelInfo.color);
	entry.append(levelInfo.name);
	if (!levelInfo.color.empty()) {
		entry.append(L"\x1b[0m");
	}
	entry.push_back(L' ');
	Append(&entry, file);
	entry.push_back(L':');
	Append(&entry, std::to_string(line));
	entry.append(L" (");
	Append(&entry, function);
	entry.append(L"): ");
	Append(&entry, message);
	entry.push_back(L'\n');
	if (entry.size() > std::numeric_limits<DWORD>::max()) {
		std::abort();
	}
	DWORD count = static_cast<DWORD>(entry.size());
	DWORD written;
	WriteConsoleW(ConsoleHandle, entry.data(), count, &written, nullptr);
}

} // namespace log
} // namespace demo
