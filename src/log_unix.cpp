// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log_internal.hpp"

#include "log.hpp"
#include "main.hpp"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <unistd.h>

namespace demo {
namespace log {

namespace {

// Return true if the output should be colorized using terminal escape
// sequences.
bool ShouldEnableColor() {
	// If $NO_COLOR is non-empty, no color.
	const char *noColor = std::getenv("NO_COLOR");
	if (noColor != nullptr && *noColor != '\0') {
		return false;
	}

	// If stderr is not a tty, no color.
	int r = isatty(STDERR_FILENO);
	if (r == 0) {
		return false;
	}

	// Check $TERM.
	const char *term = std::getenv("TERM");
	if (term == nullptr) {
		return false;
	}
	// TERM=dumb used by Xcode.
	if (std::strcmp(term, "dumb") == 0) {
		return false;
	}
	return true;
}

bool IsColorEnabled;

} // namespace

bool UnixWriter::Init() {
	IsColorEnabled = ShouldEnableColor();
	return true;
}

void UnixWriter::Log(const Record &record) {
	mBuffer.Clear();
	WriteLine(mBuffer, record, IsColorEnabled, true);
	// Ignore errors, throw this into the void.
	(void)::write(STDERR_FILENO, mBuffer.Start(), mBuffer.Size());
}

[[noreturn]]
void UnixWriter::Fail(const Record &record) {
	mBuffer.Clear();
	WriteLine(mBuffer, record, IsColorEnabled, true);
	if (IsColorEnabled) {
		mBuffer.Append("\x1b[31m");
	}
	mBuffer.Append("===== Fatal Error =====");
	if (IsColorEnabled) {
		mBuffer.Append("\x1b[0m");
	}
	mBuffer.AppendChar('\n');
	(void)::write(STDERR_FILENO, mBuffer.Start(), mBuffer.Size());
	ExitError();
}

} // namespace log
} // namespace demo
