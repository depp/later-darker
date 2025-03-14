// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log_internal.hpp"

#include "log.hpp"
#include "main.hpp"

#include <cstdlib>
#include <unistd.h>

namespace demo {
namespace log {

bool UnixWriter::Init() {
	return true;
}

void UnixWriter::Log(const Record &record) {
	mBuffer.Clear();
	WriteLine(mBuffer, record);
	// Ignore errors, throw this into the void.
	(void)::write(STDERR_FILENO, mBuffer.Start(), mBuffer.Size());
}

[[noreturn]]
void UnixWriter::Fail(const Record &record) {
	mBuffer.Clear();
	WriteLine(mBuffer, record);
	mBuffer.Append("\x1b[31m===== Fatal Error =====\x1b[0m\n");
	(void)::write(STDERR_FILENO, mBuffer.Start(), mBuffer.Size());
	ExitError();
}

} // namespace log
} // namespace demo
