// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log.hpp"

#include "text_buffer.hpp"
#include "var.hpp"
#include "wide_text_buffer.hpp"

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

constexpr std::size_t LogBufferSize = 256;

HANDLE ConsoleHandle;

struct LevelInfo {
	std::string_view color;
	std::string_view name;
};

const LevelInfo Levels[] = {
	{"\x1b[36m", "DEBUG"},
	{"", "INFO"},
	{"\x1b[33m", "WARN"},
	{"\x1b[31m", "ERROR"},
};

const LevelInfo &GetLevelInfo(Level level) {
	int index = static_cast<int>(level);
	if (index < 0 || std::size(Levels) <= index) {
		std::abort();
	}
	return Levels[index];
}

void AppendFileName(TextBuffer &out, std::string_view file) {
	// NOTE: We rely on this file being named ${prefix}src/log.cpp so we can
	// figure out what the prefix is for other files.
	constexpr std::string_view thisFile = __FILE__;
	constexpr std::string_view prefix =
		thisFile.substr(0, thisFile.size() - 11);
	if (file.size() < prefix.size() ||
	    file.substr(0, prefix.size()) != prefix) {
		out.Append(file);
		return;
	}
	std::string_view relativeFile = file.substr(prefix.size());
	for (char c : relativeFile) {
		if (c == '\\') {
			c = '/';
		}
		std::basic_string<char> a;
		out.AppendChar(c);
	}
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

void Log(Level level, std::source_location location, std::string_view message) {
	Log(level, location, message, {});
}

void Log(Level level, std::source_location location, std::string_view message,
         std::initializer_list<Attr> attributes) {
	if (ConsoleHandle == nullptr) {
		return;
	}

	// Create the log message in UTF-8.
	const LevelInfo &levelInfo = GetLevelInfo(level);
	char bufferData[LogBufferSize];
	TextBuffer buffer(bufferData);
	buffer.Append(levelInfo.color);
	buffer.Append(levelInfo.name);
	if (!levelInfo.color.empty()) {
		buffer.Append("\x1b[0m");
	}
	buffer.AppendChar(' ');
	AppendFileName(buffer, location.file_name());
	buffer.AppendChar(':');
	buffer.AppendNumber(location.line());
	// This is contains the full function signature, which is noisy. At least,
	// on MSVC.
	// buffer.Append(" (");
	// buffer.Append(location.function_name());
	// buffer.Append("): ");
	buffer.Append(": ");
	buffer.Append(message);
	for (const Attr &attr : attributes) {
		buffer.AppendChar(' ');
		buffer.Append(attr.name());
		buffer.AppendChar('=');
		const Value &value = attr.value();
		switch (value.ValueKind()) {
		case Kind::Null:
			buffer.Append("(null)");
			break;
		case Kind::Int:
			buffer.AppendNumber(value.IntValue());
			break;
		case Kind::Uint:
			buffer.AppendNumber(value.UintValue());
			break;
		case Kind::Float:
			buffer.AppendNumber(value.FloatValue());
			break;
		case Kind::Bool:
			buffer.AppendBool(value.BoolValue());
			break;
		case Kind::String: {
			std::string_view str = value.StringValue();
			buffer.Append(str);
		} break;
		}
	}
	buffer.AppendChar('\n');

	// Convert to wchar.
	wchar_t wideBufferData[LogBufferSize];
	WideTextBuffer wideBuffer(wideBufferData);
	wideBuffer.AppendMultiByte(buffer.Contents());
	// FIXME: overflow check?
	DWORD size = static_cast<DWORD>(wideBuffer.Size());
	DWORD written;
	WriteConsoleW(ConsoleHandle, wideBuffer.Start(), size, &written, nullptr);
}

} // namespace log
} // namespace demo
