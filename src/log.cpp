// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log.hpp"

#include "text_buffer.hpp"
#include "var.hpp"
#include "wide_text_buffer.hpp"

#include <array>
#include <charconv>
#include <limits>
#include <span>
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

// These names all have the same width so log messages line up.
const LevelInfo Levels[] = {
	{"\x1b[36m", "DEBUG"},
	{"", "INFO "},
	{"\x1b[33m", "WARN "},
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

void AppendLocation(TextBuffer &out, const Location &location) {
	AppendFileName(out, location.file);
	out.AppendChar(':');
	out.AppendNumber(location.line);
	out.Append(" (");
	out.Append(location.function);
	out.AppendChar(')');
}

struct LogBuffer {
	TextBuffer buffer{bufferData};
	WideTextBuffer wideBuffer{wideBufferData};
	char bufferData[LogBufferSize];
	wchar_t wideBufferData[LogBufferSize];

	LogBuffer() : buffer{bufferData}, wideBuffer{wideBufferData} {}

	// Format a log message and write it to the console.
	void Log(Level level, const Location &location, std::string_view message,
	         std::span<const Attr> attributes);
};

void LogBuffer::Log(Level level, const Location &location,
                    std::string_view message,
                    std::span<const Attr> attributes) {
	// Create the log message in UTF-8.
	const LevelInfo &levelInfo = GetLevelInfo(level);
	buffer.Clear();
	buffer.Append(levelInfo.color);
	buffer.Append(levelInfo.name);
	if (!levelInfo.color.empty()) {
		buffer.Append("\x1b[0m");
	}
	buffer.AppendChar(' ');
	AppendLocation(buffer, location);
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

	// Convert to wide characters.
	wideBuffer.AppendMultiByte(buffer.Contents());
	// FIXME: overflow check?
	DWORD size = static_cast<DWORD>(wideBuffer.Size());
	DWORD written;
	WriteConsoleW(ConsoleHandle, wideBuffer.Start(), size, &written, nullptr);
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

void Log(Level level, const Location &location, std::string_view message) {
	Log(level, location, message, {});
}

void Log(Level level, const Location &location, std::string_view message,
         std::initializer_list<Attr> attributes) {
	if (ConsoleHandle == nullptr) {
		return;
	}

	LogBuffer buffer;
	buffer.Log(level, location, message, attributes);
}

[[noreturn]]
void CheckFail(const Location &location, std::string_view condition) {
	LogBuffer buffer;
	buffer.Log(Level::Error, location, "Check failed.",
	           std::initializer_list<Attr>{{"condition", condition}});

	buffer.buffer.Clear();
	buffer.buffer.Append("Check failed.\nCondition: ");
	buffer.buffer.Append(condition);
	buffer.buffer.Append("\nLocation: ");
	AppendLocation(buffer.buffer, location);
	buffer.buffer.AppendChar('\0');
	buffer.wideBuffer.Clear();
	buffer.wideBuffer.AppendMultiByte(buffer.buffer.Contents());

	MessageBoxW(nullptr, buffer.wideBuffer.Start(), nullptr, MB_ICONSTOP);
	ExitProcess(1);
}

} // namespace log
} // namespace demo
