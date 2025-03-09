// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#include "log.hpp"

#include <cstdio>
#include <string>

namespace demo {

namespace {

const std::string_view LogLevels[] = {
	"DEBUG",
	"INFO",
	"WARN",
	"ERROR",
};

std::string_view LogLevelName(LogLevel level) {
	int index = static_cast<int>(level);
	if (index < 0 || std::size(LogLevels) <= index) {
		std::abort();
	}
	return LogLevels[index];
}

} // namespace

void LogImpl(LogLevel level, std::string_view message) {
	std::string entry;
	entry.append(LogLevelName(level));
	entry.append(": ");
	entry.append(message);
	entry.push_back('\n');
	std::fwrite(entry.data(), 1, entry.size(), stderr);
}

} // namespace demo
