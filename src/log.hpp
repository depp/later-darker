// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include <string_view>

namespace demo {
namespace log {

// Log message severity level.
enum class Level {
	Debug,
	Info,
	Warn,
	Error,
};

// Initialize the logging system.
void Init();

// Write a message to the log.
void Log(Level level, std::string_view message);

} // namespace log
} // namespace demo

#define LOG(level, message) ::demo::log::Log(::demo::log::Level::level, message)
