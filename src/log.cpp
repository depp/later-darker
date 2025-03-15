// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log.hpp"

#include "log_internal.hpp"
#include "text_buffer.hpp"

#include <string_view>

namespace demo {
namespace log {

const Location Location::Zero{};

namespace {

bool HasLog;

// Format for quoting a string.
enum class Context {
	Inline, // String appears inline, with other content.
	Line,   // String appears on its own line.
};

struct LevelInfo {
	std::string_view color;
	std::string_view name;
	std::string_view emoji;
};

// These names all have the same width so log messages line up.
const LevelInfo Levels[] = {
	{"\x1b[36m", "DEBUG", "📘"},
	{"", "INFO ", "📄"},
	{"\x1b[33m", "WARN ", "⚠️"},
	{"\x1b[31m", "ERROR", "🛑"},
};

const LevelInfo &GetLevelInfo(Level level) {
	return Levels[static_cast<int>(level)];
}

// Return true if the string should be quoted when logged.
template <typename Char>
bool DoesNeedQuotes(std::basic_string_view<Char> str, Context context) {
	if (str.empty()) {
		return true;
	}
	Char minCh = 33;
	if (context == Context::Line) {
		if (str[0] == ' ' || *(str.end() - 1) == ' ') {
			return true;
		}
		minCh = 32;
	}
	for (const auto ch : str) {
		if (ch < minCh || 126 < ch || ch == '"' || ch == '\\') {
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
		std::string s;
		s.append(str);
	} break;
	case Kind::WideString: {
		std::wstring_view wideStr = value.WideStringValue();
		if (DoesNeedQuotes(wideStr, context)) {
			out.AppendWideQuoted(wideStr);
		} else {
			out.AppendWide(wideStr);
		}
	} break;
	}
}

} // namespace

void WriteLine(TextBuffer &buffer, const Record &record, bool useColor,
               bool useEmoji) {
	const LevelInfo &levelInfo = GetLevelInfo(record.level());
	if (useEmoji) {
		buffer.Append(levelInfo.emoji);
		buffer.AppendChar(' ');
	}
	if (useColor && !levelInfo.color.empty()) {
		buffer.Append(levelInfo.color);
	}
	buffer.Append(levelInfo.name);
	if (useColor && !levelInfo.color.empty()) {
		buffer.Append("\x1b[0m");
	}
	buffer.AppendChar(' ');
	if (!record.location().is_empty()) {
		AppendLocation(buffer, record.location());
		buffer.Append(": ");
	}
	buffer.Append(record.message());
	for (const Attr &attr : record.attributes()) {
		buffer.AppendChar(' ');
		buffer.Append(attr.name());
		buffer.AppendChar('=');
		AppendValue(buffer, attr.value(), Context::Inline);
	}
	buffer.AppendChar('\n');
}

void WriteBlock(TextBuffer &buffer, const Record &record) {
	buffer.Append(record.message());
	buffer.AppendChar('\n');
	for (const Attr &attr : record.attributes()) {
		buffer.AppendChar('\n');
		buffer.Append(attr.name());
		buffer.Append(": ");
		AppendValue(buffer, attr.value(), Context::Line);
	}
	if (!record.location().is_empty()) {
		buffer.Append("\nlocation: ");
		AppendLocation(buffer, record.location());
	}
}

void Init() {
	HasLog = Writer::Init();
}

void Record::Log() const {
	if (!HasLog) {
		return;
	}

	Writer writer;
	writer.Log(*this);
}

[[noreturn]]
void Record::Fail() const {
	Writer writer;
	writer.Fail(*this);
}

} // namespace log
} // namespace demo
