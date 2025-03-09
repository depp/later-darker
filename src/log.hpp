// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include <string_view>

namespace demo {

// Log message severity level.
enum class LogLevel {
	Debug,
	Info,
	Warn,
	Error,
};

// Initialize the logging system.
void LogInit();

// Write a message to the log.
void LogImpl(LogLevel level, std::string_view message);

} // namespace demo
