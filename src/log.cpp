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

// Format for quoting a string.
enum class Context {
	Inline, // String appears inline, with other content.
	Line,   // String appears on its own line.
};

constexpr std::size_t LogBufferSize = 256;

HANDLE ConsoleHandle;

bool LogAvailable() {
	return ConsoleHandle != nullptr;
}

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
	return Levels[static_cast<int>(level)];
}

// Return true if the string should be quoted when logged.
bool DoesNeedQuotes(std::string_view str, Context context) {
	if (str.empty()) {
		return true;
	}
	unsigned minCh;
	switch (context) {
	case Context::Inline:
		minCh = 33;
		break;
	case Context::Line:
		minCh = 32;
		if (str[0] == ' ' || *(str.end() - 1) == ' ') {
			return true;
		}
		break;
	}
	for (auto p = str.begin(), e = str.end(); p != e; ++p) {
		const unsigned ch = static_cast<unsigned char>(*p);
		if (ch < minCh || 127 < ch || ch == '"' || ch == '\\') {
			return true;
		}
	}
	return false;
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

void AppendValue(TextBuffer &out, const Value &value, Context context) {
	switch (value.ValueKind()) {
	case Kind::Null:
		out.Append("(null)");
		break;
	case Kind::Int:
		out.AppendNumber(value.IntValue());
		break;
	case Kind::Uint:
		out.AppendNumber(value.UintValue());
		break;
	case Kind::Float:
		out.AppendNumber(value.FloatValue());
		break;
	case Kind::Bool:
		out.AppendBool(value.BoolValue());
		break;
	case Kind::String: {
		std::string_view str = value.StringValue();
		if (DoesNeedQuotes(str, context)) {
			out.AppendQuoted(str);
		} else {
			out.Append(str);
		}
	} break;
	}
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

	// Format a message as a multi-line block. Used for dialogs.
	void LogBlock(const Location &location, std::string_view message,
	              std::span<const Attr> attributes);
};

void LogBuffer::Log(Level level, const Location &location,
                    std::string_view message,
                    std::span<const Attr> attributes) {
	if (!LogAvailable()) {
		return;
	}

	// Create the log message in UTF-8.
	const LevelInfo &levelInfo = GetLevelInfo(level);
	buffer.Clear();
	buffer.Append(levelInfo.color);
	buffer.Append(levelInfo.name);
	if (!levelInfo.color.empty()) {
		buffer.Append("\x1b[0m");
	}
	buffer.AppendChar(' ');
	if (!location.IsEmpty()) {
		AppendLocation(buffer, location);
		buffer.Append(": ");
	}
	buffer.Append(message);
	for (const Attr &attr : attributes) {
		buffer.AppendChar(' ');
		buffer.Append(attr.name());
		buffer.AppendChar('=');
		AppendValue(buffer, attr.value(), Context::Inline);
	}
	buffer.AppendChar('\n');

	// Convert to wide characters.
	wideBuffer.Clear();
	wideBuffer.AppendMultiByte(buffer.Contents());
	// FIXME: overflow check?
	DWORD size = static_cast<DWORD>(wideBuffer.Size());
	DWORD written;
	WriteConsoleW(ConsoleHandle, wideBuffer.Start(), size, &written, nullptr);
}

void LogBuffer::LogBlock(const Location &location, std::string_view message,
                         std::span<const Attr> attributes) {
	buffer.Clear();
	buffer.Append(message);
	buffer.AppendChar('\n');
	for (const Attr &attr : attributes) {
		buffer.AppendChar('\n');
		buffer.Append(attr.name());
		buffer.Append(": ");
		AppendValue(buffer, attr.value(), Context::Line);
	}
	if (!location.IsEmpty()) {
		buffer.Append("\nlocation: ");
		AppendLocation(buffer, location);
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

void Log(Level level, const Location &location, std::string_view message) {
	Log(level, location, message, {});
}

void Log(Level level, const Location &location, std::string_view message,
         std::initializer_list<Attr> attributes) {
	if (!LogAvailable()) {
		return;
	}

	LogBuffer buffer;
	buffer.Log(level, location, message, attributes);
}

[[noreturn]]
void Fail(const Location &location, std::string_view message,
          std::span<const Attr> attributes) {
	LogBuffer buffer;

	buffer.Log(Level::Error, location, message, attributes);

	buffer.LogBlock(location, message, attributes);
	buffer.buffer.AppendChar('\0');
	buffer.wideBuffer.Clear();
	buffer.wideBuffer.AppendMultiByte(buffer.buffer.Contents());
	MessageBoxW(nullptr, buffer.wideBuffer.Start(), nullptr, MB_ICONSTOP);
	ExitProcess(1);
}

[[noreturn]]
void CheckFail(const Location &location, std::string_view condition) {
	Fail(location, "Check failed.",
	     std::array<Attr, 1>({{"condition", condition}}));
}

} // namespace log
} // namespace demo
