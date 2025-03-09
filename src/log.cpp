// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log.hpp"

#include "var.hpp"

#include <charconv>
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

void AppendFileName(os_string *dest, std::string_view file) {
	// NOTE: We rely on this file being named ${prefix}src/log.cpp so we can
	// figure out what the prefix is for other files.
	constexpr std::string_view thisFile = __FILE__;
	constexpr std::string_view prefix =
		thisFile.substr(0, thisFile.size() - 11);
	if (file.size() < prefix.size() ||
	    file.substr(0, prefix.size()) != prefix) {
		Append(dest, file);
		return;
	}
	std::string_view relativeFile = file.substr(prefix.size());
	for (char c : relativeFile) {
		if (c == '\\') {
			c = '/';
		}
		dest->push_back(c);
	}
}

template <typename Func>
void AppendToChars(os_string *dest, Func toChars) {
	std::string buffer;
	while (true) {
		buffer.resize(buffer.capacity(), '\0');
		std::to_chars_result result =
			toChars(buffer.data(), buffer.data() + buffer.size());
		if (result.ec == std::errc{}) {
			Append(dest,
			       std::string_view(buffer.data(), result.ptr - buffer.data()));
			return;
		}
		buffer.push_back('\0');
	}
};

constexpr std::wstring_view BoolTrue = L"true";
constexpr std::wstring_view BoolFalse = L"false";

std::wstring_view ToString(bool value) {
	return value ? BoolTrue : BoolFalse;
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
	Log(level, file, line, function, message, {});
}

void Log(Level level, std::string_view file, int line,
         std::string_view function, std::string_view message,
         std::initializer_list<Attr> attributes) {
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
	AppendFileName(&entry, file);
	entry.push_back(L':');
	Append(&entry, std::to_string(line));
	entry.append(L" (");
	Append(&entry, function);
	entry.append(L"): ");
	Append(&entry, message);
	for (const Attr &attr : attributes) {
		entry.push_back(L' ');
		Append(&entry, attr.name());
		entry.push_back(L'=');
		const Value &value = attr.value();
		switch (value.ValueKind()) {
		case Kind::Null:
			entry.append(L"(null)");
			break;
		case Kind::Int: {
			long long x{value.IntValue()};
			AppendToChars(&entry,
			              [x](char *first, char *last) -> std::to_chars_result {
							  return std::to_chars(first, last, x);
						  });
		} break;
		case Kind::Uint: {
			unsigned long long x{value.UintValue()};
			AppendToChars(&entry,
			              [x](char *first, char *last) -> std::to_chars_result {
							  return std::to_chars(first, last, x);
						  });
		} break;
		case Kind::Float: {
			double x{value.FloatValue()};
			AppendToChars(&entry,
			              [x](char *first, char *last) -> std::to_chars_result {
							  return std::to_chars(first, last, x);
						  });
		} break;
		case Kind::Bool:
			entry.append(ToString(value.BoolValue()));
			break;
		case Kind::String: {
			std::string_view str = value.StringValue();
			Append(&entry, str); // FIXME: quoting!
		} break;
		}
	}
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
