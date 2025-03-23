// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include "text_buffer.hpp"

#if _WIN32
#include "wide_text_buffer.hpp"
#endif

#include <cstddef>

namespace demo {
namespace log {

class Record;

// Local buffer size for constructing log messages.
constexpr std::size_t LogBufferSize = 256;

// Write a record as a single line.
void WriteLine(TextBuffer &buffer, const Record &record, bool useColor,
               bool useEmoji);

// Write a record as a multi-line block.
void WriteBlock(TextBuffer &buffer, const Record &record);

#if _WIN32

// Sink for writing log messages on Windows.
class WindowsWriter {
public:
	// Initialize the log destination. Return true if logging is available.
	static bool Init();

	WindowsWriter() : mBuffer{mBufferData}, mWideBuffer{mWideBufferData} {}

	// Write a record to the log.
	void Log(const Record &record);

	// Fail the program with a given error message.
	[[noreturn]]
	void Fail(const Record &record);

private:
	TextBuffer mBuffer;
	char mBufferData[LogBufferSize];
	WideTextBuffer mWideBuffer;
	wchar_t mWideBufferData[LogBufferSize];
};

using Writer = WindowsWriter;

#else

// Sink for writing log messages on Unix-based systems.
class UnixWriter {
public:
	// Initialize the log destination. Return true if logging is available.
	static bool Init();

	UnixWriter() : mBuffer{mBufferData} {}

	// Write a record to the log.
	void Log(const Record &record);

	// Fail the program with a given error message.
	[[noreturn]]
	void Fail(const Record &record);

private:
	TextBuffer mBuffer;
	char mBufferData[LogBufferSize];
};

using Writer = UnixWriter;

#endif

} // namespace log
} // namespace demo
