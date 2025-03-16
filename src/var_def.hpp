// Copyright 2025 Dietrich Epp <depp@zdome.net>
// Licensed under the Mozilla Public License Version 2.0.
// SPDX-License-Identifier: MPL-2.0

// Variable definitions. Don't include this file directly. This must be included
// from a file that defines the following macros:
//
// DEFVAR(name, type, description)

DEFVAR(DebugContext, bool, "If true, create a debug OpenGL context.")
DEFVAR(AllocConsole, bool, "If true, allocate a console (Windows).")
DEFVAR(ProjectPath, os_string, "Path to the directory containing this project.")
