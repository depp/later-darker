// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0
#pragma once

#include "os_string.hpp"

namespace demo {

namespace var {

// If true, create a debug OpenGL context.
extern bool DebugContext;

// If true, allocate a console (Windows).
extern bool AllocConsole;

} // namespace var

// Parse the program's command-line arguments.
void ParseCommandArguments(int argCount, os_char **args);

} // namespace demo
